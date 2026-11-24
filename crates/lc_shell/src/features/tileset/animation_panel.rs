//! Right panel content for managing tile animations (anchors and frames).

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{TileAnimationDef, TilesetDef};
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_core::Command;
use lc_feature_tileset::{
    AddAnimationFrameCommand, RemoveAnimationFrameCommand, RemoveTileAnimationCommand,
    SetTileAnimationCommand,
};
use lc_ui_api::AssetService;

use super::right_panel::paint_tile_thumbnail;
use super::state::TilesetViewState;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};

/// Renders the complete animation tool attributes panel (anchors list + frames list).
pub fn render_animation_panel(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    // Ensure an anchor is selected if animations exist
    if state.selected_animation_anchor.is_none() {
        state.selected_animation_anchor = tileset.animations.keys().next().copied();
    } else if let Some(sel) = state.selected_animation_anchor {
        if !tileset.animations.contains_key(&sel) {
            state.selected_animation_anchor = tileset.animations.keys().next().copied();
        }
    }

    let total_h = ui.available_height();
    let divider_h = 10.0;
    let section_h = ((total_h - divider_h) * 0.5).max(110.0);

    Frame::new()
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            let inner_w = ui.available_width();
            let inner_section_h = section_h - 8.0;

            // 1. Top Section: Anchors List
            ui.allocate_ui(vec2(inner_w, inner_section_h), |ui| {
                ui.set_min_size(vec2(inner_w, inner_section_h));
                render_anchors_section(ui, t, tileset, project, assets, state, commands);
            });

            ui.add_space(4.0);
            ui.painter().hline(
                ui.available_rect_before_wrap().x_range(),
                ui.cursor().min.y,
                Stroke::new(1.0, t.hairline().color),
            );
            ui.add_space(4.0);

            // 2. Bottom Section: Frames List for the Selected Anchor
            ui.allocate_ui(vec2(inner_w, inner_section_h), |ui| {
                ui.set_min_size(vec2(inner_w, inner_section_h));
                render_frames_section(ui, t, tileset, project, assets, state, commands);
            });
        });
}

/// Renders the top anchors section.
fn render_anchors_section(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    let section_h = ui.available_height();
    // Header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("ANCHORS")
                .size(11.0)
                .strong()
                .color(t.ink_strong),
        );
        ui.label(
            RichText::new(format!("({})", tileset.animations.len()))
                .size(10.5)
                .color(t.muted),
        );

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Icon::Add, 20.0, IconButtonStyle::bordered().square())
                .on_hover_text("Create new animation from selected tile")
                .clicked()
            {
                let candidate_id = state
                    .selected_tile
                    .or_else(|| (0..256).find(|id| !tileset.animations.contains_key(id)))
                    .unwrap_or(0);

                let anim = TileAnimationDef::new(candidate_id);
                commands.push(Box::new(SetTileAnimationCommand::new(tileset.id, anim)));
                state.selected_animation_anchor = Some(candidate_id);
            }
        });
    });

    ui.add_space(4.0);

    if tileset.animations.is_empty() {
        let empty_h = (section_h - 32.0).max(60.0);
        ui.allocate_ui(vec2(ui.available_width(), empty_h), |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No animation anchors yet.")
                            .size(11.0)
                            .color(t.muted),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Select a tile and click '+' above.")
                            .size(10.5)
                            .color(t.faint),
                    );
                });
            });
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("tileset_anim_anchors_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (&anchor_id, anim) in &tileset.animations {
                let is_selected = state.selected_animation_anchor == Some(anchor_id);
                let row_h = 30.0;
                let (row_rect, row_resp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());

                if row_resp.clicked() {
                    state.selected_animation_anchor = Some(anchor_id);
                }

                // Row fill & border
                let fill = if is_selected {
                    Color32::from_rgb(34, 40, 50)
                } else if row_resp.hovered() {
                    Color32::from_rgb(28, 32, 40)
                } else {
                    Color32::from_rgb(20, 24, 30)
                };
                ui.painter().rect_filled(row_rect, CornerRadius::ZERO, fill);

                let stroke = if is_selected {
                    Stroke::new(2.0, t.accent)
                } else {
                    Stroke::new(1.0, Color32::from_rgb(45, 52, 62))
                };
                ui.painter()
                    .rect_stroke(row_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

                // Thumbnail box
                let thumb_size = 22.0;
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(row_rect.min.x + 4.0, row_rect.center().y - thumb_size / 2.0),
                    vec2(thumb_size, thumb_size),
                );
                draw_thumb(
                    ui.painter(),
                    ui.ctx(),
                    thumb_rect,
                    project,
                    assets,
                    tileset.id,
                    anchor_id,
                );

                // Anchor ID & frame count
                let text_pos = Pos2::new(thumb_rect.max.x + 6.0, row_rect.center().y);
                ui.painter().text(
                    text_pos,
                    Align2::LEFT_CENTER,
                    format!("#{anchor_id}"),
                    FontId::monospace(11.0),
                    Color32::WHITE,
                );

                // Delete button
                let del_size = 20.0;
                let del_rect = Rect::from_min_size(
                    Pos2::new(
                        row_rect.max.x - del_size - 6.0,
                        row_rect.center().y - del_size / 2.0,
                    ),
                    vec2(del_size, del_size),
                );
                let mut del_child = ui.new_child(egui::UiBuilder::new().max_rect(del_rect));
                if icon_button(
                    &mut del_child,
                    Icon::Delete,
                    del_size,
                    IconButtonStyle::ghost_danger(),
                )
                .on_hover_text("Delete animation")
                .clicked()
                {
                    commands.push(Box::new(RemoveTileAnimationCommand::new(
                        tileset.id, anchor_id,
                    )));
                    if state.selected_animation_anchor == Some(anchor_id) {
                        state.selected_animation_anchor = None;
                    }
                }

                let frames_info = format!("{} frames", anim.frames.len());
                let frames_pos = Pos2::new(del_rect.min.x - 8.0, row_rect.center().y);
                ui.painter().text(
                    frames_pos,
                    Align2::RIGHT_CENTER,
                    frames_info,
                    FontId::proportional(10.5),
                    t.muted,
                );

                ui.add_space(2.0);
            }
        });
}

/// Renders the bottom frames section for the currently selected anchor.
fn render_frames_section(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    let section_h = ui.available_height();
    let Some(anchor_id) = state.selected_animation_anchor else {
        let empty_h = (section_h - 32.0).max(60.0);
        ui.allocate_ui(vec2(ui.available_width(), empty_h), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select an anchor above.")
                        .size(11.0)
                        .color(t.muted),
                );
            });
        });
        return;
    };

    let Some(anim) = tileset.animations.get(&anchor_id) else {
        return;
    };

    // Header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("FRAMES (#{})", anchor_id))
                .size(11.0)
                .strong()
                .color(t.ink_strong),
        );
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Icon::Add, 20.0, IconButtonStyle::bordered().square())
                .on_hover_text("Add selected tile as frame")
                .clicked()
            {
                let frame_tile = state.selected_tile.unwrap_or(anchor_id);
                commands.push(Box::new(AddAnimationFrameCommand::new(
                    tileset.id, anchor_id, frame_tile,
                )));
            }
        });
    });

    ui.add_space(4.0);

    if anim.frames.is_empty() {
        let empty_h = (section_h - 32.0).max(60.0);
        ui.allocate_ui(vec2(ui.available_width(), empty_h), |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No frames in animation.")
                            .size(11.0)
                            .color(t.muted),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Click tiles on atlas to add frames.")
                            .size(10.5)
                            .color(t.faint),
                    );
                });
            });
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("tileset_anim_frames_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (idx, &frame_tile_id) in anim.frames.iter().enumerate() {
                let row_h = 28.0;
                let (row_rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());

                let bg_fill = if idx % 2 == 0 {
                    Color32::from_rgb(22, 26, 32)
                } else {
                    Color32::from_rgb(18, 22, 28)
                };
                ui.painter()
                    .rect_filled(row_rect, CornerRadius::ZERO, bg_fill);
                ui.painter().rect_stroke(
                    row_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::from_rgb(38, 44, 52)),
                    StrokeKind::Inside,
                );

                // Frame badge label: F1, F2...
                let label_pos = Pos2::new(row_rect.min.x + 6.0, row_rect.center().y);
                ui.painter().text(
                    label_pos,
                    Align2::LEFT_CENTER,
                    format!("F{}", idx + 1),
                    FontId::monospace(11.0),
                    Color32::from_rgb(255, 190, 80),
                );

                // Thumbnail box
                let thumb_size = 20.0;
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(
                        row_rect.min.x + 32.0,
                        row_rect.center().y - thumb_size / 2.0,
                    ),
                    vec2(thumb_size, thumb_size),
                );
                draw_thumb(
                    ui.painter(),
                    ui.ctx(),
                    thumb_rect,
                    project,
                    assets,
                    tileset.id,
                    frame_tile_id,
                );

                // Tile ID text
                let id_pos = Pos2::new(thumb_rect.max.x + 6.0, row_rect.center().y);
                ui.painter().text(
                    id_pos,
                    Align2::LEFT_CENTER,
                    format!("#{frame_tile_id}"),
                    FontId::monospace(10.5),
                    Color32::WHITE,
                );

                // Actions: Reorder up, reorder down, delete
                let btn_size = 18.0;
                let right_edge = row_rect.max.x - 6.0;

                // Delete frame button
                let del_rect = Rect::from_min_size(
                    Pos2::new(right_edge - btn_size, row_rect.center().y - btn_size / 2.0),
                    vec2(btn_size, btn_size),
                );
                let mut del_child = ui.new_child(egui::UiBuilder::new().max_rect(del_rect));
                if icon_button(
                    &mut del_child,
                    Icon::Delete,
                    btn_size,
                    IconButtonStyle::ghost_danger(),
                )
                .on_hover_text("Remove frame")
                .clicked()
                {
                    commands.push(Box::new(RemoveAnimationFrameCommand::new(
                        tileset.id, anchor_id, idx,
                    )));
                }

                // Move Down button
                if idx + 1 < anim.frames.len() {
                    let dn_rect = Rect::from_min_size(
                        Pos2::new(
                            del_rect.min.x - btn_size - 2.0,
                            row_rect.center().y - btn_size / 2.0,
                        ),
                        vec2(btn_size, btn_size),
                    );
                    let mut dn_child = ui.new_child(egui::UiBuilder::new().max_rect(dn_rect));
                    if icon_button(
                        &mut dn_child,
                        Icon::ChevronDown,
                        btn_size,
                        IconButtonStyle::ghost(),
                    )
                    .on_hover_text("Move frame down")
                    .clicked()
                    {
                        let mut new_anim = anim.clone();
                        new_anim.frames.swap(idx, idx + 1);
                        commands.push(Box::new(SetTileAnimationCommand::new(tileset.id, new_anim)));
                    }
                }

                // Move Up button
                if idx > 0 {
                    let up_rect = Rect::from_min_size(
                        Pos2::new(
                            del_rect.min.x - (btn_size * 2.0) - 4.0,
                            row_rect.center().y - btn_size / 2.0,
                        ),
                        vec2(btn_size, btn_size),
                    );
                    let mut up_child = ui.new_child(egui::UiBuilder::new().max_rect(up_rect));
                    if icon_button(
                        &mut up_child,
                        Icon::ChevronUp,
                        btn_size,
                        IconButtonStyle::ghost(),
                    )
                    .on_hover_text("Move frame up")
                    .clicked()
                    {
                        let mut new_anim = anim.clone();
                        new_anim.frames.swap(idx, idx - 1);
                        commands.push(Box::new(SetTileAnimationCommand::new(tileset.id, new_anim)));
                    }
                }

                ui.add_space(2.0);
            }
        });
}

fn draw_thumb(
    painter: &egui::Painter,
    ctx: &egui::Context,
    rect: Rect,
    project: &Project,
    assets: &dyn AssetService,
    tileset_id: lc_core::id::TilesetDefId,
    tile_id: u32,
) {
    painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(14, 16, 20));
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(60, 68, 80)),
        StrokeKind::Inside,
    );
    let tr = TileRef {
        tileset_id,
        tile_id,
        flip_x: false,
        flip_y: false,
    };
    paint_tile_thumbnail(painter, ctx, rect, project, assets, &tr);
}
