//! Layer instances, kinds, and entity instances.

use crate::fields::FieldValues;
use crate::grid::Grid;
use crate::id::{EntityDefId, EntityId, LayerDefId, TilesetDefId};
use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

/// Type classification for layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerKind {
    /// Integer-based collision or logic grid.
    IntGrid,
    /// Vector layer containing arbitrary positioned entities.
    Entities,
    /// Tile placement grid referencing tileset cells.
    Tiles,
    /// Reserved rule-based automatic layer (deferred in v1 per ADR-0001).
    Auto,
}

/// Reference to a specific tile in a tileset with transformation flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    /// Source tileset definition ID.
    pub tileset_id: TilesetDefId,
    /// Index of the tile within the tileset.
    pub tile_id: u32,
    /// Horizontally flipped.
    pub flip_x: bool,
    /// Vertically flipped.
    pub flip_y: bool,
}

impl TileRef {
    /// Constructs a simple tile reference with no flipping.
    pub const fn new(tileset_id: TilesetDefId, tile_id: u32) -> Self {
        Self {
            tileset_id,
            tile_id,
            flip_x: false,
            flip_y: false,
        }
    }
}

/// Placed entity instance within an Entities layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityInstance {
    /// Stable entity instance ID.
    pub id: EntityId,
    /// Identifier of the entity template definition.
    pub def_id: EntityDefId,
    /// Position in level pixel coordinates.
    pub pos_px: IVec2,
    /// Dimensions in level pixels.
    pub size_px: UVec2,
    /// Entity-specific field values.
    pub fields: FieldValues,
}

/// Content payload carried by a layer instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayerData {
    /// Integer grid storing values per cell.
    IntGrid(Grid<i32>),
    /// Tile grid storing optional tile references per cell.
    Tiles(Grid<Option<TileRef>>),
    /// Unordered collection of entity instances.
    Entities(Vec<EntityInstance>),
    /// Reserved data payload for auto-layers (deferred in v1).
    Auto,
}

/// Concrete layer instance attached to a level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerInstance {
    /// Identifier of the defining layer template.
    pub def_id: LayerDefId,
    /// Visibility toggle state.
    pub visible: bool,
    /// Layer cell or entity data.
    pub data: LayerData,
}

impl LayerInstance {
    /// Creates a new layer instance.
    pub fn new(def_id: LayerDefId, data: LayerData) -> Self {
        Self {
            def_id,
            visible: true,
            data,
        }
    }

    /// Returns the kind corresponding to the current layer data payload.
    pub fn kind(&self) -> LayerKind {
        match &self.data {
            LayerData::IntGrid(_) => LayerKind::IntGrid,
            LayerData::Tiles(_) => LayerKind::Tiles,
            LayerData::Entities(_) => LayerKind::Entities,
            LayerData::Auto => LayerKind::Auto,
        }
    }
}
