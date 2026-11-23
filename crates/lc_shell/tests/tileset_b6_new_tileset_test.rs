//! Tests for Tileset Workspace Step B6: '+ New Tileset' creation workflow and validation.

use egui::{vec2, Context, Rect, Shape};
use image::{ImageBuffer, Rgba};
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::creation::{
    copy_image_to_assets, create_tileset_from_png, CreateTilesetError,
};
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;
use std::io::Cursor;

fn create_valid_png(w: u32, h: u32) -> Vec<u8> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(w, h, Rgba([100, 150, 200, 255]));
    let mut bytes = Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png)
        .expect("PNG encode");
    bytes.into_inner()
}

fn collect_texts(shape: &Shape, texts: &mut Vec<String>) {
    match shape {
        Shape::Text(t) => texts.push(t.galley.text().to_string()),
        Shape::Vec(v) => {
            for s in v {
                collect_texts(s, texts);
            }
        }
        _ => {}
    }
}

#[test]
fn test_create_tileset_from_png_error_variants_and_success() {
    let test_dir = std::env::temp_dir().join(format!("lc_b6_unit_{}", std::process::id()));
    let project_root = test_dir.join("project");
    let outside_dir = test_dir.join("outside");
    std::fs::create_dir_all(&project_root).expect("create project");
    std::fs::create_dir_all(&outside_dir).expect("create outside");

    // 1. NotPng error
    let txt_path = project_root.join("not_image.txt");
    std::fs::write(&txt_path, b"hello text").expect("write txt");
    let err_not_png = create_tileset_from_png(&project_root, &txt_path, &[]);
    assert_eq!(err_not_png, Err(CreateTilesetError::NotPng));

    // 2. Unreadable error (non-existent file or corrupted bytes)
    let missing_path = project_root.join("missing.png");
    let err_unreadable = create_tileset_from_png(&project_root, &missing_path, &[]);
    assert_eq!(err_unreadable, Err(CreateTilesetError::Unreadable));

    let corrupted_path = project_root.join("corrupted.png");
    std::fs::write(&corrupted_path, b"not really a png").expect("write corrupt");
    let err_corrupt = create_tileset_from_png(&project_root, &corrupted_path, &[]);
    assert_eq!(err_corrupt, Err(CreateTilesetError::Unreadable));

    // 3. OutsideProject error
    let outside_png = outside_dir.join("outside.png");
    std::fs::write(&outside_png, create_valid_png(32, 32)).expect("write outside png");
    let err_outside = create_tileset_from_png(&project_root, &outside_png, &[]);
    assert_eq!(err_outside, Err(CreateTilesetError::OutsideProject));

    // 4. Success inside project
    let inside_png = project_root.join("terrain.png");
    std::fs::write(&inside_png, create_valid_png(64, 64)).expect("write inside png");
    let existing = vec!["terrain".to_string()];
    let res = create_tileset_from_png(&project_root, &inside_png, &existing);
    assert!(res.is_ok(), "Expected Ok tileset def, got {:?}", res);
    let def = res.unwrap();
    assert_eq!(def.name, "terrain copy", "Name collision must be resolved");
    assert_eq!(def.path, "terrain.png");
    assert_eq!(def.tile_size, glam::UVec2::splat(16));
    assert_eq!(def.spacing, 0);
    assert_eq!(def.padding, 0);

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_copy_image_to_assets_and_modal_confirmation() {
    let test_dir = std::env::temp_dir().join(format!("lc_b6_modal_{}", std::process::id()));
    let project_root = test_dir.join("project");
    let outside_dir = test_dir.join("outside");
    std::fs::create_dir_all(&project_root).expect("create project");
    std::fs::create_dir_all(&outside_dir).expect("create outside");

    let outside_png = outside_dir.join("dungeon.png");
    std::fs::write(&outside_png, create_valid_png(48, 48)).expect("write png");

    // Copy helper copies into <project_root>/assets/dungeon.png
    let copied_path = copy_image_to_assets(&project_root, &outside_png).expect("copy");
    assert!(copied_path.exists());
    assert_eq!(copied_path, project_root.join("assets").join("dungeon.png"));

    // Success on copied image
    let def = create_tileset_from_png(&project_root, &copied_path, &[]).expect("create");
    assert_eq!(def.name, "dungeon");
    assert_eq!(def.path, "assets/dungeon.png");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_headless_picking_non_png_shows_visible_error_message() {
    let test_dir = std::env::temp_dir().join(format!("lc_b6_headless_{}", std::process::id()));
    std::fs::create_dir_all(&test_dir).expect("create dir");
    let not_png = test_dir.join("invalid.txt");
    std::fs::write(&not_png, b"invalid").expect("write txt");

    let project = Project::new("B6 Error Banner Test");
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.project_path = Some(test_dir.join("test.lcproj"));

    // Simulate dialog manager sending NewTileset with non-PNG
    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Initial render
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    out1.textures_delta.clear();

    // Trigger dialog message directly via creation validation
    let existing: Vec<String> = app
        .project
        .defs
        .tilesets
        .iter()
        .map(|t| t.name.clone())
        .collect();
    let err = create_tileset_from_png(&test_dir, &not_png, &existing).unwrap_err();
    app.tileset_view_state.error_message = Some(err.to_string());

    // Render frame with error
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    out2.textures_delta.clear();

    let mut texts = Vec::new();
    for shape in &out2.shapes {
        collect_texts(&shape.shape, &mut texts);
    }

    assert!(
        texts.iter().any(|t| t.contains("File must be a PNG image")),
        "Error message must be visible in UI text output: {:?}",
        texts
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}
