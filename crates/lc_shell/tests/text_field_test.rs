//! Headless integration tests for the shared text_field component.
//!
//! Verifies geometry, padding, alignment, long text handling, and outline contracts
//! under the project theme (Dark) at 1366x768.

use egui::epaint::ClippedShape;
use egui::{vec2, Color32, Context, Id, Pos2, RawInput, Rect, Shape};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_shell::shared::ui::reorder_row::{reorder_row, ReorderRowSpec, Slot};
use lc_shell::shared::ui::text_field::{text_field, FieldStyle};
use lc_shell::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

fn collect_text_shapes<'a>(shapes: &'a [ClippedShape], out: &mut Vec<&'a egui::epaint::TextShape>) {
    for clipped in shapes {
        collect_from_shape(&clipped.shape, out);
    }
}

fn collect_rect_shapes<'a>(shapes: &'a [ClippedShape], out: &mut Vec<&'a egui::epaint::RectShape>) {
    for clipped in shapes {
        collect_rect_from_shape(&clipped.shape, out);
    }
}

fn collect_from_shape<'a>(shape: &'a Shape, out: &mut Vec<&'a egui::epaint::TextShape>) {
    match shape {
        Shape::Text(t) => out.push(t),
        Shape::Vec(children) => {
            for child in children {
                collect_from_shape(child, out);
            }
        }
        _ => {}
    }
}

fn collect_rect_from_shape<'a>(shape: &'a Shape, out: &mut Vec<&'a egui::epaint::RectShape>) {
    match shape {
        Shape::Rect(r) => out.push(r),
        Shape::Vec(children) => {
            for child in children {
                collect_rect_from_shape(child, out);
            }
        }
        _ => {}
    }
}

#[test]
fn test_text_field_width_and_height_contracts() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut val_plain = "V0".to_string();
    let mut val_scope = "V1".to_string();
    let mut val_slot = "V2".to_string();

    let mut output = ctx.run_ui(raw_input, |ui| {
        // 1. Plain UI: fills ui.available_width()
        let available_before = ui.available_width();
        let res_plain = text_field(ui, &mut val_plain, FieldStyle::default());
        let plain_rect = res_plain.response.rect;
        assert!(
            (plain_rect.width() - available_before).abs() < 0.5,
            "plain text_field width {} must match available width {}",
            plain_rect.width(),
            available_before
        );
        assert!(
            (plain_rect.height() - CONTROL_HEIGHT_FORM).abs() < 0.5,
            "plain text_field height {} must be 32.0",
            plain_rect.height()
        );

        // 2. In ui.scope with set_width(912)
        ui.scope(|ui| {
            ui.set_width(912.0);
            let res_scope = text_field(ui, &mut val_scope, FieldStyle::default());
            let scope_rect = res_scope.response.rect;
            assert!(
                (scope_rect.width() - 912.0).abs() < 0.5,
                "scope text_field width {} must match 912.0",
                scope_rect.width()
            );
            assert!(
                (scope_rect.height() - CONTROL_HEIGHT_FORM).abs() < 0.5,
                "scope text_field height {} must be 32.0",
                scope_rect.height()
            );
        });

        // 3. In Flex slot of reorder_row (slot width 934)
        // Row non-body width is handle (20) + handle_gap (8) + trailing_gap (8) + delete (32) = 68.
        // Total row width 934 + 68 = 1002 gives a flex body slot of exactly 934.
        ui.scope(|ui| {
            ui.set_width(1002.0);
            let spec = ReorderRowSpec::new(Id::new("row_test_934"), "Test Row");
            reorder_row(ui, spec, &[Slot::Flex], |ui, body| {
                body.slot(ui, 0, |ui| {
                    let res_slot = text_field(ui, &mut val_slot, FieldStyle::default());
                    let slot_rect = res_slot.response.rect;
                    assert!(
                        (slot_rect.width() - 934.0).abs() < 0.5,
                        "slot text_field width {} must match 934.0",
                        slot_rect.width()
                    );
                    assert!(
                        (slot_rect.height() - CONTROL_HEIGHT_FORM).abs() < 0.5,
                        "slot text_field height {} must be 32.0",
                        slot_rect.height()
                    );
                });
            });
        });
    });

    output.textures_delta.clear();
}

#[test]
fn test_text_field_text_position_and_vertical_centering() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut val = "ItemName".to_string();
    let mut field_rect = Rect::ZERO;

    let mut output = ctx.run_ui(raw_input, |ui| {
        let res = text_field(ui, &mut val, FieldStyle::default());
        field_rect = res.response.rect;
    });

    let mut text_shapes = Vec::new();
    collect_text_shapes(&output.shapes, &mut text_shapes);
    output.textures_delta.clear();

    let item_text = text_shapes
        .into_iter()
        .find(|t| t.galley.job.text == "ItemName")
        .expect("text_field text 'ItemName' should be present in painted output");

    let left_padding = item_text.pos.x - field_rect.min.x;
    assert!(
        (left_padding - 8.0).abs() < 0.5,
        "text left padding ({left_padding}) must be 8.0 within 0.5 px"
    );

    let text_center_y = item_text.pos.y + item_text.galley.size().y / 2.0;
    let field_center_y = field_rect.center().y;
    assert!(
        (text_center_y - field_center_y).abs() < 0.5,
        "text vertical center ({text_center_y}) must match field center ({field_center_y}) within 0.5 px"
    );
}

#[test]
fn test_text_field_long_text_does_not_grow_width() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut long_text = "A".repeat(300);
    let mut field_rect = Rect::ZERO;

    let mut output = ctx.run_ui(raw_input, |ui| {
        ui.scope(|ui| {
            ui.set_width(500.0);
            let res = text_field(ui, &mut long_text, FieldStyle::default());
            field_rect = res.response.rect;
        });
    });
    output.textures_delta.clear();

    assert!(
        (field_rect.width() - 500.0).abs() < 0.5,
        "field width with 300 characters ({}) must remain equal to allocated width (500.0)",
        field_rect.width()
    );
    assert!(
        (field_rect.height() - CONTROL_HEIGHT_FORM).abs() < 0.5,
        "field height ({}) must remain 32.0",
        field_rect.height()
    );
}

#[test]
fn test_text_field_outline_color_single_stroke() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let badge_color = Color32::from_rgb(33, 150, 243);
    let mut val = "Legendary".to_string();
    let mut field_rect = Rect::ZERO;

    let mut output = ctx.run_ui(raw_input, |ui| {
        let res = text_field(
            ui,
            &mut val,
            FieldStyle::default().with_outline_color(badge_color),
        );
        field_rect = res.response.rect;
    });

    let mut rect_shapes = Vec::new();
    collect_rect_shapes(&output.shapes, &mut rect_shapes);
    output.textures_delta.clear();

    // Filter stroke shapes that match the field rect
    let stroke_rects: Vec<&egui::epaint::RectShape> = rect_shapes
        .into_iter()
        .filter(|r| {
            (r.rect.min.x - field_rect.min.x).abs() < 0.5
                && (r.rect.min.y - field_rect.min.y).abs() < 0.5
                && (r.rect.max.x - field_rect.max.x).abs() < 0.5
                && (r.rect.max.y - field_rect.max.y).abs() < 0.5
                && r.stroke.width > 0.0
        })
        .collect();

    assert_eq!(
        stroke_rects.len(),
        1,
        "expected exactly ONE outline stroke on the field rect, found {}",
        stroke_rects.len()
    );

    let outline = stroke_rects[0];
    assert!(
        (outline.stroke.width - 1.5).abs() < 0.01,
        "FieldOutline::Color stroke width must be 1.5 px, got {}",
        outline.stroke.width
    );
    assert_eq!(
        outline.stroke.color, badge_color,
        "FieldOutline::Color stroke color must match badge color"
    );
    assert_ne!(
        outline.stroke.color,
        t.hairline().color,
        "no hairline stroke should be drawn under the colored outline"
    );
}

#[test]
fn test_text_field_focus_draws_accent_outline() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let badge_color = Color32::from_rgb(33, 150, 243);
    let mut val = "ActiveInput".to_string();
    let mut field_rect = Rect::ZERO;

    // Frame 1: Render and request focus
    let mut frame1 = ctx.run_ui(raw_input.clone(), |ui| {
        let res = text_field(
            ui,
            &mut val,
            FieldStyle::default().with_outline_color(badge_color),
        );
        res.response.request_focus();
    });
    frame1.textures_delta.clear();

    // Frame 2: Field is now focused
    let mut output = ctx.run_ui(raw_input, |ui| {
        let res = text_field(
            ui,
            &mut val,
            FieldStyle::default().with_outline_color(badge_color),
        );
        field_rect = res.response.rect;
        assert!(
            res.response.has_focus(),
            "field must have focus in frame 2 after request_focus"
        );
    });

    let mut rect_shapes = Vec::new();
    collect_rect_shapes(&output.shapes, &mut rect_shapes);
    output.textures_delta.clear();

    let stroke_rects: Vec<&egui::epaint::RectShape> = rect_shapes
        .into_iter()
        .filter(|r| {
            (r.rect.min.x - field_rect.min.x).abs() < 0.5
                && (r.rect.min.y - field_rect.min.y).abs() < 0.5
                && (r.rect.max.x - field_rect.max.x).abs() < 0.5
                && (r.rect.max.y - field_rect.max.y).abs() < 0.5
                && r.stroke.width > 0.0
        })
        .collect();

    assert_eq!(
        stroke_rects.len(),
        1,
        "expected exactly ONE outline stroke on the focused field rect, found {}",
        stroke_rects.len()
    );

    let outline = stroke_rects[0];
    assert!(
        (outline.stroke.width - 1.0).abs() < 0.01,
        "focused field outline stroke width must be 1.0 px, got {}",
        outline.stroke.width
    );
    assert_eq!(
        outline.stroke.color, t.accent,
        "focused field outline stroke color must be t.accent"
    );
    assert_ne!(
        outline.stroke.color, badge_color,
        "focused field outline must override the normal badge color"
    );
}
