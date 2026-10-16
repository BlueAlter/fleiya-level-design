//! Stable identifier types and deterministic ID generation for Level Craft.

use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            /// Creates a new identifier wrapping the specified raw u64 value.
            pub const fn new(val: u64) -> Self {
                Self(val)
            }

            /// Returns the underlying raw u64 value.
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_id!(ProjectId, "Stable identifier for a Project.");
define_id!(WorldId, "Stable identifier for a World.");
define_id!(LevelId, "Stable identifier for a Level.");
define_id!(LayerDefId, "Stable identifier for a Layer definition.");
define_id!(EntityDefId, "Stable identifier for an Entity definition.");
define_id!(EnumDefId, "Stable identifier for an Enum definition.");
define_id!(TilesetDefId, "Stable identifier for a Tileset definition.");
define_id!(EntityId, "Stable identifier for an Entity instance.");

/// Deterministic generator for unique identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGenerator {
    next_raw: u64,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator {
    /// Creates a new ID generator starting at 1.
    pub const fn new() -> Self {
        Self { next_raw: 1 }
    }

    /// Creates an ID generator starting from a specific seed.
    pub const fn with_seed(seed: u64) -> Self {
        Self { next_raw: seed }
    }

    /// Generates the next raw u64 value.
    pub fn next_raw(&mut self) -> u64 {
        let id = self.next_raw;
        self.next_raw = self.next_raw.saturating_add(1);
        id
    }

    /// Generates the next `ProjectId`.
    pub fn next_project_id(&mut self) -> ProjectId {
        ProjectId(self.next_raw())
    }

    /// Generates the next `WorldId`.
    pub fn next_world_id(&mut self) -> WorldId {
        WorldId(self.next_raw())
    }

    /// Generates the next `LevelId`.
    pub fn next_level_id(&mut self) -> LevelId {
        LevelId(self.next_raw())
    }

    /// Generates the next `LayerDefId`.
    pub fn next_layer_def_id(&mut self) -> LayerDefId {
        LayerDefId(self.next_raw())
    }

    /// Generates the next `EntityDefId`.
    pub fn next_entity_def_id(&mut self) -> EntityDefId {
        EntityDefId(self.next_raw())
    }

    /// Generates the next `EnumDefId`.
    pub fn next_enum_def_id(&mut self) -> EnumDefId {
        EnumDefId(self.next_raw())
    }

    /// Generates the next `TilesetDefId`.
    pub fn next_tileset_def_id(&mut self) -> TilesetDefId {
        TilesetDefId(self.next_raw())
    }

    /// Generates the next `EntityId`.
    pub fn next_entity_id(&mut self) -> EntityId {
        EntityId(self.next_raw())
    }
}
