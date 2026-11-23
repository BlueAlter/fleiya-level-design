//! B2 Collapse toggle tests.
//!
//! Asserts 28x28 chevron toggle button in attributes header row,
//! toggling `is_attributes_collapsed`, and verified expansion of atlas card.

use egui::{vec2, Context, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
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
    let mut p = Project::new("B2 Test Project");
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

fn render_tileset(app: &mut ShellApp) -> (Vec<Rect>, Vec<Shape>, Vec<(String, Rect)>) {
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

#[test]
fn test_b2_collapse_toggle_button_and_expanded_state() {
    let p = sample_project();
    let mut app = ShellApp::new(p);
    app.selected_tileset_def = Some(TilesetDefId(1));
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    // 1. In default state, collapse toggle exists at right end of title row (28x28)
    let (rects_exp, shapes_exp, texts_exp) = render_tileset(&mut app);

    // Title row has "TILESET IDENTIFIER & ATTRIBUTES"
    let title_text = texts_exp
        .iter()
        .find(|(t, _)| t == "TILESET IDENTIFIER & ATTRIBUTES")
        .expect("Attributes title label must exist");

    // Right-aligned 28x28 button on the title row
    let toggle_btn = rects_exp
        .iter()
        .find(|r| {
            (r.width() - 28.0).abs() < 0.5
                && (r.height() - 28.0).abs() < 0.5
                && (r.center().y - title_text.1.center().y).abs() < 10.0
                && r.max.x > 1300.0
        })
        .expect("28x28 collapse toggle button must exist at right end of title row");
    assert!(toggle_btn.min.x > 1300.0);

    // Attribute card exists when expanded
    let card_exp = shapes_exp.iter().find_map(|s| match s {
        Shape::Rect(r)
            if (r.rect.min.x - 272.0).abs() < 0.5
                && (r.rect.max.x - 1354.0).abs() < 0.5
                && r.rect.min.y > title_text.1.max.y =>
        {
            Some(r.rect)
        }
        _ => None,
    });
    assert!(
        card_exp.is_some(),
        "Attribute card must exist when expanded"
    );

    let atlas_card_exp = shapes_exp
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - 272.0).abs() < 0.5 && (r.rect.max.x - 1102.0).abs() < 0.5 =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .expect("Atlas card in expanded state");

    // 2. Set collapsed state
    app.tileset_view_state.is_attributes_collapsed = true;
    let (_rects_col, shapes_col, _texts_col) = render_tileset(&mut app);

    // Attribute card shape must NOT exist when collapsed
    let card_col = shapes_col.iter().find_map(|s| match s {
        Shape::Rect(r)
            if (r.rect.min.x - 272.0).abs() < 0.5
                && (r.rect.max.x - 1354.0).abs() < 0.5
                && r.rect.min.y > title_text.1.max.y =>
        {
            Some(r.rect)
        }
        _ => None,
    });
    assert!(
        card_col.is_none(),
        "Attribute card must NOT exist when collapsed"
    );

    let atlas_card_col = shapes_col
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - 272.0).abs() < 0.5 && (r.rect.max.x - 1102.0).abs() < 0.5 =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .expect("Atlas card in collapsed state");

    // Atlas top moved up, and atlas height expanded
    assert!(
        atlas_card_col.min.y < atlas_card_exp.min.y - 50.0,
        "Atlas card top in collapsed state ({}) must be significantly higher than expanded ({})",
        atlas_card_col.min.y,
        atlas_card_exp.min.y
    );
    assert!(
        atlas_card_col.height() > atlas_card_exp.height() + 50.0,
        "Atlas card height in collapsed state ({}) must be significantly larger than expanded ({})",
        atlas_card_col.height(),
        atlas_card_exp.height()
    );
}
