//! Right panel content for managing tile collision shapes, presets, and tags.

use egui::{
    vec2, Align, Color32, CornerRadius, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::collision::{CollisionShape, CollisionShapeGeometry, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_core::Command;
use lc_feature_tileset::{RemoveTileCollisionCommand, SetTileCollisionCommand};
use lc_ui_api::AssetService;

use super::right_panel::paint_tile_thumbnail;
use super::state::TilesetViewState;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};

/// Renders the complete collision tool attributes panel in the right panel shell.
pub fn render_collision_panel(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    let active_tile_id = state
        .collision_focus_tile
        .or(state
            .canvas_states
            .get(&tileset.id)
            .and_then(|cs| cs.hovered_tile))
        .or(state.selected_tile);

    ui.add_space(4.0);

    // 1. Header with active tile ID badge and thumbnail
    render_header_card(ui, t, tileset, project, assets, state, active_tile_id);

    ui.add_space(8.0);

    let Some(tile_id) = active_tile_id else {
        render_no_tile_selected(ui, t);
        return;
    };

    let col_def = tileset.collision(tile_id).cloned().unwrap_or_default();

    // 2. Presets row: Full Box, Slope Up, Slope Down, Clear
    render_presets_section(ui, t, tileset, tile_id, commands);

    ui.add_space(8.0);
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().min.y,
        Stroke::new(1.0, t.hairline().color),
    );
    ui.add_space(8.0);

    // 3. Shapes list section
    render_shapes_section(ui, t, tileset, tile_id, col_def, state, commands);
}

/// Renders the active tile info card at the top of the collision panel.
fn render_header_card(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    active_tile_id: Option<u32>,
) {
    let card_h = 36.0;
    let (card_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), card_h), Sense::hover());

    ui.painter()
        .rect_filled(card_rect, CornerRadius::ZERO, t.sunken);
    ui.painter().rect_stroke(
        card_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, t.line),
        StrokeKind::Inside,
    );

    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(card_rect.shrink(4.0)));
    child.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        if let Some(tid) = active_tile_id {
            // Tile thumbnail
            let thumb_size = 28.0;
            let (thumb_rect, _) =
                ui.allocate_exact_size(vec2(thumb_size, thumb_size), Sense::hover());
            ui.painter().rect_filled(
                thumb_rect,
                CornerRadius::ZERO,
                Color32::from_rgb(18, 22, 28),
            );
            ui.painter().rect_stroke(
                thumb_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, t.line),
                StrokeKind::Inside,
            );

            paint_tile_thumbnail(
                ui.painter(),
                ui.ctx(),
                thumb_rect,
                project,
                assets,
                &TileRef::new(tileset.id, tid),
            );

            // Tile label
            ui.label(
                RichText::new(format!("#{tid}"))
                    .size(12.0)
                    .strong()
                    .color(t.ink_strong),
            );

            // Isolate mode toggle / badge
            let is_isolated = state.collision_focus_tile == Some(tid);
            if is_isolated {
                let badge_btn = egui::Button::new(
                    RichText::new("ISOLATED")
                        .size(10.0)
                        .strong()
                        .color(t.accent_ink),
                )
                .fill(t.accent.gamma_multiply(0.25))
                .stroke(Stroke::new(1.0, t.accent))
                .corner_radius(CornerRadius::ZERO);

                if ui
                    .add(badge_btn)
                    .on_hover_text("Click to exit isolate mode (or press ESC)")
                    .clicked()
                {
                    state.exit_collision_focus();
                }
            } else {
                let iso_btn = egui::Button::new(RichText::new("Isolate").size(10.0).color(t.muted))
                    .corner_radius(CornerRadius::ZERO);

                if ui
                    .add(iso_btn)
                    .on_hover_text("Isolate this tile on canvas (Alt + Click)")
                    .clicked()
                {
                    state.collision_focus_tile = Some(tid);
                }
            }
        } else {
            ui.label(RichText::new("No tile selected").size(11.0).color(t.muted));
        }
    });
}

/// Renders quick preset buttons (Full Box, Slope Up, Slope Down, Clear).
fn render_presets_section(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    tile_id: u32,
    commands: &mut Vec<Box<dyn Command>>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("PRESETS:").size(10.0).color(t.muted));
    });

    ui.add_space(2.0);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

        if ui
            .button(RichText::new("Full Box").size(10.5))
            .on_hover_text("Solid box covering entire tile")
            .clicked()
        {
            let col = TileCollisionDef::full_tile(tileset.tile_size);
            push_col(commands, tileset.id, tile_id, col);
        }

        if ui
            .button(RichText::new("Slope /").size(10.5))
            .on_hover_text("Upward 45-degree slope triangle")
            .clicked()
        {
            let mut col = TileCollisionDef::new();
            col.shapes.push(CollisionShape::slope_up(
                col.next_shape_id(),
                tileset.tile_size,
            ));
            push_col(commands, tileset.id, tile_id, col);
        }

        if ui
            .button(RichText::new("Slope \\").size(10.5))
            .on_hover_text("Downward 45-degree slope triangle")
            .clicked()
        {
            let mut col = TileCollisionDef::new();
            col.shapes.push(CollisionShape::slope_down(
                col.next_shape_id(),
                tileset.tile_size,
            ));
            push_col(commands, tileset.id, tile_id, col);
        }

        if ui
            .button(
                RichText::new("Clear")
                    .size(10.5)
                    .color(Color32::from_rgb(220, 80, 80)),
            )
            .on_hover_text("Remove all collision shapes from this tile")
            .clicked()
        {
            push_rem(commands, tileset.id, tile_id);
        }
    });
}

/// Renders the shapes list with badges, tag editor, one-way flag, and custom data trigger.
fn render_shapes_section(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    tile_id: u32,
    mut col_def: TileCollisionDef,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    // Shapes section header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("SHAPES ({})", col_def.shapes.len()))
                .size(11.0)
                .strong()
                .color(t.ink_strong),
        );

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if ui
                .button(RichText::new("+ Poly").size(10.5))
                .on_hover_text("Add polygon")
                .clicked()
            {
                let next_id = col_def.next_shape_id();
                col_def
                    .shapes
                    .push(CollisionShape::slope_up(next_id, tileset.tile_size));
                state.selected_collision_shape_id = Some(next_id);
                push_col(commands, tileset.id, tile_id, col_def.clone());
            }
            if ui
                .button(RichText::new("+ Box").size(10.5))
                .on_hover_text("Add rectangle")
                .clicked()
            {
                let next_id = col_def.next_shape_id();
                let sz = tileset.tile_size;
                col_def.shapes.push(CollisionShape::new_rect(
                    next_id,
                    0.0,
                    0.0,
                    sz.x as f32,
                    sz.y as f32,
                ));
                state.selected_collision_shape_id = Some(next_id);
                push_col(commands, tileset.id, tile_id, col_def.clone());
            }
        });
    });

    ui.add_space(4.0);

    if col_def.shapes.is_empty() {
        render_empty_shapes(ui, t);
        return;
    }

    let mut shape_to_remove = None;
    let mut shape_modified = false;

    ScrollArea::vertical()
        .id_salt("tileset_collision_shapes_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for shape in &mut col_def.shapes {
                let is_selected = state.selected_collision_shape_id == Some(shape.id);
                let row_h = 56.0;
                let (row_rect, row_resp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());

                if row_resp.clicked() {
                    state.selected_collision_shape_id = Some(shape.id);
                }

                // Row background and stroke
                let bg_color = if is_selected {
                    Color32::from_rgb(32, 38, 48)
                } else if row_resp.hovered() {
                    Color32::from_rgb(28, 32, 40)
                } else {
                    Color32::from_rgb(22, 26, 32)
                };

                ui.painter()
                    .rect_filled(row_rect, CornerRadius::ZERO, bg_color);
                let stroke = if is_selected {
                    Stroke::new(1.5, t.accent)
                } else {
                    Stroke::new(1.0, t.line)
                };
                ui.painter()
                    .rect_stroke(row_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

                // Row contents inside child UI
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(row_rect.shrink(4.0)));
                child.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Shape badge
                        let (type_name, summary) = match &shape.geometry {
                            CollisionShapeGeometry::Rectangle { x, y, w, h } => {
                                ("BOX", format!("{w:.0}x{h:.0} @ {x:.0},{y:.0}"))
                            }
                            CollisionShapeGeometry::Polygon { points } => {
                                ("POLY", format!("{} pts", points.len()))
                            }
                            CollisionShapeGeometry::Circle { radius, .. } => {
                                ("CIRCLE", format!("r={radius:.0}"))
                            }
                        };

                        let badge_rect = Rect::from_min_size(ui.cursor().min, vec2(38.0, 16.0));
                        ui.painter().rect_filled(
                            badge_rect,
                            CornerRadius::ZERO,
                            Color32::from_rgb(14, 18, 22),
                        );
                        ui.painter().rect_stroke(
                            badge_rect,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, t.line),
                            StrokeKind::Inside,
                        );
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            type_name,
                            egui::FontId::monospace(9.5),
                            t.accent_ink,
                        );
                        ui.allocate_rect(badge_rect, Sense::hover());

                        ui.add_space(2.0);
                        ui.label(RichText::new(summary).size(10.0).color(t.muted));

                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            // Delete button
                            if icon_button(
                                ui,
                                Icon::Delete,
                                18.0,
                                IconButtonStyle::bordered().square(),
                            )
                            .on_hover_text("Delete this collision shape")
                            .clicked()
                            {
                                shape_to_remove = Some(shape.id);
                            }

                            // Custom Data button [{ }]
                            let has_data = !shape.custom_data.is_empty();
                            let cd_style = if has_data {
                                IconButtonStyle::bordered().selected(true).square()
                            } else {
                                IconButtonStyle::bordered().square()
                            };

                            let cd_btn = icon_button(ui, Icon::Code, 18.0, cd_style);
                            let tip = if has_data {
                                format!("Edit Custom Data: \"{}\"", shape.custom_data)
                            } else {
                                "Add Custom Data (JSON/text) to shape".to_string()
                            };
                            if cd_btn.on_hover_text(tip).clicked() {
                                state.custom_data_dialog.open_for_shape(
                                    tileset.id,
                                    tile_id,
                                    shape.id,
                                    Some(&shape.custom_data),
                                );
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // Tag and One-Way row
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tag:").size(10.0).color(t.muted));
                        let tag_edit = ui.add(
                            egui::TextEdit::singleline(&mut shape.tag)
                                .desired_width(75.0)
                                .font(egui::FontId::proportional(11.0)),
                        );
                        if tag_edit.changed() {
                            shape_modified = true;
                        }

                        let ow_resp =
                            ui.checkbox(&mut shape.one_way, RichText::new("1-Way").size(10.0));
                        if ow_resp.changed() {
                            shape_modified = true;
                        }
                    });
                });

                ui.add_space(2.0);
            }
        });

    if let Some(id_to_del) = shape_to_remove {
        col_def.shapes.retain(|s| s.id != id_to_del);
        if col_def.shapes.is_empty() {
            push_rem(commands, tileset.id, tile_id);
        } else {
            push_col(commands, tileset.id, tile_id, col_def);
        }
    } else if shape_modified {
        push_col(commands, tileset.id, tile_id, col_def);
    }
}

fn push_col(cmds: &mut Vec<Box<dyn Command>>, tid: TilesetDefId, tile: u32, def: TileCollisionDef) {
    cmds.push(Box::new(SetTileCollisionCommand::new(tid, tile, def)));
}

fn push_rem(cmds: &mut Vec<Box<dyn Command>>, tid: TilesetDefId, tile: u32) {
    cmds.push(Box::new(RemoveTileCollisionCommand::new(tid, tile)));
}

/// Renders placeholder when no tile is selected.
fn render_no_tile_selected(ui: &mut Ui, t: &Tokens) {
    render_centered_hint(
        ui,
        30.0,
        "No tile selected",
        t.ink_strong,
        "Click any tile in the atlas to inspect or edit its collision.",
        t.muted,
    );
}

/// Renders message when tile has no collision shapes.
fn render_empty_shapes(ui: &mut Ui, t: &Tokens) {
    render_centered_hint(
        ui,
        20.0,
        "No collision on this tile",
        t.faint,
        "Click a preset above or '+ Box' to add collision boundaries.",
        t.muted,
    );
}

fn render_centered_hint(
    ui: &mut Ui,
    top_pad: f32,
    title: &str,
    title_color: Color32,
    hint: &str,
    hint_color: Color32,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(top_pad);
        ui.label(RichText::new(title).size(11.5).strong().color(title_color));
        ui.add_space(4.0);
        ui.label(RichText::new(hint).size(10.5).color(hint_color));
    });
}
