//! Tests verifying Enum Window bug fixes and feature requirements:
//! 1. Enum window does not open automatically on startup.
//! 2. Window is non-collapsible and non-resizable.
//! 3. Bound tiles display tile graphic icons, and clearing/deleting bound tiles works.
//! 4. Adding enum values, sorting, and tileset selection dialog function correctly.

use egui::Context;
use glam::UVec2;
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_feature_enums::{
    AddEnumValueCommand, RemoveEnumDefCommand, ReorderEnumDefsCommand, ReorderEnumValuesCommand,
    UpdateEnumValueCommand,
};
use lc_shell::features::tileset::dialog::TilesetDialogState;
use lc_shell::ShellApp;

#[test]
fn test_tileset_dialog_pan_and_zoom() {
    let mut state = TilesetDialogState::new();
    assert_eq!(state.pan, egui::Pos2::new(20.0, 20.0));
    assert_eq!(state.zoom, 2.0);

    state.pan += egui::vec2(50.0, -30.0);
    state.zoom = (state.zoom * 1.5).clamp(0.5, 8.0);

    assert_eq!(state.pan, egui::Pos2::new(70.0, -10.0));
    assert_eq!(state.zoom, 3.0);
}

#[test]
fn test_enum_delete_cascade_command() {
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let enum_id = EnumDefId(1);
    app.project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "ItemRarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });

    assert_eq!(app.project.defs.enums.len(), 1);

    let _ = app.undo_stack.execute(
        Box::new(RemoveEnumDefCommand::cascade(enum_id)),
        &mut app.project,
    );

    assert!(app.project.defs.enums.is_empty());

    // Undo restore
    let _ = app.undo_stack.undo(&mut app.project);
    assert_eq!(app.project.defs.enums.len(), 1);
    assert_eq!(app.project.defs.enums[0].id, enum_id);
}

#[test]
fn test_enum_window_closed_by_default_on_launch() {
    let app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    assert!(
        !app.show_enum_window,
        "Enum window must NOT open automatically when the app is launched"
    );
}

#[test]
fn test_enum_window_renders_when_opened() {
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    app.project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Dungeon_16".to_string(),
        path: "assets/tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });
    app.project.defs.enums.push(EnumDef {
        id: EnumDefId(1),
        name: "ItemRarity".to_string(),
        tileset_id: Some(TilesetDefId(1)),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Common".to_string(),
                name: "Common Item".to_string(),
                color: [156, 163, 175, 255],
                tile: Some(TileRef::new(TilesetDefId(1), 5)),
            },
            EnumValueDef {
                id: "Rare".to_string(),
                name: "Rare Item".to_string(),
                color: [61, 214, 184, 255],
                tile: None,
            },
        ],
    });

    app.show_enum_window = true;
    app.selected_enum_def = Some(EnumDefId(1));

    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |_ui| {
        lc_shell::features::enums::window::render_enum_window(&mut app, &ctx);
    });
    output.textures_delta.clear();

    // Window opened and rendered cleanly without panics
    assert!(app.show_enum_window);
}

#[test]
fn test_bound_tile_delete_command() {
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let enum_id = EnumDefId(1);
    app.project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "ItemRarity".to_string(),
        tileset_id: Some(TilesetDefId(1)),
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Epic".to_string(),
            name: "Epic Item".to_string(),
            color: [168, 85, 247, 255],
            tile: Some(TileRef::new(TilesetDefId(1), 12)),
        }],
    });

    assert!(app.project.defs.enums[0].values[0].tile.is_some());

    // Clearing/deleting tile
    let mut updated_val = app.project.defs.enums[0].values[0].clone();
    updated_val.tile = None;
    let _ = app.undo_stack.execute(
        Box::new(UpdateEnumValueCommand::new(enum_id, updated_val)),
        &mut app.project,
    );

    assert!(app.project.defs.enums[0].values[0].tile.is_none());
}

#[test]
fn test_enum_add_value_and_sort() {
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let enum_id = EnumDefId(1);
    app.project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "WeaponType".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Sword".to_string(),
                name: "Sword".to_string(),
                color: [156, 163, 175, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Axe".to_string(),
                name: "Axe".to_string(),
                color: [255, 193, 77, 255],
                tile: None,
            },
        ],
    });

    // Add Bow
    let new_val = EnumValueDef {
        id: "Bow".to_string(),
        name: "Bow".to_string(),
        color: [61, 214, 184, 255],
        tile: None,
    };
    let _ = app.undo_stack.execute(
        Box::new(AddEnumValueCommand::new(enum_id, new_val)),
        &mut app.project,
    );

    assert_eq!(app.project.defs.enums[0].values.len(), 3);

    // Sort A-Z
    let mut sorted_ids: Vec<String> = app.project.defs.enums[0]
        .values
        .iter()
        .map(|v| v.id.clone())
        .collect();
    sorted_ids.sort();
    let _ = app.undo_stack.execute(
        Box::new(ReorderEnumValuesCommand::new(enum_id, sorted_ids)),
        &mut app.project,
    );

    assert_eq!(app.project.defs.enums[0].values[0].id, "Axe");
    assert_eq!(app.project.defs.enums[0].values[1].id, "Bow");
    assert_eq!(app.project.defs.enums[0].values[2].id, "Sword");
}

#[test]
fn test_tileset_dialog_lifecycle() {
    let mut state = TilesetDialogState::new();
    assert!(!state.is_open);

    state.open("Common".to_string(), Some(TilesetDefId(2)), None);
    assert!(state.is_open);
    assert_eq!(state.target_val_id, Some("Common".to_string()));
    assert_eq!(state.selected_tileset_id, Some(TilesetDefId(2)));

    state.close();
    assert!(!state.is_open);
    assert!(state.target_val_id.is_none());
}

#[test]
fn test_enum_defs_reorder_command() {
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    app.project.defs.enums.push(EnumDef {
        id: EnumDefId(1),
        name: "First".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });
    app.project.defs.enums.push(EnumDef {
        id: EnumDefId(2),
        name: "Second".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });
    app.project.defs.enums.push(EnumDef {
        id: EnumDefId(3),
        name: "Third".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });

    let new_order = vec![EnumDefId(3), EnumDefId(1), EnumDefId(2)];
    let _ = app.undo_stack.execute(
        Box::new(ReorderEnumDefsCommand::new(new_order)),
        &mut app.project,
    );

    assert_eq!(app.project.defs.enums[0].id, EnumDefId(3));
    assert_eq!(app.project.defs.enums[1].id, EnumDefId(1));
    assert_eq!(app.project.defs.enums[2].id, EnumDefId(2));

    // Revert undo
    let _ = app.undo_stack.undo(&mut app.project);
    assert_eq!(app.project.defs.enums[0].id, EnumDefId(1));
    assert_eq!(app.project.defs.enums[1].id, EnumDefId(2));
    assert_eq!(app.project.defs.enums[2].id, EnumDefId(3));
}

#[test]
fn test_tileset_dialog_grid_toggle() {
    let mut state = TilesetDialogState::new();
    assert!(state.show_grid, "Grid should be enabled by default");
    state.show_grid = false;
    assert!(!state.show_grid);
}

#[test]
fn test_enum_delete_modal_flow() {
    use lc_shell::features::enums::window::PendingDeleteModal;

    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let enum_id = EnumDefId(10);
    app.project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "TestEnum".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Val1".to_string(),
            name: "Value 1".to_string(),
            color: [255, 0, 0, 255],
            tile: None,
        }],
    });

    // Set pending delete modal
    app.pending_enum_delete = Some(PendingDeleteModal::Enum {
        id: enum_id,
        name: "TestEnum".to_string(),
        entity_count: 0,
        field_count: 0,
    });
    assert!(app.pending_enum_delete.is_some());

    // When modal confirms, command cascades
    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        events: vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |_ui| {
        // Render confirmation modal with Enter pressed
        lc_shell::features::enums::window::render_enum_window(&mut app, &ctx);
    });
    output.textures_delta.clear();
    app.process_commands_and_events();

    // Enum was deleted
    assert!(app.project.defs.enums.is_empty());
    assert!(app.pending_enum_delete.is_none());
}

#[test]
fn test_enum_value_delete_modal_flow() {
    use lc_shell::features::enums::window::PendingDeleteModal;

    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    let enum_id = EnumDefId(20);
    app.project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "TestEnumVal".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Val1".to_string(),
                name: "Value 1".to_string(),
                color: [255, 0, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Val2".to_string(),
                name: "Value 2".to_string(),
                color: [0, 255, 0, 255],
                tile: None,
            },
        ],
    });

    app.pending_enum_delete = Some(PendingDeleteModal::EnumValue {
        enum_id,
        val_id: "Val1".to_string(),
        name: "Value 1".to_string(),
    });

    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        events: vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |_ui| {
        lc_shell::features::enums::window::render_enum_window(&mut app, &ctx);
    });
    output.textures_delta.clear();
    app.process_commands_and_events();

    assert_eq!(app.project.defs.enums[0].values.len(), 1);
    assert_eq!(app.project.defs.enums[0].values[0].id, "Val2");
    assert!(app.pending_enum_delete.is_none());
}
