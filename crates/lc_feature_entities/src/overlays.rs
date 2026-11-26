//! Canvas field display overlays for entity instances (labels, radii, point paths, ref links).

use egui::{pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::defs::EntityDef;
use lc_core::fields::{EditorDisplayMode, EditorDisplayPos, EditorLinkStyle, FieldDef, FieldValue};
use lc_core::layer::EntityInstance;
use lc_ui_api::canvas::LayerRenderCtx;

use crate::field_defs::format_field_value;

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
                if text.is_empty() {
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
                ctx.painter.text(
                    pos,
                    align,
                    text,
                    font,
                    t.ink_strong.gamma_multiply(ctx.opacity),
                );
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
                    ctx.painter.circle_filled(pt, 3.5, overlay_color);
                    ctx.painter
                        .circle_stroke(pt, 3.5, Stroke::new(1.0, Color32::BLACK));
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

fn format_overlay_text(field: &FieldDef, value: &FieldValue, mode: EditorDisplayMode) -> String {
    let mut txt = match mode {
        EditorDisplayMode::ArrayCountWithLabel => {
            let n = match value {
                FieldValue::Array(v) => v.len(),
                _ => 0,
            };
            format!("{}: [{n}]", field.name)
        }
        EditorDisplayMode::ArrayCountNoLabel => {
            let n = match value {
                FieldValue::Array(v) => v.len(),
                _ => 0,
            };
            format!("[{n}]")
        }
        _ => {
            let mut raw = format_field_value(value);
            if field.editor_cut_long_values && raw.chars().count() > 24 {
                raw = format!("{}…", raw.chars().take(23).collect::<String>());
            }
            if let Some(ref prefix) = field.editor_text_prefix {
                raw = format!("{prefix}{raw}");
            }
            if let Some(ref suffix) = field.editor_text_suffix {
                raw.push_str(suffix);
            }
            if mode == EditorDisplayMode::NameAndValue {
                format!("{}: {raw}", field.name)
            } else {
                raw
            }
        }
    };
    if txt.trim().is_empty() {
        txt.clear();
    }
    txt
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

fn paint_link_line(
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
