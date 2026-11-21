//! 2D view and pure coordinate math for atlas viewport navigation.

use egui::{vec2, Pos2, Rect, Vec2};

use super::super::tile_bounds::TileGrid;

/// 2D view state representing the canvas zoom scale and screen translation offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickerView {
    /// Zoom scale factor, clamped between `0.25` and `32.0`.
    pub zoom: f32,
    /// Screen-space translation offset for the top-left of the page.
    pub offset: Vec2,
}

impl Default for PickerView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

impl PickerView {
    /// Creates a new view with the given zoom and offset.
    pub fn new(zoom: f32, offset: Vec2) -> Self {
        Self { zoom, offset }
    }

    /// Transforms a point from page coordinates to screen coordinates.
    pub fn page_to_screen(&self, page_pos: Pos2) -> Pos2 {
        Pos2::new(
            (page_pos.x * self.zoom + self.offset.x).round(),
            (page_pos.y * self.zoom + self.offset.y).round(),
        )
    }

    /// Transforms a point from screen coordinates to page coordinates.
    pub fn screen_to_page(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.offset.x) / self.zoom,
            (screen_pos.y - self.offset.y) / self.zoom,
        )
    }

    /// Transforms a rectangle from page coordinates to screen coordinates.
    pub fn page_to_screen_rect(&self, page_rect: Rect) -> Rect {
        Rect::from_min_max(
            self.page_to_screen(page_rect.min),
            self.page_to_screen(page_rect.max),
        )
    }
}

/// Physical dimension in points of one background tile in the neutral background texture.
pub const PAGE_BG_CELL: f32 = 16.0;

/// Repeats the neutral page background across the whole page area.
///
/// UV coordinates outside `0..1` are repeated by `TextureOptions::NEAREST_REPEAT`.
pub fn page_bg_uv(rect: Rect) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        vec2(
            (rect.width() / PAGE_BG_CELL).max(0.0),
            (rect.height() / PAGE_BG_CELL).max(0.0),
        ),
    )
}

/// Computes zoom and offset to fit an image centered in the viewport with margin.
pub fn compute_fit_view(viewport_rect: Rect, img_w: f32, img_h: f32) -> (f32, Vec2) {
    let avail_w = (viewport_rect.width() - 48.0).max(10.0);
    let avail_h = (viewport_rect.height() - 48.0).max(10.0);
    let fit_zoom = if img_w > 0.0 && img_h > 0.0 {
        (avail_w / img_w).min(avail_h / img_h).clamp(0.25, 32.0)
    } else {
        1.0
    };
    let zoomed_w = img_w * fit_zoom;
    let zoomed_h = img_h * fit_zoom;
    let center_x = viewport_rect.min.x + (viewport_rect.width() - zoomed_w) / 2.0;
    let center_y = viewport_rect.min.y + (viewport_rect.height() - zoomed_h) / 2.0;
    (fit_zoom, vec2(center_x.round(), center_y.round()))
}

/// Computes new zoom and offset when zooming towards a screen cursor position.
pub fn zoom_towards_cursor(
    cursor: Pos2,
    old_zoom: f32,
    old_offset: Vec2,
    zoom_factor: f32,
) -> (f32, Vec2) {
    let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 32.0);
    if (new_zoom - old_zoom).abs() < 0.0001 {
        return (old_zoom, old_offset);
    }
    let page_cursor_x = (cursor.x - old_offset.x) / old_zoom;
    let page_cursor_y = (cursor.y - old_offset.y) / old_zoom;
    let new_offset_x = (cursor.x - page_cursor_x * new_zoom).round();
    let new_offset_y = (cursor.y - page_cursor_y * new_zoom).round();
    (new_zoom, vec2(new_offset_x, new_offset_y))
}

/// Hit-tests a screen position against the grid under the given zoom and offset.
pub fn tile_at_screen_pos(
    screen_pos: Pos2,
    zoom: f32,
    offset: Vec2,
    grid: &TileGrid,
) -> Option<(u32, u32)> {
    if zoom <= 0.0 {
        return None;
    }
    let page_pos = Pos2::new(
        (screen_pos.x - offset.x) / zoom,
        (screen_pos.y - offset.y) / zoom,
    );
    grid.cell_at(page_pos)
}

/// Clamps the canvas offset so the page remains partially visible inside the viewport.
pub fn clamp_offset(offset: Vec2, viewport_rect: Rect, img_w: f32, img_h: f32, zoom: f32) -> Vec2 {
    let zoomed_w = img_w * zoom;
    let zoomed_h = img_h * zoom;
    vec2(
        offset.x.clamp(
            viewport_rect.min.x + 24.0 - zoomed_w,
            viewport_rect.max.x - 24.0,
        ),
        offset.y.clamp(
            viewport_rect.min.y + 24.0 - zoomed_h,
            viewport_rect.max.y - 24.0,
        ),
    )
}
