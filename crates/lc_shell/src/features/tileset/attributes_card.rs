//! Attributes card and image metrics info line for the selected tileset definition.

use egui::{vec2, CornerRadius, DragValue, Frame, Id, Margin, RichText, Stroke, TextEdit, Ui};
use glam::UVec2;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{compute_tileset_grid, GridMetrics, UpdateTilesetDefCommand};

use super::file_dialog::TilesetDialogManager;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;

/// Renders the "TILESET IDENTIFIER & ATTRIBUTES" card and the image info line below it.
pub fn render_attributes_card(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    dialogs: &TilesetDialogManager,
    texture: Option<&egui::TextureHandle>,
) {
    let card_frame = Frame::new()
        .fill(t.panel)
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, t.line_strong))
        .inner_margin(Margin::same(12));

    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // Card Title
            ui.label(
                RichText::new(tr(keys::CARD_IDENTIFIER_ATTRIBUTES))
                    .font(egui::FontId::proportional(11.0))
                    .color(t.muted),
            );
            ui.add_space(8.0);

            // Row 1: Identifier & Source Image
            ui.horizontal(|ui| {
                // Identifier
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(tr(keys::FIELD_IDENTIFIER))
                            .font(egui::FontId::proportional(11.5))
                            .color(t.muted),
                    );

                    let mut name_buf = ui
                        .ctx()
                        .data(|d| d.get_temp::<String>(Id::new(("ts_name_buf", tileset.id))))
                        .unwrap_or_else(|| tileset.name.clone());

                    let resp = ui.add_sized(vec2(160.0, 24.0), TextEdit::singleline(&mut name_buf));

                    let enter_pressed =
                        resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if resp.changed() {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::new(("ts_name_buf", tileset.id)), name_buf.clone())
                        });
                    }

                    if resp.lost_focus() || enter_pressed {
                        let trimmed = name_buf.trim();
                        if !trimmed.is_empty()
                            && trimmed != tileset.name
                            && !app
                                .project
                                .defs
                                .tilesets
                                .iter()
                                .any(|other| other.id != tileset.id && other.name == trimmed)
                        {
                            let mut updated = tileset.clone();
                            updated.name = trimmed.to_string();
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                        } else {
                            // Revert buffer to actual name
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(
                                    Id::new(("ts_name_buf", tileset.id)),
                                    tileset.name.clone(),
                                )
                            });
                        }
                    }
                });

                ui.add_space(16.0);

                // Source Image path + Browse + Clear
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(tr(keys::FIELD_SOURCE_IMAGE))
                            .font(egui::FontId::proportional(11.5))
                            .color(t.muted),
                    );

                    ui.horizontal(|ui| {
                        let mut path_buf = ui
                            .ctx()
                            .data(|d| d.get_temp::<String>(Id::new(("ts_path_buf", tileset.id))))
                            .unwrap_or_else(|| tileset.path.clone());

                        let path_w = (ui.available_width() - 140.0).max(120.0);
                        let resp =
                            ui.add_sized(vec2(path_w, 24.0), TextEdit::singleline(&mut path_buf));

                        let enter_pressed =
                            resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if resp.changed() {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(
                                    Id::new(("ts_path_buf", tileset.id)),
                                    path_buf.clone(),
                                )
                            });
                        }

                        if resp.lost_focus() || enter_pressed {
                            let trimmed = path_buf.trim();
                            if trimmed != tileset.path {
                                let mut updated = tileset.clone();
                                updated.path = trimmed.to_string();
                                app.execute_command(Box::new(UpdateTilesetDefCommand::new(
                                    updated,
                                )));
                            }
                        }

                        // Browse button
                        if ui.button(tr(keys::BUTTON_BROWSE)).clicked() {
                            dialogs.request_browse_image(tileset.id);
                        }

                        // Clear button
                        if ui.button(tr(keys::BUTTON_CLEAR)).clicked() && !tileset.path.is_empty() {
                            let mut updated = tileset.clone();
                            updated.path = String::new();
                            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(Id::new(("ts_path_buf", tileset.id)), String::new())
                            });
                        }
                    });
                });
            });

            ui.add_space(10.0);

            // Row 2: Tile Size (W / H), Spacing, Padding
            ui.horizontal(|ui| {
                // Tile Width
                let mut tile_w = tileset.tile_size.x;
                ui.label(
                    RichText::new(tr(keys::FIELD_TILE_SIZE))
                        .font(egui::FontId::proportional(11.5))
                        .color(t.muted),
                );
                let resp_tw = ui.add(
                    DragValue::new(&mut tile_w)
                        .range(1..=512)
                        .speed(1.0)
                        .prefix("W: ")
                        .suffix(" px"),
                );

                // Tile Height
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
                    RichText::new(tr(keys::FIELD_SPACING))
                        .font(egui::FontId::proportional(11.5))
                        .color(t.muted),
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
                    RichText::new(tr(keys::FIELD_PADDING))
                        .font(egui::FontId::proportional(11.5))
                        .color(t.muted),
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

    ui.add_space(6.0);

    // Image info line under the card
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
                        "⚠ {}px width and {}px height unused",
                        metrics.leftover_x, metrics.leftover_y
                    ))
                    .font(egui::FontId::proportional(11.5))
                    .color(egui::Color32::from_rgb(234, 179, 8)),
                );
            }
        } else if !tileset.path.is_empty() {
            ui.label(
                RichText::new("⚠ Source image file could not be loaded.")
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
