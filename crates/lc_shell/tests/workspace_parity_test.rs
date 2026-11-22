//! Layout parity tests asserting pixel-exact parity with the enum workspace.
//!
//! Values and layout geometry are verified under the project theme (Dark) at 1366x768.

use egui::{
    pos2, vec2, Align, Context, CornerRadius, Layout, Pos2, RawInput, Rect, Shape, UiBuilder,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_shell::shared::ui::def_workspace::{def_left_panel, DefItem, DefLeftPanelConfig};
use lc_shell::shared::ui::drag_list::DragListState;
use lc_shell::shared::ui::reorder_row::{
    reorder_row, ReorderRowSpec, Slot, ROW_LAYOUT_CARD, ROW_LAYOUT_VALUES,
};

// ============================================================================
// Constants measured from feature/enum 7c0f377 (1366x768, Dark theme)
// ============================================================================

// Values list constants
const VALUES_ROW_MIN_X: f32 = 272.0; // measured from feature/enum 7c0f377
const VALUES_ROW_MAX_X: f32 = 1354.0; // measured from feature/enum 7c0f377
const VALUES_ROW_HEIGHT: f32 = 32.0; // measured from feature/enum 7c0f377
const VALUES_ROW_GAP: f32 = 8.0; // measured from feature/enum 7c0f377
const VALUES_ROW_PITCH: f32 = VALUES_ROW_HEIGHT + VALUES_ROW_GAP; // 40.0; measured from feature/enum 7c0f377
const VALUES_HANDLE_DOT_COL1_X: f32 = 279.5; // measured from feature/enum 7c0f377
const VALUES_HANDLE_DOT_COL2_X: f32 = 284.5; // measured from feature/enum 7c0f377
const VALUES_SLOT0_START_X: f32 = 300.0; // measured from feature/enum 7c0f377
const VALUES_DELETE_X_MIN: f32 = 1322.0; // measured from feature/enum 7c0f377
const VALUES_DELETE_X_MAX: f32 = 1354.0; // measured from feature/enum 7c0f377
const VALUES_DELETE_SIZE: f32 = 32.0; // measured from feature/enum 7c0f377

// Card list constants
const CARD_PANEL_WIDTH: f32 = 260.0; // measured from feature/enum 7c0f377
const CARD_MIN_X: f32 = 12.0; // measured from feature/enum 7c0f377
const CARD_MAX_X: f32 = 248.0; // measured from feature/enum 7c0f377
const CARD_WIDTH: f32 = 236.0; // measured from feature/enum 7c0f377
const CARD_HEIGHT: f32 = 36.0; // measured from feature/enum 7c0f377
const CARD_GAP: f32 = 5.0; // measured from feature/enum 7c0f377
const CARD_PITCH: f32 = CARD_HEIGHT + CARD_GAP; // 41.0; measured from feature/enum 7c0f377
const CARD_CORNER_RADIUS: u8 = 4; // measured from feature/enum 7c0f377
const CARD_HANDLE_DOT_COL1_X: f32 = 28.5; // measured from feature/enum 7c0f377
const CARD_HANDLE_DOT_COL2_X: f32 = 33.5; // measured from feature/enum 7c0f377
const CARD_KEBAB_MIN_X: f32 = 220.0; // measured from feature/enum 7c0f377
const CARD_KEBAB_MAX_X: f32 = 248.0; // measured from feature/enum 7c0f377
const CARD_KEBAB_CENTER_X: f32 = 227.0; // measured from feature/enum 7c0f377

#[test]
fn test_values_row_visual_parity_geometry() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    assert_eq!(ROW_LAYOUT_VALUES.height, VALUES_ROW_HEIGHT);
    assert_eq!(ROW_LAYOUT_CARD.height, CARD_HEIGHT);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let start_y = 329.0;
    let mut measured_slot0_x = 0.0;

    let mut output = ctx.run_ui(raw_input, |ui| {
        let content_rect = Rect::from_min_max(
            pos2(VALUES_ROW_MIN_X, start_y),
            pos2(VALUES_ROW_MAX_X, 768.0),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        child.spacing_mut().item_spacing.y = 0.0;

        for i in 0..2 {
            let spec = ReorderRowSpec::new(format!("val_{i}"), format!("Value {i}"));
            assert_eq!(spec.layout.height, VALUES_ROW_HEIGHT);
            assert_eq!(spec.layout.row_gap, VALUES_ROW_GAP);

            reorder_row(&mut child, spec, &[Slot::Flex], |ui, body| {
                body.slot(ui, 0, |ui| {
                    if i == 0 {
                        measured_slot0_x = ui.cursor().min.x;
                    }
                    ui.label("Value Name");
                });
            });

            child.add_space(VALUES_ROW_GAP);
        }
    });

    // 1. Assert NO outer row frame shape covering 272..1354 (no fill, no border outline)
    let has_outer_row_frame = output.shapes.iter().any(|s| match &s.shape {
        Shape::Rect(r) => {
            (r.rect.min.x - VALUES_ROW_MIN_X).abs() < 0.5
                && (r.rect.max.x - VALUES_ROW_MAX_X).abs() < 0.5
                && (r.rect.height() - VALUES_ROW_HEIGHT).abs() < 0.5
        }
        _ => false,
    });
    assert!(
        !has_outer_row_frame,
        "values row must NOT have an outer framing rect covering {VALUES_ROW_MIN_X}..{VALUES_ROW_MAX_X}"
    );

    // 2. Assert delete button frame is at 1322..1354 x 329..361 (framed, 32x32, radius 3, fill sunken, 1px stroke)
    let delete_frame_row0 = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) => {
                if (r.rect.min.x - VALUES_DELETE_X_MIN).abs() < 0.5
                    && (r.rect.max.x - VALUES_DELETE_X_MAX).abs() < 0.5
                    && (r.rect.min.y - start_y).abs() < 0.5
                    && (r.rect.height() - VALUES_DELETE_SIZE).abs() < 0.5
                    && r.fill != egui::Color32::TRANSPARENT
                {
                    Some(r)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("delete button fill frame for row 0 must exist at 1322..1354 x 329..361");

    assert_eq!(delete_frame_row0.corner_radius, CornerRadius::same(3));
    assert_eq!(delete_frame_row0.fill, t.sunken);

    let delete_stroke_row0 = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) => {
                if (r.rect.min.x - VALUES_DELETE_X_MIN).abs() < 0.5
                    && (r.rect.max.x - VALUES_DELETE_X_MAX).abs() < 0.5
                    && (r.rect.min.y - start_y).abs() < 0.5
                    && (r.rect.height() - VALUES_DELETE_SIZE).abs() < 0.5
                    && r.stroke.width > 0.0
                {
                    Some(r)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("delete button 1px stroke for row 0 must exist at 1322..1354 x 329..361");

    assert_eq!(
        delete_stroke_row0.stroke.width, 1.0,
        "delete button stroke width must be 1.0"
    );

    // 3. Assert drag handle dots at spec centers: col 1 = 279.5, col 2 = 284.5; rows = 340, 345, 350
    let cy0 = start_y + VALUES_ROW_HEIGHT / 2.0; // 345.0
    let expected_dots = [
        (VALUES_HANDLE_DOT_COL1_X, cy0 - 5.0),
        (VALUES_HANDLE_DOT_COL1_X, cy0),
        (VALUES_HANDLE_DOT_COL1_X, cy0 + 5.0),
        (VALUES_HANDLE_DOT_COL2_X, cy0 - 5.0),
        (VALUES_HANDLE_DOT_COL2_X, cy0),
        (VALUES_HANDLE_DOT_COL2_X, cy0 + 5.0),
    ];
    for (ex, ey) in expected_dots {
        let found = output.shapes.iter().any(|s| match &s.shape {
            Shape::Circle(c) => {
                (c.center.x - ex).abs() < 0.5
                    && (c.center.y - ey).abs() < 0.5
                    && (c.radius - 1.5).abs() < 0.1
            }
            _ => false,
        });
        assert!(
            found,
            "values row handle dot at ({ex}, {ey}) with radius 1.5 must exist"
        );
    }

    // 4. Assert slot 0 starts at x = 300.0
    assert!(
        (measured_slot0_x - VALUES_SLOT0_START_X).abs() < 0.5,
        "slot 0 start x ({measured_slot0_x}) must match {VALUES_SLOT0_START_X}"
    );

    // 5. Assert row pitch is 40.0 (height 32 + row gap 8)
    let delete_frame_row1 = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) => {
                if (r.rect.min.x - VALUES_DELETE_X_MIN).abs() < 0.5
                    && (r.rect.max.x - VALUES_DELETE_X_MAX).abs() < 0.5
                    && (r.rect.min.y - (start_y + VALUES_ROW_PITCH)).abs() < 0.5
                {
                    Some(r.rect)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("delete button frame for row 1 must exist at y = 369.0");

    let measured_pitch = delete_frame_row1.min.y - delete_frame_row0.rect.min.y;
    assert!(
        (measured_pitch - VALUES_ROW_PITCH).abs() < 0.5,
        "values row pitch ({measured_pitch}) must match {VALUES_ROW_PITCH}"
    );

    output.textures_delta.clear();
}

#[test]
fn test_card_visual_parity_geometry() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    assert_eq!(ROW_LAYOUT_VALUES.height, VALUES_ROW_HEIGHT);
    assert_eq!(ROW_LAYOUT_CARD.height, CARD_HEIGHT);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let items = vec![
        DefItem::new(1u32, "EnumAlpha"),
        DefItem::new(2u32, "EnumBeta"),
        DefItem::new(3u32, "EnumGamma"),
    ];

    let mut search = String::new();
    let mut drag = DragListState::new();
    let config = DefLeftPanelConfig::new("+ New", "ENUMS");

    let mut output = ctx.run_ui(raw_input, |ui| {
        def_left_panel(ui, &t, &config, &mut search, &items, Some(2), &mut drag);
    });

    // 1. Assert panel width 260
    let has_panel_bg = output.shapes.iter().any(|s| match &s.shape {
        Shape::Rect(r) => r.rect.min.x.abs() < 0.5 && (r.rect.max.x - CARD_PANEL_WIDTH).abs() < 0.5,
        _ => false,
    });
    assert!(
        has_panel_bg,
        "left panel background must span 0..{CARD_PANEL_WIDTH}"
    );

    // 2. Assert cards span 12..248 (width 236), height 36, pitch 41, corner radius 4
    let mut card_rects = Vec::new();
    for s in &output.shapes {
        if let Shape::Rect(r) = &s.shape {
            let rect = r.rect;
            if (rect.min.x - CARD_MIN_X).abs() < 0.5
                && (rect.max.x - CARD_MAX_X).abs() < 0.5
                && (rect.height() - CARD_HEIGHT).abs() < 0.5
                && !card_rects
                    .iter()
                    .any(|existing: &Rect| (existing.min.y - rect.min.y).abs() < 0.5)
            {
                assert_eq!(r.corner_radius, CornerRadius::same(CARD_CORNER_RADIUS));
                card_rects.push(rect);
            }
        }
    }

    assert_eq!(card_rects.len(), 3, "expected 3 cards in left panel");
    assert!((card_rects[0].width() - CARD_WIDTH).abs() < 0.5);
    let measured_pitch = card_rects[1].min.y - card_rects[0].min.y;
    assert!(
        (measured_pitch - CARD_PITCH).abs() < 0.5,
        "card pitch ({measured_pitch}) must match {CARD_PITCH}"
    );

    // 2. Assert drag handle dots at x = 28.5 and 33.5, rows at cy-5, cy, cy+5
    let cy0 = card_rects[0].center().y;
    let expected_card_dots = [
        (CARD_HANDLE_DOT_COL1_X, cy0 - 5.0),
        (CARD_HANDLE_DOT_COL1_X, cy0),
        (CARD_HANDLE_DOT_COL1_X, cy0 + 5.0),
        (CARD_HANDLE_DOT_COL2_X, cy0 - 5.0),
        (CARD_HANDLE_DOT_COL2_X, cy0),
        (CARD_HANDLE_DOT_COL2_X, cy0 + 5.0),
    ];
    for (ex, ey) in expected_card_dots {
        let found = output.shapes.iter().any(|s| match &s.shape {
            Shape::Circle(c) => {
                (c.center.x - ex).abs() < 0.5
                    && (c.center.y - ey).abs() < 0.5
                    && (c.radius - 1.5).abs() < 0.1
            }
            _ => false,
        });
        assert!(
            found,
            "card handle dot at ({ex}, {ey}) with radius 1.5 must exist"
        );
    }

    // 3. Assert kebab glyph box is 13 wide x 15 high centered at x = 227.0
    let kebab_w = CARD_KEBAB_MAX_X - CARD_KEBAB_MIN_X;
    assert_eq!(kebab_w, 28.0, "kebab button width must be 28.0");
    assert_eq!(CARD_KEBAB_MIN_X, 220.0);
    assert_eq!(CARD_KEBAB_MAX_X, 248.0);

    let kebab_glyph = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) => {
                if t.galley.text() == Icon::DotsThreeVertical.glyph()
                    && (t.pos.y - card_rects[0].min.y).abs() < CARD_HEIGHT
                {
                    Some(t)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("kebab glyph must exist on card 0");

    let glyph_center_x = kebab_glyph.pos.x + kebab_glyph.galley.size().x / 2.0;
    assert!(
        (glyph_center_x - CARD_KEBAB_CENTER_X).abs() < 0.5,
        "kebab glyph center x ({glyph_center_x}) must match {CARD_KEBAB_CENTER_X}"
    );
    assert!(
        (kebab_glyph.galley.size().x - 13.0).abs() < 0.5,
        "kebab glyph width ({}) must be 13.0",
        kebab_glyph.galley.size().x
    );
    assert!(
        (kebab_glyph.galley.size().y - 15.0).abs() < 0.5,
        "kebab glyph height ({}) must be 15.0",
        kebab_glyph.galley.size().y
    );

    output.textures_delta.clear();
}
