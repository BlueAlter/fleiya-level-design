//! Geometry helpers shared by the tile painting, rectangle, and selection tools.

use glam::{IVec2, UVec2};
use lc_core::layer::TileRef;
use lc_ui_api::session::TileSelection;

/// Expands a rectangular palette selection into absolute grid cells anchored at `origin`.
pub fn stamp_cells(
    ts: &TileSelection,
    origin: UVec2,
    flip_x: bool,
    flip_y: bool,
) -> Vec<(u32, u32, Option<TileRef>)> {
    let width = ts.width.max(1);
    ts.tiles
        .iter()
        .enumerate()
        .map(|(index, &tile_id)| {
            let dx = (index as u32) % width;
            let dy = (index as u32) / width;
            let mut tile_ref = TileRef::new(ts.tileset_id, tile_id);
            tile_ref.flip_x = flip_x;
            tile_ref.flip_y = flip_y;
            (origin.x + dx, origin.y + dy, Some(tile_ref))
        })
        .collect()
}

/// Expands a rectangular palette selection into flat tile refs, preserving row-major order.
pub fn pattern_tiles(ts: &TileSelection, flip_x: bool, flip_y: bool) -> Vec<TileRef> {
    ts.tiles
        .iter()
        .map(|&tile_id| {
            let mut tile_ref = TileRef::new(ts.tileset_id, tile_id);
            tile_ref.flip_x = flip_x;
            tile_ref.flip_y = flip_y;
            tile_ref
        })
        .collect()
}

/// Converts a rectangular grid drag into a row-major list of cell coordinates.
pub fn rect_cells(a: UVec2, b: UVec2) -> Vec<IVec2> {
    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    let mut cells = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            cells.push(IVec2::new(x as i32, y as i32));
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::id::TilesetDefId;

    fn selection(width: u32, tiles: &[u32]) -> TileSelection {
        TileSelection::rect(TilesetDefId::new(1), tiles.to_vec(), width)
    }

    #[test]
    fn stamp_cells_are_offset_by_selection_width() {
        let ts = selection(2, &[10, 11, 12, 13]);
        let cells = stamp_cells(&ts, UVec2::new(5, 7), false, false);
        let coords: Vec<(u32, u32, u32)> = cells
            .iter()
            .map(|(x, y, tile)| (*x, *y, tile.expect("tile").tile_id))
            .collect();
        assert_eq!(coords, vec![(5, 7, 10), (6, 7, 11), (5, 8, 12), (6, 8, 13)]);
    }

    #[test]
    fn stamp_cells_carry_flip_flags() {
        let ts = selection(1, &[3]);
        let cells = stamp_cells(&ts, UVec2::ZERO, true, true);
        let tile = cells[0].2.expect("tile");
        assert!(tile.flip_x);
        assert!(tile.flip_y);
    }

    #[test]
    fn rect_cells_normalize_corners_and_keep_row_major_order() {
        let cells = rect_cells(UVec2::new(3, 4), UVec2::new(1, 2));
        assert_eq!(
            cells,
            vec![
                IVec2::new(1, 2),
                IVec2::new(2, 2),
                IVec2::new(3, 2),
                IVec2::new(1, 3),
                IVec2::new(2, 3),
                IVec2::new(3, 3),
                IVec2::new(1, 4),
                IVec2::new(2, 4),
                IVec2::new(3, 4),
            ]
        );
    }

    #[test]
    fn pattern_tiles_preserve_row_major_order_and_flips() {
        let ts = selection(2, &[1, 2]);
        let tiles = pattern_tiles(&ts, true, false);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].tile_id, 1);
        assert_eq!(tiles[1].tile_id, 2);
        assert!(tiles.iter().all(|t| t.flip_x && !t.flip_y));
    }
}
