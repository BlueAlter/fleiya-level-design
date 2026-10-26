//! Project-wide entity instance helpers shared by commands and tools.

use lc_core::id::{EntityDefId, EntityId, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData};
use lc_core::project::Project;

/// Location of an entity instance inside the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityLocation {
    /// World containing the level.
    pub world_id: WorldId,
    /// Level containing the layer.
    pub level_id: LevelId,
    /// Entity instance identifier.
    pub entity_id: EntityId,
}

/// Lists every instance of `def_id` across all worlds and levels.
pub fn find_entity_def_instances(p: &Project, def_id: EntityDefId) -> Vec<EntityLocation> {
    let mut out = Vec::new();
    for world in &p.worlds {
        for level in &world.levels {
            for layer in &level.layers {
                if let LayerData::Entities(entities) = &layer.data {
                    for entity in entities {
                        if entity.def_id == def_id {
                            out.push(EntityLocation {
                                world_id: world.id,
                                level_id: level.id,
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

/// Counts instances of `def_id` across the project.
pub fn count_entity_def_instances(p: &Project, def_id: EntityDefId) -> usize {
    p.worlds
        .iter()
        .flat_map(|w| &w.levels)
        .flat_map(|l| &l.layers)
        .filter_map(|layer| match &layer.data {
            LayerData::Entities(entities) => Some(entities),
            _ => None,
        })
        .flatten()
        .filter(|e| e.def_id == def_id)
        .count()
}

/// Visits every entity instance of `def_id`, mutably.
pub(crate) fn for_each_instance_mut(
    p: &mut Project,
    def_id: EntityDefId,
    mut visit: impl FnMut(LevelId, &mut EntityInstance),
) {
    for world in &mut p.worlds {
        for level in &mut world.levels {
            for layer in &mut level.layers {
                if let LayerData::Entities(entities) = &mut layer.data {
                    for entity in entities {
                        if entity.def_id == def_id {
                            visit(level.id, entity);
                        }
                    }
                }
            }
        }
    }
}

/// Returns the next free entity instance id across the whole project.
pub fn next_entity_id(p: &Project) -> EntityId {
    let max = p
        .worlds
        .iter()
        .flat_map(|w| &w.levels)
        .flat_map(|l| &l.layers)
        .filter_map(|layer| match &layer.data {
            LayerData::Entities(entities) => Some(entities),
            _ => None,
        })
        .flatten()
        .map(|e| e.id.0)
        .max()
        .unwrap_or(0);
    EntityId(max.saturating_add(1))
}
