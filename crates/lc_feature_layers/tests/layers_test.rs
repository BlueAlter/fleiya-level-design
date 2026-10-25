//! Integration and headless UI tests for lc_feature_layers.

use glam::UVec2;
use lc_core::command::{DocKind, UndoStack};
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::grid::Grid;
use lc_core::id::{IdGenerator, LayerDefId, LevelId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_layers::commands::{
    RemoveLayerDefCommand, ReorderLayerDefsCommand, ToggleLayerVisibilityCommand,
};
use lc_feature_layers::panel::LayersPanel;
use lc_feature_layers::view::LayerDefDocumentView;
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::{DocumentView, PanelId, PanelView};

fn setup_test_project_with_layers() -> (Project, LayerDefId, LayerDefId, LevelId) {
    let mut gen = IdGenerator::new();
    let mut project = Project::new("Test Project");

    let l1_id = gen.next_layer_def_id();
    let l1_def = LayerDef {
        id: l1_id,
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![IntGridValueDef {
            value: 1,
            name: "Solid".to_string(),
            color: [220, 50, 50, 255],
        }],
    };

    let l2_id = gen.next_layer_def_id();
    let l2_def = LayerDef {
        id: l2_id,
        name: "Tiles_Foreground".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    };

    project.defs.layers.push(l1_def);
    project.defs.layers.push(l2_def);

    let world_id = WorldId(1);
    let mut world = World::new(world_id, "MainWorld");

    let level_id = LevelId(10);
    let mut level = Level::new(level_id, "Level_0", UVec2::new(320, 240));

    level.layers.push(LayerInstance::new(
        l1_id,
        LayerData::IntGrid(Grid::new(20, 15, 0)),
    ));
    level.layers.push(LayerInstance::new(
        l2_id,
        LayerData::Tiles(Grid::new(20, 15, None)),
    ));

    world.levels.push(level);
    project.worlds.push(world);

    (project, l1_id, l2_id, level_id)
}

#[test]
fn test_factory_and_panel_registration() {
    let mut reg = Registry::new();
    lc_feature_layers::register(&mut reg);

    // Factory creates LayerDefDocumentView for DocKind::LayerDef
    let layer_id = LayerDefId::new(42);
    let doc_view = reg.create_document(DocKind::LayerDef(layer_id));
    assert!(doc_view.is_some());
    assert_eq!(doc_view.unwrap().doc_kind(), DocKind::LayerDef(layer_id));

    // Factory returns None for other DocKinds
    assert!(reg.create_document(DocKind::ProjectSettings).is_none());

    // Panel is registered
    let panel = reg.find_panel(&PanelId::new("layers"));
    assert!(panel.is_some());
    assert_eq!(panel.unwrap().title(), "Layers");
}

#[test]
fn test_layer_def_document_view_properties() {
    let (mut project, l1_id, _, _) = setup_test_project_with_layers();
    let mut view = LayerDefDocumentView::new(l1_id);

    assert_eq!(view.doc_kind(), DocKind::LayerDef(l1_id));
    assert!(view.is_valid(&project));
    assert_eq!(view.title(&project), "Layer: Collisions");

    // Headless UI render
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    // Invalidation when def is deleted
    project.defs.layers.clear();
    assert!(!view.is_valid(&project));
    assert_eq!(view.title(&project), "Layer (deleted)");
}

#[test]
fn test_layers_panel_empty_state_when_no_active_level() {
    let (project, _, _, _) = setup_test_project_with_layers();
    let mut panel = LayersPanel::new();
    assert_eq!(panel.id().as_str(), LayersPanel::PANEL_ID);
    assert_eq!(panel.title(), "Layers");

    let undo = UndoStack::new();
    let session = SessionState {
        active_level: None,
        ..Default::default()
    };
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        assert!(cx.active_level().is_none());
        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    assert!(events.is_empty());
    assert!(pending_commands.is_empty());
}

#[test]
fn test_layers_panel_with_active_level_and_selection() {
    let (project, l1_id, l2_id, level_id) = setup_test_project_with_layers();
    let mut panel = LayersPanel::new();

    let undo = UndoStack::new();
    let mut session = SessionState {
        active_level: Some(level_id),
        active_layer: Some(l1_id),
        ..Default::default()
    };
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        assert_eq!(cx.active_level(), Some(level_id));
        assert_eq!(cx.active_layer(), Some(l1_id));

        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();

        // Emit layer selection event
        cx.emit(EditorEvent::SetActiveLayer(Some(l2_id)));
    }

    assert_eq!(events, vec![EditorEvent::SetActiveLayer(Some(l2_id))]);

    // Update session state as shell does and verify readability via cx.active_layer()
    session.active_layer = Some(l2_id);
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.active_layer(), Some(l2_id));
    }
}

#[test]
fn test_layer_visibility_command_revert() {
    let (mut project, l1_id, _, level_id) = setup_test_project_with_layers();

    assert!(
        project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    let cmd = ToggleLayerVisibilityCommand::new(level_id, l1_id);
    let mut undo = UndoStack::new();

    undo.execute(Box::new(cmd), &mut project).unwrap();
    assert!(
        !project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    undo.undo(&mut project).unwrap();
    assert!(
        project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    undo.redo(&mut project).unwrap();
    assert!(
        !project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );
}

#[test]
fn test_layer_reorder_command_revert() {
    let (mut project, l1_id, l2_id, level_id) = setup_test_project_with_layers();

    // Initial order: [l1, l2]
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );

    let mut undo = UndoStack::new();
    let cmd = ReorderLayerDefsCommand::new(vec![l2_id, l1_id]);
    undo.execute(Box::new(cmd), &mut project).unwrap();

    // Reordered: [l2, l1]
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l2_id, l1_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l2_id, l1_id]
    );

    // Undo reverts back to [l1, l2]
    undo.undo(&mut project).unwrap();
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
}

#[test]
fn test_cascade_remove_and_revert() {
    let (mut project, l1_id, l2_id, level_id) = setup_test_project_with_layers();

    let mut undo = UndoStack::new();
    let cmd = RemoveLayerDefCommand::new(l1_id);
    undo.execute(Box::new(cmd), &mut project).unwrap();

    // Def removed and level layer removed
    assert_eq!(project.defs.layers.len(), 1);
    assert_eq!(project.defs.layers[0].id, l2_id);
    assert_eq!(project.level(level_id).unwrap().layers.len(), 1);
    assert_eq!(project.level(level_id).unwrap().layers[0].def_id, l2_id);

    // Revert restores l1_id at index 0
    undo.undo(&mut project).unwrap();
    assert_eq!(project.defs.layers.len(), 2);
    assert_eq!(project.defs.layers[0].id, l1_id);
    assert_eq!(project.level(level_id).unwrap().layers.len(), 2);
    assert_eq!(project.level(level_id).unwrap().layers[0].def_id, l1_id);
}
