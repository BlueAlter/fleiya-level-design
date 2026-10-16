//! Selection state representing active items in an editor tab.

use glam::IVec2;
use lc_core::id::{EntityId, LayerDefId, LevelId};

/// Active selection within a document canvas.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Selection {
    /// Nothing is selected (e.g. shows level properties in level tab).
    #[default]
    None,
    /// A level itself is selected.
    Level(LevelId),
    /// One or more entity instances are selected.
    Entities(Vec<EntityId>),
    /// A collection of tile/intgrid cells on a layer is selected.
    Tiles {
        /// Target layer definition ID.
        layer_id: LayerDefId,
        /// Selected grid cell coordinates.
        cells: Vec<IVec2>,
    },
}

impl Selection {
    /// Returns true if the selection is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Selection::None => true,
            Selection::Entities(ids) => ids.is_empty(),
            Selection::Tiles { cells, .. } => cells.is_empty(),
            Selection::Level(_) => false,
        }
    }

    /// Returns true if a specific level is selected.
    pub fn is_level(&self, level_id: LevelId) -> bool {
        matches!(self, Selection::Level(id) if *id == level_id)
    }

    /// Returns a slice of entity IDs if entities are selected.
    pub fn entities(&self) -> Option<&[EntityId]> {
        match self {
            Selection::Entities(ids) => Some(ids),
            _ => None,
        }
    }

    /// Returns the single selected entity ID, if exactly one entity is selected.
    pub fn single_entity(&self) -> Option<EntityId> {
        match self {
            Selection::Entities(ids) if ids.len() == 1 => ids.first().copied(),
            _ => None,
        }
    }
}
