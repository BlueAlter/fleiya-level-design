//! Shared editor session state holding transient UI focus, active tools, and selections.

use crate::selection::Selection;
use lc_core::command::DocKind;
use lc_core::id::{EntityId, LayerDefId, LevelId, TilesetDefId};
use lc_core::layer::TileRef;

/// Interactive canvas picking request initiated from the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasPickMode {
    /// Pick a grid/pixel coordinate on the level canvas for a `Point` field.
    Point {
        level_id: LevelId,
        entity_id: EntityId,
        field_name: String,
        array_index: Option<usize>,
    },
    /// Pick an entity instance on the level canvas for an `EntityRef` field.
    EntityRef {
        level_id: LevelId,
        entity_id: EntityId,
        field_name: String,
        array_index: Option<usize>,
    },
}

/// Selected tile pattern from the tile palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSelection {
    /// ID of the tileset containing the selected tiles.
    pub tileset_id: TilesetDefId,
    /// Linear tile indices selected.
    pub tiles: Vec<u32>,
    /// Width of the selected rectangular block in tiles (default 1).
    pub width: u32,
}

impl TileSelection {
    /// Creates a selection of a single tile.
    pub fn single(tileset_id: TilesetDefId, tile_id: u32) -> Self {
        Self {
            tileset_id,
            tiles: vec![tile_id],
            width: 1,
        }
    }

    /// Creates a rectangular selection of tiles.
    pub fn rect(tileset_id: TilesetDefId, tiles: Vec<u32>, width: u32) -> Self {
        Self {
            tileset_id,
            tiles,
            width: width.max(1),
        }
    }

    /// Returns the primary/first selected tile reference, if any.
    pub fn primary_tile(&self) -> Option<TileRef> {
        self.tiles
            .first()
            .map(|&id| TileRef::new(self.tileset_id, id))
    }
}

/// Shared editor session state holding transient UI focus, active tools, and selections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    /// Active document currently focused in the central dock area.
    pub active_document: Option<DocKind>,
    /// Active level ID being edited (either directly focused or selected).
    pub active_level: Option<LevelId>,
    /// Active layer definition ID currently selected for editing/inspection.
    pub active_layer: Option<LayerDefId>,
    /// Current selection in the active document canvas.
    pub selection: Selection,
    /// Currently selected canvas tool name (e.g., "Select", "Paint", "Erase").
    pub active_tool_name: String,
    /// Current tile palette selection (tileset + selected tile IDs).
    pub tile_selection: Option<TileSelection>,
    /// Active canvas picking mode initiated by the inspector, if any.
    pub pick_mode: Option<CanvasPickMode>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            active_document: None,
            active_level: None,
            active_layer: None,
            selection: Selection::None,
            active_tool_name: "Select".to_string(),
            tile_selection: None,
            pick_mode: None,
        }
    }
}
