//! B1 Attribute card tests.
//!
//! Asserts order of rows (Source image, Identifier, Tiles layout),
//! button for Source image with 32x32 Reload and Clear icon buttons,
//! and 4 small layout fields on one line at 1366x768.

use egui::{vec2, Context, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{EnumDef, TileMarking, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::enums::view::render_enum_view;
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;

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

fn sample_project() -> Project {
    let mut p = Project::new("B1 Test Project");
    p.defs.enums = vec![EnumDef {
        id: EnumDefId(1),
        name: "ItemRarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    }];
    p.defs.tilesets = vec![TilesetDef {
        id: TilesetDefId(1),
        name: "WorldTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
    }];
    p
}

fn render_headless(
    app: &mut ShellApp,
    mode: AppMode,
) -> (Vec<Rect>, Vec<Shape>, Vec<(String, Rect)>) {
    app.active_mode = mode;
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

    let mut output = ctx.run_ui(raw_input, |ui| match mode {
        AppMode::Enums => render_enum_view(app, ui),
        AppMode::Tilesets => render_tileset_view(app, ui),
        _ => {}
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
fn test_b1_attribute_card_layout() {
    let p = sample_project();
    let mut enum_app = ShellApp::new(p.clone());
    enum_app.enum_draft = p.defs.enums.clone();
    enum_app.selected_enum_def = Some(EnumDefId(1));
    let (enum_rects, _, _) = render_headless(&mut enum_app, AppMode::Enums);

    let enum_id_field = enum_rects
        .iter()
        .find(|r| r.min.x > 390.0 && r.min.x < 420.0 && (r.height() - 32.0).abs() < 1.0)
        .expect("Enum identifier field");
    let expected_ctrl_x = enum_id_field.min.x;

    let mut ts_app = ShellApp::new(p);
    ts_app.selected_tileset_def = Some(TilesetDefId(1));
    ts_app.tileset_view_state.select_tileset(TilesetDefId(1));
    let (ts_rects, ts_shapes, ts_texts) = render_headless(&mut ts_app, AppMode::Tilesets);

    // 1. Verify row order: "Source image" < "Identifier" < "Tiles layout"
    let src_pos = ts_texts
        .iter()
        .position(|(t, _)| t == "Source image")
        .expect("Source image label");
    let id_pos = ts_texts
        .iter()
        .position(|(t, _)| t == "Identifier")
        .expect("Identifier label");
    let layout_pos = ts_texts
        .iter()
        .position(|(t, _)| t == "Tiles layout")
        .expect("Tiles layout label");

    assert!(
        src_pos < id_pos,
        "Source image (index {src_pos}) must appear before Identifier (index {id_pos})"
    );
    assert!(
        id_pos < layout_pos,
        "Identifier (index {id_pos}) must appear before Tiles layout (index {layout_pos})"
    );

    // 2. Attribute card bounds
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
        .expect("Attribute card 272..1354");
    let card_inner_right = ts_attr_card.rect.max.x - 12.0 - ts_attr_card.stroke.width;

    // 3. Clear button right edge == card inner right
    let clear_btn = ts_rects
        .iter()
        .find(|r| {
            (r.width() - 32.0).abs() < 0.5
                && (r.height() - 32.0).abs() < 0.5
                && (r.max.x - card_inner_right).abs() < 1.0
        })
        .expect("Clear button 32x32 at card inner right");

    // 4. Reload button (32x32) sits 6px left of Clear button
    let reload_btn = ts_rects
        .iter()
        .find(|r| {
            (r.width() - 32.0).abs() < 0.5
                && (r.height() - 32.0).abs() < 0.5
                && (clear_btn.min.x - r.max.x - 6.0).abs() < 1.0
        })
        .expect("Reload button 32x32 6px left of Clear button");
    assert!((clear_btn.min.x - reload_btn.max.x - 6.0).abs() < 1.0);

    // 5. Source image button starts at expected_ctrl_x, height 32, right edge 6px left of reload
    let src_btn = ts_rects
        .iter()
        .find(|r| {
            (r.min.x - expected_ctrl_x).abs() < 1.0
                && (r.height() - 32.0).abs() < 1.0
                && (reload_btn.min.x - r.max.x - 6.0).abs() < 1.0
        })
        .expect("Source image button starting at expected_ctrl_x and ending 6px before reload");
    assert!((src_btn.min.x - expected_ctrl_x).abs() < 1.0);

    // 6. Tiles layout has all 4 small fields on the same line at 1366x768
    let mut small_fields: Vec<Rect> = ts_rects
        .iter()
        .filter(|r| (r.width() - 72.0).abs() < 1.0 && (r.height() - 32.0).abs() < 1.0)
        .copied()
        .collect();
    small_fields.sort_by(|a, b| a.min.x.partial_cmp(&b.min.x).unwrap());
    small_fields
        .dedup_by(|a, b| (a.min.x - b.min.x).abs() < 0.5 && (a.min.y - b.min.y).abs() < 0.5);
    assert_eq!(
        small_fields.len(),
        4,
        "Tiles layout must have 4 unique small fields of width 72, found {}",
        small_fields.len()
    );
    // All 4 small fields share the same top/y coordinate
    let y0 = small_fields[0].min.y;
    for sf in &small_fields {
        assert!(
            (sf.min.y - y0).abs() < 1.0,
            "Small fields must share the same y coordinate on wide layout"
        );
    }
}

#[test]
fn test_b1_source_image_states() {
    let mut p = sample_project();
    // 1. Missing image
    p.defs.tilesets[0].path = "assets/missing_texture.png".to_string();

    let mut ts_app = ShellApp::new(p.clone());
    ts_app.selected_tileset_def = Some(TilesetDefId(1));
    ts_app.tileset_view_state.select_tileset(TilesetDefId(1));
    let (_, _, ts_texts) = render_headless(&mut ts_app, AppMode::Tilesets);

    let missing_label = ts_texts
        .iter()
        .find(|(t, _)| t == "missing_texture.png")
        .expect("Missing file name text");
    assert_eq!(missing_label.0, "missing_texture.png");

    // 2. Empty image
    p.defs.tilesets[0].path = String::new();
    let mut ts_app_empty = ShellApp::new(p);
    ts_app_empty.selected_tileset_def = Some(TilesetDefId(1));
    ts_app_empty
        .tileset_view_state
        .select_tileset(TilesetDefId(1));
    let (_, _, ts_texts_empty) = render_headless(&mut ts_app_empty, AppMode::Tilesets);

    assert!(
        ts_texts_empty.iter().any(|(t, _)| t == "Choose image..."),
        "Empty state must show 'Choose image...'"
    );
}

#[test]
fn test_b1_layout_wrapping_below_760() {
    let p = sample_project();
    let mut ts_app = ShellApp::new(p);
    ts_app.selected_tileset_def = Some(TilesetDefId(1));
    ts_app.tileset_view_state.select_tileset(TilesetDefId(1));
    ts_app.active_mode = AppMode::Tilesets;

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    ts_app.assets.set_project_dir(Some(repo_root.to_path_buf()));
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Screen width 700 (< 760)
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(700.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_tileset_view(&mut ts_app, ui);
    });
    output.textures_delta.clear();

    let mut rects = Vec::new();
    let mut shapes = Vec::new();
    let mut text_items = Vec::new();
    for clipped in &output.shapes {
        walk_shapes(&clipped.shape, &mut rects, &mut shapes, &mut text_items);
    }

    let mut small_fields: Vec<Rect> = rects
        .iter()
        .filter(|r| (r.width() - 72.0).abs() < 1.0 && (r.height() - 32.0).abs() < 1.0)
        .copied()
        .collect();
    small_fields.sort_by(|a, b| {
        a.min
            .y
            .partial_cmp(&b.min.y)
            .unwrap()
            .then(a.min.x.partial_cmp(&b.min.x).unwrap())
    });
    small_fields
        .dedup_by(|a, b| (a.min.x - b.min.x).abs() < 0.5 && (a.min.y - b.min.y).abs() < 0.5);

    assert_eq!(small_fields.len(), 4, "Must have 4 small fields");
    // W and H on line 1, Spacing and Padding on line 2
    assert!(
        (small_fields[0].min.y - small_fields[1].min.y).abs() < 1.0,
        "W and H must share row 1"
    );
    assert!(
        (small_fields[2].min.y - small_fields[3].min.y).abs() < 1.0,
        "Spacing and Padding must share row 2"
    );
    assert!(
        small_fields[2].min.y > small_fields[0].max.y,
        "Row 2 must be below Row 1"
    );
}
