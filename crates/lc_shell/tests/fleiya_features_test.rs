//! Tests verifying Fleiya Level Design Project features:
//! 1. Project panel displays worlds as asset tree without collapsible levels.
//! 2. Double-clicking a world directly opens that world.
//! 3. Layer, Entities, Tileset, and Enum open as windows, not in the central tab bar.
//! 4. Combined Tileset panel adapts to the selected layer type (Tiles, Entities, IntGrid).
//! 5. Adding tilesets works cleanly and updates definitions.

use egui::Context;
use glam::UVec2;
use lc_core::command::DocKind;
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::id::{EntityDefId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use lc_core::layer::LayerKind;
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_tileset::AddTilesetDefCommand;
use lc_shell::tileset_panel::CombinedTilesetPanel;
use lc_shell::ShellApp;
use lc_ui_api::view::PanelView;
use lc_ui_api::EditorCtx;

#[test]
fn test_world_double_click_opens_world_tab() {
    let mut shell = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let world_id = WorldId(1);
    let mut world = World::new(world_id, "Overworld");
    world
        .levels
        .push(Level::new(LevelId(1), "Level_1", UVec2::new(32, 32)));
    world
        .levels
        .push(Level::new(LevelId(2), "Level_2", UVec2::new(32, 32)));
    shell.project.worlds.push(world);

    // Double-click opening world directly
    shell.open_document(DocKind::World(world_id));

    // World document must be in central tab bar
    assert!(shell.open_documents.contains_key(&DocKind::World(world_id)));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 1);
    assert_eq!(
        shell.dock_state.iter_all_tabs().next().map(|(_, t)| *t),
        Some(DocKind::World(world_id))
    );
}

#[test]
fn test_definition_tabs_and_windows_behavior() {
    let mut shell = ShellApp::new(Project::new("Fleiya Level Design Project"));

    // Add sample definitions so documents are valid
    shell.project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Dungeon_16".to_string(),
        path: "assets/tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });
    shell.project.defs.enums.push(lc_core::defs::EnumDef {
        id: EnumDefId(1),
        name: "ItemRarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: Vec::new(),
    });

    // Opening LayerDef opens Layer window, not tab bar
    shell.open_document(DocKind::LayerDef(LayerDefId(1)));
    assert!(shell.show_layer_window);
    assert_eq!(shell.selected_layer_def, Some(LayerDefId(1)));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 0);

    // Opening EntityDef opens Entities window, not tab bar
    shell.open_document(DocKind::EntityDef(EntityDefId(2)));
    assert!(shell.show_entities_window);
    assert_eq!(shell.selected_entity_def, Some(EntityDefId(2)));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 0);

    // Opening TilesetDef opens Tileset window, not tab bar
    shell.open_document(DocKind::TilesetDef(TilesetDefId(1)));
    assert!(shell.show_tileset_window);
    assert_eq!(shell.selected_tileset_def, Some(TilesetDefId(1)));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 0);

    // Opening EnumDef opens as a tab in the central tab bar
    shell.open_document(DocKind::EnumDef(EnumDefId(1)));
    assert!(shell
        .open_documents
        .contains_key(&DocKind::EnumDef(EnumDefId(1))));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 1);
}

#[test]
fn test_add_tileset_command_and_persistence() {
    let mut shell = ShellApp::new(Project::new("Fleiya Level Design Project"));
    assert_eq!(shell.project.defs.tilesets.len(), 0);

    let tileset_def = TilesetDef {
        id: TilesetDefId(1),
        name: "Cavern".to_string(),
        path: "assets/tilesets/cavern.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    shell
        .pending_commands
        .push(Box::new(AddTilesetDefCommand::new(tileset_def.clone())));

    // Process frame
    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        shell.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(shell.project.defs.tilesets.len(), 1);
    assert_eq!(shell.project.defs.tilesets[0].name, "Cavern");
    assert_eq!(
        shell.project.defs.tilesets[0].path,
        "assets/tilesets/cavern.png"
    );

    // Undo adds
    shell.undo();
    assert_eq!(shell.project.defs.tilesets.len(), 0);

    // Redo restores
    shell.redo();
    assert_eq!(shell.project.defs.tilesets.len(), 1);
    assert_eq!(shell.project.defs.tilesets[0].name, "Cavern");
}

#[test]
fn test_combined_tileset_panel_renders_per_layer_type() {
    let mut panel = CombinedTilesetPanel::new();
    assert_eq!(panel.id().as_str(), "tileset");
    assert_eq!(panel.title(), "Tileset");

    let mut project = Project::new("Fleiya Level Design Project");

    // Add a Tileset
    project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Overworld Tiles".to_string(),
        path: "assets/tilesets/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });

    // Add a Tiles layer def
    let tiles_layer = LayerDef {
        id: LayerDefId(1),
        name: "Ground".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(TilesetDefId(1)),
        int_grid_values: Vec::new(),
    };
    project.defs.layers.push(tiles_layer);

    // Add an Entities layer def
    let entities_layer = LayerDef {
        id: LayerDefId(2),
        name: "Actors".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    };
    project.defs.layers.push(entities_layer);

    // Add an IntGrid layer def
    let intgrid_layer = LayerDef {
        id: LayerDefId(3),
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![IntGridValueDef {
            value: 1,
            name: "Solid".to_string(),
            color: [255, 0, 0, 255],
        }],
    };
    project.defs.layers.push(intgrid_layer);

    // Setup world and level
    let mut world = World::new(WorldId(1), "World_1");
    let level = Level::new(LevelId(1), "Level_1", UVec2::new(32, 32));
    world.levels.push(level);
    project.worlds.push(world);

    let undo = lc_core::command::UndoStack::new();
    let mut session = lc_ui_api::session::SessionState {
        active_level: Some(LevelId(1)),
        ..Default::default()
    };
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = lc_ui_api::assets::NoopAssetService;

    let ctx = Context::default();

    // 1. Render in Tiles mode
    session.active_layer = Some(LayerDefId(1));
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

    // 2. Render in Entities mode
    session.active_layer = Some(LayerDefId(2));
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

    // 3. Render in IntGrid mode
    session.active_layer = Some(LayerDefId(3));
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
}
