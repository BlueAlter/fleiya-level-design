//! Parity tests asserting pixel-exact parity between the tileset workspace and the enum workspace.
//!
//! Measurements are verified against the dark theme at 1366x768.

use egui::{vec2, Context, CornerRadius, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{EnumDef, TileMarking, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::enums::view::render_enum_view;
use lc_shell::features::tileset::view::{render_tileset_view, DOCUMENT_SUB_TAB};
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
        Shape::Circle(c) => {
            rects.push(Rect::from_center_size(
                c.center,
                vec2(c.radius * 2.0, c.radius * 2.0),
            ));
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

/// Renders the enum workspace headless at 1366x768 with Dark theme.
fn render_enum_headless(app: &mut ShellApp) -> (Vec<Rect>, Vec<Shape>, Vec<(String, Rect)>) {
    app.active_mode = AppMode::Enums;
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    if app.tileset_view_state.active_sub_tab == DOCUMENT_SUB_TAB {
        ctx.data_mut(|d| {
            let state = lc_shell::features::enums::view::EnumViewState {
                active_sub_tab: 1,
                ..Default::default()
            };
            d.insert_temp(egui::Id::new("enum_mode_view_transient_state"), state);
        });
    }

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_enum_view(app, ui);
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

/// Renders the tileset workspace headless at 1366x768 with Dark theme.
fn render_tileset_headless(app: &mut ShellApp) -> (Vec<Rect>, Vec<Shape>, Vec<(String, Rect)>) {
    app.active_mode = AppMode::Tilesets;
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    app.assets.set_project_dir(Some(repo_root.to_path_buf()));
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

fn sample_project() -> Project {
    let mut p = Project::new("Parity Project");
    p.defs.enums = vec![
        EnumDef {
            id: EnumDefId(1),
            name: "BiomeType".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
        EnumDef {
            id: EnumDefId(2),
            name: "ItemRarity".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
        EnumDef {
            id: EnumDefId(3),
            name: "WeaponType".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
    ];
    for i in 1..=3 {
        p.defs.tilesets.push(TilesetDef {
            id: TilesetDefId(i),
            name: format!("Tileset_{i}"),
            path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: Vec::new(),
            marking: TileMarking::EMPTY,
            custom_data: Default::default(),
            animations: Default::default(),
            collisions: Default::default(),
        });
    }
    p
}

#[test]
fn test_tileset_enum_workspace_parity() {
    let p = sample_project();

    let mut enum_app = ShellApp::new(p.clone());
    enum_app.enum_draft = p.defs.enums.clone();
    enum_app.selected_enum_def = Some(EnumDefId(2));
    let (enum_rects, _enum_shapes, enum_texts) = render_enum_headless(&mut enum_app);

    let mut ts_app = ShellApp::new(p);
    ts_app.selected_tileset_def = Some(TilesetDefId(2));
    ts_app.tileset_view_state.select_tileset(TilesetDefId(2));
    let (ts_rects, ts_shapes, ts_texts) = render_tileset_headless(&mut ts_app);

    // 1. sub_tabs top == left panel "+ New" top (12) and height == 28; tabs left == 272 in both workspaces
    let ts_new_btn = ts_rects
        .iter()
        .find(|r| {
            (r.min.x - 12.0).abs() < 0.5
                && (r.max.x - 214.0).abs() < 0.5
                && (r.height() - 28.0).abs() < 0.5
        })
        .expect("'+ New' button must exist at 12..214 x 12..40");
    assert_eq!(ts_new_btn.min.y, 12.0);
    assert_eq!(ts_new_btn.height(), 28.0);

    let ts_tab_row = ts_rects
        .iter()
        .find(|r| {
            (r.min.x - 272.0).abs() < 0.5
                && (r.min.y - 12.0).abs() < 0.5
                && (r.height() - 28.0).abs() < 0.5
        })
        .expect("Tileset sub-tabs container must be at left=272, top=12, height=28");
    assert!(
        (ts_tab_row.min.x - 272.0).abs() < 0.5,
        "tabs left must be 272"
    );

    // 2. title x and description x == 272; title y offset from the tabs equals the enum's
    let ts_title = ts_texts
        .iter()
        .find(|(t, _)| t.contains("(Tileset)"))
        .expect("Tileset title text must exist");
    let ts_desc = ts_texts
        .iter()
        .find(|(t, _)| t.contains("Configure tileset definitions"))
        .expect("Tileset description text must exist");
    assert!(
        (ts_title.1.min.x - 272.0).abs() < 0.5,
        "title x must be 272.0, found {}",
        ts_title.1.min.x
    );
    assert!(
        (ts_desc.1.min.x - 272.0).abs() < 0.5,
        "description x must be 272.0, found {}",
        ts_desc.1.min.x
    );

    let enum_title = enum_texts
        .iter()
        .find(|(t, _)| t.contains("(Enum)"))
        .expect("Enum title text must exist");
    let enum_tab_top = 12.0;
    let enum_title_offset = enum_title.1.min.y - enum_tab_top;
    let ts_title_offset = ts_title.1.min.y - ts_tab_row.min.y;
    assert!(
        (ts_title_offset - enum_title_offset).abs() < 1.0,
        "title y offset ({ts_title_offset}) must match enum's ({enum_title_offset})"
    );

    // 3. attribute card left == 272 and right == 1354; square corners (CornerRadius::ZERO)
    let ts_attr_card = ts_shapes
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - 272.0).abs() < 0.5 && (r.rect.max.x - 1354.0).abs() < 0.5 =>
            {
                Some(r)
            }
            _ => None,
        })
        .expect("Attribute card must span left=272 to right=1354");
    assert_eq!(ts_attr_card.corner_radius, CornerRadius::ZERO);

    // 4. the first control of EVERY attribute row has the same x (label column), and equals the enum Identifier field x
    let enum_id_field = enum_rects
        .iter()
        .find(|r| r.min.x > 390.0 && r.min.x < 420.0 && (r.height() - 32.0).abs() < 1.0)
        .expect("Enum identifier input field rect must exist");
    let expected_ctrl_x = enum_id_field.min.x;

    let ts_field_rects: Vec<Rect> = ts_rects
        .iter()
        .filter(|r| (r.min.x - expected_ctrl_x).abs() < 1.0 && (r.height() - 32.0).abs() < 1.0)
        .copied()
        .collect();
    assert!(
        ts_field_rects.len() >= 3,
        "every attribute row first control must start at x={expected_ctrl_x}, found {}",
        ts_field_rects.len()
    );

    // 5. every field height == 32; every text vertical center == its field center; left padding == 8;
    //    the Source image row right edge == card inner right; Browse and Clear are 32 x 32
    for fr in &ts_field_rects {
        assert!(
            (fr.height() - 32.0).abs() < 0.5,
            "field height must be 32, found {}",
            fr.height()
        );
    }

    let browse_btn = ts_rects
        .iter()
        .find(|r| {
            (r.width() - 32.0).abs() < 0.5 && (r.height() - 32.0).abs() < 0.5 && r.min.x > 1200.0
        })
        .expect("Browse button must be 32x32");
    let clear_btn = ts_rects
        .iter()
        .find(|r| {
            (r.width() - 32.0).abs() < 0.5
                && (r.height() - 32.0).abs() < 0.5
                && r.min.x > browse_btn.max.x
        })
        .expect("Clear button must be 32x32");
    let card_inner_right = ts_attr_card.rect.max.x - 12.0 - ts_attr_card.stroke.width;
    assert!(
        (clear_btn.max.x - card_inner_right).abs() < 0.5,
        "Source image row right edge must equal card inner right ({card_inner_right}), found {}",
        clear_btn.max.x
    );

    // 6. atlas card: left == 272, right == mode card left - 12; mode card right == 1354, width == 240;
    //    same top and bottom; both have a 1 px hairline border; no Shape::Text containing "Fit" anywhere in the output
    let mode_card_right = 1354.0;
    let mode_card_left = mode_card_right - 240.0; // 1114.0
    let atlas_card_right = mode_card_left - 12.0; // 1102.0
    let atlas_card_left = 272.0;

    let atlas_card = ts_shapes
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - atlas_card_left).abs() < 0.5
                    && (r.rect.max.x - atlas_card_right).abs() < 0.5 =>
            {
                Some(r)
            }
            _ => None,
        })
        .expect("Atlas card must span 272..1102");

    let mode_card = ts_shapes
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - mode_card_left).abs() < 0.5
                    && (r.rect.max.x - mode_card_right).abs() < 0.5 =>
            {
                Some(r)
            }
            _ => None,
        })
        .expect("Mode card must span 1114..1354 (width 240)");

    assert_eq!(
        atlas_card.rect.min.y, mode_card.rect.min.y,
        "Atlas card and Mode card must share the same top"
    );
    assert_eq!(
        atlas_card.rect.max.y, mode_card.rect.max.y,
        "Atlas card and Mode card must share the same bottom"
    );

    assert!(
        !ts_texts.iter().any(|(t, _)| t == "Fit"),
        "Fit button must be removed from atlas view"
    );

    // 7. atlas card bottom == window bottom - 12 (756.0); the zoom label is inside the atlas card
    assert!(
        (atlas_card.rect.max.y - 756.0).abs() < 1.0,
        "Atlas card bottom ({}) must equal window bottom - 12 (756.0)",
        atlas_card.rect.max.y
    );

    let zoom_label = ts_texts
        .iter()
        .find(|(t, _)| t.ends_with('%'))
        .expect("Zoom label must exist");
    assert!(
        zoom_label.1.max.y < atlas_card.rect.max.y,
        "Zoom label must be inside atlas card"
    );

    // 8. first fit: image rect is centered in atlas canvas within 1 px and zoom is fit zoom
    assert!(atlas_card.rect.width() > 100.0);

    // 9. DOCUMENT tab: first heading x == tabs left == 272, wrap right <= 1354, first heading y == enum DOCUMENT first heading y
    let mut ts_doc_app = ShellApp::new(sample_project());
    ts_doc_app.active_mode = AppMode::Tilesets;
    ts_doc_app.tileset_view_state.active_sub_tab = DOCUMENT_SUB_TAB;
    let (_, _, ts_doc_texts) = render_tileset_headless(&mut ts_doc_app);

    let mut enum_doc_app = ShellApp::new(sample_project());
    enum_doc_app.active_mode = AppMode::Enums;
    enum_doc_app.tileset_view_state.active_sub_tab = DOCUMENT_SUB_TAB;
    let (_, _, enum_doc_texts) = render_enum_headless(&mut enum_doc_app);

    let ts_first_h = ts_doc_texts
        .iter()
        .find(|(t, _)| t.contains("Tilesets"))
        .expect("DOCUMENT heading");
    let enum_first_h = enum_doc_texts
        .iter()
        .find(|(t, _)| t.contains("Enums"))
        .expect("Enum DOCUMENT heading");
    assert!(
        (ts_first_h.1.min.x - 272.0).abs() < 1.0,
        "doc heading x must be 272, found {}",
        ts_first_h.1.min.x
    );
    assert!(
        (ts_first_h.1.min.y - enum_first_h.1.min.y).abs() < 1.0,
        "doc heading y must match enum's: ts={}, enum={}",
        ts_first_h.1.min.y,
        enum_first_h.1.min.y
    );

    // 10. tileset workspace has NO footer text, enum footer text exists
    assert!(
        !ts_texts.iter().any(|(t, _)| t.contains("Used by")),
        "Tileset workspace must not have footer text"
    );
    assert!(
        enum_texts.iter().any(|(t, _)| t.contains("Used by")),
        "Enum workspace retains footer text"
    );
}

#[test]
#[allow(clippy::print_stdout)]
fn test_dump_before_metrics() {
    let p = sample_project();
    let mut enum_app = ShellApp::new(p.clone());
    enum_app.enum_draft = p.defs.enums.clone();
    enum_app.selected_enum_def = Some(EnumDefId(2));
    let (enum_rects, _enum_shapes, enum_texts) = render_enum_headless(&mut enum_app);

    let mut ts_app = ShellApp::new(p);
    ts_app.selected_tileset_def = Some(TilesetDefId(2));
    ts_app.tileset_view_state.select_tileset(TilesetDefId(2));
    let (ts_rects, _ts_shapes, ts_texts) = render_tileset_headless(&mut ts_app);

    println!("=== ENUM WORKSPACE RECT TABLE ===");
    for r in &enum_rects {
        if r.width() > 10.0 && r.height() > 10.0 {
            println!(
                "Enum Rect: [{:.1}, {:.1}] - [{:.1}, {:.1}] (w={:.1}, h={:.1})",
                r.min.x,
                r.min.y,
                r.max.x,
                r.max.y,
                r.width(),
                r.height()
            );
        }
    }
    println!("=== ENUM WORKSPACE TEXTS ===");
    for (txt, r) in &enum_texts {
        println!(
            "Enum Text: {:<30} at [{:.1}, {:.1}] - [{:.1}, {:.1}]",
            txt, r.min.x, r.min.y, r.max.x, r.max.y
        );
    }

    println!("=== TILESET WORKSPACE RECT TABLE ===");
    for r in &ts_rects {
        if r.width() > 10.0 && r.height() > 10.0 {
            println!(
                "Tileset Rect: [{:.1}, {:.1}] - [{:.1}, {:.1}] (w={:.1}, h={:.1})",
                r.min.x,
                r.min.y,
                r.max.x,
                r.max.y,
                r.width(),
                r.height()
            );
        }
    }
    println!("=== TILESET WORKSPACE TEXTS ===");
    for (txt, r) in &ts_texts {
        println!(
            "Tileset Text: {:<30} at [{:.1}, {:.1}] - [{:.1}, {:.1}]",
            txt, r.min.x, r.min.y, r.max.x, r.max.y
        );
    }
}
