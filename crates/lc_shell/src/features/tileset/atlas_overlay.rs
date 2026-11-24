//! Tile marking and custom data overlay rendering on the atlas canvas.

use egui::{vec2, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind};
use lc_core::defs::{EnumDef, TilesetDef};
use lc_core::project::Project;
use lc_ui_api::AssetService;

use super::right_panel::paint_tile_thumbnail;
use super::state::CUSTOM_DATA_VALUE_ID;

/// Contextual parameters required for rendering tile marking overlays.
pub struct TileOverlayContext<'a> {
    pub tileset: &'a TilesetDef,
    pub enum_def: Option<&'a EnumDef>,
    pub is_marking_mode: bool,
    pub selected_val_id: Option<&'a str>,
    pub project: &'a Project,
    pub assets: &'a dyn AssetService,
}

/// Renders the marking overlay, concentric tag borders, and custom data indicator on a tile.
pub fn render_tile_overlay(
    painter: &egui::Painter,
    ctx: &egui::Context,
    tile_rect: Rect,
    tile_id: u32,
    cx: &TileOverlayContext<'_>,
) {
    let tags = cx.tileset.marking.tags_at(tile_id);
    let custom_data_opt = cx.tileset.custom_data.get(&tile_id);
    let has_custom_data = custom_data_opt.is_some_and(|s| !s.trim().is_empty());

    if !cx.is_marking_mode {
        if !tags.is_empty() {
            draw_all_markers(painter, ctx, tile_rect, &tags, cx.enum_def, cx);
        }
        if has_custom_data {
            draw_custom_data_badge(painter, tile_rect);
        }
        return;
    }

    let is_custom_data_selected = cx.selected_val_id == Some(CUSTOM_DATA_VALUE_ID);

    if let Some(enum_def) = cx.enum_def {
        if is_custom_data_selected {
            // When "Custom data" is selected: show ALL markers (concentric nested outlines for multi-marked tiles)
            draw_all_markers(painter, ctx, tile_rect, &tags, Some(enum_def), cx);

            if has_custom_data {
                draw_custom_data_badge(painter, tile_rect);
            }
        } else if let Some(sel_id) = cx.selected_val_id {
            // When an enum value is selected: ONLY show marker if this tile has that enum value!
            if cx.tileset.marking.has_tag(sel_id, tile_id) {
                if let Some(val) = enum_def.value(sel_id) {
                    let color = Color32::from_rgba_unmultiplied(
                        val.color[0],
                        val.color[1],
                        val.color[2],
                        val.color[3],
                    );

                    painter.rect_stroke(
                        tile_rect,
                        CornerRadius::same(1),
                        Stroke::new(2.0, Color32::BLACK),
                        StrokeKind::Outside,
                    );
                    painter.rect_stroke(
                        tile_rect,
                        CornerRadius::same(1),
                        Stroke::new(2.0, color),
                        StrokeKind::Inside,
                    );

                    // Fixed screen-space marker box (14x14) in top-left
                    draw_marker_icon(painter, ctx, tile_rect, Some(val), cx.project, cx.assets);
                }
            }
        }
    }
}

/// Draws all concentric tag outlines and the top-left marker icon for a tile.
fn draw_all_markers(
    painter: &egui::Painter,
    ctx: &egui::Context,
    tile_rect: Rect,
    tags: &[&str],
    enum_def: Option<&EnumDef>,
    cx: &TileOverlayContext<'_>,
) {
    let Some(enum_def) = enum_def else { return };
    if tags.is_empty() {
        return;
    }

    painter.rect_stroke(
        tile_rect,
        CornerRadius::same(1),
        Stroke::new(2.0, Color32::BLACK),
        StrokeKind::Outside,
    );

    let mut inset = 0.0;
    for tag in tags {
        if let Some(val) = enum_def.value(tag) {
            let color = Color32::from_rgba_unmultiplied(
                val.color[0],
                val.color[1],
                val.color[2],
                val.color[3],
            );
            let frame = tile_rect.shrink(inset);
            if frame.width() > 2.0 && frame.height() > 2.0 {
                painter.rect_stroke(
                    frame,
                    CornerRadius::same(1),
                    Stroke::new(2.0, color),
                    StrokeKind::Inside,
                );
                inset += 2.0;
            }
        }
    }

    // Fixed screen-space marker box (14x14) in top-left
    draw_marker_icon(
        painter,
        ctx,
        tile_rect,
        tags.first().and_then(|t| enum_def.value(t)),
        cx.project,
        cx.assets,
    );
}

/// Draws the fixed screen-space marker icon (bound tile thumbnail or solid color chip).
fn draw_marker_icon(
    painter: &egui::Painter,
    ctx: &egui::Context,
    tile_rect: Rect,
    val_opt: Option<&lc_core::defs::EnumValueDef>,
    project: &Project,
    assets: &dyn AssetService,
) {
    let Some(val) = val_opt else { return };
    let marker_box_size = 14.0;
    if tile_rect.width() < marker_box_size || tile_rect.height() < marker_box_size {
        return;
    }

    let icon_rect = Rect::from_min_size(
        tile_rect.min + vec2(2.0, 2.0),
        vec2(marker_box_size, marker_box_size),
    );
    painter.rect_filled(
        icon_rect,
        CornerRadius::same(2),
        Color32::from_rgb(20, 24, 28),
    );
    painter.rect_stroke(
        icon_rect,
        CornerRadius::same(2),
        Stroke::new(1.0, Color32::from_rgb(45, 52, 60)),
        StrokeKind::Inside,
    );

    let mut drew_tile = false;
    if let Some(ref bound_tile) = val.tile {
        drew_tile = paint_tile_thumbnail(painter, ctx, icon_rect, project, assets, bound_tile);
    }
    if !drew_tile {
        let dot_size = 8.0;
        let dot_rect = Rect::from_min_size(
            Pos2::new(
                icon_rect.center().x - dot_size / 2.0,
                icon_rect.center().y - dot_size / 2.0,
            ),
            vec2(dot_size, dot_size),
        );
        let color =
            Color32::from_rgba_unmultiplied(val.color[0], val.color[1], val.color[2], val.color[3]);
        painter.rect_filled(dot_rect, CornerRadius::same(1), color);
    }
}

/// Draws a subtle custom data indicator badge in the bottom-right corner of the tile.
fn draw_custom_data_badge(painter: &egui::Painter, tile_rect: Rect) {
    let badge_size = 12.0;
    if tile_rect.width() < badge_size || tile_rect.height() < badge_size {
        return;
    }
    let badge_rect = Rect::from_min_size(
        Pos2::new(
            tile_rect.max.x - badge_size - 1.0,
            tile_rect.max.y - badge_size - 1.0,
        ),
        vec2(badge_size, badge_size),
    );
    painter.rect_filled(
        badge_rect,
        CornerRadius::same(2),
        Color32::from_rgb(26, 32, 40),
    );
    painter.rect_stroke(
        badge_rect,
        CornerRadius::same(2),
        Stroke::new(1.0, Color32::from_rgb(100, 140, 180)),
        StrokeKind::Inside,
    );
}
