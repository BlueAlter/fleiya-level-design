//! Tests for Tileset Workspace Step B4: Atlas card fit & centering.

use egui::{vec2, Context, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::render_tileset_view;
use lc_shell::ShellApp;

fn collect_rects(shape: &Shape, out: &mut Vec<Rect>) {
    match shape {
        Shape::Rect(r) => out.push(r.rect),
        Shape::Vec(v) => {
            for s in v {
                collect_rects(s, out);
            }
        }
        _ => {}
    }
}

/// Expected fit rule from B4 prompt:
/// zoom = min((avail_w - 32) / img_w, (avail_h - 32) / img_h, 8.0). Clamped to min 0.1, max 8.0.
/// pan = center of available rect - (img_size * zoom) / 2.
fn expected_atlas_fit(available_rect: Rect, img_w: f32, img_h: f32) -> (f32, egui::Vec2) {
    let avail_w = (available_rect.width() - 32.0).max(1.0);
    let avail_h = (available_rect.height() - 32.0).max(1.0);
    let zoom = ((avail_w / img_w).min(avail_h / img_h))
        .min(8.0)
        .clamp(0.1, 8.0);
    let zoomed_size = vec2(img_w * zoom, img_h * zoom);
    let pan = available_rect.center() - zoomed_size / 2.0;
    (zoom, pan.to_vec2())
}

#[test]
fn test_b4_atlas_fit_and_centering_on_frame_1_and_switch() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mut project = Project::new("B4 Fit Centering Test");
    let ts_a = TilesetDefId(1);
    let ts_b = TilesetDefId(2);

    project.defs.tilesets.push(TilesetDef {
        id: ts_a,
        name: "TilesetA".to_string(),
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
    project.defs.tilesets.push(TilesetDef {
        id: ts_b,
        name: "TilesetB".to_string(),
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

    let mut app = ShellApp::new(project);
    app.assets.set_project_dir(Some(repo_root.to_path_buf()));
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(ts_a);

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Frame 1 render for Tileset A (16x16)
    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    output.textures_delta.clear();

    // Verify atlas card bounds
    let mut rects = Vec::new();
    for shape in &output.shapes {
        collect_rects(&shape.shape, &mut rects);
    }
    let atlas_card = rects
        .iter()
        .find(|r| {
            (r.min.x - 272.0).abs() < 1.0
                && (r.max.x - 1102.0).abs() < 1.0
                && (r.max.y - 756.0).abs() < 1.0
        })
        .copied()
        .expect("Atlas card rect");

    // The inner area of the atlas card inside its 1 px hairline stroke
    let atlas_card_inner = atlas_card.shrink(1.0);

    let (expected_zoom_a, _expected_pan_a) = expected_atlas_fit(atlas_card_inner, 16.0, 16.0);

    let cs_a = app.tileset_view_state.canvas_state_mut(ts_a);
    let actual_zoom_a = cs_a.view.zoom;
    let actual_pan_a = cs_a.view.offset;

    assert!(
        (actual_zoom_a - expected_zoom_a).abs() < 0.01,
        "Frame 1 Tileset A zoom must match expected fit zoom: actual={}, expected={}",
        actual_zoom_a,
        expected_zoom_a
    );

    let expected_center = atlas_card.center();
    let actual_center_a = egui::Pos2::new(
        actual_pan_a.x + (16.0 * actual_zoom_a) / 2.0,
        actual_pan_a.y + (16.0 * actual_zoom_a) / 2.0,
    );
    let dist_a = actual_center_a.distance(expected_center);
    assert!(
        dist_a < 1.0,
        "Frame 1 Tileset A image must be centered within 1.0 px of atlas card center: dist={}, actual={:?}, expected={:?}",
        dist_a,
        actual_center_a,
        expected_center
    );

    // Switch to Tileset B (16x16) and render Frame 1 after switch
    app.tileset_view_state.select_tileset(ts_b);
    let mut output_b = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    output_b.textures_delta.clear();

    let (expected_zoom_b, _expected_pan_b) = expected_atlas_fit(atlas_card_inner, 16.0, 16.0);
    let cs_b = app.tileset_view_state.canvas_state_mut(ts_b);
    let actual_zoom_b = cs_b.view.zoom;
    let actual_pan_b = cs_b.view.offset;

    assert!(
        (actual_zoom_b - expected_zoom_b).abs() < 0.01,
        "Frame 1 after switch Tileset B zoom must match expected fit zoom: actual={}, expected={}",
        actual_zoom_b,
        expected_zoom_b
    );

    let actual_center_b = egui::Pos2::new(
        actual_pan_b.x + (16.0 * actual_zoom_b) / 2.0,
        actual_pan_b.y + (16.0 * actual_zoom_b) / 2.0,
    );
    let dist_b = actual_center_b.distance(expected_center);
    assert!(
        dist_b < 1.0,
        "Frame 1 after switch Tileset B image must be centered within 1.0 px of atlas card center: dist={}, actual={:?}, expected={:?}",
        dist_b,
        actual_center_b,
        expected_center
    );

    // Pan and zoom away inside valid viewport bounds
    {
        let cs = app.tileset_view_state.canvas_state_mut(ts_b);
        cs.view.zoom = 2.0;
        cs.view.offset = vec2(600.0, 400.0);
    }

    // Pressing Enter must NOT reset fit or pan/zoom of the active tileset
    let mut input_enter = egui::RawInput {
        screen_rect: Some(full_rect),
        ..Default::default()
    };
    input_enter.events.push(egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output_enter = ctx.run_ui(input_enter, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output_enter.textures_delta.clear();

    let cs_enter = app.tileset_view_state.canvas_state_mut(ts_b);
    assert!(
        (cs_enter.view.zoom - 2.0).abs() < 0.01,
        "Enter key must NOT reset zoom: actual={}",
        cs_enter.view.zoom
    );
    assert!(
        (cs_enter.view.offset - vec2(600.0, 400.0)).length() < 0.5,
        "Enter key must NOT reset pan: actual={:?}",
        cs_enter.view.offset
    );

    // Press 'F' key to reset fit
    let mut input_f = egui::RawInput {
        screen_rect: Some(full_rect),
        ..Default::default()
    };
    input_f.events.push(egui::Event::Key {
        key: egui::Key::F,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output_f = ctx.run_ui(input_f, |ui| {
        render_tileset_view(&mut app, ui);
    });
    output_f.textures_delta.clear();

    let cs_f = app.tileset_view_state.canvas_state_mut(ts_b);
    assert!(
        (cs_f.view.zoom - expected_zoom_b).abs() < 0.01,
        "F key must reset zoom to fit zoom: actual={}, expected={}",
        cs_f.view.zoom,
        expected_zoom_b
    );
    let center_f = egui::Pos2::new(
        cs_f.view.offset.x + (16.0 * cs_f.view.zoom) / 2.0,
        cs_f.view.offset.y + (16.0 * cs_f.view.zoom) / 2.0,
    );
    assert!(
        center_f.distance(expected_center) < 1.0,
        "F key must reset pan to centered pan: actual={:?}, expected={:?}",
        center_f,
        expected_center
    );
}
