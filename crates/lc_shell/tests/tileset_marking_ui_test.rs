use egui::{vec2, Context, Pos2, Rect};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::command::Command;
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;
use lc_feature_tileset::{AddTilesetTagCommand, RemoveTilesetTagCommand, SetMarkingSourceCommand, SetTileMarkingCommand};
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;

fn setup_test_project() -> Project {
    let mut project = Project::new("Marking UI Test");

    let ts_id = TilesetDefId(1);
    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "WorldTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec!["Ground".to_string()],
        marking: TileMarking {
            source_enum: Some(EnumDefId(1)),
            ..Default::default()
        },
    });

    project.defs.enums.push(EnumDef {
        id: EnumDefId(1),
        name: "TerrainType".to_string(),
        tileset_id: Some(ts_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Grass".to_string(),
                name: "Grass".to_string(),
                color: [34, 197, 94, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Water".to_string(),
                name: "Water".to_string(),
                color: [59, 130, 246, 255],
                tile: None,
            },
        ],
    });

    project
}

#[test]
fn test_tileset_tags_and_marking_ui_rendering() {
    let project = setup_test_project();
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    // Verify tag text "Ground" is rendered
    let tileset = app.project.defs.tileset(TilesetDefId(1)).unwrap();
    assert_eq!(tileset.tags, vec!["Ground"]);
    assert_eq!(tileset.marking.source_enum, Some(EnumDefId(1)));

    // Test tag addition command
    let mut add_cmd = Box::new(AddTilesetTagCommand::new(TilesetDefId(1), "Decoration"));
    add_cmd.apply(&mut app.project).expect("tag addition should succeed");
    assert_eq!(
        app.project.defs.tileset(TilesetDefId(1)).unwrap().tags,
        vec!["Ground", "Decoration"]
    );

    // Test tag removal command
    let mut remove_cmd = Box::new(RemoveTilesetTagCommand::new(TilesetDefId(1), "Ground"));
    remove_cmd.apply(&mut app.project).expect("tag removal should succeed");
    assert_eq!(
        app.project.defs.tileset(TilesetDefId(1)).unwrap().tags,
        vec!["Decoration"]
    );
    remove_cmd.revert(&mut app.project);
    assert_eq!(
        app.project.defs.tileset(TilesetDefId(1)).unwrap().tags,
        vec!["Ground", "Decoration"]
    );

    // Test tile marking command and revert
    let mut mark_cmd = Box::new(SetTileMarkingCommand::new(
        TilesetDefId(1),
        "Grass",
        4,
        true,
    ));
    mark_cmd.apply(&mut app.project).expect("tile marking should succeed");
    assert!(app.project.defs.tileset(TilesetDefId(1)).unwrap().marking.has_tag("Grass", 4));

    mark_cmd.revert(&mut app.project);
    assert!(!app.project.defs.tileset(TilesetDefId(1)).unwrap().marking.has_tag("Grass", 4));

    // Test clearing marking source enum
    let mut clear_src_cmd = Box::new(SetMarkingSourceCommand::new(TilesetDefId(1), None));
    clear_src_cmd.apply(&mut app.project).expect("clearing source should succeed");
    assert_eq!(app.project.defs.tileset(TilesetDefId(1)).unwrap().marking.source_enum, None);
    clear_src_cmd.revert(&mut app.project);
    assert_eq!(
        app.project.defs.tileset(TilesetDefId(1)).unwrap().marking.source_enum,
        Some(EnumDefId(1))
    );
}
