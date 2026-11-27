//! Editor appearance settings: theme picker and preference conversion helpers.

use egui::{RichText, Ui};
use lc_canvas::theme::{self, ThemeChoice};
use lc_io::ThemePreference;

use crate::features::editor::shell_app::ShellApp;

/// Converts a serialized [`ThemePreference`] into a [`ThemeChoice`].
pub const fn theme_choice_from_pref(pref: ThemePreference) -> ThemeChoice {
    match pref {
        ThemePreference::System => ThemeChoice::System,
        ThemePreference::Dark => ThemeChoice::Dark,
        ThemePreference::Light => ThemeChoice::Light,
        ThemePreference::ProDark => ThemeChoice::ProDark,
        ThemePreference::ProMedium => ThemeChoice::ProMedium,
        ThemePreference::ProLight => ThemeChoice::ProLight,
        ThemePreference::SoftDark => ThemeChoice::SoftDark,
        ThemePreference::SoftLight => ThemeChoice::SoftLight,
        ThemePreference::SolarizedDark => ThemeChoice::SolarizedDark,
    }
}

/// Converts a [`ThemeChoice`] into a serializable [`ThemePreference`].
pub const fn theme_pref_from_choice(choice: ThemeChoice) -> ThemePreference {
    match choice {
        ThemeChoice::System => ThemePreference::System,
        ThemeChoice::Dark => ThemePreference::Dark,
        ThemeChoice::Light => ThemePreference::Light,
        ThemeChoice::ProDark => ThemePreference::ProDark,
        ThemeChoice::ProMedium => ThemePreference::ProMedium,
        ThemeChoice::ProLight => ThemePreference::ProLight,
        ThemeChoice::SoftDark => ThemePreference::SoftDark,
        ThemeChoice::SoftLight => ThemePreference::SoftLight,
        ThemeChoice::SolarizedDark => ThemePreference::SolarizedDark,
    }
}

/// The top-bar appearance picker menu.
pub fn theme_menu(ui: &mut Ui, app: &mut ShellApp) {
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
