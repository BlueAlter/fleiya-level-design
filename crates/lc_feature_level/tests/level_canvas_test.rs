use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use egui::{pos2, vec2, Rect};
use glam::{IVec2, UVec2};
use lc_core::command::{DocKind, UndoStack};
use lc_core::grid::Grid;
use lc_core::id::{LayerDefId, LevelId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_level::{register, LevelCanvasState, LevelDocumentView};
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool, LayerHandler, LayerRenderCtx};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::registry::Registry;
use lc_ui_api::selection::Selection;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;

struct MockLayerHandler {
    kind: LayerKind,
    render_count: Arc<AtomicUsize>,
}

impl LayerHandler for MockLayerHandler {
    fn kind(&self) -> LayerKind {
        self.kind
    }

    fn render(&self, _ctx: &mut LayerRenderCtx) {
        self.render_count.fetch_add(1, Ordering::SeqCst);
    }

    fn tools(&self) -> Vec<Box<dyn CanvasTool>> {
        Vec::new()
    }
}

struct MockCanvasTool {
    name: String,
    inputs: Arc<AtomicUsize>,
}

impl CanvasTool for MockCanvasTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_input(&mut self, _cv: &mut CanvasCtx, _cx: &mut EditorCtx) {
        self.inputs.fetch_add(1, Ordering::SeqCst);
    }
}

struct MockInspectorSection {
    render_count: Arc<AtomicUsize>,
}

impl InspectorSection for MockInspectorSection {
    fn applies(&self, sel: &Selection) -> bool {
        matches!(sel, Selection::Level(_))
    }

    fn ui(&mut self, ui: &mut egui::Ui, _cx: &mut EditorCtx, _sel: &Selection) {
        self.render_count.fetch_add(1, Ordering::SeqCst);
        ui.label("MockInspectorActive");
    }
}

fn create_test_project_with_level() -> (Project, LevelId, LayerDefId) {
    let mut project = Project::new("LevelTest");
    let mut world = World::new(WorldId::new(1), "World_1");
    let level_id = LevelId::new(10);
    let mut level = Level::new(level_id, "Forest_1", UVec2::new(320, 240));
    level.world_pos = IVec2::new(0, 0);

    let layer_def_id = LayerDefId::new(100);
    let layer = LayerInstance {
        def_id: layer_def_id,
        visible: true,
        data: LayerData::IntGrid(Grid::new(20, 15, 0)),
    };
    level.layers.push(layer);

    world.levels.push(level);
    project.worlds.push(world);

    (project, level_id, layer_def_id)
}

#[test]
fn test_level_document_view_metadata_and_validity() {
    let (mut project, level_id, _) = create_test_project_with_level();
    let view = LevelDocumentView::new(level_id);

    assert_eq!(view.title(&project), "Forest_1");
    assert!(view.is_valid(&project));
    assert_eq!(view.doc_kind(), DocKind::Level(level_id));

    // Remove the level: view should become invalid
    project.worlds[0].levels.clear();
    assert!(!view.is_valid(&project));
}

#[test]
fn test_level_canvas_fit_and_pan_zoom() {
    let (_, level_id, _) = create_test_project_with_level();
    let level = Level::new(level_id, "Test", UVec2::new(400, 300));
    let mut state = LevelCanvasState::default();
    let canvas_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));

    state.fit_to_level(&level, canvas_rect);
    assert!(state.initialized);
    assert!(state.viewport.zoom > 0.0);

    let old_pan = state.viewport.pan;
    state.viewport.pan_by(vec2(100.0, -50.0));
    assert_eq!(state.viewport.pan, old_pan + vec2(100.0, -50.0));

    let old_zoom = state.viewport.zoom;
    state.viewport.zoom_at(2.0, pos2(400.0, 300.0));
    assert!((state.viewport.zoom - old_zoom * 2.0).abs() < 0.01);
}

#[test]
fn test_layer_render_loop_and_viewport_culling() {
    let (project, level_id, _) = create_test_project_with_level();
    let render_counter = Arc::new(AtomicUsize::new(0));

    let handler = Box::new(MockLayerHandler {
        kind: LayerKind::IntGrid,
        render_count: render_counter.clone(),
    });

    let mut view = LevelDocumentView::new(level_id).with_handler(handler);

    let ctx = egui::Context::default();
    let undo_stack = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    // First frame: level is in view, handler should render
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert_eq!(render_counter.load(Ordering::SeqCst), 1);

    // Pan viewport far away: layer should be culled
    view.state_mut().viewport.pan_by(vec2(100_000.0, 100_000.0));

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    // Count should not have incremented because the layer is culled
    assert_eq!(render_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_floating_inspector_sections_and_fallback() {
    let (project, level_id, _) = create_test_project_with_level();
    let inspector_counter = Arc::new(AtomicUsize::new(0));

    let section = Box::new(MockInspectorSection {
        render_count: inspector_counter.clone(),
    });

    let mut view = LevelDocumentView::new(level_id).with_inspector_section(section);

    let ctx = egui::Context::default();
    let undo_stack = UndoStack::new();
    let mut session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    // Selection is None: mock inspector should not apply, count stays 0
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert_eq!(inspector_counter.load(Ordering::SeqCst), 0);

    // Selection is Level: mock inspector should apply and render
    session.selection = Selection::Level(level_id);
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert_eq!(inspector_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_tool_management() {
    let (_, level_id, _) = create_test_project_with_level();
    let mut view = LevelDocumentView::new(level_id);

    let tool_counter = Arc::new(AtomicUsize::new(0));
    let tool = Box::new(MockCanvasTool {
        name: "MockPaint".to_string(),
        inputs: tool_counter,
    });

    view.set_tool(tool);
    assert!(view.tool().is_some());
    assert_eq!(view.tool().unwrap().name(), "MockPaint");

    view.clear_tool();
    assert!(view.tool().is_none());
}

#[test]
fn test_factory_registration() {
    let mut registry = Registry::new();
    register(&mut registry);

    let level_id = LevelId::new(20);
    let doc = registry.create_document(DocKind::Level(level_id));
    assert!(doc.is_some());
    assert_eq!(doc.unwrap().doc_kind(), DocKind::Level(level_id));

    // Other doc kinds should return None
    assert!(registry.create_document(DocKind::ProjectSettings).is_none());
}
