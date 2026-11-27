//! Layer handler for Entities layers: draws LDtk entity shapes, selection gizmos,
//! resize handles, and field editor display overlays.

use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, EntityRenderMode};
use lc_core::layer::{EntityInstance, LayerData, LayerKind};
use lc_ui_api::canvas::{CanvasTool, LayerHandler, LayerRenderCtx};

use crate::overlays::render_field_overlays;
use crate::tools::EntityCanvasTool;

/// Handler for `LayerKind::Entities`.
#[derive(Debug, Default)]
pub struct EntitiesLayerHandler;

impl EntitiesLayerHandler {
    /// Creates a new `EntitiesLayerHandler`.
    pub fn new() -> Self {
        Self
    }
}

impl LayerHandler for EntitiesLayerHandler {
    fn kind(&self) -> LayerKind {
        LayerKind::Entities
    }

    fn render(&self, ctx: &mut LayerRenderCtx) {
        if !ctx.layer.visible {
            return;
        }

        let LayerData::Entities(entities) = &ctx.layer.data else {
            return;
        };

        let t = Tokens::get(ctx.painter.ctx());
        let (visible_min, visible_max) = ctx.viewport.visible_world_bounds();
        let level_offset = ctx.level.world_pos;
        let selected_ids = ctx.selection.entities().unwrap_or_default();

        for entity in entities {
            let entity_world_min = level_offset + entity.pos_px;
            let entity_world_max =
                entity_world_min + IVec2::new(entity.size_px.x as i32, entity.size_px.y as i32);

            let def = ctx.project.defs.entity(entity.def_id);
            let is_selected = selected_ids.contains(&entity.id);

            let offscreen = entity_world_max.x < visible_min.x
                || entity_world_min.x > visible_max.x
                || entity_world_max.y < visible_min.y
                || entity_world_min.y > visible_max.y;

            let screen_rect = Rect::from_two_pos(
                ctx.viewport.world_to_screen(entity_world_min),
                ctx.viewport.world_to_screen(entity_world_max),
            );

            let rgba = def
                .map(|d| d.smart_color(&entity.fields, &ctx.project.defs.enums))
                .unwrap_or([200, 80, 200, 255]);
            let base_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);

            if !offscreen {
                render_entity_shape(ctx, &t, def, screen_rect, base_color, is_selected);
                render_pivot_and_label(ctx, &t, def, entity, entity_world_min, screen_rect);
                if is_selected {
                    render_selection_gizmos(ctx, &t, def, screen_rect);
                }
            }

            if let Some(def) = def {
                render_field_overlays(ctx, &t, def, entity, screen_rect, base_color, is_selected);
            }
        }
    }

    fn tools(&self) -> Vec<Box<dyn CanvasTool>> {
        vec![Box::new(EntityCanvasTool::new())]
    }
}

fn render_entity_shape(
    ctx: &LayerRenderCtx,
    t: &Tokens,
    def: Option<&EntityDef>,
    screen_rect: Rect,
    base_color: Color32,
    is_selected: bool,
) {
    let hollow = def.is_some_and(|d| d.hollow);
    let fill_opacity = def.map_or(0.55, |d| {
        if hollow {
            0.06
        } else {
            d.fill_opacity.clamp(0.0, 1.0) * 0.55
        }
    });
    let line_opacity = def.map_or(1.0, |d| d.line_opacity.clamp(0.0, 1.0));
    let fill = base_color.gamma_multiply(ctx.opacity * fill_opacity);
    let border = if is_selected {
        t.accent_ink
    } else if ctx.is_active {
        base_color.gamma_multiply(ctx.opacity * line_opacity)
    } else {
        t.line.gamma_multiply(ctx.opacity * line_opacity)
    };
    let stroke_w = if is_selected {
        2.0
    } else if ctx.is_active {
        1.5
    } else {
        1.0
    };
    let stroke = Stroke::new(stroke_w, border);

    let mode = def.map_or(EntityRenderMode::Rectangle, |d| d.render_mode);
    match mode {
        EntityRenderMode::Rectangle | EntityRenderMode::Tile => {
            ctx.painter
                .rect_filled(screen_rect, CornerRadius::ZERO, fill);
            ctx.painter
                .rect_stroke(screen_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);
        }
        EntityRenderMode::Ellipse => {
            let center = screen_rect.center();
            let rx = screen_rect.width() * 0.5;
            let ry = screen_rect.height() * 0.5;
            const SEGS: usize = 28;
            let pts: Vec<Pos2> = (0..SEGS)
                .map(|i| {
                    let th = (i as f32 / SEGS as f32) * std::f32::consts::TAU;
                    pos2(center.x + rx * th.cos(), center.y + ry * th.sin())
                })
                .collect();
            ctx.painter
                .add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            ctx.painter.add(egui::Shape::closed_line(pts, stroke));
        }
        EntityRenderMode::Cross => {
            ctx.painter
                .rect_filled(screen_rect, CornerRadius::ZERO, fill.gamma_multiply(0.35));
            ctx.painter.rect_stroke(
                screen_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, border.gamma_multiply(0.5)),
                StrokeKind::Inside,
            );
            ctx.painter
                .line_segment([screen_rect.left_top(), screen_rect.right_bottom()], stroke);
            ctx.painter
                .line_segment([screen_rect.right_top(), screen_rect.left_bottom()], stroke);
        }
    }
}

fn render_pivot_and_label(
    ctx: &LayerRenderCtx,
    t: &Tokens,
    def: Option<&EntityDef>,
    entity: &EntityInstance,
    entity_world_min: IVec2,
    screen_rect: Rect,
) {
    let pivot = def
        .map(|d| d.pivot)
        .unwrap_or_else(|| glam::Vec2::splat(0.5));
    let pivot_world = entity_world_min
        + IVec2::new(
            (entity.size_px.x as f32 * pivot.x) as i32,
            (entity.size_px.y as f32 * pivot.y) as i32,
        );
    let pivot_screen = ctx.viewport.world_to_screen(pivot_world);
    ctx.painter.circle_filled(
        pivot_screen,
        2.5,
        Color32::WHITE.gamma_multiply(ctx.opacity),
    );

    let show_name = def.is_none_or(|d| d.show_name);
    if show_name && ctx.viewport.zoom >= 0.6 && screen_rect.width() >= 20.0 {
        let label = def.map(|d| d.name.as_str()).unwrap_or("Entity");
        let font = FontId::proportional((11.0 * ctx.viewport.zoom).clamp(8.0, 13.0));
        ctx.painter.text(
            screen_rect.center_top() + vec2(0.0, 2.0),
            Align2::CENTER_TOP,
            label,
            font,
            t.ink_strong.gamma_multiply(ctx.opacity),
        );
    }
}

fn render_selection_gizmos(
    ctx: &LayerRenderCtx,
    t: &Tokens,
    def: Option<&EntityDef>,
    screen_rect: Rect,
) {
    ctx.painter.rect_stroke(
        screen_rect.expand(1.5),
        CornerRadius::ZERO,
        Stroke::new(1.5, t.accent),
        StrokeKind::Outside,
    );

    let rx = def.is_some_and(|d| d.resizable_x);
    let ry = def.is_some_and(|d| d.resizable_y);
    if !rx && !ry {
        return;
    }

    let handles: [(Pos2, bool); 8] = [
        (screen_rect.left_top(), rx && ry),
        (pos2(screen_rect.center().x, screen_rect.min.y), ry),
        (screen_rect.right_top(), rx && ry),
        (pos2(screen_rect.max.x, screen_rect.center().y), rx),
        (screen_rect.right_bottom(), rx && ry),
        (pos2(screen_rect.center().x, screen_rect.max.y), ry),
        (screen_rect.left_bottom(), rx && ry),
        (pos2(screen_rect.min.x, screen_rect.center().y), rx),
    ];

    for (pos, enabled) in handles {
        if enabled {
            let hr = Rect::from_center_size(pos, vec2(6.0, 6.0));
            ctx.painter.rect_filled(hr, CornerRadius::ZERO, t.accent);
            ctx.painter.rect_stroke(
                hr,
                CornerRadius::ZERO,
                Stroke::new(1.0, Color32::WHITE),
                StrokeKind::Outside,
            );
        }
    }
}
