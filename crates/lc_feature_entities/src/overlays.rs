//! Canvas field display overlays for entity instances (labels, radii, point paths, ref links).

use egui::{pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::defs::EntityDef;
use lc_core::fields::{EditorDisplayMode, EditorDisplayPos, EditorLinkStyle, FieldValue};
use lc_core::layer::EntityInstance;
use lc_ui_api::canvas::LayerRenderCtx;

use crate::visual_effects::format_overlay_text;

pub(crate) fn render_field_overlays(
    ctx: &LayerRenderCtx,
    t: &Tokens,
    def: &EntityDef,
    entity: &EntityInstance,
    screen_rect: Rect,
    base_color: Color32,
    is_selected: bool,
) {
    let grid_px = ctx
        .project
        .defs
        .layer(ctx.layer.def_id)
        .map(|l| l.grid_size.x.max(1) as f32)
        .unwrap_or(16.0);
    let mut above_y = screen_rect.min.y - 4.0;
    let mut beneath_y = screen_rect.max.y + 4.0;

    for field in &def.fields {
        let mode = field.editor_display_mode;
        if mode == EditorDisplayMode::Hidden {
            continue;
        }
        let Some(value) = entity
            .fields
            .get(&field.name)
            .or(field.default_value.as_ref())
        else {
            continue;
        };
        if matches!(value, FieldValue::Null) && !field.editor_always_show {
            continue;
        }

        let scale = field.editor_display_scale.clamp(0.3, 4.0);
        let overlay_color = base_color.gamma_multiply(ctx.opacity);

        match mode {
            EditorDisplayMode::Hidden => {}
            EditorDisplayMode::ValueOnly
            | EditorDisplayMode::NameAndValue
            | EditorDisplayMode::ArrayCountWithLabel
            | EditorDisplayMode::ArrayCountNoLabel
            | EditorDisplayMode::EntityTile
            | EditorDisplayMode::LevelTile => {
                if !is_selected && !field.editor_always_show && ctx.viewport.zoom < 0.75 {
                    continue;
                }
                let text = format_overlay_text(field, value, mode);
                let color_discs = collect_color_discs(field.base_type(), value, mode);
                if text.is_empty() && color_discs.is_empty() {
                    continue;
                }
                let font =
                    FontId::proportional((10.5 * scale * ctx.viewport.zoom).clamp(8.0, 18.0));
                let (pos, align) = match field.editor_display_pos {
                    EditorDisplayPos::Above => {
                        let p = pos2(screen_rect.center().x, above_y);
                        above_y -= font.size + 2.0;
                        (p, Align2::CENTER_BOTTOM)
                    }
                    EditorDisplayPos::Center => (screen_rect.center(), Align2::CENTER_CENTER),
                    EditorDisplayPos::Beneath => {
                        let p = pos2(screen_rect.center().x, beneath_y);
                        beneath_y += font.size + 2.0;
                        (p, Align2::CENTER_TOP)
                    }
                };
                if !text.is_empty() {
                    let text_pos = if color_discs.is_empty() {
                        pos
                    } else {
                        pos2(pos.x - (color_discs.len() as f32 * 6.0), pos.y)
                    };
                    ctx.painter.text(
                        text_pos,
                        align,
                        text,
                        font.clone(),
                        t.ink_strong.gamma_multiply(ctx.opacity),
                    );
                }
                if !color_discs.is_empty() {
                    let radius = (4.5 * scale * ctx.viewport.zoom).clamp(3.0, 8.0);
                    let total_w = color_discs.len() as f32 * (radius * 2.2);
                    let start_x = pos.x - total_w * 0.5 + radius;
                    let cy = match align {
                        Align2::CENTER_BOTTOM => pos.y - radius,
                        Align2::CENTER_TOP => pos.y + radius,
                        _ => pos.y,
                    };
                    for (i, c) in color_discs.iter().enumerate() {
                        let center = pos2(start_x + i as f32 * (radius * 2.4), cy);
                        ctx.painter
                            .circle_filled(center, radius, c.gamma_multiply(ctx.opacity));
                        ctx.painter.circle_stroke(
                            center,
                            radius,
                            Stroke::new(1.0, Color32::BLACK.gamma_multiply(ctx.opacity)),
                        );
                    }
                }
            }
            EditorDisplayMode::RadiusPx | EditorDisplayMode::RadiusGrid => {
                let raw_val = match value {
                    FieldValue::Int(v) => *v as f32,
                    FieldValue::Float(v) => *v as f32,
                    _ => 0.0,
                };
                let px_radius = if mode == EditorDisplayMode::RadiusGrid {
                    raw_val * grid_px
                } else {
                    raw_val
                };
                let screen_r = px_radius * ctx.viewport.zoom;
                if screen_r > 1.0 {
                    let origin = entity_pivot_screen(ctx, def, entity);
                    ctx.painter
                        .circle_stroke(origin, screen_r, Stroke::new(1.2, overlay_color));
                }
            }
            EditorDisplayMode::Points
            | EditorDisplayMode::PointStar
            | EditorDisplayMode::PointPath
            | EditorDisplayMode::PointPathLoop => {
                let pts = collect_point_screens(ctx, value);
                if pts.is_empty() {
                    continue;
                }
                let origin = entity_pivot_screen(ctx, def, entity);
                let line_stroke = Stroke::new(1.5, overlay_color);

                if mode == EditorDisplayMode::PointStar {
                    for &pt in &pts {
                        ctx.painter.line_segment([origin, pt], line_stroke);
                    }
                } else if mode == EditorDisplayMode::PointPath
                    || mode == EditorDisplayMode::PointPathLoop
                {
                    let mut prev = origin;
                    for &pt in &pts {
                        ctx.painter.line_segment([prev, pt], line_stroke);
                        prev = pt;
                    }
                    if mode == EditorDisplayMode::PointPathLoop && pts.len() >= 2 {
                        ctx.painter.line_segment([prev, origin], line_stroke);
                    }
                }

                for pt in pts {
                    paint_diamond_marker(ctx.painter, pt, 4.0, overlay_color);
                }
            }
            EditorDisplayMode::RefLinkBetweenPivots | EditorDisplayMode::RefLinkBetweenCenters => {
                let targets = collect_entity_refs(value);
                let from = if mode == EditorDisplayMode::RefLinkBetweenPivots {
                    entity_pivot_screen(ctx, def, entity)
                } else {
                    screen_rect.center()
                };

                for target_id in targets {
                    let Some(target) = ctx.level.entity(target_id) else {
                        continue;
                    };
                    let to = if mode == EditorDisplayMode::RefLinkBetweenPivots {
                        let t_def = ctx.project.defs.entity(target.def_id);
                        let tp = t_def.map(|d| d.pivot).unwrap_or(glam::Vec2::splat(0.5));
                        let world = ctx.level.world_pos
                            + target.pos_px
                            + IVec2::new(
                                (target.size_px.x as f32 * tp.x) as i32,
                                (target.size_px.y as f32 * tp.y) as i32,
                            );
                        ctx.viewport.world_to_screen(world)
                    } else {
                        let min = ctx
                            .viewport
                            .world_to_screen(ctx.level.world_pos + target.pos_px);
                        let max = ctx.viewport.world_to_screen(
                            ctx.level.world_pos + target.pos_px + target.size_px.as_ivec2(),
                        );
                        Rect::from_two_pos(min, max).center()
                    };

                    paint_link_line(
                        ctx.painter,
                        from,
                        to,
                        field.editor_link_style,
                        Stroke::new(1.5, overlay_color),
                    );
                }
            }
        }
    }
}

/// Draws an LDtk-style diamond point marker centered at `pt`.
pub fn paint_diamond_marker(painter: &Painter, pt: Pos2, size: f32, color: Color32) {
    let pts = vec![
        pos2(pt.x, pt.y - size),
        pos2(pt.x + size, pt.y),
        pos2(pt.x, pt.y + size),
        pos2(pt.x - size, pt.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        pts,
        color,
        Stroke::new(1.0, Color32::BLACK),
    ));
}

fn collect_color_discs(
    base: &lc_core::fields::FieldType,
    value: &FieldValue,
    mode: EditorDisplayMode,
) -> Vec<Color32> {
    if !matches!(
        mode,
        EditorDisplayMode::ValueOnly | EditorDisplayMode::NameAndValue
    ) || !matches!(base, lc_core::fields::FieldType::Color)
    {
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

fn entity_pivot_screen(ctx: &LayerRenderCtx, def: &EntityDef, entity: &EntityInstance) -> Pos2 {
    let world = ctx.level.world_pos
        + entity.pos_px
        + IVec2::new(
            (entity.size_px.x as f32 * def.pivot.x) as i32,
            (entity.size_px.y as f32 * def.pivot.y) as i32,
        );
    ctx.viewport.world_to_screen(world)
}

fn collect_point_screens(ctx: &LayerRenderCtx, value: &FieldValue) -> Vec<Pos2> {
    match value {
        FieldValue::Point(pt) => vec![ctx.viewport.world_to_screen(ctx.level.world_pos + *pt)],
        FieldValue::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                FieldValue::Point(pt) => {
                    Some(ctx.viewport.world_to_screen(ctx.level.world_pos + *pt))
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn collect_entity_refs(value: &FieldValue) -> Vec<lc_core::id::EntityId> {
    match value {
        FieldValue::EntityRef(Some(id)) => vec![*id],
        FieldValue::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                FieldValue::EntityRef(Some(id)) => Some(*id),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Paints a directional link line matching LDtk's `FieldLinkStyle` presets.
pub fn paint_link_line(
    painter: &Painter,
    from: Pos2,
    to: Pos2,
    style: EditorLinkStyle,
    stroke: Stroke,
) {
    let diff = to - from;
    let len = diff.length();
    if len < 2.0 {
        return;
    }
    let dir = diff / len;
    let perp = vec2(-dir.y, dir.x);

    match style {
        EditorLinkStyle::StraightArrow => {
            painter.line_segment([from, to], stroke);
            paint_arrowhead(painter, to, dir, stroke);
        }
        EditorLinkStyle::CurvedArrow => {
            let ctrl = from + diff * 0.5 + perp * (len * 0.18).min(48.0);
            let mut prev = from;
            for i in 1..=16 {
                let u = i as f32 / 16.0;
                let inv = 1.0 - u;
                let p = pos2(
                    inv * inv * from.x + 2.0 * inv * u * ctrl.x + u * u * to.x,
                    inv * inv * from.y + 2.0 * inv * u * ctrl.y + u * u * to.y,
                );
                painter.line_segment([prev, p], stroke);
                prev = p;
            }
            let end_dir = (to - ctrl).normalized();
            paint_arrowhead(painter, to, end_dir, stroke);
        }
        EditorLinkStyle::ArrowsLine => {
            painter.line_segment([from, to], stroke);
            let mid = from + diff * 0.5;
            paint_arrowhead(painter, mid, dir, stroke);
            paint_arrowhead(painter, to, dir, stroke);
        }
        EditorLinkStyle::DashedLine => {
            let dash = 6.0;
            let gap = 4.0;
            let mut d = 0.0;
            while d < len {
                let a = from + dir * d;
                let b = from + dir * (d + dash).min(len);
                painter.line_segment([a, b], stroke);
                d += dash + gap;
            }
        }
        EditorLinkStyle::ZigZag => {
            let mid = from + diff * 0.5;
            let p1 = pos2(mid.x, from.y);
            let p2 = pos2(mid.x, to.y);
            painter.line_segment([from, p1], stroke);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, to], stroke);
            paint_arrowhead(painter, to, (to - p2).normalized(), stroke);
        }
    }
}

fn paint_arrowhead(painter: &Painter, tip: Pos2, dir: egui::Vec2, stroke: Stroke) {
    let perp = vec2(-dir.y, dir.x);
    let base = tip - dir * 8.0;
    painter.line_segment([tip, base + perp * 4.0], stroke);
    painter.line_segment([tip, base - perp * 4.0], stroke);
}
