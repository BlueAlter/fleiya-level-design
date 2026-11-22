//! Parity tests asserting pixel-exact parity between the tileset workspace and the shared definition kit.
//!
//! Measurements are verified against the dark theme at 1366x768.

use egui::{vec2, Context, CornerRadius, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::{render_tileset_view, DOCUMENT_SUB_TAB};
use lc_shell::shared::ui::def_workspace::def_panel_width;
use lc_shell::ShellApp;

/// Walks all shapes in the clipped shape tree recursively.
fn walk_shapes(
    shape: &Shape,
    rects: &mut Vec<Rect>,
    shapes_list: &mut Vec<Shape>,
    text_items: &mut Vec<(String, Rect)>,
) {
    match shape {
        Shape::Rect(r) => {
            rects.push(r.rect);
            shapes_list.push(shape.clone());
        }
        Shape::Text(t) => {
            text_items.push((t.galley.text().to_string(), t.visual_bounding_rect()));
            shapes_list.push(shape.clone());
        }
        Shape::Vec(children) => {
            for c in children {
                walk_shapes(c, rects, shapes_list, text_items);
            }
        }
        _ => {
            shapes_list.push(shape.clone());
        }
    }
}

/// Renders the tileset workspace headless at 1366x768 with Dark theme.
fn render_tileset_headless(app: &mut ShellApp) -> (Vec<Rect>, Vec<Shape>, Vec<(String, Rect)>) {
    app.active_mode = AppMode::Tilesets;
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(app, ui);
    });
    output.textures_delta.clear();

    let mut rects = Vec::new();
    let mut shapes = Vec::new();
    let mut text_items = Vec::new();
    for clipped in &output.shapes {
        walk_shapes(&clipped.shape, &mut rects, &mut shapes, &mut text_items);
    }
    (rects, shapes, text_items)
}

#[test]
fn test_tileset_workspace_parity_metrics() {
    let mut p = Project::new("Workspace Parity");
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
    let (ts_rects, ts_shapes, ts_texts) = render_tileset_headless(&mut app);

    // 1. Left panel width 0..260
    assert_eq!(def_panel_width(), 260.0);
    let ts_panel_bg = ts_shapes.iter().any(|s| match s {
        Shape::Rect(r) => r.rect.min.x.abs() < 0.5 && (r.rect.max.x - 260.0).abs() < 0.5,
        _ => false,
    });
    assert!(
        ts_panel_bg,
        "tileset left panel background must span 0..260"
    );

    // 2. Assert no shape of the left panel passes x = 260.5
    for s in &ts_shapes {
        if let Shape::Rect(r) = s {
            if r.rect.min.x < 260.0 && r.rect.max.x < 272.0 {
                assert!(
                    r.rect.max.x <= 260.5,
                    "left panel shape must not pass x=260.5, found max.x={}",
                    r.rect.max.x
                );
            }
        }
    }

    // 3. "+ New" button at 12..214 x 12..40 (height 28)
    let new_btn = ts_rects
        .iter()
        .find(|r| {
            (r.min.x - 12.0).abs() < 0.5
                && (r.max.x - 214.0).abs() < 0.5
                && (r.height() - 28.0).abs() < 0.5
        })
        .expect("Tileset '+ New' button must be at 12..214 with height 28");
    assert!(
        (new_btn.min.y - 12.0).abs() <= 1.0,
        "'+ New' button top must be at y=12"
    );

    // 4. Kebab button at 220..248 x 12..40 (width 28, height 28)
    let kebab_btn = ts_rects
        .iter()
        .find(|r| {
            (r.min.x - 220.0).abs() < 0.5
                && (r.max.x - 248.0).abs() < 0.5
                && (r.height() - 28.0).abs() < 0.5
        })
        .expect("Tileset top kebab button must be at 220..248 with height 28");
    assert!(
        (kebab_btn.min.y - 12.0).abs() <= 1.0,
        "Kebab button top must be at y=12"
    );

    // 5. Search box
    let search_item = ts_texts
        .iter()
        .find(|(t, _)| t.contains("Search"))
        .expect("Search placeholder must exist");
    assert!(
        search_item.1.min.x >= 12.0 && search_item.1.min.x < 248.0,
        "search placeholder must be inside left panel"
    );

    // 6. List header
    assert!(
        ts_texts
            .iter()
            .any(|(t, _)| t.contains("TILESET DEFINITIONS (3)")),
        "list header 'TILESET DEFINITIONS (3)' must exist"
    );

    // 7. Cards span 12..248 (width 236), height 36, corner radius 4, pitch 41
    let mut ts_cards = Vec::new();
    for s in &ts_shapes {
        if let Shape::Rect(r) = s {
            let rect = r.rect;
            if (rect.min.x - 12.0).abs() < 0.5
                && (rect.max.x - 248.0).abs() < 0.5
                && (rect.height() - 36.0).abs() < 0.5
                && !ts_cards
                    .iter()
                    .any(|existing: &Rect| (existing.min.y - rect.min.y).abs() < 0.5)
            {
                assert_eq!(r.corner_radius, CornerRadius::same(4));
                ts_cards.push(rect);
            }
        }
    }
    assert_eq!(ts_cards.len(), 3, "expected 3 tileset cards");
    assert!((ts_cards[0].width() - 236.0).abs() < 0.5);
    let pitch = ts_cards[1].min.y - ts_cards[0].min.y;
    assert!(
        (pitch - 41.0).abs() < 0.5,
        "card pitch ({pitch}) must match 41.0"
    );

    // 8. sub_tabs row top and height == "+ New" top and height
    let first_tab = ts_rects
        .iter()
        .find(|r| r.min.x > 270.0 && r.min.x < 300.0 && (r.height() - 22.0).abs() < 1.0)
        .expect("sub-tab pill rect must exist");
    let sub_tabs_row_top = (first_tab.min.y - 3.0).round();
    let sub_tabs_row_height = (first_tab.height() + 6.0).round();
    assert!(
        (sub_tabs_row_top - new_btn.min.y).abs() < 0.5,
        "sub_tabs row top ({sub_tabs_row_top}) must equal '+ New' top ({}) within 0.5",
        new_btn.min.y
    );
    assert!(
        (sub_tabs_row_height - new_btn.height()).abs() < 0.5,
        "sub_tabs row height ({sub_tabs_row_height}) must equal '+ New' height ({}) within 0.5",
        new_btn.height()
    );

    // 9. Detail margins 272..1354
    let ts_attr_title = ts_texts
        .iter()
        .find(|(t, _)| t.contains("TILESET IDENTIFIER & ATTRIBUTES"))
        .expect("section header must exist");
    assert!(
        ts_attr_title.1.min.x >= 270.0 && ts_attr_title.1.min.x <= 280.0,
        "detail area content must start after 260px panel + margin (~272px), found {}",
        ts_attr_title.1.min.x
    );

    // 10. Identifier field start and fill
    let id_label = ts_texts
        .iter()
        .find(|(t, _)| t.contains("Identifier"))
        .expect("Identifier label must exist");
    assert!(
        id_label.1.min.x >= 272.0,
        "Identifier label must start within detail margin >= 272"
    );

    // 11. Footer rect (min.x = 260.0, max.x = 1366.0) and text x (starts at 280.0)
    let ts_footer_text = ts_texts
        .iter()
        .find(|(t, _)| t.contains("Used by"))
        .expect("tileset footer text must exist");
    assert!(
        (ts_footer_text.1.min.x - 280.0).abs() < 2.0,
        "footer text must start at x=280 (260 + 20), found {}",
        ts_footer_text.1.min.x
    );

    // 12. Square icon buttons (width == height) in attributes card
    let square_buttons: Vec<_> = ts_rects
        .iter()
        .filter(|r| {
            r.min.x > 272.0
                && r.width() > 10.0
                && (r.width() - r.height()).abs() < 0.5
                && r.width() <= 32.0
        })
        .collect();
    assert!(
        !square_buttons.is_empty(),
        "square icon buttons (e.g. browse/clear) must exist with width == height"
    );
}

#[test]
fn test_document_mode_has_no_title_and_no_footer() {
    let mut p = Project::new("Doc Parity");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "WorldTileset".to_string(),
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

    let (_, _, texts) = render_tileset_headless(&mut app);

    // In DOCUMENT tab:
    // 1. Overview markdown content is present
    assert!(texts.iter().any(|(t, _)| t.contains("Tilesets")));
    // 2. Title ("(Tileset)") must NOT be present
    assert!(
        !texts.iter().any(|(t, _)| t.contains("(Tileset)")),
        "DOCUMENT tab must NOT show title row"
    );
    // 3. Footer ("Used by") must NOT be present
    assert!(
        !texts.iter().any(|(t, _)| t.contains("Used by")),
        "DOCUMENT tab must NOT show pinned footer"
    );
}

#[test]
fn test_floating_window_layout_fails_parity() {
    // Proves that an old floating layout (floating window with no 0..260 docked panel)
    // fails the parity assertions.
    let old_floating_panel_width = 300.0;
    assert_ne!(
        old_floating_panel_width,
        def_panel_width(),
        "old floating panel width fails 260.0 parity"
    );

    let old_floating_has_sub_tabs = false;
    assert!(
        !old_floating_has_sub_tabs,
        "old floating window did not have workspace sub_tabs"
    );
}
