//! Attributes card and image metrics info line for the selected tileset definition.

use std::ops::RangeInclusive;
use std::path::Path;

use egui::{vec2, Align, CornerRadius, Id, Layout, RichText, Sense, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{compute_tileset_grid, GridMetrics, UpdateTilesetDefCommand};

use super::file_dialog::TilesetDialogManager;
pub use super::file_picker::file_picker_button;
use super::i18n::keys;
use super::tags_row::{render_marking_enum_row, render_tags_row};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::text_field::{text_field, validate_identifier, FieldOutline, FieldStyle};
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Width of the label column in definition attribute rows, matching Enum workspace.
pub(crate) const LABEL_COL_W: f32 = 106.78125;

/// Renders a definition property row with a fixed-width label column.
pub(crate) fn def_attribute_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let (label_rect, _) =
            ui.allocate_exact_size(vec2(LABEL_COL_W, CONTROL_HEIGHT_FORM), Sense::hover());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(label_rect));
        child.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(RichText::new(label).strong().color(t.ink_strong));
        });
        control(ui)
    })
    .inner
}

/// Renders a small numeric text_field with a label and "px" suffix, handling commit/cancel.
#[allow(clippy::too_many_arguments)]
fn numeric_field(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    label: &str,
    temp_key: &'static str,
    current_val: u32,
    valid_range: RangeInclusive<u32>,
    err_text: &str,
    update_fn: impl FnOnce(&mut TilesetDef, u32),
) {
    let key = (temp_key, tileset.id);
    let mut buf = ui
        .ctx()
        .data(|d| d.get_temp::<String>(Id::new(key)))
        .unwrap_or_else(|| current_val.to_string());

    let parsed = buf
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|v| valid_range.contains(v));

    ui.label(RichText::new(label).strong().color(t.ink_strong));
    ui.add_space(4.0);

    let mut style = FieldStyle::default().with_outline_preset(FieldOutline::Hairline);
    if parsed.is_none() && !buf.trim().is_empty() {
        style = style.with_error(err_text);
    }

    let res = ui
        .allocate_ui_with_layout(
            vec2(72.0, CONTROL_HEIGHT_FORM),
            Layout::left_to_right(Align::Center),
            |ui| text_field(ui, &mut buf, style),
        )
        .inner;

    ui.add_space(4.0);
    ui.label(RichText::new("px").size(12.0).color(t.muted));

    if res.changed {
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::new(key), buf.clone()));
    }
    if res.cancelled {
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::new(key), current_val.to_string()));
    } else if res.committed {
        if let Some(val) = parsed {
            if val != current_val {
                let mut updated = tileset.clone();
                update_fn(&mut updated, val);
                app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
            }
        } else {
            ui.ctx()
                .data_mut(|d| d.insert_temp(Id::new(key), current_val.to_string()));
        }
    }
}

fn render_w_h(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    let (tx, ty) = (tileset.tile_size.x, tileset.tile_size.y);
    numeric_field(
        app,
        ui,
        t,
        tileset,
        "W",
        "ts_tile_w_buf",
        tx,
        1..=512,
        "Invalid tile width (1-512)",
        |ts, v| ts.tile_size.x = v,
    );
    ui.add_space(10.0);
    numeric_field(
        app,
        ui,
        t,
        tileset,
        "H",
        "ts_tile_h_buf",
        ty,
        1..=512,
        "Invalid tile height (1-512)",
        |ts, v| ts.tile_size.y = v,
    );
}

fn render_sp_pd(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    numeric_field(
        app,
        ui,
        t,
        tileset,
        "Spacing",
        "ts_spacing_buf",
        tileset.spacing,
        0..=64,
        "Invalid spacing (0-64)",
        |ts, v| ts.spacing = v,
    );
    ui.add_space(10.0);
    numeric_field(
        app,
        ui,
        t,
        tileset,
        "Padding",
        "ts_padding_buf",
        tileset.padding,
        0..=64,
        "Invalid padding (0-64)",
        |ts, v| ts.padding = v,
    );
}

/// Renders the "TILESET IDENTIFIER & ATTRIBUTES" section and the image info line below it.
pub fn render_attributes_card(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    dialogs: &TilesetDialogManager,
    mut texture: Option<&mut egui::TextureHandle>,
) {
    // Section Header row: "TILESET IDENTIFIER & ATTRIBUTES" + right-aligned 28x28 chevron toggle
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr(keys::CARD_IDENTIFIER_ATTRIBUTES))
                .size(11.0)
                .strong()
                .color(t.accent_ink),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let chevron = if app.tileset_view_state.is_attributes_collapsed {
                Icon::ChevronDown
            } else {
                Icon::ChevronUp
            };
            let tooltip = if app.tileset_view_state.is_attributes_collapsed {
                "Show attributes"
            } else {
                "Hide attributes"
            };
            let btn =
                icon_button(ui, chevron, 28.0, IconButtonStyle::bordered()).on_hover_text(tooltip);
            if btn.clicked() {
                app.tileset_view_state.is_attributes_collapsed =
                    !app.tileset_view_state.is_attributes_collapsed;
            }
        });
    });

    if app.tileset_view_state.is_attributes_collapsed {
        return;
    }

    ui.add_space(4.0);

    egui::Frame::new()
        .fill(t.panel)
        .stroke(t.hairline())
        .inner_margin(egui::Margin::same(12))
        .corner_radius(CornerRadius::same(4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;

                // Row 1: Source image (button + 32x32 Reload + 32x32 Clear)
                def_attribute_row(ui, t, tr(keys::FIELD_SOURCE_IMAGE), |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    let full_path = app.assets.resolve_path(&tileset.path);
                    let is_empty = tileset.path.trim().is_empty();
                    let is_missing = !is_empty && !full_path.exists();

                    let display_name = if is_empty {
                        tr(keys::CHOOSE_IMAGE)
                    } else {
                        Path::new(&tileset.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&tileset.path)
                    };

                    let tooltip = if is_empty {
                        String::new()
                    } else if is_missing {
                        format!("File not found: {}", tileset.path)
                    } else {
                        tileset.path.clone()
                    };

                    let trailing_w = 32.0 + 6.0 + 32.0;
                    let gap = 6.0;
                    let btn_w = (ui.available_width() - trailing_w - gap).max(60.0);

                    let picker_resp = file_picker_button(
                        ui,
                        t,
                        display_name,
                        is_empty,
                        is_missing,
                        &tooltip,
                        btn_w,
                    );
                    if picker_resp.clicked() {
                        let start_dir = if !is_empty {
                            full_path.parent().map(|p| p.to_path_buf())
                        } else {
                            app.project_path
                                .as_deref()
                                .and_then(|p| p.parent())
                                .map(|p| p.to_path_buf())
                        };
                        dialogs.request_browse_image(tileset.id, start_dir);
                    }

                    ui.add_space(gap);

                    // Reload icon button (32x32)
                    ui.add_enabled_ui(!is_empty, |ui| {
                        let reload_resp =
                            icon_button(ui, Icon::Refresh, 32.0, IconButtonStyle::bordered())
                                .on_hover_text("Reload image from disk");
                        if reload_resp.clicked() {
                            if let Ok(bytes) = lc_io::read_bytes(&full_path) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let rgba = img.to_rgba8();
                                    let (w, h) = rgba.dimensions();
                                    let color_img = egui::ColorImage::from_rgba_unmultiplied(
                                        [w as usize, h as usize],
                                        &rgba.into_raw(),
                                    );
                                    if let Some(ref mut tex) = texture {
                                        tex.set(color_img, egui::TextureOptions::NEAREST);
                                    }
                                }
                            }
                            ui.ctx().request_repaint();
                        }
                    });

                    ui.add_space(gap);

                    // Clear icon button (32x32)
                    ui.add_enabled_ui(!is_empty, |ui| {
                        let clear_resp =
                            icon_button(ui, Icon::Close, 32.0, IconButtonStyle::bordered())
                                .on_hover_text(tr(keys::BUTTON_CLEAR));
                        if clear_resp.clicked() {
                            let mut updated = tileset.clone();
                            updated.path = String::new();
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    });
                });

                ui.add_space(8.0);

                // Row 2: Identifier (text_field + validate_identifier, fills row)
                def_attribute_row(ui, t, tr(keys::FIELD_IDENTIFIER), |ui| {
                    let mut name_buf = ui
                        .ctx()
                        .data(|d| d.get_temp::<String>(Id::new(("ts_name_buf", tileset.id))))
                        .unwrap_or_else(|| tileset.name.clone());

                    let existing_names: Vec<String> = app
                        .project
                        .defs
                        .tilesets
                        .iter()
                        .filter(|other| other.id != tileset.id)
                        .map(|other| other.name.clone())
                        .collect();

                    let validation =
                        validate_identifier(&name_buf, &existing_names, Some(&tileset.name));
                    let error_msg = validation.err().map(|e| e.to_string());

                    let mut style =
                        FieldStyle::default().with_outline_preset(FieldOutline::Hairline);
                    if let Some(ref err) = error_msg {
                        style = style.with_error(err.as_str());
                    }

                    let res = text_field(ui, &mut name_buf, style);
                    if res.changed {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::new(("ts_name_buf", tileset.id)), name_buf.clone())
                        });
                    }

                    if res.cancelled {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(
                                Id::new(("ts_name_buf", tileset.id)),
                                tileset.name.clone(),
                            )
                        });
                    } else if res.committed {
                        if validation.is_ok() {
                            let trimmed = name_buf.trim();
                            if trimmed != tileset.name {
                                let mut updated = tileset.clone();
                                updated.name = trimmed.to_string();
                                app.execute_command(Box::new(UpdateTilesetDefCommand::new(
                                    updated,
                                )));
                            }
                        } else {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(
                                    Id::new(("ts_name_buf", tileset.id)),
                                    tileset.name.clone(),
                                )
                            });
                        }
                    }
                });

                ui.add_space(8.0);

                // Row 3: Tiles layout (W, H, Spacing, Padding)
                let wrap = ui.available_width() < 760.0;
                if !wrap {
                    def_attribute_row(ui, t, tr(keys::FIELD_TILES_LAYOUT), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        render_w_h(app, ui, t, tileset);
                        ui.add_space(10.0);
                        render_sp_pd(app, ui, t, tileset);
                    });
                } else {
                    def_attribute_row(ui, t, tr(keys::FIELD_TILES_LAYOUT), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        render_w_h(app, ui, t, tileset);
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let _ = ui.allocate_exact_size(
                            vec2(LABEL_COL_W, CONTROL_HEIGHT_FORM),
                            Sense::hover(),
                        );
                        render_sp_pd(app, ui, t, tileset);
                    });
                }

                ui.add_space(8.0);

                // Row 4: Tags
                render_tags_row(app, ui, t, tileset);

                ui.add_space(8.0);

                // Row 5: Enum for tile marking
                render_marking_enum_row(app, ui, t, tileset);
            });
        });

    // Inline warning under the card ONLY when leftover pixels do not fit the grid or source image is missing
    let (img_w, img_h) = if let Some(tex) = texture {
        let sz = tex.size();
        (sz[0] as u32, sz[1] as u32)
    } else {
        (0, 0)
    };

    if img_w > 0 && img_h > 0 {
        let metrics: GridMetrics = compute_tileset_grid(
            [img_w, img_h],
            tileset.tile_size,
            tileset.spacing,
            tileset.padding,
        );
        if metrics.leftover_x > 0 || metrics.leftover_y > 0 {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let msg = if metrics.leftover_x > 0 && metrics.leftover_y > 0 {
                    format!(
                        "Warning: {}x{} image with {}x{} tiles leaves {} px right and {} px bottom unused",
                        img_w, img_h, tileset.tile_size.x, tileset.tile_size.y, metrics.leftover_x, metrics.leftover_y
                    )
                } else if metrics.leftover_x > 0 {
                    format!(
                        "Warning: {}x{} image with {}x{} tiles leaves {} px right unused",
                        img_w, img_h, tileset.tile_size.x, tileset.tile_size.y, metrics.leftover_x
                    )
                } else {
                    format!(
                        "Warning: {}x{} image with {}x{} tiles leaves {} px bottom unused",
                        img_w, img_h, tileset.tile_size.x, tileset.tile_size.y, metrics.leftover_y
                    )
                };
                ui.label(
                    RichText::new(msg)
                        .font(egui::FontId::proportional(11.5))
                        .color(egui::Color32::from_rgb(234, 179, 8)),
                );
            });
        }
    } else if !tileset.path.is_empty() {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Warning: Source image file could not be loaded.")
                    .font(egui::FontId::proportional(11.5))
                    .color(egui::Color32::from_rgb(234, 179, 8)),
            );
        });
    }
}
