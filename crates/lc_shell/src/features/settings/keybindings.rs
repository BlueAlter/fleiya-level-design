//! Interactive keyboard shortcut editor with collision detection.

use egui::{vec2, Align, CornerRadius, Frame, Layout, Margin, RichText, Stroke, Ui};
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::{button, hud, ButtonKind};
use lc_io::ShortcutAction;

use super::shortcuts::{capture_shortcut_from_ctx, ShortcutCapture};
use super::state::SettingsViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::section_container::SectionContainer;

/// Renders the Keybindings category card with live rebinding and conflict badges.
pub fn render_keybindings_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
    query_lower: &str,
) {
    if let Some(target_action) = state.capturing_shortcut {
        if let Some(capture) = capture_shortcut_from_ctx(ui.ctx()) {
            match capture {
                ShortcutCapture::Cancelled => {
                    state.capturing_shortcut = None;
                    state.notice = Some("Shortcut capture cancelled.".to_string());
                }
                ShortcutCapture::Captured(binding) => {
                    let conflicts = app
                        .preferences
                        .shortcuts
                        .conflicts_for(target_action, binding);
                    app.preferences.shortcuts.set(target_action, binding);
                    app.save_preferences();
                    state.capturing_shortcut = None;
                    if conflicts.is_empty() {
                        state.notice = Some(format!(
                            "Bound '{}' to {}",
                            target_action.label(),
                            binding.display_label()
                        ));
                    } else {
                        let names: Vec<&str> = conflicts.iter().map(|a| a.label()).collect();
                        state.notice = Some(format!(
                            "Bound '{}' to {} (conflicts with: {})",
                            target_action.label(),
                            binding.display_label(),
                            names.join(", ")
                        ));
                    }
                }
            }
        }
    }

    let conflicts = app.preferences.shortcuts.all_conflicts();
    if !conflicts.is_empty() {
        Frame::new()
            .fill(lc_canvas::theme::tint(t.warn, 0.14))
            .stroke(Stroke::new(1.0, t.warn))
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!(
                        "Detected {} shortcut collision(s). Rebind or reset conflicting rows below.",
                        conflicts.len()
                    ))
                    .color(t.warn)
                    .strong(),
                );
            });
        ui.add_space(10.0);
    }

    let mut collapsed = false;
    SectionContainer::new("GLOBAL SHORTCUTS").show(
        ui,
        t,
        &mut collapsed,
        |_| {},
        |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(
                        "Click a shortcut badge to record a new key combination (Esc cancels).",
                    )
                    .size(12.0)
                    .color(t.muted),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, "RESET ALL", ButtonKind::Secondary, true).clicked() {
                        app.preferences.shortcuts.reset_all();
                        app.save_preferences();
                        state.capturing_shortcut = None;
                        state.notice = Some("All shortcuts reset to defaults.".to_string());
                    }
                });
            });
            ui.add_space(10.0);

            for action in ShortcutAction::ALL {
                let binding = app.preferences.shortcuts.get(action);
                let label_lower = action.label().to_lowercase();
                let desc_lower = action.description().to_lowercase();
                let key_lower = binding.display_label().to_lowercase();

                if !query_lower.is_empty()
                    && !label_lower.contains(query_lower)
                    && !desc_lower.contains(query_lower)
                    && !key_lower.contains(query_lower)
                    && !"keybindings shortcuts".contains(query_lower)
                {
                    continue;
                }

                let row_conflicts = app.preferences.shortcuts.conflicts_for(action, binding);
                let is_capturing = state.capturing_shortcut == Some(action);
                let is_default = binding == lc_io::ShortcutMap::default_for(action);

                Frame::new()
                    .fill(t.sunken)
                    .stroke(if !row_conflicts.is_empty() {
                        Stroke::new(1.0, t.warn)
                    } else {
                        t.hairline()
                    })
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(action.label()).strong().color(t.ink_strong),
                                    );
                                    if !row_conflicts.is_empty() {
                                        ui.add_space(6.0);
                                        hud(ui, "CONFLICT", t.warn);
                                    }
                                });
                                ui.label(
                                    RichText::new(action.description())
                                        .size(11.5)
                                        .color(t.muted),
                                );
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if button(ui, "RESET", ButtonKind::Ghost, !is_default).clicked() {
                                    app.preferences.shortcuts.reset_action(action);
                                    app.save_preferences();
                                    if is_capturing {
                                        state.capturing_shortcut = None;
                                    }
                                    state.notice =
                                        Some(format!("Reset '{}' to default.", action.label()));
                                }

                                ui.add_space(6.0);
                                let badge_text = if is_capturing {
                                    "PRESS KEYS...".to_string()
                                } else {
                                    binding.display_label()
                                };
                                let badge_kind = if is_capturing {
                                    ButtonKind::Primary
                                } else if !row_conflicts.is_empty() {
                                    ButtonKind::Danger
                                } else {
                                    ButtonKind::Secondary
                                };
                                let resp = ui.allocate_ui(vec2(130.0, 28.0), |ui| {
                                    button(ui, &badge_text, badge_kind, true)
                                });
                                if resp.inner.clicked() {
                                    state.capturing_shortcut =
                                        if is_capturing { None } else { Some(action) };
                                }
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        },
    );
}
