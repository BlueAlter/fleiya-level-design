//! Coordinate mathematics and data structures for tile grid layout and multi-tile bounds.
//!
//! Provides [`TileGrid`] and [`TileRect`] for pure coordinate math, shared between
//! the Tile Picker, enum value icon previews, and tileset features without egui UI dependencies.

use egui::{pos2, vec2, Pos2, Rect};
use serde::{Deserialize, Serialize};

/// Layout configuration of a tileset's sliced grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileGrid {
    /// Width of an individual tile in pixels.
    pub tile_w: u32,
    /// Height of an individual tile in pixels.
    pub tile_h: u32,
    /// Horizontal and vertical spacing between tiles in pixels.
    pub spacing: u32,
    /// Border padding around the tileset in pixels.
    pub padding: u32,
    /// Total number of tile columns.
    pub cols: u32,
    /// Total number of tile rows.
    pub rows: u32,
}

impl TileGrid {
    /// Creates a new `TileGrid` descriptor.
    pub fn new(tile_w: u32, tile_h: u32, spacing: u32, padding: u32, cols: u32, rows: u32) -> Self {
        Self {
            tile_w,
            tile_h,
            spacing,
            padding,
            cols,
            rows,
        }
    }

    /// Computes the pixel rectangle of a given cell `(col, row)` in page coordinates.
    pub fn cell_rect(&self, col: u32, row: u32) -> Rect {
        let stride_x = self.tile_w + self.spacing;
        let stride_y = self.tile_h + self.spacing;
        let x = self.padding + col * stride_x;
        let y = self.padding + row * stride_y;
        Rect::from_min_size(
            pos2(x as f32, y as f32),
            vec2(self.tile_w as f32, self.tile_h as f32),
        )
    }

    /// Maps a page coordinate `(x, y)` to a grid cell `(col, row)` if within valid tile boundaries.
    ///
    /// Returns `None` if the position falls in padding, spacing gaps, or outside the grid bounds.
    pub fn cell_at(&self, page_pos: Pos2) -> Option<(u32, u32)> {
        if page_pos.x < self.padding as f32 || page_pos.y < self.padding as f32 {
            return None;
        }
        let rel_x = page_pos.x - self.padding as f32;
        let rel_y = page_pos.y - self.padding as f32;
        let stride_x = self.tile_w + self.spacing;
        let stride_y = self.tile_h + self.spacing;
        if stride_x == 0 || stride_y == 0 {
            return None;
        }

        let c = (rel_x / stride_x as f32).floor() as i32;
        let r = (rel_y / stride_y as f32).floor() as i32;
        if c < 0 || c >= self.cols as i32 || r < 0 || r >= self.rows as i32 {
            return None;
        }

        let col = c as u32;
        let row = r as u32;

        let in_cell_x = rel_x - (col * stride_x) as f32;
        let in_cell_y = rel_y - (row * stride_y) as f32;
        if in_cell_x <= self.tile_w as f32 && in_cell_y <= self.tile_h as f32 {
            Some((col, row))
        } else {
            None
        }
    }

    /// Linear tile ID for a given `(col, row)` cell.
    pub fn tile_id(&self, col: u32, row: u32) -> u32 {
        row * self.cols + col
    }

    /// Converts a linear tile ID into `(col, row)` coordinates.
    pub fn col_row(&self, tile_id: u32) -> (u32, u32) {
        if self.cols == 0 {
            (0, 0)
        } else {
            (tile_id % self.cols, tile_id / self.cols)
        }
    }

    /// Total pixel width occupied by the tileset page.
    pub fn total_page_width(&self) -> f32 {
        if self.cols == 0 {
            0.0
        } else {
            (self.padding * 2
                + self.cols * self.tile_w
                + self.cols.saturating_sub(1) * self.spacing) as f32
        }
    }

    /// Total pixel height occupied by the tileset page.
    pub fn total_page_height(&self) -> f32 {
        if self.rows == 0 {
            0.0
        } else {
            (self.padding * 2
                + self.rows * self.tile_h
                + self.rows.saturating_sub(1) * self.spacing) as f32
        }
    }
}

/// A rectangular selection of tiles defined in cell coordinates `(col, row, w, h)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRect {
    /// Top-left column index.
    pub col: u32,
    /// Top-left row index.
    pub row: u32,
    /// Width in number of tile cells.
    pub w: u32,
    /// Height in number of tile cells.
    pub h: u32,
}

impl TileRect {
    /// Creates a 1x1 tile rect for a single cell.
    pub const fn single(col: u32, row: u32) -> Self {
        Self {
            col,
            row,
            w: 1,
            h: 1,
        }
    }

    /// Constructs a normalized and inclusive `TileRect` spanning from corner `a` to corner `b`.
    ///
    /// Handles drags in any direction (e.g. backward, upward) by normalizing to minimum coordinates.
    pub fn from_cells(a: (u32, u32), b: (u32, u32)) -> Self {
        let min_col = a.0.min(b.0);
        let max_col = a.0.max(b.0);
        let min_row = a.1.min(b.1);
        let max_row = a.1.max(b.1);
        Self {
            col: min_col,
            row: min_row,
            w: max_col - min_col + 1,
            h: max_row - min_row + 1,
        }
    }

    /// Clamps this `TileRect` to ensure it stays strictly within the grid dimensions `cols x rows`.
    pub fn clamp(&self, cols: u32, rows: u32) -> Self {
        if cols == 0 || rows == 0 {
            return Self {
                col: 0,
                row: 0,
                w: 0,
                h: 0,
            };
        }
        let col = self.col.min(cols - 1);
        let row = self.row.min(rows - 1);
        let max_col = (self.col + self.w.max(1) - 1).min(cols - 1);
        let max_row = (self.row + self.h.max(1) - 1).min(rows - 1);
        let w = max_col - col + 1;
        let h = max_row - row + 1;
        Self { col, row, w, h }
    }

    /// Returns `true` if the given `(col, row)` falls inside this rect.
    pub fn contains(&self, col: u32, row: u32) -> bool {
        col >= self.col && col < self.col + self.w && row >= self.row && row < self.row + self.h
    }

    /// Iterator over all `(col, row)` cell coordinates contained in this rect.
    pub fn cells(&self) -> impl Iterator<Item = (u32, u32)> {
        let start_col = self.col;
        let start_row = self.row;
        let w = self.w;
        let h = self.h;
        (0..h).flat_map(move |dr| (0..w).map(move |dc| (start_col + dc, start_row + dr)))
    }

    /// Linear tile ID of the top-left cell in this rect.
    pub fn top_left_id(&self, grid: &TileGrid) -> u32 {
        grid.tile_id(self.col, self.row)
    }

    /// Page pixel rectangle enclosing all cells in this `TileRect`, including internal spacing.
    pub fn union_rect(&self, grid: &TileGrid) -> Rect {
        if self.w == 0 || self.h == 0 {
            return Rect::NOTHING;
        }
        let tl = grid.cell_rect(self.col, self.row);
        let br = grid.cell_rect(self.col + self.w - 1, self.row + self.h - 1);
        Rect::from_min_max(tl.min, br.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tile() {
        let grid = TileGrid::new(16, 16, 0, 0, 8, 8);
        let rect = TileRect::from_cells((3, 2), (3, 2));
        assert_eq!(rect.col, 3);
        assert_eq!(rect.row, 2);
        assert_eq!(rect.w, 1);
        assert_eq!(rect.h, 1);
        assert!(rect.contains(3, 2));
        assert!(!rect.contains(4, 2));
        assert_eq!(rect.cells().count(), 1);
        assert_eq!(rect.cells().next(), Some((3, 2)));
        assert_eq!(rect.top_left_id(&grid), 2 * 8 + 3);

        let pixel_rect = rect.union_rect(&grid);
        assert_eq!(pixel_rect.min, pos2(48.0, 32.0));
        assert_eq!(pixel_rect.max, pos2(64.0, 48.0));
    }

    #[test]
    fn test_backward_drag() {
        // Dragging backward from bottom-right (5, 5) to top-left (2, 1)
        let rect = TileRect::from_cells((5, 5), (2, 1));
        assert_eq!(rect.col, 2);
        assert_eq!(rect.row, 1);
        assert_eq!(rect.w, 4); // cols 2, 3, 4, 5 = 4
        assert_eq!(rect.h, 5); // rows 1, 2, 3, 4, 5 = 5

        // Dragging top-right to bottom-left
        let rect2 = TileRect::from_cells((6, 1), (3, 4));
        assert_eq!(rect2.col, 3);
        assert_eq!(rect2.row, 1);
        assert_eq!(rect2.w, 4);
        assert_eq!(rect2.h, 4);
    }

    #[test]
    fn test_drag_outside_image_clamp() {
        let raw = TileRect {
            col: 2,
            row: 2,
            w: 10,
            h: 10,
        };
        // Clamping to a 4x4 grid (valid indices 0..4)
        let clamped = raw.clamp(4, 4);
        assert_eq!(clamped.col, 2);
        assert_eq!(clamped.row, 2);
        assert_eq!(clamped.w, 2); // cols 2, 3
        assert_eq!(clamped.h, 2); // rows 2, 3

        let out_of_bounds = TileRect {
            col: 10,
            row: 10,
            w: 3,
            h: 3,
        };
        let clamped_out = out_of_bounds.clamp(4, 4);
        assert_eq!(clamped_out.col, 3);
        assert_eq!(clamped_out.row, 3);
        assert_eq!(clamped_out.w, 1);
        assert_eq!(clamped_out.h, 1);
    }

    #[test]
    fn test_non_zero_spacing_and_padding() {
        // tile size 16x16, spacing 2, padding 4, 4 cols, 4 rows
        let grid = TileGrid::new(16, 16, 2, 4, 4, 4);

        // Cell (0, 0)
        let r0 = grid.cell_rect(0, 0);
        assert_eq!(r0.min, pos2(4.0, 4.0));
        assert_eq!(r0.max, pos2(20.0, 20.0));

        // Cell (1, 0) stride is 16 + 2 = 18; starts at 4 + 18 = 22
        let r1 = grid.cell_rect(1, 0);
        assert_eq!(r1.min, pos2(22.0, 4.0));
        assert_eq!(r1.max, pos2(38.0, 20.0));

        // cell_at hit tests
        assert_eq!(grid.cell_at(pos2(10.0, 10.0)), Some((0, 0)));
        assert_eq!(grid.cell_at(pos2(25.0, 10.0)), Some((1, 0)));

        // In padding before (0, 0)
        assert_eq!(grid.cell_at(pos2(2.0, 10.0)), None);

        // In spacing gap between cell 0 and cell 1 (x in 20.1..21.9)
        assert_eq!(grid.cell_at(pos2(21.0, 10.0)), None);

        // Multi-cell union_rect spanning (0, 0) to (1, 1)
        let sel = TileRect::from_cells((0, 0), (1, 1));
        let union_rect = sel.union_rect(&grid);
        assert_eq!(union_rect.min, pos2(4.0, 4.0));
        assert_eq!(union_rect.max, pos2(38.0, 38.0));
    }
}
