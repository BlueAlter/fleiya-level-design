//! Commands for managing Enum definitions, values, reordering, and reference-safe deletion.

use lc_core::command::{Command, DocKind};
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::error::CoreError;
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_core::id::{EntityDefId, EntityId, EnumDefId, LevelId, WorldId};
use lc_core::project::Project;

/// Detailed record of a project location referencing an `EnumDef` or `EnumValueDef`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumReferenceLocation {
    /// An entity definition has a field referencing this enum.
    EntityField {
        entity_id: EntityDefId,
        entity_name: String,
        field_name: String,
    },
    /// An entity instance inside a level has a field value holding this enum value.
    EntityInstanceValue {
        world_id: WorldId,
        level_id: LevelId,
        entity_id: EntityId,
        field_name: String,
        value: String,
    },
}

/// Checks whether a `FieldType` references `enum_id` (including inside `Array`).
pub fn field_type_references_enum(ft: &FieldType, enum_id: EnumDefId) -> bool {
    match ft {
        FieldType::Enum(id) => *id == enum_id,
        FieldType::Array(inner) => field_type_references_enum(inner, enum_id),
        _ => false,
    }
}

/// Scans the project for any `EntityDef` fields referencing `enum_id`.
pub fn find_enum_def_references(p: &Project, enum_id: EnumDefId) -> Vec<EnumReferenceLocation> {
    let mut refs = Vec::new();
    for entity in &p.defs.entities {
        for field in &entity.fields {
            if field_type_references_enum(&field.field_type, enum_id) {
                refs.push(EnumReferenceLocation::EntityField {
                    entity_id: entity.id,
                    entity_name: entity.name.clone(),
                    field_name: field.name.clone(),
                });
            }
        }
    }
    refs
}

/// Checks whether a `FieldValue` uses a specific enum value ID (including inside `Array`).
pub fn field_value_uses_enum_val(fv: &FieldValue, val_id: &str) -> bool {
    match fv {
        FieldValue::Enum(v) => v == val_id,
        FieldValue::Array(items) => items.iter().any(|i| field_value_uses_enum_val(i, val_id)),
        _ => false,
    }
}

/// Scans the project for any field defaults or entity instance values using `val_id`.
pub fn find_enum_value_references(
    p: &Project,
    enum_id: EnumDefId,
    val_id: &str,
) -> Vec<EnumReferenceLocation> {
    let mut refs = Vec::new();

    // Check entity field defaults
    for entity in &p.defs.entities {
        for field in &entity.fields {
            if field_type_references_enum(&field.field_type, enum_id) {
                if let Some(ref def_val) = field.default_value {
                    if field_value_uses_enum_val(def_val, val_id) {
                        refs.push(EnumReferenceLocation::EntityField {
                            entity_id: entity.id,
                            entity_name: entity.name.clone(),
                            field_name: field.name.clone(),
                        });
                    }
                }
            }
        }
    }

    // Check entity instances in levels
    for world in &p.worlds {
        for level in &world.levels {
            for layer in &level.layers {
                if let lc_core::layer::LayerData::Entities(ref entities) = layer.data {
                    for ent in entities {
                        for (fname, fval) in ent.fields.iter() {
                            if field_value_uses_enum_val(fval, val_id) {
                                refs.push(EnumReferenceLocation::EntityInstanceValue {
                                    world_id: world.id,
                                    level_id: level.id,
                                    entity_id: ent.id,
                                    field_name: fname.clone(),
                                    value: val_id.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    refs
}

/// Command to add a new `EnumDef` to the project.
#[derive(Debug, Clone)]
pub struct AddEnumDefCommand {
    def: EnumDef,
    inserted_index: Option<usize>,
}

impl AddEnumDefCommand {
    /// Creates a command to add a new enum definition.
    pub fn new(def: EnumDef) -> Self {
        Self {
            def,
            inserted_index: None,
        }
    }

    /// Read-only access to the definition to be added.
    pub fn def(&self) -> &EnumDef {
        &self.def
    }
}

impl Command for AddEnumDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if p.defs.enum_def(self.def.id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "EnumDef with id {:?} already exists",
                self.def.id
            )));
        }

        let idx = self.inserted_index.unwrap_or(p.defs.enums.len());
        if idx <= p.defs.enums.len() {
            p.defs.enums.insert(idx, self.def.clone());
        } else {
            p.defs.enums.push(self.def.clone());
        }
        self.inserted_index = Some(idx);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(pos) = p.defs.enums.iter().position(|e| e.id == self.def.id) {
            p.defs.enums.remove(pos);
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Enum '{}'", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to update an existing `EnumDef`'s metadata (name, values) with stroke merge support.
#[derive(Debug, Clone)]
pub struct UpdateEnumDefCommand {
    def: EnumDef,
    old_def: Option<EnumDef>,
    stroke_id: Option<u64>,
}

impl UpdateEnumDefCommand {
    /// Creates a command updating an enum definition.
    pub fn new(def: EnumDef) -> Self {
        Self {
            def,
            old_def: None,
            stroke_id: None,
        }
    }

    /// Creates a command with an associated stroke ID for drag/keystroke merging.
    pub fn with_stroke(def: EnumDef, stroke_id: u64) -> Self {
        Self {
            def,
            old_def: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for UpdateEnumDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let existing = p.defs.enum_def_mut(self.def.id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.def.id))
        })?;

        if self.old_def.is_none() {
            self.old_def = Some(existing.clone());
        }

        *existing = self.def.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(old), Some(curr)) = (&self.old_def, p.defs.enum_def_mut(self.def.id)) {
            *curr = old.clone();
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<UpdateEnumDefCommand>() {
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
        format!("Edit Enum '{}'", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to remove an `EnumDef` with reference blocking or cascading cleanup.
#[derive(Debug, Clone)]
pub struct RemoveEnumDefCommand {
    id: EnumDefId,
    cascade: bool,
    removed_index_and_def: Option<(usize, EnumDef)>,
    cascaded_fields: Vec<(EntityDefId, usize, FieldDef)>,
}

impl RemoveEnumDefCommand {
    /// Creates a command to safely remove an enum definition, blocking if referenced.
    pub fn safe(id: EnumDefId) -> Self {
        Self {
            id,
            cascade: false,
            removed_index_and_def: None,
            cascaded_fields: Vec::new(),
        }
    }

    /// Creates a command to cascade removal of referencing entity fields.
    pub fn cascade(id: EnumDefId) -> Self {
        Self {
            id,
            cascade: true,
            removed_index_and_def: None,
            cascaded_fields: Vec::new(),
        }
    }
}

impl Command for RemoveEnumDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let refs = find_enum_def_references(p, self.id);
        if !refs.is_empty() && !self.cascade {
            return Err(CoreError::DefNotFound(format!(
                "Cannot delete EnumDef {:?} because it is referenced by {} entity field(s)",
                self.id,
                refs.len()
            )));
        }

        let pos = p
            .defs
            .enums
            .iter()
            .position(|e| e.id == self.id)
            .ok_or_else(|| {
                CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.id))
            })?;

        // If cascading, remove referencing fields from entity definitions
        if self.cascade {
            self.cascaded_fields.clear();
            for entity in &mut p.defs.entities {
                let mut f_idx = 0;
                while f_idx < entity.fields.len() {
                    if field_type_references_enum(&entity.fields[f_idx].field_type, self.id) {
                        let removed = entity.fields.remove(f_idx);
                        self.cascaded_fields.push((entity.id, f_idx, removed));
                    } else {
                        f_idx += 1;
                    }
                }
            }
        }

        let def = p.defs.enums.remove(pos);
        self.removed_index_and_def = Some((pos, def));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some((pos, def)) = self.removed_index_and_def.take() {
            if pos <= p.defs.enums.len() {
                p.defs.enums.insert(pos, def);
            } else {
                p.defs.enums.push(def);
            }
        }

        // Restore any cascaded fields in reverse order to preserve indices
        for (ent_id, idx, field) in self.cascaded_fields.drain(..).rev() {
            if let Some(entity) = p.defs.entity_mut(ent_id) {
                if idx <= entity.fields.len() {
                    entity.fields.insert(idx, field);
                } else {
                    entity.fields.push(field);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Enum #{}", self.id.0)
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to add an `EnumValueDef` to an `EnumDef`.
#[derive(Debug, Clone)]
pub struct AddEnumValueCommand {
    enum_id: EnumDefId,
    value: EnumValueDef,
    inserted_index: Option<usize>,
}

impl AddEnumValueCommand {
    /// Creates a command adding a new enum value option.
    pub fn new(enum_id: EnumDefId, value: EnumValueDef) -> Self {
        Self {
            enum_id,
            value,
            inserted_index: None,
        }
    }
}

impl Command for AddEnumValueCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enum_def = p.defs.enum_def_mut(self.enum_id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
        })?;

        if enum_def.value(&self.value.id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "EnumValue with id '{}' already exists in enum {:?}",
                self.value.id, self.enum_id
            )));
        }

        let idx = self.inserted_index.unwrap_or(enum_def.values.len());
        if idx <= enum_def.values.len() {
            enum_def.values.insert(idx, self.value.clone());
        } else {
            enum_def.values.push(self.value.clone());
        }
        self.inserted_index = Some(idx);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(enum_def) = p.defs.enum_def_mut(self.enum_id) {
            if let Some(pos) = enum_def.values.iter().position(|v| v.id == self.value.id) {
                enum_def.values.remove(pos);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Enum Value '{}'", self.value.id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.enum_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to update an `EnumValueDef` (name, color, tile) with stroke merge support.
#[derive(Debug, Clone)]
pub struct UpdateEnumValueCommand {
    enum_id: EnumDefId,
    value: EnumValueDef,
    old_value: Option<EnumValueDef>,
    stroke_id: Option<u64>,
}

impl UpdateEnumValueCommand {
    /// Creates a command updating an enum value definition.
    pub fn new(enum_id: EnumDefId, value: EnumValueDef) -> Self {
        Self {
            enum_id,
            value,
            old_value: None,
            stroke_id: None,
        }
    }

    /// Creates a command with an associated stroke ID for merging rapid keystrokes/color adjustments.
    pub fn with_stroke(enum_id: EnumDefId, value: EnumValueDef, stroke_id: u64) -> Self {
        Self {
            enum_id,
            value,
            old_value: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for UpdateEnumValueCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enum_def = p.defs.enum_def_mut(self.enum_id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
        })?;

        let existing = enum_def.value_mut(&self.value.id).ok_or_else(|| {
            CoreError::DefNotFound(format!(
                "EnumValue '{}' not found in enum {:?}",
                self.value.id, self.enum_id
            ))
        })?;

        if self.old_value.is_none() {
            self.old_value = Some(existing.clone());
        }

        *existing = self.value.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(old), Some(enum_def)) = (&self.old_value, p.defs.enum_def_mut(self.enum_id)) {
            if let Some(existing) = enum_def.value_mut(&self.value.id) {
                *existing = old.clone();
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<UpdateEnumValueCommand>() {
            if self.enum_id == other.enum_id
                && self.value.id == other.value.id
                && self.stroke_id.is_some()
                && self.stroke_id == other.stroke_id
            {
                self.value = other.value.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Enum Value '{}'", self.value.id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.enum_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to remove an `EnumValueDef` with reference blocking or cascading cleanup.
#[derive(Debug, Clone)]
pub struct RemoveEnumValueCommand {
    enum_id: EnumDefId,
    val_id: String,
    cascade: bool,
    removed_index_and_val: Option<(usize, EnumValueDef)>,
    cleared_defaults: Vec<(EntityDefId, String, FieldValue)>,
}

impl RemoveEnumValueCommand {
    /// Creates a safe command that blocks removal if the enum value is referenced.
    pub fn safe(enum_id: EnumDefId, val_id: impl Into<String>) -> Self {
        Self {
            enum_id,
            val_id: val_id.into(),
            cascade: false,
            removed_index_and_val: None,
            cleared_defaults: Vec::new(),
        }
    }

    /// Creates a cascading command that clears referencing default values.
    pub fn cascade(enum_id: EnumDefId, val_id: impl Into<String>) -> Self {
        Self {
            enum_id,
            val_id: val_id.into(),
            cascade: true,
            removed_index_and_val: None,
            cleared_defaults: Vec::new(),
        }
    }
}

impl Command for RemoveEnumValueCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let refs = find_enum_value_references(p, self.enum_id, &self.val_id);
        if !refs.is_empty() && !self.cascade {
            return Err(CoreError::DefNotFound(format!(
                "Cannot delete EnumValue '{}' in enum {:?} because it is referenced in {} location(s)",
                self.val_id,
                self.enum_id,
                refs.len()
            )));
        }

        let pos = {
            let enum_def = p.defs.enum_def(self.enum_id).ok_or_else(|| {
                CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
            })?;

            enum_def
                .values
                .iter()
                .position(|v| v.id == self.val_id)
                .ok_or_else(|| {
                    CoreError::DefNotFound(format!(
                        "EnumValue '{}' not found in enum {:?}",
                        self.val_id, self.enum_id
                    ))
                })?
        };

        // If cascading, clear default values referencing this value
        if self.cascade {
            self.cleared_defaults.clear();
            for entity in &mut p.defs.entities {
                for field in &mut entity.fields {
                    if field_type_references_enum(&field.field_type, self.enum_id) {
                        if let Some(ref def_val) = field.default_value {
                            if field_value_uses_enum_val(def_val, &self.val_id) {
                                let old_val = field.default_value.take().unwrap();
                                self.cleared_defaults.push((
                                    entity.id,
                                    field.name.clone(),
                                    old_val,
                                ));
                            }
                        }
                    }
                }
            }
        }

        let enum_def = p.defs.enum_def_mut(self.enum_id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
        })?;
        let removed = enum_def.values.remove(pos);
        self.removed_index_and_val = Some((pos, removed));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some((pos, val)), Some(enum_def)) = (
            self.removed_index_and_val.take(),
            p.defs.enum_def_mut(self.enum_id),
        ) {
            if pos <= enum_def.values.len() {
                enum_def.values.insert(pos, val);
            } else {
                enum_def.values.push(val);
            }
        }

        // Restore cleared default values
        for (ent_id, fname, fval) in self.cleared_defaults.drain(..) {
            if let Some(entity) = p.defs.entity_mut(ent_id) {
                if let Some(field) = entity.fields.iter_mut().find(|f| f.name == fname) {
                    field.default_value = Some(fval);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Enum Value '{}'", self.val_id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.enum_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to reorder values within an `EnumDef`.
#[derive(Debug, Clone)]
pub struct ReorderEnumValuesCommand {
    enum_id: EnumDefId,
    new_order: Vec<String>,
    old_order: Option<Vec<String>>,
}

impl ReorderEnumValuesCommand {
    /// Creates a command specifying a new sequence of value IDs.
    pub fn new(enum_id: EnumDefId, new_order: Vec<String>) -> Self {
        Self {
            enum_id,
            new_order,
            old_order: None,
        }
    }
}

impl Command for ReorderEnumValuesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enum_def = p.defs.enum_def_mut(self.enum_id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
        })?;

        if self.old_order.is_none() {
            self.old_order = Some(enum_def.values.iter().map(|v| v.id.clone()).collect());
        }

        let mut reordered = Vec::with_capacity(enum_def.values.len());
        for id in &self.new_order {
            if let Some(pos) = enum_def.values.iter().position(|v| v.id == *id) {
                reordered.push(enum_def.values.remove(pos));
            }
        }
        // Append any remaining values not explicitly mentioned
        reordered.append(&mut enum_def.values);
        enum_def.values = reordered;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(old_order), Some(enum_def)) =
            (&self.old_order, p.defs.enum_def_mut(self.enum_id))
        {
            let mut reordered = Vec::with_capacity(enum_def.values.len());
            for id in old_order {
                if let Some(pos) = enum_def.values.iter().position(|v| v.id == *id) {
                    reordered.push(enum_def.values.remove(pos));
                }
            }
            reordered.append(&mut enum_def.values);
            enum_def.values = reordered;
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Reorder Enum #{} Values", self.enum_id.0)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.enum_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to rename an `EnumValueDef`'s string ID, cascading the rename to all stored instance values.
#[derive(Debug, Clone)]
pub struct RenameEnumValueIdCommand {
    enum_id: EnumDefId,
    old_id: String,
    new_id: String,
    affected_defaults: Vec<(EntityDefId, String)>,
    affected_instances: Vec<(WorldId, LevelId, EntityId, String)>,
}

impl RenameEnumValueIdCommand {
    /// Creates a command renaming an enum value's key identifier and cascading to all instances.
    pub fn new(enum_id: EnumDefId, old_id: impl Into<String>, new_id: impl Into<String>) -> Self {
        Self {
            enum_id,
            old_id: old_id.into(),
            new_id: new_id.into(),
            affected_defaults: Vec::new(),
            affected_instances: Vec::new(),
        }
    }
}

impl Command for RenameEnumValueIdCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let enum_def = p.defs.enum_def_mut(self.enum_id).ok_or_else(|| {
            CoreError::DefNotFound(format!("EnumDef with id {:?} not found", self.enum_id))
        })?;

        if enum_def.value(&self.new_id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "EnumValue with id '{}' already exists in enum {:?}",
                self.new_id, self.enum_id
            )));
        }

        let val = enum_def.value_mut(&self.old_id).ok_or_else(|| {
            CoreError::DefNotFound(format!(
                "EnumValue '{}' not found in enum {:?}",
                self.old_id, self.enum_id
            ))
        })?;
        val.id = self.new_id.clone();

        // Cascade to entity field defaults
        self.affected_defaults.clear();
        for entity in &mut p.defs.entities {
            for field in &mut entity.fields {
                if field_type_references_enum(&field.field_type, self.enum_id) {
                    if let Some(ref mut def_val) = field.default_value {
                        if rename_field_val_enum(def_val, &self.old_id, &self.new_id) {
                            self.affected_defaults.push((entity.id, field.name.clone()));
                        }
                    }
                }
            }
        }

        // Cascade to level entity instances
        self.affected_instances.clear();
        for world in &mut p.worlds {
            for level in &mut world.levels {
                for layer in &mut level.layers {
                    if let lc_core::layer::LayerData::Entities(ref mut entities) = layer.data {
                        for ent in entities {
                            let keys: Vec<String> =
                                ent.fields.iter().map(|(k, _)| k.clone()).collect();
                            for fname in keys {
                                if let Some(fval) = ent.fields.get_mut(&fname) {
                                    if rename_field_val_enum(fval, &self.old_id, &self.new_id) {
                                        self.affected_instances
                                            .push((world.id, level.id, ent.id, fname));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(enum_def) = p.defs.enum_def_mut(self.enum_id) {
            if let Some(val) = enum_def.value_mut(&self.new_id) {
                val.id = self.old_id.clone();
            }
        }

        for (ent_id, fname) in &self.affected_defaults {
            if let Some(entity) = p.defs.entity_mut(*ent_id) {
                if let Some(field) = entity.fields.iter_mut().find(|f| f.name == *fname) {
                    if let Some(ref mut def_val) = field.default_value {
                        let _ = rename_field_val_enum(def_val, &self.new_id, &self.old_id);
                    }
                }
            }
        }

        for (wid, lid, eid, fname) in &self.affected_instances {
            if let Some(world) = p.world_mut(*wid) {
                if let Some(level) = world.level_mut(*lid) {
                    for layer in &mut level.layers {
                        if let lc_core::layer::LayerData::Entities(ref mut entities) = layer.data {
                            if let Some(ent) = entities.iter_mut().find(|e| e.id == *eid) {
                                if let Some(fval) = ent.fields.get_mut(fname) {
                                    let _ = rename_field_val_enum(fval, &self.new_id, &self.old_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Rename Enum Value '{}' -> '{}'", self.old_id, self.new_id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::EnumDef(self.enum_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn rename_field_val_enum(fv: &mut FieldValue, old_id: &str, new_id: &str) -> bool {
    match fv {
        FieldValue::Enum(v) if v == old_id => {
            *v = new_id.to_string();
            true
        }
        FieldValue::Array(items) => {
            let mut changed = false;
            for item in items {
                if rename_field_val_enum(item, old_id, new_id) {
                    changed = true;
                }
            }
            changed
        }
        _ => false,
    }
}
