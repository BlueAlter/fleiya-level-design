//! Editor appearance settings: the theme picker.

use egui::{RichText, Ui};
use lc_canvas::theme::{self, ThemeChoice};

use crate::features::editor::shell_app::ShellApp;

/// The appearance picker.
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
