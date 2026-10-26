//! Command for editing field values on entity instances from the inspector.
//!
//! Field edits carry an optional stroke id so a whole typing session or drag
//! merges into one undo step.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::fields::FieldValue;
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

/// Sets field values on entity instances, e.g. from the floating inspector.
#[derive(Debug, Clone)]
pub struct SetEntityFieldValuesCommand {
    level_id: LevelId,
    updates: Vec<EntityFieldUpdate>,
    previous: Vec<(EntityId, String, Option<FieldValue>)>,
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
}

impl Command for SetEntityFieldValuesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        if self.previous.is_empty() {
            for update in &self.updates {
                if let Some(entity) = level.entity(update.entity_id) {
                    self.previous.push((
                        update.entity_id,
                        update.field_name.clone(),
                        entity.fields.get(&update.field_name).cloned(),
                    ));
                }
            }
        }
        for update in &self.updates {
            if let Some(entity) = level.entity_mut(update.entity_id) {
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
        for (entity_id, field_name, value) in &self.previous {
            if let Some(entity) = level.entity_mut(*entity_id) {
                match value {
                    Some(value) => {
                        entity.fields.set(field_name.clone(), value.clone());
                    }
                    None => {
                        entity.fields.remove(field_name);
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
