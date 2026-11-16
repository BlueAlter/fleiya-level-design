use egui::{vec2, Context, Event, PointerButton, Pos2, RawInput, Rect, Shape};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_shell::shared::ui::def_workspace::{
    def_field_row, def_footer, def_left_panel, def_panel_width, def_right_header, def_row_layout,
    def_section, format_footer_left, DefItem, DefLeftPanelAction, DefLeftPanelConfig,
    DEF_CARD_HEIGHT, DEF_HANDLE_WIDTH, DEF_KEBAB_WIDTH,
};
use lc_shell::shared::ui::DragListState;

#[test]
fn test_def_panel_width_constant() {
    assert_eq!(def_panel_width(), 260.0);
}

#[test]
fn test_def_row_layout_geometry() {
    let r = egui::Rect::from_min_size(egui::pos2(10.0, 50.0), egui::vec2(240.0, DEF_CARD_HEIGHT));
    let layout = def_row_layout(r);

    assert!(layout.handle.min.x >= r.min.x);
    assert!(layout.handle.max.x < layout.name.min.x);
    assert!(layout.name.max.x < layout.kebab.min.x);
    assert!(layout.kebab.max.x <= r.max.x);
    assert_eq!(layout.handle.width(), DEF_HANDLE_WIDTH);
    assert_eq!(layout.kebab.width(), DEF_KEBAB_WIDTH);
}

#[test]
fn test_format_footer_left() {
    assert_eq!(
        format_footer_left("EnumDefId(1)", "2 Entity Def(s)", false),
        "ID: EnumDefId(1) • Used by 2 Entity Def(s)"
    );
    assert_eq!(
        format_footer_left("TilesetDefId(2)", "4 Level(s)", true),
        "ID: TilesetDefId(2) • Used by 4 Level(s) • Unsaved changes"
    );
}

#[test]
fn test_headless_render_def_left_panel() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);
    let mut search = String::new();
    let mut drag = DragListState::new();

    let items = vec![
        DefItem::new(1u32, "Item One"),
        DefItem::new(2u32, "Item Two"),
        DefItem::new(3u32, "Item Three"),
    ];

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let config = DefLeftPanelConfig::new("+ New Item", "TEST DEFINITIONS").with_menu(true);
        let actions = def_left_panel(
            ui,
            &tokens,
            &config,
            &mut search,
            &items,
            Some(2),
            &mut drag,
        );
        assert!(actions.is_empty());
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_right_header() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);
    let mut active_tab = 0;

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let changed = def_right_header(
            ui,
            &tokens,
            &mut active_tab,
            &["DEF", "DOCUMENT"],
            "BiomeType",
            "Enum",
            "A test enum definition",
        );
        assert!(!changed);
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_section() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_section(ui, &tokens, "TEST SECTION", |ui| {
            ui.label("Inside section card");
        });
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_field_row() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_field_row(ui, &tokens, "Identifier", |ui| {
            let mut s = "value".to_string();
            ui.text_edit_singleline(&mut s);
        });
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_footer() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_footer(ui, &tokens, "ID: DefId(1) • Used by 0", |ui| {
            let _ = ui.button("Save");
        });
    });
    output.textures_delta.clear();
}

#[test]
fn test_def_left_panel_metrics_and_outlines_at_1366x768() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    for (item_count, new_label) in [
        (3, "+ New Item"),
        (
            12,
            "+ New Super Long Definition Label That Definitely Exceeds Normal Width",
        ),
    ] {
        let items: Vec<DefItem<u32>> = (0..item_count)
            .map(|i| DefItem::new(i, format!("Definition {i}")))
            .collect();
        let selected_id = Some(1u32); // Item 1 is selected
        let mut search = String::new();
        let mut drag = DragListState::new();

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let config = DefLeftPanelConfig::new(new_label, "DEFINITIONS").with_menu(true);
            let actions =
                def_left_panel(ui, &t, &config, &mut search, &items, selected_id, &mut drag);
            assert!(actions.is_empty());
        });

        // 0. Panel fill max.x == 260, + New button fill == Tokens.accent, ⋮ rect == 220..248 x 12..40
        let panel_fill = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                Shape::Rect(r) if (r.rect.min.x - 0.0).abs() < 0.5 && r.fill == t.panel => {
                    Some(r.rect)
                }
                _ => None,
            })
            .expect("panel fill rect must be present");
        assert_eq!(panel_fill.min.x, 0.0);
        assert_eq!(panel_fill.max.x, 260.0, "panel fill max.x == 260");

        let new_btn = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                Shape::Rect(r)
                    if (r.rect.min.x - 12.0).abs() < 0.5
                        && (r.rect.max.x - 214.0).abs() < 0.5
                        && (r.rect.min.y - 12.0).abs() < 0.5
                        && (r.rect.max.y - 40.0).abs() < 0.5
                        && r.fill == t.accent =>
                {
                    Some(r)
                }
                _ => None,
            })
            .expect("+ New button must be 12..214 x 12..40 with fill Tokens.accent");
        assert_eq!(new_btn.fill, t.accent);

        let menu_btn = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                Shape::Rect(r)
                    if (r.rect.min.x - 220.0).abs() < 0.5
                        && (r.rect.max.x - 248.0).abs() < 0.5
                        && (r.rect.min.y - 12.0).abs() < 0.5
                        && (r.rect.max.y - 40.0).abs() < 0.5 =>
                {
                    Some(r)
                }
                _ => None,
            })
            .expect("⋮ rect == 220..248 x 12..40");
        assert_eq!(menu_btn.fill, t.sunken);

        // No shape of the panel extends past x=260
        for s in &output.shapes {
            match &s.shape {
                Shape::Rect(r) => {
                    assert!(
                        r.rect.max.x <= 260.5,
                        "no shape of the panel extends past x=260: rect max.x={}",
                        r.rect.max.x
                    );
                }
                Shape::Text(t) => {
                    assert!(
                        t.pos.x <= 260.5,
                        "no shape of the panel extends past x=260: text pos.x={}",
                        t.pos.x
                    );
                }
                _ => {}
            }
        }

        // 1. Every card row spans 12.0..248.0 (width 236.0, height 36.0)
        let card_rect_strokes: Vec<(Rect, egui::Color32)> = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Rect(r) => {
                    let rect = r.rect;
                    if (rect.min.x - 12.0).abs() < 0.5
                        && (rect.width() - 236.0).abs() < 0.5
                        && (rect.height() - 36.0).abs() < 0.5
                        && r.stroke.width > 0.0
                    {
                        Some((rect, r.stroke.color))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        assert!(
            card_rect_strokes.len() >= 3,
            "must find rendered card rows with outlines"
        );

        // 2. Outlines: selected row has accent outline; unselected rows have hairline outline
        let (r0, c0) = card_rect_strokes[0];
        let (r1, c1) = card_rect_strokes[1];
        let (r2, c2) = card_rect_strokes[2];

        assert_eq!(
            c0,
            t.hairline().color,
            "unselected row must have hairline outline"
        );
        assert_eq!(c1, t.accent, "selected row must have accent outline");
        assert_eq!(
            c2,
            t.hairline().color,
            "unselected row must have hairline outline"
        );

        // 3. Card height 36 and gap 5 -> pitch 41.0
        assert_eq!(r0.height(), 36.0);
        assert!(
            (r1.min.y - r0.min.y - 41.0).abs() < 0.5,
            "row pitch must be 41.0 (36 height + 5 gap)"
        );
        assert!(
            (r2.min.y - r1.min.y - 41.0).abs() < 0.5,
            "row pitch must be 41.0 (36 height + 5 gap)"
        );

        // 4. Kebab button icon is placed inside x: 217..237 (glyph galley pos.x = 221.5, center = 227.0)
        let has_kebab_icon = output.shapes.iter().any(|s| match &s.shape {
            Shape::Text(t) => (t.pos.x - 221.5).abs() < 1.0,
            _ => false,
        });
        assert!(has_kebab_icon, "kebab icon must be inside x: 217..237");

        // 5. Drag handle is placed inside x: 23..39 (glyph galley pos.x = 24.0, center = 31.0)
        let has_drag_icon = output.shapes.iter().any(|s| match &s.shape {
            Shape::Text(t) => (t.pos.x - 24.0).abs() < 1.0,
            _ => false,
        });
        assert!(has_drag_icon, "drag handle icon must be inside x: 23..39");

        // 6. Name body starts at x: 51.0
        let has_name_body = output.shapes.iter().any(|s| match &s.shape {
            Shape::Text(t) => (t.pos.x - 51.0).abs() < 1.0,
            _ => false,
        });
        assert!(has_name_body, "definition name body must start at x = 51.0");

        // 5. Controls span 12.0..248.0
        let has_spanning_controls = output.shapes.iter().any(|s| match &s.shape {
            Shape::Rect(r) => {
                (r.rect.min.x - 12.0).abs() < 0.5 && (r.rect.max.x - 248.0).abs() < 0.5
            }
            _ => false,
        });
        assert!(
            has_spanning_controls,
            "controls must span within 12.0..248.0"
        );

        output.textures_delta.clear();
    }
}

#[test]
fn test_def_left_panel_interactions() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let items = vec![
        DefItem::new(10u32, "Alpha"),
        DefItem::new(20u32, "Beta"),
        DefItem::new(30u32, "Gamma"),
    ];

    // Frame 1: Measure row 0 center and kebab 0 center
    let raw_input1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut search = String::new();
    let mut drag = DragListState::new();
    let config = DefLeftPanelConfig::new("+ New", "ITEMS");

    let mut out1 = ctx.run_ui(raw_input1, |ui| {
        def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
    });

    let mut card_rects: Vec<Rect> = Vec::new();
    for s in &out1.shapes {
        if let Shape::Rect(r) = &s.shape {
            let rect = r.rect;
            if (rect.min.x - 12.0).abs() < 0.5
                && (rect.width() - 236.0).abs() < 0.5
                && (rect.height() - 36.0).abs() < 0.5
                && !card_rects
                    .iter()
                    .any(|existing: &Rect| (existing.min.y - rect.min.y).abs() < 0.5)
            {
                card_rects.push(rect);
            }
        }
    }
    assert_eq!(card_rects.len(), 3);
    let row0_center = card_rects[0].center();
    let kebab0_center = Pos2::new(227.0, card_rects[0].center().y);
    out1.textures_delta.clear();

    // Interaction 1: Clicking a row emits Selected(ID) once
    let raw_input2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![
            Event::PointerMoved(row0_center),
            Event::PointerButton {
                pos: row0_center,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            Event::PointerButton {
                pos: row0_center,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(raw_input2, |ui| {
        let actions = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
        assert_eq!(actions, vec![DefLeftPanelAction::Selected(10)]);
    });
    out2.textures_delta.clear();

    // Interaction 2: Right clicking a row emits ContextMenu
    let raw_input3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![
            Event::PointerMoved(row0_center),
            Event::PointerButton {
                pos: row0_center,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: Default::default(),
            },
            Event::PointerButton {
                pos: row0_center,
                button: PointerButton::Secondary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out3 = ctx.run_ui(raw_input3, |ui| {
        let actions = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
        assert!(actions
            .iter()
            .any(|a| matches!(a, DefLeftPanelAction::ContextMenu { id: 10, .. })));
    });
    out3.textures_delta.clear();

    // Interaction 3: Clicking kebab button emits ContextMenu
    let raw_input4 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![
            Event::PointerMoved(kebab0_center),
            Event::PointerButton {
                pos: kebab0_center,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            Event::PointerButton {
                pos: kebab0_center,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out4 = ctx.run_ui(raw_input4, |ui| {
        let actions = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
        assert!(actions
            .iter()
            .any(|a| matches!(a, DefLeftPanelAction::ContextMenu { id: 10, .. })));
    });
    out4.textures_delta.clear();

    // Interaction 4: Dragging handle from row 0 past row 1 emits Reorder { from: 0, to: 1 }
    let handle0_center = Pos2::new(31.0, card_rects[0].center().y);
    let drop_pos = Pos2::new(31.0, card_rects[1].max.y + 1.0);

    // Step A: press on handle and start drag
    let raw_input_drag1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![
            Event::PointerMoved(handle0_center),
            Event::PointerButton {
                pos: handle0_center,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
        ..Default::default()
    };
    let mut out_d1 = ctx.run_ui(raw_input_drag1, |ui| {
        let _ = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
    });
    drag.begin(&ctx, 0, card_rects[0]);
    out_d1.textures_delta.clear();

    // Step B: pointer moves down past row 1
    let raw_input_drag2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![Event::PointerMoved(drop_pos)],
        ..Default::default()
    };
    let mut out_d2 = ctx.run_ui(raw_input_drag2, |ui| {
        let _ = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
    });
    out_d2.textures_delta.clear();

    // Step C: release mouse to drop
    let raw_input_drag3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        events: vec![Event::PointerButton {
            pos: drop_pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out_d3 = ctx.run_ui(raw_input_drag3, |ui| {
        let actions = def_left_panel(ui, &t, &config, &mut search, &items, None, &mut drag);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, DefLeftPanelAction::Reorder { from: 0, to: 1 })),
            "dragging handle must emit Reorder action, got: {actions:?}"
        );
    });
    out_d3.textures_delta.clear();
}
