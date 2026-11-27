//! Dynamic field system for entities and levels.

use crate::defs::TilesetRect;
use crate::id::{EntityDefId, EntityId, EnumDefId, FieldDefId, TilesetDefId};
use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Supported primitive and structured field types (100% LDtk parity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    /// 64-bit signed integer.
    Int,
    /// 64-bit floating point number.
    Float,
    /// Boolean flag.
    Bool,
    /// Single-line UTF-8 text string.
    String,
    /// Multi-line text string with optional syntax language mode.
    Multilines,
    /// 32-bit RGBA color `[R, G, B, A]`.
    Color,
    /// Enum reference identified by `EnumDefId`.
    Enum(EnumDefId),
    /// 2D grid/pixel integer point coordinates.
    Point,
    /// Relative file path string.
    FilePath,
    /// Weak reference to another entity by `EntityId`.
    EntityRef,
    /// Reference to a tile region inside a tileset.
    Tile,
    /// Homogeneous collection of items of the inner field type.
    Array(Box<FieldType>),
}

/// Dynamic concrete field value corresponding to a `FieldType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    /// Explicit null value (when `FieldDef::can_be_null` is enabled).
    Null,
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Single-line text string value.
    String(String),
    /// Multi-line text string value.
    Multilines(String),
    /// RGBA color value.
    Color([u8; 4]),
    /// Selected enum value name.
    Enum(String),
    /// 2D integer point value.
    Point(IVec2),
    /// Relative file path value.
    FilePath(String),
    /// Reference to an entity, if set.
    EntityRef(Option<EntityId>),
    /// Tile region value, if set.
    Tile(Option<TilesetRect>),
    /// Array of child field values.
    Array(Vec<FieldValue>),
}

/// Language mode for `FieldType::Multilines` syntax/editor hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextLanguageMode {
    #[default]
    Plain,
    Python,
    Ruby,
    Js,
    Lua,
    C,
    Haxe,
    Markdown,
    Json,
    Xml,
    Log,
}

/// Restriction on which entity definitions an `EntityRef` field can target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AllowedRefs {
    /// Any entity in the project.
    Any,
    /// Only entities sharing the same definition as the owner.
    #[default]
    OnlySame,
    /// Only entities carrying at least one tag in `allowed_ref_tags`.
    OnlyTags,
    /// Only entities of a specific `allowed_refs_entity_id`.
    OnlySpecificEntity,
}

/// How a field value is displayed on the level canvas (LDtk `EditorDisplayMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EditorDisplayMode {
    #[default]
    Hidden,
    ValueOnly,
    NameAndValue,
    EntityTile,
    LevelTile,
    Points,
    PointStar,
    PointPath,
    PointPathLoop,
    RadiusPx,
    RadiusGrid,
    ArrayCountWithLabel,
    ArrayCountNoLabel,
    RefLinkBetweenPivots,
    RefLinkBetweenCenters,
}

/// Vertical anchor position for field display labels on the level canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EditorDisplayPos {
    #[default]
    Above,
    Center,
    Beneath,
}

/// Visual line style for `EntityRef` links drawn on the level canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EditorLinkStyle {
    ZigZag,
    #[default]
    StraightArrow,
    CurvedArrow,
    ArrowsLine,
    DashedLine,
}

fn default_true() -> bool {
    true
}

fn default_one_f32() -> f32 {
    1.0
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_true(v: &bool) -> bool {
    *v
}

fn is_one_f32(v: &f32) -> bool {
    (*v - 1.0).abs() <= f32::EPSILON
}

fn is_zero_field_id(id: &FieldDefId) -> bool {
    id.0 == 0
}

fn is_default_allowed_refs(v: &AllowedRefs) -> bool {
    *v == AllowedRefs::OnlySame
}

fn is_default_display_mode(v: &EditorDisplayMode) -> bool {
    *v == EditorDisplayMode::Hidden
}

fn is_default_display_pos(v: &EditorDisplayPos) -> bool {
    *v == EditorDisplayPos::Above
}

fn is_default_link_style(v: &EditorLinkStyle) -> bool {
    *v == EditorLinkStyle::StraightArrow
}

/// Definition of a custom field schema (100% LDtk parity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Stable identifier of this field definition (0 if legacy/unassigned).
    #[serde(default, skip_serializing_if = "is_zero_field_id")]
    pub id: FieldDefId,
    /// Field identifier key.
    pub name: String,
    /// Expected field type.
    pub field_type: FieldType,
    /// Optional default value for newly instantiated instances.
    pub default_value: Option<FieldValue>,
    /// Optional human-readable documentation / tooltip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Whether instances can store `FieldValue::Null`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub can_be_null: bool,
    /// Optional override value used when a nullable field is null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_override: Option<FieldValue>,
    /// Optional numeric minimum bound (for `Int` / `Float`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Optional numeric maximum bound (for `Int` / `Float`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Optional regex validation pattern (for `String` / `Multilines`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    /// Optional minimum element count for `Array` fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array_min_length: Option<u32>,
    /// Optional maximum element count for `Array` fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array_max_length: Option<u32>,
    /// Optional language mode for `Multilines` fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_language_mode: Option<TextLanguageMode>,
    /// Accepted file extensions for `FilePath` fields (empty = any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accept_file_types: Vec<String>,
    /// Target entity restriction for `EntityRef` fields.
    #[serde(default, skip_serializing_if = "is_default_allowed_refs")]
    pub allowed_refs: AllowedRefs,
    /// Target entity definition ID when `allowed_refs == AllowedRefs::OnlySpecificEntity`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_refs_entity_id: Option<EntityDefId>,
    /// Target tags when `allowed_refs == AllowedRefs::OnlyTags`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ref_tags: Vec<String>,
    /// Whether setting an `EntityRef` also updates the target entity's matching ref.
    #[serde(default, skip_serializing_if = "is_false")]
    pub symmetrical_ref: bool,
    /// Whether creating new entities auto-chains `EntityRef` from the previous entity.
    #[serde(default, skip_serializing_if = "is_false")]
    pub auto_chain_ref: bool,
    /// Whether `Point` fields can be placed outside level bounds.
    #[serde(default, skip_serializing_if = "is_false")]
    pub allow_out_of_bounds: bool,
    /// Whether this field provides the entity's dynamic Smart Color.
    #[serde(default, skip_serializing_if = "is_false")]
    pub use_for_smart_color: bool,
    /// Whether this field is indexed/searchable in the Table of Contents.
    #[serde(default, skip_serializing_if = "is_false")]
    pub searchable: bool,
    /// How this field is visualized on the level canvas.
    #[serde(default, skip_serializing_if = "is_default_display_mode")]
    pub editor_display_mode: EditorDisplayMode,
    /// Relative vertical anchor for canvas value display.
    #[serde(default, skip_serializing_if = "is_default_display_pos")]
    pub editor_display_pos: EditorDisplayPos,
    /// Font/gizmo scale multiplier in `0.1..=4.0`.
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub editor_display_scale: f32,
    /// Link drawing style for `EntityRef` canvas visualization.
    #[serde(default, skip_serializing_if = "is_default_link_style")]
    pub editor_link_style: EditorLinkStyle,
    /// Show field in inspector/canvas even when default or empty.
    #[serde(default, skip_serializing_if = "is_false")]
    pub editor_always_show: bool,
    /// Truncate long text values in canvas display.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub editor_cut_long_values: bool,
    /// Optional unit prefix prepended to canvas display (e.g. "$").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_text_prefix: Option<String>,
    /// Optional unit suffix appended to canvas display (e.g. "px", "hp").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_text_suffix: Option<String>,
    /// Whether this field is included when parent Entity `export_to_toc` is enabled.
    #[serde(default, skip_serializing_if = "is_false")]
    pub export_to_toc: bool,
    /// Optional tileset restriction/selection for `Tile` fields (`LDtk tilesetUid`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tileset_id: Option<TilesetDefId>,
    /// Show this field in world overview rendering.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub editor_show_in_world: bool,
}

impl Default for FieldDef {
    fn default() -> Self {
        Self {
            id: FieldDefId(0),
            name: String::new(),
            field_type: FieldType::Int,
            default_value: None,
            doc: None,
            can_be_null: false,
            default_override: None,
            min: None,
            max: None,
            regex: None,
            array_min_length: None,
            array_max_length: None,
            text_language_mode: None,
            accept_file_types: Vec::new(),
            allowed_refs: AllowedRefs::OnlySame,
            allowed_refs_entity_id: None,
            allowed_ref_tags: Vec::new(),
            symmetrical_ref: false,
            auto_chain_ref: false,
            allow_out_of_bounds: false,
            use_for_smart_color: false,
            searchable: false,
            editor_display_mode: EditorDisplayMode::Hidden,
            editor_display_pos: EditorDisplayPos::Above,
            editor_display_scale: 1.0,
            editor_link_style: EditorLinkStyle::StraightArrow,
            editor_always_show: false,
            editor_cut_long_values: true,
            editor_text_prefix: None,
            editor_text_suffix: None,
            export_to_toc: false,
            tileset_id: None,
            editor_show_in_world: true,
        }
    }
}

impl FieldDef {
    /// Creates a new field definition with LDtk defaults.
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            ..Self::default()
        }
    }

    /// Returns true if the inner type (unwrapping `Array`) matches `predicate`.
    pub fn base_type(&self) -> &FieldType {
        match &self.field_type {
            FieldType::Array(inner) => inner.as_ref(),
            other => other,
        }
    }

    /// Returns true if this field is an array field.
    pub fn is_array(&self) -> bool {
        matches!(self.field_type, FieldType::Array(_))
    }
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
