//! Attributes card and image metrics info line for the selected tileset definition.

use egui::{vec2, Color32, DragValue, Id, RichText, Ui};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{compute_tileset_grid, GridMetrics, UpdateTilesetDefCommand};

use super::file_dialog::TilesetDialogManager;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::def_workspace::{def_field_row, def_section};
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::text_field::{text_field, validate_identifier, FieldStyle};
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

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
            // Row 1: Identifier
            def_field_row(ui, t, tr(keys::FIELD_IDENTIFIER), |ui| {
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

                let res = text_field(ui, &mut name_buf, style);
                if res.changed {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(Id::new(("ts_name_buf", tileset.id)), name_buf.clone())
                    });
                }

                if res.committed {
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

            ui.add_space(6.0);

            // Row 2: Source image
            def_field_row(ui, t, tr(keys::FIELD_SOURCE_IMAGE), |ui| {
                ui.horizontal(|ui| {
                    let mut path_buf = ui
                        .ctx()
                        .data(|d| d.get_temp::<String>(Id::new(("ts_path_buf", tileset.id))))
                        .unwrap_or_else(|| tileset.path.clone());

                    let field_w = (ui.available_width() - (28.0 * 2.0 + 8.0 * 2.0)).max(100.0);
                    let res = ui
                        .allocate_ui(vec2(field_w, CONTROL_HEIGHT_FORM), |ui| {
                            text_field(ui, &mut path_buf, FieldStyle::default().with_outline(true))
                        })
                        .inner;

                    if res.changed {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::new(("ts_path_buf", tileset.id)), path_buf.clone())
                        });
                    }

                    if res.committed {
                        let trimmed = path_buf.trim();
                        if trimmed != tileset.path {
                            let mut updated = tileset.clone();
                            updated.path = trimmed.to_string();
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        }
                    }

                    // Browse button
                    let browse_btn = icon_button(ui, Icon::Open, 28.0, IconButtonStyle::bordered())
                        .on_hover_text(tr(keys::BUTTON_BROWSE));
                    if browse_btn.clicked() {
                        dialogs.request_browse_image(tileset.id);
                    }

                    // Clear button
                    let clear_btn = icon_button(ui, Icon::Close, 28.0, IconButtonStyle::bordered())
                        .on_hover_text(tr(keys::BUTTON_CLEAR));
                    if clear_btn.clicked() && !tileset.path.is_empty() {
                        let mut updated = tileset.clone();
                        updated.path = String::new();
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::new(("ts_path_buf", tileset.id)), String::new())
                        });
                    }
                });
            });

            ui.add_space(6.0);

            // Row 3: Tile size (W / H), Spacing, Padding
            def_field_row(ui, t, tr(keys::FIELD_TILE_SIZE), |ui| {
                ui.horizontal(|ui| {
                    let mut tile_w = tileset.tile_size.x;
                    let resp_tw = ui.add(
                        DragValue::new(&mut tile_w)
                            .range(1..=512)
                            .speed(1.0)
                            .prefix("W: ")
                            .suffix(" px"),
                    );

                    let mut tile_h = tileset.tile_size.y;
                    let resp_th = ui.add(
                        DragValue::new(&mut tile_h)
                            .range(1..=512)
                            .speed(1.0)
                            .prefix("H: ")
                            .suffix(" px"),
                    );

                    let tile_size_changed =
                        tile_w != tileset.tile_size.x || tile_h != tileset.tile_size.y;
                    let tile_size_committed = resp_tw.drag_stopped()
                        || resp_tw.lost_focus()
                        || resp_th.drag_stopped()
                        || resp_th.lost_focus()
                        || ((resp_tw.has_focus() || resp_th.has_focus())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)));

                    if tile_size_changed && tile_size_committed {
                        let mut updated = tileset.clone();
                        updated.tile_size = UVec2::new(tile_w, tile_h);
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }

                    ui.add_space(16.0);

                    // Spacing
                    ui.label(
                        RichText::new(format!("{}:", tr(keys::FIELD_SPACING)))
                            .size(11.5)
                            .color(t.ink_strong),
                    );
                    let mut spacing = tileset.spacing;
                    let resp_sp = ui.add(
                        DragValue::new(&mut spacing)
                            .range(0..=64)
                            .speed(1.0)
                            .suffix(" px"),
                    );
                    let sp_committed = resp_sp.drag_stopped()
                        || resp_sp.lost_focus()
                        || (resp_sp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                    if spacing != tileset.spacing && sp_committed {
                        let mut updated = tileset.clone();
                        updated.spacing = spacing;
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }

                    ui.add_space(16.0);

                    // Padding
                    ui.label(
                        RichText::new(format!("{}:", tr(keys::FIELD_PADDING)))
                            .size(11.5)
                            .color(t.ink_strong),
                    );
                    let mut padding = tileset.padding;
                    let resp_pd = ui.add(
                        DragValue::new(&mut padding)
                            .range(0..=64)
                            .speed(1.0)
                            .suffix(" px"),
                    );
                    let pd_committed = resp_pd.drag_stopped()
                        || resp_pd.lost_focus()
                        || (resp_pd.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                    if padding != tileset.padding && pd_committed {
                        let mut updated = tileset.clone();
                        updated.padding = padding;
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }
                });
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
                    "Atlas {}x{} px -> {} x {} tiles ({})",
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
