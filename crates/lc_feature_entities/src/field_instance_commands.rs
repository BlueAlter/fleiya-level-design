//! Command for editing field values on entity instances from the inspector.
//!
//! Field edits carry an optional stroke id so a whole typing session or drag
//! merges into one undo step.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::fields::{FieldType, FieldValue};
use lc_core::id::{EntityId, LevelId};
use lc_core::project::Project;

/// One field value assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityFieldUpdate {
    /// Target entity instance.
    pub entity_id: EntityId,
    /// Field name on the entity definition.
    pub field_name: String,
    /// New value.
    pub value: FieldValue,
}

#[derive(Debug, Clone)]
struct RecordedChange {
    entity_id: EntityId,
    field_name: String,
    previous: Option<FieldValue>,
}

/// Sets field values on entity instances, e.g. from the floating inspector.
///
/// Automatically maintains reciprocal links when the field's [`FieldDef::symmetrical_ref`]
/// flag is enabled, and restores all modified entities on `revert`.
#[derive(Debug, Clone)]
pub struct SetEntityFieldValuesCommand {
    level_id: LevelId,
    updates: Vec<EntityFieldUpdate>,
    previous: Vec<RecordedChange>,
    stroke_id: Option<u64>,
}

impl SetEntityFieldValuesCommand {
    /// Creates a command assigning `updates`.
    pub fn new(level_id: LevelId, updates: Vec<EntityFieldUpdate>) -> Self {
        Self {
            level_id,
            updates,
            previous: Vec::new(),
            stroke_id: None,
        }
    }

    /// Creates a command that merges field edits sharing `stroke_id`.
    pub fn with_stroke(level_id: LevelId, updates: Vec<EntityFieldUpdate>, stroke_id: u64) -> Self {
        Self {
            stroke_id: Some(stroke_id),
            ..Self::new(level_id, updates)
        }
    }

    fn record_once(&mut self, entity_id: EntityId, field_name: &str, prev_val: Option<FieldValue>) {
        if !self
            .previous
            .iter()
            .any(|c| c.entity_id == entity_id && c.field_name == field_name)
        {
            self.previous.push(RecordedChange {
                entity_id,
                field_name: field_name.to_string(),
                previous: prev_val,
            });
        }
    }
}

impl Command for SetEntityFieldValuesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if p.level(self.level_id).is_none() {
            return Err(CoreError::LevelNotFound(self.level_id));
        }

        let updates = self.updates.clone();
        for update in &updates {
            let Some((def_id, old_val)) = p.level(self.level_id).and_then(|lvl| {
                lvl.entity(update.entity_id)
                    .map(|e| (e.def_id, e.fields.get(&update.field_name).cloned()))
            }) else {
                continue;
            };
            self.record_once(update.entity_id, &update.field_name, old_val.clone());

            let is_symmetrical = p
                .defs
                .entity(def_id)
                .and_then(|d| d.field_def(&update.field_name))
                .is_some_and(|f| f.symmetrical_ref && matches!(f.field_type, FieldType::EntityRef));

            if is_symmetrical {
                let old_target = match old_val {
                    Some(FieldValue::EntityRef(t)) => t,
                    _ => None,
                };
                let new_target = match update.value {
                    FieldValue::EntityRef(t) => t,
                    _ => None,
                };

                if let Some(prev_target_id) = old_target {
                    if Some(prev_target_id) != new_target {
                        let prev_target_val = p.level(self.level_id).and_then(|lvl| {
                            lvl.entity(prev_target_id)
                                .and_then(|e| e.fields.get(&update.field_name).cloned())
                        });
                        if prev_target_val == Some(FieldValue::EntityRef(Some(update.entity_id))) {
                            self.record_once(prev_target_id, &update.field_name, prev_target_val);
                            if let Some(target_ent) = p
                                .level_mut(self.level_id)
                                .and_then(|lvl| lvl.entity_mut(prev_target_id))
                            {
                                target_ent
                                    .fields
                                    .set(update.field_name.clone(), FieldValue::EntityRef(None));
                            }
                        }
                    }
                }

                if let Some(next_target_id) = new_target {
                    if next_target_id != update.entity_id {
                        let target_has_field = p.level(self.level_id).and_then(|lvl| {
                            lvl.entity(next_target_id).and_then(|e| {
                                p.defs
                                    .entity(e.def_id)
                                    .and_then(|d| d.field_def(&update.field_name))
                                    .filter(|f| {
                                        f.symmetrical_ref
                                            && matches!(f.field_type, FieldType::EntityRef)
                                    })
                                    .map(|_| e.fields.get(&update.field_name).cloned())
                            })
                        });
                        if let Some(prev_target_val) = target_has_field {
                            self.record_once(next_target_id, &update.field_name, prev_target_val);
                            if let Some(target_ent) = p
                                .level_mut(self.level_id)
                                .and_then(|lvl| lvl.entity_mut(next_target_id))
                            {
                                target_ent.fields.set(
                                    update.field_name.clone(),
                                    FieldValue::EntityRef(Some(update.entity_id)),
                                );
                            }
                        }
                    }
                }
            }

            if let Some(entity) = p
                .level_mut(self.level_id)
                .and_then(|lvl| lvl.entity_mut(update.entity_id))
            {
                entity
                    .fields
                    .set(update.field_name.clone(), update.value.clone());
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(level) = p.level_mut(self.level_id) else {
            return;
        };
        for change in self.previous.iter().rev() {
            if let Some(entity) = level.entity_mut(change.entity_id) {
                match &change.previous {
                    Some(value) => {
                        entity.fields.set(change.field_name.clone(), value.clone());
                    }
                    None => {
                        entity.fields.remove(&change.field_name);
                    }
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        let Some(other) = next.as_any().downcast_ref::<Self>() else {
            return false;
        };
        let same_targets = self.updates.len() == other.updates.len()
            && self
                .updates
                .iter()
                .zip(&other.updates)
                .all(|(a, b)| a.entity_id == b.entity_id && a.field_name == b.field_name);
        if self.level_id == other.level_id
            && self.stroke_id.is_some()
            && self.stroke_id == other.stroke_id
            && same_targets
        {
            self.updates = other.updates.clone();
            return true;
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit {} Entity Field(s)", self.updates.len())
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
