//! Atlas interactive viewport and tile grid view.

use egui::{vec2, CornerRadius, PointerButton, Rect, RichText, Sense, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{compute_tileset_grid, SetTileCustomDataCommand, SetTileMarkingCommand};

use super::atlas_overlay::{render_tile_overlay, TileOverlayContext};
use super::state::CUSTOM_DATA_VALUE_ID;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::atlas_canvas::{render_atlas_canvas, AtlasEvent, TileOverlayCallback};
use crate::shared::ui::custom_data_dialog::{render_custom_data_dialog, CustomDataOutcome};
use crate::shared::ui::tile_bounds::{TileGrid, TileRect};

const PAGE_BG_BYTES: &[u8] =
    include_bytes!("../../../../../assets/Bg/tileset_picker_bg_netral.png");

/// Fit rule from workspace spec:
/// zoom = min((avail_w - 32) / img_w, (avail_h - 32) / img_h, 8.0). Clamped to min 0.1, max 8.0.
/// pan = center of available rect - (img_size * zoom) / 2.
pub fn compute_atlas_fit(available_rect: egui::Rect, img_w: f32, img_h: f32) -> (f32, egui::Vec2) {
    if img_w <= 0.0 || img_h <= 0.0 {
        return (1.0, egui::Vec2::ZERO);
    }
    let avail_w = (available_rect.width() - 32.0).max(1.0);
    let avail_h = (available_rect.height() - 32.0).max(1.0);
    let zoom = ((avail_w / img_w).min(avail_h / img_h))
        .min(8.0)
        .clamp(0.1, 8.0);
    let zoomed_size = vec2(img_w * zoom, img_h * zoom);
    let pan = available_rect.center() - zoomed_size / 2.0;
    (zoom, pan.to_vec2())
}

/// Renders the atlas view with pan, zoom, hover tooltip, tile selection, and marking overlay via `render_atlas_canvas`.
pub fn render_atlas_view(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    texture: Option<&TextureHandle>,
) {
    let Some(tex) = texture else {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                if tileset.path.is_empty() {
                    ui.label(
                        RichText::new("No image loaded")
                            .font(egui::FontId::proportional(13.0))
                            .color(t.muted),
                    );
                } else {
                    ui.label(
                        RichText::new(format!(
                            "Warning: Source image file could not be loaded: {}",
                            tileset.path
                        ))
                        .font(egui::FontId::proportional(13.0))
                        .color(t.muted),
                    );
                }
            });
        });
        return;
    };

    let sz = tex.size();
    let (img_w, img_h) = (sz[0] as u32, sz[1] as u32);
    if img_w == 0 || img_h == 0 {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Warning: Image has zero dimensions")
                        .font(egui::FontId::proportional(13.0))
                        .color(t.muted),
                );
            });
        });
        return;
    }

    let metrics = compute_tileset_grid(
        [img_w, img_h],
        tileset.tile_size,
        tileset.spacing,
        tileset.padding,
    );
    let grid = TileGrid::new(
        tileset.tile_size.x,
        tileset.tile_size.y,
        tileset.spacing,
        tileset.padding,
        metrics.cols,
        metrics.rows,
    );

    let page_bg = if let Some(ref bg) = app.tileset_view_state.page_bg {
        Some(bg.clone())
    } else {
        image::load_from_memory(PAGE_BG_BYTES).ok().map(|img| {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &rgba.into_raw(),
            );
            let handle = ui.ctx().load_texture(
                "tileset_atlas_page_bg",
                color_image,
                egui::TextureOptions::NEAREST_REPEAT,
            );
            app.tileset_view_state.page_bg = Some(handle.clone());
            handle
        })
    };

    let available_rect = ui.available_rect_before_wrap();
    let fit_key = ui.input(|i| i.key_pressed(egui::Key::F) || i.key_pressed(egui::Key::Home));
    let needs_fit = app
        .tileset_view_state
        .canvas_state_mut(tileset.id)
        .needs_initial_fit
        || fit_key;

    if needs_fit && img_w > 0 && img_h > 0 {
        let (fit_zoom, fit_offset) = compute_atlas_fit(available_rect, img_w as f32, img_h as f32);
        let cs = app.tileset_view_state.canvas_state_mut(tileset.id);
        cs.view.zoom = fit_zoom;
        cs.view.offset = fit_offset;
        cs.needs_initial_fit = false;
    }

    let enum_def_opt = tileset
        .marking
        .source_enum
        .and_then(|id| app.project.defs.enum_def(id))
        .cloned();

    let active_val_id = app.tileset_view_state.selected_marking_value.clone();
    let is_marking_mode = app.tileset_view_state.active_mode == "Marking";
    let ctx_clone = ui.ctx().clone();
    let assets = app.assets.clone();
    let project = app.project.clone();

    let overlay_cx = TileOverlayContext {
        tileset,
        enum_def: enum_def_opt.as_ref(),
        is_marking_mode,
        selected_val_id: active_val_id.as_deref(),
        project: &project,
        assets: assets.as_ref(),
    };

    let mut overlay = |painter: &egui::Painter, (col, row): (u32, u32), tile_rect: Rect| {
        let tile_id = row * grid.cols + col;
        render_tile_overlay(painter, &ctx_clone, tile_rect, tile_id, &overlay_cx);
    };

    let (response, events) = {
        let canvas_state = app.tileset_view_state.canvas_state_mut(tileset.id);
        render_atlas_canvas(
            ui,
            t,
            canvas_state,
            tex,
            page_bg.as_ref(),
            &grid,
            vec2(img_w as f32, img_h as f32),
            Some(&mut overlay as &mut TileOverlayCallback<'_>),
            None,
        )
    };

    if (response.double_clicked() || fit_key) && img_w > 0 && img_h > 0 {
        let (fit_zoom, fit_offset) = compute_atlas_fit(available_rect, img_w as f32, img_h as f32);
        let cs = app.tileset_view_state.canvas_state_mut(tileset.id);
        cs.view.zoom = fit_zoom;
        cs.view.offset = fit_offset;
    }

    // Hover tooltip: "Tile N" + tag names / custom data
    if let Some(hovered_id) = app
        .tileset_view_state
        .canvas_state_mut(tileset.id)
        .hovered_tile
    {
        let mut tooltip_lines = vec![format!("Tile {hovered_id}")];
        let tags = tileset.marking.tags_at(hovered_id);
        if !tags.is_empty() {
            if let Some(ref enum_def) = enum_def_opt {
                let tag_names: Vec<&str> = tags
                    .iter()
                    .filter_map(|tag| enum_def.value(tag).map(|v| v.name.as_str()))
                    .collect();
                if !tag_names.is_empty() {
                    tooltip_lines.push(tag_names.join(" + "));
                }
            }
        }
        if let Some(custom_data) = tileset.custom_data.get(&hovered_id) {
            if !custom_data.trim().is_empty() {
                tooltip_lines.push(format!("Data: {custom_data}"));
            }
        }
        if tooltip_lines.len() == 1 && grid.cols > 0 {
            let col = hovered_id % grid.cols;
            let row = hovered_id / grid.cols;
            tooltip_lines[0] = format!("Tile {hovered_id} ({col}, {row})");
        }
        response.on_hover_text(tooltip_lines.join("\n"));
    }

    // Process events (clicks never mutate project data directly, only via Commands)
    let mut toggle_marking: Option<(String, u32, bool)> = None;

    for event in events {
        match event {
            AtlasEvent::Clicked {
                cell,
                tile_id,
                button,
                ..
            } => {
                app.tileset_view_state.selected_tile = Some(tile_id);
                app.tileset_view_state
                    .canvas_state_mut(tileset.id)
                    .selected_tile_rect = Some(TileRect::single(cell.0, cell.1));

                if tileset.marking.source_enum.is_none() {
                    // When no enum for marking is selected, clicking opens Custom Data dialog
                    if button == PointerButton::Primary {
                        let existing = tileset.custom_data.get(&tile_id).map(|s| s.as_str());
                        app.tileset_view_state
                            .custom_data_dialog
                            .open(tileset.id, tile_id, existing);
                    }
                } else if app.tileset_view_state.active_mode == "Marking" {
                    if let Some(ref val_id) = app.tileset_view_state.selected_marking_value {
                        if val_id == CUSTOM_DATA_VALUE_ID {
                            if button == PointerButton::Primary {
                                let existing =
                                    tileset.custom_data.get(&tile_id).map(|s| s.as_str());
                                app.tileset_view_state
                                    .custom_data_dialog
                                    .open(tileset.id, tile_id, existing);
                            } else if button == PointerButton::Secondary
                                && tileset.custom_data.contains_key(&tile_id)
                            {
                                app.execute_command(Box::new(SetTileCustomDataCommand::new(
                                    tileset.id,
                                    tile_id,
                                    None::<String>,
                                )));
                            }
                        } else if button == PointerButton::Primary {
                            // Left click marks
                            if !tileset.marking.has_tag(val_id, tile_id) {
                                toggle_marking = Some((val_id.clone(), tile_id, true));
                            }
                        } else if button == PointerButton::Secondary {
                            // Right click unmarks
                            if tileset.marking.has_tag(val_id, tile_id) {
                                toggle_marking = Some((val_id.clone(), tile_id, false));
                            }
                        }
                    }
                }
            }
            AtlasEvent::DragRectEnd(rect) => {
                app.tileset_view_state.selected_tile = Some(rect.top_left_id(&grid));
            }
            _ => {}
        }
    }

    if let Some((val_id, tile_id, active)) = toggle_marking {
        app.execute_command(Box::new(SetTileMarkingCommand::new(
            tileset.id, val_id, tile_id, active,
        )));
    }

    // Render custom data modal dialog if open for this tileset
    if app.tileset_view_state.custom_data_dialog.is_open
        && app.tileset_view_state.custom_data_dialog.tileset_id == Some(tileset.id)
    {
        let tile_id = app.tileset_view_state.custom_data_dialog.tile_id;
        let outcome = render_custom_data_dialog(
            ui.ctx(),
            t,
            &mut app.tileset_view_state.custom_data_dialog,
            None::<fn(&mut Ui)>,
        );

        if let Some(outcome) = outcome {
            match outcome {
                CustomDataOutcome::Save(content) => {
                    let data = if content.trim().is_empty() {
                        None
                    } else {
                        Some(content)
                    };
                    app.execute_command(Box::new(SetTileCustomDataCommand::new(
                        tileset.id, tile_id, data,
                    )));
                }
                CustomDataOutcome::Delete => {
                    app.execute_command(Box::new(SetTileCustomDataCommand::new(
                        tileset.id,
                        tile_id,
                        None::<String>,
                    )));
                }
                CustomDataOutcome::Cancel => {}
            }
        }
    }

    // Synchronize pan and zoom settings in view state
    let (zoom, offset) = {
        let cs = app.tileset_view_state.canvas_state_mut(tileset.id);
        (cs.view.zoom, cs.view.offset)
    };
    let pz = app.tileset_view_state.pan_zoom_mut(tileset.id);
    pz.zoom = zoom;
    pz.pan = offset;
}
