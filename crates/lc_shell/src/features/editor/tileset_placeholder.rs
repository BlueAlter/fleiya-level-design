//! Temporary placeholder view for the Tilesets workspace.

use egui::{CornerRadius, RichText, Ui};
use lc_canvas::theme::{self, Tokens};

/// Renders the temporary placeholder page for the Tilesets workspace.
///
/// This page is temporary: the real workspace task replaces it entirely when the
/// tileset workspace lands, so plain English literals are acceptable here. It must not
/// mutate the project or queue commands.
pub fn render_tileset_placeholder(ui: &mut Ui, t: &Tokens) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Tilesets")
                        .font(egui::FontId::new(26.0, theme::display_for(ui.ctx())))
                        .color(t.ink_strong),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "The tileset workspace is under construction. For now use the Tileset Window from the Windows menu.",
                    )
                    .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(t.muted),
                );
            });
        });
    });
}
