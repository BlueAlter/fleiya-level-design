//! LDtk instance limit enforcement (`max_count`, `limit_scope`, `limit_behavior`).

use glam::{IVec2, UVec2};
use lc_core::command::{Command, DocKind};
use lc_core::defs::{EntityLimitBehavior, EntityLimitScope};
use lc_core::error::CoreError;
use lc_core::id::{EntityDefId, EntityId, LayerDefId, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData};
use lc_core::project::Project;

use crate::refs::{restore_cleared_refs, scrub_dangling_refs, ClearedRef};

/// Exact location of an entity instance including its layer and index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedEntityLocation {
    /// World ID.
    pub world_id: WorldId,
    /// Level ID.
    pub level_id: LevelId,
    /// Layer definition ID.
    pub layer_def_id: LayerDefId,
    /// Index inside `LayerData::Entities`.
    pub index: usize,
    /// Entity ID.
    pub entity_id: EntityId,
}

#[derive(Debug, Clone)]
struct RemovedExcess {
    world_id: WorldId,
    level_id: LevelId,
    layer_def_id: LayerDefId,
    index: usize,
    entity: EntityInstance,
}

#[derive(Debug, Clone)]
struct MovedLast {
    world_id: WorldId,
    level_id: LevelId,
    layer_def_id: LayerDefId,
    entity_id: EntityId,
    old_pos: IVec2,
    old_size: UVec2,
}

/// Lists every instance of `def_id` within the given `scope`.
pub fn instances_in_limit_scope(
    p: &Project,
    def_id: EntityDefId,
    scope: EntityLimitScope,
    world_id: WorldId,
    level_id: LevelId,
    layer_def_id: LayerDefId,
) -> Vec<ScopedEntityLocation> {
    let mut out = Vec::new();
    for world in &p.worlds {
        if scope == EntityLimitScope::PerWorld && world.id != world_id {
            continue;
        }
        if matches!(
            scope,
            EntityLimitScope::PerLevel | EntityLimitScope::PerLayer
        ) && world.id != world_id
        {
            continue;
        }
        for level in &world.levels {
            if matches!(
                scope,
                EntityLimitScope::PerLevel | EntityLimitScope::PerLayer
            ) && level.id != level_id
            {
                continue;
            }
            for layer in &level.layers {
                if scope == EntityLimitScope::PerLayer && layer.def_id != layer_def_id {
                    continue;
                }
                if let LayerData::Entities(entities) = &layer.data {
                    for (index, entity) in entities.iter().enumerate() {
                        if entity.def_id == def_id {
                            out.push(ScopedEntityLocation {
                                world_id: world.id,
                                level_id: level.id,
                                layer_def_id: layer.def_id,
                                index,
                                entity_id: entity.id,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

/// Returns `true` if placing an entity of `def_id` is allowed (i.e. not blocked by `PreventAdding`).
pub fn can_place_with_limits(
    p: &Project,
    def_id: EntityDefId,
    world_id: WorldId,
    level_id: LevelId,
    layer_def_id: LayerDefId,
) -> bool {
    let Some(def) = p.defs.entity(def_id) else {
        return false;
    };
    if def.max_count == 0 || def.limit_behavior != EntityLimitBehavior::PreventAdding {
        return true;
    }
    let existing =
        instances_in_limit_scope(p, def_id, def.limit_scope, world_id, level_id, layer_def_id);
    existing.len() < def.max_count as usize
}

/// Places an [`EntityInstance`] while enforcing the definition's LDtk `max_count`,
/// `limit_scope`, and `limit_behavior`.
#[derive(Debug, Clone)]
pub struct PlaceEntityWithLimitsCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    entity: EntityInstance,
    prev_next_entity_id: Option<u64>,
    removed_excess: Vec<RemovedExcess>,
    cleared_refs: Vec<ClearedRef>,
    moved_last: Option<MovedLast>,
    inserted: bool,
}

impl PlaceEntityWithLimitsCommand {
    /// Creates a command placing `entity` onto `(level_id, layer_def_id)`.
    pub fn new(level_id: LevelId, layer_def_id: LayerDefId, entity: EntityInstance) -> Self {
        Self {
            level_id,
            layer_def_id,
            entity,
            prev_next_entity_id: None,
            removed_excess: Vec::new(),
            cleared_refs: Vec::new(),
            moved_last: None,
            inserted: false,
        }
    }

    /// The entity instance being placed.
    pub fn entity(&self) -> &EntityInstance {
        &self.entity
    }
}

impl Command for PlaceEntityWithLimitsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world_id = p
            .level_and_world(self.level_id)
            .map(|(w, _)| w.id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let def = p
            .defs
            .entity(self.entity.def_id)
            .cloned()
            .ok_or(CoreError::EntityDefNotFound(self.entity.def_id))?;

        self.removed_excess.clear();
        self.cleared_refs.clear();
        self.moved_last = None;
        self.inserted = false;

        if def.max_count > 0 {
            let scoped = instances_in_limit_scope(
                p,
                def.id,
                def.limit_scope,
                world_id,
                self.level_id,
                self.layer_def_id,
            );
            let limit = def.max_count as usize;
            if scoped.len() >= limit {
                match def.limit_behavior {
                    EntityLimitBehavior::PreventAdding => {
                        return Err(CoreError::DefNotFound(format!(
                            "Entity '{}' reached max count ({limit})",
                            def.name
                        )));
                    }
                    EntityLimitBehavior::MoveLastOne => {
                        if let Some(last) = scoped.last() {
                            if let Some(entity) = p
                                .world_mut(last.world_id)
                                .and_then(|w| w.level_mut(last.level_id))
                                .and_then(|l| l.layer_mut(last.layer_def_id))
                                .and_then(|layer| match &mut layer.data {
                                    LayerData::Entities(list) => list.get_mut(last.index),
                                    _ => None,
                                })
                            {
                                self.moved_last = Some(MovedLast {
                                    world_id: last.world_id,
                                    level_id: last.level_id,
                                    layer_def_id: last.layer_def_id,
                                    entity_id: entity.id,
                                    old_pos: entity.pos_px,
                                    old_size: entity.size_px,
                                });
                                entity.pos_px = self.entity.pos_px;
                                entity.size_px = self.entity.size_px;
                                return Ok(());
                            }
                        }
                    }
                    EntityLimitBehavior::DiscardOldOnes => {
                        let to_remove = scoped.len().saturating_sub(limit).saturating_add(1);
                        let targets: Vec<ScopedEntityLocation> =
                            scoped.into_iter().take(to_remove).collect();
                        let mut removed_ids = Vec::with_capacity(targets.len());
                        // Remove in reverse index order per layer so indices stay valid.
                        let mut ordered = targets;
                        ordered.sort_by(|a, b| {
                            a.world_id
                                .0
                                .cmp(&b.world_id.0)
                                .then(a.level_id.0.cmp(&b.level_id.0))
                                .then(a.layer_def_id.0.cmp(&b.layer_def_id.0))
                                .then(b.index.cmp(&a.index))
                        });
                        for loc in ordered {
                            if let Some(LayerData::Entities(list)) = p
                                .world_mut(loc.world_id)
                                .and_then(|w| w.level_mut(loc.level_id))
                                .and_then(|l| l.layer_mut(loc.layer_def_id))
                                .map(|layer| &mut layer.data)
                            {
                                if loc.index < list.len() && list[loc.index].id == loc.entity_id {
                                    let removed = list.remove(loc.index);
                                    removed_ids.push(removed.id);
                                    self.removed_excess.push(RemovedExcess {
                                        world_id: loc.world_id,
                                        level_id: loc.level_id,
                                        layer_def_id: loc.layer_def_id,
                                        index: loc.index,
                                        entity: removed,
                                    });
                                }
                            }
                        }
                        scrub_dangling_refs(p, &removed_ids, &mut self.cleared_refs);
                    }
                }
            }
        }

        if self.prev_next_entity_id.is_none() {
            self.prev_next_entity_id = p.next_entity_id;
        }
        let next_floor = self.entity.id.0.saturating_add(1);
        p.next_entity_id = Some(p.next_entity_id.unwrap_or(next_floor).max(next_floor));

        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;
        let LayerData::Entities(entities) = &mut layer.data else {
            return Err(CoreError::DefNotFound(format!(
                "Layer {:?} is not an Entities layer",
                self.layer_def_id
            )));
        };
        entities.push(self.entity.clone());
        self.inserted = true;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(moved) = self.moved_last.take() {
            if let Some(LayerData::Entities(entities)) = p
                .world_mut(moved.world_id)
                .and_then(|w| w.level_mut(moved.level_id))
                .and_then(|l| l.layer_mut(moved.layer_def_id))
                .map(|layer| &mut layer.data)
            {
                if let Some(entity) = entities.iter_mut().find(|e| e.id == moved.entity_id) {
                    entity.pos_px = moved.old_pos;
                    entity.size_px = moved.old_size;
                }
            }
            return;
        }

        if self.inserted {
            if let Some(LayerData::Entities(entities)) = p
                .level_mut(self.level_id)
                .and_then(|l| l.layer_mut(self.layer_def_id))
                .map(|layer| &mut layer.data)
            {
                if let Some(pos) = entities.iter().rposition(|e| e.id == self.entity.id) {
                    entities.remove(pos);
                }
            }
            self.inserted = false;
        }

        restore_cleared_refs(p, &mut self.cleared_refs);

        for removed in self.removed_excess.drain(..).rev() {
            if let Some(LayerData::Entities(entities)) = p
                .world_mut(removed.world_id)
                .and_then(|w| w.level_mut(removed.level_id))
                .and_then(|l| l.layer_mut(removed.layer_def_id))
                .map(|layer| &mut layer.data)
            {
                let idx = removed.index.min(entities.len());
                entities.insert(idx, removed.entity);
            }
        }

        p.next_entity_id = self.prev_next_entity_id.take();
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Place Entity #{}", self.entity.id.0)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
