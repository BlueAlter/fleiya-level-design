//! Right panel shell for the Tileset workspace.

use egui::{
    vec2, Color32, CornerRadius, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
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
                        // Ensure an active value is selected if possible
                        if state.selected_marking_value.is_none() {
                            state.selected_marking_value =
                                enum_def.values.first().map(|v| v.id.clone());
                        } else if let Some(ref sel) = state.selected_marking_value {
                            if enum_def.value(sel).is_none() {
                                state.selected_marking_value =
                                    enum_def.values.first().map(|v| v.id.clone());
                            }
                        }

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
                                    let is_selected = state.selected_marking_value.as_deref()
                                        == Some(&val.id);
                                    let marked_count = tileset
                                        .marking
                                        .tags
                                        .get(&val.id)
                                        .map_or(0, |s| s.len());

                                    let row_height = 26.0;
                                    let (row_rect, row_resp) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), row_height),
                                        Sense::click(),
                                    );

                                    if is_selected {
                                        ui.painter().rect_filled(
                                            row_rect,
                                            CornerRadius::same(3),
                                            t.selection_fill,
                                        );
                                        ui.painter().rect_stroke(
                                            row_rect,
                                            CornerRadius::same(3),
                                            Stroke::new(1.0, t.selection_stroke),
                                            StrokeKind::Inside,
                                        );
                                    } else if row_resp.hovered() {
                                        ui.painter().rect_filled(
                                            row_rect,
                                            CornerRadius::same(3),
                                            t.sunken,
                                        );
                                    }

                                    if row_resp.clicked() {
                                        state.selected_marking_value = Some(val.id.clone());
                                    }

                                    let chip_size = 14.0;
                                    let chip_y = row_rect.center().y - chip_size / 2.0;
                                    let chip_rect = Rect::from_min_size(
                                        Pos2::new(row_rect.min.x + 8.0, chip_y),
                                        vec2(chip_size, chip_size),
                                    );
                                    let color = Color32::from_rgba_unmultiplied(
                                        val.color[0],
                                        val.color[1],
                                        val.color[2],
                                        val.color[3],
                                    );
                                    ui.painter().rect_filled(chip_rect, CornerRadius::same(2), color);
                                    ui.painter().rect_stroke(
                                        chip_rect,
                                        CornerRadius::same(2),
                                        t.hairline(),
                                        StrokeKind::Inside,
                                    );

                                    // Value name
                                    let text_pos =
                                        Pos2::new(chip_rect.max.x + 8.0, row_rect.center().y);
                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::LEFT_CENTER,
                                        &val.name,
                                        egui::FontId::proportional(12.0),
                                        if is_selected {
                                            t.accent_ink
                                        } else {
                                            t.ink_strong
                                        },
                                    );

                                    // Count on right
                                    if marked_count > 0 {
                                        let count_pos =
                                            Pos2::new(row_rect.max.x - 8.0, row_rect.center().y);
                                        ui.painter().text(
                                            count_pos,
                                            egui::Align2::RIGHT_CENTER,
                                            format!("{marked_count}"),
                                            egui::FontId::proportional(11.0),
                                            t.muted,
                                        );
                                    }

                                    ui.add_space(2.0);
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
