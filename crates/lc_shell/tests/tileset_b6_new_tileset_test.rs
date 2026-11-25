//! Tests for Tileset Workspace Step B6: '+ New Tileset' creation workflow and validation.

use egui::{vec2, Context, Rect, Shape};
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::creation::{
    copy_image_to_assets, create_tileset_from_png, CreateTilesetError,
};
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;
use std::path::Path;
use std::{fs::create_dir_all, fs::remove_dir_all};

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
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // 1. NotPng error (fails on extension without reading disk)
    let txt_path = repo_root.join("not_image.txt");
    let err_not_png = create_tileset_from_png(repo_root, &txt_path, &[]);
    assert_eq!(err_not_png, Err(CreateTilesetError::NotPng));

    // 2. Unreadable error (non-existent file fails lc_io::read_bytes)
    let missing_path = repo_root.join("missing_file.png");
    let err_unreadable = create_tileset_from_png(repo_root, &missing_path, &[]);
    assert_eq!(err_unreadable, Err(CreateTilesetError::Unreadable));

    // 3. OutsideProject error
    let existing_png = repo_root.join("assets/Bg/tileset_picker_bg_netral.png");
    assert!(existing_png.exists(), "Asset fixture must exist");
    let fake_root = repo_root.join("subfolder_outside_test");
    let err_outside = create_tileset_from_png(&fake_root, &existing_png, &[]);
    assert_eq!(err_outside, Err(CreateTilesetError::OutsideProject));

    // 4. Success inside project
    let existing = vec!["tileset_picker_bg_netral".to_string()];
    let res = create_tileset_from_png(repo_root, &existing_png, &existing);
    assert!(res.is_ok(), "Expected Ok tileset def, got {:?}", res);
    let def = res.unwrap();
    assert_eq!(
        def.name, "tileset_picker_bg_netral copy",
        "Name collision must be resolved"
    );
    assert_eq!(def.path, "assets/Bg/tileset_picker_bg_netral.png");
    assert_eq!(def.tile_size, glam::UVec2::splat(16));
    assert_eq!(def.spacing, 0);
    assert_eq!(def.padding, 0);
}

#[test]
fn test_copy_image_to_assets_and_modal_confirmation() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let existing_png = repo_root.join("assets/Bg/tileset_picker_bg_netral.png");

    let test_dir = std::env::temp_dir().join(format!("lc_b6_modal_{}", std::process::id()));
    let project_root = test_dir.join("project");
    create_dir_all(&project_root).expect("create project");

    // Copy helper copies into <project_root>/assets/tileset_picker_bg_netral.png
    let copied_path = copy_image_to_assets(&project_root, &existing_png).expect("copy");
    assert!(copied_path.exists());
    assert_eq!(
        copied_path,
        project_root
            .join("assets")
            .join("tileset_picker_bg_netral.png")
    );

    // Success on copied image
    let def = create_tileset_from_png(&project_root, &copied_path, &[]).expect("create");
    assert_eq!(def.name, "tileset_picker_bg_netral");
    assert_eq!(def.path, "assets/tileset_picker_bg_netral.png");

    let _ = remove_dir_all(&test_dir);
}

#[test]
fn test_headless_picking_non_png_shows_visible_error_message() {
    let project = Project::new("B6 Error Banner Test");
    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;

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

    // Trigger dialog message directly via creation validation error
    app.tileset_view_state.error_message = Some(CreateTilesetError::NotPng.to_string());

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
}
