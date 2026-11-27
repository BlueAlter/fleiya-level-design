//! Menu bar rendering and global shortcut handling.

use egui::{RichText, Ui};
use egui_dock::DockState;
use lc_canvas::theme::{self};
use lc_canvas::widgets::{hud, hud_elided, status_dot};
use lc_core::project::Project;
use lc_io::ShortcutAction;
use lc_ui_api::events::EditorEvent;

use crate::features::editor::app_mode::AppMode;
use crate::features::editor::shell_app::ShellApp;
use crate::features::settings::shortcuts::{actions_in_dispatch_order, to_egui_shortcut};

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
                    RichText::new("FLEIYA LEVEL DESIGN")
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

                    ui.separator();
                    let pref_label = format!(
                        "Preferences...  ({})",
                        app.preferences
                            .shortcuts
                            .get(ShortcutAction::OpenSettingsAlt)
                            .display_label()
                    );
                    if ui.button(pref_label).clicked() {
                        app.active_mode = AppMode::Settings;
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

                menu_button(ui, "WINDOWS", |ui| {
                    if ui.button("Layers Workspace  Ctrl+2").clicked() {
                        app.active_mode = crate::features::editor::app_mode::AppMode::Layers;
                        if app.selected_layer_def.is_none() {
                            app.selected_layer_def = app.project.defs.layers.first().map(|l| l.id);
                        }
                        if let Some(id) = app.selected_layer_def {
                            app.layer_view_state.select_layer(id);
                        }
                        ui.close();
                    }
                    if ui.button("Entities Workspace  Ctrl+3").clicked() {
                        app.active_mode = crate::features::editor::app_mode::AppMode::Entities;
                        if app.selected_entity_def.is_none() {
                            app.selected_entity_def =
                                app.project.defs.entities.first().map(|e| e.id);
                        }
                        if let Some(id) = app.selected_entity_def {
                            app.entity_view_state.select_entity(id);
                        }
                        ui.close();
                    }
                    if ui.button("Tilesets Workspace  Ctrl+4").clicked() {
                        app.active_mode = crate::features::editor::app_mode::AppMode::Tilesets;
                        if app.selected_tileset_def.is_none() {
                            app.selected_tileset_def =
                                app.project.defs.tilesets.first().map(|t| t.id);
                        }
                        if let Some(id) = app.selected_tileset_def {
                            app.tileset_view_state.select_tileset(id);
                        }
                        ui.close();
                    }
                    if ui.button("Enums Workspace  Ctrl+5").clicked() {
                        app.active_mode = AppMode::Enums;
                        ui.close();
                    }
                    if ui.button("Settings Workspace  Ctrl+6").clicked() {
                        app.active_mode = AppMode::Settings;
                        ui.close();
                    }
                    let in_editor = app.active_mode == AppMode::Editor;
                    ui.add_enabled_ui(in_editor, |ui| {
                        if ui
                            .checkbox(&mut app.show_entities_window, "Entities Window")
                            .clicked()
                        {
                            ui.close();
                        }
                        if ui
                            .checkbox(&mut app.show_layer_window, "Layers Window")
                            .clicked()
                        {
                            ui.close();
                        }
                    });
                });

                // Centered workspace title in the top bar
                if app.preferences.appearance.show_workspace_titles {
                    let title_text = app.active_mode.workspace_title();
                    let bar_rect = ui.max_rect();
                    let galley = ui.painter().layout_no_wrap(
                        title_text.to_owned(),
                        egui::FontId::new(11.0, theme::mono_bold_for(ui.ctx())),
                        t.ink_strong,
                    );
                    let title_pos = egui::pos2(
                        bar_rect.center().x - galley.size().x * 0.5,
                        bar_rect.center().y - galley.size().y * 0.5,
                    );
                    ui.painter().galley(title_pos, galley, t.ink_strong);
                }

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
                    crate::features::settings::theme::theme_menu(ui, app);
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

/// Handles application-wide keyboard shortcuts using user-configured bindings.
pub fn handle_global_shortcuts(app: &mut ShellApp, ctx: &egui::Context) {
    if app.active_mode == AppMode::Settings && app.settings_view_state.capturing_shortcut.is_some()
    {
        return;
    }

    let ctrl_2 = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num2);
    if ctx.input_mut(|i| i.consume_shortcut(&ctrl_2)) {
        app.active_mode = AppMode::Layers;
        if app.selected_layer_def.is_none() {
            app.selected_layer_def = app.project.defs.layers.first().map(|l| l.id);
        }
        if let Some(id) = app.selected_layer_def {
            app.layer_view_state.select_layer(id);
        }
    }

    let ordered = actions_in_dispatch_order(&app.preferences.shortcuts);
    for (action, binding) in ordered {
        let shortcut = to_egui_shortcut(binding);
        if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
            dispatch_shortcut_action(app, action);
        }
    }
}

fn dispatch_shortcut_action(app: &mut ShellApp, action: ShortcutAction) {
    match action {
        ShortcutAction::SaveProject => {
            app.events.push(EditorEvent::SaveProject);
        }
        ShortcutAction::Undo => {
            app.undo();
        }
        ShortcutAction::Redo | ShortcutAction::RedoAlt => {
            app.redo();
        }
        ShortcutAction::ToggleBottomDock => {
            app.dock_layout.toggle_bottom_dock();
        }
        ShortcutAction::WorkspaceEditor => {
            // Editor mode remains disabled in the workspace rail until level canvas cutover.
        }
        ShortcutAction::WorkspaceEntities => {
            app.active_mode = AppMode::Entities;
            if app.selected_entity_def.is_none() {
                app.selected_entity_def = app.project.defs.entities.first().map(|e| e.id);
            }
            if let Some(id) = app.selected_entity_def {
                app.entity_view_state.select_entity(id);
            }
        }
        ShortcutAction::WorkspaceTilesets => {
            app.active_mode = AppMode::Tilesets;
            if app.selected_tileset_def.is_none() {
                app.selected_tileset_def = app.project.defs.tilesets.first().map(|t| t.id);
            }
            if let Some(id) = app.selected_tileset_def {
                app.tileset_view_state.select_tileset(id);
            }
        }
        ShortcutAction::WorkspaceEnums => {
            app.active_mode = AppMode::Enums;
        }
        ShortcutAction::WorkspaceSettings | ShortcutAction::OpenSettingsAlt => {
            app.active_mode = AppMode::Settings;
        }
    }
}
