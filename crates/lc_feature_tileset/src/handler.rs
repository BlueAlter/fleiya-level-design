//! Layer handlers for Tiles and IntGrid layers, responsible for canvas rendering and tool provision.

use egui::{Color32, CornerRadius, FontId, Stroke, StrokeKind};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::layer::{LayerData, LayerKind};
use lc_ui_api::canvas::{CanvasTool, LayerHandler, LayerRenderCtx};
use lc_ui_api::viewport::Viewport;

use crate::tools::{EraseTool, EyedropperTool, FillTool, PaintTool, RectTool, TileCanvasTool};

/// Handler for `LayerKind::Tiles`, rendering placed tiles and providing tile editing tools.
#[derive(Debug, Default)]
pub struct TilesLayerHandler;

impl TilesLayerHandler {
    /// Creates a new `TilesLayerHandler`.
    pub fn new() -> Self {
        Self
    }
}

impl LayerHandler for TilesLayerHandler {
    fn kind(&self) -> LayerKind {
        LayerKind::Tiles
    }

    fn render(&self, ctx: &mut LayerRenderCtx) {
        if !ctx.layer.visible {
            return;
        }

        let grid = match &ctx.layer.data {
            LayerData::Tiles(grid) => grid,
            _ => return,
        };

        let layer_def = ctx
            .project
            .defs
            .layers
            .iter()
            .find(|l| l.id == ctx.layer.def_id);

        let grid_size = layer_def
            .map(|d| IVec2::new(d.grid_size.x as i32, d.grid_size.y as i32))
            .unwrap_or_else(|| IVec2::splat(16));

        // Viewport culling: only render cells within visible world bounds
        let (min_world, max_world) = ctx.viewport.visible_world_bounds();
        let min_grid = Viewport::world_to_grid(min_world, ctx.level.world_pos, grid_size);
        let max_grid = Viewport::world_to_grid(max_world, ctx.level.world_pos, grid_size);

        let min_x = min_grid.x.clamp(0, grid.width() as i32) as u32;
        let max_x = (max_grid.x + 1).clamp(0, grid.width() as i32) as u32;
        let min_y = min_grid.y.clamp(0, grid.height() as i32) as u32;
        let max_y = (max_grid.y + 1).clamp(0, grid.height() as i32) as u32;

        let t = Tokens::get(ctx.painter.ctx());
        let tile_fill_color = Color32::from_rgb(52, 94, 142).gamma_multiply(ctx.opacity);
        let tile_stroke = Stroke::new(1.0, t.line.gamma_multiply(ctx.opacity));

        for y in min_y..max_y {
            for x in min_x..max_x {
                if let Some(Some(tile_ref)) = grid.get(x, y) {
                    let screen_rect = ctx.viewport.grid_to_screen(
                        IVec2::new(x as i32, y as i32),
                        ctx.level.world_pos,
                        grid_size,
                    );

                    // Render placed tile cell
                    ctx.painter
                        .rect_filled(screen_rect, CornerRadius::ZERO, tile_fill_color);
                    ctx.painter.rect_stroke(
                        screen_rect,
                        CornerRadius::ZERO,
                        tile_stroke,
                        StrokeKind::Inside,
                    );

                    // If zoom is high enough, show tile ID label
                    if ctx.viewport.zoom >= 0.8 && screen_rect.width() >= 16.0 {
                        let label = format!("#{}", tile_ref.tile_id);
                        let font = FontId::monospace((10.0 * ctx.viewport.zoom).clamp(8.0, 14.0));
                        ctx.painter.text(
                            screen_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            font,
                            Color32::WHITE.gamma_multiply(ctx.opacity),
                        );
                    }
                }
            }
        }
    }

    fn tools(&self) -> Vec<Box<dyn CanvasTool>> {
        vec![
            Box::new(TileCanvasTool::new(LayerKind::Tiles)),
            Box::new(PaintTool::new(LayerKind::Tiles)),
            Box::new(EraseTool::new(LayerKind::Tiles)),
            Box::new(RectTool::new(LayerKind::Tiles)),
            Box::new(FillTool::new(LayerKind::Tiles)),
            Box::new(EyedropperTool::new(LayerKind::Tiles)),
        ]
    }
}

/// Handler for `LayerKind::IntGrid`, rendering integer color cells and providing intgrid editing tools.
#[derive(Debug, Default)]
pub struct IntGridLayerHandler;

impl IntGridLayerHandler {
    /// Creates a new `IntGridLayerHandler`.
    pub fn new() -> Self {
        Self
    }
}

impl LayerHandler for IntGridLayerHandler {
    fn kind(&self) -> LayerKind {
        LayerKind::IntGrid
    }

    fn render(&self, ctx: &mut LayerRenderCtx) {
        if !ctx.layer.visible {
            return;
        }

        let grid = match &ctx.layer.data {
            LayerData::IntGrid(grid) => grid,
            _ => return,
        };

        let layer_def = ctx
            .project
            .defs
            .layers
            .iter()
            .find(|l| l.id == ctx.layer.def_id);

        let grid_size = layer_def
            .map(|d| IVec2::new(d.grid_size.x as i32, d.grid_size.y as i32))
            .unwrap_or_else(|| IVec2::splat(16));

        // Viewport culling
        let (min_world, max_world) = ctx.viewport.visible_world_bounds();
        let min_grid = Viewport::world_to_grid(min_world, ctx.level.world_pos, grid_size);
        let max_grid = Viewport::world_to_grid(max_world, ctx.level.world_pos, grid_size);

        let min_x = min_grid.x.clamp(0, grid.width() as i32) as u32;
        let max_x = (max_grid.x + 1).clamp(0, grid.width() as i32) as u32;
        let min_y = min_grid.y.clamp(0, grid.height() as i32) as u32;
        let max_y = (max_grid.y + 1).clamp(0, grid.height() as i32) as u32;

        let t = Tokens::get(ctx.painter.ctx());
        let cell_stroke = Stroke::new(1.0, t.line.gamma_multiply(ctx.opacity));

        for y in min_y..max_y {
            for x in min_x..max_x {
                if let Some(&val) = grid.get(x, y) {
                    if val == 0 {
                        continue;
                    }

                    let cell_color = layer_def
                        .and_then(|d| {
                            d.int_grid_values.iter().find(|v| v.value == val).map(|v| {
                                Color32::from_rgba_unmultiplied(
                                    v.color[0], v.color[1], v.color[2], v.color[3],
                                )
                            })
                        })
                        .unwrap_or_else(|| {
                            // Deterministic color from integer value
                            let r = ((val * 53) % 200 + 55) as u8;
                            let g = ((val * 97) % 200 + 55) as u8;
                            let b = ((val * 149) % 200 + 55) as u8;
                            Color32::from_rgb(r, g, b)
                        })
                        .gamma_multiply(ctx.opacity);

                    let screen_rect = ctx.viewport.grid_to_screen(
                        IVec2::new(x as i32, y as i32),
                        ctx.level.world_pos,
                        grid_size,
                    );

                    ctx.painter
                        .rect_filled(screen_rect, CornerRadius::ZERO, cell_color);
                    ctx.painter.rect_stroke(
                        screen_rect,
                        CornerRadius::ZERO,
                        cell_stroke,
                        StrokeKind::Inside,
                    );

                    if ctx.viewport.zoom >= 0.75 && screen_rect.width() >= 14.0 {
                        let label = format!("{val}");
                        let font = FontId::monospace((11.0 * ctx.viewport.zoom).clamp(9.0, 15.0));
                        ctx.painter.text(
                            screen_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            font,
                            Color32::WHITE.gamma_multiply(ctx.opacity),
                        );
                    }
                }
            }
        }
    }

    fn tools(&self) -> Vec<Box<dyn CanvasTool>> {
        vec![
            Box::new(TileCanvasTool::new(LayerKind::IntGrid)),
            Box::new(PaintTool::new(LayerKind::IntGrid)),
            Box::new(EraseTool::new(LayerKind::IntGrid)),
            Box::new(RectTool::new(LayerKind::IntGrid)),
            Box::new(FillTool::new(LayerKind::IntGrid)),
            Box::new(EyedropperTool::new(LayerKind::IntGrid)),
        ]
    }
}
