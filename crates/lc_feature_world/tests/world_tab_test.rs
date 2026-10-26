//! Integration tests for the world overview document tab.

use egui::{vec2, Event, Pos2, RawInput, Rect, Vec2};
use glam::{IVec2, UVec2};
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::id::{LevelId, WorldId};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::{World, WorldLayout};
use lc_feature_world::{level_screen_rect, register, WorldDocumentView};
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::selection::Selection;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;

const SCREEN: Vec2 = Vec2::new(800.0, 600.0);

fn screen() -> Rect {
    Rect::from_min_size(Pos2::ZERO, SCREEN)
}

fn project_with_levels(levels: Vec<Level>) -> (Project, WorldId) {
    let world_id = WorldId::new(1);
    let mut world = World::new(world_id, "Overworld");
    world.levels = levels;
    let mut project = Project::new("World Tab");
    project.worlds.push(world);
    (project, world_id)
}

fn level(id: u64, pos: IVec2, size: UVec2, depth: i32) -> Level {
    let mut level = Level::new(LevelId::new(id), format!("Level_{id}"), size);
    level.world_pos = pos;
    level.world_depth = depth;
    level
}

fn press(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    }
}

struct Harness {
    ctx: egui::Context,
    view: WorldDocumentView,
    project: Project,
    events: Vec<EditorEvent>,
    pending: Vec<Box<dyn Command>>,
    time: f64,
}

impl Harness {
    fn new(project: Project, world_id: WorldId) -> Self {
        let mut harness = Self {
            ctx: egui::Context::default(),
            view: WorldDocumentView::new(world_id),
            project,
            events: Vec::new(),
            pending: Vec::new(),
            time: 0.0,
        };
        // First frame initializes the fit-to-world camera.
        harness.frame(Vec::new());
        harness
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.05;
        let undo = UndoStack::new();
        let session = SessionState::default();
        let assets = NoopAssetService;
        let mut raw = RawInput {
            screen_rect: Some(screen()),
            time: Some(self.time),
            ..Default::default()
        };
        raw.events = events;

        let events_out = &mut self.events;
        let pending = &mut self.pending;
        let project = &self.project;
        let view = &mut self.view;
        let mut output = self.ctx.run_ui(raw, |ui| {
            let mut cx = EditorCtx::new(project, &undo, &session, events_out, pending, &assets);
            view.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    fn center_of(&self, level_id: LevelId) -> Pos2 {
        let level = self.project.level(level_id).expect("level exists");
        level_screen_rect(level, &self.view.state().viewport).center()
    }

    fn click(&mut self, pos: Pos2) {
        self.frame(vec![Event::PointerMoved(pos)]);
        self.frame(vec![Event::PointerMoved(pos), press(pos, true)]);
        self.frame(vec![press(pos, false)]);
    }

    /// Advances input time past the double-click window without emitting events.
    fn settle(&mut self) {
        self.time += 1.0;
    }

    fn double_click(&mut self, pos: Pos2) {
        self.click(pos);
        self.frame(vec![Event::PointerMoved(pos), press(pos, true)]);
        self.frame(vec![press(pos, false)]);
    }

    fn drag(&mut self, from: Pos2, to: Pos2) {
        self.frame(vec![Event::PointerMoved(from)]);
        self.frame(vec![Event::PointerMoved(from), press(from, true)]);
        self.frame(vec![Event::PointerMoved(from.lerp(to, 0.5))]);
        self.frame(vec![Event::PointerMoved(to)]);
        self.frame(vec![press(to, false)]);
    }
}

#[test]
fn factory_registers_world_documents() {
    let mut registry = Registry::new();
    register(&mut registry);

    let world_id = WorldId::new(7);
    let doc = registry
        .create_document(DocKind::World(world_id))
        .expect("world factory registered");
    assert_eq!(doc.doc_kind(), DocKind::World(world_id));

    assert!(registry.create_document(DocKind::ProjectSettings).is_none());
}

#[test]
fn document_metadata_tracks_the_world() {
    let (mut project, world_id) =
        project_with_levels(vec![level(1, IVec2::ZERO, UVec2::new(100, 100), 0)]);
    let view = WorldDocumentView::new(world_id);

    assert_eq!(view.title(&project), "Overworld");
    assert!(view.is_valid(&project));

    project.worlds.clear();
    assert!(!view.is_valid(&project));
    assert_eq!(view.title(&project), "World");
}

#[test]
fn only_the_active_depth_is_interactive() {
    let (project, world_id) = project_with_levels(vec![
        level(1, IVec2::ZERO, UVec2::new(100, 100), 0),
        level(2, IVec2::ZERO, UVec2::new(100, 100), 1),
    ]);
    let mut harness = Harness::new(project, world_id);
    assert_eq!(harness.view.state().active_depth, 0);

    let center = harness.center_of(LevelId::new(1));
    harness.click(center);
    assert!(
        harness.events.iter().any(|event| matches!(
            event,
            EditorEvent::SelectionChanged(Selection::Level(id)) if *id == LevelId::new(1)
        )),
        "clicking the shared rectangle selects depth 0: {:?}",
        harness.events
    );

    harness.events.clear();
    harness.settle();
    harness.view.state_mut().active_depth = 1;
    harness.click(center);
    assert!(
        harness.events.iter().any(|event| matches!(
            event,
            EditorEvent::SelectionChanged(Selection::Level(id)) if *id == LevelId::new(2)
        )),
        "after switching depth the same rectangle selects depth 1: {:?}",
        harness.events
    );
    assert!(
        !harness
            .events
            .iter()
            .any(|event| matches!(event, EditorEvent::OpenDocument(_))),
        "a single click must not open the level"
    );
}

#[test]
fn double_click_opens_the_level_document() {
    let (project, world_id) =
        project_with_levels(vec![level(1, IVec2::ZERO, UVec2::new(100, 100), 0)]);
    let mut harness = Harness::new(project, world_id);
    let center = harness.center_of(LevelId::new(1));

    harness.double_click(center);

    assert!(
        harness.events.iter().any(|event| matches!(
            event,
            EditorEvent::OpenDocument(DocKind::Level(id)) if *id == LevelId::new(1)
        )),
        "double click emits OpenDocument(Level): {:?}",
        harness.events
    );
}

#[test]
fn dragging_a_level_emits_one_snapped_move_command() {
    let (project, world_id) =
        project_with_levels(vec![level(1, IVec2::ZERO, UVec2::new(100, 100), 0)]);
    let mut harness = Harness::new(project, world_id);
    let from = harness.center_of(LevelId::new(1));

    harness.drag(from, from + vec2(64.0, 64.0));

    assert!(harness.view.state().drag.is_none(), "drag state is cleared");
    assert_eq!(harness.pending.len(), 1, "one undo entry per drag");
    let command = harness.pending.remove(0);
    assert_eq!(command.label(), "Move Level");

    let mut stack = UndoStack::new();
    stack
        .execute(command, &mut harness.project)
        .expect("move applies");
    let moved = harness
        .project
        .level(LevelId::new(1))
        .expect("level exists");
    assert_ne!(moved.world_pos, IVec2::ZERO);
    assert_eq!(moved.world_pos.x % 16, 0);
    assert_eq!(moved.world_pos.y % 16, 0);

    let moved_pos = moved.world_pos;
    stack.undo(&mut harness.project);
    assert_eq!(
        harness
            .project
            .level(LevelId::new(1))
            .expect("level exists")
            .world_pos,
        IVec2::ZERO,
        "undo restores the original position"
    );
    assert_ne!(moved_pos, IVec2::ZERO);
}

#[test]
fn free_layout_does_not_snap_when_grid_snap_is_off() {
    let (project, world_id) =
        project_with_levels(vec![level(1, IVec2::ZERO, UVec2::new(100, 100), 0)]);
    let mut harness = Harness::new(project, world_id);
    harness.view.state_mut().grid_snap = false;
    let from = harness.center_of(LevelId::new(1));

    harness.drag(from, from + vec2(20.0, 20.0));

    assert_eq!(harness.pending.len(), 1);
    let command = harness.pending.remove(0);
    let mut stack = UndoStack::new();
    stack
        .execute(command, &mut harness.project)
        .expect("move applies");
    let moved = harness
        .project
        .level(LevelId::new(1))
        .expect("level exists");
    assert_ne!(moved.world_pos, IVec2::ZERO);
    assert!(
        moved.world_pos.x % 16 != 0 || moved.world_pos.y % 16 != 0,
        "unsnapped drag keeps arbitrary world coordinates: {:?}",
        moved.world_pos
    );
}

#[test]
fn linear_layout_drag_reorders_all_levels_in_one_command() {
    let (mut project, world_id) = project_with_levels(vec![
        level(1, IVec2::ZERO, UVec2::new(100, 100), 0),
        level(2, IVec2::new(148, 0), UVec2::new(100, 100), 0),
        level(3, IVec2::new(296, 0), UVec2::new(100, 100), 0),
    ]);
    project.worlds[0].layout = WorldLayout::LinearHorizontal;
    let mut harness = Harness::new(project, world_id);

    let from = harness.center_of(LevelId::new(1));
    let last = harness.center_of(LevelId::new(3));
    let to = last + vec2(10.0, 0.0);
    harness.drag(from, to);

    assert_eq!(harness.pending.len(), 1, "reorder is one undo entry");
    let command = harness.pending.remove(0);
    assert_eq!(command.label(), "Move Levels");

    let mut stack = UndoStack::new();
    stack
        .execute(command, &mut harness.project)
        .expect("reorder applies");
    let world = harness.project.world(world_id).expect("world exists");
    let mut positions: Vec<(i32, u64)> = world
        .levels
        .iter()
        .map(|level| (level.world_pos.x, level.id.0))
        .collect();
    positions.sort_unstable();
    assert_eq!(
        positions,
        vec![(0, 2), (148, 3), (296, 1)],
        "dragged level moved to the end and the line was re-spaced"
    );
    assert!(world.levels.iter().all(|level| level.world_pos.y == 0));
}

#[test]
fn ui_renders_overlapping_depths_and_empty_worlds() {
    let (project, world_id) = project_with_levels(vec![
        level(1, IVec2::ZERO, UVec2::new(100, 100), 0),
        level(2, IVec2::new(40, 40), UVec2::new(100, 100), 1),
    ]);
    let mut harness = Harness::new(project, world_id);
    harness.frame(Vec::new());
    assert!(harness.view.state().initialized);
    assert!(harness.view.state().viewport.zoom > 0.0);

    let (empty, empty_world) = project_with_levels(Vec::new());
    let mut harness = Harness::new(empty, empty_world);
    harness.frame(Vec::new());
    assert!(harness.view.state().initialized, "empty world still fits");
}
