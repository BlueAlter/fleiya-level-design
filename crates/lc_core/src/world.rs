//! World organization and multi-level layout modes.

use crate::id::{LevelId, WorldId};
use crate::level::Level;
use glam::UVec2;
use serde::{Deserialize, Serialize};

/// Arrangement layout mode for levels within a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldLayout {
    /// Arbitrary 2D positioning (freeform coordinates).
    Free,
    /// Grid-aligned positioning (Metroidvania / room-based grid snap).
    GridVania,
    /// Levels ordered along a horizontal line.
    LinearHorizontal,
    /// Levels ordered along a vertical line.
    LinearVertical,
}

/// A world containing a collection of positioned levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct World {
    /// Stable identifier for this world.
    pub id: WorldId,
    /// Display name (e.g. "Overworld", "Nether").
    pub name: String,
    /// Level arrangement layout style.
    pub layout: WorldLayout,
    /// Default grid sizing for GridVania or level snapping.
    pub default_grid: Option<UVec2>,
    /// Levels belonging to this world.
    pub levels: Vec<Level>,
}

impl World {
    /// Creates a new world with default Free layout.
    pub fn new(id: WorldId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            layout: WorldLayout::Free,
            default_grid: None,
            levels: Vec::new(),
        }
    }

    /// Looks up a level by its ID.
    pub fn level(&self, id: LevelId) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == id)
    }

    /// Looks up a mutable reference to a level by its ID.
    pub fn level_mut(&mut self, id: LevelId) -> Option<&mut Level> {
        self.levels.iter_mut().find(|l| l.id == id)
    }
}
