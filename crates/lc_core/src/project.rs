//! Top-level project model, settings, and root lookup facilities.

use crate::defs::{Definitions, EntityDef, EnumDef, LayerDef, TilesetDef};
use crate::id::{EntityDefId, EntityId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use crate::layer::LayerData;
use crate::level::Level;
use crate::world::World;
use glam::UVec2;
use serde::{Deserialize, Serialize};

/// Global configuration settings for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default grid sizing for newly created levels or layers.
    pub default_grid_size: UVec2,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            default_grid_size: UVec2::splat(16),
        }
    }
}

/// The single source of truth for all Level Craft workspace data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Schema format version number.
    pub version: u32,
    /// Project title.
    pub name: String,
    /// Shared templates and definitions (tilesets, enums, entities, layers).
    pub defs: Definitions,
    /// Worlds contained in this project.
    pub worlds: Vec<World>,
    /// Global project settings.
    pub settings: ProjectSettings,
    /// Monotonic high-water mark for allocating non-reusable `EntityId` values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_entity_id: Option<u64>,
}

impl Project {
    /// Current format version for new projects.
    pub const CURRENT_VERSION: u32 = 1;

    /// Creates a new empty project with the specified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            name: name.into(),
            defs: Definitions::new(),
            worlds: Vec::new(),
            settings: ProjectSettings::default(),
            next_entity_id: None,
        }
    }

    /// Returns the highest live `EntityId` raw value across all worlds and levels.
    pub fn max_live_entity_id(&self) -> u64 {
        let mut max_id = 0_u64;
        for world in &self.worlds {
            for level in &world.levels {
                for layer in &level.layers {
                    if let LayerData::Entities(entities) = &layer.data {
                        for entity in entities {
                            max_id = max_id.max(entity.id.0);
                        }
                    }
                }
            }
        }
        max_id
    }

    /// Allocates a monotonically increasing `EntityId` that is never reused after deletion.
    pub fn allocate_entity_id(&mut self) -> EntityId {
        let floor = self.max_live_entity_id().saturating_add(1);
        let id = self.next_entity_id.unwrap_or(floor).max(floor);
        self.next_entity_id = Some(id.saturating_add(1));
        EntityId(id)
    }

    /// Looks up a world by ID.
    pub fn world(&self, id: WorldId) -> Option<&World> {
        self.worlds.iter().find(|w| w.id == id)
    }

    /// Looks up a mutable reference to a world by ID.
    pub fn world_mut(&mut self, id: WorldId) -> Option<&mut World> {
        self.worlds.iter_mut().find(|w| w.id == id)
    }

    /// Looks up a level by ID across all worlds in the project.
    pub fn level(&self, id: LevelId) -> Option<&Level> {
        for world in &self.worlds {
            if let Some(level) = world.level(id) {
                return Some(level);
            }
        }
        None
    }

    /// Looks up a mutable reference to a level by ID across all worlds.
    pub fn level_mut(&mut self, id: LevelId) -> Option<&mut Level> {
        for world in &mut self.worlds {
            if let Some(level) = world.level_mut(id) {
                return Some(level);
            }
        }
        None
    }

    /// Looks up both the containing world and the level by level ID.
    pub fn level_and_world(&self, id: LevelId) -> Option<(&World, &Level)> {
        for world in &self.worlds {
            if let Some(level) = world.level(id) {
                return Some((world, level));
            }
        }
        None
    }

    /// Looks up a tileset definition by ID.
    pub fn tileset_def(&self, id: TilesetDefId) -> Option<&TilesetDef> {
        self.defs.tileset(id)
    }

    /// Looks up an enum definition by ID.
    pub fn enum_def(&self, id: EnumDefId) -> Option<&EnumDef> {
        self.defs.enum_def(id)
    }

    /// Looks up an entity definition by ID.
    pub fn entity_def(&self, id: EntityDefId) -> Option<&EntityDef> {
        self.defs.entity(id)
    }

    /// Looks up a layer definition by ID.
    pub fn layer_def(&self, id: LayerDefId) -> Option<&LayerDef> {
        self.defs.layer(id)
    }
}
