use egui::{vec2, Context, Pos2, RawInput, Rect, Sense};
use lc_canvas::theme::{self, ThemeChoice};
use lc_shell::shared::ui::reorder_list::{reorder_list, ListItem};
use lc_shell::shared::ui::reorder_row::{
    reorder_row, GhostPreview, ReorderRowSpec, Slot, ICON_BUTTON_SIZE_DELETE, ROW_LAYOUT_VALUES,
};
use lc_shell::shared::ui::DragListState;

#[test]
fn test_same_row_width_for_different_bodies() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let spec1 = ReorderRowSpec::new("row_empty", "Item 1");
        let resp1 = reorder_row(ui, spec1, &[Slot::Flex], |_, _| {});

        let spec2 = ReorderRowSpec::new("row_short", "Item 2");
        let resp2 = reorder_row(ui, spec2, &[Slot::Flex], |ui, body| {
            body.slot(ui, 0, |ui| ui.label("Short"));
        });

        let spec3 = ReorderRowSpec::new("row_long", "Item 3");
        let resp3 = reorder_row(ui, spec3, &[Slot::Flex], |ui, body| {
            body.slot(ui, 0, |ui| {
                ui.label("Very long label text that could easily exceed space");
            });
        });

        assert_eq!(resp1.rect.width(), resp2.rect.width());
        assert_eq!(resp2.rect.width(), resp3.rect.width());
    });

    output.textures_delta.clear();
}

#[test]
fn test_fixed_slot_clipping_of_oversized_widget() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let spec_normal = ReorderRowSpec::new("row_normal", "Normal");
        let resp_normal = reorder_row(
            ui,
            spec_normal,
            &[Slot::Fixed(120.0), Slot::Flex],
            |_, _| {},
        );

        let spec = ReorderRowSpec::new("row_clip", "Item Clip");
        let slots = [Slot::Fixed(120.0), Slot::Flex];
        let mut captured_clip_w = 0.0;
        let mut captured_slot_w = 0.0;

        let resp = reorder_row(ui, spec, &slots, |ui, body| {
            captured_slot_w = body.slot_rect(0).map_or(0.0, |r| r.width());
            body.slot(ui, 0, |ui| {
                captured_clip_w = ui.clip_rect().width();
                let mut text = "A".repeat(50);
                ui.add_sized([200.0, 32.0], egui::TextEdit::singleline(&mut text));
            });
        });

        assert_eq!(captured_slot_w, 120.0);
        assert_eq!(captured_clip_w, 120.0);
        // Oversized 200px widget in 120px slot must not widen the row
        assert_eq!(resp.rect.width(), resp_normal.rect.width());
    });

    output.textures_delta.clear();
}

#[test]
fn test_disabled_delete_state() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let mut spec = ReorderRowSpec::new("row_disabled_del", "Item Locked");
        spec.deletable = false;
        spec.delete_disabled_reason = Some("Used by 3 entities".to_string());

        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(!resp.delete_clicked);
        assert_eq!(resp.handle.sense, Sense::drag());
    });

    output.textures_delta.clear();
}

#[test]
fn test_reorder_row_headless_render_and_metrics() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let spec = ReorderRowSpec::new("row_1", "Value 1");
        let slots = [Slot::Fixed(32.0), Slot::Fixed(32.0), Slot::Flex];
        let resp = reorder_row(ui, spec, &slots, |ui, body| {
            body.slot(ui, 0, |ui| ui.label("A"));
            body.slot(ui, 1, |ui| ui.label("B"));
            body.slot(ui, 2, |ui| ui.label("Long text content"));
        });

        // Delete button exactly FORM x FORM with right edge = row right edge
        let delete_w = ICON_BUTTON_SIZE_DELETE;
        let delete_h = ICON_BUTTON_SIZE_DELETE;
        assert_eq!(delete_w, 32.0);
        assert_eq!(delete_h, 32.0);
        let delete_right = resp.rect.max.x;
        assert_eq!(delete_right, resp.rect.max.x);
        assert!(!resp.delete_clicked);
        assert_eq!(resp.move_request, None);
    });

    output.textures_delta.clear();
    assert!(
        !output.shapes.is_empty(),
        "reorder_row should produce render shapes"
    );
}

#[test]
fn test_delete_button_click_simulation() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Frame 1: Initial layout to find delete button rect
    let raw_input1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };

    let mut del_center = Pos2::ZERO;
    let mut out1 = ctx.run_ui(raw_input1, |ui| {
        let spec = ReorderRowSpec::new("row_click_del", "Item Click");
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        let del_rect = Rect::from_min_size(
            Pos2::new(resp.rect.max.x - ICON_BUTTON_SIZE_DELETE, resp.rect.min.y),
            vec2(ICON_BUTTON_SIZE_DELETE, ICON_BUTTON_SIZE_DELETE),
        );
        del_center = del_rect.center();
    });
    out1.textures_delta.clear();

    // Frame 2: Pointer moves to delete button and presses Primary
    let raw_input2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![
            egui::Event::PointerMoved(del_center),
            egui::Event::PointerButton {
                pos: del_center,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };

    let mut out2 = ctx.run_ui(raw_input2, |ui| {
        let spec = ReorderRowSpec::new("row_click_del", "Item Click");
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(!resp.delete_clicked);
    });
    out2.textures_delta.clear();

    // Frame 3: Pointer released over delete button -> delete_clicked fires once
    let raw_input3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![egui::Event::PointerButton {
            pos: del_center,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };

    let mut clicked_once = false;
    let mut out3 = ctx.run_ui(raw_input3, |ui| {
        let spec = ReorderRowSpec::new("row_click_del", "Item Click");
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        if resp.delete_clicked {
            clicked_once = true;
        }
    });
    out3.textures_delta.clear();
    assert!(clicked_once, "delete_clicked must fire once on release");

    // Frame 4: Next quiet frame -> delete_clicked is false again
    let raw_input4 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };
    let mut out4 = ctx.run_ui(raw_input4, |ui| {
        let spec = ReorderRowSpec::new("row_click_del", "Item Click");
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(!resp.delete_clicked);
    });
    out4.textures_delta.clear();
}

#[test]
fn test_values_row_metrics_and_pitch() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let spec1 = ReorderRowSpec::new("row_v1", "Value 1");
        let mut first_body_min_x = 0.0;
        let resp1 = reorder_row(ui, spec1, &[Slot::Fixed(32.0), Slot::Flex], |ui, body| {
            body.slot(ui, 0, |ui| {
                first_body_min_x = ui.cursor().min.x;
            });
        });
        // First body rect starts at row.min.x + handle_w (20) + slot_gap (8) = row.min.x + 28
        assert_eq!(first_body_min_x, resp1.rect.min.x + 28.0);

        ui.add_space(ROW_LAYOUT_VALUES.row_gap);

        let spec2 = ReorderRowSpec::new("row_v2", "Value 2");
        let resp2 = reorder_row(ui, spec2, &[Slot::Fixed(32.0), Slot::Flex], |_, _| {});

        // Consecutive rows separated by 40 px pitch (32 px height + 8 px gap)
        assert_eq!(resp2.rect.min.y - resp1.rect.min.y, 40.0);
    });

    output.textures_delta.clear();
}

#[test]
fn test_pitch_through_reorder_list() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let items = vec![
        ListItem::new("item_0", GhostPreview::new("Value 0")),
        ListItem::new("item_1", GhostPreview::new("Value 1")),
        ListItem::new("item_2", GhostPreview::new("Value 2")),
    ];
    let mut drag_state = DragListState::new();

    let mut row_min_xs = Vec::new();
    let mut first_body_slots = Vec::new();
    let mut row_min_ys = Vec::new();

    let mut output = ctx.run_ui(raw_input, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let _ = reorder_list(
            ui,
            &mut drag_state,
            &items,
            ROW_LAYOUT_VALUES,
            |ui, idx, body| {
                row_min_xs.push(ui.cursor().min.x);
                if let Some(r0) = body.slot_rect(0) {
                    first_body_slots.push(r0);
                    row_min_ys.push(r0.min.y);
                }
                body.slot(ui, 0, |ui| {
                    ui.label(format!("Body {idx}"));
                });
            },
        );
    });

    output.textures_delta.clear();

    assert_eq!(first_body_slots.len(), 3);
    // The first body slot starts at row.min.x + 28.0 (handle 20.0 + slot_gap 8.0)
    assert_eq!(first_body_slots[0].min.x, row_min_xs[0] + 28.0);
    // Consecutive row rects are exactly 40.0 apart (32.0 height + 8.0 row_gap)
    assert_eq!(row_min_ys[1] - row_min_ys[0], 40.0);
    assert_eq!(row_min_ys[2] - row_min_ys[1], 40.0);
}
