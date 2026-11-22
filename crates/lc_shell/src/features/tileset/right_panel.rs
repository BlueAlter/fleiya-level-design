//! Right panel shell for the Tileset workspace.

use egui::{vec2, Color32, CornerRadius, RichText, ScrollArea, Sense, StrokeKind, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::project::Project;
use lc_feature_tileset::mode_availability;

use crate::features::tileset::state::TilesetViewState;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Renders the right panel shell (mode dropdown and tool content area).
pub fn render_right_panel(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    state: &mut TilesetViewState,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add_space(8.0);

        // Mode dropdown toolbar (32 high, framed button with chevron icon)
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let modes = mode_availability(tileset);
            let current_label = modes
                .iter()
                .find(|m| m.id == state.active_mode)
                .map(|m| m.label)
                .unwrap_or("None");

            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
            let combo_w = (ui.available_width() - 8.0).max(100.0);

            egui::ComboBox::from_id_salt("tileset_mode_selector")
                .selected_text(current_label)
                .width(combo_w)
                .icon(|ui, rect, visuals, is_open| {
                    let icon = if is_open {
                        Icon::ChevronUp
                    } else {
                        Icon::ChevronDown
                    };
                    lc_canvas::icons::paint_icon(
                        ui.painter(),
                        rect.center(),
                        icon,
                        12.0,
                        visuals.text_color(),
                    );
                })
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
                            let btn = ui.add_enabled(false, egui::Button::new(mode.label));
                            if !mode.hint.is_empty() {
                                btn.on_disabled_hover_text(mode.hint);
                            }
                        }
                    }
                });
        });

        ui.add_space(8.0);

        // Content for selected mode
        match state.active_mode.as_str() {
            "None" => {
                // Empty content for "None" mode: clicks never change data
                ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
            }
            "Marking" => {
                if let Some(source_id) = tileset.marking.source_enum {
                    if let Some(enum_def) = project.defs.enum_def(source_id) {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!("Values ({})", enum_def.values.len()))
                                    .size(11.0)
                                    .strong()
                                    .color(t.accent_ink),
                            );
                        });
                        ui.add_space(6.0);

                        ScrollArea::vertical()
                            .id_salt("tileset_marking_values_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for val in &enum_def.values {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        // Color chip
                                        let (chip_rect, _) = ui
                                            .allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                                        let color = Color32::from_rgba_unmultiplied(
                                            val.color[0],
                                            val.color[1],
                                            val.color[2],
                                            val.color[3],
                                        );
                                        ui.painter().rect_filled(
                                            chip_rect,
                                            CornerRadius::same(2),
                                            color,
                                        );
                                        ui.painter().rect_stroke(
                                            chip_rect,
                                            CornerRadius::same(2),
                                            t.hairline(),
                                            StrokeKind::Inside,
                                        );

                                        // Value Name
                                        ui.label(
                                            RichText::new(&val.name).size(12.0).color(t.ink_strong),
                                        );
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new("(Missing source enum)")
                                    .font(egui::FontId::proportional(12.0))
                                    .color(t.muted),
                            );
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("No source enum selected")
                                .font(egui::FontId::proportional(12.0))
                                .color(t.muted),
                        );
                    });
                }
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
}
