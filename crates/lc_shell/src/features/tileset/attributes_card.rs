//! Attributes card and image metrics info line for the selected tileset definition.

use std::ops::RangeInclusive;
use std::path::Path;

use egui::{
    vec2, Align, CornerRadius, Id, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{compute_tileset_grid, GridMetrics, UpdateTilesetDefCommand};

use super::file_dialog::TilesetDialogManager;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::def_workspace::def_section;
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

/// Renders a file picker button with a label/filename, hover outline, and file status coloring.
pub fn file_picker_button(
    ui: &mut Ui,
    t: &Tokens,
    filename: &str,
    is_empty: bool,
    is_missing: bool,
    tooltip: &str,
    desired_width: f32,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(desired_width, CONTROL_HEIGHT_FORM), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = resp.hovered();
        painter.rect_filled(rect, CornerRadius::same(3), t.sunken);
        let stroke = if hovered {
            Stroke::new(1.0, t.accent)
        } else {
            t.hairline()
        };
        painter.rect_stroke(rect, CornerRadius::same(3), stroke, StrokeKind::Inside);

        let text_color = if is_empty {
            t.muted
        } else if is_missing {
            t.bad
        } else {
            t.ink_strong
        };

        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 8.0, rect.min.y),
            Pos2::new(rect.max.x - 8.0, rect.max.y),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        child.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(RichText::new(filename).color(text_color));
        });
    }

    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
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

/// Renders the "TILESET IDENTIFIER & ATTRIBUTES" section and the image info line below it.
pub fn render_attributes_card(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    dialogs: &TilesetDialogManager,
    mut texture: Option<&mut egui::TextureHandle>,
) {
    def_section(ui, t, tr(keys::CARD_IDENTIFIER_ATTRIBUTES), |ui| {
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

                let picker_resp =
                    file_picker_button(ui, t, display_name, is_empty, is_missing, &tooltip, btn_w);
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

                let mut style = FieldStyle::default().with_outline_preset(FieldOutline::Hairline);
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
                        d.insert_temp(Id::new(("ts_name_buf", tileset.id)), tileset.name.clone())
                    });
                } else if res.committed {
                    if validation.is_ok() {
                        let trimmed = name_buf.trim();
                        if trimmed != tileset.name {
                            let mut updated = tileset.clone();
                            updated.name = trimmed.to_string();
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
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
                    numeric_field(
                        app,
                        ui,
                        t,
                        tileset,
                        "W",
                        "ts_tile_w_buf",
                        tileset.tile_size.x,
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
                        tileset.tile_size.y,
                        1..=512,
                        "Invalid tile height (1-512)",
                        |ts, v| ts.tile_size.y = v,
                    );
                    ui.add_space(10.0);
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
                });
            } else {
                def_attribute_row(ui, t, tr(keys::FIELD_TILES_LAYOUT), |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    numeric_field(
                        app,
                        ui,
                        t,
                        tileset,
                        "W",
                        "ts_tile_w_buf",
                        tileset.tile_size.x,
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
                        tileset.tile_size.y,
                        1..=512,
                        "Invalid tile height (1-512)",
                        |ts, v| ts.tile_size.y = v,
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let (label_rect, _) = ui.allocate_exact_size(
                        vec2(LABEL_COL_W, CONTROL_HEIGHT_FORM),
                        Sense::hover(),
                    );
                    let _ = label_rect;
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
                });
            }
        });
    });

    ui.add_space(6.0);

    // Image info line under the card (no emoji or pictograph characters)
    let (img_w, img_h) = if let Some(tex) = texture {
        let sz = tex.size();
        (sz[0] as u32, sz[1] as u32)
    } else {
        (0, 0)
    };

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        if img_w > 0 && img_h > 0 {
            let metrics: GridMetrics = compute_tileset_grid(
                [img_w, img_h],
                tileset.tile_size,
                tileset.spacing,
                tileset.padding,
            );
            ui.label(
                RichText::new(format!(
                    "Atlas {}x{} px, {} x {} tiles ({})",
                    img_w, img_h, metrics.cols, metrics.rows, metrics.total_tiles
                ))
                .font(egui::FontId::proportional(12.0))
                .color(t.muted),
            );

            if metrics.leftover_x > 0 || metrics.leftover_y > 0 {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!(
                        "Warning: {}px width and {}px height unused",
                        metrics.leftover_x, metrics.leftover_y
                    ))
                    .font(egui::FontId::proportional(11.5))
                    .color(egui::Color32::from_rgb(234, 179, 8)),
                );
            }
        } else if !tileset.path.is_empty() {
            ui.label(
                RichText::new("Warning: Source image file could not be loaded.")
                    .font(egui::FontId::proportional(11.5))
                    .color(egui::Color32::from_rgb(234, 179, 8)),
            );
        } else {
            ui.label(
                RichText::new("No image assigned.")
                    .font(egui::FontId::proportional(12.0))
                    .color(t.muted),
            );
        }
    });
}
