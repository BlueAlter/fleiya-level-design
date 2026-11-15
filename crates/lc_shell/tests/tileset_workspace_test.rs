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

#[test]
fn test_tileset_workspace_consolidation_guard() {
    let view_src = include_str!("../src/features/tileset/view.rs");
    let mod_editor_src = include_str!("../src/features/editor/mod.rs");
    let shell_app_src = include_str!("../src/features/editor/shell_app.rs");
    let menu_src = include_str!("../src/features/editor/menu.rs");
    let mod_tileset_src = include_str!("../src/features/tileset/mod.rs");
    let left_panel_src = include_str!("../src/features/tileset/left_panel.rs");
    let right_panel_src = include_str!("../src/features/tileset/right_panel.rs");
    let attr_card_src = include_str!("../src/features/tileset/attributes_card.rs");
    let atlas_view_src = include_str!("../src/features/tileset/atlas_view.rs");

    let all_sources = [
        ("view.rs", view_src),
        ("editor/mod.rs", mod_editor_src),
        ("shell_app.rs", shell_app_src),
        ("menu.rs", menu_src),
        ("tileset/mod.rs", mod_tileset_src),
        ("left_panel.rs", left_panel_src),
        ("right_panel.rs", right_panel_src),
        ("attributes_card.rs", attr_card_src),
        ("atlas_view.rs", atlas_view_src),
    ];

    for (name, src) in all_sources {
        assert!(
            !src.to_lowercase().contains("under construction"),
            "File {} must not contain 'under construction'",
            name
        );
    }

    assert!(
        !mod_editor_src.contains("tileset_placeholder"),
        "tileset_placeholder module must not be registered"
    );
    assert!(
        !shell_app_src.contains("render_tileset_placeholder"),
        "shell_app must not call render_tileset_placeholder"
    );
    assert!(
        !shell_app_src.contains("show_tileset_window"),
        "shell_app must not reference show_tileset_window"
    );

    let mut def_count = 0;
    for (_, src) in all_sources {
        for line in src.lines() {
            let trimmed = line.trim();
            if (trimmed.starts_with("pub fn render_tileset_view")
                || trimmed.starts_with("fn render_tileset_view"))
                && trimmed.contains('(')
            {
                def_count += 1;
            }
        }
    }

    assert_eq!(
        def_count, 1,
        "Exactly one fn render_tileset_view must be defined"
    );
}
