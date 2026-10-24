//! Canvas background, grid and selection helpers.

use egui::{CornerRadius, Painter, Rect, Stroke, StrokeKind};

use crate::theme::Tokens;
use lc_ui_api::viewport::Viewport;

/// Draws a world-aligned grid inside `rect`.
///
/// `grid_size` is the world-space size of one cell. Minor lines are dropped
/// when cells become too small, keeping the grid crisp at low zoom levels.
pub fn draw_grid(
    painter: &Painter,
    rect: Rect,
    viewport: &Viewport,
    grid_size: i32,
    tokens: &Tokens,
) {
    if grid_size <= 0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let cell_px = viewport.zoom * grid_size as f32;
    // At very low zoom, draw every eighth line only.
    let step = if cell_px < 3.0 { 8 } else { 1 };
    let (min, max) = viewport.visible_world_bounds();

    let start_x = min.x.div_euclid(grid_size).div_euclid(step) * step;
    let end_x = max.x.div_euclid(grid_size).div_euclid(step) * step + step;
    let start_y = min.y.div_euclid(grid_size).div_euclid(step) * step;
    let end_y = max.y.div_euclid(grid_size).div_euclid(step) * step + step;

    let p = painter.with_clip_rect(rect);
    let origin_x = viewport.screen_rect.min.x + viewport.pan.x;
    let origin_y = viewport.screen_rect.min.y + viewport.pan.y;

    let mut gx = start_x;
    while gx <= end_x {
        let x = origin_x + (gx * grid_size) as f32 * viewport.zoom;
        if x >= rect.left() - 1.0 && x <= rect.right() + 1.0 {
            let major = gx.rem_euclid(8) == 0;
            let stroke = if gx == 0 {
                Stroke::new(1.0, tokens.axis_y)
            } else if major {
                Stroke::new(1.0, tokens.grid_major)
            } else {
                Stroke::new(1.0, tokens.grid_minor)
            };
            p.vline(x + 0.5, rect.y_range(), stroke);
        }
        gx += step;
    }

    let mut gy = start_y;
    while gy <= end_y {
        let y = origin_y + (gy * grid_size) as f32 * viewport.zoom;
        if y >= rect.top() - 1.0 && y <= rect.bottom() + 1.0 {
            let major = gy.rem_euclid(8) == 0;
            let stroke = if gy == 0 {
                Stroke::new(1.0, tokens.axis_x)
            } else if major {
                Stroke::new(1.0, tokens.grid_major)
            } else {
                Stroke::new(1.0, tokens.grid_minor)
            };
            p.hline(rect.x_range(), y + 0.5, stroke);
        }
        gy += step;
    }
}

/// Draws a checkerboard, used behind transparent tiles.
pub fn draw_checkerboard(painter: &Painter, rect: Rect, cell: f32, tokens: &Tokens) {
    let cell = cell.max(4.0);
    let p = painter.with_clip_rect(rect);
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                let min = rect.min + egui::vec2(col as f32 * cell, row as f32 * cell);
                let square = Rect::from_min_size(min, egui::vec2(cell, cell)).intersect(rect);
                p.rect_filled(square, CornerRadius::ZERO, tokens.sunken);
            }
        }
    }
}

/// Draws a selection rectangle with a 1 px outline.
pub fn selection_rect(painter: &Painter, rect: Rect, tokens: &Tokens) {
    painter.rect_filled(rect, CornerRadius::ZERO, tokens.selection_fill);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, tokens.selection_stroke),
        StrokeKind::Inside,
    );
}

/// Draws a resize/selection handle centred on `center`.
pub fn handle(painter: &Painter, center: egui::Pos2, tokens: &Tokens) {
    let rect = Rect::from_center_size(center, egui::vec2(6.0, 6.0));
    painter.rect_filled(rect, CornerRadius::ZERO, tokens.accent);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, tokens.invert_bg),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_and_selection_draw_without_panicking() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let rect = ui.max_rect();
            let mut viewport = Viewport::new(rect);
            viewport.set_zoom(1.0);
            draw_grid(ui.painter(), rect, &viewport, 16, &Tokens::DARK);
            draw_checkerboard(ui.painter(), rect, 16.0, &Tokens::DARK);
            selection_rect(ui.painter(), rect.shrink(10.0), &Tokens::DARK);
            handle(ui.painter(), rect.center(), &Tokens::DARK);
        });
        output.textures_delta.clear();
    }
}
