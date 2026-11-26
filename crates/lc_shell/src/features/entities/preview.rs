//! Live LDtk-faithful entity preview widget inside the Entity Workspace.

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_core::defs::{
    EntityDef, EntityRenderMode, SmartTileSource, TileRenderMode, TilesetDef, TilesetRect,
};
use lc_core::fields::{EditorDisplayMode, FieldValue, FieldValues};
use lc_feature_entities::{effective_preview_values, paint_diamond_marker, paint_link_line};
use lc_ui_api::AssetService;

use super::preview_badges::{
    paint_ellipse, paint_entity_tile, paint_preview_field_badges, resolve_tile_ref_rect,
};
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
    let _ = tile_ref.tile_id;
    Some(TilesetRect::new(
        ts.id, ts.padding, ts.padding, tile_w, tile_h,
    ))
}

/// Convenience helper returning sample values for all fields on `def`.
pub fn preview_samples_for_entity(app: &ShellApp, def: &EntityDef) -> FieldValues {
    effective_preview_values(def, &app.project.defs.enums, &app.project.defs.tilesets)
}

/// Renders a live LDtk-faithful preview of `def` with sample/default field values, smart color,
/// smart tile override, spatial overlays (radii, points, paths, ref links), and field badges.
pub fn render_entity_live_preview(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    def: &EntityDef,
    sample_values: &FieldValues,
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

    let pad = 34.0;
    let max_w = (rect.width() - pad * 2.0).max(16.0);
    let max_h = (rect.height() - pad * 2.0).max(16.0);
    let ent_w = def.size_px.x.max(1) as f32;
    let ent_h = def.size_px.y.max(1) as f32;

    // Fit cleanly even when a RadiusPx / RadiusGrid circle extends further than entity bounds.
    let grid_px = 16.0_f32;
    let mut extent_w = ent_w;
    let mut extent_h = ent_h;
    for field in &def.fields {
        if matches!(
            field.editor_display_mode,
            EditorDisplayMode::RadiusPx | EditorDisplayMode::RadiusGrid
        ) {
            if let Some(val) = sample_values.get(&field.name) {
                let raw = match val {
                    FieldValue::Int(v) => *v as f32,
                    FieldValue::Float(v) => *v as f32,
                    _ => 0.0,
                };
                let r = if field.editor_display_mode == EditorDisplayMode::RadiusGrid {
                    raw * grid_px
                } else {
                    raw
                };
                extent_w = extent_w.max(r * 2.0);
                extent_h = extent_h.max(r * 2.0);
            }
        }
    }

    let scale = (max_w / extent_w).min(max_h / extent_h).clamp(0.45, 3.5);
    let draw_size = vec2(ent_w * scale, ent_h * scale);
    let ent_rect = Rect::from_center_size(rect.center(), draw_size);

    let smart_rgba = def.smart_color(sample_values, &app.project.defs.enums);
    let base_color =
        Color32::from_rgba_unmultiplied(smart_rgba[0], smart_rgba[1], smart_rgba[2], smart_rgba[3]);
    let fill_alpha = if def.hollow {
        0.08
    } else {
        def.fill_opacity.clamp(0.0, 1.0) * 0.45
    };
    let line_alpha = def.line_opacity.clamp(0.0, 1.0);
    let fill_color = base_color.gamma_multiply(fill_alpha);
    let stroke = Stroke::new(1.5, base_color.gamma_multiply(line_alpha));

    let smart_tile = def.smart_tile(sample_values, &app.project.defs.enums);
    if smart_tile.is_some() {
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
    } else {
        match def.render_mode {
            EntityRenderMode::Rectangle => {
                painter.rect_filled(ent_rect, CornerRadius::ZERO, fill_color);
                painter.rect_stroke(ent_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);
            }
            EntityRenderMode::Ellipse => paint_ellipse(&painter, ent_rect, fill_color, stroke),
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
    }

    if let Some(source) = smart_tile {
        let ts_id = match source {
            SmartTileSource::Rect(r) => r.tileset_id,
            SmartTileSource::Ref(tr) => tr.tileset_id,
        };
        if let Some(ts) = app.project.defs.tileset(ts_id) {
            if let Ok(tex) = app.assets.load_texture(ui.ctx(), &ts.path) {
                let rect_px = match source {
                    SmartTileSource::Rect(r) => r,
                    SmartTileSource::Ref(tr) => resolve_tile_ref_rect(tr, ts, Some(&tex)),
                };
                paint_entity_tile(
                    &painter,
                    &tex,
                    rect_px,
                    ent_rect,
                    def.tile_render_mode,
                    def.nine_slice_borders,
                    def.tile_opacity.clamp(0.0, 1.0),
                    scale,
                );
            }
        }
    }

    if def.tile_render_mode == TileRenderMode::NineSlice {
        let [top, right, bottom, left] = def.nine_slice_borders;
        let guide = Stroke::new(1.0, t.accent.gamma_multiply(0.75));
        if top > 0 {
            painter.hline(
                ent_rect.x_range(),
                ent_rect.min.y + top as f32 * scale,
                guide,
            );
        }
        if bottom > 0 {
            painter.hline(
                ent_rect.x_range(),
                ent_rect.max.y - bottom as f32 * scale,
                guide,
            );
        }
        if left > 0 {
            painter.vline(
                ent_rect.min.x + left as f32 * scale,
                ent_rect.y_range(),
                guide,
            );
        }
        if right > 0 {
            painter.vline(
                ent_rect.max.x - right as f32 * scale,
                ent_rect.y_range(),
                guide,
            );
        }
    }

    let pivot_pos = pos2(
        ent_rect.min.x + ent_rect.width() * def.pivot.x.clamp(0.0, 1.0),
        ent_rect.min.y + ent_rect.height() * def.pivot.y.clamp(0.0, 1.0),
    );

    paint_preview_spatial_overlays(
        &painter,
        rect,
        ent_rect,
        pivot_pos,
        def,
        sample_values,
        base_color,
        scale,
        grid_px,
        t,
    );

    painter.circle_filled(pivot_pos, 3.5, Color32::WHITE);
    painter.circle_stroke(pivot_pos, 3.5, Stroke::new(1.2, Color32::BLACK));

    let mut above_y = ent_rect.min.y - 4.0;
    if def.show_name && !def.name.is_empty() {
        painter.text(
            pos2(ent_rect.center().x, above_y),
            Align2::CENTER_BOTTOM,
            &def.name,
            FontId::proportional(11.0),
            base_color
                .gamma_multiply(0.95)
                .lerp_to_gamma(t.ink_strong, 0.35),
        );
        above_y -= 13.0;
    }

    paint_preview_field_badges(
        app,
        ui,
        &painter,
        t,
        ent_rect,
        above_y,
        def,
        sample_values,
        base_color,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_preview_spatial_overlays(
    painter: &Painter,
    canvas_rect: Rect,
    ent_rect: Rect,
    pivot_pos: Pos2,
    def: &EntityDef,
    sample_values: &FieldValues,
    base_color: Color32,
    scale: f32,
    grid_px: f32,
    t: &Tokens,
) {
    for field in &def.fields {
        let Some(value) = sample_values.get(&field.name) else {
            continue;
        };
        match field.editor_display_mode {
            EditorDisplayMode::RadiusPx | EditorDisplayMode::RadiusGrid => {
                let raw = match value {
                    FieldValue::Int(v) => *v as f32,
                    FieldValue::Float(v) => *v as f32,
                    _ => 0.0,
                };
                let r_px = if field.editor_display_mode == EditorDisplayMode::RadiusGrid {
                    raw * grid_px
                } else {
                    raw
                };
                let screen_r = (r_px * scale).clamp(4.0, canvas_rect.height() * 0.46);
                painter.circle_stroke(
                    pivot_pos,
                    screen_r,
                    Stroke::new(1.3, base_color.gamma_multiply(0.85)),
                );
            }
            EditorDisplayMode::Points
            | EditorDisplayMode::PointStar
            | EditorDisplayMode::PointPath
            | EditorDisplayMode::PointPathLoop => {
                let pts = collect_preview_points(value, canvas_rect, ent_rect);
                if pts.is_empty() {
                    continue;
                }
                let stroke = Stroke::new(1.5, base_color.gamma_multiply(0.9));
                match field.editor_display_mode {
                    EditorDisplayMode::PointStar => {
                        for &pt in &pts {
                            paint_link_line(
                                painter,
                                pivot_pos,
                                pt,
                                field.editor_link_style,
                                stroke,
                            );
                        }
                    }
                    EditorDisplayMode::PointPath | EditorDisplayMode::PointPathLoop => {
                        let mut prev = pivot_pos;
                        for &pt in &pts {
                            paint_link_line(painter, prev, pt, field.editor_link_style, stroke);
                            prev = pt;
                        }
                        if field.editor_display_mode == EditorDisplayMode::PointPathLoop
                            && pts.len() >= 2
                        {
                            paint_link_line(
                                painter,
                                prev,
                                pivot_pos,
                                field.editor_link_style,
                                stroke,
                            );
                        }
                    }
                    _ => {}
                }
                for pt in pts {
                    paint_diamond_marker(painter, pt, 4.0, base_color);
                }
            }
            EditorDisplayMode::RefLinkBetweenPivots | EditorDisplayMode::RefLinkBetweenCenters => {
                let count = match value {
                    FieldValue::EntityRef(Some(_)) => 1,
                    FieldValue::Array(v) => v
                        .iter()
                        .filter(|item| matches!(item, FieldValue::EntityRef(Some(_))))
                        .count(),
                    _ => 0,
                };
                let from = if field.editor_display_mode == EditorDisplayMode::RefLinkBetweenPivots {
                    pivot_pos
                } else {
                    ent_rect.center()
                };
                let stroke = Stroke::new(1.5, base_color.gamma_multiply(0.9));
                for idx in 0..count.min(3) {
                    let target_center = pos2(
                        (canvas_rect.max.x - 24.0).max(ent_rect.max.x + 20.0),
                        (canvas_rect.min.y + 28.0 + idx as f32 * 28.0)
                            .min(canvas_rect.max.y - 20.0),
                    );
                    let target_rect = Rect::from_center_size(target_center, vec2(18.0, 18.0));
                    painter.rect_filled(
                        target_rect,
                        CornerRadius::ZERO,
                        t.sunken.gamma_multiply(0.75),
                    );
                    painter.rect_stroke(
                        target_rect,
                        CornerRadius::ZERO,
                        t.hairline(),
                        StrokeKind::Inside,
                    );
                    let to = if field.editor_display_mode == EditorDisplayMode::RefLinkBetweenPivots
                    {
                        pos2(target_rect.center().x, target_rect.max.y)
                    } else {
                        target_rect.center()
                    };
                    paint_link_line(painter, from, to, field.editor_link_style, stroke);
                }
            }
            _ => {}
        }
    }
}

fn collect_preview_points(value: &FieldValue, canvas_rect: Rect, ent_rect: Rect) -> Vec<Pos2> {
    let raw: Vec<_> = match value {
        FieldValue::Point(pt) => vec![*pt],
        FieldValue::Array(items) => items
            .iter()
            .filter_map(|v| match v {
                FieldValue::Point(pt) => Some(*pt),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    let rx = (canvas_rect.width() * 0.38).max(ent_rect.width() * 0.8 + 14.0);
    let ry = (canvas_rect.height() * 0.34).max(ent_rect.height() * 0.8 + 12.0);
    raw.into_iter()
        .enumerate()
        .map(|(i, pt)| {
            let sx = (pt.x as f32).clamp(-rx, rx);
            let sy = (pt.y as f32).clamp(-ry, ry);
            let angle = (i as f32) * 1.9;
            let px = if sx == 0.0 && sy == 0.0 {
                ent_rect.center().x + angle.cos() * 28.0
            } else {
                ent_rect.center().x + sx
            };
            let py = if sx == 0.0 && sy == 0.0 {
                ent_rect.center().y + angle.sin() * 22.0
            } else {
                ent_rect.center().y + sy
            };
            pos2(
                px.clamp(canvas_rect.min.x + 8.0, canvas_rect.max.x - 8.0),
                py.clamp(canvas_rect.min.y + 8.0, canvas_rect.max.y - 8.0),
            )
        })
        .collect()
}
