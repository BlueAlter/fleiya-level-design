//! Right panel shell for the Tileset workspace.

use egui::{ComboBox, CornerRadius, RichText, Sense, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::mode_availability;

use crate::features::tileset::state::TilesetViewState;

/// Renders the right panel shell (mode dropdown and tool content area).
pub fn render_right_panel(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    state: &mut TilesetViewState,
) {
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.panel);
    ui.painter().vline(
        rect.min.x,
        rect.y_range(),
        egui::Stroke::new(1.0, t.line_strong),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.vertical(|ui| {
            // Mode toolbar
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Mode:")
                        .font(egui::FontId::proportional(12.0))
                        .color(t.muted),
                );

                let modes = mode_availability(tileset);
                let current_label = modes
                    .iter()
                    .find(|m| m.id == state.active_mode)
                    .map(|m| m.label)
                    .unwrap_or("None");

                ComboBox::from_id_salt("tileset_mode_dropdown")
                    .selected_text(current_label)
                    .width(140.0)
                    .show_ui(ui, |ui| {
                        for mode in modes {
                            if mode.available {
                                if ui
                                    .selectable_label(state.active_mode == mode.id, mode.label)
                                    .clicked()
                                {
                                    state.active_mode = mode.id.to_string();
                                }
                            } else {
                                let label = ui.add_enabled(false, egui::Button::new(mode.label));
                                if !mode.hint.is_empty() {
                                    label.on_disabled_hover_text(mode.hint);
                                }
                            }
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();

            // Content for selected mode
            match state.active_mode.as_str() {
                "None" => {
                    // Empty content for "None" mode: clicks never change data
                    ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
                }
                _ => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Coming in a later update")
                                .font(egui::FontId::proportional(12.0))
                                .color(t.muted),
                        );
                    });
                }
            }
        });
    });
}
