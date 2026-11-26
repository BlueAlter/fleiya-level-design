//! Project-level definitions and templates.

pub use crate::collision::{CollisionShape, CollisionShapeGeometry, TileCollisionDef};
pub use crate::entity::{
    EntityDef, EntityLimitBehavior, EntityLimitScope, EntityRenderMode, TileRenderMode, TilesetRect,
};
use crate::id::{EntityDefId, EnumDefId, LayerDefId, TilesetDefId};
use crate::layer::{LayerKind, TileRef};
pub use crate::marking::TileMarking;
use glam::UVec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Definition of an external tileset asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Tags assigned to this tileset definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Tile marking configuration (e.g. source enum).
    #[serde(default, skip_serializing_if = "TileMarking::is_empty")]
    pub marking: TileMarking,
    /// Custom string data associated per tile (e.g. JSON, XML, strings).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub custom_data: BTreeMap<u32, String>,
    /// Tile animation definitions keyed by anchor tile ID.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub animations: BTreeMap<u32, TileAnimationDef>,
    /// Tile collision definitions keyed by tile ID.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collisions: BTreeMap<u32, TileCollisionDef>,
}

fn default_frame_duration_ms() -> u32 {
    100
}

/// Playback mode for tile animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationPlayMode {
    #[default]
    Loop,
    PingPong,
    Once,
}

/// Definition of a tile animation tied to an anchor tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileAnimationDef {
    /// ID of the anchor tile representing this animation in maps and the atlas.
    pub anchor_tile_id: u32,
    /// Ordered list of tile IDs representing consecutive animation frames.
    pub frames: Vec<u32>,
    /// Duration of each frame in milliseconds.
    #[serde(default = "default_frame_duration_ms")]
    pub frame_duration_ms: u32,
    /// Playback mode: Loop, PingPong, or Once.
    #[serde(default)]
    pub mode: AnimationPlayMode,
    /// Whether instances have randomized starting frame offsets.
    #[serde(default)]
    pub random_start: bool,
}

impl TileAnimationDef {
    /// Creates a new tile animation with the anchor tile as its initial frame.
    pub fn new(anchor_tile_id: u32) -> Self {
        Self {
            anchor_tile_id,
            frames: vec![anchor_tile_id],
            frame_duration_ms: 100,
            mode: AnimationPlayMode::Loop,
            random_start: false,
        }
    }

    /// Computes the active frame tile ID given elapsed milliseconds.
    pub fn active_frame(&self, elapsed_ms: u64) -> u32 {
        if self.frames.is_empty() {
            return self.anchor_tile_id;
        }
        if self.frames.len() == 1 {
            return self.frames.first().copied().unwrap_or(self.anchor_tile_id);
        }

        let dur = (self.frame_duration_ms.max(1)) as u64;
        let total_frames = self.frames.len();

        match self.mode {
            AnimationPlayMode::Loop => {
                let frame_idx = ((elapsed_ms / dur) % total_frames as u64) as usize;
                self.frames
                    .get(frame_idx)
                    .copied()
                    .unwrap_or(self.anchor_tile_id)
            }
            AnimationPlayMode::PingPong => {
                let cycle_len = (total_frames * 2).saturating_sub(2).max(1);
                let cycle_idx = ((elapsed_ms / dur) % cycle_len as u64) as usize;
                if cycle_idx < total_frames {
                    self.frames
                        .get(cycle_idx)
                        .copied()
                        .unwrap_or(self.anchor_tile_id)
                } else {
                    let back_idx = cycle_len - cycle_idx;
                    self.frames
                        .get(back_idx)
                        .copied()
                        .unwrap_or(self.anchor_tile_id)
                }
            }
            AnimationPlayMode::Once => {
                let frame_idx = ((elapsed_ms / dur) as usize).min(total_frames - 1);
                self.frames
                    .get(frame_idx)
                    .copied()
                    .unwrap_or(self.anchor_tile_id)
            }
        }
    }
}

impl Default for TilesetDef {
    fn default() -> Self {
        Self {
            id: TilesetDefId(0),
            name: String::new(),
            path: String::new(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: Vec::new(),
            marking: TileMarking::new(),
            custom_data: BTreeMap::new(),
            animations: BTreeMap::new(),
            collisions: BTreeMap::new(),
        }
    }
}

impl TilesetDef {
    /// Constructs a new tileset definition with default tags and marking.
    pub fn new(
        id: TilesetDefId,
        name: impl Into<String>,
        path: impl Into<String>,
        tile_size: UVec2,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            path: path.into(),
            tile_size,
            spacing: 0,
            padding: 0,
            tags: Vec::new(),
            marking: TileMarking::new(),
            custom_data: BTreeMap::new(),
            animations: BTreeMap::new(),
            collisions: BTreeMap::new(),
        }
    }

    /// Returns the animation definition for an anchor tile, if one exists.
    pub fn animation(&self, anchor: u32) -> Option<&TileAnimationDef> {
        self.animations.get(&anchor)
    }

    /// Returns a mutable reference to the animation definition for an anchor tile.
    pub fn animation_mut(&mut self, anchor: u32) -> Option<&mut TileAnimationDef> {
        self.animations.get_mut(&anchor)
    }

    /// Returns true if the tile ID is configured as an animation anchor.
    pub fn is_anchor(&self, tile_id: u32) -> bool {
        self.animations.contains_key(&tile_id)
    }

    /// Finds which anchor an animation frame belongs to, if any.
    pub fn anchor_for_frame(&self, frame_tile_id: u32) -> Option<u32> {
        for (anchor, anim) in &self.animations {
            if anim.frames.contains(&frame_tile_id) {
                return Some(*anchor);
            }
        }
        None
    }

    /// Returns the collision definition for a tile ID, if configured.
    pub fn collision(&self, tile_id: u32) -> Option<&TileCollisionDef> {
        self.collisions.get(&tile_id)
    }

    /// Returns a mutable reference to the collision definition for a tile ID.
    pub fn collision_mut(&mut self, tile_id: u32) -> Option<&mut TileCollisionDef> {
        self.collisions.get_mut(&tile_id)
    }

    /// Returns true if the given tile ID has collision shapes defined.
    pub fn has_collision(&self, tile_id: u32) -> bool {
        self.collisions.get(&tile_id).is_some_and(|c| !c.is_empty())
    }
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

    /// Snapshots tile markings for all tilesets using the specified enum as their source.
    pub fn marking_snapshots(&self, enum_id: EnumDefId) -> Vec<(TilesetDefId, TileMarking)> {
        self.tilesets
            .iter()
            .filter(|t| t.marking.source_enum == Some(enum_id))
            .map(|t| (t.id, t.marking.clone()))
            .collect()
    }

    /// Restores tile markings on tilesets matching the snapshots.
    pub fn restore_markings(&mut self, snapshots: &[(TilesetDefId, TileMarking)]) {
        for (id, marking) in snapshots {
            if let Some(t) = self.tileset_mut(*id) {
                t.marking = marking.clone();
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_animation_loop() {
        let mut anim = TileAnimationDef::new(10);
        anim.frames = vec![10, 11, 12];
        anim.frame_duration_ms = 100;
        anim.mode = AnimationPlayMode::Loop;

        assert_eq!(anim.active_frame(0), 10);
        assert_eq!(anim.active_frame(50), 10);
        assert_eq!(anim.active_frame(100), 11);
        assert_eq!(anim.active_frame(199), 11);
        assert_eq!(anim.active_frame(200), 12);
        assert_eq!(anim.active_frame(300), 10);
        assert_eq!(anim.active_frame(450), 11);
    }

    #[test]
    fn test_tile_animation_ping_pong() {
        let mut anim = TileAnimationDef::new(10);
        anim.frames = vec![10, 11, 12];
        anim.frame_duration_ms = 100;
        anim.mode = AnimationPlayMode::PingPong;

        // Cycle order: 10 -> 11 -> 12 -> 11 -> 10 -> ...
        assert_eq!(anim.active_frame(0), 10);
        assert_eq!(anim.active_frame(100), 11);
        assert_eq!(anim.active_frame(200), 12);
        assert_eq!(anim.active_frame(300), 11);
        assert_eq!(anim.active_frame(400), 10);
        assert_eq!(anim.active_frame(500), 11);
    }

    #[test]
    fn test_tile_animation_once() {
        let mut anim = TileAnimationDef::new(10);
        anim.frames = vec![10, 11, 12];
        anim.frame_duration_ms = 100;
        anim.mode = AnimationPlayMode::Once;

        assert_eq!(anim.active_frame(0), 10);
        assert_eq!(anim.active_frame(100), 11);
        assert_eq!(anim.active_frame(200), 12);
        assert_eq!(anim.active_frame(9999), 12);
    }

    #[test]
    fn test_tileset_animation_lookup() {
        let mut ts = TilesetDef::default();
        let mut anim = TileAnimationDef::new(5);
        anim.frames = vec![5, 6, 7];
        ts.animations.insert(5, anim);

        assert!(ts.is_anchor(5));
        assert!(!ts.is_anchor(6));
        assert_eq!(ts.anchor_for_frame(5), Some(5));
        assert_eq!(ts.anchor_for_frame(6), Some(5));
        assert_eq!(ts.anchor_for_frame(7), Some(5));
        assert_eq!(ts.anchor_for_frame(8), None);
    }
}
