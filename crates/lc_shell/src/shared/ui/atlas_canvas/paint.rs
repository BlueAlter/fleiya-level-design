//! Painting functions for atlas canvas layers: background, shadow, texture, grid, overlays, selection, and badges.

use egui::{
    vec2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, TextureHandle, Ui,
};
use lc_canvas::theme::Tokens;

use super::super::tile_bounds::TileGrid;
use super::state::{AtlasCanvasState, TileDimPredicate, TileOverlayCallback};
use super::view::page_bg_uv;

/// Paints all atlas layers inside the clipped viewport.
#[allow(clippy::too_many_arguments)]
pub fn paint_canvas_layers(
    ui: &Ui,
    painter: &Painter,
    t: &Tokens,
    state: &AtlasCanvasState,
    viewport_rect: Rect,
    texture: &TextureHandle,
    page_bg: Option<&TextureHandle>,
    grid: &TileGrid,
    img_w: f32,
    img_h: f32,
    hovered_c_r: Option<(u32, u32)>,
    overlay: &mut Option<&mut TileOverlayCallback<'_>>,
    dim: Option<&TileDimPredicate<'_>>,
) {
    // 1. Workspace canvas background
    painter.rect_filled(viewport_rect, CornerRadius::ZERO, t.canvas);

    let page_rect = Rect::from_min_size(Pos2::ZERO, vec2(img_w, img_h));
    let screen_page_rect = state.view.page_to_screen_rect(page_rect);

    // 2. Drop shadow behind the page
    painter.add(
        egui::epaint::Shadow {
            offset: [4, 6],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(130),
        }
        .as_shape(screen_page_rect, CornerRadius::ZERO),
    );

    // 3. Page background
    painter.rect_filled(screen_page_rect, CornerRadius::ZERO, t.panel);
    if let Some(bg) = page_bg {
        painter.image(
            bg.id(),
            screen_page_rect,
            page_bg_uv(page_rect),
            Color32::WHITE,
        );
    }

    // 4. Tileset texture
    painter.image(
        texture.id(),
        screen_page_rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    // 5. Page outline
    painter.rect_stroke(
        screen_page_rect,
        CornerRadius::ZERO,
        Stroke::new(2.0, Color32::BLACK),
        StrokeKind::Outside,
    );

    // 6. Grid lines
    if state.show_grid && grid.cols > 0 && grid.rows > 0 {
        let grid_clip = screen_page_rect.intersect(viewport_rect);
        let grid_painter = painter.with_clip_rect(grid_clip);
        let stride_x = grid.tile_w + grid.spacing;
        let stride_y = grid.tile_h + grid.spacing;
        let step_x = (stride_x as f32 * state.view.zoom).round();
        let step_y = (stride_y as f32 * state.view.zoom).round();

        if step_x >= 4.0 && step_y >= 4.0 {
            let grid_stroke = Stroke::new(1.0, Color32::from_white_alpha(35));
            let mut shapes = Vec::new();

            for c in 0..=grid.cols {
                let page_x = if c < grid.cols {
                    grid.cell_rect(c, 0).min.x
                } else {
                    grid.cell_rect(grid.cols - 1, 0).max.x
                };
                let screen_x = (page_x * state.view.zoom + state.view.offset.x).round();
                if screen_x >= grid_clip.min.x && screen_x <= grid_clip.max.x {
                    shapes.push(egui::Shape::line_segment(
                        [
                            Pos2::new(screen_x, grid_clip.min.y),
                            Pos2::new(screen_x, grid_clip.max.y),
                        ],
                        grid_stroke,
                    ));
                }
            }

            for r in 0..=grid.rows {
                let page_y = if r < grid.rows {
                    grid.cell_rect(0, r).min.y
                } else {
                    grid.cell_rect(0, grid.rows - 1).max.y
                };
                let screen_y = (page_y * state.view.zoom + state.view.offset.y).round();
                if screen_y >= grid_clip.min.y && screen_y <= grid_clip.max.y {
                    shapes.push(egui::Shape::line_segment(
                        [
                            Pos2::new(grid_clip.min.x, screen_y),
                            Pos2::new(grid_clip.max.x, screen_y),
                        ],
                        grid_stroke,
                    ));
                }
            }

            grid_painter.extend(shapes);
        }
    }

    // 7. Dimming & Overlay callbacks
    if (dim.is_some() || overlay.is_some()) && grid.cols > 0 && grid.rows > 0 {
        for c in 0..grid.cols {
            for r in 0..grid.rows {
                let cell_page = grid.cell_rect(c, r);
                let cell_screen = state.view.page_to_screen_rect(cell_page);
                if !cell_screen.intersects(viewport_rect) {
                    continue;
                }
                if let Some(dim_fn) = dim {
                    if dim_fn((c, r)) {
                        painter.rect_filled(
                            cell_screen,
                            CornerRadius::ZERO,
                            Color32::from_black_alpha(160),
                        );
                    }
                }
                if let Some(ref mut over_fn) = overlay {
                    over_fn(painter, (c, r), cell_screen);
                }
            }
        }
    }

    // 8. Selection Highlight
    let active_sel = state.live_drag_rect.or(state.selected_tile_rect);
    if let Some(sel) = active_sel {
        if grid.cols > 0 && grid.rows > 0 {
            let clamped = sel.clamp(grid.cols, grid.rows);
            let page_union = clamped.union_rect(grid);
            let screen_box = state.view.page_to_screen_rect(page_union);

            if state.is_right_dragging {
                painter.rect_filled(
                    screen_box,
                    CornerRadius::ZERO,
                    Color32::from_rgba_unmultiplied(61, 214, 184, 45),
                );
                painter.rect_stroke(
                    screen_box,
                    CornerRadius::ZERO,
                    Stroke::new(2.0, t.accent),
                    StrokeKind::Inside,
                );
            } else {
                painter.rect_filled(
                    screen_box,
                    CornerRadius::ZERO,
                    Color32::from_rgba_unmultiplied(255, 193, 77, 60),
                );
                painter.rect_stroke(
                    screen_box,
                    CornerRadius::ZERO,
                    Stroke::new(2.5, Color32::from_rgb(255, 193, 77)),
                    StrokeKind::Inside,
                );
            }
        }
    }

    // 9. Single Tile Hover Highlight Box
    if !state.is_right_dragging {
        if let Some(cell) = hovered_c_r {
            let cell_page = grid.cell_rect(cell.0, cell.1);
            let cell_screen = state.view.page_to_screen_rect(cell_page);
            painter.rect_filled(
                cell_screen,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(61, 214, 184, 45),
            );
            painter.rect_stroke(
                cell_screen,
                CornerRadius::ZERO,
                Stroke::new(1.5, t.accent),
                StrokeKind::Inside,
            );
        }
    }

    // 10. Tile ID / Dimensions Badge in top-right
    let active_rect = state.live_drag_rect.or(state.selected_tile_rect);
    let badge_info = if let Some(r) = active_rect {
        if grid.cols > 0 && grid.rows > 0 {
            let c = r.clamp(grid.cols, grid.rows);
            let id = c.top_left_id(grid);
            if c.w > 1 || c.h > 1 {
                Some(format!("#{id} ({}x{})", c.w, c.h))
            } else {
                Some(format!("#{id}"))
            }
        } else {
            None
        }
    } else {
        state.hovered_tile.map(|id| format!("#{id}"))
    };

    if let Some(badge_text) = badge_info {
        let font_id = FontId::monospace(12.0);
        let galley = ui
            .painter()
            .layout_no_wrap(badge_text, font_id, t.ink_strong);
        let badge_padding = vec2(8.0, 5.0);
        let badge_size = galley.size() + badge_padding * 2.0;

        let badge_pos = Pos2::new(
            viewport_rect.right() - badge_size.x - 12.0,
            viewport_rect.top() + 12.0,
        );
        let badge_rect = Rect::from_min_size(badge_pos, badge_size);

        let bp = ui.painter().with_clip_rect(viewport_rect);
        bp.rect_filled(
            badge_rect,
            CornerRadius::ZERO,
            Color32::from_black_alpha(220),
        );
        bp.rect_stroke(
            badge_rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, t.line_strong),
            StrokeKind::Inside,
        );
        bp.galley(badge_pos + badge_padding, galley, t.ink_strong);
    }

    // 11. Zoom % label in bottom-right corner
    let zoom_text = format!("{:.0}%", state.view.zoom * 100.0);
    let zoom_font = FontId::monospace(11.0);
    let zoom_galley = ui
        .painter()
        .layout_no_wrap(zoom_text, zoom_font, t.ink_strong);
    let zoom_pad = vec2(6.0, 3.0);
    let zoom_size = zoom_galley.size() + zoom_pad * 2.0;
    let zoom_pos = Pos2::new(
        viewport_rect.right() - zoom_size.x - 10.0,
        viewport_rect.bottom() - zoom_size.y - 10.0,
    );
    let zoom_rect = Rect::from_min_size(zoom_pos, zoom_size);
    let zp = ui.painter().with_clip_rect(viewport_rect);
    zp.rect_filled(
        zoom_rect,
        CornerRadius::same(3),
        Color32::from_black_alpha(180),
    );
    zp.rect_stroke(
        zoom_rect,
        CornerRadius::same(3),
        Stroke::new(1.0, t.hairline().color),
        StrokeKind::Inside,
    );
    zp.galley(zoom_pos + zoom_pad, zoom_galley, t.ink_strong);
}
