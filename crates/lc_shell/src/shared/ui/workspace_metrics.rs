//! Universal workspace layout & spacing metrics (Design Tokens).
//!
//! Standardized metrics and layout helpers for all editor workspaces
//! (Enums, Tileset, Entities, Layers, Worlds).
//! See `docs/WORKSPACE_STANDARDS.md` for full specification and layout contracts.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::workspace_metrics::*;
//!
//! ui.horizontal(|ui| {
//!     card_label(ui, "Identifier", &tokens);
//!     ui.text_edit_singleline(&mut name);
//! });
//! ```

use egui::{vec2, Align, Layout, RichText, Sense, Ui};
use lc_canvas::theme::Tokens;

/// Padding on all four sides of both panels (left panel and detail panel).
pub const WORKSPACE_PADDING: f32 = 12.0;

/// Fixed width of the left list/hierarchy panel.
pub const LEFT_PANEL_WIDTH: f32 = 260.0;

/// Compact control height (toolbars, search inputs, tab strips, footer buttons).
pub const CONTROL_HEIGHT_COMPACT: f32 = 28.0;

/// Form control height (card fields, dropdowns/combos, tag chips, values rows, action triggers).
pub const CONTROL_HEIGHT_FORM: f32 = 32.0;

/// Width of the card label column in form definition cards.
pub const FORM_LABEL_COL_W: f32 = 110.0;

/// Exact square size of compact icon buttons (e.g. toolbar kebab 28x28).
pub const ICON_BUTTON_SIZE_COMPACT: f32 = CONTROL_HEIGHT_COMPACT;

/// Exact square size of form icon buttons (export 32x32, tags "+" and "⌄" 32x32, trash 32x32).
pub const ICON_BUTTON_SIZE_FORM: f32 = CONTROL_HEIGHT_FORM;

/// Width allocated for list item drag handles.
pub const DRAG_HANDLE_WIDTH: f32 = 16.0;

/// Default horizontal gap between slots in a row.
pub const SLOT_GAP: f32 = 6.0;

/// Default vertical gap between rows in a list.
pub const ROW_GAP: f32 = 4.0;

/// Size of row delete buttons (matches standard form control size).
pub const ICON_BUTTON_SIZE_DELETE: f32 = ICON_BUTTON_SIZE_FORM;

/// Renders a fixed-width form label column (110px) ensuring consistent control alignment.
pub fn card_label(ui: &mut Ui, text: &str, t: &Tokens) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(FORM_LABEL_COL_W, CONTROL_HEIGHT_FORM), Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.label(RichText::new(text).strong().color(t.ink_strong));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_constants_invariants() {
        const {
            assert!(ICON_BUTTON_SIZE_COMPACT == CONTROL_HEIGHT_COMPACT);
            assert!(ICON_BUTTON_SIZE_FORM == CONTROL_HEIGHT_FORM);
            assert!(ICON_BUTTON_SIZE_DELETE == CONTROL_HEIGHT_FORM);
            assert!(LEFT_PANEL_WIDTH > 200.0);
            assert!(WORKSPACE_PADDING > 0.0);
            assert!(FORM_LABEL_COL_W > 80.0);
            assert!(DRAG_HANDLE_WIDTH > 0.0);
            assert!(SLOT_GAP > 0.0);
            assert!(ROW_GAP > 0.0);
        }
    }

    #[test]
    fn test_card_label_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let t = Tokens::get(ui.ctx());
            card_label(ui, "Test Label", &t);
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "card_label should produce render shapes"
        );
    }
}
