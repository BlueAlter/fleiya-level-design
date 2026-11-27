//! Live LDtk-faithful entity preview widget inside the Entity Workspace.

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Sense, Stroke,
    StrokeKind, TextureHandle, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, EntityRenderMode, TileRenderMode, TilesetDef, TilesetRect};
use lc_ui_api::AssetService;

use crate::features::editor::shell_app::ShellApp;

/// Resolves the effective [`TilesetRect`] for an [`EntityDef`] (preferring `tile_rect`,
/// falling back to legacy single-tile `tile`).
pub fn resolve_entity_tile_rect(def: &EntityDef, tilesets: &[TilesetDef]) -> Option<TilesetRect> {
    if let Some(rect) = def.tile_rect {
        return Some(rect);
    }
    let tile_ref = def.tile?;
    let ts = tilesets.iter().find(|t| t.id == tile_ref.tileset_id)?;
    let tile_w = ts.tile_size.x.max(1);
    let tile_h = ts.tile_size.y.max(1);
    // Fallback assuming standard grid if image dimensions aren't available here.
    let _ = tile_ref.tile_id;
    Some(TilesetRect::new(
        ts.id, ts.padding, ts.padding, tile_w, tile_h,
    ))
}

/// Renders a live preview of `def` with checkerboard backdrop, render mode, tile scaling,
/// 9-slice guide lines, pivot crosshair, and optional name label.
pub fn render_entity_live_preview(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    def: &EntityDef,
    height: f32,
) {
    let avail_w = ui.available_width().max(120.0);
    let (rect, _) = ui.allocate_exact_size(vec2(avail_w, height), Sense::hover());

    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::ZERO, t.canvas);
    draw_checkerboard(&painter, rect, 12.0, t);
    painter.rect_stroke(rect, CornerRadius::ZERO, t.hairline(), StrokeKind::Inside);

    let pad = 28.0;
    let max_w = (rect.width() - pad * 2.0).max(16.0);
    let max_h = (rect.height() - pad * 2.0).max(16.0);
    let ent_w = def.size_px.x.max(1) as f32;
    let ent_h = def.size_px.y.max(1) as f32;
    let scale = (max_w / ent_w).min(max_h / ent_h).clamp(0.5, 4.0);

    let draw_size = vec2(ent_w * scale, ent_h * scale);
    let ent_rect = Rect::from_center_size(rect.center(), draw_size);

    let base_color =
        Color32::from_rgba_unmultiplied(def.color[0], def.color[1], def.color[2], def.color[3]);
    let fill_alpha = if def.hollow {
        0.08
    } else {
        def.fill_opacity.clamp(0.0, 1.0) * 0.45
    };
    let line_alpha = def.line_opacity.clamp(0.0, 1.0);
    let fill_color = base_color.gamma_multiply(fill_alpha);
    let stroke = Stroke::new(1.5, base_color.gamma_multiply(line_alpha));

    match def.render_mode {
        EntityRenderMode::Rectangle => {
            painter.rect_filled(ent_rect, CornerRadius::ZERO, fill_color);
            painter.rect_stroke(ent_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);
        }
        EntityRenderMode::Ellipse => {
            paint_ellipse(&painter, ent_rect, fill_color, stroke);
        }
        EntityRenderMode::Cross => {
            painter.rect_filled(
                ent_rect,
                CornerRadius::ZERO,
                fill_color.gamma_multiply(0.35),
            );
            painter.rect_stroke(
                ent_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, base_color.gamma_multiply(line_alpha * 0.6)),
                StrokeKind::Inside,
            );
            painter.line_segment([ent_rect.left_top(), ent_rect.right_bottom()], stroke);
            painter.line_segment([ent_rect.right_top(), ent_rect.left_bottom()], stroke);
        }
        EntityRenderMode::Tile => {
            painter.rect_filled(
                ent_rect,
                CornerRadius::ZERO,
                fill_color.gamma_multiply(0.25),
            );
            painter.rect_stroke(
                ent_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, base_color.gamma_multiply(line_alpha * 0.5)),
                StrokeKind::Inside,
            );
        }
    }

    // Draw tile texture if tile_rect (or legacy tile) is configured
    let tile_rect = def.tile_rect.or_else(|| {
        def.tile.and_then(|tr| {
            let ts = app.project.defs.tileset(tr.tileset_id)?;
            Some(TilesetRect::new(
                ts.id,
                ts.padding,
                ts.padding,
                ts.tile_size.x.max(1),
                ts.tile_size.y.max(1),
            ))
        })
    });

    if let Some(tr) = tile_rect {
        if let Some(ts) = app.project.defs.tileset(tr.tileset_id) {
            if let Ok(tex) = app.assets.load_texture(ui.ctx(), &ts.path) {
                paint_entity_tile(
                    &painter,
                    &tex,
                    tr,
                    ent_rect,
                    def.tile_render_mode,
                    def.nine_slice_borders,
                    def.tile_opacity.clamp(0.0, 1.0),
                    scale,
                );
            }
        }
    }

    // Draw 9-slice border guides when NineSlice mode is active
    if def.tile_render_mode == TileRenderMode::NineSlice {
        let [top, right, bottom, left] = def.nine_slice_borders;
        let guide = Stroke::new(1.0, t.accent.gamma_multiply(0.75));
        if top > 0 {
            let y = ent_rect.min.y + top as f32 * scale;
            painter.hline(ent_rect.x_range(), y, guide);
        }
        if bottom > 0 {
            let y = ent_rect.max.y - bottom as f32 * scale;
            painter.hline(ent_rect.x_range(), y, guide);
        }
        if left > 0 {
            let x = ent_rect.min.x + left as f32 * scale;
            painter.vline(x, ent_rect.y_range(), guide);
        }
        if right > 0 {
            let x = ent_rect.max.x - right as f32 * scale;
            painter.vline(x, ent_rect.y_range(), guide);
        }
    }

    // Pivot marker crosshair
    let pivot_pos = pos2(
        ent_rect.min.x + ent_rect.width() * def.pivot.x.clamp(0.0, 1.0),
        ent_rect.min.y + ent_rect.height() * def.pivot.y.clamp(0.0, 1.0),
    );
    painter.circle_filled(pivot_pos, 4.0, Color32::WHITE);
    painter.circle_stroke(pivot_pos, 4.0, Stroke::new(1.2, Color32::BLACK));

    // Name label
    if def.show_name && !def.name.is_empty() {
        painter.text(
            pos2(ent_rect.center().x, ent_rect.min.y - 4.0),
            Align2::CENTER_BOTTOM,
            &def.name,
            FontId::proportional(11.5),
            t.ink_strong,
        );
    }
}

/// Paints an entity's tile texture inside `dest_rect` according to [`TileRenderMode`].
#[allow(clippy::too_many_arguments)]
pub fn paint_entity_tile(
    painter: &Painter,
    tex: &TextureHandle,
    src_px: TilesetRect,
    dest_rect: Rect,
    mode: TileRenderMode,
    nine_slice: [u32; 4],
    opacity: f32,
    zoom: f32,
) {
    let tex_sz = tex.size_vec2();
    if tex_sz.x <= 0.0 || tex_sz.y <= 0.0 || src_px.w == 0 || src_px.h == 0 {
        return;
    }

    let u0 = src_px.x as f32 / tex_sz.x;
    let v0 = src_px.y as f32 / tex_sz.y;
    let u1 = (src_px.x + src_px.w) as f32 / tex_sz.x;
    let v1 = (src_px.y + src_px.h) as f32 / tex_sz.y;
    let full_uv = Rect::from_min_max(pos2(u0, v0), pos2(u1, v1));
    let tint = Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0).round() as u8);

    match mode {
        TileRenderMode::Stretch => {
            painter.image(tex.id(), dest_rect, full_uv, tint);
        }
        TileRenderMode::FitInside => {
            let aspect = src_px.w as f32 / src_px.h as f32;
            let dest_aspect = dest_rect.width() / dest_rect.height().max(1.0);
            let fitted = if dest_aspect > aspect {
                let w = dest_rect.height() * aspect;
                Rect::from_center_size(dest_rect.center(), vec2(w, dest_rect.height()))
            } else {
                let h = dest_rect.width() / aspect.max(0.001);
                Rect::from_center_size(dest_rect.center(), vec2(dest_rect.width(), h))
            };
            painter.image(tex.id(), fitted, full_uv, tint);
        }
        TileRenderMode::Cover => {
            let clipped = painter.with_clip_rect(dest_rect);
            let aspect = src_px.w as f32 / src_px.h as f32;
            let dest_aspect = dest_rect.width() / dest_rect.height().max(1.0);
            let covered = if dest_aspect > aspect {
                let h = dest_rect.width() / aspect.max(0.001);
                Rect::from_center_size(dest_rect.center(), vec2(dest_rect.width(), h))
            } else {
                let w = dest_rect.height() * aspect;
                Rect::from_center_size(dest_rect.center(), vec2(w, dest_rect.height()))
            };
            clipped.image(tex.id(), covered, full_uv, tint);
        }
        TileRenderMode::FullSizeUncropped => {
            let native = Rect::from_center_size(
                dest_rect.center(),
                vec2(src_px.w as f32 * zoom, src_px.h as f32 * zoom),
            );
            painter.image(tex.id(), native, full_uv, tint);
        }
        TileRenderMode::FullSizeCropped => {
            let clipped = painter.with_clip_rect(dest_rect);
            let native = Rect::from_min_size(
                dest_rect.min,
                vec2(src_px.w as f32 * zoom, src_px.h as f32 * zoom),
            );
            clipped.image(tex.id(), native, full_uv, tint);
        }
        TileRenderMode::Repeat => {
            let clipped = painter.with_clip_rect(dest_rect);
            let step_x = (src_px.w as f32 * zoom).max(4.0);
            let step_y = (src_px.h as f32 * zoom).max(4.0);
            let cols = (dest_rect.width() / step_x).ceil() as usize;
            let rows = (dest_rect.height() / step_y).ceil() as usize;
            for r in 0..rows.min(64) {
                for c in 0..cols.min(64) {
                    let tile_dest = Rect::from_min_size(
                        pos2(
                            dest_rect.min.x + c as f32 * step_x,
                            dest_rect.min.y + r as f32 * step_y,
                        ),
                        vec2(step_x, step_y),
                    );
                    clipped.image(tex.id(), tile_dest, full_uv, tint);
                }
            }
        }
        TileRenderMode::NineSlice => {
            let [top_px, right_px, bottom_px, left_px] = nine_slice;
            let sw = src_px.w as f32;
            let sh = src_px.h as f32;
            let l = (left_px as f32).min(sw * 0.49);
            let r = (right_px as f32).min(sw * 0.49);
            let t = (top_px as f32).min(sh * 0.49);
            let b = (bottom_px as f32).min(sh * 0.49);

            let dl = (l * zoom).min(dest_rect.width() * 0.49);
            let dr = (r * zoom).min(dest_rect.width() * 0.49);
            let dt = (t * zoom).min(dest_rect.height() * 0.49);
            let db = (b * zoom).min(dest_rect.height() * 0.49);

            let xs = [
                dest_rect.min.x,
                dest_rect.min.x + dl,
                dest_rect.max.x - dr,
                dest_rect.max.x,
            ];
            let ys = [
                dest_rect.min.y,
                dest_rect.min.y + dt,
                dest_rect.max.y - db,
                dest_rect.max.y,
            ];

            let us = [u0, u0 + l / tex_sz.x, u1 - r / tex_sz.x, u1];
            let vs = [v0, v0 + t / tex_sz.y, v1 - b / tex_sz.y, v1];

            for row in 0..3 {
                for col in 0..3 {
                    let sub_dest =
                        Rect::from_min_max(pos2(xs[col], ys[row]), pos2(xs[col + 1], ys[row + 1]));
                    if sub_dest.width() <= 0.0 || sub_dest.height() <= 0.0 {
                        continue;
                    }
                    let sub_uv =
                        Rect::from_min_max(pos2(us[col], vs[row]), pos2(us[col + 1], vs[row + 1]));
                    painter.image(tex.id(), sub_dest, sub_uv, tint);
                }
            }
        }
    }
}

fn paint_ellipse(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    const SEGMENTS: usize = 32;
    let pts: Vec<Pos2> = (0..SEGMENTS)
        .map(|i| {
            let theta = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            pos2(center.x + rx * theta.cos(), center.y + ry * theta.sin())
        })
        .collect();
    painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
    painter.add(egui::Shape::closed_line(pts, stroke));
}
