use egui::{vec2, Context, Pos2, Rect};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_core::collision::{CollisionShape, CollisionShapeGeometry, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::tileset::collision_panel::render_collision_panel;
use lc_shell::features::tileset::collision_toolbar::render_collision_toolbar;
use lc_shell::features::tileset::right_panel::render_right_panel;
use lc_shell::features::tileset::state::{CollisionSnapMode, CollisionTool, TilesetViewState};
use lc_shell::shared::ui::custom_data_dialog::CustomDataDialogState;
use lc_shell::ShellApp;

fn setup_test_project() -> Project {
    let mut project = Project::new("Collision UI Test");
    let ts_id = TilesetDefId(1);
    let mut collisions = std::collections::BTreeMap::new();

    let mut col5 = TileCollisionDef::new();
    col5.shapes.push(CollisionShape {
        id: 1,
        geometry: CollisionShapeGeometry::Rectangle {
            x: 0.0,
            y: 0.0,
            w: 16.0,
            h: 16.0,
        },
        tag: "Solid".to_string(),
        one_way: false,
        custom_data: "friction=1.0".to_string(),
    });
    collisions.insert(5, col5);

    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "TestTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec![],
        marking: Default::default(),
        custom_data: Default::default(),
        animations: Default::default(),
        collisions,
    });

    project
}

#[test]
fn test_collision_view_state_defaults() {
    let state = TilesetViewState::default();
    assert_eq!(state.collision_tool, CollisionTool::Select);
    assert_eq!(state.collision_snap, CollisionSnapMode::Pixel);
    assert_eq!(state.collision_focus_tile, None);
    assert_eq!(state.selected_collision_shape_id, None);
    assert!(state.collision_clipboard.is_none());
    assert!(state.show_collision_overlay);
}

#[test]
fn test_collision_snap_mode_cycling() {
    let mut snap = CollisionSnapMode::Pixel;
    assert_eq!(snap.label(), "Pixel");

    snap.cycle();
    assert_eq!(snap, CollisionSnapMode::TileSize);
    assert_eq!(snap.label(), "Tile");

    snap.cycle();
    assert_eq!(snap, CollisionSnapMode::None);
    assert_eq!(snap.label(), "Off");

    snap.cycle();
    assert_eq!(snap, CollisionSnapMode::Pixel);
}

#[test]
fn test_custom_data_dialog_open_for_shape() {
    let mut dialog = CustomDataDialogState::default();
    assert!(!dialog.is_open);

    dialog.open_for_shape(TilesetDefId(1), 5, 2, Some("friction=1.5"));
    assert!(dialog.is_open);
    assert_eq!(dialog.tileset_id, Some(TilesetDefId(1)));
    assert_eq!(dialog.tile_id, 5);
    assert_eq!(dialog.shape_id, Some(2));
    assert_eq!(
        dialog.title_override.as_deref(),
        Some("TILE #5 SHAPE #2 CUSTOM DATA")
    );
    assert_eq!(dialog.text_buffer, "friction=1.5");

    dialog.close();
    assert!(!dialog.is_open);
    assert_eq!(dialog.shape_id, None);
    assert_eq!(dialog.title_override, None);
}

#[test]
fn test_collision_panel_rendering() {
    let project = setup_test_project();
    let app = ShellApp::new(project);
    let mut state = TilesetViewState {
        active_mode: "Collision".to_string(),
        selected_tile: Some(5),
        ..Default::default()
    };

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(240.0, 600.0))),
        ..Default::default()
    };

    let tileset = app.project.defs.tileset(TilesetDefId(1)).unwrap();
    let mut commands = Vec::new();

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_collision_panel(
            ui,
            &t,
            tileset,
            &app.project,
            app.assets.as_ref(),
            &mut state,
            &mut commands,
        );
    });
    output.textures_delta.clear();

    // Verify right panel dispatches to collision panel when active_mode == "Collision"
    let raw_input2 = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(240.0, 600.0))),
        ..Default::default()
    };
    let mut output2 = ctx.run_ui(raw_input2, |ui| {
        render_right_panel(
            ui,
            &t,
            tileset,
            &app.project,
            app.assets.as_ref(),
            &mut state,
            &mut commands,
        );
    });
    output2.textures_delta.clear();
}

#[test]
fn test_collision_toolbar_rendering() {
    let project = setup_test_project();
    let mut app = ShellApp::new(project);
    app.tileset_view_state.active_mode = "Collision".to_string();
    app.tileset_view_state.selected_tile = Some(5);

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 40.0))),
        ..Default::default()
    };

    let tileset = app.project.defs.tileset(TilesetDefId(1)).unwrap().clone();

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_collision_toolbar(&mut app, ui, &t, &tileset);
    });
    output.textures_delta.clear();
}
