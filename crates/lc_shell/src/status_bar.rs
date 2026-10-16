//! Status bar rendering for the bottom window strip.

use crate::shell_app::ShellApp;
use egui::Ui;

/// Renders the full-width status bar at the bottom of the editor.
pub fn render_status_bar(app: &mut ShellApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        // 1. Grid coordinates
        if let Some(pos) = app.cursor_grid_pos {
            ui.label(format!("Grid: [{}, {}]", pos.x, pos.y));
        } else {
            ui.label("Grid: -");
        }
        ui.separator();

        // 2. Zoom percentage
        ui.label(format!("{:.0}%", app.zoom_level * 100.0));
        ui.separator();

        // 3. Active tool
        ui.label(&app.session.active_tool_name);
        ui.separator();

        // 4. Dirty indicator
        if app.is_dirty() {
            ui.colored_label(egui::Color32::from_rgb(255, 180, 50), "* Unsaved changes");
        } else {
            ui.colored_label(egui::Color32::GRAY, "Saved");
        }
        ui.separator();

        // 5. Status message
        ui.label(&app.status_message);
    });
}
