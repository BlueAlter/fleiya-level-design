//! Full-window Settings Workspace (`AppMode::Settings`, Ctrl+6).

use egui::{
    vec2, Align, CornerRadius, CursorIcon, Frame, Layout, Margin, Pos2, Rect, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, TextEdit, Ui,
};
use lc_canvas::icons::paint_icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hud, ButtonKind};

use super::i18n::{self, keys};
use super::sections::render_category_sections;
use super::state::{SettingsCategory, SettingsViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::def_workspace::{def_footer, def_panel_width, DEF_FOOTER_HEIGHT};
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

/// Renders the full Settings Workspace page (`AppMode::Settings`).
pub fn render_settings_view(app: &mut ShellApp, ui: &mut Ui) {
    i18n::register();
    let t = Tokens::get(ui.ctx());

    let mut state = std::mem::take(&mut app.settings_view_state);

    let full_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(full_rect, CornerRadius::ZERO, t.bg);

    let left_w = def_panel_width();
    let left_rect = Rect::from_min_max(
        full_rect.min,
        Pos2::new(full_rect.min.x + left_w, full_rect.max.y),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        render_left_categories(ui, &t, &mut state);
    });

    let right_rect = Rect::from_min_max(Pos2::new(left_rect.max.x, full_rect.min.y), full_rect.max);
    let body_bottom = (right_rect.max.y - DEF_FOOTER_HEIGHT).max(right_rect.min.y);
    let content_rect = Rect::from_min_max(
        Pos2::new(
            right_rect.min.x + WORKSPACE_PADDING,
            right_rect.min.y + WORKSPACE_PADDING,
        ),
        Pos2::new(
            (right_rect.max.x - WORKSPACE_PADDING).max(right_rect.min.x + WORKSPACE_PADDING),
            (body_bottom - WORKSPACE_PADDING).max(right_rect.min.y + WORKSPACE_PADDING),
        ),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        render_right_content(app, ui, &t, &mut state);
    });

    let footer_rect = Rect::from_min_max(Pos2::new(right_rect.min.x, body_bottom), right_rect.max);
    ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
        render_footer(app, ui, &t, &mut state);
    });

    app.settings_view_state = state;
}

fn render_left_categories(ui: &mut Ui, t: &Tokens, state: &mut SettingsViewState) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.panel);
    ui.painter().vline(rect.max.x, rect.y_range(), t.hairline());

    let inner = rect.shrink(WORKSPACE_PADDING);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.set_width(inner.width());
        hud(ui, tr(keys::HEADER), t.muted);
        ui.add_space(8.0);

        let search = TextEdit::singleline(&mut state.search_query)
            .hint_text(tr(keys::SEARCH_HINT))
            .desired_width(f32::INFINITY)
            .min_size(vec2(ui.available_width(), CONTROL_HEIGHT_COMPACT));
        ui.add_sized([ui.available_width(), CONTROL_HEIGHT_COMPACT], search);
        ui.add_space(12.0);

        let query_lower = state.search_query.trim().to_lowercase();

        ScrollArea::vertical()
            .id_salt("settings_category_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                hud(ui, tr(keys::GROUP_APPLICATION), t.faint);
                ui.add_space(6.0);
                for cat in SettingsCategory::APPLICATION {
                    if cat.matches_query(&query_lower) {
                        category_row(ui, t, state, cat);
                    }
                }

                ui.add_space(12.0);
                hud(ui, tr(keys::GROUP_PROJECT), t.faint);
                ui.add_space(6.0);
                for cat in SettingsCategory::PROJECT {
                    if cat.matches_query(&query_lower) {
                        category_row(ui, t, state, cat);
                    }
                }
            });
    });
}

fn category_row(ui: &mut Ui, t: &Tokens, state: &mut SettingsViewState, cat: SettingsCategory) {
    let selected = state.selected_category == cat;
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

    let (fill, stroke, text_col) = if selected {
        (t.sunken, Stroke::new(1.0, t.accent), t.ink_strong)
    } else if resp.hovered() {
        (t.sunken, t.hairline(), t.ink_strong)
    } else {
        (t.bg, t.hairline(), t.ink)
    };

    ui.painter()
        .rect(rect, CornerRadius::ZERO, fill, stroke, StrokeKind::Inside);
    paint_icon(
        ui.painter(),
        Pos2::new(rect.min.x + 16.0, rect.center().y),
        cat.icon(),
        14.0,
        if selected { t.accent_ink } else { t.muted },
    );
    ui.painter().text(
        Pos2::new(rect.min.x + 32.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        cat.label(),
        egui::FontId::new(13.0, theme::medium_for(ui.ctx())),
        text_col,
    );

    if resp.clicked() {
        state.selected_category = cat;
        state.capturing_shortcut = None;
    }
    ui.add_space(5.0);
}

fn render_right_content(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut SettingsViewState,
) {
    let query_lower = state.search_query.trim().to_lowercase();
    let active_cat = state.selected_category;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        if let Some(notice) = state.notice.clone() {
            let mut dismiss = false;
            Frame::new()
                .fill(theme::tint(t.accent, 0.12))
                .stroke(Stroke::new(1.0, t.accent))
                .corner_radius(CornerRadius::ZERO)
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&notice).color(t.ink_strong));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if button(ui, "DISMISS", ButtonKind::Ghost, true).clicked() {
                                dismiss = true;
                            }
                        });
                    });
                });
            if dismiss {
                state.notice = None;
            }
            ui.add_space(10.0);
        }

        if query_lower.is_empty() {
            ui.label(
                RichText::new(active_cat.label())
                    .font(egui::FontId::new(22.0, theme::display_for(ui.ctx())))
                    .color(t.ink_strong),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(active_cat.description())
                    .size(12.0)
                    .color(t.muted),
            );
        } else {
            ui.label(
                RichText::new(format!(
                    "Search results for \"{}\"",
                    state.search_query.trim()
                ))
                .font(egui::FontId::new(20.0, theme::display_for(ui.ctx())))
                .color(t.ink_strong),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Matching categories across application and project settings.")
                    .size(12.0)
                    .color(t.muted),
            );
        }
        ui.add_space(14.0);

        ScrollArea::vertical()
            .id_salt(("settings_detail_scroll", active_cat, &query_lower))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if query_lower.is_empty() {
                    render_category_sections(app, ui, t, state, active_cat, "");
                } else {
                    let matching: Vec<SettingsCategory> = SettingsCategory::ALL
                        .iter()
                        .copied()
                        .filter(|c| c.matches_query(&query_lower))
                        .collect();
                    if matching.is_empty() {
                        Frame::new()
                            .fill(t.panel)
                            .stroke(t.hairline())
                            .corner_radius(CornerRadius::ZERO)
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new("No settings matched your query.").color(t.muted),
                                );
                            });
                    } else {
                        for cat in matching {
                            render_category_sections(app, ui, t, state, cat, &query_lower);
                            ui.add_space(14.0);
                        }
                    }
                }
                ui.add_space(16.0);
            });
    });
}

fn render_footer(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, state: &mut SettingsViewState) {
    let left_status = if app.preferences_path.is_some() {
        tr(keys::FOOTER_SAVED)
    } else {
        tr(keys::FOOTER_MEMORY_ONLY)
    };

    def_footer(ui, t, left_status, |ui| {
        if button(ui, tr(keys::RESET_SECTION), ButtonKind::Secondary, true).clicked() {
            reset_active_category(app, ui.ctx(), state);
        }
    });
}

fn reset_active_category(app: &mut ShellApp, ctx: &egui::Context, state: &mut SettingsViewState) {
    match state.selected_category {
        SettingsCategory::General => {
            app.preferences.general = lc_io::GeneralPreferences::default();
            app.apply_preferences(ctx);
            app.save_preferences();
        }
        SettingsCategory::Appearance => {
            app.preferences.appearance = lc_io::AppearancePreferences::default();
            app.apply_preferences(ctx);
            app.save_preferences();
        }
        SettingsCategory::Canvas => {
            app.preferences.canvas = lc_io::CanvasPreferences::default();
            app.save_preferences();
        }
        SettingsCategory::Keybindings => {
            app.preferences.shortcuts.reset_all();
            app.save_preferences();
            state.capturing_shortcut = None;
        }
        SettingsCategory::System => {
            app.preferences.system = lc_io::SystemPreferences::default();
            app.save_preferences();
        }
        SettingsCategory::Project => {
            let defaults = lc_core::ProjectSettings::default();
            if app.project.settings != defaults {
                app.execute_command(Box::new(lc_core::UpdateProjectSettingsCommand::new(
                    defaults,
                )));
            }
        }
    }
    state.notice = Some(format!(
        "Reset '{}' to defaults.",
        state.selected_category.label()
    ));
}
