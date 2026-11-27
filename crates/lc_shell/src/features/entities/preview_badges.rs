//! Stacked badge/swatch/mini-tile overlays and tile/ellipse painters for Entity Live Preview.

use egui::{pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, SmartTileSource, TileRenderMode, TilesetDef, TilesetRect};
use lc_core::fields::{
    EditorDisplayMode, EditorDisplayPos, FieldDef, FieldType, FieldValue, FieldValues,
};
use lc_core::layer::TileRef;
use lc_feature_entities::format_overlay_text;
use lc_ui_api::AssetService;

use crate::features::editor::shell_app::ShellApp;

/// Resolves a [`TileRef`] into a pixel [`TilesetRect`] using loaded texture dimensions when available.
pub fn resolve_tile_ref_rect(
    tile_ref: TileRef,
    ts: &TilesetDef,
    tex: Option<&TextureHandle>,
) -> TilesetRect {
    let tw = ts.tile_size.x.max(1);
    let th = ts.tile_size.y.max(1);
    let cols = tex
        .map(|t| {
            let img_w = t.size()[0] as u32;
            let eff_w = img_w.saturating_sub(ts.padding * 2);
            (eff_w + ts.spacing)
                .checked_div(tw + ts.spacing)
                .unwrap_or(1)
                .max(1)
        })
        .unwrap_or(1);
    let col = tile_ref.tile_id % cols;
    let row = tile_ref.tile_id / cols;
    TilesetRect::new(
        ts.id,
        ts.padding + col * (tw + ts.spacing),
        ts.padding + row * (th + ts.spacing),
        tw,
        th,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_preview_field_badges(
    app: &ShellApp,
    ui: &Ui,
    painter: &Painter,
    t: &Tokens,
    ent_rect: Rect,
    mut above_y: f32,
    def: &EntityDef,
    sample_values: &FieldValues,
    base_color: Color32,
) {
    let mut beneath_y = ent_rect.max.y + 4.0;
    let mut center_offset_y = 0.0_f32;
    let text_color = base_color.lerp_to_gamma(t.ink_strong, 0.55);

    for field in &def.fields {
        let mode = field.editor_display_mode;
        if !matches!(
            mode,
            EditorDisplayMode::ValueOnly
                | EditorDisplayMode::NameAndValue
                | EditorDisplayMode::ArrayCountWithLabel
                | EditorDisplayMode::ArrayCountNoLabel
        ) {
            continue;
        }
        let Some(value) = sample_values.get(&field.name) else {
            continue;
        };

        let scale = field.editor_display_scale.clamp(0.5, 2.5);
        let font_sz = (10.0 * scale).clamp(8.0, 15.0);
        let text = format_overlay_text(field, value, mode);
        let swatch_colors = preview_swatch_colors(field, value);
        let tile_thumbs = preview_tile_rects(field, value, &app.project.defs.enums);

        let (pos, align) = match field.editor_display_pos {
            EditorDisplayPos::Above => {
                let p = pos2(ent_rect.center().x, above_y);
                above_y -= font_sz + 3.0;
                (p, Align2::CENTER_BOTTOM)
            }
            EditorDisplayPos::Center => {
                let p = pos2(ent_rect.center().x, ent_rect.center().y + center_offset_y);
                center_offset_y += font_sz + 2.0;
                (p, Align2::CENTER_CENTER)
            }
            EditorDisplayPos::Beneath => {
                let p = pos2(ent_rect.center().x, beneath_y);
                beneath_y += font_sz + 3.0;
                (p, Align2::CENTER_TOP)
            }
        };

        if !swatch_colors.is_empty() {
            let r = (4.5 * scale).clamp(3.0, 7.0);
            let cy = match align {
                Align2::CENTER_BOTTOM => pos.y - r,
                Align2::CENTER_TOP => pos.y + r,
                _ => pos.y,
            };
            let label_shift = if text.is_empty() { 0.0 } else { 18.0 };
            if !text.is_empty() {
                painter.text(
                    pos2(pos.x - swatch_colors.len() as f32 * (r + 1.0), pos.y),
                    Align2::RIGHT_CENTER,
                    &text,
                    FontId::proportional(font_sz),
                    text_color,
                );
            }
            let start_x = pos.x + label_shift * 0.5 - (swatch_colors.len() as f32 * r);
            for (i, c) in swatch_colors.iter().take(4).enumerate() {
                let center = pos2(start_x + i as f32 * (r * 2.2), cy);
                painter.circle_filled(center, r, *c);
                painter.circle_stroke(center, r, Stroke::new(1.0, Color32::BLACK));
            }
            continue;
        }

        if !tile_thumbs.is_empty()
            && matches!(
                mode,
                EditorDisplayMode::ValueOnly | EditorDisplayMode::NameAndValue
            )
        {
            let icon_sz = (14.0 * scale).clamp(10.0, 20.0);
            let cy = match align {
                Align2::CENTER_BOTTOM => pos.y - icon_sz * 0.5,
                Align2::CENTER_TOP => pos.y + icon_sz * 0.5,
                _ => pos.y,
            };
            let mut rendered_any = false;
            let count = tile_thumbs.len().min(3);
            let start_x = pos.x - (count as f32 * (icon_sz + 2.0)) * 0.5 + icon_sz * 0.5;
            for (i, src) in tile_thumbs.iter().take(3).enumerate() {
                let icon_rect = Rect::from_center_size(
                    pos2(start_x + i as f32 * (icon_sz + 2.0), cy),
                    vec2(icon_sz, icon_sz),
                );
                let ts_id = match src {
                    SmartTileSource::Rect(r) => r.tileset_id,
                    SmartTileSource::Ref(tr) => tr.tileset_id,
                };
                if let Some(ts) = app.project.defs.tileset(ts_id) {
                    if let Ok(tex) = app.assets.load_texture(ui.ctx(), &ts.path) {
                        let px_rect = match src {
                            SmartTileSource::Rect(r) => *r,
                            SmartTileSource::Ref(tr) => resolve_tile_ref_rect(*tr, ts, Some(&tex)),
                        };
                        paint_entity_tile(
                            painter,
                            &tex,
                            px_rect,
                            icon_rect,
                            TileRenderMode::FitInside,
                            [0; 4],
                            1.0,
                            1.0,
                        );
                        rendered_any = true;
                    }
                }
            }
            if rendered_any {
                if mode == EditorDisplayMode::NameAndValue {
                    painter.text(
                        pos2(start_x - icon_sz * 0.5 - 4.0, cy),
                        Align2::RIGHT_CENTER,
                        format!("{} =", field.name),
                        FontId::proportional(font_sz),
                        text_color,
                    );
                }
                continue;
            }
        }

        if !text.is_empty() {
            painter.text(pos, align, text, FontId::proportional(font_sz), text_color);
        }
    }
}

fn preview_swatch_colors(field: &FieldDef, value: &FieldValue) -> Vec<Color32> {
    if !matches!(field.base_type(), FieldType::Color) {
        return Vec::new();
    }
    match value {
        FieldValue::Color(c) => vec![Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])],
        FieldValue::Array(items) => items
            .iter()
            .filter_map(|v| match v {
                FieldValue::Color(c) => {
                    Some(Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn preview_tile_rects(
    field: &FieldDef,
    value: &FieldValue,
    enums: &[lc_core::defs::EnumDef],
) -> Vec<SmartTileSource> {
    match (field.base_type(), value) {
        (FieldType::Tile, FieldValue::Tile(Some(r))) => vec![SmartTileSource::Rect(*r)],
        (FieldType::Tile, FieldValue::Array(items)) => items
            .iter()
            .filter_map(|v| match v {
                FieldValue::Tile(Some(r)) => Some(SmartTileSource::Rect(*r)),
                _ => None,
            })
            .collect(),
        (FieldType::Enum(eid), FieldValue::Enum(id)) => enums
            .iter()
            .find(|e| e.id == *eid)
            .and_then(|e| e.value(id))
            .and_then(|v| v.tile)
            .into_iter()
            .map(SmartTileSource::Ref)
            .collect(),
        (FieldType::Enum(eid), FieldValue::Array(items)) => items
            .iter()
            .filter_map(|v| match v {
                FieldValue::Enum(id) => enums
                    .iter()
                    .find(|e| e.id == *eid)
                    .and_then(|e| e.value(id))
                    .and_then(|vd| vd.tile)
                    .map(SmartTileSource::Ref),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
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
            let t_px = (top_px as f32).min(sh * 0.49);
            let b = (bottom_px as f32).min(sh * 0.49);
            let dl = (l * zoom).min(dest_rect.width() * 0.49);
            let dr = (r * zoom).min(dest_rect.width() * 0.49);
            let dt = (t_px * zoom).min(dest_rect.height() * 0.49);
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
            let vs = [v0, v0 + t_px / tex_sz.y, v1 - b / tex_sz.y, v1];
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

pub(crate) fn paint_ellipse(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
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
