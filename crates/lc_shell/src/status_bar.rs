//! Status bar rendering for the bottom window strip.

use egui::{Align, Layout, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{hud, hud_elided, rule, status_dot};

use crate::shell_app::ShellApp;

/// Renders the full-width status bar at the bottom of the editor.
pub fn render_status_bar(app: &mut ShellApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());

    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(theme::PANEL_PAD as f32);

        // 1. Grid coordinates
        if let Some(pos) = app.cursor_grid_pos {
            hud(ui, &format!("GRID {}:{}", pos.x, pos.y), t.muted);
        } else {
            hud(ui, "GRID -", t.muted);
        }
        rule(ui, 12.0);

        // 2. Zoom percentage
        hud(ui, &format!("{:.0}%", app.zoom_level * 100.0), t.muted);
        rule(ui, 12.0);

        // 3. Active tool
        hud(ui, &app.session.active_tool_name, t.accent_ink);
        rule(ui, 12.0);

        // 4. Dirty indicator
        if app.is_dirty() {
            status_dot(ui, t.warn);
            hud(ui, "UNSAVED", t.warn);
        } else {
            hud(ui, "SAVED", t.faint);
        }

        // 5. Status message, right aligned.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(theme::PANEL_PAD as f32);
            hud_elided(ui, &app.status_message, t.muted, 520.0);
        });
    });
}
