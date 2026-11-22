//! Attributes card and image metrics info line for the selected tileset definition.

use egui::{
    vec2, Align, Color32, CornerRadius, Id, Key, Layout, Margin, RichText, Sense, Stroke,
    StrokeKind, TextEdit, Ui,
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
use crate::shared::ui::text_field::{validate_identifier, FieldResponse, FieldStyle};
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Local text field wrapper ensuring vertically centered text and 8px padding.
fn tileset_text_field(
    ui: &mut Ui,
    text: &mut String,
    style: FieldStyle<'_>,
    width: Option<f32>,
) -> FieldResponse {
    let t = Tokens::get(ui.ctx());
    let mut edit = TextEdit::singleline(text)
        .vertical_align(Align::Center)
        .margin(Margin::symmetric(8, 0));

    if let Some(w) = width {
        edit = edit.desired_width(w).min_size(vec2(w, CONTROL_HEIGHT_FORM));
    } else {
        let avail_w = ui.available_width();
        edit = edit
            .desired_width(avail_w)
            .min_size(vec2(avail_w, CONTROL_HEIGHT_FORM));
    }

    if style.monospace {
        edit = edit.font(egui::TextStyle::Monospace);
    }
    if let Some(hint) = style.hint {
        edit = edit.hint_text(hint);
    }

    let response = if let Some(w) = width {
        ui.add_sized([w, CONTROL_HEIGHT_FORM], edit)
    } else {
        let avail_w = ui.available_width();
        ui.add_sized([avail_w, CONTROL_HEIGHT_FORM], edit)
    };

    let changed = response.changed();
    let esc_pressed = response.has_focus() && ui.input(|i| i.key_pressed(Key::Escape));
    let enter_pressed = response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let lost_focus = response.lost_focus();

    let cancelled = esc_pressed;
    let committed = (lost_focus && !cancelled) || enter_pressed;

    if cancelled {
        response.surrender_focus();
    }

    if let Some(err) = style.error {
        ui.painter().rect_stroke(
            response.rect,
            CornerRadius::same(4),
            Stroke::new(1.0, t.bad),
            StrokeKind::Inside,
        );
        response.clone().on_hover_text(err);
    } else if style.outline || response.has_focus() {
        let stroke_color = if response.has_focus() {
            t.accent
        } else {
            t.line
        };
        ui.painter().rect_stroke(
            response.rect,
            CornerRadius::same(4),
            Stroke::new(1.0, stroke_color),
            StrokeKind::Inside,
        );
    }

    FieldResponse {
        response,
        changed,
        committed,
        cancelled,
    }
}

/// Renders a definition property row with a fixed-width label column.
fn def_attribute_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let (label_rect, _) =
            ui.allocate_exact_size(vec2(106.78125, CONTROL_HEIGHT_FORM), Sense::hover());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(label_rect));
        child.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(RichText::new(label).strong().color(t.ink_strong));
        });
        control(ui)
    })
    .inner
}

/// Renders the "TILESET IDENTIFIER & ATTRIBUTES" section and the image info line below it.
pub fn render_attributes_card(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    dialogs: &TilesetDialogManager,
    texture: Option<&egui::TextureHandle>,
) {
    def_section(ui, t, tr(keys::CARD_IDENTIFIER_ATTRIBUTES), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            // Row 1: Identifier
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

                let mut style = FieldStyle::default().with_outline(true);
                if let Some(ref err) = error_msg {
                    style = style.with_error(err.as_str());
                }

                let res = tileset_text_field(ui, &mut name_buf, style, None);
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

            // Row 2: Source image
            def_attribute_row(ui, t, tr(keys::FIELD_SOURCE_IMAGE), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                let mut path_buf = ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(Id::new(("ts_path_buf", tileset.id))))
                    .unwrap_or_else(|| tileset.path.clone());

                let card_inner_right = 1354.0 - 12.0 - 1.0;
                let trailing_w = 32.0 + 6.0 + 32.0;
                let gap = 6.0;
                let field_w = (card_inner_right - ui.cursor().min.x - trailing_w - gap).max(60.0);

                let res = tileset_text_field(
                    ui,
                    &mut path_buf,
                    FieldStyle::default().with_outline(true),
                    Some(field_w),
                );
                if res.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_path_buf", tileset.id)), path_buf.clone())
                    });
                }
                if res.cancelled {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_path_buf", tileset.id)), tileset.path.clone())
                    });
                } else if res.committed {
                    let trimmed = path_buf.trim();
                    if trimmed != tileset.path {
                        let mut updated = tileset.clone();
                        updated.path = trimmed.to_string();
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }
                }

                ui.add_space(gap);

                // Browse button (32x32)
                let (browse_rect, browse_resp) =
                    ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                let browse_hovered = browse_resp.hovered();
                ui.painter().rect_filled(
                    browse_rect,
                    CornerRadius::same(3),
                    if browse_hovered { t.sunken } else { t.panel },
                );
                ui.painter().rect_stroke(
                    browse_rect,
                    CornerRadius::same(3),
                    t.hairline(),
                    StrokeKind::Inside,
                );
                lc_canvas::icons::paint_icon(
                    ui.painter(),
                    browse_rect.center(),
                    Icon::Open,
                    14.0,
                    t.ink_strong,
                );
                let browse_btn = browse_resp.on_hover_text(tr(keys::BUTTON_BROWSE));
                if browse_btn.clicked() {
                    dialogs.request_browse_image(tileset.id);
                }

                ui.add_space(gap);

                // Clear button (32x32)
                let (clear_rect, clear_resp) =
                    ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                let clear_hovered = clear_resp.hovered();
                ui.painter().rect_filled(
                    clear_rect,
                    CornerRadius::same(3),
                    if clear_hovered { t.sunken } else { t.panel },
                );
                ui.painter().rect_stroke(
                    clear_rect,
                    CornerRadius::same(3),
                    t.hairline(),
                    StrokeKind::Inside,
                );
                lc_canvas::icons::paint_icon(
                    ui.painter(),
                    clear_rect.center(),
                    Icon::Close,
                    14.0,
                    t.ink_strong,
                );
                let clear_btn = clear_resp.on_hover_text(tr(keys::BUTTON_CLEAR));
                if clear_btn.clicked() && !tileset.path.is_empty() {
                    let mut updated = tileset.clone();
                    updated.path = String::new();
                    app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_path_buf", tileset.id)), String::new())
                    });
                }
            });

            ui.add_space(8.0);

            // Row 3: Tile size
            def_attribute_row(ui, t, tr(keys::FIELD_TILE_SIZE), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let mut w_buf = ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(Id::new(("ts_tile_w_buf", tileset.id))))
                    .unwrap_or_else(|| tileset.tile_size.x.to_string());

                let mut h_buf = ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(Id::new(("ts_tile_h_buf", tileset.id))))
                    .unwrap_or_else(|| tileset.tile_size.y.to_string());

                let w_parse = w_buf
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|&v| (1..=512).contains(&v));
                let h_parse = h_buf
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|&v| (1..=512).contains(&v));

                let mut w_style = FieldStyle::default().with_outline(true);
                if w_parse.is_none() && !w_buf.trim().is_empty() {
                    w_style = w_style.with_error("Invalid tile width (1-512)");
                }
                let res_w = tileset_text_field(ui, &mut w_buf, w_style, Some(96.0));
                if res_w.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_tile_w_buf", tileset.id)), w_buf.clone())
                    });
                }
                if res_w.cancelled {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            Id::new(("ts_tile_w_buf", tileset.id)),
                            tileset.tile_size.x.to_string(),
                        )
                    });
                } else if res_w.committed {
                    if let Some(val) = w_parse {
                        if val != tileset.tile_size.x {
                            let mut updated = tileset.clone();
                            updated.tile_size.x = val;
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    } else {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(
                                Id::new(("ts_tile_w_buf", tileset.id)),
                                tileset.tile_size.x.to_string(),
                            )
                        });
                    }
                }

                ui.label(RichText::new("px").size(12.0).color(t.muted));
                ui.add_space(10.0);

                let mut h_style = FieldStyle::default().with_outline(true);
                if h_parse.is_none() && !h_buf.trim().is_empty() {
                    h_style = h_style.with_error("Invalid tile height (1-512)");
                }
                let res_h = tileset_text_field(ui, &mut h_buf, h_style, Some(96.0));
                if res_h.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_tile_h_buf", tileset.id)), h_buf.clone())
                    });
                }
                if res_h.cancelled {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            Id::new(("ts_tile_h_buf", tileset.id)),
                            tileset.tile_size.y.to_string(),
                        )
                    });
                } else if res_h.committed {
                    if let Some(val) = h_parse {
                        if val != tileset.tile_size.y {
                            let mut updated = tileset.clone();
                            updated.tile_size.y = val;
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    } else {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(
                                Id::new(("ts_tile_h_buf", tileset.id)),
                                tileset.tile_size.y.to_string(),
                            )
                        });
                    }
                }

                ui.label(RichText::new("px").size(12.0).color(t.muted));
            });

            ui.add_space(8.0);

            // Row 4: Spacing
            def_attribute_row(ui, t, tr(keys::FIELD_SPACING), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let mut sp_buf = ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(Id::new(("ts_spacing_buf", tileset.id))))
                    .unwrap_or_else(|| tileset.spacing.to_string());

                let sp_parse = sp_buf.trim().parse::<u32>().ok().filter(|&v| v <= 64);
                let mut sp_style = FieldStyle::default().with_outline(true);
                if sp_parse.is_none() && !sp_buf.trim().is_empty() {
                    sp_style = sp_style.with_error("Invalid spacing (0-64)");
                }

                let res_sp = tileset_text_field(ui, &mut sp_buf, sp_style, Some(96.0));
                if res_sp.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_spacing_buf", tileset.id)), sp_buf.clone())
                    });
                }
                if res_sp.cancelled {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            Id::new(("ts_spacing_buf", tileset.id)),
                            tileset.spacing.to_string(),
                        )
                    });
                } else if res_sp.committed {
                    if let Some(val) = sp_parse {
                        if val != tileset.spacing {
                            let mut updated = tileset.clone();
                            updated.spacing = val;
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    } else {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(
                                Id::new(("ts_spacing_buf", tileset.id)),
                                tileset.spacing.to_string(),
                            )
                        });
                    }
                }

                ui.label(RichText::new("px").size(12.0).color(t.muted));
            });

            ui.add_space(8.0);

            // Row 5: Padding
            def_attribute_row(ui, t, tr(keys::FIELD_PADDING), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let mut pd_buf = ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(Id::new(("ts_padding_buf", tileset.id))))
                    .unwrap_or_else(|| tileset.padding.to_string());

                let pd_parse = pd_buf.trim().parse::<u32>().ok().filter(|&v| v <= 64);
                let mut pd_style = FieldStyle::default().with_outline(true);
                if pd_parse.is_none() && !pd_buf.trim().is_empty() {
                    pd_style = pd_style.with_error("Invalid padding (0-64)");
                }

                let res_pd = tileset_text_field(ui, &mut pd_buf, pd_style, Some(96.0));
                if res_pd.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_padding_buf", tileset.id)), pd_buf.clone())
                    });
                }
                if res_pd.cancelled {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            Id::new(("ts_padding_buf", tileset.id)),
                            tileset.padding.to_string(),
                        )
                    });
                } else if res_pd.committed {
                    if let Some(val) = pd_parse {
                        if val != tileset.padding {
                            let mut updated = tileset.clone();
                            updated.padding = val;
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    } else {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(
                                Id::new(("ts_padding_buf", tileset.id)),
                                tileset.padding.to_string(),
                            )
                        });
                    }
                }

                ui.label(RichText::new("px").size(12.0).color(t.muted));
            });
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
                    .color(Color32::from_rgb(234, 179, 8)),
                );
            }
        } else if !tileset.path.is_empty() {
            ui.label(
                RichText::new("Warning: Source image file could not be loaded.")
                    .font(egui::FontId::proportional(11.5))
                    .color(Color32::from_rgb(234, 179, 8)),
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
