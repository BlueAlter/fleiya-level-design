//! Category content renderers for the Settings workspace.

use egui::{vec2, ComboBox, CornerRadius, DragValue, RichText, Slider, Stroke, Ui};
use lc_canvas::theme::{ThemeChoice, Tokens};
use lc_canvas::widgets::{button, hud, meta_row, ButtonKind};
use lc_core::{ProjectSettings, RenameProjectCommand, UpdateProjectSettingsCommand};
use lc_io::WorkspaceStartupMode;

use super::keybindings::render_keybindings_section;
use super::state::{SettingsCategory, SettingsViewState};
use super::theme::{theme_choice_from_pref, theme_pref_from_choice};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::text_field::{text_field, FieldStyle};
use crate::shared::ui::workspace_metrics::card_label;

/// Renders the right-hand section cards for a given [`SettingsCategory`].
pub fn render_category_sections(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
    category: SettingsCategory,
    query_lower: &str,
) {
    match category {
        SettingsCategory::General => render_general_section(app, ui, t, state),
        SettingsCategory::Appearance => render_appearance_section(app, ui, t),
        SettingsCategory::Canvas => render_canvas_section(app, ui, t),
        SettingsCategory::Keybindings => {
            render_keybindings_section(app, ui, t, state, query_lower);
        }
        SettingsCategory::System => render_system_section(app, ui, t, state),
        SettingsCategory::Project => render_project_section(app, ui, t, state),
    }
}

fn render_general_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
) {
    let mut changed = false;
    let mut collapsed = false;
    SectionContainer::new("STARTUP & WORKFLOW").show(ui, t, &mut collapsed, |_| {}, |ui| {
        ui.horizontal(|ui| {
            card_label(ui, "Startup Mode", t);
            let current = app.preferences.general.default_workspace;
            ComboBox::from_id_salt("settings_startup_workspace")
                .selected_text(current.label())
                .show_ui(ui, |ui| {
                    for mode in WorkspaceStartupMode::ALL {
                        if ui
                            .selectable_label(current == mode, mode.label())
                            .clicked()
                        {
                            app.preferences.general.default_workspace = mode;
                            changed = true;
                        }
                    }
                });
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            card_label(ui, "Language", t);
            let current_lang = app.preferences.general.language.clone();
            ComboBox::from_id_salt("settings_language")
                .selected_text(current_lang.as_str())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(current_lang == "en-US", "English (en-US)")
                        .clicked()
                    {
                        app.preferences.general.language = "en-US".to_string();
                        changed = true;
                    }
                });
        });
        ui.add_space(6.0);

        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.general.auto_open_last_project,
            "Auto-open last project on startup",
        )
        .changed()
        {
            changed = true;
        }
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.general.confirm_before_exit_unsaved,
            "Confirm before discarding unsaved changes",
        )
        .changed()
        {
            changed = true;
        }

        if let Some(last) = app.preferences.general.last_project_path.clone() {
            ui.add_space(8.0);
            meta_row(ui, "LAST PROJECT", &last);
            ui.add_space(4.0);
            if button(ui, "CLEAR RECENT PATH", ButtonKind::Ghost, true).clicked() {
                app.preferences.general.last_project_path = None;
                state.notice = Some("Cleared recent project path.".to_string());
                changed = true;
            }
        }
    });

    if changed {
        app.apply_preferences(ui.ctx());
        app.save_preferences();
    }
}

fn render_appearance_section(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    let mut changed = false;
    let mut collapsed = false;

    SectionContainer::new("COLOR PALETTE").show(ui, t, &mut collapsed, |_| {}, |ui| {
        ui.horizontal(|ui| {
            card_label(ui, "Editor Theme", t);
            let active_choice = theme_choice_from_pref(app.preferences.appearance.theme);
            ComboBox::from_id_salt("settings_theme_choice")
                .selected_text(active_choice.label())
                .show_ui(ui, |ui| {
                    for choice in ThemeChoice::ALL {
                        let palette = choice.tokens(true);
                        let is_selected = active_choice == choice;
                        ui.horizontal(|ui| {
                            let (swatch_rect, _) =
                                ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                swatch_rect,
                                CornerRadius::ZERO,
                                palette.bg,
                            );
                            ui.painter().rect_stroke(
                                swatch_rect,
                                CornerRadius::ZERO,
                                Stroke::new(1.0, palette.accent),
                                egui::StrokeKind::Inside,
                            );
                            if ui.selectable_label(is_selected, choice.label()).clicked() {
                                app.preferences.appearance.theme = theme_pref_from_choice(choice);
                                changed = true;
                            }
                        });
                    }
                });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            card_label(ui, "UI Scale", t);
            if ui
                .add(
                    Slider::new(&mut app.preferences.appearance.ui_scale, 0.75..=2.0)
                        .step_by(0.05)
                        .suffix("x"),
                )
                .changed()
            {
                changed = true;
            }
            if button(ui, "1.0X", ButtonKind::Ghost, true).clicked() {
                app.preferences.appearance.ui_scale = 1.0;
                changed = true;
            }
        });
    });

    ui.add_space(12.0);
    let mut chrome_collapsed = false;
    SectionContainer::new("WORKSPACE CHROME").show(ui, t, &mut chrome_collapsed, |_| {}, |ui| {
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.appearance.show_workspace_titles,
            "Show centered workspace title in top menu bar",
        )
        .changed()
        {
            changed = true;
        }
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.appearance.show_status_bar_coords,
            "Show cursor grid coordinates in status bar",
        )
        .changed()
        {
            changed = true;
        }
    });

    if changed {
        app.apply_preferences(ui.ctx());
        app.save_preferences();
    }
}

fn render_canvas_section(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    let mut changed = false;
    let mut collapsed = false;

    SectionContainer::new("VIEWPORT & INTERACTION").show(ui, t, &mut collapsed, |_| {}, |ui| {
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.canvas.show_grid_by_default,
            "Show level grid by default",
        )
        .changed()
        {
            changed = true;
        }
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.canvas.snap_to_grid_by_default,
            "Snap entities and shapes to grid by default",
        )
        .changed()
        {
            changed = true;
        }
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            card_label(ui, "Zoom Speed", t);
            if ui
                .add(
                    Slider::new(&mut app.preferences.canvas.zoom_sensitivity, 0.25..=3.0)
                        .step_by(0.05)
                        .suffix("x"),
                )
                .changed()
            {
                changed = true;
            }
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            card_label(ui, "Pan Speed", t);
            if ui
                .add(
                    Slider::new(&mut app.preferences.canvas.pan_speed, 0.25..=3.0)
                        .step_by(0.05)
                        .suffix("x"),
                )
                .changed()
            {
                changed = true;
            }
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            card_label(ui, "Checkerboard", t);
            if ui
                .add(
                    DragValue::new(&mut app.preferences.canvas.checkerboard_tile_px)
                        .range(4..=64)
                        .suffix(" px"),
                )
                .changed()
            {
                changed = true;
            }
        });
    });

    if changed {
        app.save_preferences();
    }
}

fn render_system_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
) {
    let mut changed = false;
    let mut collapsed = false;

    SectionContainer::new("AUTO-SAVE & HISTORY").show(ui, t, &mut collapsed, |_| {}, |ui| {
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.system.auto_save_enabled,
            "Enable automatic project saving when dirty",
        )
        .changed()
        {
            changed = true;
        }
        ui.add_space(6.0);

        ui.add_enabled_ui(app.preferences.system.auto_save_enabled, |ui| {
            ui.horizontal(|ui| {
                card_label(ui, "Interval", t);
                if ui
                    .add(
                        DragValue::new(&mut app.preferences.system.auto_save_interval_secs)
                            .range(30..=1800)
                            .suffix(" sec"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            card_label(ui, "Undo Limit", t);
            if ui
                .add(
                    DragValue::new(&mut app.preferences.system.max_undo_history)
                        .range(20..=1000)
                        .suffix(" steps"),
                )
                .changed()
            {
                changed = true;
            }
        });
    });

    ui.add_space(12.0);
    let mut console_collapsed = false;
    SectionContainer::new("CONSOLE & LOGGING").show(ui, t, &mut console_collapsed, |_| {}, |ui| {
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.system.console_default_debug,
            "Include DEBUG logs in console by default",
        )
        .changed()
        {
            changed = true;
        }
        if styled_checkbox(
            ui,
            t,
            &mut app.preferences.system.console_collapse_duplicates,
            "Collapse consecutive duplicate log lines",
        )
        .changed()
        {
            changed = true;
        }
    });

    ui.add_space(12.0);
    let mut storage_collapsed = false;
    SectionContainer::new("CONFIGURATION STORAGE").show(
        ui,
        t,
        &mut storage_collapsed,
        |_| {},
        |ui| {
        let path_text = app
            .preferences_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "In-memory session (headless / tests)".to_string());
        meta_row(ui, "CONFIG FILE", &path_text);
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if button(ui, "RESET DOCK LAYOUT", ButtonKind::Secondary, true).clicked() {
                app.reset_layout();
                state.notice = Some("Dock layout restored to defaults.".to_string());
            }
            ui.add_space(8.0);
            if button(ui, "RESET ALL PREFERENCES", ButtonKind::Danger, true).clicked() {
                app.preferences = lc_io::AppPreferences::default();
                app.apply_preferences(ui.ctx());
                app.save_preferences();
                state.notice = Some("All application preferences reset to defaults.".to_string());
            }
        });
    });

    if changed {
        app.save_preferences();
    }
}

fn render_project_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
) {
    state.sync_project_name(&app.project.name);

    let mut collapsed = false;
    SectionContainer::new("PROJECT CONFIGURATION").show(ui, t, &mut collapsed, |_| {}, |ui| {
        ui.horizontal(|ui| {
            card_label(ui, "Project Name", t);
            let res = text_field(
                ui,
                &mut state.project_name_draft,
                FieldStyle::default()
                    .with_outline(true)
                    .with_hint("Project title"),
            );
            if res.cancelled {
                state.project_name_draft = app.project.name.clone();
            } else if res.committed {
                let trimmed = state.project_name_draft.trim().to_string();
                if trimmed.is_empty() {
                    state.project_name_draft = app.project.name.clone();
                    state.notice = Some("Project name cannot be empty.".to_string());
                } else if trimmed != app.project.name {
                    app.execute_command(Box::new(RenameProjectCommand::new(trimmed.clone())));
                    state.synced_project_name = Some(trimmed);
                    state.notice = Some("Updated project name (undoable).".to_string());
                }
            }
        });
        ui.add_space(8.0);

        let mut grid = app.project.settings.default_grid_size;
        let mut grid_changed = false;
        ui.horizontal(|ui| {
            card_label(ui, "Default Grid", t);
            ui.label(RichText::new("W").color(t.muted));
            if ui
                .add(DragValue::new(&mut grid.x).range(1..=512).suffix(" px"))
                .changed()
            {
                grid_changed = true;
            }
            ui.add_space(8.0);
            ui.label(RichText::new("H").color(t.muted));
            if ui
                .add(DragValue::new(&mut grid.y).range(1..=512).suffix(" px"))
                .changed()
            {
                grid_changed = true;
            }
        });

        if grid_changed && grid != app.project.settings.default_grid_size {
            app.execute_command(Box::new(UpdateProjectSettingsCommand::new(
                ProjectSettings {
                    default_grid_size: grid,
                },
            )));
            state.notice = Some(format!(
                "Updated default grid size to {} x {} px (undoable).",
                grid.x, grid.y
            ));
        }

        ui.add_space(10.0);
        hud(ui, "PROJECT METADATA", t.faint);
        ui.add_space(6.0);
        let project_file = app
            .project_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unsaved project".to_string());
        meta_row(ui, "FILE PATH", &project_file);
        meta_row(ui, "SCHEMA VERSION", &app.project.version.to_string());
        meta_row(ui, "WORLDS", &app.project.worlds.len().to_string());
        meta_row(ui, "ENTITIES", &app.project.defs.entities.len().to_string());
        meta_row(ui, "TILESETS", &app.project.defs.tilesets.len().to_string());
        meta_row(ui, "ENUMS", &app.project.defs.enums.len().to_string());
        meta_row(ui, "LAYERS", &app.project.defs.layers.len().to_string());
    });
}
