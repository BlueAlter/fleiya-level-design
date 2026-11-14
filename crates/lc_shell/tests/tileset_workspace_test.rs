use egui::Context;
use glam::UVec2;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::{
    render_tileset_view, sections_for, DOCUMENT_SUB_TAB, TILESET_DEF_SUB_TAB,
};
use lc_shell::ShellApp;

#[test]
fn test_sections_for_document_and_definition_tabs() {
    let doc_sections = sections_for(DOCUMENT_SUB_TAB);
    assert!(!doc_sections.title, "DOCUMENT tab must not show title");
    assert!(!doc_sections.footer, "DOCUMENT tab must not show footer");

    let def_sections = sections_for(TILESET_DEF_SUB_TAB);
    assert!(def_sections.title, "TILESET DEF tab must show title");
    assert!(def_sections.footer, "TILESET DEF tab must show footer");
}

#[test]
fn test_headless_render_zero_tilesets() {
    let mut app = ShellApp::new(Project::new("Zero Tilesets"));
    app.active_mode = AppMode::Tilesets;
    assert_eq!(app.project.defs.tilesets.len(), 0);

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    assert!(app.project.defs.tilesets.is_empty());
    assert!(app.pending_commands.is_empty());
}

#[test]
fn test_headless_render_one_tileset() {
    let mut p = Project::new("One Tileset");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Cavern".to_string(),
        path: "assets/tilesets/cavern.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;
    let initial_project = app.project.clone();

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.project, initial_project);
    assert_eq!(
        app.tileset_view_state.selected_tileset_id,
        Some(TilesetDefId(1))
    );
    assert_eq!(app.selected_tileset_def, Some(TilesetDefId(1)));
}

#[test]
fn test_headless_render_many_tilesets() {
    let mut p = Project::new("Many Tilesets");
    for i in 1..=5 {
        p.defs.tilesets.push(TilesetDef {
            id: TilesetDefId(i),
            name: format!("Dungeon_Theme_{i}"),
            path: format!("assets/tiles/dungeon_{i}.png"),
            tile_size: UVec2::new(16, 16),
            spacing: 1,
            padding: 2,
        });
    }

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;
    let initial_project = app.project.clone();

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.project, initial_project);
    assert_eq!(
        app.tileset_view_state.selected_tileset_id,
        Some(TilesetDefId(1))
    );
}

#[test]
fn test_headless_render_missing_image_no_panic() {
    let mut p = Project::new("Missing Image");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(10),
        name: "MissingTileset".to_string(),
        path: "non_existent/path/tiles.png".to_string(),
        tile_size: UVec2::new(32, 32),
        spacing: 0,
        padding: 0,
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
        Some(TilesetDefId(10))
    );
}

#[test]
fn test_headless_render_document_tab() {
    let mut p = Project::new("Document Tab Test");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "WorldTiles".to_string(),
        path: "assets/world.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.active_sub_tab = DOCUMENT_SUB_TAB;

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.tileset_view_state.active_sub_tab, DOCUMENT_SUB_TAB);
}

#[test]
fn test_headless_render_long_names() {
    let mut p = Project::new("Long Names Test");
    let very_long_name =
        "Super_Extremely_Long_Tileset_Definition_Identifier_Name_That_Should_Not_Break_Layout_"
            .repeat(2);
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(99),
        name: very_long_name,
        path: "assets/long.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
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
        Some(TilesetDefId(99))
    );
}
