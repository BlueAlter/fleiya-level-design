//! Layer handler for Entities layers: draws entity footprints and provides
//! the entity editing tool.

use egui::{Align2, Color32, CornerRadius, FontId, Stroke, StrokeKind};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::layer::{LayerData, LayerKind};
use lc_ui_api::canvas::{CanvasTool, LayerHandler, LayerRenderCtx};

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

        for entity in entities {
            let entity_world_min = level_offset + entity.pos_px;
            let entity_world_max =
                entity_world_min + IVec2::new(entity.size_px.x as i32, entity.size_px.y as i32);

            // Viewport culling.
            if entity_world_max.x < visible_min.x
                || entity_world_min.x > visible_max.x
                || entity_world_max.y < visible_min.y
                || entity_world_min.y > visible_max.y
            {
                continue;
            }

            let screen_rect = egui::Rect::from_two_pos(
                ctx.viewport.world_to_screen(entity_world_min),
                ctx.viewport.world_to_screen(entity_world_max),
            );

            let def = ctx.project.defs.entity(entity.def_id);
            let color = def
                .map(|d| {
                    Color32::from_rgba_unmultiplied(d.color[0], d.color[1], d.color[2], d.color[3])
                })
                .unwrap_or(Color32::from_rgb(200, 80, 200));

            let fill = color.gamma_multiply(ctx.opacity * 0.55);
            let border = color.gamma_multiply(ctx.opacity);
            let stroke = if ctx.is_active {
                Stroke::new(2.0, border)
            } else {
                Stroke::new(1.0, t.line.gamma_multiply(ctx.opacity))
            };

            ctx.painter
                .rect_filled(screen_rect, CornerRadius::ZERO, fill);
            ctx.painter
                .rect_stroke(screen_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

            // Pivot marker.
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

            // Name label when there is room.
            if ctx.viewport.zoom >= 0.6 && screen_rect.width() >= 24.0 {
                let label = def.map(|d| d.name.as_str()).unwrap_or("Entity");
                let font = FontId::proportional((11.0 * ctx.viewport.zoom).clamp(8.0, 13.0));
                ctx.painter.text(
                    screen_rect.center_top() + egui::vec2(0.0, 2.0),
                    Align2::CENTER_TOP,
                    label,
                    font,
                    t.ink_strong.gamma_multiply(ctx.opacity),
                );
            }
        }
    }

    fn tools(&self) -> Vec<Box<dyn CanvasTool>> {
        vec![Box::new(EntityCanvasTool::new())]
    }
}
