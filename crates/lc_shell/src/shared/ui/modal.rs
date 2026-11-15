//! Universal confirmation & dialog modal component with scrim and keyboard shortcuts.
//!
//! Provides a standardized modal dialog supporting custom body closures,
//! keyboard focus management (Esc cancels, Enter triggers primary/danger),
//! background scrim overlay, and structured buttons.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};
//!
//! let buttons = [
//!     ModalButton::new(1, "Discard", ModalButtonKind::Danger),
//!     ModalButton::new(2, "Cancel", ModalButtonKind::Normal),
//! ];
//! let result = modal(ctx, "discard_dialog", ModalSpec {
//!     title: "Discard Changes?".into(),
//!     body: |ui| { ui.label("Discard unsaved changes?"); },
//!     buttons: &buttons,
//!     width: 320.0,
//! });
//! ```

use egui::{
    vec2, Align2, Color32, Context, CornerRadius, Frame, Id, Key, Margin, Order, RichText, Sense,
    Stroke, Ui,
};
use lc_canvas::theme::Tokens;

use super::workspace_metrics::CONTROL_HEIGHT_COMPACT;

/// Visual classification of a modal button determining its emphasis and colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalButtonKind {
    /// Accent-filled primary action.
    Primary,
    /// Destructive/danger action (red fill).
    Danger,
    /// Subtle/secondary action (e.g. Cancel or Close).
    Normal,
}

/// Description of a single modal action button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalButton<ButtonId> {
    /// Unique identifier returned when this button is clicked or triggered.
    pub id: ButtonId,
    /// Display label for the button.
    pub label: String,
    /// Emphasis tier of the button.
    pub kind: ModalButtonKind,
}

impl<ButtonId> ModalButton<ButtonId> {
    /// Constructs a new [`ModalButton`].
    pub fn new(id: ButtonId, label: impl Into<String>, kind: ModalButtonKind) -> Self {
        Self {
            id,
            label: label.into(),
            kind,
        }
    }
}

/// Specification for presenting a modal dialog.
pub struct ModalSpec<'a, ButtonId, F>
where
    F: FnOnce(&mut Ui),
{
    /// Dialog heading.
    pub title: String,
    /// Closure rendering custom body content.
    pub body: F,
    /// List of interactive action buttons at the bottom.
    pub buttons: &'a [ModalButton<ButtonId>],
    /// Preferred width of the dialog frame in pixels.
    pub width: f32,
}

/// Action determined from keyboard events during modal presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalKeyAction {
    /// Enter key was pressed to trigger the primary or destructive action.
    TriggerPrimary,
    /// Escape key was pressed to cancel the dialog.
    Cancel,
    /// No modal navigation key was pressed.
    None,
}

/// Pure function to evaluate modal keyboard shortcuts.
pub fn handle_modal_keys(esc_pressed: bool, enter_pressed: bool) -> ModalKeyAction {
    if esc_pressed {
        ModalKeyAction::Cancel
    } else if enter_pressed {
        ModalKeyAction::TriggerPrimary
    } else {
        ModalKeyAction::None
    }
}

/// Resolves a [`ModalKeyAction`] to a specific button ID from the available buttons.
pub fn resolve_modal_action<ButtonId: Clone>(
    action: ModalKeyAction,
    buttons: &[ModalButton<ButtonId>],
) -> Option<ButtonId> {
    match action {
        ModalKeyAction::TriggerPrimary => buttons
            .iter()
            .find(|b| b.kind == ModalButtonKind::Primary || b.kind == ModalButtonKind::Danger)
            .or_else(|| buttons.first())
            .map(|b| b.id.clone()),
        ModalKeyAction::Cancel => buttons
            .iter()
            .find(|b| b.kind == ModalButtonKind::Normal)
            .or_else(|| buttons.last())
            .map(|b| b.id.clone()),
        ModalKeyAction::None => None,
    }
}

/// Renders a modal dialog over a scrim covering the whole window.
///
/// Returns `Some(ButtonId)` if a button was clicked or triggered via keyboard.
/// Outside clicks are absorbed and do NOT close the modal.
pub fn modal<ButtonId, F>(
    ctx: &Context,
    id: impl egui::AsId,
    spec: ModalSpec<'_, ButtonId, F>,
) -> Option<ButtonId>
where
    ButtonId: Clone + PartialEq + 'static,
    F: FnOnce(&mut Ui),
{
    let base_id = Id::new(id);
    let mut clicked_button = None;

    // 1. Scrim overlay: covers the entire screen, blocks clicks underneath.
    let screen_rect = ctx.viewport_rect();
    egui::Area::new(base_id.with("scrim_area"))
        .order(Order::Middle)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let (rect, _resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click_and_drag());
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(160));
        });

    // 2. Keyboard shortcut processing
    let esc = ctx.input(|i| i.key_pressed(Key::Escape));
    let enter = ctx.input(|i| i.key_pressed(Key::Enter));
    let key_action = handle_modal_keys(esc, enter);
    if let Some(btn_id) = resolve_modal_action(key_action, spec.buttons) {
        clicked_button = Some(btn_id);
    }

    // 3. Dialog window centered in foreground
    egui::Area::new(base_id.with("dialog_area"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            let t = Tokens::get(ctx);
            Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.line_strong))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(spec.width);

                    // Heading
                    ui.heading(RichText::new(&spec.title).strong().color(t.ink_strong));
                    ui.add_space(8.0);

                    // Body
                    (spec.body)(ui);
                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        for (idx, btn) in spec.buttons.iter().enumerate() {
                            let is_primary = btn.kind == ModalButtonKind::Primary
                                || btn.kind == ModalButtonKind::Danger;

                            let mut button_widget = egui::Button::new(match btn.kind {
                                ModalButtonKind::Danger => {
                                    RichText::new(&btn.label).color(Color32::WHITE)
                                }
                                ModalButtonKind::Primary => {
                                    RichText::new(&btn.label).color(t.invert_fg)
                                }
                                ModalButtonKind::Normal => RichText::new(&btn.label).color(t.ink),
                            })
                            .min_size(vec2(0.0, CONTROL_HEIGHT_COMPACT));

                            match btn.kind {
                                ModalButtonKind::Danger => {
                                    button_widget =
                                        button_widget.fill(Color32::from_rgb(220, 38, 38));
                                }
                                ModalButtonKind::Primary => {
                                    button_widget = button_widget.fill(t.accent);
                                }
                                ModalButtonKind::Normal => {
                                    button_widget =
                                        button_widget.fill(t.sunken).stroke(t.hairline());
                                }
                            }

                            let resp = ui.add(button_widget);
                            if idx == 0 && is_primary && ui.memory(|m| m.focused().is_none()) {
                                resp.request_focus();
                            }

                            if resp.clicked() {
                                clicked_button = Some(btn.id.clone());
                            }
                        }
                    });
                });
        });

    clicked_button
}

#[cfg(test)]
mod tests {
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
    fn test_resolve_modal_action() {
        let buttons = vec![
            ModalButton::new(1, "Discard", ModalButtonKind::Danger),
            ModalButton::new(2, "Cancel", ModalButtonKind::Normal),
        ];

        assert_eq!(
            resolve_modal_action(ModalKeyAction::TriggerPrimary, &buttons),
            Some(1)
        );
        assert_eq!(
            resolve_modal_action(ModalKeyAction::Cancel, &buttons),
            Some(2)
        );
        assert_eq!(resolve_modal_action(ModalKeyAction::None, &buttons), None);
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
}
