//! Additional workspace tests for tileset features: switching, missing enums, and deadlock avoidance.

use egui::{Context, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;

#[test]
fn test_headless_render_source_enum_missing() {
    let mut p = Project::new("Missing Source Enum");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "MissingEnumTileset".to_string(),
        path: "assets/tileset.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec!["Dungeon".to_string()],
        marking: TileMarking {
            source_enum: Some(lc_core::id::EnumDefId(999)),
            ..Default::default()
        },
        custom_data: Default::default(),
    });

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    assert_eq!(
        app.tileset_view_state.selected_tileset_id,
        Some(TilesetDefId(1))
    );
}

#[test]
fn test_tileset_switching_resets_selection_and_mode_preserves_pan_zoom() {
    let mut state = lc_shell::features::tileset::state::TilesetViewState::default();
    let ts1 = TilesetDefId(1);
    let ts2 = TilesetDefId(2);

    state.select_tileset(ts1);
    state.selected_tile = Some(5);
    state.active_mode = "None".to_string();
    {
        let cs1 = state.canvas_state_mut(ts1);
        cs1.view.zoom = 2.5;
        cs1.view.offset = egui::vec2(100.0, 50.0);
    }

    // Switch to tileset 2
    state.select_tileset(ts2);
    assert_eq!(state.selected_tileset_id, Some(ts2));
    assert_eq!(
        state.selected_tile, None,
        "selected tile must be reset to None on switch"
    );
    assert_eq!(
        state.active_mode, "None",
        "active mode must be reset to None on switch"
    );

    // Modify tileset 2's canvas state
    {
        let cs2 = state.canvas_state_mut(ts2);
        cs2.view.zoom = 1.0;
        cs2.view.offset = egui::vec2(0.0, 0.0);
    }
    state.selected_tile = Some(12);

    // Switch back to tileset 1
    state.select_tileset(ts1);
    assert_eq!(state.selected_tileset_id, Some(ts1));
    assert_eq!(
        state.selected_tile, None,
        "selected tile must be reset to None on switch"
    );
    assert_eq!(
        state.active_mode, "None",
        "active mode must be reset to None on switch"
    );

    // Verify tileset 1's zoom and pan are preserved
    let cs1 = state.canvas_state_mut(ts1);
    assert_eq!(cs1.view.zoom, 2.5, "zoom for tileset 1 must be preserved");
    assert_eq!(
        cs1.view.offset,
        egui::vec2(100.0, 50.0),
        "pan for tileset 1 must be preserved"
    );
}

#[test]
fn test_atlas_view_missing_and_empty_image_labels() {
    let mut p = Project::new("Image Warnings");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "MissingPath".to_string(),
        path: "assets/missing.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
    });
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(2),
        name: "EmptyPath".to_string(),
        path: String::new(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
    });

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let ctx = Context::default();
    let mut output1 = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });

    let text_items1: Vec<String> = output1
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect();
    output1.textures_delta.clear();

    assert!(
        text_items1
            .iter()
            .any(|t| t
                .contains("Warning: Source image file could not be loaded: assets/missing.png")),
        "clean warning must be shown for missing image file"
    );

    // Switch to empty path tileset
    app.tileset_view_state.select_tileset(TilesetDefId(2));
    let mut output2 = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });

    let text_items2: Vec<String> = output2
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect();
    output2.textures_delta.clear();

    assert!(
        text_items2.iter().any(|t| t == "No image loaded"),
        "clean label 'No image loaded' must be shown for empty path"
    );
}

#[test]
fn test_atlas_view_multi_frame_render_no_deadlock() {
    let mut app = ShellApp::new(Project::new("Sample"));
    let mut sample = std::path::PathBuf::from("examples/sample_project/project.lcproj");
    if !sample.exists() {
        sample = std::path::PathBuf::from("../../examples/sample_project/project.lcproj");
    }
    if sample.exists() {
        app.open_project_from_path(&sample).unwrap();
    }

    app.active_mode = AppMode::Tilesets;

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    for _ in 0..5 {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(raw_input, |ui| {
            app.ui(ui);
        });
        output.textures_delta.clear();
    }
}
