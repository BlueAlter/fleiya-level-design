//! Tile collision shapes and focus overlay rendering for atlas canvas.

use egui::{vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind};
use glam::Vec2;
use lc_canvas::theme::Tokens;
use lc_core::collision::{CollisionShape, CollisionShapeGeometry, TileCollisionDef};
use lc_core::defs::TilesetDef;

use super::state::{CollisionHandleTarget, CollisionSnapMode, CollisionTool, TilesetViewState};

/// Context parameters for rendering collision shapes and isolate/focus overlays.
#[derive(Debug, Clone)]
pub struct CollisionOverlayContext {
    /// Currently isolated tile ID in focus mode.
    pub focused_tile: Option<u32>,
    /// Whether the editor is currently in Collision mode.
    pub in_collision_mode: bool,
    /// Whether collision shapes should be drawn across the atlas.
    pub show_overlay: bool,
    /// Selected shape ID within the active tile.
    pub selected_shape_id: Option<u32>,
    /// Current grid snapping mode.
    pub snap: CollisionSnapMode,
    /// Active collision tool.
    pub tool: CollisionTool,
    /// In-progress drag start coordinate.
    pub drag_start: Option<Vec2>,
    /// In-progress drag current coordinate.
    pub drag_current: Option<Vec2>,
    /// In-progress drag handle target.
    pub drag_handle: Option<CollisionHandleTarget>,
    /// In-progress polygon draft points.
    pub poly_draft: Vec<Vec2>,
}

impl CollisionOverlayContext {
    /// Captures the view state fields needed for collision rendering.
    pub fn from_state(state: &TilesetViewState) -> Self {
        Self {
            focused_tile: state.collision_focus_tile,
            in_collision_mode: state.active_mode == "Collision",
            show_overlay: state.show_collision_overlay,
            selected_shape_id: state.selected_collision_shape_id,
            snap: state.collision_snap,
            tool: state.collision_tool,
            drag_start: state.collision_drag_start,
            drag_current: state.collision_drag_current,
            drag_handle: state.collision_drag_handle,
            poly_draft: state.collision_poly_draft.clone(),
        }
    }
}

/// Converts a local coordinate `(0..tile_w, 0..tile_h)` to screen position.
pub fn local_to_screen(local: Vec2, tile_rect: Rect, tile_size: glam::UVec2) -> Pos2 {
    let tw = (tile_size.x.max(1)) as f32;
    let th = (tile_size.y.max(1)) as f32;
    let sx = tile_rect.min.x + (local.x / tw) * tile_rect.width();
    let sy = tile_rect.min.y + (local.y / th) * tile_rect.height();
    Pos2::new(sx, sy)
}

/// Converts a screen position to local coordinate `(0..tile_w, 0..tile_h)`.
pub fn screen_to_local(screen: Pos2, tile_rect: Rect, tile_size: glam::UVec2) -> Vec2 {
    let tw = (tile_size.x.max(1)) as f32;
    let th = (tile_size.y.max(1)) as f32;
    let rw = tile_rect.width().max(1.0);
    let rh = tile_rect.height().max(1.0);
    let lx = ((screen.x - tile_rect.min.x) / rw) * tw;
    let ly = ((screen.y - tile_rect.min.y) / rh) * th;
    Vec2::new(lx, ly)
}

/// Snaps a local coordinate according to the active `CollisionSnapMode`.
pub fn snap_local_point(local: Vec2, tile_size: glam::UVec2, snap: CollisionSnapMode) -> Vec2 {
    let tw = tile_size.x as f32;
    let th = tile_size.y as f32;
    match snap {
        CollisionSnapMode::Pixel => Vec2::new(
            local.x.round().clamp(0.0, tw),
            local.y.round().clamp(0.0, th),
        ),
        CollisionSnapMode::TileSize => {
            let step_x = (tw / 2.0).max(1.0);
            let step_y = (th / 2.0).max(1.0);
            Vec2::new(
                ((local.x / step_x).round() * step_x).clamp(0.0, tw),
                ((local.y / step_y).round() * step_y).clamp(0.0, th),
            )
        }
        CollisionSnapMode::None => Vec2::new(local.x.clamp(0.0, tw), local.y.clamp(0.0, th)),
    }
}

/// Renders collision shapes and isolate/focus overlays for a tile on the canvas.
pub fn render_tile_collision_overlay(
    painter: &egui::Painter,
    tile_rect: Rect,
    tile_id: u32,
    tileset: &TilesetDef,
    cx: &CollisionOverlayContext,
    t: &Tokens,
) {
    let is_focused = cx.in_collision_mode && cx.focused_tile == Some(tile_id);

    // 1. If focused tile, draw active isolate highlight frame and sub-grid guides
    if is_focused {
        // High-contrast accent border around isolated tile
        painter.rect_stroke(
            tile_rect,
            CornerRadius::ZERO,
            Stroke::new(2.5, t.accent),
            StrokeKind::Inside,
        );

        // Sub-pixel or tile-size guides when zoomed in
        draw_focus_guides(painter, tile_rect, tileset.tile_size, cx.snap);
    }

    // 2. Render established collision shapes for this tile
    if cx.show_overlay || cx.in_collision_mode {
        if let Some(col_def) = tileset.collisions.get(&tile_id) {
            render_tile_shapes(
                painter,
                tile_rect,
                col_def,
                tileset.tile_size,
                is_focused,
                cx.selected_shape_id,
            );
        }
    }

    // 3. Render live in-progress drawing preview on focused tile
    if is_focused {
        render_in_progress_preview(painter, tile_rect, tileset.tile_size, cx);
    }
}

/// Draws subtle sub-grid lines on the isolated tile.
fn draw_focus_guides(
    painter: &egui::Painter,
    tile_rect: Rect,
    tile_size: glam::UVec2,
    snap: CollisionSnapMode,
) {
    let tw = tile_size.x.max(1);
    let th = tile_size.y.max(1);

    match snap {
        CollisionSnapMode::Pixel => {
            // Draw 1px pixel grid only when tile on screen is large enough
            let px_w = tile_rect.width() / tw as f32;
            let px_h = tile_rect.height() / th as f32;
            if px_w >= 6.0 && px_h >= 6.0 {
                let stroke = Stroke::new(1.0, Color32::from_white_alpha(16));
                for x in 1..tw {
                    let sx = tile_rect.min.x + x as f32 * px_w;
                    painter.line_segment(
                        [
                            Pos2::new(sx, tile_rect.min.y),
                            Pos2::new(sx, tile_rect.max.y),
                        ],
                        stroke,
                    );
                }
                for y in 1..th {
                    let sy = tile_rect.min.y + y as f32 * px_h;
                    painter.line_segment(
                        [
                            Pos2::new(tile_rect.min.x, sy),
                            Pos2::new(tile_rect.max.x, sy),
                        ],
                        stroke,
                    );
                }
            }
        }
        CollisionSnapMode::TileSize => {
            // Half-tile center guides
            let stroke = Stroke::new(1.0, Color32::from_white_alpha(32));
            let cx = tile_rect.center().x;
            let cy = tile_rect.center().y;
            painter.line_segment(
                [
                    Pos2::new(cx, tile_rect.min.y),
                    Pos2::new(cx, tile_rect.max.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(tile_rect.min.x, cy),
                    Pos2::new(tile_rect.max.x, cy),
                ],
                stroke,
            );
        }
        CollisionSnapMode::None => {}
    }
}

/// Renders all collision shapes in a tile definition.
fn render_tile_shapes(
    painter: &egui::Painter,
    tile_rect: Rect,
    col_def: &TileCollisionDef,
    tile_size: glam::UVec2,
    is_focused: bool,
    selected_shape_id: Option<u32>,
) {
    for shape in &col_def.shapes {
        let is_selected = is_focused && selected_shape_id == Some(shape.id);

        let (fill_color, stroke_color, stroke_width) = if is_selected {
            (
                Color32::from_rgba_unmultiplied(100, 200, 255, 90),
                Color32::from_rgb(255, 230, 90),
                2.0,
            )
        } else if shape.one_way {
            (
                Color32::from_rgba_unmultiplied(60, 210, 160, 50),
                Color32::from_rgb(70, 220, 170),
                1.5,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(60, 160, 240, 55),
                Color32::from_rgb(80, 180, 255),
                1.5,
            )
        };

        let stroke = Stroke::new(stroke_width, stroke_color);

        match &shape.geometry {
            CollisionShapeGeometry::Rectangle { x, y, w, h } => {
                let p_min = local_to_screen(Vec2::new(*x, *y), tile_rect, tile_size);
                let p_max = local_to_screen(Vec2::new(*x + *w, *y + *h), tile_rect, tile_size);
                let rect = Rect::from_two_pos(p_min, p_max);

                painter.rect_filled(rect, CornerRadius::ZERO, fill_color);
                painter.rect_stroke(rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

                if shape.one_way {
                    draw_one_way_indicator(painter, rect.min, Pos2::new(rect.max.x, rect.min.y));
                }

                if is_selected {
                    draw_rect_handles(painter, rect);
                }
            }
            CollisionShapeGeometry::Polygon { points } => {
                if points.len() >= 3 {
                    let screen_pts: Vec<Pos2> = points
                        .iter()
                        .map(|p| local_to_screen(*p, tile_rect, tile_size))
                        .collect();

                    painter.add(egui::Shape::convex_polygon(
                        screen_pts.clone(),
                        fill_color,
                        stroke,
                    ));

                    if shape.one_way && screen_pts.len() >= 2 {
                        draw_one_way_indicator(painter, screen_pts[0], screen_pts[1]);
                    }

                    if is_selected {
                        for pt in &screen_pts {
                            draw_handle(painter, *pt);
                        }
                    }
                }
            }
            CollisionShapeGeometry::Circle { cx, cy, radius } => {
                let center = local_to_screen(Vec2::new(*cx, *cy), tile_rect, tile_size);
                let tw = tile_size.x.max(1) as f32;
                let r = (*radius / tw) * tile_rect.width();

                painter.circle(center, r, fill_color, stroke);

                if is_selected {
                    draw_handle(painter, center + vec2(r, 0.0));
                    draw_handle(painter, center - vec2(r, 0.0));
                    draw_handle(painter, center + vec2(0.0, r));
                    draw_handle(painter, center - vec2(0.0, r));
                }
            }
        }

        // Shape tag pill if tagged and zoomed in enough
        if !shape.tag.is_empty() && tile_rect.width() >= 48.0 && !is_selected {
            draw_shape_tag_badge(painter, tile_rect, shape, tile_size);
        }
    }
}

/// Renders in-progress preview geometry while user is drawing a box or polygon.
fn render_in_progress_preview(
    painter: &egui::Painter,
    tile_rect: Rect,
    tile_size: glam::UVec2,
    cx: &CollisionOverlayContext,
) {
    match cx.tool {
        CollisionTool::Box => {
            if let (Some(start), Some(curr)) = (cx.drag_start, cx.drag_current) {
                let p1 = local_to_screen(start, tile_rect, tile_size);
                let p2 = local_to_screen(curr, tile_rect, tile_size);
                let preview_rect = Rect::from_two_pos(p1, p2);

                painter.rect_filled(
                    preview_rect,
                    CornerRadius::ZERO,
                    Color32::from_rgba_unmultiplied(80, 200, 255, 60),
                );
                painter.rect_stroke(
                    preview_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.5, Color32::from_rgb(100, 220, 255)),
                    StrokeKind::Inside,
                );
            }
        }
        CollisionTool::Polygon => {
            if !cx.poly_draft.is_empty() {
                let pts: Vec<Pos2> = cx
                    .poly_draft
                    .iter()
                    .map(|p| local_to_screen(*p, tile_rect, tile_size))
                    .collect();

                let stroke = Stroke::new(1.5, Color32::from_rgb(80, 220, 160));

                // Draw existing draft segments
                for i in 0..pts.len().saturating_sub(1) {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }

                // Draw draft vertices
                for (idx, pt) in pts.iter().enumerate() {
                    let handle_col = if idx == 0 {
                        Color32::from_rgb(100, 255, 120) // Highlight start point
                    } else {
                        Color32::from_rgb(255, 230, 90)
                    };
                    painter.rect_filled(
                        Rect::from_center_size(*pt, vec2(6.0, 6.0)),
                        CornerRadius::ZERO,
                        handle_col,
                    );
                }

                // Draw segment to current cursor pos
                if let Some(curr) = cx.drag_current {
                    if let Some(&last_pt) = pts.last() {
                        let curr_screen = local_to_screen(curr, tile_rect, tile_size);
                        painter.line_segment(
                            [last_pt, curr_screen],
                            Stroke::new(1.0, Color32::from_white_alpha(150)),
                        );

                        // If hovering near first point with >= 3 points, highlight close loop
                        if pts.len() >= 3 && curr_screen.distance(pts[0]) < 12.0 {
                            painter.circle_stroke(
                                pts[0],
                                8.0,
                                Stroke::new(2.0, Color32::from_rgb(60, 255, 120)),
                            );
                        }
                    }
                }
            }
        }
        CollisionTool::Select => {
            if let (Some(_), Some(curr)) = (cx.drag_handle, cx.drag_current) {
                let curr_screen = local_to_screen(curr, tile_rect, tile_size);
                painter.circle_filled(curr_screen, 5.0, Color32::from_rgb(255, 230, 90));
                painter.circle_stroke(
                    curr_screen,
                    5.0,
                    Stroke::new(2.0, Color32::from_rgb(20, 24, 30)),
                );
            }
        }
    }
}

/// Draws an indicator for one-way platform shapes along the top segment.
fn draw_one_way_indicator(painter: &egui::Painter, p1: Pos2, p2: Pos2) {
    let stroke = Stroke::new(2.5, Color32::from_rgb(80, 255, 160));
    painter.line_segment([p1, p2], stroke);

    // Directional upward arrow tick in center
    let center = Pos2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
    let tip = center - vec2(0.0, 6.0);
    painter.line_segment(
        [center, tip],
        Stroke::new(1.5, Color32::from_rgb(80, 255, 160)),
    );
}

/// Draws resize handles at the 4 corners of a rectangular shape.
fn draw_rect_handles(painter: &egui::Painter, rect: Rect) {
    draw_handle(painter, rect.left_top());
    draw_handle(painter, rect.right_top());
    draw_handle(painter, rect.right_bottom());
    draw_handle(painter, rect.left_bottom());
}

/// Draws a single circular resize handle dot.
fn draw_handle(painter: &egui::Painter, center: Pos2) {
    painter.circle_filled(center, 4.0, Color32::WHITE);
    painter.circle_stroke(center, 4.0, Stroke::new(1.5, Color32::from_rgb(20, 24, 30)));
}

/// Draws a tiny text pill badge showing the shape's semantic tag.
fn draw_shape_tag_badge(
    painter: &egui::Painter,
    tile_rect: Rect,
    shape: &CollisionShape,
    tile_size: glam::UVec2,
) {
    let anchor = match &shape.geometry {
        CollisionShapeGeometry::Rectangle { x, y, .. } => {
            local_to_screen(Vec2::new(*x, *y), tile_rect, tile_size)
        }
        CollisionShapeGeometry::Polygon { points } => points
            .first()
            .map(|p| local_to_screen(*p, tile_rect, tile_size))
            .unwrap_or(tile_rect.min),
        CollisionShapeGeometry::Circle { cx, cy, .. } => {
            local_to_screen(Vec2::new(*cx, *cy), tile_rect, tile_size)
        }
    };

    let font = FontId::monospace(8.5);
    painter.text(
        anchor + vec2(2.0, 2.0),
        Align2::LEFT_TOP,
        &shape.tag,
        font,
        Color32::from_white_alpha(180),
    );
}
