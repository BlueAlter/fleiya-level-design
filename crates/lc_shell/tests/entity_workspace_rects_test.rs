//! Headless geometry & layout parity tests for `AppMode::Entities` at 1366x768 (Dark theme).
//!
//! Verifies compliance with `docs/WORKSPACE_STANDARDS.md`:
//! - Left panel spans `0.0..260.0` (`LEFT_PANEL_WIDTH`)
//! - Entity cards span `12.0..248.0`, height `36.0`, corner radius `4`, pitch `41.0`
//! - Content section containers use `CornerRadius::ZERO` and align within `272.0..1354.0`
//! - Footer bar renders across `260.0..1366.0` at bottom `40.0` px (`728.0..768.0`)

use egui::epaint::{ClippedShape, RectShape, TextShape};
use egui::{vec2, Context, CornerRadius, Pos2, RawInput, Rect, Shape};
use glam::Vec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{EntityDef, EntityRenderMode};
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_core::id::EntityDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::entities::view::render_entity_view;
use lc_shell::shared::ui::reorder_row::{ROW_LAYOUT_CARD, ROW_LAYOUT_VALUES};
use lc_shell::shared::ui::workspace_metrics::{
    CONTROL_HEIGHT_FORM, LEFT_PANEL_WIDTH, WORKSPACE_PADDING,
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

fn setup_entity_rect_app() -> ShellApp {
    let mut project = Project::new("Entity Rect Test");
    for (id, name, color) in [
        (1, "Player", [96, 165, 250, 255]),
        (2, "Slime", [148, 200, 120, 255]),
        (3, "Chest", [240, 160, 60, 255]),
    ] {
        let mut def = EntityDef::new(EntityDefId::new(id), name);
        def.color = color;
        def.render_mode = EntityRenderMode::Rectangle;
        def.pivot = Vec2::new(0.5, 1.0);
        if id == 1 {
            def.fields = vec![
                FieldDef {
                    name: "health".to_string(),
                    field_type: FieldType::Int,
                    default_value: Some(FieldValue::Int(100)),
                    ..Default::default()
                },
                FieldDef {
                    name: "speed".to_string(),
                    field_type: FieldType::Float,
                    default_value: Some(FieldValue::Float(4.5)),
                    ..Default::default()
                },
            ];
        }
        project.defs.entities.push(def);
    }

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Entities;
    app.entity_view_state.select_entity(EntityDefId::new(1));
    app.entity_view_state.selected_field_idx = Some(0);
    app
}

#[test]
fn entity_workspace_rects_match_workspace_standards() {
    assert_eq!(ROW_LAYOUT_CARD.height, 36.0);
    assert_eq!(ROW_LAYOUT_VALUES.height, CONTROL_HEIGHT_FORM);

    let mut app = setup_entity_rect_app();
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(SCREEN_W, SCREEN_H))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_entity_view(&mut app, ui);
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

    // 2. Left panel entity cards span x=12..248, height=36, CornerRadius(4), pitch=41
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
    assert_eq!(cards.len(), 3, "expected 3 entity cards in left panel");
    assert!(
        approx(cards[1].min.y - cards[0].min.y, 41.0),
        "entity card vertical pitch must be 41.0px, got {}",
        cards[1].min.y - cards[0].min.y
    );

    // 3. Footer is removed and left panel entity cards render 1x1 (22x22) bound tile/color icons
    let has_footer = collected.rects.iter().any(|r| {
        approx(r.rect.min.x, LEFT_PANEL_WIDTH)
            && approx(r.rect.max.x, SCREEN_W)
            && approx(r.rect.min.y, SCREEN_H - 40.0)
            && approx(r.rect.max.y, SCREEN_H)
    });
    assert!(
        !has_footer,
        "entity workspace bottom footer must be removed"
    );

    let has_square_left_icon = collected.rects.iter().any(|r| {
        r.rect.min.x > WORKSPACE_PADDING
            && r.rect.max.x < LEFT_PANEL_WIDTH
            && approx(r.rect.width(), 22.0)
            && approx(r.rect.height(), 22.0)
    });
    assert!(
        has_square_left_icon,
        "left panel entity cards must include a 1x1 (22x22) icon container"
    );

    // 4. Right LIVE PREVIEW column sits cleanly at x=1114..1354 (240px) with CornerRadius::ZERO
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
        "right LIVE PREVIEW container must render cleanly at x=1114..1354 with CornerRadius::ZERO"
    );

    // 5. Center SectionContainers align to content_min_x (272.0) and use CornerRadius::ZERO
    let section_rects: Vec<&RectShape> = collected
        .rects
        .iter()
        .filter(|r| {
            approx(r.rect.min.x, content_min_x)
                && r.rect.max.x <= right_preview_min_x
                && r.rect.width() > 500.0
                && r.rect.height() >= 28.0
                && r.rect.min.y < SCREEN_H - 40.0
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
            "SectionContainer / content cards must use CornerRadius::ZERO, got {:?}",
            r.corner_radius
        );
    }

    // 6. Key top-section labels, Right LIVE PREVIEW, and ENTITY SETTINGS are visible at 1366x768
    let texts: Vec<&str> = collected.texts.iter().map(|t| t.galley.text()).collect();
    for expected in [
        "ENTITY DEFINITIONS (3)",
        "ENTITY DEF",
        "DOCUMENT",
        "Entity identifier",
        "Show identifier",
        "LIVE PREVIEW",
        "ENTITY SETTINGS",
        "Misc options",
        "Editor visual",
    ] {
        assert!(
            texts.contains(&expected),
            "expected label `{expected}` in rendered output; got sample: {:?}",
            &texts[..texts.len().min(30)]
        );
    }

    // 7. Collapsing ENTITY SETTINGS keeps LIVE PREVIEW visible and shows 2-column ENTITY VALUES
    app.entity_view_state.collapse_settings = true;
    let raw_input_2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(SCREEN_W, SCREEN_H))),
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(raw_input_2, |ui| {
        render_entity_view(&mut app, ui);
    });
    out2.textures_delta.clear();
    let collected2 = collect(&out2.shapes);
    let texts2: Vec<&str> = collected2.texts.iter().map(|t| t.galley.text()).collect();
    for expected in [
        "LIVE PREVIEW",
        "ENTITY SETTINGS",
        "ENTITY VALUES (2 DEFINED)",
        "+ SINGLE VALUE",
        "+ ARRAY",
        "GENERAL SETTINGS",
        "VALUE SPECIFICATIONS",
        "DISPLAY",
        "Limits",
    ] {
        assert!(
            texts2.contains(&expected),
            "expected `{expected}` when ENTITY SETTINGS collapsed; got: {:?}",
            &texts2[..texts2.len().min(45)]
        );
    }

    // Verify fixed-height right config card matches left column height
    let has_fixed_right_card = collected2.rects.iter().any(|r| {
        r.rect.min.x > 480.0
            && r.rect.max.x < right_preview_min_x
            && r.rect.width() > 220.0
            && (r.rect.height() - 376.0).abs() < 2.0
            && r.corner_radius == CornerRadius::ZERO
    });
    assert!(
        has_fixed_right_card,
        "right Entity Values config card must keep fixed scrollable height ~376px"
    );

    // Verify field type pill badge uses semantic field_type_color (Int = rgb(55, 135, 155))
    let int_color =
        lc_shell::features::entities::field_type_grid::field_type_color(&FieldType::Int);
    assert!(
        collected2
            .rects
            .iter()
            .any(|r| r.stroke.color == int_color && approx(r.rect.height(), 20.0)),
        "field type identifier pill must use the matching popup semantic color"
    );

    // 8. Equal-width symmetric + SINGLE VALUE and + ARRAY buttons
    let single_btn = collected2
        .texts
        .iter()
        .find(|t| t.galley.text() == "+ SINGLE VALUE")
        .expect("+ SINGLE VALUE button text");
    let array_btn = collected2
        .texts
        .iter()
        .find(|t| t.galley.text() == "+ ARRAY")
        .expect("+ ARRAY button text");
    assert!(
        approx(single_btn.pos.y, array_btn.pos.y),
        "+ SINGLE VALUE and + ARRAY must sit on the exact same horizontal line"
    );

    // 9. Collapsing ENTITY VALUES hides its 2-column body while keeping both toggle headers
    app.entity_view_state.collapse_fields = true;
    let raw_input_3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(SCREEN_W, SCREEN_H))),
        ..Default::default()
    };
    let mut out3 = ctx.run_ui(raw_input_3, |ui| {
        render_entity_view(&mut app, ui);
    });
    out3.textures_delta.clear();
    let collected3 = collect(&out3.shapes);
    let texts3: Vec<&str> = collected3.texts.iter().map(|t| t.galley.text()).collect();
    assert!(texts3.contains(&"ENTITY SETTINGS"));
    assert!(texts3.contains(&"ENTITY VALUES (2 DEFINED)"));
    assert!(!texts3.contains(&"GENERAL SETTINGS"));
}
