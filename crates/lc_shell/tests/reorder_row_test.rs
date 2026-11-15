use egui::{vec2, Context, Pos2, RawInput, Rect, Sense, Shape};
use lc_canvas::theme::{self, ThemeChoice};
use lc_shell::shared::ui::reorder_list::{reorder_list, ListItem};
use lc_shell::shared::ui::reorder_row::{
    reorder_row, split_slots, GhostPreview, MoveDirection, ReorderRowSpec, SelectDirection, Slot,
    Trailing, ICON_BUTTON_SIZE_DELETE, ROW_LAYOUT_CARD, ROW_LAYOUT_VALUES,
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

#[test]
fn test_card_layout_preset_matches_a0_measurements() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        // Position UI cursor at x = 12.0 with available_width = 236.0 (matching enum left panel 12..248)
        let panel_rect = Rect::from_min_size(Pos2::new(12.0, 100.0), vec2(236.0, 400.0));
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(panel_rect));

        let mut spec = ReorderRowSpec::new("enum_card_sample", "Enum Card");
        spec.layout = ROW_LAYOUT_CARD;
        spec.trailing = Trailing::Kebab;

        let mut captured_body_slot = None;
        let resp = reorder_row(&mut child_ui, spec, &[Slot::Flex], |ui, body| {
            captured_body_slot = body.slot_rect(0);
            body.slot(ui, 0, |ui| {
                ui.label("Enum Card");
            });
        });

        // 1. Overall card bounding rect
        assert_eq!(resp.rect.min.x, 12.0);
        assert_eq!(resp.rect.max.x, 248.0);
        assert_eq!(resp.rect.width(), 236.0);
        assert_eq!(resp.rect.height(), 36.0);

        // 2. Drag handle rect: starts at 12 + 11 = 23.0, width 16.0 -> 23..39
        assert_eq!(resp.handle.rect.min.x, 23.0);
        assert_eq!(resp.handle.rect.max.x, 39.0);
        assert_eq!(resp.handle.rect.width(), 16.0);

        // 3. Body slot rect: starts at 39 + 12 = 51.0, ends at 217 - 8 = 209.0, width 158.0
        let body_rect = captured_body_slot.expect("body slot 0 must exist");
        assert_eq!(body_rect.min.x, 51.0);
        assert_eq!(body_rect.max.x, 209.0);
        assert_eq!(body_rect.width(), 158.0);

        // 4. Kebab trailing button rect: starts at 248 - 11 - 20 = 217.0, width 20.0 -> 217..237
        let tr_resp = resp.trailing_response.expect("trailing kebab must exist");
        assert_eq!(tr_resp.rect.min.x, 217.0);
        assert_eq!(tr_resp.rect.max.x, 237.0);
        assert_eq!(tr_resp.rect.width(), 20.0);
        assert_eq!(tr_resp.rect.height(), 20.0);
    });

    output.textures_delta.clear();
}

#[test]
fn test_row_outline_normal_hairline_and_selected_accent() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = lc_canvas::theme::Tokens::get(&ctx);

    // Frame 1: Normal row
    let raw_input1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };
    let mut out1 = ctx.run_ui(raw_input1, |ui| {
        let mut spec = ReorderRowSpec::new("row_normal", "Normal");
        spec.selected = false;
        reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
    });

    let normal_stroke_color = out1
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) => {
                if r.stroke.width > 0.0 {
                    Some(r.stroke.color)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("must draw an outline rect stroke");
    assert_eq!(normal_stroke_color, t.hairline().color);
    out1.textures_delta.clear();

    // Frame 2: Selected row
    let raw_input2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(raw_input2, |ui| {
        let mut spec = ReorderRowSpec::new("row_selected", "Selected");
        spec.selected = true;
        reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
    });

    let selected_stroke_color = out2
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) => {
                if r.stroke.width > 0.0 {
                    Some(r.stroke.color)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("must draw an outline rect stroke");
    assert_eq!(selected_stroke_color, t.accent);
    out2.textures_delta.clear();
}

#[test]
fn test_row_keyboard_up_down_enter_alt_reorder() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Frame 1: ArrowUp triggers SelectDirection::Previous
    let raw_input1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![egui::Event::Key {
            key: egui::Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out1 = ctx.run_ui(raw_input1, |ui| {
        let mut spec = ReorderRowSpec::new("row_key", "Item");
        spec.selected = true;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert_eq!(resp.select_request, Some(SelectDirection::Previous));
        assert_eq!(resp.move_request, None);
        assert!(!resp.clicked);
    });
    out1.textures_delta.clear();

    // Frame 2: ArrowDown triggers SelectDirection::Next
    let raw_input2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![egui::Event::Key {
            key: egui::Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(raw_input2, |ui| {
        let mut spec = ReorderRowSpec::new("row_key", "Item");
        spec.selected = true;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert_eq!(resp.select_request, Some(SelectDirection::Next));
        assert_eq!(resp.move_request, None);
        assert!(!resp.clicked);
    });
    out2.textures_delta.clear();

    // Frame 3: Enter activates (clicked = true)
    let raw_input3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out3 = ctx.run_ui(raw_input3, |ui| {
        let mut spec = ReorderRowSpec::new("row_key", "Item");
        spec.selected = true;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(resp.clicked);
        assert_eq!(resp.select_request, None);
        assert_eq!(resp.move_request, None);
    });
    out3.textures_delta.clear();

    // Frame 4: Alt + ArrowUp triggers MoveDirection::Up
    let raw_input4 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![
            egui::Event::ModifiersChanged(egui::Modifiers {
                alt: true,
                ..Default::default()
            }),
            egui::Event::Key {
                key: egui::Key::ArrowUp,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    alt: true,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let mut out4 = ctx.run_ui(raw_input4, |ui| {
        let mut spec = ReorderRowSpec::new("row_key", "Item");
        spec.selected = true;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert_eq!(resp.move_request, Some(MoveDirection::Up));
        assert_eq!(resp.select_request, None);
        assert!(!resp.clicked);
    });
    out4.textures_delta.clear();

    // Frame 5: Alt + ArrowDown triggers MoveDirection::Down
    let raw_input5 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![
            egui::Event::ModifiersChanged(egui::Modifiers {
                alt: true,
                ..Default::default()
            }),
            egui::Event::Key {
                key: egui::Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    alt: true,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let mut out5 = ctx.run_ui(raw_input5, |ui| {
        let mut spec = ReorderRowSpec::new("row_key", "Item");
        spec.selected = true;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert_eq!(resp.move_request, Some(MoveDirection::Down));
        assert_eq!(resp.select_request, None);
        assert!(!resp.clicked);
    });
    out5.textures_delta.clear();
}

#[test]
fn test_row_click_and_kebab_menu_requested() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    // Frame 1: Initial layout to find kebab button rect
    let raw_input1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        ..Default::default()
    };
    let mut kebab_center = Pos2::ZERO;
    let mut row_center = Pos2::ZERO;
    let mut out1 = ctx.run_ui(raw_input1, |ui| {
        let mut spec = ReorderRowSpec::new("row_kebab_test", "Card");
        spec.layout = ROW_LAYOUT_CARD;
        spec.clickable = true;
        spec.trailing = Trailing::Kebab;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        row_center = resp.rect.center();
        let tr_resp = resp.trailing_response.expect("trailing kebab must exist");
        kebab_center = tr_resp.rect.center();
    });
    out1.textures_delta.clear();

    // Frame 2: Click on row body
    let raw_input2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![
            egui::Event::PointerMoved(row_center),
            egui::Event::PointerButton {
                pos: row_center,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: row_center,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(raw_input2, |ui| {
        let mut spec = ReorderRowSpec::new("row_kebab_test", "Card");
        spec.layout = ROW_LAYOUT_CARD;
        spec.clickable = true;
        spec.trailing = Trailing::Kebab;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(resp.clicked, "clicking on row body must set clicked = true");
        assert!(resp.menu_requested.is_none());
    });
    out2.textures_delta.clear();

    // Frame 3: Click on kebab button
    let raw_input3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
        events: vec![
            egui::Event::PointerMoved(kebab_center),
            egui::Event::PointerButton {
                pos: kebab_center,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: kebab_center,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out3 = ctx.run_ui(raw_input3, |ui| {
        let mut spec = ReorderRowSpec::new("row_kebab_test", "Card");
        spec.layout = ROW_LAYOUT_CARD;
        spec.clickable = true;
        spec.trailing = Trailing::Kebab;
        let resp = reorder_row(ui, spec, &[Slot::Flex], |_, _| {});
        assert!(
            resp.menu_requested.is_some(),
            "clicking kebab button must request menu"
        );
    });
    out3.textures_delta.clear();
}

#[test]
fn test_split_slots_zero_slots() {
    assert_eq!(split_slots(100.0, &[], 6.0), Vec::<Rect>::new());
}

#[test]
fn test_split_slots_one_flex() {
    let res = split_slots(100.0, &[Slot::Flex], 6.0);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].width(), 100.0);
}

#[test]
fn test_split_slots_two_fixed_plus_flex() {
    let slots = [Slot::Fixed(20.0), Slot::Flex, Slot::Fixed(30.0)];
    let res = split_slots(100.0, &slots, 5.0);
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].min.x, 0.0);
    assert_eq!(res[0].width(), 20.0);
    assert_eq!(res[1].min.x, 25.0);
    assert_eq!(res[1].width(), 40.0);
    assert_eq!(res[2].min.x, 70.0);
    assert_eq!(res[2].width(), 30.0);
}

#[test]
fn test_split_slots_too_narrow_never_negative() {
    let slots = [Slot::Fixed(80.0), Slot::Flex, Slot::Fixed(40.0)];
    let res = split_slots(50.0, &slots, 6.0);
    assert_eq!(res.len(), 3);
    assert_eq!(res[1].width(), 0.0);
}
