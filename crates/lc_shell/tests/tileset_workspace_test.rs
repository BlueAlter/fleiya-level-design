use egui::Context;
use glam::UVec2;
use lc_core::defs::{TileMarking, TilesetDef};
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
    assert!(!def_sections.footer, "TILESET DEF tab must not show footer");
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
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
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
            tags: Vec::new(),
            marking: TileMarking::EMPTY,
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
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
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
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
    });

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.active_sub_tab = DOCUMENT_SUB_TAB;

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    let mut texts = Vec::new();
    fn extract_text(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
            egui::Shape::Vec(children) => {
                for c in children {
                    extract_text(c, out);
                }
            }
            _ => {}
        }
    }
    for s in &output.shapes {
        extract_text(&s.shape, &mut texts);
    }

    assert_eq!(app.tileset_view_state.active_sub_tab, DOCUMENT_SUB_TAB);
    assert!(
        !texts.iter().any(|t| t.contains("(Tileset)")),
        "DOCUMENT tab must not render tileset title"
    );
    assert!(
        !texts.iter().any(|t| t.contains("Used by")),
        "DOCUMENT tab must not render footer"
    );
    assert!(
        !texts
            .iter()
            .any(|t| t.contains("TILESET IDENTIFIER & ATTRIBUTES")),
        "DOCUMENT tab must not render attribute card"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("Tilesets") || t.contains("tileset")),
        "DOCUMENT tab must render overview markdown document"
    );
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
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
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

#[test]
fn test_tileset_workspace_headless_rect_target_layout_1366x768() {
    use egui::Rect;

    let mut p = Project::new("Target Layout Test");
    for i in 1..=3 {
        p.defs.tilesets.push(TilesetDef {
            id: TilesetDefId(i),
            name: format!("Tileset_{i}"),
            path: format!("assets/tileset_{i}.png"),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: Vec::new(),
            marking: TileMarking::EMPTY,
        });
    }

    let mut app = ShellApp::new(p);
    app.active_mode = AppMode::Tilesets;

    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1366.0, 768.0),
        )),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output.textures_delta.clear();

    fn walk_shapes(
        shape: &egui::Shape,
        fill_rects: &mut Vec<Rect>,
        stroke_rects: &mut Vec<Rect>,
        text_items: &mut Vec<(String, Rect)>,
        lines: &mut Vec<[egui::Pos2; 2]>,
    ) {
        match shape {
            egui::Shape::Rect(r) => {
                if r.fill != egui::Color32::TRANSPARENT {
                    fill_rects.push(r.rect);
                }
                if r.stroke.width > 0.0 && r.stroke.color != egui::Color32::TRANSPARENT {
                    stroke_rects.push(r.rect);
                }
            }
            egui::Shape::Text(t) => {
                text_items.push((t.galley.text().to_string(), t.visual_bounding_rect()));
            }
            egui::Shape::LineSegment { points, .. } => {
                lines.push(*points);
            }
            egui::Shape::Vec(children) => {
                for c in children {
                    walk_shapes(c, fill_rects, stroke_rects, text_items, lines);
                }
            }
            _ => {}
        }
    }

    let mut fill_rects = Vec::new();
    let mut stroke_rects = Vec::new();
    let mut text_items = Vec::new();
    let mut lines = Vec::new();
    for clipped in &output.shapes {
        walk_shapes(
            &clipped.shape,
            &mut fill_rects,
            &mut stroke_rects,
            &mut text_items,
            &mut lines,
        );
    }

    // 1. sub_tabs rect is ABOVE the title
    let sub_tab = text_items
        .iter()
        .find(|(t, _)| t == "TILESET DEF")
        .expect("sub_tabs TILESET DEF must be present");
    let title = text_items
        .iter()
        .find(|(t, _)| t.contains("(Tileset)"))
        .expect("title must be present");
    let sub_tab_y = sub_tab.1.min.y;
    let title_y = title.1.min.y;
    assert!(
        sub_tab_y < title_y,
        "sub_tabs rect (y={sub_tab_y}) must be ABOVE title (y={title_y})"
    );

    // 2. every left row is bordered
    let left_row_borders: Vec<_> = stroke_rects
        .iter()
        .filter(|r| r.min.x < 260.0 && r.max.x <= 260.0 && (r.height() - 36.0).abs() < 2.0)
        .collect();
    assert_eq!(
        left_row_borders.len(),
        3,
        "every left row (3 total) must be bordered, found: {:?}",
        left_row_borders
    );

    // 3. atlas rect fills area between card and bottom - 12 (756.0)
    assert!(
        !text_items.iter().any(|(t, _)| t.contains("Used by")),
        "footer text must be removed from tileset workspace"
    );

    let card_title = text_items
        .iter()
        .find(|(t, _)| t.contains("TILESET IDENTIFIER & ATTRIBUTES"))
        .expect("card must exist");

    let atlas_backdrop = fill_rects.iter().find(|r| {
        r.min.x >= 250.0
            && r.min.y > card_title.1.max.y
            && (r.max.y - 756.0).abs() < 1.0
            && r.height() > 200.0
    });
    assert!(
        atlas_backdrop.is_some(),
        "atlas rect must fill area between card and bottom - 12 (756.0)"
    );

    // 4. right panel is a bordered tools card (240px wide)
    let tools_card = stroke_rects.iter().find(|r| {
        r.min.x > 1100.0 && (r.width() - 240.0).abs() < 1.0 && (r.max.x - 1354.0).abs() < 1.0
    });
    assert!(
        tools_card.is_some(),
        "right panel must be a 240px bordered tools card"
    );

    // 5. Attributes card fields exist (no tags or marking row)
    let id_row = text_items.iter().find(|(t, _)| t.contains("Identifier"));
    assert!(id_row.is_some(), "Identifier row must exist");

    let source_row = text_items.iter().find(|(t, _)| t.contains("Source image"));
    assert!(source_row.is_some(), "Source image row must exist");

    let layout_row = text_items.iter().find(|(t, _)| t.contains("Tiles layout"));
    assert!(layout_row.is_some(), "Tiles layout row must exist");

    let tags_row = text_items.iter().find(|(t, _)| t.contains("Tags:"));
    assert!(
        tags_row.is_none(),
        "Tags row must NOT exist (separate task)"
    );
}
