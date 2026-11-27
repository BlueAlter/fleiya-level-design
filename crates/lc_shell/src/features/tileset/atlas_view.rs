//! Atlas interactive viewport and tile grid view.

use egui::{vec2, CornerRadius, PointerButton, Rect, RichText, Sense, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::{TileAnimationDef, TilesetDef};
use lc_feature_tileset::{
    compute_tileset_grid, AddAnimationFrameCommand, RemoveAnimationFrameCommand,
    RemoveTileAnimationCommand, SetTileAnimationCommand, SetTileCollisionCommand,
    SetTileCustomDataCommand, SetTileMarkingCommand,
};

use super::animation_preview::render_animation_preview_card;
use super::atlas_overlay::{render_tile_hover_tooltip, render_tile_overlay, TileOverlayContext};
use super::state::CUSTOM_DATA_VALUE_ID;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::atlas_canvas::{render_atlas_canvas, AtlasEvent, TileOverlayCallback};
use crate::shared::ui::custom_data_dialog::{render_custom_data_dialog, CustomDataOutcome};
use crate::shared::ui::tile_bounds::{TileGrid, TileRect};

/// Fit rule from workspace spec:
/// zoom = min((avail_w - 32) / img_w, (avail_h - 32) / img_h, 8.0). Clamped to min 0.1, max 8.0.
/// pan = center of available rect - (img_size * zoom) / 2.
pub fn compute_atlas_fit(available_rect: egui::Rect, img_w: f32, img_h: f32) -> (f32, egui::Vec2) {
    if img_w <= 0.0 || img_h <= 0.0 {
        return (1.0, egui::Vec2::ZERO);
    }
    let avail_w = (available_rect.width() - 32.0).max(1.0);
    let avail_h = (available_rect.height() - 32.0).max(1.0);
    let zoom = ((avail_w / img_w).min(avail_h / img_h)).clamp(0.1, 8.0);
    let pan = available_rect.center() - vec2(img_w * zoom, img_h * zoom) / 2.0;
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
        let msg = if tileset.path.is_empty() {
            "No image loaded".to_string()
        } else {
            format!(
                "Warning: Source image file could not be loaded: {}",
                tileset.path
            )
        };
        render_empty_placeholder(ui, t, &msg);
        return;
    };

    let sz = tex.size();
    let (img_w, img_h) = (sz[0] as u32, sz[1] as u32);
    if img_w == 0 || img_h == 0 {
        render_empty_placeholder(ui, t, "Warning: Image has zero dimensions");
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

    let page_bg = app.tileset_view_state.ensure_page_bg(ui.ctx());

    let is_frozen = app.tileset_view_state.custom_data_dialog.is_open
        || app.tileset_view_state.is_enum_dropdown_open;

    let available_rect = ui.available_rect_before_wrap();
    let atlas_focused =
        app.tileset_view_state.focused_card == Some(super::state::TilesetCard::Atlas);
    let fit_key = !is_frozen
        && atlas_focused
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|i| i.key_pressed(egui::Key::F) || i.key_pressed(egui::Key::Home));
    let needs_fit = !is_frozen
        && (app
            .tileset_view_state
            .canvas_state_mut(tileset.id)
            .needs_initial_fit
            || fit_key);

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
    let is_animation_mode = app.tileset_view_state.active_mode == "Animation";
    let selected_anim_anchor = app.tileset_view_state.selected_animation_anchor;
    let is_anim_playing = app.tileset_view_state.is_animation_playing;
    let anim_start_time = app.tileset_view_state.animation_start_time;
    let ctx_clone = ui.ctx().clone();
    let assets = app.assets.clone();
    let project = app.project.clone();

    let overlay_cx = TileOverlayContext {
        tileset,
        enum_def: enum_def_opt.as_ref(),
        is_marking_mode,
        is_animation_mode,
        selected_val_id: active_val_id.as_deref(),
        selected_animation_anchor: selected_anim_anchor,
        is_animation_playing: is_anim_playing,
        animation_start_time: anim_start_time,
        project: &project,
        assets: assets.as_ref(),
    };

    let col_cx =
        super::collision_overlay::CollisionOverlayContext::from_state(&app.tileset_view_state);

    let focus_tile = app.tileset_view_state.collision_focus_tile;
    let in_collision = app.tileset_view_state.active_mode == "Collision";
    let dim_fn = move |(col, row): (u32, u32)| -> bool {
        if in_collision {
            if let Some(focus_id) = focus_tile {
                let tid = row * grid.cols + col;
                return tid != focus_id;
            }
        }
        false
    };
    let dim_pred: Option<&crate::shared::ui::atlas_canvas::TileDimPredicate<'_>> =
        if in_collision && focus_tile.is_some() {
            Some(&dim_fn)
        } else {
            None
        };

    let mut overlay = |painter: &egui::Painter, (col, row): (u32, u32), tile_rect: Rect| {
        let tile_id = row * grid.cols + col;
        render_tile_overlay(painter, &ctx_clone, tile_rect, tile_id, &overlay_cx);
        super::collision_overlay::render_tile_collision_overlay(
            painter, tile_rect, tile_id, tileset, &col_cx, t,
        );
    };

    let (response, events) = {
        let cs = app.tileset_view_state.canvas_state_mut(tileset.id);
        let mut render_fn = |ui: &mut Ui| {
            render_atlas_canvas(
                ui,
                t,
                cs,
                tex,
                page_bg.as_ref(),
                &grid,
                vec2(img_w as f32, img_h as f32),
                Some(&mut overlay as &mut TileOverlayCallback<'_>),
                dim_pred,
            )
        };
        if is_frozen {
            ui.add_enabled_ui(false, |ui| render_fn(ui)).inner
        } else {
            render_fn(ui)
        }
    };

    render_animation_preview_card(
        ui,
        t,
        response.rect,
        tileset,
        &app.project,
        app.assets.as_ref(),
        &app.tileset_view_state,
    );

    if !is_frozen && app.tileset_view_state.active_mode == "Collision" {
        let canvas_view = app.tileset_view_state.canvas_state_mut(tileset.id).view;
        let cmds = super::collision_interaction::handle_collision_focus_interaction(
            ui,
            &response,
            tileset,
            &grid,
            canvas_view,
            &mut app.tileset_view_state,
        );
        for cmd in cmds {
            app.execute_command(cmd);
        }
    }

    let is_collision_isolated = app.tileset_view_state.active_mode == "Collision"
        && app.tileset_view_state.collision_focus_tile.is_some();

    if !is_frozen
        && !is_collision_isolated
        && (response.double_clicked() || fit_key)
        && img_w > 0
        && img_h > 0
    {
        let (fit_zoom, fit_offset) = compute_atlas_fit(available_rect, img_w as f32, img_h as f32);
        let cs = app.tileset_view_state.canvas_state_mut(tileset.id);
        cs.view.zoom = fit_zoom;
        cs.view.offset = fit_offset;
    }

    // Hover tooltip: positioned at cursor, width-bounded and height-bounded with scrolling
    if !is_frozen && !is_collision_isolated {
        if let Some(hovered_id) = app
            .tileset_view_state
            .canvas_state_mut(tileset.id)
            .hovered_tile
        {
            let tags = tileset.marking.tags_at(hovered_id);
            let tag_names: Vec<String> = enum_def_opt.as_ref().map_or_else(Vec::new, |ed| {
                tags.iter()
                    .filter_map(|tag| ed.value(tag).map(|v| v.name.clone()))
                    .collect()
            });
            let custom_data_text = tileset
                .custom_data
                .get(&hovered_id)
                .filter(|s| !s.trim().is_empty())
                .cloned();

            let title = if grid.cols > 0 {
                format!(
                    "Tile {hovered_id} ({}, {})",
                    hovered_id % grid.cols,
                    hovered_id / grid.cols
                )
            } else {
                format!("Tile {hovered_id}")
            };

            render_tile_hover_tooltip(
                &response,
                t,
                &title,
                &tag_names,
                custom_data_text.as_deref(),
            );
        }
    }

    // Process events (suppressed while dialog or dropdown is open)
    let mut toggle_marking: Option<(String, u32, bool)> = None;

    if !is_frozen {
        for event in events {
            match event {
                AtlasEvent::Clicked {
                    cell,
                    tile_id,
                    button,
                    modifiers,
                } => {
                    if is_collision_isolated {
                        continue;
                    }
                    app.tileset_view_state.selected_tile = Some(tile_id);
                    app.tileset_view_state
                        .canvas_state_mut(tileset.id)
                        .selected_tile_rect = Some(TileRect::single(cell.0, cell.1));

                    let is_cd_target = app.tileset_view_state.active_mode == "None"
                        || (app.tileset_view_state.active_mode == "Marking"
                            && app.tileset_view_state.selected_marking_value.as_deref()
                                == Some(CUSTOM_DATA_VALUE_ID));

                    if is_cd_target {
                        if button == PointerButton::Primary {
                            let existing = tileset.custom_data.get(&tile_id).map(|s| s.as_str());
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
                    } else {
                        match app.tileset_view_state.active_mode.as_str() {
                            "Marking" => {
                                if let Some(ref val_id) =
                                    app.tileset_view_state.selected_marking_value
                                {
                                    let has = tileset.marking.has_tag(val_id, tile_id);
                                    if button == PointerButton::Primary && !has {
                                        toggle_marking = Some((val_id.clone(), tile_id, true));
                                    } else if button == PointerButton::Secondary && has {
                                        toggle_marking = Some((val_id.clone(), tile_id, false));
                                    }
                                }
                            }
                            "Animation" => {
                                let is_ctrl = modifiers.command || modifiers.ctrl;
                                if is_ctrl {
                                    if button == PointerButton::Primary {
                                        if !tileset.animations.contains_key(&tile_id) {
                                            app.execute_command(Box::new(
                                                SetTileAnimationCommand::new(
                                                    tileset.id,
                                                    TileAnimationDef::new(tile_id),
                                                ),
                                            ));
                                        }
                                        app.tileset_view_state.selected_animation_anchor =
                                            Some(tile_id);
                                    } else if button == PointerButton::Secondary
                                        && tileset.animations.contains_key(&tile_id)
                                    {
                                        app.execute_command(Box::new(
                                            RemoveTileAnimationCommand::new(tileset.id, tile_id),
                                        ));
                                        if app.tileset_view_state.selected_animation_anchor
                                            == Some(tile_id)
                                        {
                                            app.tileset_view_state.selected_animation_anchor =
                                                tileset
                                                    .animations
                                                    .keys()
                                                    .copied()
                                                    .find(|&id| id != tile_id);
                                        }
                                    }
                                } else if button == PointerButton::Primary {
                                    // Plain click only adds frames when an anchor is already selected;
                                    // creating a new anchor requires Ctrl+Click or the '+' button.
                                    if let Some(anchor_id) =
                                        app.tileset_view_state.selected_animation_anchor
                                    {
                                        app.execute_command(Box::new(
                                            AddAnimationFrameCommand::new(
                                                tileset.id, anchor_id, tile_id,
                                            ),
                                        ));
                                    }
                                } else if button == PointerButton::Secondary {
                                    if let Some(anchor_id) =
                                        app.tileset_view_state.selected_animation_anchor
                                    {
                                        if let Some(anim) = tileset.animations.get(&anchor_id) {
                                            if let Some(pos) =
                                                anim.frames.iter().rposition(|&f| f == tile_id)
                                            {
                                                app.execute_command(Box::new(
                                                    RemoveAnimationFrameCommand::new(
                                                        tileset.id, anchor_id, pos,
                                                    ),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            "Collision" if button == PointerButton::Primary && modifiers.alt => {
                                app.tileset_view_state.collision_focus_tile = Some(tile_id);
                            }
                            _ => {}
                        }
                    }
                }
                AtlasEvent::DragRectEnd(rect) => {
                    if is_collision_isolated {
                        continue;
                    }
                    let btn = app
                        .tileset_view_state
                        .canvas_state_mut(tileset.id)
                        .last_drag_button
                        .unwrap_or(PointerButton::Primary);
                    if app.tileset_view_state.active_mode == "Animation"
                        && (rect.w > 1 || rect.h > 1)
                    {
                        if let Some(aid) = app.tileset_view_state.selected_animation_anchor {
                            for (c, r) in rect.cells() {
                                let tid = grid.tile_id(c, r);
                                if btn == PointerButton::Primary {
                                    app.execute_command(Box::new(AddAnimationFrameCommand::new(
                                        tileset.id, aid, tid,
                                    )));
                                } else if btn == PointerButton::Secondary {
                                    if let Some(anim) = tileset.animations.get(&aid) {
                                        if let Some(pos) =
                                            anim.frames.iter().rposition(|&f| f == tid)
                                        {
                                            app.execute_command(Box::new(
                                                RemoveAnimationFrameCommand::new(
                                                    tileset.id, aid, pos,
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        app.tileset_view_state.selected_tile = Some(rect.top_left_id(&grid));
                        app.tileset_view_state
                            .canvas_state_mut(tileset.id)
                            .selected_tile_rect = Some(rect);
                    }
                }
                _ => {}
            }
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
        let shape_id_opt = app.tileset_view_state.custom_data_dialog.shape_id;
        if let Some(outcome) = render_custom_data_dialog(
            ui.ctx(),
            t,
            &mut app.tileset_view_state.custom_data_dialog,
            None::<fn(&mut Ui)>,
        ) {
            let data = match outcome {
                CustomDataOutcome::Save(c) if !c.trim().is_empty() => Some(c),
                CustomDataOutcome::Cancel => return,
                _ => None,
            };

            if let Some(shape_id) = shape_id_opt {
                if let Some(mut col) = tileset.collision(tile_id).cloned() {
                    if let Some(shape) = col.shapes.iter_mut().find(|s| s.id == shape_id) {
                        shape.custom_data = data.unwrap_or_default();
                        app.execute_command(Box::new(SetTileCollisionCommand::new(
                            tileset.id, tile_id, col,
                        )));
                    }
                }
            } else {
                app.execute_command(Box::new(SetTileCustomDataCommand::new(
                    tileset.id, tile_id, data,
                )));
            }
        }
    }

    let view = app.tileset_view_state.canvas_state_mut(tileset.id).view;
    let pz = app.tileset_view_state.pan_zoom_mut(tileset.id);
    pz.zoom = view.zoom;
    pz.pan = view.offset;
}

fn render_empty_placeholder(ui: &mut Ui, t: &Tokens, message: &str) {
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(message)
                    .font(egui::FontId::proportional(13.0))
                    .color(t.muted),
            );
        });
    });
}
