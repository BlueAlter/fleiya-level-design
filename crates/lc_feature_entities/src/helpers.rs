//! Project-wide entity instance helpers shared by commands and tools.

use lc_core::fields::{AllowedRefs, FieldDef};
use lc_core::id::{EntityDefId, EntityId, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData};
use lc_core::project::Project;

use crate::field_editors::{entity_choice_label, EntityChoice};

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

/// Finds the level containing a specific entity instance.
pub fn find_entity_location(p: &Project, entity_id: EntityId) -> Option<EntityLocation> {
    for world in &p.worlds {
        for level in &world.levels {
            for layer in &level.layers {
                if let LayerData::Entities(entities) = &layer.data {
                    if entities.iter().any(|e| e.id == entity_id) {
                        return Some(EntityLocation {
                            world_id: world.id,
                            level_id: level.id,
                            entity_id,
                        });
                    }
                }
            }
        }
    }
    None
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

/// Returns the next free entity instance id across the whole project,
/// honoring the monotonic high-water mark when set.
pub fn next_entity_id(p: &Project) -> EntityId {
    let floor = p.max_live_entity_id().saturating_add(1);
    EntityId(p.next_entity_id.unwrap_or(floor).max(floor))
}

/// Filters project entity instances according to [`FieldDef::allowed_refs`] rules.
pub fn filter_entity_choices(
    p: &Project,
    owner_def_id: EntityDefId,
    field: &FieldDef,
) -> Vec<EntityChoice> {
    let mut out = Vec::new();
    for world in &p.worlds {
        for level in &world.levels {
            for layer in &level.layers {
                if let LayerData::Entities(entities) = &layer.data {
                    for entity in entities {
                        let Some(target_def) = p.defs.entity(entity.def_id) else {
                            continue;
                        };
                        let allowed = match field.allowed_refs {
                            AllowedRefs::Any => true,
                            AllowedRefs::OnlySame => entity.def_id == owner_def_id,
                            AllowedRefs::OnlySpecificEntity => {
                                field.allowed_refs_entity_id == Some(entity.def_id)
                            }
                            AllowedRefs::OnlyTags => field.allowed_ref_tags.iter().any(|tag| {
                                target_def.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                            }),
                        };
                        if allowed {
                            out.push(EntityChoice {
                                id: entity.id,
                                label: entity_choice_label(&target_def.name, entity.id),
                            });
                        }
                    }
                }
            }
        }
    }
    out
}
