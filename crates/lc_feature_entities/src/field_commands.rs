//! Commands for adding, updating and removing entity field definitions.
//!
//! Every command migrates instance values as well as the schema so undo restores
//! the project byte-for-byte.

use lc_core::command::{Command, DocKind};
use lc_core::defs::EnumDef;
use lc_core::error::CoreError;
use lc_core::fields::{FieldDef, FieldValue};
use lc_core::id::{EntityDefId, EntityId, LevelId};
use lc_core::project::Project;

use crate::field_defs::{coerce_field_value, default_field_value};
use crate::helpers::for_each_instance_mut;

#[derive(Debug, Clone)]
struct FieldMigration {
    level_id: LevelId,
    entity_id: EntityId,
    old_name: String,
    new_name: String,
    old_value: Option<FieldValue>,
}

fn enum_defs(p: &Project) -> &[EnumDef] {
    &p.defs.enums
}

fn default_for(field: &FieldDef, enums: &[EnumDef]) -> FieldValue {
    field
        .default_value
        .clone()
        .map(|value| coerce_field_value(&value, &field.field_type, enums))
        .unwrap_or_else(|| default_field_value(&field.field_type, enums))
}

fn validate_field_name(name: &str) -> Result<String, CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::DefNotFound(
            "Field name must not be empty".to_string(),
        ));
    }
    Ok(name.trim().to_string())
}

/// Adds a field definition and seeds every existing instance with its default.
#[derive(Debug, Clone)]
pub struct AddEntityFieldCommand {
    def_id: EntityDefId,
    field: FieldDef,
    inserted_values: Vec<(LevelId, EntityId)>,
}

impl AddEntityFieldCommand {
    /// Creates a command adding `field` to `def_id`.
    pub fn new(def_id: EntityDefId, field: FieldDef) -> Self {
        Self {
            def_id,
            field,
            inserted_values: Vec::new(),
        }
    }
}

impl Command for AddEntityFieldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enums = enum_defs(p).to_vec();
        let name = validate_field_name(&self.field.name)?;
        self.field.name = name.clone();

        let def = p
            .defs
            .entity_mut(self.def_id)
            .ok_or(CoreError::EntityDefNotFound(self.def_id))?;
        if def.field_def(&name).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "Entity '{}' already has a field named '{name}'",
                def.name
            )));
        }

        let default = default_for(&self.field, &enums);
        if self.field.default_value.is_some() {
            self.field.default_value = Some(default.clone());
        }
        def.fields.push(self.field.clone());

        let inserted = &mut self.inserted_values;
        inserted.clear();
        for_each_instance_mut(p, self.def_id, |level_id, entity| {
            if entity.fields.get(&name).is_none() {
                entity.fields.set(name.clone(), default.clone());
                inserted.push((level_id, entity.id));
            }
        });
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(def) = p.defs.entity_mut(self.def_id) {
            if let Some(pos) = def.fields.iter().position(|f| f.name == self.field.name) {
                def.fields.remove(pos);
            }
        }
        for (level_id, entity_id) in self.inserted_values.drain(..) {
            if let Some(entity) = p
                .level_mut(level_id)
                .and_then(|level| level.entity_mut(entity_id))
            {
                entity.fields.remove(&self.field.name);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Field '{}'", self.field.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.def_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Updates a field definition and migrates existing instance values.
#[derive(Debug, Clone)]
pub struct UpdateEntityFieldCommand {
    def_id: EntityDefId,
    old_name: String,
    field: FieldDef,
    previous: Option<(usize, FieldDef)>,
    migrations: Vec<FieldMigration>,
    stroke_id: Option<u64>,
}

impl UpdateEntityFieldCommand {
    /// Creates a command replacing the field named by `old_name`.
    pub fn new(def_id: EntityDefId, old_name: impl Into<String>, field: FieldDef) -> Self {
        Self {
            def_id,
            old_name: old_name.into(),
            field,
            previous: None,
            migrations: Vec::new(),
            stroke_id: None,
        }
    }

    /// Creates a command that merges consecutive default-value edits.
    pub fn with_stroke(
        def_id: EntityDefId,
        old_name: impl Into<String>,
        field: FieldDef,
        stroke_id: u64,
    ) -> Self {
        Self {
            stroke_id: Some(stroke_id),
            ..Self::new(def_id, old_name, field)
        }
    }
}

impl Command for UpdateEntityFieldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enums = enum_defs(p).to_vec();
        let new_name = validate_field_name(&self.field.name)?;
        self.field.name = new_name.clone();
        if let Some(default) = &self.field.default_value {
            self.field.default_value =
                Some(coerce_field_value(default, &self.field.field_type, &enums));
        }

        let def = p
            .defs
            .entity_mut(self.def_id)
            .ok_or(CoreError::EntityDefNotFound(self.def_id))?;
        let index = def
            .fields
            .iter()
            .position(|f| f.name == self.old_name)
            .ok_or_else(|| {
                CoreError::DefNotFound(format!(
                    "Entity '{}' has no field named '{}'",
                    def.name, self.old_name
                ))
            })?;
        if new_name != self.old_name && def.field_def(&new_name).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "Entity '{}' already has a field named '{new_name}'",
                def.name
            )));
        }

        if self.previous.is_none() {
            self.previous = Some((index, def.fields[index].clone()));
        }
        def.fields[index] = self.field.clone();

        let old_name = self.old_name.clone();
        let field_type = self.field.field_type.clone();
        let default = default_for(&self.field, &enums);
        let migrations = &mut self.migrations;
        migrations.clear();
        for_each_instance_mut(p, self.def_id, |level_id, entity| {
            let old_value = entity.fields.get(&old_name).cloned();
            let new_value = match &old_value {
                Some(value) => Some(coerce_field_value(value, &field_type, &enums)),
                None if old_name != new_name => Some(default.clone()),
                None => None,
            };
            migrations.push(FieldMigration {
                level_id,
                entity_id: entity.id,
                old_name: old_name.clone(),
                new_name: new_name.clone(),
                old_value,
            });

            if old_name != new_name {
                entity.fields.remove(&old_name);
            }
            match new_value {
                Some(value) => {
                    entity.fields.set(new_name.clone(), value);
                }
                None => {
                    entity.fields.remove(&new_name);
                }
            }
        });
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        for migration in self.migrations.drain(..) {
            let Some(entity) = p
                .level_mut(migration.level_id)
                .and_then(|level| level.entity_mut(migration.entity_id))
            else {
                continue;
            };
            if migration.old_name != migration.new_name {
                entity.fields.remove(&migration.new_name);
            }
            match migration.old_value {
                Some(value) => {
                    entity.fields.set(migration.old_name, value);
                }
                None => {
                    entity.fields.remove(&migration.old_name);
                }
            }
        }

        if let (Some((index, previous)), Some(def)) =
            (self.previous.take(), p.defs.entity_mut(self.def_id))
        {
            if let Some(current) = def.fields.iter().position(|f| f.name == self.field.name) {
                def.fields.remove(current);
            }
            if index <= def.fields.len() {
                def.fields.insert(index, previous);
            } else {
                def.fields.push(previous);
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.def_id == other.def_id
                && self.field.field_type == other.field.field_type
                && other.old_name == self.field.name
                && self.stroke_id.is_some()
                && self.stroke_id == other.stroke_id
            {
                self.field = other.field.clone();
                for migration in &mut self.migrations {
                    migration.new_name = self.field.name.clone();
                }
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Field '{}'", self.field.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.def_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes a field definition and its values from every instance.
#[derive(Debug, Clone)]
pub struct RemoveEntityFieldCommand {
    def_id: EntityDefId,
    name: String,
    removed: Option<(usize, FieldDef)>,
    removed_values: Vec<(LevelId, EntityId, FieldValue)>,
}

impl RemoveEntityFieldCommand {
    /// Creates a command removing the field named `name`.
    pub fn new(def_id: EntityDefId, name: impl Into<String>) -> Self {
        Self {
            def_id,
            name: name.into(),
            removed: None,
            removed_values: Vec::new(),
        }
    }
}

impl Command for RemoveEntityFieldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let def = p
            .defs
            .entity_mut(self.def_id)
            .ok_or(CoreError::EntityDefNotFound(self.def_id))?;
        let index = def
            .fields
            .iter()
            .position(|f| f.name == self.name)
            .ok_or_else(|| {
                CoreError::DefNotFound(format!(
                    "Entity '{}' has no field named '{}'",
                    def.name, self.name
                ))
            })?;
        let field = def.fields.remove(index);
        self.removed = Some((index, field));

        let removed = &mut self.removed_values;
        removed.clear();
        for_each_instance_mut(p, self.def_id, |level_id, entity| {
            if let Some(value) = entity.fields.remove(&self.name) {
                removed.push((level_id, entity.id, value));
            }
        });
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some((index, field)), Some(def)) =
            (self.removed.take(), p.defs.entity_mut(self.def_id))
        {
            if index <= def.fields.len() {
                def.fields.insert(index, field);
            } else {
                def.fields.push(field);
            }
        }
        for (level_id, entity_id, value) in self.removed_values.drain(..) {
            if let Some(entity) = p
                .level_mut(level_id)
                .and_then(|level| level.entity_mut(entity_id))
            {
                entity.fields.set(self.name.clone(), value);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Field '{}'", self.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EntityDef(self.def_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
