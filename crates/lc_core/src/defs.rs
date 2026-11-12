//! Project-level definitions and templates.

use crate::fields::FieldDef;
use crate::id::{EntityDefId, EnumDefId, LayerDefId, TilesetDefId};
use crate::layer::{LayerKind, TileRef};
use glam::{UVec2, Vec2};
use serde::{Deserialize, Serialize};

/// Definition of an external tileset asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilesetDef {
    /// Stable identifier of the tileset.
    pub id: TilesetDefId,
    /// Human-readable display name.
    pub name: String,
    /// Relative path to the image file from project root.
    pub path: String,
    /// Size of an individual tile in pixels.
    pub tile_size: UVec2,
    /// Spacing between adjacent tiles in pixels.
    pub spacing: u32,
    /// Padding around the tileset border in pixels.
    pub padding: u32,
}

/// An individual value option within an `EnumDef`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValueDef {
    /// Unique identifier key for this value (e.g. "North", "Desert").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Associated UI/badge color in RGBA.
    pub color: [u8; 4],
    /// Optional tile icon representation.
    pub tile: Option<TileRef>,
}

/// Definition of a custom enumerated type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDef {
    /// Stable identifier of the enum.
    pub id: EnumDefId,
    /// Name of the enumeration (e.g. "Biome", "Direction").
    pub name: String,
    /// Optional tileset bound for value tile icons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tileset_id: Option<TilesetDefId>,
    /// Tags assigned to this enum definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Ordered list of possible enum values.
    pub values: Vec<EnumValueDef>,
}

impl EnumDef {
    /// Constructs a new empty enum definition.
    pub fn new(id: EnumDefId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            tileset_id: None,
            tags: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Finds a value definition by its string ID.
    pub fn value(&self, id: &str) -> Option<&EnumValueDef> {
        self.values.iter().find(|v| v.id == id)
    }

    /// Finds a mutable reference to a value definition by its string ID.
    pub fn value_mut(&mut self, id: &str) -> Option<&mut EnumValueDef> {
        self.values.iter_mut().find(|v| v.id == id)
    }
}

/// Definition of a value within an IntGrid layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntGridValueDef {
    /// Integer value stored in the grid cells.
    pub value: i32,
    /// Human-readable label (e.g. "Solid", "Ladder").
    pub name: String,
    /// Color representation in the editor.
    pub color: [u8; 4],
}

/// Definition of a layer template in the project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerDef {
    /// Stable identifier of the layer definition.
    pub id: LayerDefId,
    /// Human-readable name (e.g. "Collisions", "Background").
    pub name: String,
    /// Type of layer.
    pub kind: LayerKind,
    /// Grid cell dimensions in pixels.
    pub grid_size: UVec2,
    /// Optional tileset reference (for Tiles or IntGrid with auto-rules).
    pub tileset_id: Option<TilesetDefId>,
    /// Configured integer values if this is an IntGrid layer.
    pub int_grid_values: Vec<IntGridValueDef>,
}

/// Definition of an entity template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDef {
    /// Stable identifier of the entity definition.
    pub id: EntityDefId,
    /// Entity class name (e.g. "Player", "Coin").
    pub name: String,
    /// Default size in pixels.
    pub size_px: UVec2,
    /// Display color in the editor.
    pub color: [u8; 4],
    /// Alignment pivot point [0.0..1.0, 0.0..1.0].
    pub pivot: Vec2,
    /// Optional tile representation for display in canvas.
    pub tile: Option<TileRef>,
    /// Custom field schema definitions.
    pub fields: Vec<FieldDef>,
}

impl EntityDef {
    /// Finds a field definition by its name.
    pub fn field_def(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }
}

/// Aggregation of all project definitions and templates.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Definitions {
    /// Tileset definitions.
    pub tilesets: Vec<TilesetDef>,
    /// Enum definitions.
    pub enums: Vec<EnumDef>,
    /// Entity definitions.
    pub entities: Vec<EntityDef>,
    /// Layer definitions.
    pub layers: Vec<LayerDef>,
}

impl Definitions {
    /// Creates an empty collection of definitions.
    pub const fn new() -> Self {
        Self {
            tilesets: Vec::new(),
            enums: Vec::new(),
            entities: Vec::new(),
            layers: Vec::new(),
        }
    }

    /// Looks up a tileset definition by ID.
    pub fn tileset(&self, id: TilesetDefId) -> Option<&TilesetDef> {
        self.tilesets.iter().find(|t| t.id == id)
    }

    /// Looks up a mutable tileset definition by ID.
    pub fn tileset_mut(&mut self, id: TilesetDefId) -> Option<&mut TilesetDef> {
        self.tilesets.iter_mut().find(|t| t.id == id)
    }

    /// Looks up an enum definition by ID.
    pub fn enum_def(&self, id: EnumDefId) -> Option<&EnumDef> {
        self.enums.iter().find(|e| e.id == id)
    }

    /// Looks up a mutable enum definition by ID.
    pub fn enum_def_mut(&mut self, id: EnumDefId) -> Option<&mut EnumDef> {
        self.enums.iter_mut().find(|e| e.id == id)
    }

    /// Looks up an entity definition by ID.
    pub fn entity(&self, id: EntityDefId) -> Option<&EntityDef> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Looks up a mutable entity definition by ID.
    pub fn entity_mut(&mut self, id: EntityDefId) -> Option<&mut EntityDef> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Looks up a layer definition by ID.
    pub fn layer(&self, id: LayerDefId) -> Option<&LayerDef> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Looks up a mutable layer definition by ID.
    pub fn layer_mut(&mut self, id: LayerDefId) -> Option<&mut LayerDef> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
}
