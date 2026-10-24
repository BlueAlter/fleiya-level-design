//! Central workspace empty state shown when no document tabs are open.

use egui::{Align, CornerRadius, Layout, Margin, Rect, Ui, Vec2};
use lc_canvas::grid::draw_grid;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, hud_elided, ButtonKind};
use lc_core::command::DocKind;
use lc_ui_api::viewport::Viewport;

use crate::shell_app::ShellApp;

/// Draws the workspace surface that replaces an empty central dock area.
///
/// The faint grid is a visual signature, not a canvas: it has no input handling
/// and no document state. The card offers the one safe action available without
/// a project tree selection: opening project settings.
pub fn render_workspace_empty(app: &mut ShellApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);

    let mut viewport = Viewport::new(rect);
    viewport.set_zoom(1.0);
    // The workspace grid is decorative: suppress the red/green world axes so it
    // reads as a quiet background instead of an active canvas.
    let grid_tokens = Tokens {
        axis_x: t.grid_major,
        axis_y: t.grid_major,
        ..t
    };
    draw_grid(ui.painter(), rect, &viewport, 32, &grid_tokens);

    let card_size = Vec2::new(
        (rect.width() - 48.0).clamp(320.0, 520.0),
        (rect.height() - 48.0).clamp(220.0, 240.0),
    );
    let card_rect = Rect::from_center_size(rect.center(), card_size);

    ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect), |ui| {
        egui::Frame::new()
            .fill(t.canvas)
            .stroke(t.hairline())
            .inner_margin(Margin::same(24))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(card_size.y - 48.0);

                hud(ui, "WORKSPACE / NO DOCUMENT", t.accent_ink);
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Open a document")
                        .font(egui::FontId::new(26.0, theme::display_for(ui.ctx())))
                        .color(t.ink_strong),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Choose a world, level, definition or project settings from the project panel.",
                    )
                    .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(t.muted),
                );
                ui.add_space(16.0);
                if button(ui, "OPEN PROJECT SETTINGS", ButtonKind::Primary, true).clicked() {
                    app.open_document(DocKind::ProjectSettings);
                }
                ui.add_space(12.0);
                hline(ui);
                ui.add_space(8.0);
                hud(ui, "CTRL+S SAVE  ·  CTRL+` CONSOLE  ·  CTRL+Z UNDO", t.faint);
                ui.add_space(10.0);
                hline(ui);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    hud(ui, "PROJECT", t.faint);
                    ui.add_space(6.0);
                    hud_elided(ui, &app.project.name, t.muted, 240.0);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        hud(ui, &format!("VERSION {}", app.project.version), t.faint);
                    });
                });
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::project::Project;

    #[test]
    fn workspace_empty_state_renders_without_panicking() {
        let mut app = ShellApp::new(Project::new("Test"));
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_workspace_empty(&mut app, ui);
        });
        output.textures_delta.clear();
    }
}
