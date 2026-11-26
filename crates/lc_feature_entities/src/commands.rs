//! Commands for entity definition metadata and lifecycle.

use lc_core::command::{Command, DocKind};
use lc_core::defs::EntityDef;
use lc_core::error::CoreError;
use lc_core::id::{EntityDefId, LayerDefId, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData};
use lc_core::project::Project;

use crate::helpers::{count_entity_def_instances, find_entity_def_instances};
use crate::refs::{restore_cleared_refs, scrub_dangling_refs, ClearedRef};

#[derive(Debug, Clone)]
struct RemovedInstance {
    world_id: WorldId,
    level_id: LevelId,
    layer_def_id: LayerDefId,
    index: usize,
    entity: EntityInstance,
}

#[derive(Debug, Clone)]
pub struct AddEntityDefCommand {
    def: EntityDef,
    inserted_index: Option<usize>,
}

impl AddEntityDefCommand {
    /// Creates a command adding `def`.
    pub fn new(def: EntityDef) -> Self {
        Self {
            def,
            inserted_index: None,
        }
    }

    /// The definition this command adds.
    pub fn def(&self) -> &EntityDef {
        &self.def
    }
}

impl Command for AddEntityDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if p.defs.entity(self.def.id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "EntityDef with id {:?} already exists",
                self.def.id
            )));
        }
        let idx = p.defs.entities.len();
        p.defs.entities.push(self.def.clone());
        self.inserted_index = Some(idx);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(idx) = self.inserted_index.take() {
            if idx < p.defs.entities.len() && p.defs.entities[idx].id == self.def.id {
                p.defs.entities.remove(idx);
            } else if let Some(pos) = p.defs.entities.iter().position(|e| e.id == self.def.id) {
                p.defs.entities.remove(pos);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Entity '{}'", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Replaces an [`EntityDef`]'s metadata and field schema.
#[derive(Debug, Clone)]
pub struct UpdateEntityDefCommand {
    def: EntityDef,
    old_def: Option<EntityDef>,
    stroke_id: Option<u64>,
}

impl UpdateEntityDefCommand {
    /// Creates a command replacing the definition with `def`.
    pub fn new(def: EntityDef) -> Self {
        Self {
            def,
            old_def: None,
            stroke_id: None,
        }
    }

    /// Creates a command that merges consecutive edits sharing `stroke_id`.
    pub fn with_stroke(def: EntityDef, stroke_id: u64) -> Self {
        Self {
            def,
            old_def: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for UpdateEntityDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let existing = p
            .defs
            .entity_mut(self.def.id)
            .ok_or(CoreError::EntityDefNotFound(self.def.id))?;
        if self.old_def.is_none() {
            self.old_def = Some(existing.clone());
        }
        *existing = self.def.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(old), Some(current)) = (&self.old_def, p.defs.entity_mut(self.def.id)) {
            *current = old.clone();
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.def.id == other.def.id
                && self.stroke_id.is_some()
                && self.stroke_id == other.stroke_id
            {
                self.def = other.def.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Entity '{}'", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes an [`EntityDef`], either blocking or cascading when it is in use.
#[derive(Debug, Clone)]
pub struct RemoveEntityDefCommand {
    id: EntityDefId,
    cascade: bool,
    removed: Option<(usize, EntityDef)>,
    removed_instances: Vec<RemovedInstance>,
    cleared_refs: Vec<ClearedRef>,
}

impl RemoveEntityDefCommand {
    /// Creates a command that fails while instances of `id` exist.
    pub fn safe(id: EntityDefId) -> Self {
        Self {
            id,
            cascade: false,
            removed: None,
            removed_instances: Vec::new(),
            cleared_refs: Vec::new(),
        }
    }

    /// Creates a command that also removes every instance of `id` and clears
    /// entity references pointing at them.
    pub fn cascade(id: EntityDefId) -> Self {
        Self {
            id,
            cascade: true,
            removed: None,
            removed_instances: Vec::new(),
            cleared_refs: Vec::new(),
        }
    }
}

impl Command for RemoveEntityDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let pos = p
            .defs
            .entities
            .iter()
            .position(|e| e.id == self.id)
            .ok_or(CoreError::EntityDefNotFound(self.id))?;

        let used = count_entity_def_instances(p, self.id);
        if used > 0 && !self.cascade {
            return Err(CoreError::DefNotFound(format!(
                "Cannot delete EntityDef {:?} because {used} instance(s) still use it",
                self.id
            )));
        }

        self.removed_instances.clear();
        self.cleared_refs.clear();

        let mut removed_ids = Vec::new();
        if self.cascade {
            for world in &mut p.worlds {
                for level in &mut world.levels {
                    for layer in &mut level.layers {
                        if let LayerData::Entities(entities) = &mut layer.data {
                            let mut index = entities.len();
                            while index > 0 {
                                index -= 1;
                                if entities[index].def_id == self.id {
                                    let entity = entities.remove(index);
                                    removed_ids.push(entity.id);
                                    self.removed_instances.push(RemovedInstance {
                                        world_id: world.id,
                                        level_id: level.id,
                                        layer_def_id: layer.def_id,
                                        index,
                                        entity,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            scrub_dangling_refs(p, &removed_ids, &mut self.cleared_refs);
        }

        let def = p.defs.entities.remove(pos);
        self.removed = Some((pos, def));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some((pos, def)) = self.removed.take() {
            if pos <= p.defs.entities.len() {
                p.defs.entities.insert(pos, def);
            } else {
                p.defs.entities.push(def);
            }
        }

        restore_cleared_refs(p, &mut self.cleared_refs);

        for removed in self.removed_instances.drain(..).rev() {
            let Some(world) = p.world_mut(removed.world_id) else {
                continue;
            };
            let Some(level) = world.level_mut(removed.level_id) else {
                continue;
            };
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
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Entity #{}", self.id.0)
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Number of instances and their locations, for the definition page banner.
pub fn entity_def_usage(
    p: &Project,
    id: EntityDefId,
) -> (usize, Vec<crate::helpers::EntityLocation>) {
    let locations = find_entity_def_instances(p, id);
    (locations.len(), locations)
}

/// Generates a unique entity definition name derived from `base_name`.
pub fn generate_unique_entity_name(existing: &[String], base_name: &str) -> String {
    let clean = base_name.trim();
    let base = if clean.is_empty() { "Entity" } else { clean };
    let candidate = format!("{base}_Copy");
    if !existing.iter().any(|n| n == &candidate) {
        return candidate;
    }
    let mut counter = 2usize;
    loop {
        let numbered = format!("{base}_Copy_{counter}");
        if !existing.iter().any(|n| n == &numbered) {
            return numbered;
        }
        counter += 1;
    }
}

/// Reorders an [`EntityDef`] within `Project.defs.entities`.
#[derive(Debug, Clone)]
pub struct ReorderEntityDefCommand {
    id: EntityDefId,
    target_index: usize,
    old_index: Option<usize>,
}

impl ReorderEntityDefCommand {
    /// Creates a command moving `id` to `target_index`.
    pub fn new(id: EntityDefId, target_index: usize) -> Self {
        Self {
            id,
            target_index,
            old_index: None,
        }
    }
}

impl Command for ReorderEntityDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let old_idx = p
            .defs
            .entities
            .iter()
            .position(|e| e.id == self.id)
            .ok_or(CoreError::EntityDefNotFound(self.id))?;
        self.old_index = Some(old_idx);
        let item = p.defs.entities.remove(old_idx);
        let insert_idx = self.target_index.min(p.defs.entities.len());
        p.defs.entities.insert(insert_idx, item);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old_idx) = self.old_index.take() {
            if let Some(cur_idx) = p.defs.entities.iter().position(|e| e.id == self.id) {
                let item = p.defs.entities.remove(cur_idx);
                let restore_idx = old_idx.min(p.defs.entities.len());
                p.defs.entities.insert(restore_idx, item);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Reorder Entity #{}", self.id.0)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
