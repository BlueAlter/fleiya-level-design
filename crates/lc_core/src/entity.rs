//! Entity definition templates and LDtk-compatible rendering/limit configuration.

use crate::defs::EnumDef;
use crate::fields::{FieldDef, FieldValue, FieldValues};
use crate::id::{EntityDefId, FieldDefId, TilesetDefId};
use crate::layer::TileRef;
use glam::{UVec2, Vec2};
use serde::{Deserialize, Serialize};

/// Pixel rectangle within a tileset image (supports single or multi-tile regions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TilesetRect {
    /// Source tileset definition ID.
    pub tileset_id: TilesetDefId,
    /// Top-left X coordinate in pixels inside the tileset image.
    pub x: u32,
    /// Top-left Y coordinate in pixels inside the tileset image.
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

impl TilesetRect {
    /// Creates a new tileset pixel rectangle.
    pub const fn new(tileset_id: TilesetDefId, x: u32, y: u32, w: u32, h: u32) -> Self {
        Self {
            tileset_id,
            x,
            y,
            w,
            h,
        }
    }
}

/// Visual shape mode used to render an entity in the editor (LDtk `RenderMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntityRenderMode {
    #[default]
    Rectangle,
    Ellipse,
    Tile,
    Cross,
}

/// How an entity's tile texture scales inside its bounding box (LDtk `TileRenderMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TileRenderMode {
    Cover,
    #[default]
    FitInside,
    Repeat,
    Stretch,
    FullSizeCropped,
    FullSizeUncropped,
    NineSlice,
}

/// Scope over which `EntityDef::max_count` is enforced (LDtk `LimitScope`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntityLimitScope {
    PerLayer,
    #[default]
    PerLevel,
    PerWorld,
}

/// Behavior when placing an entity exceeds `EntityDef::max_count` (LDtk `LimitBehavior`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntityLimitBehavior {
    DiscardOldOnes,
    PreventAdding,
    #[default]
    MoveLastOne,
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

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

fn is_zero_borders(v: &[u32; 4]) -> bool {
    *v == [0, 0, 0, 0]
}

fn is_one_f32(v: &f32) -> bool {
    (*v - 1.0).abs() <= f32::EPSILON
}

fn is_default_render_mode(v: &EntityRenderMode) -> bool {
    *v == EntityRenderMode::Rectangle
}

fn is_default_tile_render_mode(v: &TileRenderMode) -> bool {
    *v == TileRenderMode::FitInside
}

fn is_default_limit_scope(v: &EntityLimitScope) -> bool {
    *v == EntityLimitScope::PerLevel
}

fn is_default_limit_behavior(v: &EntityLimitBehavior) -> bool {
    *v == EntityLimitBehavior::MoveLastOne
}

/// Definition of an entity template (100% LDtk parity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDef {
    /// Stable identifier of the entity definition.
    pub id: EntityDefId,
    /// Entity class name (e.g. "Player", "Coin").
    pub name: String,
    /// Default size in pixels.
    pub size_px: UVec2,
    /// Display color in the editor.
    pub color: [u8; 4],
    /// Alignment pivot point `[0.0..1.0, 0.0..1.0]`.
    pub pivot: Vec2,
    /// Optional legacy single-tile representation for display in canvas.
    pub tile: Option<TileRef>,
    /// Custom field schema definitions.
    pub fields: Vec<FieldDef>,
    /// Tags assigned to this entity definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional human-readable documentation / notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Whether instances are exported to the Table of Contents.
    #[serde(default, skip_serializing_if = "is_false")]
    pub export_to_toc: bool,
    /// Whether instances can be placed partially or completely outside level bounds.
    #[serde(default, skip_serializing_if = "is_false")]
    pub allow_out_of_bounds: bool,
    /// Whether instances can be resized horizontally on the level canvas.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resizable_x: bool,
    /// Whether instances can be resized vertically on the level canvas.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resizable_y: bool,
    /// Whether resizing preserves the original width/height aspect ratio.
    #[serde(default, skip_serializing_if = "is_false")]
    pub keep_aspect_ratio: bool,
    /// Maximum allowed instances (`0` = unlimited).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub max_count: u32,
    /// Scope used to evaluate `max_count`.
    #[serde(default, skip_serializing_if = "is_default_limit_scope")]
    pub limit_scope: EntityLimitScope,
    /// Action taken when `max_count` is reached.
    #[serde(default, skip_serializing_if = "is_default_limit_behavior")]
    pub limit_behavior: EntityLimitBehavior,
    /// Primary shape/tile rendering mode in the editor.
    #[serde(default, skip_serializing_if = "is_default_render_mode")]
    pub render_mode: EntityRenderMode,
    /// Optional source tileset selected when `render_mode == EntityRenderMode::Tile`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tileset_id: Option<TilesetDefId>,
    /// Scaling mode when rendering a tile for this entity.
    #[serde(default, skip_serializing_if = "is_default_tile_render_mode")]
    pub tile_render_mode: TileRenderMode,
    /// Optional tileset pixel rectangle (supports multi-tile regions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_rect: Option<TilesetRect>,
    /// Optional separate tileset pixel rectangle used as UI icon in panels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_tile_rect: Option<TilesetRect>,
    /// 9-slice border margins `[top, right, bottom, left]` in pixels.
    #[serde(default, skip_serializing_if = "is_zero_borders")]
    pub nine_slice_borders: [u32; 4],
    /// Opacity of the entity's display tile in `0.0..=1.0`.
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub tile_opacity: f32,
    /// Opacity of the entity's background fill in `0.0..=1.0`.
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub fill_opacity: f32,
    /// Opacity of the entity's outline stroke in `0.0..=1.0`.
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub line_opacity: f32,
    /// When `true`, renders hollow interior and only hits border edges on click.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hollow: bool,
    /// Whether the entity identifier label is shown on the canvas.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub show_name: bool,
}

impl Default for EntityDef {
    fn default() -> Self {
        Self {
            id: EntityDefId(0),
            name: String::new(),
            size_px: UVec2::new(16, 16),
            color: [148, 200, 120, 255],
            pivot: Vec2::new(0.5, 0.5),
            tile: None,
            fields: Vec::new(),
            tags: Vec::new(),
            doc: None,
            export_to_toc: false,
            allow_out_of_bounds: false,
            resizable_x: false,
            resizable_y: false,
            keep_aspect_ratio: false,
            max_count: 0,
            limit_scope: EntityLimitScope::PerLevel,
            limit_behavior: EntityLimitBehavior::MoveLastOne,
            render_mode: EntityRenderMode::Rectangle,
            tileset_id: None,
            tile_render_mode: TileRenderMode::FitInside,
            tile_rect: None,
            ui_tile_rect: None,
            nine_slice_borders: [0, 0, 0, 0],
            tile_opacity: 1.0,
            fill_opacity: 1.0,
            line_opacity: 1.0,
            hollow: false,
            show_name: true,
        }
    }
}

impl EntityDef {
    /// Creates a new entity definition with LDtk-compatible defaults.
    pub fn new(id: EntityDefId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            ..Self::default()
        }
    }

    /// Finds a field definition by its name.
    pub fn field_def(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Finds a field definition by its stable ID.
    pub fn field_def_by_id(&self, id: FieldDefId) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Computes LDtk's `__smartColor` from the first field marked `use_for_smart_color`.
    ///
    /// Inspects `Color` fields directly and `Enum` fields via their enum value's color,
    /// falling back to `self.color` when no smart-color field has a non-null value.
    pub fn smart_color(&self, fields: &FieldValues, enums: &[EnumDef]) -> [u8; 4] {
        for field in &self.fields {
            if !field.use_for_smart_color {
                continue;
            }
            let Some(value) = fields.get(&field.name) else {
                continue;
            };
            match value {
                FieldValue::Color(rgba) => return *rgba,
                FieldValue::Enum(val_id) => {
                    if let crate::fields::FieldType::Enum(enum_id) = field.field_type {
                        if let Some(val_def) = enums
                            .iter()
                            .find(|e| e.id == enum_id)
                            .and_then(|e| e.value(val_id))
                        {
                            return val_def.color;
                        }
                    }
                }
                _ => {}
            }
        }
        self.color
    }
}
