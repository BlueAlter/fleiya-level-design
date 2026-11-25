//! Right panel content for managing tile collision shapes, custom properties, and tags.

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Margin, Rect, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::collision::{CollisionShapeGeometry, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_core::Command;
use lc_feature_tileset::{RemoveTileCollisionCommand, SetTileCollisionCommand};
use lc_ui_api::AssetService;

use super::custom_properties::{
    parse_custom_properties, render_fast_custom_properties, serialize_custom_properties,
};
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

    Frame::new()
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            // 1. Header with active tile ID badge and thumbnail (isolate toggle is on toolbar)
            render_header_card(ui, t, tileset, project, assets, active_tile_id);

            ui.add_space(10.0);

            let Some(tile_id) = active_tile_id else {
                render_no_tile_selected(ui, t);
                return;
            };

            let col_def = tileset.collision(tile_id).cloned().unwrap_or_default();

            // 2. Shapes list section with fast inline custom data properties
            render_shapes_section(ui, t, tileset, tile_id, col_def, state, commands);
        });
}

/// Renders the active tile info card at the top of the collision panel.
fn render_header_card(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
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
        } else {
            ui.label(RichText::new("No tile selected").size(11.0).color(t.muted));
        }
    });
}

/// Renders the shapes list with badges, tag editor, one-way flag, and fast custom properties.
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
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().min.y,
        Stroke::new(1.0, t.hairline().color),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("SHAPES ({})", col_def.shapes.len()))
                .size(11.0)
                .strong()
                .color(t.ink_strong),
        );
    });

    ui.add_space(8.0);

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

                let bg_color = if is_selected {
                    Color32::from_rgb(32, 38, 48)
                } else {
                    Color32::from_rgb(22, 26, 32)
                };
                let stroke = if is_selected {
                    Stroke::new(1.5, t.accent)
                } else {
                    Stroke::new(1.0, t.line)
                };

                let card_frame = egui::Frame::new()
                    .fill(bg_color)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(egui::Margin::symmetric(8, 6));

                let card_resp = card_frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Row 1: Header (Badge, summary, delete button)
                        ui.horizontal(|ui| {
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
                                Align2::CENTER_CENTER,
                                type_name,
                                FontId::monospace(9.5),
                                t.accent_ink,
                            );
                            ui.allocate_rect(badge_rect, Sense::hover());

                            ui.add_space(4.0);
                            ui.label(RichText::new(summary).size(10.5).color(t.muted));

                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
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
                            });
                        });

                        ui.add_space(4.0);

                        // Row 2: Tag and One-Way with generous spacing
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.label(Icon::Tag.rich(ui.ctx(), 11.0, t.muted));
                            let tag_edit = ui.add(
                                egui::TextEdit::singleline(&mut shape.tag)
                                    .hint_text("tag...")
                                    .desired_width(85.0)
                                    .font(FontId::proportional(11.0)),
                            );
                            if tag_edit.changed() {
                                shape_modified = true;
                            }

                            ui.add_space(6.0);
                            let ow_resp =
                                ui.checkbox(&mut shape.one_way, RichText::new("1-Way").size(10.5));
                            if ow_resp.changed() {
                                shape_modified = true;
                            }
                        });

                        ui.add_space(6.0);
                        ui.painter().hline(
                            ui.available_rect_before_wrap().x_range(),
                            ui.cursor().min.y,
                            Stroke::new(1.0, t.hairline().color),
                        );
                        ui.add_space(4.0);

                        // Row 3: Fast inline custom properties editor
                        let mut props = parse_custom_properties(&shape.custom_data);
                        if render_fast_custom_properties(ui, t, &mut props) {
                            shape.custom_data = serialize_custom_properties(&props);
                            shape_modified = true;
                        }
                    });
                });

                if card_resp.response.clicked() {
                    state.selected_collision_shape_id = Some(shape.id);
                }

                ui.add_space(6.0);
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

/// Renders empty state placeholder when no collision exists on active tile.
fn render_empty_shapes(ui: &mut Ui, t: &Tokens) {
    ui.add_space(16.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("No collision on this tile")
                .size(11.5)
                .color(t.muted),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Use the toolbar presets or drawing tools to add collision boundaries.")
                .size(10.0)
                .color(t.muted.gamma_multiply(0.7)),
        );
    });
}

/// Renders empty state when no tile is hovered or selected.
fn render_no_tile_selected(ui: &mut Ui, t: &Tokens) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Select a tile in the atlas")
                .size(11.5)
                .color(t.muted),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Click on any tile in the atlas to edit its collision boundaries.")
                .size(10.0)
                .color(t.muted.gamma_multiply(0.7)),
        );
    });
}

fn push_col(
    commands: &mut Vec<Box<dyn Command>>,
    tileset_id: TilesetDefId,
    tile_id: u32,
    col: TileCollisionDef,
) {
    commands.push(Box::new(SetTileCollisionCommand::new(
        tileset_id, tile_id, col,
    )));
}

fn push_rem(commands: &mut Vec<Box<dyn Command>>, tileset_id: TilesetDefId, tile_id: u32) {
    commands.push(Box::new(RemoveTileCollisionCommand::new(
        tileset_id, tile_id,
    )));
}
