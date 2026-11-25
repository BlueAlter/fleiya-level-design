//! Canvas input handling and shape manipulation for collision focus mode.

use egui::{Key, PointerButton, Response, Ui};
use glam::Vec2;
use lc_core::collision::{CollisionShape, CollisionShapeGeometry, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_feature_tileset::collision_commands::{RemoveTileCollisionCommand, SetTileCollisionCommand};

use super::collision_overlay::{local_to_screen, screen_to_local, snap_local_point};
use super::state::{CollisionTool, TilesetViewState};
use crate::shared::ui::atlas_canvas::PickerView;
use crate::shared::ui::tile_bounds::TileGrid;

/// Checks if a 2D point lies within an axis-aligned rectangle.
pub fn point_in_rect(p: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    let (min_x, max_x) = if w >= 0.0 { (x, x + w) } else { (x + w, x) };
    let (min_y, max_y) = if h >= 0.0 { (y, y + h) } else { (y + h, y) };
    p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y
}

/// Checks if a 2D point lies within a polygon boundary using ray-casting.
pub fn point_in_polygon(p: Vec2, points: &[Vec2]) -> bool {
    if points.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let pi = points[i];
        let pj = points[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Checks if a 2D point lies within a circular boundary.
pub fn point_in_circle(p: Vec2, cx: f32, cy: f32, radius: f32) -> bool {
    (p - Vec2::new(cx, cy)).length_squared() <= radius * radius
}

/// Hit-tests all collision shapes in reverse order, returning the top-most shape ID.
pub fn hit_test_shapes(local_pos: Vec2, col_def: &TileCollisionDef) -> Option<u32> {
    for shape in col_def.shapes.iter().rev() {
        let hit = match &shape.geometry {
            CollisionShapeGeometry::Rectangle { x, y, w, h } => {
                point_in_rect(local_pos, *x, *y, *w, *h)
            }
            CollisionShapeGeometry::Polygon { points } => point_in_polygon(local_pos, points),
            CollisionShapeGeometry::Circle { cx, cy, radius } => {
                point_in_circle(local_pos, *cx, *cy, *radius)
            }
        };
        if hit {
            return Some(shape.id);
        }
    }
    None
}

/// Handles pointer and keyboard interaction on the atlas canvas in collision focus mode.
pub fn handle_collision_focus_interaction(
    ui: &mut Ui,
    response: &Response,
    tileset: &TilesetDef,
    grid: &TileGrid,
    canvas_view: PickerView,
    state: &mut TilesetViewState,
) -> Vec<Box<dyn lc_core::command::Command>> {
    let mut commands: Vec<Box<dyn lc_core::command::Command>> = Vec::new();

    let focus_tile_id = match state.collision_focus_tile {
        Some(id) => id,
        None => return commands,
    };

    if grid.cols == 0 || grid.rows == 0 {
        return commands;
    }

    let col = focus_tile_id % grid.cols;
    let row = focus_tile_id / grid.cols;
    let tile_page = grid.cell_rect(col, row);
    let tile_screen = canvas_view.page_to_screen_rect(tile_page);

    let pointer_pos = ui.ctx().input(|i| i.pointer.hover_pos());
    let primary_clicked = response.clicked_by(PointerButton::Primary);
    let primary_down = ui.ctx().input(|i| i.pointer.primary_down());
    let double_clicked = response.double_clicked_by(PointerButton::Primary);

    // Update live cursor tracked position
    if let Some(pos) = pointer_pos {
        if tile_screen.contains(pos) || state.collision_drag_start.is_some() {
            let local_raw = screen_to_local(pos, tile_screen, tileset.tile_size);
            let snapped = snap_local_point(local_raw, tileset.tile_size, state.collision_snap);
            state.collision_drag_current = Some(snapped);
        }
    }

    // 1. Tool-specific interaction
    match state.collision_tool {
        CollisionTool::Box => {
            if let Some(pos) = pointer_pos {
                if tile_screen.contains(pos) && primary_down && state.collision_drag_start.is_none()
                {
                    let local_raw = screen_to_local(pos, tile_screen, tileset.tile_size);
                    let snapped =
                        snap_local_point(local_raw, tileset.tile_size, state.collision_snap);
                    state.collision_drag_start = Some(snapped);
                    state.collision_drag_current = Some(snapped);
                }
            }

            // Drag release commits new box
            if !primary_down {
                if let Some(start) = state.collision_drag_start.take() {
                    let curr = state.collision_drag_current.unwrap_or(start);
                    let min_x = start.x.min(curr.x);
                    let min_y = start.y.min(curr.y);
                    let w = (start.x - curr.x).abs();
                    let h = (start.y - curr.y).abs();

                    if w >= 1.0 && h >= 1.0 {
                        let mut col_def = tileset
                            .collisions
                            .get(&focus_tile_id)
                            .cloned()
                            .unwrap_or_else(TileCollisionDef::new);
                        let shape_id = col_def.next_shape_id();
                        let new_shape = CollisionShape::new_rect(shape_id, min_x, min_y, w, h);
                        col_def.shapes.push(new_shape);

                        commands.push(Box::new(SetTileCollisionCommand::new(
                            tileset.id,
                            focus_tile_id,
                            col_def,
                        )));
                        state.selected_collision_shape_id = Some(shape_id);
                    }
                }
            }
        }
        CollisionTool::Polygon => {
            if double_clicked && state.collision_poly_draft.len() >= 3 {
                close_polygon_draft(focus_tile_id, tileset, state, &mut commands);
            } else if primary_clicked {
                if let Some(pos) = pointer_pos {
                    if tile_screen.contains(pos) {
                        let local_raw = screen_to_local(pos, tile_screen, tileset.tile_size);
                        let snapped =
                            snap_local_point(local_raw, tileset.tile_size, state.collision_snap);

                        // If clicking near first draft point and >= 3 points, close loop
                        if state.collision_poly_draft.len() >= 3 {
                            let first_screen = local_to_screen(
                                state.collision_poly_draft[0],
                                tile_screen,
                                tileset.tile_size,
                            );
                            if pos.distance(first_screen) < 12.0 {
                                close_polygon_draft(focus_tile_id, tileset, state, &mut commands);
                                return commands;
                            }
                        }

                        state.collision_poly_draft.push(snapped);
                    }
                }
            }
        }
        CollisionTool::Select => {
            if primary_clicked {
                if let Some(pos) = pointer_pos {
                    if tile_screen.contains(pos) {
                        let local_raw = screen_to_local(pos, tile_screen, tileset.tile_size);
                        if let Some(col_def) = tileset.collisions.get(&focus_tile_id) {
                            state.selected_collision_shape_id = hit_test_shapes(local_raw, col_def);
                        } else {
                            state.selected_collision_shape_id = None;
                        }
                    }
                }
            }
        }
    }

    // 2. Keyboard shortcuts in focus mode (when no text widget has focus)
    if !ui.ctx().egui_wants_keyboard_input() {
        // Enter closes polygon draft if >= 3 vertices
        let enter_pressed = ui
            .ctx()
            .input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        if enter_pressed && state.collision_poly_draft.len() >= 3 {
            close_polygon_draft(focus_tile_id, tileset, state, &mut commands);
        }

        // Delete / Backspace removes selected collision shape
        let delete_pressed = ui.ctx().input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Delete)
                || i.consume_key(egui::Modifiers::NONE, Key::Backspace)
        });
        if delete_pressed {
            if let Some(shape_id) = state.selected_collision_shape_id.take() {
                if let Some(mut col_def) = tileset.collisions.get(&focus_tile_id).cloned() {
                    col_def.shapes.retain(|s| s.id != shape_id);
                    if col_def.shapes.is_empty() {
                        commands.push(Box::new(RemoveTileCollisionCommand::new(
                            tileset.id,
                            focus_tile_id,
                        )));
                    } else {
                        commands.push(Box::new(SetTileCollisionCommand::new(
                            tileset.id,
                            focus_tile_id,
                            col_def,
                        )));
                    }
                }
            }
        }

        // Escape cancels draft if active, or exits focus mode
        let esc_pressed = ui
            .ctx()
            .input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        if esc_pressed {
            if !state.collision_poly_draft.is_empty() {
                state.collision_poly_draft.clear();
            } else {
                state.exit_collision_focus();
            }
        }
    }

    commands
}

/// Commits the in-progress polygon draft to the active tile collision definition.
fn close_polygon_draft(
    focus_tile_id: u32,
    tileset: &TilesetDef,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn lc_core::command::Command>>,
) {
    if state.collision_poly_draft.len() < 3 {
        return;
    }

    let mut col_def = tileset
        .collisions
        .get(&focus_tile_id)
        .cloned()
        .unwrap_or_else(TileCollisionDef::new);
    let shape_id = col_def.next_shape_id();
    let points = std::mem::take(&mut state.collision_poly_draft);
    let new_shape = CollisionShape::new_polygon(shape_id, points);
    col_def.shapes.push(new_shape);

    commands.push(Box::new(SetTileCollisionCommand::new(
        tileset.id,
        focus_tile_id,
        col_def,
    )));
    state.selected_collision_shape_id = Some(shape_id);
}
