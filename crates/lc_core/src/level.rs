//! Level data model and instance management.

use crate::fields::FieldValues;
use crate::id::{EntityId, LayerDefId, LevelId};
use crate::layer::{EntityInstance, LayerData, LayerInstance};
use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

/// An individual level containing positioned layers and custom fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Level {
    /// Stable identifier for this level.
    pub id: LevelId,
    /// Name of the level (e.g. "Level_0", "BossArena").
    pub name: String,
    /// Position within the world coordinate plane.
    pub world_pos: IVec2,
    /// Z-depth ordering in the world (overlapping levels at different depths are supported).
    pub world_depth: i32,
    /// Dimensions of the level in pixels.
    pub size_px: UVec2,
    /// Ordered list of layers (index corresponds to render order).
    pub layers: Vec<LayerInstance>,
    /// Level-specific custom field values.
    pub fields: FieldValues,
}

impl Level {
    /// Creates a new level with empty layers and fields.
    pub fn new(id: LevelId, name: impl Into<String>, size_px: UVec2) -> Self {
        Self {
            id,
            name: name.into(),
            world_pos: IVec2::ZERO,
            world_depth: 0,
            size_px,
            layers: Vec::new(),
            fields: FieldValues::new(),
        }
    }

    /// Looks up a layer instance by its definition ID.
    pub fn layer(&self, def_id: LayerDefId) -> Option<&LayerInstance> {
        self.layers.iter().find(|l| l.def_id == def_id)
    }

    /// Looks up a mutable layer instance by its definition ID.
    pub fn layer_mut(&mut self, def_id: LayerDefId) -> Option<&mut LayerInstance> {
        self.layers.iter_mut().find(|l| l.def_id == def_id)
    }

    /// Looks up an entity instance by ID across all Entities layers in this level.
    pub fn entity(&self, id: EntityId) -> Option<&EntityInstance> {
        for layer in &self.layers {
            if let LayerData::Entities(entities) = &layer.data {
                if let Some(entity) = entities.iter().find(|e| e.id == id) {
                    return Some(entity);
                }
            }
        }
        None
    }

    /// Looks up a mutable entity instance by ID across all Entities layers in this level.
    pub fn entity_mut(&mut self, id: EntityId) -> Option<&mut EntityInstance> {
        for layer in &mut self.layers {
            if let LayerData::Entities(entities) = &mut layer.data {
                if let Some(entity) = entities.iter_mut().find(|e| e.id == id) {
                    return Some(entity);
                }
            }
        }
        None
    }

    /// Iterates over all entity instances across all Entities layers.
    pub fn iter_entities(&self) -> impl Iterator<Item = &EntityInstance> {
        self.layers
            .iter()
            .filter_map(|l| match &l.data {
                LayerData::Entities(entities) => Some(entities.as_slice()),
                _ => None,
            })
            .flatten()
    }
}
