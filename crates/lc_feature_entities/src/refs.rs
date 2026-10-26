//! Shared bookkeeping for clearing and restoring weak `EntityRef` values.
//!
//! Entity references are weak: deleting an entity instance (or cascading an
//! entity definition) clears every value that pointed at it, across entity
//! instances, level fields, and field defaults. Revert restores them.

use lc_core::fields::FieldValue;
use lc_core::id::{EntityDefId, EntityId, LevelId};
use lc_core::layer::LayerData;
use lc_core::project::Project;

#[derive(Debug, Clone)]
pub(crate) enum RefOwner {
    Level {
        level_id: LevelId,
        field_name: String,
    },
    EntityDefault {
        def_id: EntityDefId,
        field_name: String,
    },
    EntityInstance {
        level_id: LevelId,
        entity_id: EntityId,
        field_name: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct ClearedRef {
    owner: RefOwner,
    path: Vec<usize>,
    target: EntityId,
}

/// Clears every reference to the removed entity ids, remembering what was
/// cleared so undo can restore it.
pub(crate) fn scrub_dangling_refs(
    p: &mut Project,
    removed: &[EntityId],
    out: &mut Vec<ClearedRef>,
) {
    if removed.is_empty() {
        return;
    }

    for world in &mut p.worlds {
        for level in &mut world.levels {
            let level_id = level.id;
            let field_names: Vec<String> =
                level.fields.iter().map(|(name, _)| name.clone()).collect();
            for field_name in field_names {
                let mut path = Vec::new();
                if let Some(value) = level.fields.get_mut(&field_name) {
                    scrub_value(
                        value,
                        removed,
                        &mut path,
                        RefOwner::Level {
                            level_id,
                            field_name,
                        },
                        out,
                    );
                }
            }
        }
    }

    for def in &mut p.defs.entities {
        let def_id = def.id;
        for field in &mut def.fields {
            if let Some(value) = &mut field.default_value {
                let mut path = Vec::new();
                scrub_value(
                    value,
                    removed,
                    &mut path,
                    RefOwner::EntityDefault {
                        def_id,
                        field_name: field.name.clone(),
                    },
                    out,
                );
            }
        }
    }

    for world in &mut p.worlds {
        for level in &mut world.levels {
            let level_id = level.id;
            for layer in &mut level.layers {
                if let LayerData::Entities(entities) = &mut layer.data {
                    for entity in entities {
                        let entity_id = entity.id;
                        let field_names: Vec<String> =
                            entity.fields.iter().map(|(name, _)| name.clone()).collect();
                        for field_name in field_names {
                            let mut path = Vec::new();
                            if let Some(value) = entity.fields.get_mut(&field_name) {
                                scrub_value(
                                    value,
                                    removed,
                                    &mut path,
                                    RefOwner::EntityInstance {
                                        level_id,
                                        entity_id,
                                        field_name,
                                    },
                                    out,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn scrub_value(
    value: &mut FieldValue,
    removed: &[EntityId],
    path: &mut Vec<usize>,
    owner: RefOwner,
    out: &mut Vec<ClearedRef>,
) {
    match value {
        FieldValue::EntityRef(Some(target)) if removed.contains(target) => {
            out.push(ClearedRef {
                owner,
                path: path.clone(),
                target: *target,
            });
            *value = FieldValue::EntityRef(None);
        }
        FieldValue::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                path.push(index);
                scrub_value(item, removed, path, owner.clone(), out);
                path.pop();
            }
        }
        _ => {}
    }
}

pub(crate) fn restore_cleared_refs(p: &mut Project, refs: &mut Vec<ClearedRef>) {
    for cleared in refs.drain(..) {
        match &cleared.owner {
            RefOwner::Level {
                level_id,
                field_name,
            } => {
                if let Some(value) = p
                    .level_mut(*level_id)
                    .and_then(|level| level.fields.get_mut(field_name))
                {
                    restore_ref(value, &cleared.path, cleared.target);
                }
            }
            RefOwner::EntityDefault { def_id, field_name } => {
                if let Some(value) = p
                    .defs
                    .entity_mut(*def_id)
                    .and_then(|def| def.fields.iter_mut().find(|f| f.name == *field_name))
                    .and_then(|field| field.default_value.as_mut())
                {
                    restore_ref(value, &cleared.path, cleared.target);
                }
            }
            RefOwner::EntityInstance {
                level_id,
                entity_id,
                field_name,
            } => {
                if let Some(value) = p
                    .level_mut(*level_id)
                    .and_then(|level| level.entity_mut(*entity_id))
                    .and_then(|entity| entity.fields.get_mut(field_name))
                {
                    restore_ref(value, &cleared.path, cleared.target);
                }
            }
        }
    }
}

fn restore_ref(value: &mut FieldValue, path: &[usize], target: EntityId) -> bool {
    if path.is_empty() {
        if matches!(value, FieldValue::EntityRef(_)) {
            *value = FieldValue::EntityRef(Some(target));
            return true;
        }
        return false;
    }
    if let FieldValue::Array(items) = value {
        if let Some(item) = items.get_mut(path[0]) {
            return restore_ref(item, &path[1..], target);
        }
    }
    false
}
