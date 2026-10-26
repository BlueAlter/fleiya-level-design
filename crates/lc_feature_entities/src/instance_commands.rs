//! Commands for entity instances on Entities layers: place, move, resize,
//! delete, duplicate and edit field values.
//!
//! Moving, resizing and field edits accept an optional stroke id. Commands
//! sharing a stroke merge into one undo step, which is how a continuous drag
//! or a typing session stays a single history entry.

use glam::{IVec2, UVec2};
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{EntityId, LayerDefId, LevelId};
use lc_core::layer::{EntityInstance, LayerData};
use lc_core::project::Project;

use crate::helpers::find_entity_location;
use crate::refs::{restore_cleared_refs, scrub_dangling_refs, ClearedRef};

/// Adds one or more prepared entity instances to an Entities layer.
///
/// Placement and duplication both build the instances (stable ids, defaults,
/// offset positions) before executing this command.
#[derive(Debug, Clone)]
pub struct AddEntityInstancesCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    instances: Vec<EntityInstance>,
}

impl AddEntityInstancesCommand {
    /// Creates a command adding `instances` to `layer_def_id`.
    pub fn new(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        instances: Vec<EntityInstance>,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            instances,
        }
    }

    /// The instances this command adds.
    pub fn instances(&self) -> &[EntityInstance] {
        &self.instances
    }
}

impl Command for AddEntityInstancesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if self.instances.is_empty() {
            return Ok(());
        }
        for instance in &self.instances {
            if find_entity_location(p, instance.id).is_some() {
                return Err(CoreError::DefNotFound(format!(
                    "Entity instance {} already exists",
                    instance.id.0
                )));
            }
        }

        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;
        let LayerData::Entities(entities) = &mut layer.data else {
            return Err(CoreError::DefNotFound(
                "Target layer is not an Entities layer".to_string(),
            ));
        };

        for instance in &self.instances {
            entities.push(instance.clone());
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(level) = p.level_mut(self.level_id) else {
            return;
        };
        let Some(layer) = level.layer_mut(self.layer_def_id) else {
            return;
        };
        let LayerData::Entities(entities) = &mut layer.data else {
            return;
        };
        for instance in self.instances.iter().rev() {
            if let Some(pos) = entities.iter().position(|e| e.id == instance.id) {
                entities.remove(pos);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        match self.instances.len() {
            1 => "Place Entity".to_string(),
            n => format!("Place {n} Entities"),
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
struct RemovedInstance {
    layer_def_id: LayerDefId,
    index: usize,
    entity: EntityInstance,
}

/// Deletes entity instances and clears weak references to them.
#[derive(Debug, Clone)]
pub struct RemoveEntityInstancesCommand {
    level_id: LevelId,
    ids: Vec<EntityId>,
    removed: Vec<RemovedInstance>,
    cleared_refs: Vec<ClearedRef>,
}

impl RemoveEntityInstancesCommand {
    /// Creates a command deleting `ids` from `level_id`.
    pub fn new(level_id: LevelId, ids: Vec<EntityId>) -> Self {
        Self {
            level_id,
            ids,
            removed: Vec::new(),
            cleared_refs: Vec::new(),
        }
    }
}

impl Command for RemoveEntityInstancesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        self.removed.clear();
        self.cleared_refs.clear();

        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        for layer in &mut level.layers {
            if let LayerData::Entities(entities) = &mut layer.data {
                let mut index = entities.len();
                while index > 0 {
                    index -= 1;
                    if self.ids.contains(&entities[index].id) {
                        let entity = entities.remove(index);
                        self.removed.push(RemovedInstance {
                            layer_def_id: layer.def_id,
                            index,
                            entity,
                        });
                    }
                }
            }
        }

        if self.removed.is_empty() {
            return Ok(());
        }

        let removed_ids: Vec<EntityId> = self.removed.iter().map(|r| r.entity.id).collect();
        scrub_dangling_refs(p, &removed_ids, &mut self.cleared_refs);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(level) = p.level_mut(self.level_id) else {
            return;
        };
        for removed in self.removed.drain(..).rev() {
            let Some(layer) = level.layer_mut(removed.layer_def_id) else {
                continue;
            };
            if let LayerData::Entities(entities) = &mut layer.data {
                if removed.index <= entities.len() {
                    entities.insert(removed.index, removed.entity);
                } else {
                    entities.push(removed.entity);
                }
            }
        }
        restore_cleared_refs(p, &mut self.cleared_refs);
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        match self.removed.len() {
            0 | 1 => "Delete Entity".to_string(),
            n => format!("Delete {n} Entities"),
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Moves entity instances to absolute level-local pixel positions.
#[derive(Debug, Clone)]
pub struct SetEntityPositionsCommand {
    level_id: LevelId,
    positions: Vec<(EntityId, IVec2)>,
    previous: Vec<(EntityId, IVec2)>,
    stroke_id: Option<u64>,
}

impl SetEntityPositionsCommand {
    /// Creates a command moving entities to `positions`.
    pub fn new(level_id: LevelId, positions: Vec<(EntityId, IVec2)>) -> Self {
        Self {
            level_id,
            positions,
            previous: Vec::new(),
            stroke_id: None,
        }
    }

    /// Creates a command that merges consecutive moves sharing `stroke_id`.
    pub fn with_stroke(
        level_id: LevelId,
        positions: Vec<(EntityId, IVec2)>,
        stroke_id: u64,
    ) -> Self {
        Self {
            stroke_id: Some(stroke_id),
            ..Self::new(level_id, positions)
        }
    }
}

impl Command for SetEntityPositionsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        if self.previous.is_empty() {
            for (id, _) in &self.positions {
                if let Some(entity) = level.entity(*id) {
                    self.previous.push((*id, entity.pos_px));
                }
            }
        }
        for (id, pos) in &self.positions {
            if let Some(entity) = level.entity_mut(*id) {
                entity.pos_px = *pos;
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(level) = p.level_mut(self.level_id) else {
            return;
        };
        for (id, pos) in &self.previous {
            if let Some(entity) = level.entity_mut(*id) {
                entity.pos_px = *pos;
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        let Some(other) = next.as_any().downcast_ref::<Self>() else {
            return false;
        };
        if self.level_id == other.level_id
            && self.stroke_id.is_some()
            && self.stroke_id == other.stroke_id
            && same_ids(&self.positions, &other.positions)
        {
            self.positions = other.positions.clone();
            return true;
        }
        false
    }

    fn label(&self) -> String {
        match self.positions.len() {
            1 => "Move Entity".to_string(),
            n => format!("Move {n} Entities"),
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Resizes entity instances to absolute pixel sizes.
#[derive(Debug, Clone)]
pub struct SetEntitySizesCommand {
    level_id: LevelId,
    sizes: Vec<(EntityId, UVec2)>,
    previous: Vec<(EntityId, UVec2)>,
    stroke_id: Option<u64>,
}

impl SetEntitySizesCommand {
    /// Creates a command resizing entities to `sizes`.
    pub fn new(level_id: LevelId, sizes: Vec<(EntityId, UVec2)>) -> Self {
        Self {
            level_id,
            sizes,
            previous: Vec::new(),
            stroke_id: None,
        }
    }

    /// Creates a command that merges consecutive resizes sharing `stroke_id`.
    pub fn with_stroke(level_id: LevelId, sizes: Vec<(EntityId, UVec2)>, stroke_id: u64) -> Self {
        Self {
            stroke_id: Some(stroke_id),
            ..Self::new(level_id, sizes)
        }
    }
}

impl Command for SetEntitySizesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        if self.previous.is_empty() {
            for (id, _) in &self.sizes {
                if let Some(entity) = level.entity(*id) {
                    self.previous.push((*id, entity.size_px));
                }
            }
        }
        for (id, size) in &self.sizes {
            if let Some(entity) = level.entity_mut(*id) {
                entity.size_px = *size;
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(level) = p.level_mut(self.level_id) else {
            return;
        };
        for (id, size) in &self.previous {
            if let Some(entity) = level.entity_mut(*id) {
                entity.size_px = *size;
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        let Some(other) = next.as_any().downcast_ref::<Self>() else {
            return false;
        };
        if self.level_id == other.level_id
            && self.stroke_id.is_some()
            && self.stroke_id == other.stroke_id
            && same_ids(&self.sizes, &other.sizes)
        {
            self.sizes = other.sizes.clone();
            return true;
        }
        false
    }

    fn label(&self) -> String {
        match self.sizes.len() {
            1 => "Resize Entity".to_string(),
            n => format!("Resize {n} Entities"),
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn same_ids<T>(a: &[(EntityId, T)], b: &[(EntityId, T)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(left, right)| left.0 == right.0)
}
