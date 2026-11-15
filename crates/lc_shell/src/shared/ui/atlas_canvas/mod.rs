//! Reusable atlas canvas widget with pan, zoom, grid, tile selection, and overlays.
//!
//! Extracted from `TilePicker` so tileset workspaces, tile pickers, and entity editors
//! share identical viewport navigation and tile interaction semantics.

pub mod paint;
pub mod state;
pub mod view;

pub use paint::paint_canvas_layers;
pub use state::{AtlasCanvasState, AtlasEvent, TileDimPredicate, TileOverlayCallback};
pub use view::{
    clamp_offset, compute_fit_view, page_bg_uv, tile_at_screen_pos, zoom_towards_cursor,
    PickerView, PAGE_BG_CELL,
};

use egui::{
    vec2, CursorIcon, Key, Modifiers, PointerButton, Response, Sense, TextureHandle, Ui, Vec2,
};
use lc_canvas::theme::Tokens;

use super::tile_bounds::{TileGrid, TileRect};

/// Renders the interactive atlas canvas with pan, zoom, grid, overlays, and selection.
#[allow(clippy::too_many_arguments)]
pub fn render_atlas_canvas(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut AtlasCanvasState,
    texture: &TextureHandle,
    page_bg: Option<&TextureHandle>,
    grid: &TileGrid,
    img_size: Vec2,
    mut overlay: Option<&mut TileOverlayCallback<'_>>,
    dim: Option<&TileDimPredicate<'_>>,
) -> (Response, Vec<AtlasEvent>) {
    let mut events = Vec::new();
    let img_w = img_size.x.max(1.0);
    let img_h = img_size.y.max(1.0);

    let (viewport_rect, response) =
        ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

    // 1. Initial fit
    if state.needs_initial_fit {
        state.fit_to_viewport(viewport_rect, img_w, img_h);
        state.needs_initial_fit = false;
    }

    // 2. Navigation: Fit on double-click or F / Home key
    let double_clicked = response.double_clicked();
    let fit_key = ui.ctx().input_mut(|i| {
        i.consume_key(Modifiers::NONE, Key::F) || i.consume_key(Modifiers::NONE, Key::Home)
    });
    if double_clicked || fit_key {
        state.fit_to_viewport(viewport_rect, img_w, img_h);
    }

    // 3. Navigation: Pan (Middle-drag or Space+drag)
    let space_down = ui.ctx().input(|i| i.key_down(Key::Space));
    if space_down {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }
    if response.dragged_by(PointerButton::Middle)
        || (space_down && response.dragged_by(PointerButton::Primary))
    {
        if space_down {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }
        state.view.offset += response.drag_delta();
    }

    // Mouse wheel pan
    let (modifiers, scroll_delta) = ui.ctx().input(|i| (i.modifiers, i.smooth_scroll_delta));
    if !modifiers.ctrl
        && !modifiers.command
        && (scroll_delta.x.abs() > 0.0 || scroll_delta.y.abs() > 0.0)
    {
        if let Some(cursor) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            if viewport_rect.contains(cursor) {
                if modifiers.shift {
                    let dx = if scroll_delta.x.abs() > 0.0 {
                        scroll_delta.x
                    } else {
                        scroll_delta.y
                    };
                    state.view.offset.x += dx;
                } else {
                    state.view.offset.y += scroll_delta.y;
                    if scroll_delta.x.abs() > 0.0 {
                        state.view.offset.x += scroll_delta.x;
                    }
                }
            }
        }
    }

    // Ctrl+scroll zoom towards cursor
    let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
    if (zoom_delta - 1.0).abs() > 0.001 {
        if let Some(cursor) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            if viewport_rect.contains(cursor) {
                let (new_zoom, new_offset) =
                    zoom_towards_cursor(cursor, state.view.zoom, state.view.offset, zoom_delta);
                state.view.zoom = new_zoom;
                state.view.offset = new_offset;
            }
        }
    }

    // Clamp offset
    state.view.offset = clamp_offset(
        state.view.offset,
        viewport_rect,
        img_w,
        img_h,
        state.view.zoom,
    );

    // 4. Multi-select with Right-Click Drag
    let right_pressed = ui
        .ctx()
        .input(|i| i.pointer.button_pressed(PointerButton::Secondary));
    let right_down = ui
        .ctx()
        .input(|i| i.pointer.button_down(PointerButton::Secondary));
    let right_released = ui
        .ctx()
        .input(|i| i.pointer.button_released(PointerButton::Secondary));

    if right_pressed {
        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
            if viewport_rect.contains(pos) && grid.cols > 0 && grid.rows > 0 {
                let page_pos = state.view.screen_to_page(pos);
                let rel_x = page_pos.x - grid.padding as f32;
                let rel_y = page_pos.y - grid.padding as f32;
                let stride_x = (grid.tile_w + grid.spacing).max(1) as f32;
                let stride_y = (grid.tile_h + grid.spacing).max(1) as f32;
                let col = (rel_x / stride_x)
                    .floor()
                    .clamp(0.0, grid.cols.saturating_sub(1) as f32)
                    as u32;
                let row = (rel_y / stride_y)
                    .floor()
                    .clamp(0.0, grid.rows.saturating_sub(1) as f32)
                    as u32;

                state.drag_start_cell = Some((col, row));
                state.is_right_dragging = true;
                let single_rect = TileRect::single(col, row);
                state.live_drag_rect = Some(single_rect);
                state.selected_tile_rect = Some(single_rect);
                events.push(AtlasEvent::DragRectStart(single_rect));
            }
        }
    }

    if state.is_right_dragging && right_down {
        if let Some(start_cell) = state.drag_start_cell {
            if let Some(curr_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                let edge_margin = 24.0;
                let scroll_step = 6.0;
                let mut auto_scroll = vec2(0.0, 0.0);
                if curr_pos.x < viewport_rect.min.x + edge_margin {
                    auto_scroll.x += scroll_step;
                }
                if curr_pos.x > viewport_rect.max.x - edge_margin {
                    auto_scroll.x -= scroll_step;
                }
                if curr_pos.y < viewport_rect.min.y + edge_margin {
                    auto_scroll.y += scroll_step;
                }
                if curr_pos.y > viewport_rect.max.y - edge_margin {
                    auto_scroll.y -= scroll_step;
                }
                if auto_scroll != vec2(0.0, 0.0) {
                    state.view.offset += auto_scroll;
                    ui.ctx().request_repaint();
                }

                let curr_page = state.view.screen_to_page(curr_pos);
                let rel_x = curr_page.x - grid.padding as f32;
                let rel_y = curr_page.y - grid.padding as f32;
                let stride_x = (grid.tile_w + grid.spacing).max(1) as f32;
                let stride_y = (grid.tile_h + grid.spacing).max(1) as f32;
                let end_col = (rel_x / stride_x)
                    .floor()
                    .clamp(0.0, grid.cols.saturating_sub(1) as f32)
                    as u32;
                let end_row = (rel_y / stride_y)
                    .floor()
                    .clamp(0.0, grid.rows.saturating_sub(1) as f32)
                    as u32;

                let sel = TileRect::from_cells(start_cell, (end_col, end_row))
                    .clamp(grid.cols, grid.rows);
                state.live_drag_rect = Some(sel);
                state.selected_tile_rect = Some(sel);
                events.push(AtlasEvent::DragRectUpdate(sel));
            }
        }
    }

    if state.is_right_dragging && right_released {
        state.is_right_dragging = false;
        state.drag_start_cell = None;
        if let Some(sel) = state.live_drag_rect.take().or(state.selected_tile_rect) {
            state.selected_tile_rect = Some(sel);
            events.push(AtlasEvent::DragRectEnd(sel));
        }
    }

    // 5. Hover Detection & Clicks
    let mut hovered_c_r = None;
    let mut hovered_tid = None;
    if !state.is_right_dragging && !space_down {
        if let Some(mpos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            if viewport_rect.contains(mpos) {
                let page_pos = state.view.screen_to_page(mpos);
                if let Some(cell) = grid.cell_at(page_pos) {
                    hovered_c_r = Some(cell);
                    hovered_tid = Some(grid.tile_id(cell.0, cell.1));
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
            }
        }
    }
    state.hovered_tile = hovered_tid;
    events.push(AtlasEvent::Hovered(state.hovered_tile));

    // Handle clicks
    if !state.is_right_dragging && !space_down {
        if let Some(cell) = hovered_c_r {
            let tid = grid.tile_id(cell.0, cell.1);
            if response.clicked() {
                events.push(AtlasEvent::Clicked {
                    cell,
                    tile_id: tid,
                    button: PointerButton::Primary,
                    modifiers: ui.ctx().input(|i| i.modifiers),
                });
            } else if response.secondary_clicked() {
                events.push(AtlasEvent::Clicked {
                    cell,
                    tile_id: tid,
                    button: PointerButton::Secondary,
                    modifiers: ui.ctx().input(|i| i.modifiers),
                });
            } else if response.middle_clicked() {
                events.push(AtlasEvent::Clicked {
                    cell,
                    tile_id: tid,
                    button: PointerButton::Middle,
                    modifiers: ui.ctx().input(|i| i.modifiers),
                });
            }
        }
    }

    // 6. Paint all visual layers
    let painter = ui.painter().with_clip_rect(viewport_rect);
    paint_canvas_layers(
        ui,
        &painter,
        t,
        state,
        viewport_rect,
        texture,
        page_bg,
        grid,
        img_w,
        img_h,
        hovered_c_r,
        &mut overlay,
        dim,
    );

    (response, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2, Pos2, Rect};

    #[test]
    fn test_compute_fit_view() {
        let viewport = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
        let (zoom, offset) = compute_fit_view(viewport, 400.0, 300.0);
        assert!((zoom - 1.84).abs() < 0.01);
        let zoomed_w = 400.0 * zoom;
        let expected_x = (800.0 - zoomed_w) / 2.0;
        assert!((offset.x - expected_x).abs() <= 1.0);
    }

    #[test]
    fn test_zoom_towards_cursor() {
        let cursor = pos2(200.0, 150.0);
        let old_zoom = 1.0;
        let old_offset = vec2(50.0, 30.0);
        let (new_zoom, new_offset) = zoom_towards_cursor(cursor, old_zoom, old_offset, 2.0);

        assert_eq!(new_zoom, 2.0);
        let page_x_before = (cursor.x - old_offset.x) / old_zoom;
        let page_y_before = (cursor.y - old_offset.y) / old_zoom;
        let page_x_after = (cursor.x - new_offset.x) / new_zoom;
        let page_y_after = (cursor.y - new_offset.y) / new_zoom;

        assert!((page_x_before - page_x_after).abs() <= 1.0);
        assert!((page_y_before - page_y_after).abs() <= 1.0);
    }

    #[test]
    fn test_tile_hit_testing_zooms_and_pans() {
        let grid = TileGrid::new(16, 16, 0, 0, 10, 10);

        let hit = tile_at_screen_pos(pos2(20.0, 20.0), 1.0, Vec2::ZERO, &grid);
        assert_eq!(hit, Some((1, 1)));

        let hit2 = tile_at_screen_pos(pos2(140.0, 140.0), 2.0, vec2(100.0, 100.0), &grid);
        assert_eq!(hit2, Some((1, 1)));

        let hit_outside = tile_at_screen_pos(pos2(500.0, 500.0), 1.0, Vec2::ZERO, &grid);
        assert_eq!(hit_outside, None);
    }
}
