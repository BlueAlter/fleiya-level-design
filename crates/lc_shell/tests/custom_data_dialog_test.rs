use egui::Context;
use lc_canvas::theme::{self, Tokens};
use lc_core::id::TilesetDefId;
use lc_shell::shared::ui::custom_data_dialog::{
    render_custom_data_dialog, CustomDataDialogState, CustomDataOutcome,
};
use lc_shell::shared::ui::info_banner::info_banner;

#[test]
fn test_info_banner_headless_rendering() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    let t = Tokens::get(&ctx);

    let mut preview_ran = false;
    let mut output = ctx.run_ui(Default::default(), |ui| {
        info_banner(
            ui,
            &t,
            "Sample guidance message for testing custom tile data",
            Some(|ui: &mut egui::Ui| {
                ui.label("Preview");
                preview_ran = true;
            }),
        );
    });
    output.textures_delta.clear();

    assert!(preview_ran, "Preview closure should have executed");
}

#[test]
fn test_custom_data_dialog_state_lifecycle() {
    let mut state = CustomDataDialogState::default();
    assert!(!state.is_open);

    state.open(
        TilesetDefId(1),
        42,
        Some("{\"damage\": 100, \"type\": \"fire\"}"),
    );
    assert!(state.is_open);
    assert_eq!(state.tileset_id, Some(TilesetDefId(1)));
    assert_eq!(state.tile_id, 42);
    assert_eq!(state.text_buffer, "{\"damage\": 100, \"type\": \"fire\"}");
    assert_eq!(state.original_text, "{\"damage\": 100, \"type\": \"fire\"}");

    state.close();
    assert!(!state.is_open);
}

#[test]
fn test_custom_data_dialog_headless_render() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    let t = Tokens::get(&ctx);

    let mut state = CustomDataDialogState::default();
    state.open(TilesetDefId(1), 52, Some("Tile 52 Custom Properties"));

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let outcome = render_custom_data_dialog(
            ui.ctx(),
            &t,
            &mut state,
            Some(|ui: &mut egui::Ui| {
                ui.label("TilePreview");
            }),
        );
        assert_eq!(outcome, None, "Dialog remains open while user edits");
    });
    output.textures_delta.clear();

    assert!(state.is_open);
}

#[test]
fn test_custom_data_dialog_escape_cancels() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    let t = Tokens::get(&ctx);

    let mut state = CustomDataDialogState::default();
    state.open(TilesetDefId(1), 10, Some("Original data"));

    // Simulate Escape key press
    let raw_input = egui::RawInput {
        events: vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        ..Default::default()
    };

    let mut outcome_result = None;
    let mut output = ctx.run_ui(raw_input, |ui| {
        outcome_result =
            render_custom_data_dialog(ui.ctx(), &t, &mut state, None::<fn(&mut egui::Ui)>);
    });
    output.textures_delta.clear();

    assert_eq!(outcome_result, Some(CustomDataOutcome::Cancel));
    assert!(!state.is_open, "State should be closed on Escape");
}
