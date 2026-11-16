use super::*;
use egui::{vec2, Pos2, RawInput, Rect};
use lc_canvas::theme::{self, ThemeChoice};

#[test]
fn test_handle_modal_keys_pure_logic() {
    assert_eq!(handle_modal_keys(false, false), ModalKeyAction::None);
    assert_eq!(handle_modal_keys(true, false), ModalKeyAction::Cancel);
    assert_eq!(
        handle_modal_keys(false, true),
        ModalKeyAction::TriggerPrimary
    );
    // Esc takes precedence if both somehow fire simultaneously
    assert_eq!(handle_modal_keys(true, true), ModalKeyAction::Cancel);
}

#[test]
fn test_determine_initial_focus_button() {
    let danger_buttons = vec![
        ModalButton::new(1, "Delete", ModalButtonKind::Danger),
        ModalButton::new(2, "Cancel", ModalButtonKind::Normal),
    ];

    // Danger modal without explicit default: initial focus goes to Cancel/Normal
    assert_eq!(
        determine_initial_focus_button(None, &danger_buttons),
        Some(2)
    );

    // Danger modal with explicit default: initial focus goes to the specified button
    assert_eq!(
        determine_initial_focus_button(Some(&1), &danger_buttons),
        Some(1)
    );

    // Normal modal without explicit default: initial focus goes to Primary
    let normal_buttons = vec![
        ModalButton::new(10, "Confirm", ModalButtonKind::Primary),
        ModalButton::new(20, "Cancel", ModalButtonKind::Normal),
    ];
    assert_eq!(
        determine_initial_focus_button(None, &normal_buttons),
        Some(10)
    );
}

#[test]
fn test_resolve_modal_action_scenarios() {
    // 1. Danger-only + Cancel
    let danger_cancel = vec![
        ModalButton::new(1, "Discard", ModalButtonKind::Danger),
        ModalButton::new(2, "Cancel", ModalButtonKind::Normal),
    ];
    // Enter NEVER triggers Danger
    assert_eq!(
        resolve_modal_action(ModalKeyAction::TriggerPrimary, &danger_cancel),
        None
    );
    // Esc triggers Cancel
    assert_eq!(
        resolve_modal_action(ModalKeyAction::Cancel, &danger_cancel),
        Some(2)
    );

    // 2. Primary + Cancel
    let primary_cancel = vec![
        ModalButton::new(10, "Save", ModalButtonKind::Primary),
        ModalButton::new(20, "Cancel", ModalButtonKind::Normal),
    ];
    // Enter triggers Primary
    assert_eq!(
        resolve_modal_action(ModalKeyAction::TriggerPrimary, &primary_cancel),
        Some(10)
    );
    // Esc triggers Cancel
    assert_eq!(
        resolve_modal_action(ModalKeyAction::Cancel, &primary_cancel),
        Some(20)
    );

    // 3. Danger first in the slice
    let danger_first = vec![
        ModalButton::new(100, "Delete All", ModalButtonKind::Danger),
        ModalButton::new(200, "Cancel", ModalButtonKind::Normal),
    ];
    // Danger is at index 0, Enter still returns None (never triggers Danger)
    assert_eq!(
        resolve_modal_action(ModalKeyAction::TriggerPrimary, &danger_first),
        None
    );

    // 4. No buttons
    let empty_buttons: Vec<ModalButton<u32>> = Vec::new();
    assert_eq!(
        resolve_modal_action(ModalKeyAction::TriggerPrimary, &empty_buttons),
        None
    );
    assert_eq!(
        resolve_modal_action(ModalKeyAction::Cancel, &empty_buttons),
        None
    );
    assert_eq!(
        resolve_modal_action(ModalKeyAction::None, &empty_buttons),
        None
    );
}

#[test]
fn test_modal_headless_render() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let buttons = [
        ModalButton::new(10, "Confirm", ModalButtonKind::Primary),
        ModalButton::new(20, "Cancel", ModalButtonKind::Normal),
    ];

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let res = modal(
            ui.ctx(),
            "test_dialog",
            ModalSpec {
                title: "Test Modal".into(),
                body: |ui| {
                    ui.label("This is a modal dialog test.");
                },
                buttons: &buttons,
                width: 350.0,
                default_button: None,
            },
        );
        assert_eq!(res, None);
    });

    output.textures_delta.clear();
    assert!(
        !output.shapes.is_empty(),
        "Modal should produce render shapes (scrim, window, buttons)"
    );
}

#[test]
fn test_danger_modal_does_not_trigger_on_enter_headless() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let buttons = [
        ModalButton::new(1, "Delete Item", ModalButtonKind::Danger),
        ModalButton::new(2, "Cancel", ModalButtonKind::Normal),
    ];

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        events: vec![egui::Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let res = modal(
            ui.ctx(),
            "danger_dialog",
            ModalSpec {
                title: "Delete Forever?".into(),
                body: |ui| {
                    ui.label("Are you sure?");
                },
                buttons: &buttons,
                width: 350.0,
                default_button: None,
            },
        );
        // Enter must NOT trigger the Danger button
        assert_eq!(res, None);
    });

    output.textures_delta.clear();
}

#[test]
fn test_confirm_modal_tab_focus_and_enter_navigation() {
    let buttons = [
        ModalButton::new(1, "Cancel", ModalButtonKind::Normal),
        ModalButton::new(2, "Delete", ModalButtonKind::Danger),
    ];

    // Part 1: Enter with initial focus (Cancel) returns Cancel (1)
    let ctx1 = Context::default();
    theme::install_fonts(&ctx1);
    theme::apply(&ctx1, ThemeChoice::Dark);

    let raw1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        ..Default::default()
    };
    let mut out1 = ctx1.run_ui(raw1, |ui| {
        let res = modal(
            ui.ctx(),
            "focus_modal_1",
            ModalSpec {
                title: "Confirm".into(),
                body: |_| {},
                buttons: &buttons,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(res, None);
    });
    out1.textures_delta.clear();

    let raw2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        events: vec![egui::Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out2 = ctx1.run_ui(raw2, |ui| {
        let res = modal(
            ui.ctx(),
            "focus_modal_1",
            ModalSpec {
                title: "Confirm".into(),
                body: |_| {},
                buttons: &buttons,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(res, Some(1), "Enter on initial focus must return Cancel");
    });
    out2.textures_delta.clear();

    // Part 2: Tab to Danger button, then press Enter -> returns Danger (2)
    let ctx2 = Context::default();
    theme::install_fonts(&ctx2);
    theme::apply(&ctx2, ThemeChoice::Dark);

    let raw_f1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        ..Default::default()
    };
    let mut out_f1 = ctx2.run_ui(raw_f1, |ui| {
        let res = modal(
            ui.ctx(),
            "focus_modal_2",
            ModalSpec {
                title: "Confirm".into(),
                body: |_| {},
                buttons: &buttons,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(res, None);
    });
    out_f1.textures_delta.clear();

    // Press Tab to shift focus to Danger
    let raw_f2 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        events: vec![egui::Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out_f2 = ctx2.run_ui(raw_f2, |ui| {
        let res = modal(
            ui.ctx(),
            "focus_modal_2",
            ModalSpec {
                title: "Confirm".into(),
                body: |_| {},
                buttons: &buttons,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(res, None);
    });
    out_f2.textures_delta.clear();

    // Press Enter while Danger has focus -> returns Danger (2)
    let raw_f3 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        events: vec![egui::Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out_f3 = ctx2.run_ui(raw_f3, |ui| {
        let res = modal(
            ui.ctx(),
            "focus_modal_2",
            ModalSpec {
                title: "Confirm".into(),
                body: |_| {},
                buttons: &buttons,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(
            res,
            Some(2),
            "Enter when Danger button is focused must return Danger id"
        );
    });
    out_f3.textures_delta.clear();

    // Part 3: Danger-only modal with nothing focused -> Enter returns None
    let ctx3 = Context::default();
    theme::install_fonts(&ctx3);
    theme::apply(&ctx3, ThemeChoice::Dark);

    let danger_only = [ModalButton::new(99, "Destroy", ModalButtonKind::Danger)];
    let raw_d1 = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
        events: vec![egui::Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };
    let mut out_d1 = ctx3.run_ui(raw_d1, |ui| {
        let res = modal(
            ui.ctx(),
            "danger_only_modal",
            ModalSpec {
                title: "Destroy".into(),
                body: |_| {},
                buttons: &danger_only,
                width: 300.0,
                default_button: None,
            },
        );
        assert_eq!(
            res, None,
            "Enter with nothing focused must return None for Danger-only modal"
        );
    });
    out_d1.textures_delta.clear();
}
