use egui::{vec2, Context, Pos2, Rect};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::atlas_overlay::{render_tile_overlay, TileOverlayContext};
use lc_shell::features::tileset::state::CUSTOM_DATA_VALUE_ID;
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;

fn setup_marking_project() -> Project {
    let mut project = Project::new("Atlas Marking Test");
    let ts_id = TilesetDefId(1);
    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "WorldTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec![],
        marking: TileMarking {
            source_enum: Some(EnumDefId(1)),
            tags: {
                let mut map = std::collections::BTreeMap::new();
                map.insert("Gold".to_string(), {
                    let mut set = std::collections::BTreeSet::new();
                    set.insert(50);
                    set.insert(52);
                    set
                });
                map.insert("Gem".to_string(), {
                    let mut set = std::collections::BTreeSet::new();
                    set.insert(52);
                    set
                });
                map
            },
        },
        custom_data: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(52, "Some custom data".to_string());
            map
        },
    });

    project.defs.enums.push(EnumDef {
        id: EnumDefId(1),
        name: "ItemType".to_string(),
        tileset_id: Some(ts_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Gold".to_string(),
                name: "Gold".to_string(),
                color: [255, 215, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Gem".to_string(),
                name: "Gem".to_string(),
                color: [0, 191, 255, 255],
                tile: None,
            },
        ],
    });

    project
}

#[test]
fn test_atlas_overlay_filtering_and_custom_data_mode() {
    let project = setup_marking_project();
    let app = ShellApp::new(project);
    let tileset = app.project.defs.tileset(TilesetDefId(1)).unwrap();
    let enum_def = app.project.defs.enum_def(EnumDefId(1)).unwrap();

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let painter = ui.painter();
        let tile_rect = Rect::from_min_size(Pos2::new(10.0, 10.0), vec2(32.0, 32.0));

        // 1. When "Gold" is selected: tile 50 is marked
        let cx_gold = TileOverlayContext {
            tileset,
            enum_def: Some(enum_def),
            is_marking_mode: true,
            selected_val_id: Some("Gold"),
            project: &app.project,
            assets: app.assets.as_ref(),
        };
        render_tile_overlay(painter, ui.ctx(), tile_rect, 50, &cx_gold);

        // 2. When "Gem" is selected: tile 50 is NOT marked (filtered out)
        let cx_gem = TileOverlayContext {
            tileset,
            enum_def: Some(enum_def),
            is_marking_mode: true,
            selected_val_id: Some("Gem"),
            project: &app.project,
            assets: app.assets.as_ref(),
        };
        render_tile_overlay(painter, ui.ctx(), tile_rect, 50, &cx_gem);

        // 3. When Custom Data is selected: tile 52 has multiple tags (concentric outline)
        let cx_custom_data = TileOverlayContext {
            tileset,
            enum_def: Some(enum_def),
            is_marking_mode: true,
            selected_val_id: Some(CUSTOM_DATA_VALUE_ID),
            project: &app.project,
            assets: app.assets.as_ref(),
        };
        render_tile_overlay(painter, ui.ctx(), tile_rect, 52, &cx_custom_data);
    });
    output.textures_delta.clear();
}

#[test]
fn test_custom_data_dialog_open_and_command_execution() {
    let project = setup_marking_project();
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Dialog initial state is closed
    assert!(!app.tileset_view_state.custom_data_dialog.is_open);

    // Open dialog for tile 52
    let existing = app
        .project
        .defs
        .tileset(TilesetDefId(1))
        .unwrap()
        .custom_data
        .get(&52)
        .cloned();
    app.tileset_view_state
        .custom_data_dialog
        .open(TilesetDefId(1), 52, existing.as_deref());

    assert!(app.tileset_view_state.custom_data_dialog.is_open);
    assert_eq!(app.tileset_view_state.custom_data_dialog.tile_id, 52);
    assert_eq!(
        app.tileset_view_state.custom_data_dialog.text_buffer,
        "Some custom data"
    );

    // Run UI to render dialog
    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();
}

#[test]
fn test_custom_data_dialog_freezes_atlas() {
    let project = setup_marking_project();
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Initial render
    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    let initial_zoom = app
        .tileset_view_state
        .pan_zoom
        .get(&TilesetDefId(1))
        .map(|pz| pz.zoom)
        .unwrap_or(1.0);

    // Open custom data dialog
    app.tileset_view_state
        .custom_data_dialog
        .open(TilesetDefId(1), 52, Some("line 1\nline 2"));
    assert!(app.tileset_view_state.custom_data_dialog.is_open);

    // Simulate key presses (F key) and mouse scrolling while dialog is open
    let events = vec![egui::Event::Key {
        key: egui::Key::F,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }];

    let raw_input_with_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events,
        ..Default::default()
    };
    let mut output2 = ctx.run_ui(raw_input_with_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output2.textures_delta.clear();

    // Atlas zoom and pan must remain frozen and unchanged!
    let current_zoom = app
        .tileset_view_state
        .pan_zoom
        .get(&TilesetDefId(1))
        .map(|pz| pz.zoom)
        .unwrap_or(1.0);
    assert_eq!(
        initial_zoom, current_zoom,
        "Atlas zoom must remain frozen while dialog is open"
    );
}

#[test]
fn test_none_mode_shows_overlay_markers() {
    let project = setup_marking_project();
    let app = ShellApp::new(project);
    let tileset = app.project.defs.tileset(TilesetDefId(1)).unwrap();
    let enum_def = app.project.defs.enum_def(EnumDefId(1)).unwrap();

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let painter = ui.painter();
        let tile_rect = Rect::from_min_size(Pos2::new(10.0, 10.0), vec2(32.0, 32.0));

        // When mode is "None" (is_marking_mode = false):
        // tile 50 (has Gold tag) and tile 52 (has tags + custom data) must still draw overlay indicators!
        let cx_none = TileOverlayContext {
            tileset,
            enum_def: Some(enum_def),
            is_marking_mode: false,
            selected_val_id: None,
            project: &app.project,
            assets: app.assets.as_ref(),
        };

        render_tile_overlay(painter, ui.ctx(), tile_rect, 50, &cx_none);
        render_tile_overlay(painter, ui.ctx(), tile_rect, 52, &cx_none);
    });
    output.textures_delta.clear();
    assert!(
        !output.shapes.is_empty(),
        "Overlay shapes must be rendered even in None mode"
    );
}

#[test]
fn test_enum_dropdown_freezes_atlas() {
    let project = setup_marking_project();
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Initial render
    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    let initial_zoom = app
        .tileset_view_state
        .pan_zoom
        .get(&TilesetDefId(1))
        .map(|pz| pz.zoom)
        .unwrap_or(1.0);

    // Simulate dropdown being open
    app.tileset_view_state.is_enum_dropdown_open = true;

    // Simulate F key press
    let events = vec![egui::Event::Key {
        key: egui::Key::F,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }];
    let raw_input_with_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events,
        ..Default::default()
    };
    let mut output2 = ctx.run_ui(raw_input_with_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output2.textures_delta.clear();

    let current_zoom = app
        .tileset_view_state
        .pan_zoom
        .get(&TilesetDefId(1))
        .map(|pz| pz.zoom)
        .unwrap_or(1.0);
    assert_eq!(
        initial_zoom, current_zoom,
        "Atlas zoom must remain frozen while dropdown is open"
    );
}
