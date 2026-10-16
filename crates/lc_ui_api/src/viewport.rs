//! Viewport pan, zoom, and coordinate conversion system.

use glam::IVec2;

/// Default minimum zoom limit (5%).
pub const DEFAULT_MIN_ZOOM: f32 = 0.05;
/// Default maximum zoom limit (3200%).
pub const DEFAULT_MAX_ZOOM: f32 = 32.0;

/// Mathematical 2D viewport managing screen-space camera pan and zoom,
/// with conversions between Screen, World, and Grid coordinate spaces.
#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    /// Screen rect bounding the canvas area.
    pub screen_rect: egui::Rect,
    /// Screen-space pan offset of the world origin `(0, 0)` relative to `screen_rect.min`.
    pub pan: egui::Vec2,
    /// Current zoom scaling factor (1.0 = 100%).
    pub zoom: f32,
    /// Minimum allowed zoom level.
    pub min_zoom: f32,
    /// Maximum allowed zoom level.
    pub max_zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            screen_rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO),
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            min_zoom: DEFAULT_MIN_ZOOM,
            max_zoom: DEFAULT_MAX_ZOOM,
        }
    }
}

impl Viewport {
    /// Creates a new viewport configured for a given screen rectangle.
    pub fn new(screen_rect: egui::Rect) -> Self {
        Self {
            screen_rect,
            ..Default::default()
        }
    }

    /// Sets the zoom factor, clamped between `min_zoom` and `max_zoom`.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Pans the camera by a screen-space delta.
    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    /// Adjusts zoom by multiplying with `factor`, keeping `pivot_screen` stationary in world coordinates.
    pub fn zoom_at(&mut self, factor: f32, pivot_screen: egui::Pos2) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(self.min_zoom, self.max_zoom);
        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        let pivot_rel = pivot_screen - self.screen_rect.min;
        let scale = new_zoom / old_zoom;
        self.pan = pivot_rel - (pivot_rel - self.pan) * scale;
        self.zoom = new_zoom;
    }

    /// Converts screen-space coordinates to integer world coordinates (pixels).
    ///
    /// Uses `.floor()` to ensure mathematically correct rounding for both positive and negative world coordinates.
    ///
    /// ### Quantization and Sub-pixel Placement Note
    /// When `zoom < 1.0` (zoomed out), one screen pixel corresponds to `1.0 / zoom` world units (e.g. at 50% zoom,
    /// 1 screen pixel = 2 world pixels). Consequently, screen cursor movements quantize across world pixels, and
    /// multiple adjacent screen pixels map to the same integer coordinate. For smooth continuous manipulation
    /// (such as rulers or vector gizmos), use [`Self::screen_to_world_f32`].
    pub fn screen_to_world(&self, screen: egui::Pos2) -> IVec2 {
        let rel = screen - self.screen_rect.min - self.pan;
        let wx = (rel.x / self.zoom).floor() as i32;
        let wy = (rel.y / self.zoom).floor() as i32;
        IVec2::new(wx, wy)
    }

    /// Converts screen-space coordinates to continuous floating-point world coordinates.
    ///
    /// Useful for smooth sub-pixel gizmos, vector previews, or floating-point interpolations.
    pub fn screen_to_world_f32(&self, screen: egui::Pos2) -> glam::Vec2 {
        let rel = screen - self.screen_rect.min - self.pan;
        glam::Vec2::new(rel.x / self.zoom, rel.y / self.zoom)
    }

    /// Converts integer world coordinates (pixels) to screen-space coordinates.
    pub fn world_to_screen(&self, world: IVec2) -> egui::Pos2 {
        let x = self.screen_rect.min.x + self.pan.x + (world.x as f32) * self.zoom;
        let y = self.screen_rect.min.y + self.pan.y + (world.y as f32) * self.zoom;
        egui::pos2(x, y)
    }

    /// Converts world coordinates to grid cell coordinates within a level.
    ///
    /// Correctly handles negative coordinates using Euclidean division.
    pub fn world_to_grid(world: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2 {
        let local = world - level_world_pos;
        let gx = local.x.div_euclid(grid_size.x.max(1));
        let gy = local.y.div_euclid(grid_size.y.max(1));
        IVec2::new(gx, gy)
    }

    /// Converts grid cell coordinates within a level to top-left world coordinates.
    pub fn grid_to_world(grid: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2 {
        level_world_pos + IVec2::new(grid.x * grid_size.x, grid.y * grid_size.y)
    }

    /// Converts screen coordinates to grid cell coordinates within a level.
    pub fn screen_to_grid(
        &self,
        screen: egui::Pos2,
        level_world_pos: IVec2,
        grid_size: IVec2,
    ) -> IVec2 {
        let world = self.screen_to_world(screen);
        Self::world_to_grid(world, level_world_pos, grid_size)
    }

    /// Converts a grid cell to a screen-space rectangle.
    pub fn grid_to_screen(
        &self,
        grid: IVec2,
        level_world_pos: IVec2,
        grid_size: IVec2,
    ) -> egui::Rect {
        let top_left_world = Self::grid_to_world(grid, level_world_pos, grid_size);
        let bottom_right_world = top_left_world + grid_size;
        let min_pos = self.world_to_screen(top_left_world);
        let max_pos = self.world_to_screen(bottom_right_world);
        egui::Rect::from_two_pos(min_pos, max_pos)
    }

    /// Returns the minimum and maximum world coordinates currently visible in the screen rect.
    pub fn visible_world_bounds(&self) -> (IVec2, IVec2) {
        let min_world = self.screen_to_world(self.screen_rect.min);
        let max_world = self.screen_to_world(self.screen_rect.max);
        (min_world, max_world)
    }
}
