//! World overview document tab: level placement, depth selection, and layouts.

use egui::{vec2, Frame, Margin, Rect, Ui};
use glam::IVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::controls::icon_button;
use lc_canvas::widgets::hud;
use lc_core::command::DocKind;
use lc_core::commands::{AddLevelCommand, MoveLevelCommand};
use lc_core::id::{LevelId, WorldId};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::{World, WorldLayout};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::selection::Selection;
use lc_ui_api::view::DocumentView;
use lc_ui_api::viewport::Viewport;

use crate::canvas::render_world_canvas;
use crate::commands::{
    DuplicateLevelCommand, SetLevelPositionsCommand, SetWorldLayoutCommand, WorldScopedCommand,
};
use crate::helpers::{level_name, next_level_id};
use crate::layout::{
    arrange_linear, axis_order, default_level_size, depths, snap_position, world_bounds,
    world_grid, LINEAR_SPACING,
};

/// In-progress drag of a level rectangle in the world canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelDrag {
    /// Level being dragged.
    pub level_id: LevelId,
    /// World position where the drag started.
    pub start_world: IVec2,
    /// Stored position before the drag.
    pub original_pos: IVec2,
    /// Snapped preview position shown under the pointer.
    pub preview_pos: IVec2,
    /// Whether the pointer moved far enough to count as a drag.
    pub moved: bool,
}

/// Transient view state for a world overview tab.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldViewState {
    /// Camera pan and zoom.
    pub viewport: Viewport,
    /// Whether the initial fit-to-world happened.
    pub initialized: bool,
    /// Depth that is fully coloured and interactive.
    pub active_depth: i32,
    /// Level selected in this world tab.
    pub selected_level: Option<LevelId>,
    /// Whether free-layout drags snap to the world grid.
    pub grid_snap: bool,
    /// Active drag gesture, if any.
    pub drag: Option<LevelDrag>,
}

impl Default for WorldViewState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            initialized: false,
            active_depth: 0,
            selected_level: None,
            grid_snap: true,
            drag: None,
        }
    }
}

impl WorldViewState {
    /// Fits the viewport to the bounds of all levels in `world`.
    pub fn fit_to_world(&mut self, world: &World, canvas_rect: Rect) {
        if canvas_rect.width() <= 0.0 || canvas_rect.height() <= 0.0 {
            return;
        }

        let (min, max) = world_bounds(world).unwrap_or((IVec2::ZERO, IVec2::splat(512)));
        let width = (max.x.saturating_sub(min.x)).max(1) as f32;
        let height = (max.y.saturating_sub(min.y)).max(1) as f32;

        let pad_w = (canvas_rect.width() - 96.0).max(32.0);
        let pad_h = (canvas_rect.height() - 96.0).max(32.0);
        let zoom = (pad_w / width)
            .min(pad_h / height)
            .clamp(self.viewport.min_zoom, 1.5);
        self.viewport.set_zoom(zoom);

        let center = vec2(
            (min.x as f32 + max.x as f32) * 0.5,
            (min.y as f32 + max.y as f32) * 0.5,
        );
        self.viewport.pan =
            (canvas_rect.center() - canvas_rect.min) - vec2(center.x, center.y) * zoom;
        self.viewport.screen_rect = canvas_rect;
        self.initialized = true;
    }
}

/// Document tab view for a world's level placement overview.
pub struct WorldDocumentView {
    world_id: WorldId,
    state: WorldViewState,
}

impl WorldDocumentView {
    /// Creates a world document view for the given world ID.
    pub fn new(world_id: WorldId) -> Self {
        Self {
            world_id,
            state: WorldViewState::default(),
        }
    }

    /// Target world ID.
    pub fn world_id(&self) -> WorldId {
        self.world_id
    }

    /// Read-only access to transient view state.
    pub fn state(&self) -> &WorldViewState {
        &self.state
    }

    /// Mutable access to transient view state.
    pub fn state_mut(&mut self) -> &mut WorldViewState {
        &mut self.state
    }
}

impl DocumentView for WorldDocumentView {
    fn title(&self, p: &Project) -> String {
        p.world(self.world_id)
            .map(|world| world.name.clone())
            .unwrap_or_else(|| "World".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.world(self.world_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::World(self.world_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let Some(world) = cx.project().world(self.world_id).cloned() else {
            ui.centered_and_justified(|ui| {
                ui.label("World not found in project");
            });
            return;
        };

        let grid = world_grid(&world, cx.project().settings.default_grid_size);
        let canvas_rect = ui.available_rect_before_wrap();

        if !self.state.initialized {
            self.state.fit_to_world(&world, canvas_rect);
            if let Some(first) = world.levels.first() {
                self.state.active_depth = first.world_depth;
            }
        }

        render_world_canvas(ui, &world, &mut self.state, grid, cx);
        render_toolbar(ui, &world, &mut self.state, grid, canvas_rect, cx);
    }
}

fn render_toolbar(
    ui: &mut Ui,
    world: &World,
    state: &mut WorldViewState,
    grid: IVec2,
    canvas_rect: Rect,
    cx: &mut EditorCtx,
) {
    let t = Tokens::get(ui.ctx());
    let position = canvas_rect.min + vec2(12.0, 12.0);

    egui::Area::new(ui.id().with("world_toolbar"))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if icon_button(ui, Icon::ZoomIn, "Zoom In").clicked() {
                            state.viewport.zoom_at(1.25, canvas_rect.center());
                        }
                        if icon_button(ui, Icon::ZoomOut, "Zoom Out").clicked() {
                            state.viewport.zoom_at(1.0 / 1.25, canvas_rect.center());
                        }
                        if icon_button(ui, Icon::Fit, "Fit world to canvas").clicked() {
                            state.fit_to_world(world, canvas_rect);
                        }
                        hud(
                            ui,
                            &format!("{:.0}%", state.viewport.zoom * 100.0),
                            t.ink_strong,
                        );
                        ui.separator();
                        layout_combo(ui, world, cx);
                        ui.separator();
                        depth_combo(ui, world, state);
                        if icon_button(ui, Icon::ChevronDown, "Previous depth").clicked() {
                            set_active_depth(world, state, state.active_depth.saturating_sub(1));
                        }
                        if icon_button(ui, Icon::ChevronUp, "Next depth").clicked() {
                            set_active_depth(world, state, state.active_depth.saturating_add(1));
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal_wrapped(|ui| {
                        if icon_button(ui, Icon::Add, "Create a level at the view centre").clicked()
                        {
                            create_level(world, state, grid, canvas_rect, cx);
                        }
                        if icon_button(ui, Icon::Duplicate, "Duplicate the selected level")
                            .clicked()
                        {
                            duplicate_selected(world, state, grid, cx);
                        }
                        if icon_button(ui, Icon::Grid, "Tidy levels for the current layout")
                            .clicked()
                        {
                            tidy_levels(world, state, grid, cx);
                        }
                        if icon_button(ui, Icon::Move, "Move selected level to the active depth")
                            .clicked()
                        {
                            move_selected_to_depth(world, state, cx);
                        }
                        ui.separator();
                        hud(
                            ui,
                            &format!(
                                "{} levels · depth {}",
                                world.levels.len(),
                                state.active_depth
                            ),
                            t.faint,
                        );
                    });
                });
        });
}

fn layout_combo(ui: &mut Ui, world: &World, cx: &mut EditorCtx) {
    let current = world.layout;
    egui::ComboBox::from_id_salt("world_layout_combo")
        .selected_text(layout_label(current))
        .width(118.0)
        .show_ui(ui, |ui| {
            for candidate in [
                WorldLayout::Free,
                WorldLayout::GridVania,
                WorldLayout::LinearHorizontal,
                WorldLayout::LinearVertical,
            ] {
                if ui
                    .selectable_label(current == candidate, layout_label(candidate))
                    .clicked()
                {
                    if candidate != current {
                        cx.exec(SetWorldLayoutCommand::new(world.id, candidate));
                        if matches!(
                            candidate,
                            WorldLayout::LinearHorizontal | WorldLayout::LinearVertical
                        ) {
                            let order = axis_order(&world.levels, candidate);
                            let positions =
                                arrange_linear(&world.levels, &order, candidate, LINEAR_SPACING);
                            if !positions.is_empty() {
                                cx.exec(SetLevelPositionsCommand::new(world.id, positions));
                            }
                        }
                    }
                    ui.close();
                }
            }
        });
}

fn depth_combo(ui: &mut Ui, world: &World, state: &mut WorldViewState) {
    let options = depths(world, state.active_depth);
    egui::ComboBox::from_id_salt("world_depth_combo")
        .selected_text(format!("Depth {}", state.active_depth))
        .width(110.0)
        .show_ui(ui, |ui| {
            for depth in options {
                let count = world
                    .levels
                    .iter()
                    .filter(|level| level.world_depth == depth)
                    .count();
                if ui
                    .selectable_label(
                        depth == state.active_depth,
                        format!("Depth {depth} ({count})"),
                    )
                    .clicked()
                {
                    set_active_depth(world, state, depth);
                    ui.close();
                }
            }
        });
}

fn set_active_depth(world: &World, state: &mut WorldViewState, depth: i32) {
    state.active_depth = depth;
    state.drag = None;
    if let Some(selected) = state.selected_level {
        if world.level(selected).map(|level| level.world_depth) != Some(depth) {
            state.selected_level = None;
        }
    }
}

fn create_level(
    world: &World,
    state: &mut WorldViewState,
    grid: IVec2,
    canvas_rect: Rect,
    cx: &mut EditorCtx,
) {
    let id = next_level_id(cx.project());
    let size = default_level_size(grid);
    let center = state.viewport.screen_to_world(canvas_rect.center());
    let top_left = center - IVec2::new(size.x as i32 / 2, size.y as i32 / 2);

    let mut level = Level::new(id, level_name(id), size);
    level.world_pos = snap_position(world.layout, grid, top_left, state.grid_snap);
    level.world_depth = state.active_depth;

    cx.exec(WorldScopedCommand::new(
        AddLevelCommand::new(world.id, level),
        world.id,
    ));
    state.selected_level = Some(id);
    cx.emit(EditorEvent::SelectionChanged(Selection::Level(id)));
    cx.emit(EditorEvent::SetActiveLevel(Some(id)));
    cx.status(format!("Created {}", level_name(id)));
}

fn duplicate_selected(world: &World, state: &mut WorldViewState, grid: IVec2, cx: &mut EditorCtx) {
    let Some(source_id) = state.selected_level else {
        cx.status("Select a level to duplicate");
        return;
    };
    let Some(source) = world.level(source_id) else {
        cx.status("Select a level to duplicate");
        return;
    };

    let id = next_level_id(cx.project());
    let offset = IVec2::new(grid.x.max(16), grid.y.max(16));
    cx.exec(DuplicateLevelCommand::new(
        world.id,
        source_id,
        id,
        format!("{}_copy", source.name),
        offset,
    ));
    state.selected_level = Some(id);
    cx.emit(EditorEvent::SelectionChanged(Selection::Level(id)));
    cx.emit(EditorEvent::SetActiveLevel(Some(id)));
    cx.status(format!("Duplicated {} as {}", source.name, level_name(id)));
}

fn tidy_levels(world: &World, state: &WorldViewState, grid: IVec2, cx: &mut EditorCtx) {
    let positions = match world.layout {
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical => {
            let order = axis_order(&world.levels, world.layout);
            arrange_linear(&world.levels, &order, world.layout, LINEAR_SPACING)
        }
        WorldLayout::GridVania => snap_all(world, grid, true),
        WorldLayout::Free => {
            if state.grid_snap {
                snap_all(world, grid, true)
            } else {
                Vec::new()
            }
        }
    };

    if positions.is_empty() {
        cx.status("Nothing to tidy");
        return;
    }
    cx.exec(SetLevelPositionsCommand::new(world.id, positions));
    cx.status("Tidied level positions");
}

fn snap_all(world: &World, grid: IVec2, grid_snap: bool) -> Vec<(LevelId, IVec2)> {
    world
        .levels
        .iter()
        .filter_map(|level| {
            let pos = snap_position(world.layout, grid, level.world_pos, grid_snap);
            (pos != level.world_pos).then_some((level.id, pos))
        })
        .collect()
}

fn move_selected_to_depth(world: &World, state: &WorldViewState, cx: &mut EditorCtx) {
    let Some(id) = state.selected_level else {
        cx.status("Select a level to move");
        return;
    };
    let Some(level) = world.level(id) else {
        cx.status("Select a level to move");
        return;
    };
    if level.world_depth == state.active_depth {
        cx.status(format!(
            "{} is already on depth {}",
            level.name, state.active_depth
        ));
        return;
    }

    cx.exec(WorldScopedCommand::new(
        MoveLevelCommand::new(id, level.world_pos, state.active_depth),
        world.id,
    ));
    cx.status(format!(
        "Moved {} to depth {}",
        level.name, state.active_depth
    ));
}

fn layout_label(layout: WorldLayout) -> &'static str {
    match layout {
        WorldLayout::Free => "Layout: Free",
        WorldLayout::GridVania => "Layout: GridVania",
        WorldLayout::LinearHorizontal => "Layout: Linear H",
        WorldLayout::LinearVertical => "Layout: Linear V",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;

    #[test]
    fn fit_to_world_frames_every_level() {
        let mut world = World::new(WorldId::new(1), "W");
        let mut a = Level::new(LevelId::new(1), "A", UVec2::new(100, 100));
        a.world_pos = IVec2::new(0, 0);
        let mut b = Level::new(LevelId::new(2), "B", UVec2::new(100, 100));
        b.world_pos = IVec2::new(900, 400);
        world.levels.push(a);
        world.levels.push(b);

        let mut state = WorldViewState::default();
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(800.0, 600.0));
        state.fit_to_world(&world, rect);

        assert!(state.initialized);
        assert!(state.viewport.zoom > 0.0);
        let (min, max) = state.viewport.visible_world_bounds();
        assert!(min.x <= 0 && min.y <= 0);
        assert!(max.x >= 1000 && max.y >= 500);
    }
}
