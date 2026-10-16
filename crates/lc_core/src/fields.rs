//! Dynamic field system for entities and levels.

use crate::id::{EntityId, EnumDefId};
use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Supported primitive and structured field types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    /// 64-bit signed integer.
    Int,
    /// 64-bit floating point number.
    Float,
    /// Boolean flag.
    Bool,
    /// UTF-8 text string.
    String,
    /// 32-bit RGBA color `[R, G, B, A]`.
    Color,
    /// Enum reference identified by `EnumDefId`.
    Enum(EnumDefId),
    /// 2D grid/pixel integer point coordinates.
    Point,
    /// Weak reference to another entity by `EntityId`.
    EntityRef,
    /// Homogeneous collection of items of the inner field type.
    Array(Box<FieldType>),
}

/// Dynamic concrete field value corresponding to a `FieldType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Text string value.
    String(String),
    /// RGBA color value.
    Color([u8; 4]),
    /// Selected enum value name.
    Enum(String),
    /// 2D integer point value.
    Point(IVec2),
    /// Reference to an entity, if set.
    EntityRef(Option<EntityId>),
    /// Array of child field values.
    Array(Vec<FieldValue>),
}

/// Definition of a custom field schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field identifier key.
    pub name: String,
    /// Expected field type.
    pub field_type: FieldType,
    /// Optional default value for newly instantiated instances.
    pub default_value: Option<FieldValue>,
}

/// Container storing key-value pairs of dynamic field values.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FieldValues {
    values: BTreeMap<String, FieldValue>,
}

impl FieldValues {
    /// Creates an empty collection of field values.
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Retrieves an immutable reference to the value associated with `key`.
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.values.get(key)
    }

    /// Retrieves a mutable reference to the value associated with `key`.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut FieldValue> {
        self.values.get_mut(key)
    }

    /// Inserts or updates the value associated with `key`.
    pub fn set(&mut self, key: impl Into<String>, value: FieldValue) -> Option<FieldValue> {
        self.values.insert(key.into(), value)
    }

    /// Removes and returns the value associated with `key`, if present.
    pub fn remove(&mut self, key: &str) -> Option<FieldValue> {
        self.values.remove(key)
    }

    /// Returns `true` if there are no field values stored.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the total count of stored field values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Iterates over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FieldValue)> {
        self.values.iter()
    }
}
