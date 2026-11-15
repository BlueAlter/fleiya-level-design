//! Universal workspace layout & spacing metrics (Design Tokens).
//!
//! Standardized metrics and layout helpers for all editor workspaces
//! (Enums, Tileset, Entities, Layers, Worlds).
//! See `docs/WORKSPACE_STANDARDS.md` for full specification and layout contracts.

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

/// Renders a fixed-width form label column (110px) ensuring consistent control alignment.
pub fn card_label(ui: &mut Ui, text: &str, t: &Tokens) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(FORM_LABEL_COL_W, CONTROL_HEIGHT_FORM), Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.label(RichText::new(text).strong().color(t.ink_strong));
    });
}
