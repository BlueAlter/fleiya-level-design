//! Headless geometry & layout parity tests for `AppMode::Layers` at 1366x768 (Dark theme).
//!
//! Verifies compliance with `docs/WORKSPACE_STANDARDS.md`:
//! - Left panel spans `0.0..260.0` (`LEFT_PANEL_WIDTH`)
//! - Layer cards span `12.0..248.0`, height `36.0`, corner radius `4`, pitch `41.0`
//! - Right `LAYER PREVIEW` panel sits at `1114.0..1354.0` (`240.0 px`) with `CornerRadius::ZERO`
//! - IntGrid value rows follow `ROW_LAYOUT_VALUES` (`height = 32.0`, `pitch = 40.0`, `delete = 32x32`)

use egui::epaint::{ClippedShape, RectShape, TextShape};
use egui::{vec2, Context, CornerRadius, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::id::LayerDefId;
use lc_core::layer::LayerKind;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::layers::view::render_layer_view;
use lc_shell::shared::ui::reorder_row::{ROW_LAYOUT_CARD, ROW_LAYOUT_VALUES};
use lc_shell::shared::ui::workspace_metrics::{
    CONTROL_HEIGHT_COMPACT, CONTROL_HEIGHT_FORM, LEFT_PANEL_WIDTH, WORKSPACE_PADDING,
};
use lc_shell::ShellApp;

const SCREEN_W: f32 = 1366.0;
const SCREEN_H: f32 = 768.0;
const TOL: f32 = 0.5;

#[derive(Default)]
struct CollectedShapes {
    rects: Vec<RectShape>,
    texts: Vec<TextShape>,
}

fn walk(shape: &Shape, out: &mut CollectedShapes) {
    match shape {
        Shape::Rect(r) => out.rects.push(r.clone()),
        Shape::Text(t) => out.texts.push(t.clone()),
        Shape::Vec(v) => {
            for child in v {
                walk(child, out);
            }
        }
        _ => {}
    }
}

fn collect(shapes: &[ClippedShape]) -> CollectedShapes {
    let mut out = CollectedShapes::default();
    for s in shapes {
        walk(&s.shape, &mut out);
    }
    out
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOL
}

fn setup_layer_rect_app() -> ShellApp {
    let mut project = Project::new("Layer Rect Test");
    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(1),
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![
            IntGridValueDef {
                value: 1,
                name: "Solid".to_string(),
                color: [220, 60, 60, 255],
                ..Default::default()
            },
            IntGridValueDef {
                value: 2,
                name: "Ladder".to_string(),
                color: [240, 160, 60, 255],
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(2),
        name: "Tiles".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
        ..Default::default()
    });
    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(3),
        name: "Entities".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
        ..Default::default()
    });

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Layers;
    app.layer_view_state.select_layer(LayerDefId::new(1));
    app
}

#[test]
fn layer_workspace_rects_match_workspace_standards() {
    assert_eq!(ROW_LAYOUT_CARD.height, 36.0);
    assert_eq!(ROW_LAYOUT_VALUES.height, CONTROL_HEIGHT_FORM);

    let mut app = setup_layer_rect_app();
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(SCREEN_W, SCREEN_H))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_layer_view(&mut app, ui);
    });
    output.textures_delta.clear();
    let collected = collect(&output.shapes);

    // 1. Left panel background covers 0.0..260.0 across full viewport height
    let has_left_panel = collected.rects.iter().any(|r| {
        approx(r.rect.min.x, 0.0)
            && approx(r.rect.max.x, LEFT_PANEL_WIDTH)
            && approx(r.rect.height(), SCREEN_H)
    });
    assert!(
        has_left_panel,
        "left panel rect must span x=0..{LEFT_PANEL_WIDTH}, height={SCREEN_H}"
    );

    // 2. Left panel layer cards span x=12..248, height=36, CornerRadius(4), pitch=41
    let mut cards: Vec<Rect> = Vec::new();
    for r in &collected.rects {
        if approx(r.rect.min.x, WORKSPACE_PADDING)
            && approx(r.rect.max.x, LEFT_PANEL_WIDTH - WORKSPACE_PADDING)
            && approx(r.rect.height(), 36.0)
            && !cards.iter().any(|c| approx(c.min.y, r.rect.min.y))
        {
            assert_eq!(
                r.corner_radius,
                CornerRadius::same(4),
                "left panel card must have corner radius 4"
            );
            cards.push(r.rect);
        }
    }
    cards.sort_by(|a, b| a.min.y.total_cmp(&b.min.y));
    assert_eq!(cards.len(), 3, "expected 3 layer cards in left panel");
    assert!(
        approx(cards[1].min.y - cards[0].min.y, 41.0),
        "layer card vertical pitch must be 41.0px, got {}",
        cards[1].min.y - cards[0].min.y
    );

    // 3. Left panel cards include 1:1 (22x22) icon container
    let has_square_left_icon = collected.rects.iter().any(|r| {
        r.rect.min.x > WORKSPACE_PADDING
            && r.rect.max.x < LEFT_PANEL_WIDTH
            && approx(r.rect.width(), 22.0)
            && approx(r.rect.height(), 22.0)
    });
    assert!(
        has_square_left_icon,
        "left panel layer cards must include a 1x1 (22x22) icon container"
    );

    // 4. Right LAYER PREVIEW column sits cleanly at x=1114..1354 (240px) with CornerRadius::ZERO
    let content_min_x = LEFT_PANEL_WIDTH + WORKSPACE_PADDING;
    let content_max_x = SCREEN_W - WORKSPACE_PADDING;
    let right_preview_min_x = content_max_x - 240.0;
    let has_right_preview = collected.rects.iter().any(|r| {
        approx(r.rect.min.x, right_preview_min_x)
            && approx(r.rect.max.x, content_max_x)
            && r.rect.height() > 120.0
            && r.corner_radius == CornerRadius::ZERO
    });
    assert!(
        has_right_preview,
        "right LAYER PREVIEW container must render cleanly at x=1114..1354 with CornerRadius::ZERO"
    );

    // 5. Sub-tabs align with left toolbar at y=12 and height 28px
    let has_sub_tab_strip = collected.rects.iter().any(|r| {
        r.rect.min.x >= content_min_x - TOL
            && r.rect.max.x <= content_max_x + TOL
            && approx(r.rect.min.y, WORKSPACE_PADDING)
            && approx(r.rect.height(), CONTROL_HEIGHT_COMPACT)
    });
    assert!(
        has_sub_tab_strip,
        "sub-tab strip must sit at y={WORKSPACE_PADDING} with height={CONTROL_HEIGHT_COMPACT}"
    );

    // 6. Center SectionContainers align to content_min_x (272.0) and use CornerRadius::ZERO
    let section_rects: Vec<&RectShape> = collected
        .rects
        .iter()
        .filter(|r| {
            approx(r.rect.min.x, content_min_x)
                && r.rect.max.x <= right_preview_min_x
                && r.rect.width() > 500.0
                && r.rect.height() >= 28.0
        })
        .collect();
    assert!(
        !section_rects.is_empty(),
        "expected wide SectionContainer rects in the center column"
    );
    for r in section_rects {
        assert_eq!(
            r.corner_radius,
            CornerRadius::ZERO,
            "SectionContainer must use CornerRadius::ZERO, got {:?}",
            r.corner_radius
        );
    }

    // 7. IntGrid value delete buttons (32x32) stack at pitch 40.0px
    let mut delete_btns: Vec<Rect> = collected
        .rects
        .iter()
        .filter(|r| {
            r.rect.min.x > right_preview_min_x - 120.0
                && r.rect.max.x < right_preview_min_x
                && approx(r.rect.width(), CONTROL_HEIGHT_FORM)
                && approx(r.rect.height(), CONTROL_HEIGHT_FORM)
                && r.corner_radius == CornerRadius::same(3)
                && r.stroke.width > 0.0
        })
        .map(|r| r.rect)
        .collect();
    delete_btns.sort_by(|a, b| a.min.y.total_cmp(&b.min.y));
    delete_btns.dedup_by(|a, b| approx(a.min.y, b.min.y));
    assert!(
        delete_btns.len() >= 2,
        "expected at least 2 IntGrid value delete buttons (32x32), got {}; rects={:?}",
        delete_btns.len(),
        delete_btns
    );
    assert!(
        approx(delete_btns[1].min.y - delete_btns[0].min.y, 40.0),
        "IntGrid value row pitch must be 40.0px, got {}",
        delete_btns[1].min.y - delete_btns[0].min.y
    );

    // 8. Required LDtk-aligned labels visible in first frame
    let texts: Vec<&str> = collected.texts.iter().map(|t| t.galley.text()).collect();
    for expected in [
        "LAYER DEFINITIONS (3)",
        "LAYERS DEF",
        "AUTO TILE",
        "DOCUMENT",
        "GENERAL LAYER SETTINGS",
        "Type",
        "Layer identifier",
        "Grid size",
        "DISPLAY OPTIONS",
        "INTGRID SETTINGS",
        "Values interface icons",
        "Grid values",
        "LAYER PREVIEW",
    ] {
        assert!(
            texts.contains(&expected),
            "expected label `{expected}` in rendered output; got sample: {:?}",
            &texts[..texts.len().min(35)]
        );
    }
}
