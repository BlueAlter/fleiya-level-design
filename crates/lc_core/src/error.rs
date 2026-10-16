//! Typed error definitions for lc_core.

use crate::id::{EntityDefId, EntityId, LayerDefId, LevelId, WorldId};

/// Errors originating from core data model operations and validations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Attempted to access grid coordinates outside its dimensions.
    #[error("Out of bounds: coordinates ({x}, {y}) exceed dimensions ({width}, {height})")]
    OutOfBounds {
        /// X coordinate
        x: u32,
        /// Y coordinate
        y: u32,
        /// Grid width
        width: u32,
        /// Grid height
        height: u32,
    },

    /// The provided raw cell count does not match `width * height`.
    #[error("Invalid grid cell count: expected {expected}, got {actual}")]
    InvalidGridSize {
        /// Expected cell count
        expected: usize,
        /// Actual cell count
        actual: usize,
    },

    /// Entity instance not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),

    /// Entity definition not found.
    #[error("Entity definition not found: {0}")]
    EntityDefNotFound(EntityDefId),

    /// Level not found.
    #[error("Level not found: {0}")]
    LevelNotFound(LevelId),

    /// World not found.
    #[error("World not found: {0}")]
    WorldNotFound(WorldId),

    /// Layer definition not found.
    #[error("Layer not found: {0}")]
    LayerNotFound(LayerDefId),

    /// Generic definition lookup failure.
    #[error("Definition not found: {0}")]
    DefNotFound(String),
}
