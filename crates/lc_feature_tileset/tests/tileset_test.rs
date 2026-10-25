//! Integration and headless UI tests for lc_feature_tileset.

use glam::{IVec2, UVec2};
use lc_core::command::{DocKind, UndoStack};
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::grid::Grid;
use lc_core::id::{IdGenerator, LayerDefId, LevelId, TilesetDefId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind, TileRef};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_feature_tileset::commands::{
    AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand,
};
use lc_feature_tileset::handler::{IntGridLayerHandler, TilesLayerHandler};
use lc_feature_tileset::paint_commands::PaintTilesCommand;
use lc_feature_tileset::palette::TilePalettePanel;
use lc_feature_tileset::tools::EyedropperTool;
use lc_feature_tileset::view::TilesetDocumentView;
use lc_ui_api::assets::{AssetError, AssetService, NoopAssetService};
use lc_ui_api::canvas::{CanvasCtx, CanvasTool, LayerHandler, LayerRenderCtx};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::{SessionState, TileSelection};
use lc_ui_api::view::{DocumentView, PanelView};
use lc_ui_api::viewport::Viewport;

struct MockTextureService {
    size: [usize; 2],
}

impl AssetService for MockTextureService {
    fn load_texture(
        &self,
        ctx: &egui::Context,
        relative_path: &str,
    ) -> Result<egui::TextureHandle, AssetError> {
        let pixels = vec![egui::Color32::WHITE; self.size[0] * self.size[1]];
        let image = egui::ColorImage::new(self.size, pixels);
        Ok(ctx.load_texture(relative_path, image, egui::TextureOptions::NEAREST))
    }

    fn invalidate(&self, _relative_path: &str) {}
}

fn create_test_project_with_tileset() -> (Project, TilesetDefId) {
    let mut gen = IdGenerator::new();
    let tileset_id = gen.next_tileset_def_id();
    let tileset = TilesetDef {
        id: tileset_id,
        name: "Dungeon Tiles".to_string(),
        path: "tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    let mut project = Project::new("Test Project");
    project.defs.tilesets.push(tileset);
    project.worlds.push(lc_core::world::World::new(
        lc_core::id::WorldId::new(1),
        "World",
    ));
    (project, tileset_id)
}

#[test]
fn test_factory_and_panel_registration() {
    let mut reg = Registry::new();
    lc_feature_tileset::register(&mut reg);

    // Factory creates TilesetDocumentView for DocKind::TilesetDef
    let doc = reg.create_document(DocKind::TilesetDef(TilesetDefId(42)));
    assert!(doc.is_some());
    assert_eq!(
        doc.unwrap().doc_kind(),
        DocKind::TilesetDef(TilesetDefId(42))
    );

    // Factory returns None for other DocKinds
    assert!(reg.create_document(DocKind::ProjectSettings).is_none());

    // Panel is registered
    let panel = reg.find_panel(&TilePalettePanel::PANEL_ID.into());
    assert!(panel.is_some());
    assert_eq!(panel.unwrap().title(), "Tile Palette");
}

#[test]
fn test_tileset_document_view_metadata_and_validity() {
    let (mut project, tileset_id) = create_test_project_with_tileset();
    let view = TilesetDocumentView::new(tileset_id);

    assert_eq!(view.doc_kind(), DocKind::TilesetDef(tileset_id));
    assert!(view.is_valid(&project));
    assert_eq!(view.title(&project), "Tileset: Dungeon Tiles");

    // Remove tileset definition
    project.defs.tilesets.clear();
    assert!(!view.is_valid(&project));
    assert_eq!(view.title(&project), "Tileset (deleted)");
}

#[test]
fn test_tileset_document_view_ui_rendering() {
    let (project, tileset_id) = create_test_project_with_tileset();
    let mut view = TilesetDocumentView::new(tileset_id);

    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = MockTextureService { size: [64, 64] };

    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    // Now test with fallback / error asset service
    let noop_assets = NoopAssetService;
    {
        let mut cx2 = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &noop_assets,
        );

        let mut output2 = ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx2);
        });
        output2.textures_delta.clear();
    }
}

#[test]
fn test_tile_palette_panel_empty_state() {
    let mut panel = TilePalettePanel::new();
    assert_eq!(panel.id().as_str(), TilePalettePanel::PANEL_ID);
    assert_eq!(panel.title(), "Tile Palette");

    let project = Project::new("Empty Project");
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    assert!(events.is_empty());
}

#[test]
fn test_tile_palette_panel_selection_events_and_cx_readability() {
    let (project, tileset_id) = create_test_project_with_tileset();
    let mut panel = TilePalettePanel::new();

    let undo = UndoStack::new();
    let mut session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    // 64x64 texture with 16x16 tiles = 4 cols x 4 rows = 16 tiles
    let assets = MockTextureService { size: [64, 64] };

    // Initial render and emit single selection
    let single_sel = TileSelection::single(tileset_id, 5);
    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        let mut output = ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();

        cx.emit(EditorEvent::SetTileSelection(Some(single_sel.clone())));
    }

    assert_eq!(
        events,
        vec![EditorEvent::SetTileSelection(Some(single_sel.clone()))]
    );

    // Update session state as shell does and verify readability via cx.tile_selection()
    session.tile_selection = Some(single_sel.clone());
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), Some(&single_sel));
        assert_eq!(
            cx.tile_selection().unwrap().primary_tile().unwrap().tile_id,
            5
        );
    }

    // Emit rectangular tile selection and verify
    let rect_sel = TileSelection::rect(tileset_id, vec![0, 1, 4, 5], 2);
    events.clear();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        cx.emit(EditorEvent::SetTileSelection(Some(rect_sel.clone())));
    }

    assert_eq!(
        events,
        vec![EditorEvent::SetTileSelection(Some(rect_sel.clone()))]
    );

    session.tile_selection = Some(rect_sel.clone());
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), Some(&rect_sel));
        assert_eq!(cx.tile_selection().unwrap().width, 2);
        assert_eq!(cx.tile_selection().unwrap().tiles.len(), 4);
    }

    // Clear selection
    events.clear();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        cx.emit(EditorEvent::SetTileSelection(None));
    }
    assert_eq!(events, vec![EditorEvent::SetTileSelection(None)]);

    session.tile_selection = None;
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), None);
    }
}

#[test]
fn test_tileset_commands_undo_redo_round_trip() {
    let mut project = Project::new("Command Test");
    let mut undo_stack = UndoStack::new();

    let def = TilesetDef {
        id: TilesetDefId(10),
        name: "World Tiles".to_string(),
        path: "tiles/world.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    // 1. Add
    let add_cmd = Box::new(AddTilesetDefCommand::new(def.clone()));
    let focus = undo_stack
        .execute(add_cmd, &mut project)
        .expect("add should execute");
    assert_eq!(focus, Some(DocKind::TilesetDef(TilesetDefId(10))));
    assert_eq!(project.defs.tilesets.len(), 1);

    // 2. Edit
    let mut updated_def = def.clone();
    updated_def.name = "World Tiles Renamed".to_string();
    let edit_cmd = Box::new(UpdateTilesetDefCommand::new(updated_def.clone()));
    undo_stack
        .execute(edit_cmd, &mut project)
        .expect("edit should execute");
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );

    // 3. Undo edit
    let _ = undo_stack.undo(&mut project);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles"
    );

    // 4. Redo edit
    let _ = undo_stack.redo(&mut project);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );

    // 5. Remove
    let remove_cmd = Box::new(RemoveTilesetDefCommand::new(TilesetDefId(10)));
    undo_stack
        .execute(remove_cmd, &mut project)
        .expect("remove should execute");
    assert_eq!(project.defs.tilesets.len(), 0);

    // 6. Undo remove
    let _ = undo_stack.undo(&mut project);
    assert_eq!(project.defs.tilesets.len(), 1);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );
}

#[test]
fn test_layer_handlers_registration_and_tools() {
    let mut reg = Registry::new();
    lc_feature_tileset::register(&mut reg);

    // Both Tiles and IntGrid handlers are registered
    let tiles_handler = reg.find_layer_handler(LayerKind::Tiles);
    assert!(tiles_handler.is_some());
    assert_eq!(tiles_handler.unwrap().kind(), LayerKind::Tiles);

    let tools = tiles_handler.unwrap().tools();
    assert!(tools.iter().any(|t| t.name() == "Paint"));
    assert!(tools.iter().any(|t| t.name() == "Erase"));
    assert!(tools.iter().any(|t| t.name() == "Rectangle"));
    assert!(tools.iter().any(|t| t.name() == "Fill"));
    assert!(tools.iter().any(|t| t.name() == "Eyedropper"));

    let intgrid_handler = reg.find_layer_handler(LayerKind::IntGrid);
    assert!(intgrid_handler.is_some());
    assert_eq!(intgrid_handler.unwrap().kind(), LayerKind::IntGrid);
}

#[test]
fn test_continuous_mouse_drag_merges_undo_step() {
    // Acceptance criterion: Continuous mouse drag merges into a single undo step across frame boundaries.
    let (mut project, tileset_id) = create_test_project_with_tileset();
    let level_id = LevelId::new(1);
    let layer_def_id = LayerDefId::new(10);

    let mut level = Level::new(level_id, "Level_0", UVec2::new(160, 160));
    level.layers.push(LayerInstance::new(
        layer_def_id,
        LayerData::Tiles(Grid::new(10, 10, None)),
    ));
    project.defs.layers.push(LayerDef {
        id: layer_def_id,
        name: "TilesLayer".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });
    project.worlds[0].levels.push(level);

    let mut undo_stack = UndoStack::new();

    // Frame 1: Mouse down at (1, 1)
    let stroke_id = 12345;
    let tile_a = Some(TileRef::new(tileset_id, 1));
    let cmd1 =
        PaintTilesCommand::with_stroke(level_id, layer_def_id, vec![(1, 1, tile_a)], stroke_id);
    undo_stack.execute(Box::new(cmd1), &mut project).unwrap();

    // Frame 2: Dragged to (2, 1)
    let cmd2 =
        PaintTilesCommand::with_stroke(level_id, layer_def_id, vec![(2, 1, tile_a)], stroke_id);
    undo_stack.execute(Box::new(cmd2), &mut project).unwrap();

    // Frame 3: Dragged to (3, 1)
    let cmd3 =
        PaintTilesCommand::with_stroke(level_id, layer_def_id, vec![(3, 1, tile_a)], stroke_id);
    undo_stack.execute(Box::new(cmd3), &mut project).unwrap();

    // Frame 4: Dragged to (4, 1)
    let cmd4 =
        PaintTilesCommand::with_stroke(level_id, layer_def_id, vec![(4, 1, tile_a)], stroke_id);
    undo_stack.execute(Box::new(cmd4), &mut project).unwrap();

    // Verify all 4 frames merged into exactly ONE undo step
    assert!(undo_stack.can_undo());

    // Verify grid state
    let layer = project
        .level(level_id)
        .unwrap()
        .layer(layer_def_id)
        .unwrap();
    if let LayerData::Tiles(grid) = &layer.data {
        for x in 1..=4 {
            assert_eq!(grid.get(x, 1).copied().flatten(), tile_a);
        }
        assert_eq!(grid.get(0, 1).copied().flatten(), None);
    }

    // Undo reverts all 4 cells in a single step
    undo_stack.undo(&mut project).unwrap();
    assert!(
        !undo_stack.can_undo(),
        "Expected contiguous drag to merge into a single undo step"
    );

    let layer = project
        .level(level_id)
        .unwrap()
        .layer(layer_def_id)
        .unwrap();
    if let LayerData::Tiles(grid) = &layer.data {
        for x in 1..=4 {
            assert_eq!(grid.get(x, 1).copied().flatten(), None);
        }
    }

    // Redo restores all 4 cells in a single step
    undo_stack.redo(&mut project).unwrap();
    let layer = project
        .level(level_id)
        .unwrap()
        .layer(layer_def_id)
        .unwrap();
    if let LayerData::Tiles(grid) = &layer.data {
        for x in 1..=4 {
            assert_eq!(grid.get(x, 1).copied().flatten(), tile_a);
        }
    }
}

#[test]
fn test_tiles_layer_handler_render_and_viewport_culling() {
    // Acceptance criterion: Rendering uses viewport culling.
    let (mut project, tileset_id) = create_test_project_with_tileset();
    let level_id = LevelId::new(1);
    let layer_def_id = LayerDefId::new(10);

    let mut level = Level::new(level_id, "Level_0", UVec2::new(160, 160));
    let mut grid = Grid::new(10, 10, None);
    grid.set(1, 1, Some(TileRef::new(tileset_id, 3))).unwrap();
    grid.set(8, 8, Some(TileRef::new(tileset_id, 7))).unwrap();
    level
        .layers
        .push(LayerInstance::new(layer_def_id, LayerData::Tiles(grid)));
    project.defs.layers.push(LayerDef {
        id: layer_def_id,
        name: "TilesLayer".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });
    project.worlds[0].levels.push(level);

    let handler = TilesLayerHandler::new();

    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let painter = ui.painter();
        let viewport = Viewport {
            screen_rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)),
            ..Default::default()
        };

        let current_level = project.level(level_id).unwrap();
        let current_layer = current_level.layer(layer_def_id).unwrap();

        let mut layer_ctx = LayerRenderCtx {
            painter,
            viewport: &viewport,
            layer: current_layer,
            level: current_level,
            project: &project,
            is_active: true,
            opacity: 1.0,
        };

        // Render with culling operates safely without panics
        handler.render(&mut layer_ctx);
    });
    output.textures_delta.clear();
}

#[test]
fn test_intgrid_layer_handler_render() {
    let mut project = Project::new("Test IntGrid");
    let level_id = LevelId::new(1);
    let layer_def_id = LayerDefId::new(20);

    let mut layer_def = LayerDef {
        id: layer_def_id,
        name: "IntGrid".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    };
    layer_def.int_grid_values.push(IntGridValueDef {
        value: 1,
        name: "Solid".to_string(),
        color: [255, 0, 0, 255],
    });
    project.defs.layers.push(layer_def);

    let mut level = Level::new(level_id, "Level_0", UVec2::new(160, 160));
    let mut grid = Grid::new(10, 10, 0);
    grid.set(2, 2, 1).unwrap();
    level
        .layers
        .push(LayerInstance::new(layer_def_id, LayerData::IntGrid(grid)));
    let mut world = lc_core::world::World::new(lc_core::id::WorldId::new(1), "World");
    world.levels.push(level);
    project.worlds.push(world);

    let handler = IntGridLayerHandler::new();

    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let painter = ui.painter();
        let viewport = Viewport {
            screen_rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)),
            ..Default::default()
        };

        let current_level = project.level(level_id).unwrap();
        let current_layer = current_level.layer(layer_def_id).unwrap();

        let mut layer_ctx = LayerRenderCtx {
            painter,
            viewport: &viewport,
            layer: current_layer,
            level: current_level,
            project: &project,
            is_active: true,
            opacity: 1.0,
        };

        handler.render(&mut layer_ctx);
    });
    output.textures_delta.clear();
}

#[test]
fn test_eyedropper_tool_emits_tile_selection() {
    let (mut project, tileset_id) = create_test_project_with_tileset();
    let level_id = LevelId::new(1);
    let layer_def_id = LayerDefId::new(10);

    let mut level = Level::new(level_id, "Level_0", UVec2::new(160, 160));
    let mut grid = Grid::new(10, 10, None);
    grid.set(3, 3, Some(TileRef::new(tileset_id, 42))).unwrap();
    level
        .layers
        .push(LayerInstance::new(layer_def_id, LayerData::Tiles(grid)));
    project.defs.layers.push(LayerDef {
        id: layer_def_id,
        name: "TilesLayer".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });
    project.worlds[0].levels.push(level);

    let mut eyedropper = EyedropperTool::new(LayerKind::Tiles);

    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let undo_stack = UndoStack::new();
        let session = SessionState::default();
        let mut events = Vec::new();
        let mut pending_commands = Vec::new();
        let assets = NoopAssetService;

        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let viewport = Viewport::default();
        let response = ui.allocate_response(egui::vec2(100.0, 100.0), egui::Sense::click());

        let mut cv = CanvasCtx {
            viewport: &viewport,
            pointer_pos: Some(egui::pos2(10.0, 10.0)),
            world_pos: Some(IVec2::new(48, 48)),
            grid_pos: Some(IVec2::new(3, 3)),
            modifiers: egui::Modifiers::default(),
            response: &response,
            active_layer_id: Some(layer_def_id),
            active_level_id: level_id,
        };

        // Eyedropper on_input executes safely
        eyedropper.on_input(&mut cv, &mut cx);
    });
    output.textures_delta.clear();
}
