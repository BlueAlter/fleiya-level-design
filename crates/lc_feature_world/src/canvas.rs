//! Canvas rendering and pointer interaction for the world overview tab.

use egui::{
    vec2, Align2, CursorIcon, FontFamily, FontId, Key, PointerButton, Rect, Sense, Stroke, Ui,
};
use glam::IVec2;
use lc_canvas::grid::draw_grid;
use lc_canvas::theme::{self, Tokens};
use lc_core::commands::MoveLevelCommand;
use lc_core::id::LevelId;
use lc_core::level::Level;
use lc_core::world::{World, WorldLayout};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::selection::Selection;
use lc_ui_api::viewport::Viewport;

use crate::commands::SetLevelPositionsCommand;
use crate::layout::{arrange_linear, linear_order, pick_level_at, snap_position, LINEAR_SPACING};
use crate::view::{LevelDrag, WorldViewState};

/// Renders the world canvas and dispatches pointer input for the active depth.
pub fn render_world_canvas(
    ui: &mut Ui,
    world: &World,
    state: &mut WorldViewState,
    grid: IVec2,
    cx: &mut EditorCtx,
) {
    let t = Tokens::get(ui.ctx());
    let rect = ui.available_rect_before_wrap();
    state.viewport.screen_rect = rect;

    let response = ui.allocate_rect(rect, Sense::click_and_drag());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 0.0, t.canvas);

    handle_pan_and_zoom(ui, &response, state);

    if (matches!(world.layout, WorldLayout::GridVania) || state.grid_snap) && grid.x == grid.y {
        draw_grid(&painter, rect, &state.viewport, grid.x, &t);
    }

    let pointer_world = response
        .hover_pos()
        .map(|pos| state.viewport.screen_to_world(pos));
    let press_origin = ui.input(|i| i.pointer.press_origin());
    let hovered =
        pointer_world.and_then(|pos| pick_level_at(&world.levels, state.active_depth, pos));
    let is_panning = (ui.input(|i| i.key_down(Key::Space))
        && response.dragged_by(PointerButton::Primary))
        || response.dragged_by(PointerButton::Middle);

    update_drag(
        world,
        state,
        grid,
        &response,
        pointer_world,
        press_origin,
        hovered,
        is_panning,
        cx,
    );

    draw_levels(&painter, rect, world, state, &t);

    if !is_panning {
        if hovered.is_some() {
            let icon = if state.drag.is_some() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            };
            ui.ctx().set_cursor_icon(icon);
        }

        if response.double_clicked() {
            if let Some(level_id) = hovered {
                cx.emit(EditorEvent::SetActiveLevel(Some(level_id)));
                cx.emit(EditorEvent::SelectionChanged(Selection::Level(level_id)));
                cx.emit(EditorEvent::OpenDocument(lc_core::command::DocKind::Level(
                    level_id,
                )));
            }
        } else if response.clicked() {
            match hovered {
                Some(level_id) => select_level(state, cx, level_id),
                None => clear_selection(state, cx),
            }
        }

        if response.drag_stopped() {
            if let Some(drag) = state.drag.take() {
                if drag.moved {
                    commit_drag(world, state, drag, cx);
                }
            }
        }
    }

    draw_drag_preview(&painter, world, state, &t);
    draw_footer(&painter, rect, world, state, &t);
}

fn handle_pan_and_zoom(ui: &mut Ui, response: &egui::Response, state: &mut WorldViewState) {
    let is_space = ui.input(|i| i.key_down(Key::Space));
    let middle_down = response.dragged_by(PointerButton::Middle);
    let primary_drag = response.dragged_by(PointerButton::Primary);
    let is_panning = middle_down || (is_space && primary_drag);

    if is_panning {
        state.viewport.pan_by(response.drag_delta());
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if is_space && response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    if response.hovered() {
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_y != 0.0 {
            let factor = if scroll_y > 0.0 { 1.15 } else { 1.0 / 1.15 };
            if let Some(mouse_pos) = response.hover_pos() {
                state.viewport.zoom_at(factor, mouse_pos);
            }
        }
    }
}

// Drag updates need the world/state/pointer inputs together; splitting them
// into a struct would only move the parameter list without simplifying it.
#[allow(clippy::too_many_arguments)]
fn update_drag(
    world: &World,
    state: &mut WorldViewState,
    grid: IVec2,
    response: &egui::Response,
    pointer_world: Option<IVec2>,
    press_origin: Option<egui::Pos2>,
    hovered: Option<LevelId>,
    is_panning: bool,
    cx: &mut EditorCtx,
) {
    if is_panning {
        state.drag = None;
        return;
    }

    if response.drag_started_by(PointerButton::Primary) {
        let start_level = press_origin
            .map(|pos| {
                let world_pos = state.viewport.screen_to_world(pos);
                pick_level_at(&world.levels, state.active_depth, world_pos)
            })
            .unwrap_or(hovered);
        if let Some(level_id) = start_level {
            if let Some(level) = world.level(level_id) {
                let start_world = press_origin
                    .map(|pos| state.viewport.screen_to_world(pos))
                    .or(pointer_world)
                    .unwrap_or(level.world_pos);
                select_level(state, cx, level_id);
                state.drag = Some(LevelDrag {
                    level_id,
                    start_world,
                    original_pos: level.world_pos,
                    preview_pos: level.world_pos,
                    moved: false,
                });
            }
        }
    }

    let grid_snap = state.grid_snap;
    let layout = world.layout;
    let Some(drag) = state.drag.as_mut() else {
        return;
    };
    if !response.dragged_by(PointerButton::Primary) {
        return;
    }
    let Some(pointer) = pointer_world else {
        return;
    };

    let delta = pointer - drag.start_world;
    if delta != IVec2::ZERO {
        drag.moved = true;
    }
    drag.preview_pos = snap_position(layout, grid, drag.original_pos + delta, grid_snap);
}

fn commit_drag(world: &World, state: &WorldViewState, drag: LevelDrag, cx: &mut EditorCtx) {
    match world.layout {
        WorldLayout::Free | WorldLayout::GridVania => {
            if drag.preview_pos == drag.original_pos {
                return;
            }
            let depth = world
                .level(drag.level_id)
                .map(|level| level.world_depth)
                .unwrap_or(state.active_depth);
            cx.exec(MoveLevelCommand::new(
                drag.level_id,
                drag.preview_pos,
                depth,
            ));
            if let Some(level) = world.level(drag.level_id) {
                cx.status(format!(
                    "Moved {} to {}, {}",
                    level.name, drag.preview_pos.x, drag.preview_pos.y
                ));
            }
        }
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical => {
            let order = linear_order(&world.levels, world.layout, drag.level_id, drag.preview_pos);
            let positions = arrange_linear(&world.levels, &order, world.layout, LINEAR_SPACING);
            if positions.is_empty() {
                return;
            }
            cx.exec(SetLevelPositionsCommand::new(world.id, positions));
            if let Some(level) = world.level(drag.level_id) {
                cx.status(format!("Reordered {}", level.name));
            }
        }
    }
}

fn draw_levels(
    painter: &egui::Painter,
    canvas_rect: Rect,
    world: &World,
    state: &WorldViewState,
    t: &Tokens,
) {
    let mut visible: Vec<&Level> = world
        .levels
        .iter()
        .filter(|level| level_screen_rect(level, &state.viewport).intersects(canvas_rect))
        .collect();
    // Inactive depths first so the active depth paints on top of overlapping levels.
    visible.sort_by_key(|level| level.world_depth == state.active_depth);

    for level in visible {
        let screen_rect = level_screen_rect(level, &state.viewport);
        let is_active = level.world_depth == state.active_depth;
        let is_selected = state.selected_level == Some(level.id);

        let fill = if is_active {
            t.sunken
        } else {
            theme::tint(t.sunken, if t.dark { 0.55 } else { 0.45 })
        };
        painter.rect_filled(screen_rect, 0.0, fill);

        let stroke = if is_selected {
            Stroke::new(1.5, t.accent)
        } else if is_active {
            Stroke::new(1.0, t.line_strong)
        } else {
            Stroke::new(1.0, t.line)
        };
        painter.rect_stroke(screen_rect, 0.0, stroke, egui::StrokeKind::Inside);

        if screen_rect.height() >= 16.0 && screen_rect.width() >= 24.0 {
            let clip = screen_rect.intersect(canvas_rect);
            let label_painter = painter.with_clip_rect(clip);
            let ink = if is_active { t.ink } else { t.faint };
            label_painter.text(
                screen_rect.min + vec2(5.0, 4.0),
                Align2::LEFT_TOP,
                &level.name,
                FontId::new(11.0, FontFamily::Monospace),
                ink,
            );
            if screen_rect.height() >= 30.0 {
                label_painter.text(
                    screen_rect.left_bottom() + vec2(5.0, -4.0),
                    Align2::LEFT_BOTTOM,
                    format!("D{}", level.world_depth),
                    FontId::new(10.0, FontFamily::Monospace),
                    if is_active { t.muted } else { t.faint },
                );
            }
        }
    }
}

fn draw_drag_preview(painter: &egui::Painter, world: &World, state: &WorldViewState, t: &Tokens) {
    let Some(drag) = state.drag else {
        return;
    };
    if !drag.moved {
        return;
    }
    let Some(level) = world.level(drag.level_id) else {
        return;
    };

    let min = state.viewport.world_to_screen(drag.preview_pos);
    let max = state.viewport.world_to_screen(
        drag.preview_pos + IVec2::new(level.size_px.x.max(1) as i32, level.size_px.y.max(1) as i32),
    );
    let ghost = Rect::from_two_pos(min, max);
    painter.rect_filled(ghost, 0.0, theme::tint(t.accent, 0.16));
    painter.rect_stroke(
        ghost,
        0.0,
        Stroke::new(1.5, t.accent),
        egui::StrokeKind::Inside,
    );

    if matches!(
        world.layout,
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical
    ) {
        let line_stroke = Stroke::new(1.0, t.accent);
        if matches!(world.layout, WorldLayout::LinearHorizontal) {
            painter.vline(ghost.left(), ghost.y_range(), line_stroke);
        } else {
            painter.hline(ghost.x_range(), ghost.top(), line_stroke);
        }
    }
}

fn draw_footer(
    painter: &egui::Painter,
    canvas_rect: Rect,
    world: &World,
    state: &WorldViewState,
    t: &Tokens,
) {
    let active = world
        .levels
        .iter()
        .filter(|level| level.world_depth == state.active_depth)
        .count();
    let text = format!(
        "{} levels · {} active · depth {} · {}",
        world.levels.len(),
        active,
        state.active_depth,
        layout_name(world.layout)
    );
    painter.text(
        canvas_rect.left_bottom() + vec2(12.0, -10.0),
        Align2::LEFT_BOTTOM,
        text,
        FontId::new(11.0, FontFamily::Monospace),
        t.faint,
    );
}

fn layout_name(layout: WorldLayout) -> &'static str {
    match layout {
        WorldLayout::Free => "Free",
        WorldLayout::GridVania => "GridVania",
        WorldLayout::LinearHorizontal => "Linear H",
        WorldLayout::LinearVertical => "Linear V",
    }
}

/// Screen-space rectangle covered by a level at the current zoom and pan.
pub fn level_screen_rect(level: &Level, viewport: &Viewport) -> Rect {
    let min = viewport.world_to_screen(level.world_pos);
    let max = viewport.world_to_screen(
        level.world_pos + IVec2::new(level.size_px.x.max(1) as i32, level.size_px.y.max(1) as i32),
    );
    Rect::from_two_pos(min, max)
}

fn select_level(state: &mut WorldViewState, cx: &mut EditorCtx, level_id: LevelId) {
    state.selected_level = Some(level_id);
    cx.emit(EditorEvent::SelectionChanged(Selection::Level(level_id)));
    cx.emit(EditorEvent::SetActiveLevel(Some(level_id)));
}

fn clear_selection(state: &mut WorldViewState, cx: &mut EditorCtx) {
    if state.selected_level.is_none() {
        return;
    }
    state.selected_level = None;
    cx.emit(EditorEvent::SelectionChanged(Selection::None));
}
