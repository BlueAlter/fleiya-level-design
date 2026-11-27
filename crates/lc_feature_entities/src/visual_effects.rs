//! Sample value generation, LDtk visual-effect classification, and overlay formatting
//! shared between level canvas overlays and the Entity Workspace Live Preview.

use glam::IVec2;
use lc_core::defs::{EntityDef, EnumDef, SmartTileSource, TilesetDef, TilesetRect};
use lc_core::entity::{extract_smart_color_from_value, extract_smart_tile_from_value};
use lc_core::fields::{
    EditorDisplayMode, EditorDisplayPos, FieldDef, FieldType, FieldValue, FieldValues,
};
use lc_core::id::EntityId;

use crate::field_defs::{clamp_numeric_value, format_field_value};

/// Sentinel ID base used for sample `EntityRef` targets in Entity Live Preview.
pub const PREVIEW_SAMPLE_ENTITY_ID_BASE: u64 = 900_000;

/// Visual effect a single field + sample/instance value produces on an Entity in LDtk.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldVisualImpact {
    /// Overrides the entity's base/fill/stroke/label color via `use_for_smart_color`.
    SmartColor([u8; 4]),
    /// Replaces the entity's displayed tile graphic via `EditorDisplayMode::EntityTile`.
    EntityTileOverride(SmartTileSource),
    /// Draws a pixel-space radius circle around the entity's pivot.
    RadiusPx(f32),
    /// Draws a grid-unit radius circle (`value * grid_px`) around the entity's pivot.
    RadiusGrid { cells: f32, radius_px: f32 },
    /// Draws diamond markers and optional star/path links for `Point` / `Array<Point>`.
    SpatialPoints {
        mode: EditorDisplayMode,
        count: usize,
    },
    /// Draws directional reference links to target entities.
    RefLinks {
        mode: EditorDisplayMode,
        count: usize,
    },
    /// Draws inline text/value, color disc(s), or tile badge(s) stacked at `pos`.
    CanvasBadge {
        pos: EditorDisplayPos,
        summary: String,
    },
    /// Configures level-tile mode (`LevelTile`), which applies to levels rather than entities.
    LevelTileOnly,
    /// Does not currently alter entity visuals (`Hidden` or missing required tile/target).
    NoVisualEffect(&'static str),
}

impl FieldVisualImpact {
    /// Returns `true` if this effect changes the entity's visual rendering.
    pub fn affects_entity(&self) -> bool {
        !matches!(self, Self::LevelTileOnly | Self::NoVisualEffect(_))
    }

    /// Concise single-line label describing this visual impact in Live Preview.
    pub fn label(&self) -> String {
        match self {
            Self::SmartColor(c) => {
                format!("Entity color #{:02X}{:02X}{:02X}", c[0], c[1], c[2])
            }
            Self::EntityTileOverride(_) => "Entity tile override".to_string(),
            Self::RadiusPx(r) => format!("Radius {r:.0} px"),
            Self::RadiusGrid { cells, radius_px } => {
                format!("Radius {cells:.1} grid ({radius_px:.0} px)")
            }
            Self::SpatialPoints { mode, count } => format!("{mode:?} ({count})"),
            Self::RefLinks { mode, count } => format!("{mode:?} ({count})"),
            Self::CanvasBadge { pos, summary } => format!("{pos:?}: {summary}"),
            Self::LevelTileOnly => "LevelTile only (no entity visual)".to_string(),
            Self::NoVisualEffect(reason) => format!("No visual ({reason})"),
        }
    }
}

/// Generates a representative sample [`FieldValue`] for any of LDtk's 11 field types
/// (both Single and `Array<T>`), respecting configured `default_value`, `min`/`max`,
/// `array_min_length`/`array_max_length`, project enums, and project tilesets.
pub fn sample_field_value(
    field: &FieldDef,
    enums: &[EnumDef],
    tilesets: &[TilesetDef],
    seed_index: usize,
) -> FieldValue {
    if !field.is_array() {
        if let Some(def_val) = field.default_value.as_ref() {
            if is_meaningful_preview_default(def_val) {
                return clamp_numeric_value(def_val.clone(), field.min, field.max);
            }
        }
        return sample_scalar_value(field.base_type(), field, enums, tilesets, seed_index, 0);
    }

    let min_len = field.array_min_length.unwrap_or(0) as usize;
    let max_len = field.array_max_length.unwrap_or(u32::MAX) as usize;
    let count = 3.clamp(min_len, max_len.max(min_len)).min(6);
    let base = field.base_type();
    FieldValue::Array(
        (0..count)
            .map(|i| sample_scalar_value(base, field, enums, tilesets, seed_index, i))
            .collect(),
    )
}

/// Builds a complete [`FieldValues`] map using [`sample_field_value`] for every field in `def`.
pub fn effective_preview_values(
    def: &EntityDef,
    enums: &[EnumDef],
    tilesets: &[TilesetDef],
) -> FieldValues {
    let mut values = FieldValues::new();
    for (idx, field) in def.fields.iter().enumerate() {
        values.set(
            field.name.clone(),
            sample_field_value(field, enums, tilesets, idx),
        );
    }
    values
}

/// Evaluates all visual effects that `field` + `value` produce on the entity in LDtk.
pub fn analyze_field_visual_effects(
    field: &FieldDef,
    value: &FieldValue,
    enums: &[EnumDef],
    grid_px: u32,
) -> Vec<FieldVisualImpact> {
    let mut impacts = Vec::new();
    let base = field.base_type();

    if field.use_for_smart_color {
        if let Some(rgba) = extract_smart_color_from_value(value, base, enums) {
            impacts.push(FieldVisualImpact::SmartColor(rgba));
        }
    }

    let mode = field.editor_display_mode;
    match mode {
        EditorDisplayMode::Hidden => {
            if impacts.is_empty() {
                impacts.push(FieldVisualImpact::NoVisualEffect("Hidden"));
            }
        }
        EditorDisplayMode::LevelTile => {
            impacts.push(FieldVisualImpact::LevelTileOnly);
        }
        EditorDisplayMode::EntityTile => {
            if let Some(src) = extract_smart_tile_from_value(value, base, enums) {
                impacts.push(FieldVisualImpact::EntityTileOverride(src));
            } else if impacts.is_empty() {
                impacts.push(FieldVisualImpact::NoVisualEffect("no tile assigned"));
            }
        }
        EditorDisplayMode::RadiusPx => {
            let r = extract_numeric(value).max(0.0);
            if r > 0.0 {
                impacts.push(FieldVisualImpact::RadiusPx(r));
            } else {
                impacts.push(FieldVisualImpact::NoVisualEffect("radius 0"));
            }
        }
        EditorDisplayMode::RadiusGrid => {
            let cells = extract_numeric(value).max(0.0);
            let r = cells * grid_px.max(1) as f32;
            if r > 0.0 {
                impacts.push(FieldVisualImpact::RadiusGrid {
                    cells,
                    radius_px: r,
                });
            } else {
                impacts.push(FieldVisualImpact::NoVisualEffect("radius 0"));
            }
        }
        EditorDisplayMode::Points
        | EditorDisplayMode::PointStar
        | EditorDisplayMode::PointPath
        | EditorDisplayMode::PointPathLoop => {
            let count = count_points(value);
            if count > 0 {
                impacts.push(FieldVisualImpact::SpatialPoints { mode, count });
            } else {
                impacts.push(FieldVisualImpact::NoVisualEffect("empty points"));
            }
        }
        EditorDisplayMode::RefLinkBetweenPivots | EditorDisplayMode::RefLinkBetweenCenters => {
            let count = count_entity_refs(value);
            if count > 0 {
                impacts.push(FieldVisualImpact::RefLinks { mode, count });
            } else {
                impacts.push(FieldVisualImpact::NoVisualEffect("no target ref"));
            }
        }
        EditorDisplayMode::ValueOnly
        | EditorDisplayMode::NameAndValue
        | EditorDisplayMode::ArrayCountWithLabel
        | EditorDisplayMode::ArrayCountNoLabel => {
            let summary = format_overlay_text(field, value, mode);
            let text_or_swatch = if matches!(base, FieldType::Color) && summary.is_empty() {
                format_field_value(value)
            } else {
                summary
            };
            impacts.push(FieldVisualImpact::CanvasBadge {
                pos: field.editor_display_pos,
                summary: text_or_swatch,
            });
        }
    }

    impacts
}

/// Formats field text according to LDtk `FieldInstanceRender` rules (`+`/`-` for Bool ValueOnly,
/// `"N value(s)"` for ArrayCount, 70-char / 2-line cut for `editor_cut_long_values`, and prefix/suffix).
pub fn format_overlay_text(
    field: &FieldDef,
    value: &FieldValue,
    mode: EditorDisplayMode,
) -> String {
    let mut txt = match mode {
        EditorDisplayMode::ArrayCountWithLabel => {
            let n = match value {
                FieldValue::Array(v) => v.len(),
                _ => 0,
            };
            format!("{}: {n} value(s)", field.name)
        }
        EditorDisplayMode::ArrayCountNoLabel => {
            let n = match value {
                FieldValue::Array(v) => v.len(),
                _ => 0,
            };
            format!("{n} value(s)")
        }
        EditorDisplayMode::ValueOnly | EditorDisplayMode::NameAndValue => {
            if matches!(field.base_type(), FieldType::Bool)
                && mode == EditorDisplayMode::ValueOnly
                && !field.is_array()
            {
                if let FieldValue::Bool(b) = value {
                    return format!("{}{}", if *b { "+" } else { "-" }, field.name);
                }
            }
            if matches!(field.base_type(), FieldType::Color)
                && matches!(
                    mode,
                    EditorDisplayMode::ValueOnly | EditorDisplayMode::NameAndValue
                )
            {
                // Swatch discs render next to the optional field name.
                return if mode == EditorDisplayMode::NameAndValue {
                    format!("{} =", field.name)
                } else {
                    String::new()
                };
            }

            let mut raw = format_field_value_for_overlay(value);
            if field.editor_cut_long_values {
                raw = cut_long_overlay_text(&raw);
            }
            if matches!(field.base_type(), FieldType::Int | FieldType::Float) {
                if let Some(ref prefix) = field.editor_text_prefix {
                    raw = format!("{prefix}{raw}");
                }
                if let Some(ref suffix) = field.editor_text_suffix {
                    raw.push_str(suffix);
                }
            }
            if mode == EditorDisplayMode::NameAndValue {
                format!("{} = {raw}", field.name)
            } else {
                raw
            }
        }
        _ => String::new(),
    };
    if txt.trim().is_empty() {
        txt.clear();
    }
    txt
}

fn format_field_value_for_overlay(value: &FieldValue) -> String {
    match value {
        FieldValue::EntityRef(Some(id)) if id.0 >= PREVIEW_SAMPLE_ENTITY_ID_BASE => {
            format!("Target#{}", id.0 - PREVIEW_SAMPLE_ENTITY_ID_BASE + 1)
        }
        FieldValue::Array(items) => items
            .iter()
            .map(format_field_value_for_overlay)
            .collect::<Vec<_>>()
            .join(", "),
        other => format_field_value(other),
    }
}

fn cut_long_overlay_text(raw: &str) -> String {
    let truncated: String = raw.chars().take(70).collect();
    let lines: Vec<&str> = truncated.split('\n').take(2).collect();
    let joined = lines.join("\n");
    if raw.chars().count() > 70 || raw.split('\n').count() > 2 {
        format!("{joined}…")
    } else {
        joined
    }
}

fn is_meaningful_preview_default(v: &FieldValue) -> bool {
    match v {
        FieldValue::Null => false,
        FieldValue::String(s) | FieldValue::Multilines(s) | FieldValue::FilePath(s) => {
            !s.is_empty()
        }
        FieldValue::EntityRef(opt) => opt.is_some(),
        FieldValue::Tile(opt) => opt.is_some(),
        _ => true,
    }
}

fn sample_scalar_value(
    base: &FieldType,
    field: &FieldDef,
    enums: &[EnumDef],
    tilesets: &[TilesetDef],
    seed_index: usize,
    elem_idx: usize,
) -> FieldValue {
    match base {
        FieldType::Int => {
            let vals = [42_i64, 18, 64, 24];
            clamp_numeric_value(
                FieldValue::Int(vals[elem_idx % vals.len()]),
                field.min,
                field.max,
            )
        }
        FieldType::Float => {
            let vals = [3.5_f64, 2.0, 4.25, 1.5];
            clamp_numeric_value(
                FieldValue::Float(vals[elem_idx % vals.len()]),
                field.min,
                field.max,
            )
        }
        FieldType::Bool => FieldValue::Bool(elem_idx.is_multiple_of(2)),
        FieldType::String => {
            let words = ["Alpha", "Beta", "Gamma", "Delta"];
            let val = if field.is_array() {
                words[elem_idx % words.len()].to_string()
            } else if field.editor_cut_long_values {
                "Sample entity text exceeding preview excerpt length cleanly for truncation"
                    .to_string()
            } else {
                format!("Sample {}", field.name)
            };
            FieldValue::String(val)
        }
        FieldType::Multilines => {
            let val = if field.is_array() {
                format!("Note {}\nLine 2", elem_idx + 1)
            } else {
                "Line 1: Sample\nLine 2: Notes\nLine 3: Extra".to_string()
            };
            FieldValue::Multilines(val)
        }
        FieldType::Color => {
            let palette = [
                [255, 140, 60, 255],
                [90, 210, 120, 255],
                [96, 165, 250, 255],
                [244, 114, 182, 255],
            ];
            FieldValue::Color(palette[(seed_index + elem_idx) % palette.len()])
        }
        FieldType::Enum(enum_id) => {
            let picked = enums
                .iter()
                .find(|e| e.id == *enum_id)
                .and_then(|ed| {
                    if ed.values.is_empty() {
                        None
                    } else {
                        Some(ed.values[elem_idx % ed.values.len()].id.clone())
                    }
                })
                .unwrap_or_else(|| format!("Variant{}", elem_idx + 1));
            FieldValue::Enum(picked)
        }
        FieldType::FilePath => {
            let files = ["data/hero.json", "sprites/fx.png", "levels/zone.ron"];
            FieldValue::FilePath(files[elem_idx % files.len()].to_string())
        }
        FieldType::Tile => {
            if let Some(FieldValue::Tile(Some(r))) = field.default_value.as_ref() {
                let mut rect = *r;
                rect.x += (elem_idx as u32) * rect.w.max(16);
                return FieldValue::Tile(Some(rect));
            }
            let ts = field
                .tileset_id
                .and_then(|id| tilesets.iter().find(|t| t.id == id))
                .or_else(|| tilesets.first());
            let rect = ts.map(|t| {
                let tw = t.tile_size.x.max(1);
                let th = t.tile_size.y.max(1);
                TilesetRect::new(
                    t.id,
                    t.padding + (elem_idx as u32) * (tw + t.spacing),
                    t.padding,
                    tw,
                    th,
                )
            });
            FieldValue::Tile(rect)
        }
        FieldType::EntityRef => FieldValue::EntityRef(Some(EntityId::new(
            PREVIEW_SAMPLE_ENTITY_ID_BASE + elem_idx as u64,
        ))),
        FieldType::Point => {
            let pts = [IVec2::new(28, -14), IVec2::new(34, 18), IVec2::new(-24, 18)];
            FieldValue::Point(pts[elem_idx % pts.len()])
        }
        FieldType::Array(inner) => {
            sample_scalar_value(inner, field, enums, tilesets, seed_index, elem_idx)
        }
    }
}

fn extract_numeric(value: &FieldValue) -> f32 {
    match value {
        FieldValue::Int(v) => *v as f32,
        FieldValue::Float(v) => *v as f32,
        FieldValue::Array(items) => items.first().map(extract_numeric).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn count_points(value: &FieldValue) -> usize {
    match value {
        FieldValue::Point(_) => 1,
        FieldValue::Array(items) => items
            .iter()
            .filter(|v| matches!(v, FieldValue::Point(_)))
            .count(),
        _ => 0,
    }
}

fn count_entity_refs(value: &FieldValue) -> usize {
    match value {
        FieldValue::EntityRef(Some(_)) => 1,
        FieldValue::Array(items) => items
            .iter()
            .filter(|v| matches!(v, FieldValue::EntityRef(Some(_))))
            .count(),
        _ => 0,
    }
}
