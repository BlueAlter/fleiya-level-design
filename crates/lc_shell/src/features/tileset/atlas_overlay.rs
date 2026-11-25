//! Tile marking and custom data overlay rendering on the atlas canvas.

use egui::{
    vec2, Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, Response, RichText,
    Stroke, StrokeKind,
};
use lc_canvas::theme::Tokens;
use lc_core::defs::{EnumDef, TilesetDef};
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::AssetService;

use super::right_panel::paint_tile_thumbnail;
use super::state::CUSTOM_DATA_VALUE_ID;

/// Contextual parameters required for rendering tile marking and animation overlays.
pub struct TileOverlayContext<'a> {
    pub tileset: &'a TilesetDef,
    pub enum_def: Option<&'a EnumDef>,
    pub is_marking_mode: bool,
    pub is_animation_mode: bool,
    pub selected_val_id: Option<&'a str>,
    pub selected_animation_anchor: Option<u32>,
    pub is_animation_playing: bool,
    pub animation_start_time: std::time::Instant,
    pub project: &'a Project,
    pub assets: &'a dyn AssetService,
}

impl<'a> TileOverlayContext<'a> {
    /// Creates a default context with animation playback inactive.
    pub fn new(
        tileset: &'a TilesetDef,
        enum_def: Option<&'a EnumDef>,
        is_marking_mode: bool,
        selected_val_id: Option<&'a str>,
        project: &'a Project,
        assets: &'a dyn AssetService,
    ) -> Self {
        Self {
            tileset,
            enum_def,
            is_marking_mode,
            is_animation_mode: false,
            selected_val_id,
            selected_animation_anchor: None,
            is_animation_playing: false,
            animation_start_time: std::time::Instant::now(),
            project,
            assets,
        }
    }
}

/// Renders the marking overlay, animation frames, and custom data indicator on a tile.
pub fn render_tile_overlay(
    painter: &egui::Painter,
    ctx: &egui::Context,
    tile_rect: Rect,
    tile_id: u32,
    cx: &TileOverlayContext<'_>,
) {
    // 1. Anchor tile animation playback: dynamically swaps anchor tile UV to active frame
    if let Some(anim) = cx.tileset.animations.get(&tile_id) {
        let active_frame = if cx.is_animation_playing {
            let elapsed_ms = cx.animation_start_time.elapsed().as_millis() as u64;
            anim.active_frame(elapsed_ms)
        } else {
            anim.frames.first().copied().unwrap_or(tile_id)
        };

        if active_frame != tile_id {
            let tile_ref = TileRef {
                tileset_id: cx.tileset.id,
                tile_id: active_frame,
                flip_x: false,
                flip_y: false,
            };
            paint_tile_thumbnail(painter, ctx, tile_rect, cx.project, cx.assets, &tile_ref);
        }
        if cx.is_animation_playing {
            ctx.request_repaint();
        }
    }

    // 2. Animation mode overlay: anchor marker and selected anchor frame badges
    if cx.is_animation_mode {
        if cx.tileset.animations.contains_key(&tile_id) {
            painter.rect_stroke(
                tile_rect,
                CornerRadius::ZERO,
                Stroke::new(2.0, Color32::from_rgb(61, 214, 184)),
                StrokeKind::Inside,
            );
            draw_anchor_badge(painter, tile_rect);
        }

        if let Some(sel_anchor) = cx.selected_animation_anchor {
            if let Some(sel_anim) = cx.tileset.animations.get(&sel_anchor) {
                if let Some(pos) = sel_anim.frames.iter().position(|&f| f == tile_id) {
                    painter.rect_stroke(
                        tile_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.5, Color32::from_rgb(255, 190, 80)),
                        StrokeKind::Inside,
                    );
                    draw_frame_badge(painter, tile_rect, pos + 1);
                }
            }
        }
        return;
    }

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
                        CornerRadius::ZERO,
                        Stroke::new(2.0, Color32::BLACK),
                        StrokeKind::Outside,
                    );
                    painter.rect_stroke(
                        tile_rect,
                        CornerRadius::ZERO,
                        Stroke::new(2.0, color),
                        StrokeKind::Inside,
                    );

                    // Proportional marker box (50% of tile size) in top-left
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
        CornerRadius::ZERO,
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
                    CornerRadius::ZERO,
                    Stroke::new(2.0, color),
                    StrokeKind::Inside,
                );
                inset += 2.0;
            }
        }
    }

    // Proportional marker box (50% of tile size) in top-left
    draw_marker_icon(
        painter,
        ctx,
        tile_rect,
        tags.first().and_then(|t| enum_def.value(t)),
        cx.project,
        cx.assets,
    );
}

/// Draws the proportional marker icon (bound tile thumbnail or solid color chip).
fn draw_marker_icon(
    painter: &egui::Painter,
    ctx: &egui::Context,
    tile_rect: Rect,
    val_opt: Option<&lc_core::defs::EnumValueDef>,
    project: &Project,
    assets: &dyn AssetService,
) {
    let Some(val) = val_opt else { return };
    let marker_box_size = (tile_rect.width() * 0.5).round().max(4.0);
    if tile_rect.width() < marker_box_size || tile_rect.height() < marker_box_size {
        return;
    }

    let offset = (tile_rect.width() * 0.06).round().clamp(1.0, 3.0);
    let icon_rect = Rect::from_min_size(
        tile_rect.min + vec2(offset, offset),
        vec2(marker_box_size, marker_box_size),
    );
    painter.rect_filled(icon_rect, CornerRadius::ZERO, Color32::from_rgb(20, 24, 28));
    painter.rect_stroke(
        icon_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(45, 52, 60)),
        StrokeKind::Inside,
    );

    let mut drew_tile = false;
    if let Some(ref bound_tile) = val.tile {
        drew_tile = paint_tile_thumbnail(painter, ctx, icon_rect, project, assets, bound_tile);
    }
    if !drew_tile {
        let dot_size = (marker_box_size * 0.65).round().max(2.0);
        let dot_rect = Rect::from_min_size(
            Pos2::new(
                icon_rect.center().x - dot_size / 2.0,
                icon_rect.center().y - dot_size / 2.0,
            ),
            vec2(dot_size, dot_size),
        );
        let color =
            Color32::from_rgba_unmultiplied(val.color[0], val.color[1], val.color[2], val.color[3]);
        painter.rect_filled(dot_rect, CornerRadius::ZERO, color);
    }
}

/// Draws a subtle custom data indicator badge in the bottom-right corner of the tile.
fn draw_custom_data_badge(painter: &egui::Painter, tile_rect: Rect) {
    let badge_size = (tile_rect.width() * 0.35).round().max(4.0);
    if tile_rect.width() < badge_size || tile_rect.height() < badge_size {
        return;
    }
    let offset = (tile_rect.width() * 0.06).round().clamp(1.0, 3.0);
    let badge_rect = Rect::from_min_size(
        Pos2::new(
            tile_rect.max.x - badge_size - offset,
            tile_rect.max.y - badge_size - offset,
        ),
        vec2(badge_size, badge_size),
    );
    painter.rect_filled(
        badge_rect,
        CornerRadius::ZERO,
        Color32::from_rgb(26, 32, 40),
    );
    painter.rect_stroke(
        badge_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(100, 140, 180)),
        StrokeKind::Inside,
    );
}

/// Draws an anchor badge in the top-left corner of the tile.
fn draw_anchor_badge(painter: &egui::Painter, tile_rect: Rect) {
    let badge_size = (tile_rect.width() * 0.4).round().clamp(10.0, 20.0);
    if tile_rect.width() < badge_size || tile_rect.height() < badge_size {
        return;
    }
    let offset = (tile_rect.width() * 0.05).round().clamp(1.0, 3.0);
    let badge_rect = Rect::from_min_size(
        tile_rect.min + vec2(offset, offset),
        vec2(badge_size, badge_size),
    );
    painter.rect_filled(
        badge_rect,
        CornerRadius::ZERO,
        Color32::from_rgb(20, 24, 28),
    );
    painter.rect_stroke(
        badge_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(61, 214, 184)),
        StrokeKind::Inside,
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "A",
        FontId::monospace(badge_size * 0.6),
        Color32::from_rgb(61, 214, 184),
    );
}

/// Draws a numbered frame badge in the bottom-left corner of the tile.
fn draw_frame_badge(painter: &egui::Painter, tile_rect: Rect, frame_num: usize) {
    let badge_h = (tile_rect.height() * 0.35).round().clamp(10.0, 18.0);
    let badge_w = (badge_h * 1.5).round();
    if tile_rect.width() < badge_w || tile_rect.height() < badge_h {
        return;
    }
    let offset = (tile_rect.width() * 0.05).round().clamp(1.0, 3.0);
    let badge_rect = Rect::from_min_size(
        Pos2::new(tile_rect.min.x + offset, tile_rect.max.y - badge_h - offset),
        vec2(badge_w, badge_h),
    );
    painter.rect_filled(
        badge_rect,
        CornerRadius::ZERO,
        Color32::from_rgb(20, 24, 28),
    );
    painter.rect_stroke(
        badge_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(255, 190, 80)),
        StrokeKind::Inside,
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        format!("F{frame_num}"),
        FontId::monospace(badge_h * 0.6),
        Color32::from_rgb(255, 190, 80),
    );
}

/// Renders a structured hover tooltip for a tile on the atlas canvas.
pub fn render_tile_hover_tooltip(
    response: &Response,
    t: &Tokens,
    title: &str,
    tag_names: &[String],
    custom_data_text: Option<&str>,
) {
    response.clone().on_hover_ui_at_pointer(|ui| {
        Frame::new()
            .fill(t.panel)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_max_width(220.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(11.5).strong().color(t.ink_strong));

                    if !tag_names.is_empty() {
                        ui.add_space(2.0);
                        let limit = 6;
                        let display_tags = &tag_names[..tag_names.len().min(limit)];
                        ui.vertical(|ui| {
                            for name in display_tags {
                                ui.label(
                                    RichText::new(format!("• {name}")).size(11.0).color(t.ink),
                                );
                            }
                            if tag_names.len() > limit {
                                ui.label(
                                    RichText::new(format!(
                                        "+ {} more tags",
                                        tag_names.len() - limit
                                    ))
                                    .size(9.5)
                                    .color(t.muted),
                                );
                            }
                        });
                    }

                    if let Some(custom_data) = custom_data_text {
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new("Data:")
                                .size(10.5)
                                .strong()
                                .color(t.accent_ink),
                        );
                        let lines: Vec<&str> = custom_data.lines().collect();
                        let line_limit = 6;
                        let display_count = lines.len().min(line_limit);
                        let preview_snippet = lines[..display_count].join("\n");
                        ui.label(RichText::new(preview_snippet).size(10.0).color(t.faint));
                        if lines.len() > line_limit {
                            ui.label(
                                RichText::new(format!(
                                    "... (+{} more lines)",
                                    lines.len() - line_limit
                                ))
                                .size(9.5)
                                .italics()
                                .color(t.muted),
                            );
                        }
                    }
                });
            });
    });
}
