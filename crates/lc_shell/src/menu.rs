//! Menu bar rendering and global shortcut handling.

use egui::{Key, Ui};
use egui_dock::DockState;
use lc_core::project::Project;
use lc_ui_api::events::EditorEvent;

use crate::shell_app::ShellApp;

/// Handles top-level menu bar rendering.
pub fn render_menu_bar(app: &mut ShellApp, ui: &mut Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
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
                // If a project path was already used or default project exists, open it,
                // or report status. Full native file picker integration is tracked in T-0109.
                if let Some(ref path) = app.project_path.clone() {
                    let _ = app.open_project_from_path(path);
                } else {
                    app.status_message =
                        "Use File > Open Project in project panel or specify project file path"
                            .to_string();
                }
                ui.close();
            }
            if ui.button("Save (Ctrl+S)").clicked() {
                app.events.push(EditorEvent::SaveProject);
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
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

        ui.menu_button("View", |ui| {
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
