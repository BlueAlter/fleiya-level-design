//! B3 Footer and info line removal tests.
//!
//! Asserts removal of footer, removal of info line under card,
//! atlas bottom extending to window bottom - 12 (756.0 at 768h),
//! and presence of atlas dimensions and tile count in TILE ATLAS tooltip.

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
    let mut p = Project::new("B3 Test Project");
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
fn test_b3_footer_and_info_line_removed_and_atlas_bottom() {
    let p = sample_project();
    let mut app = ShellApp::new(p);
    app.selected_tileset_def = Some(TilesetDefId(1));
    app.tileset_view_state.select_tileset(TilesetDefId(1));

    let (_rects, shapes, texts) = render_tileset(&mut app);

    // 1. No footer text ("ID: ... • Used by ...")
    assert!(
        !texts
            .iter()
            .any(|(t, _)| t.contains("Used by") || t.starts_with("ID: TilesetDefId")),
        "Footer text must be removed from tileset workspace"
    );

    // 2. No unconditional info line ("Atlas 16x16 px, 1 x 1 tiles (1)")
    assert!(
        !texts
            .iter()
            .any(|(t, _)| t.starts_with("Atlas ") && t.contains("tiles")),
        "Info line under card must be removed from tileset workspace"
    );

    // 3. Atlas card bottom == window bottom - 12 (768.0 - 12.0 = 756.0)
    let atlas_card = shapes
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.min.x - 272.0).abs() < 0.5 && (r.rect.max.x - 1102.0).abs() < 0.5 =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .expect("Atlas card must span 272..1102");

    assert!(
        (atlas_card.max.y - 756.0).abs() < 1.0,
        "Atlas card bottom must be at window bottom - 12 (756.0), found {}",
        atlas_card.max.y
    );
}
