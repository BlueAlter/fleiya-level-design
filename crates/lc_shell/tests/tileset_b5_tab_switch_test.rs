//! Tests for Tileset Workspace Step B5: Tab-switch glitch between DEF and DOCUMENT tabs.

use egui::{vec2, Context, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::view::{
    render_tileset_view, DOCUMENT_SUB_TAB, TILESET_DEF_SUB_TAB,
};
use lc_shell::ShellApp;

fn collect_all_shapes(shape: &Shape, rects: &mut Vec<Rect>, texts: &mut Vec<(String, Rect)>) {
    match shape {
        Shape::Rect(r) => rects.push(r.rect),
        Shape::Text(t) => {
            texts.push((
                t.galley.text().to_string(),
                Rect::from_min_size(t.pos, t.galley.size()),
            ));
        }
        Shape::Vec(v) => {
            for s in v {
                collect_all_shapes(s, rects, texts);
            }
        }
        _ => {}
    }
}

fn render_frame(
    ctx: &Context,
    app: &mut ShellApp,
    full_rect: Rect,
) -> (Vec<Rect>, Vec<(String, Rect)>) {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(app, ui);
        },
    );
    output.textures_delta.clear();
    let mut rects = Vec::new();
    let mut texts = Vec::new();
    for shape in &output.shapes {
        collect_all_shapes(&shape.shape, &mut rects, &mut texts);
    }
    (rects, texts)
}

fn assert_frames_match(
    frame1_rects: &[Rect],
    frame1_texts: &[(String, Rect)],
    frame4_rects: &[Rect],
    frame4_texts: &[(String, Rect)],
    tab_name: &str,
) {
    assert_eq!(
        frame1_rects.len(),
        frame4_rects.len(),
        "{} frame 1 rects count ({}) must match frame 4 ({})",
        tab_name,
        frame1_rects.len(),
        frame4_rects.len()
    );
    for (i, (r1, r4)) in frame1_rects.iter().zip(frame4_rects.iter()).enumerate() {
        let diff_min = r1.min.distance(r4.min);
        let diff_max = r1.max.distance(r4.max);
        assert!(
            diff_min <= 0.5 && diff_max <= 0.5,
            "{} rect [{}] mismatch: frame1={:?}, frame4={:?}, diff_min={}, diff_max={}",
            tab_name,
            i,
            r1,
            r4,
            diff_min,
            diff_max
        );
    }

    assert_eq!(
        frame1_texts.len(),
        frame4_texts.len(),
        "{} frame 1 text count ({}) must match frame 4 ({})",
        tab_name,
        frame1_texts.len(),
        frame4_texts.len()
    );
    for (i, ((t1, p1), (t4, p4))) in frame1_texts.iter().zip(frame4_texts.iter()).enumerate() {
        assert_eq!(t1, t4, "{} text [{}] content mismatch", tab_name, i);
        let diff_min = p1.min.distance(p4.min);
        let diff_max = p1.max.distance(p4.max);
        assert!(
            diff_min <= 0.5 && diff_max <= 0.5,
            "{} text [{}] '{}' rect mismatch: frame1={:?}, frame4={:?}",
            tab_name,
            i,
            t1,
            p1,
            p4
        );
    }
}

#[test]
fn test_b5_tab_switch_glitch_and_stability() {
    let mut project = Project::new("B5 Tab Switch Test");
    let ts_id = TilesetDefId(1);
    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "TestTileset".to_string(),
        path: String::new(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
    });

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(ts_id);
    app.tileset_view_state.active_sub_tab = TILESET_DEF_SUB_TAB;

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));

    // 1. Render DEF for 4 frames
    for _ in 0..4 {
        render_frame(&ctx, &mut app, full_rect);
    }

    // 2. Switch to DOCUMENT tab
    app.tileset_view_state.set_sub_tab(DOCUMENT_SUB_TAB);

    // Frame 1 after switch to DOCUMENT
    let (doc_f1_rects, doc_f1_texts) = render_frame(&ctx, &mut app, full_rect);

    // Assert no leftover shapes from DEF tab
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "TILE ATLAS"),
        "DOCUMENT tab frame 1 must not contain TILE ATLAS label"
    );
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "TOOLS"),
        "DOCUMENT tab frame 1 must not contain TOOLS label"
    );
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "Source image"),
        "DOCUMENT tab frame 1 must not contain Source image label"
    );

    // Render frames 2, 3, 4 of DOCUMENT
    render_frame(&ctx, &mut app, full_rect);
    render_frame(&ctx, &mut app, full_rect);
    let (doc_f4_rects, doc_f4_texts) = render_frame(&ctx, &mut app, full_rect);

    // Frame 1 after switch must match frame 4 within 0.5 px
    assert_frames_match(
        &doc_f1_rects,
        &doc_f1_texts,
        &doc_f4_rects,
        &doc_f4_texts,
        "DOCUMENT",
    );

    // Simulate scrolling on DOCUMENT
    app.tileset_view_state.doc_scroll = 88.0;

    // 3. Switch back to DEF tab (must reset scroll)
    app.tileset_view_state.set_sub_tab(TILESET_DEF_SUB_TAB);
    assert_eq!(
        app.tileset_view_state.doc_scroll, 0.0,
        "Switching sub-tab must reset doc_scroll offset"
    );

    // Frame 1 after switch back to DEF
    let (def_f1_rects, def_f1_texts) = render_frame(&ctx, &mut app, full_rect);

    // Assert no leftover shapes from DOCUMENT tab (e.g. overview doc text)
    assert!(
        def_f1_texts.iter().any(|(t, _)| t == "TILE ATLAS"),
        "DEF tab frame 1 must contain TILE ATLAS label"
    );
    assert!(
        def_f1_texts.iter().any(|(t, _)| t == "TOOLS"),
        "DEF tab frame 1 must contain TOOLS label"
    );
    assert!(
        def_f1_texts.iter().any(|(t, _)| t.contains("(Tileset)")),
        "DEF tab frame 1 must immediately contain tileset header title"
    );

    // Render frames 2, 3, 4 of DEF
    render_frame(&ctx, &mut app, full_rect);
    render_frame(&ctx, &mut app, full_rect);
    let (def_f4_rects, def_f4_texts) = render_frame(&ctx, &mut app, full_rect);

    // Frame 1 after switch back must match frame 4 within 0.5 px
    assert_frames_match(
        &def_f1_rects,
        &def_f1_texts,
        &def_f4_rects,
        &def_f4_texts,
        "DEF",
    );
}
