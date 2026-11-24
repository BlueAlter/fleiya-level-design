use egui::{vec2, Context, Pos2, Rect};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;
use lc_shell::features::tileset::right_panel::render_right_panel;
use lc_shell::features::tileset::state::{TilesetViewState, CUSTOM_DATA_VALUE_ID};
use lc_shell::ShellApp;

fn setup_test_project() -> Project {
    let mut project = Project::new("Right Panel Test");
    let ts_id = TilesetDefId(1);
    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "TestTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec![],
        marking: TileMarking {
            source_enum: Some(EnumDefId(1)),
            ..Default::default()
        },
        custom_data: Default::default(),
        animations: Default::default(),
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
fn test_right_panel_custom_data_and_enum_rendering() {
    let project = setup_test_project();
    let app = ShellApp::new(project);
    let mut state = TilesetViewState {
        active_mode: "Marking".to_string(),
        selected_tile: Some(50),
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
    output.textures_delta.clear();

    // Initial state: first enum value "Gold" should be selected by default
    assert_eq!(state.selected_marking_value.as_deref(), Some("Gold"));

    // Selecting Custom data
    state.selected_marking_value = Some(CUSTOM_DATA_VALUE_ID.to_string());
    assert_eq!(
        state.selected_marking_value.as_deref(),
        Some(CUSTOM_DATA_VALUE_ID)
    );

    // Re-render with Custom data selected
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

    // Selection should remain Custom data
    assert_eq!(
        state.selected_marking_value.as_deref(),
        Some(CUSTOM_DATA_VALUE_ID)
    );
}
