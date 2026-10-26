//! Menu bar rendering and global shortcut handling.

use egui::{Key, RichText, Ui};
use egui_dock::DockState;
use lc_canvas::theme::{self, ThemeChoice};
use lc_canvas::widgets::{hud, hud_elided, status_dot};
use lc_core::project::Project;
use lc_ui_api::events::EditorEvent;

use crate::shell_app::ShellApp;

/// Handles top-level menu bar rendering.
pub fn render_menu_bar(app: &mut ShellApp, ui: &mut Ui) {
    let t = theme::Tokens::get(ui.ctx());

    egui::Frame::new()
        .fill(t.bg)
        .inner_margin(egui::Margin::symmetric(10, 0))
        .show(ui, |ui| {
            ui.spacing_mut().interact_size.y = theme::TOP_BAR_H;
            egui::MenuBar::new().ui(ui, |ui| {
                // Wordmark: a small accent square plus Archivo display text.
                let (mark, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(mark, egui::CornerRadius::ZERO, t.accent);
                ui.add_space(6.0);
                ui.label(
                    RichText::new("LEVEL CRAFT")
                        .font(egui::FontId::new(16.0, theme::display_for(ui.ctx())))
                        .color(t.ink_strong),
                );
                ui.add_space(12.0);

                menu_button(ui, "FILE", |ui| {
                    if ui.button("New").clicked() {
                        app.project = Project::new("Untitled");
                        app.project_path = None;
                        app.undo_stack.clear();
                        app.open_documents.clear();
                        app.dock_state = DockState::new(Vec::new());
                        app.is_dirty = false;
                        app.status_message = "New project created".to_string();
                        ui.close();
                    }
                    if ui.button("Open...").clicked() {
                        app.request_open_project_dialog();
                        ui.close();
                    }
                    if ui.button("Save (Ctrl+S)").clicked() {
                        app.events.push(EditorEvent::SaveProject);
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        app.request_save_project_as_dialog();
                        ui.close();
                    }
                });

                menu_button(ui, "EDIT", |ui| {
                    let undo_label = app
                        .undo_stack
                        .undo_label()
                        .map(|l| format!("Undo {l} (Ctrl+Z)"))
                        .unwrap_or_else(|| "Undo (Ctrl+Z)".to_string());
                    if ui
                        .add_enabled(app.undo_stack.can_undo(), egui::Button::new(undo_label))
                        .clicked()
                    {
                        app.undo();
                        ui.close();
                    }

                    let redo_label = app
                        .undo_stack
                        .redo_label()
                        .map(|l| format!("Redo {l} (Ctrl+Y)"))
                        .unwrap_or_else(|| "Redo (Ctrl+Y)".to_string());
                    if ui
                        .add_enabled(app.undo_stack.can_redo(), egui::Button::new(redo_label))
                        .clicked()
                    {
                        app.redo();
                        ui.close();
                    }
                });

                menu_button(ui, "VIEW", |ui| {
                    let bottom_label = if app.dock_layout.bottom.collapsed {
                        "Expand Bottom Dock (Ctrl+`)"
                    } else {
                        "Collapse Bottom Dock (Ctrl+`)"
                    };
                    if ui.button(bottom_label).clicked() {
                        app.dock_layout.toggle_bottom_dock();
                        ui.close();
                    }

                    if ui.button("Reset Layout").clicked() {
                        app.reset_layout();
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    hud_elided(ui, &app.project.name, t.muted, 220.0);
                    ui.add_space(10.0);

                    if app.is_dirty() {
                        hud(ui, "UNSAVED", t.warn);
                        ui.add_space(4.0);
                        status_dot(ui, t.warn);
                    } else {
                        hud(ui, "SAVED", t.faint);
                    }

                    ui.add_space(10.0);
                    theme_menu(ui, app);
                });
            });
        });
}

/// A menu-bar button in the editor's mono HUD style.
fn menu_button(ui: &mut Ui, text: &str, add: impl FnOnce(&mut Ui)) {
    let label = RichText::new(text)
        .font(egui::FontId::new(
            theme::HUD_SIZE,
            theme::mono_bold_for(ui.ctx()),
        ))
        .color(theme::Tokens::get(ui.ctx()).muted);
    ui.menu_button(label, add);
}

/// The appearance picker.
fn theme_menu(ui: &mut Ui, app: &mut ShellApp) {
    let t = theme::Tokens::get(ui.ctx());
    let label = RichText::new("THEME")
        .font(egui::FontId::new(
            theme::HUD_SIZE,
            theme::mono_for(ui.ctx()),
        ))
        .color(t.muted);
    ui.menu_button(label, |ui| {
        for choice in ThemeChoice::ALL {
            if ui
                .selectable_label(app.theme_choice == choice, choice.label())
                .clicked()
            {
                app.set_theme(ui.ctx(), choice);
                ui.close();
            }
        }
    });
}

/// Handles application-wide keyboard shortcuts.
pub fn handle_global_shortcuts(app: &mut ShellApp, ctx: &egui::Context) {
    ctx.input(|i| {
        // Ctrl+` (Backtick): Toggle bottom dock
        if i.modifiers.ctrl && i.key_pressed(Key::Backtick) {
            app.dock_layout.toggle_bottom_dock();
        }

        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y
        if i.modifiers.ctrl && i.key_pressed(Key::Z) {
            if i.modifiers.shift {
                app.redo();
            } else {
                app.undo();
            }
        } else if i.modifiers.ctrl && i.key_pressed(Key::Y) {
            app.redo();
        }

        // Ctrl+S: Save
        if i.modifiers.ctrl && i.key_pressed(Key::S) {
            app.events.push(EditorEvent::SaveProject);
        }
    });
}
