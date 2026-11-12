//! Field type helpers: defaults, value migration and display formatting.
//!
//! The data model stores [`FieldValue`] independently of its [`FieldType`], so
//! changing a field definition may produce values that no longer fit. These
//! helpers define that migration in one place; commands and editors share them.

use glam::IVec2;
use lc_core::defs::{EntityDef, EnumDef};
use lc_core::fields::{FieldType, FieldValue, FieldValues};

/// Returns the default value for `field_type`.
///
/// Enums default to their first value (or an empty string when the enum has no
/// values); arrays default to empty; every other type uses its zero value.
pub fn default_field_value(field_type: &FieldType, enum_defs: &[EnumDef]) -> FieldValue {
    match field_type {
        FieldType::Int => FieldValue::Int(0),
        FieldType::Float => FieldValue::Float(0.0),
        FieldType::Bool => FieldValue::Bool(false),
        FieldType::String => FieldValue::String(String::new()),
        FieldType::Color => FieldValue::Color([255, 255, 255, 255]),
        FieldType::Enum(enum_id) => {
            let first = enum_defs
                .iter()
                .find(|e| e.id == *enum_id)
                .and_then(|e| e.values.first())
                .map(|v| v.id.clone())
                .unwrap_or_default();
            FieldValue::Enum(first)
        }
        FieldType::Point => FieldValue::Point(IVec2::ZERO),
        FieldType::EntityRef => FieldValue::EntityRef(None),
        FieldType::Array(_) => FieldValue::Array(Vec::new()),
    }
}

/// Builds the initial field-value map for a new entity instance.
///
/// Uses the definition's stored default when present, coerced to the field
/// type, otherwise the type's zero value.
pub fn default_field_values(def: &EntityDef, enum_defs: &[EnumDef]) -> FieldValues {
    let mut values = FieldValues::new();
    for field in &def.fields {
        let value = field
            .default_value
            .as_ref()
            .map(|value| coerce_field_value(value, &field.field_type, enum_defs))
            .unwrap_or_else(|| default_field_value(&field.field_type, enum_defs));
        values.set(field.name.clone(), value);
    }
    values
}

/// Converts `value` to a value compatible with `target`.
///
/// Conversions are intentionally lossy but total: a value that cannot be
/// represented becomes the target type's default. Enum values are validated
/// against the enum definition.
pub fn coerce_field_value(
    value: &FieldValue,
    target: &FieldType,
    enum_defs: &[EnumDef],
) -> FieldValue {
    let default = || default_field_value(target, enum_defs);

    match target {
        FieldType::Int => FieldValue::Int(match value {
            FieldValue::Int(v) => *v,
            FieldValue::Float(v) => *v as i64,
            FieldValue::Bool(v) => i64::from(*v),
            FieldValue::String(v) => v.trim().parse().unwrap_or(0),
            FieldValue::Point(v) => i64::from(v.x),
            _ => 0,
        }),
        FieldType::Float => FieldValue::Float(match value {
            FieldValue::Float(v) => *v,
            FieldValue::Int(v) => *v as f64,
            FieldValue::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            FieldValue::String(v) => v.trim().parse().unwrap_or(0.0),
            FieldValue::Point(v) => f64::from(v.x),
            _ => 0.0,
        }),
        FieldType::Bool => FieldValue::Bool(match value {
            FieldValue::Bool(v) => *v,
            FieldValue::Int(v) => *v != 0,
            FieldValue::Float(v) => *v != 0.0,
            FieldValue::String(v) => matches!(v.trim(), "true" | "1" | "yes"),
            FieldValue::Point(v) => v != &IVec2::ZERO,
            _ => false,
        }),
        FieldType::String => match value {
            FieldValue::String(v) => FieldValue::String(v.clone()),
            _ => FieldValue::String(format_field_value(value)),
        },
        FieldType::Color => match value {
            FieldValue::Color(v) => FieldValue::Color(*v),
            _ => default(),
        },
        FieldType::Enum(enum_id) => {
            let candidate = match value {
                FieldValue::Enum(v) => Some(v.as_str()),
                FieldValue::String(v) => Some(v.as_str()),
                _ => None,
            };
            let valid = candidate.and_then(|candidate| {
                enum_defs
                    .iter()
                    .find(|e| e.id == *enum_id)
                    .and_then(|e| e.value(candidate))
                    .map(|v| v.id.clone())
            });
            FieldValue::Enum(valid.unwrap_or_else(|| {
                match default_field_value(target, enum_defs) {
                    FieldValue::Enum(v) => v,
                    _ => String::new(),
                }
            }))
        }
        FieldType::Point => FieldValue::Point(match value {
            FieldValue::Point(v) => *v,
            FieldValue::Int(v) => IVec2::new(*v as i32, 0),
            FieldValue::String(v) => parse_point(v).unwrap_or(IVec2::ZERO),
            _ => IVec2::ZERO,
        }),
        FieldType::EntityRef => match value {
            FieldValue::EntityRef(v) => FieldValue::EntityRef(*v),
            _ => FieldValue::EntityRef(None),
        },
        FieldType::Array(inner) => match value {
            FieldValue::Array(items) => FieldValue::Array(
                items
                    .iter()
                    .map(|item| coerce_field_value(item, inner, enum_defs))
                    .collect(),
            ),
            _ => FieldValue::Array(Vec::new()),
        },
    }
}

/// Formats a field value for compact display in labels and tooltips.
pub fn format_field_value(value: &FieldValue) -> String {
    match value {
        FieldValue::Int(v) => v.to_string(),
        FieldValue::Float(v) => v.to_string(),
        FieldValue::Bool(v) => v.to_string(),
        FieldValue::String(v) => v.clone(),
        FieldValue::Color(v) => format!("#{:02X}{:02X}{:02X}{:02X}", v[0], v[1], v[2], v[3]),
        FieldValue::Enum(v) => v.clone(),
        FieldValue::Point(v) => format!("{}, {}", v.x, v.y),
        FieldValue::EntityRef(Some(v)) => format!("entity {}", v.0),
        FieldValue::EntityRef(None) => "none".to_string(),
        FieldValue::Array(items) => items
            .iter()
            .map(format_field_value)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn parse_point(text: &str) -> Option<IVec2> {
    let (x, y) = text.split_once(',')?;
    Some(IVec2::new(x.trim().parse().ok()?, y.trim().parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::defs::{EnumDef, EnumValueDef};
    use lc_core::id::EnumDefId;

    fn enum_defs() -> Vec<EnumDef> {
        vec![EnumDef {
            id: EnumDefId::new(1),
            name: "Biome".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![
                EnumValueDef {
                    id: "Forest".to_string(),
                    name: "Forest".to_string(),
                    color: [0, 255, 0, 255],
                    tile: None,
                },
                EnumValueDef {
                    id: "Desert".to_string(),
                    name: "Desert".to_string(),
                    color: [255, 255, 0, 255],
                    tile: None,
                },
            ],
        }]
    }

    #[test]
    fn defaults_cover_every_type() {
        let enums = enum_defs();
        assert_eq!(
            default_field_value(&FieldType::Int, &enums),
            FieldValue::Int(0)
        );
        assert_eq!(
            default_field_value(&FieldType::Float, &enums),
            FieldValue::Float(0.0)
        );
        assert_eq!(
            default_field_value(&FieldType::Bool, &enums),
            FieldValue::Bool(false)
        );
        assert_eq!(
            default_field_value(&FieldType::String, &enums),
            FieldValue::String(String::new())
        );
        assert_eq!(
            default_field_value(&FieldType::Color, &enums),
            FieldValue::Color([255, 255, 255, 255])
        );
        assert_eq!(
            default_field_value(&FieldType::Enum(EnumDefId::new(1)), &enums),
            FieldValue::Enum("Forest".to_string())
        );
        assert_eq!(
            default_field_value(&FieldType::Point, &enums),
            FieldValue::Point(IVec2::ZERO)
        );
        assert_eq!(
            default_field_value(&FieldType::EntityRef, &enums),
            FieldValue::EntityRef(None)
        );
        assert_eq!(
            default_field_value(&FieldType::Array(Box::new(FieldType::Int)), &enums),
            FieldValue::Array(Vec::new())
        );
    }

    #[test]
    fn coercion_is_total_and_validates_enums() {
        let enums = enum_defs();
        let enum_ty = FieldType::Enum(EnumDefId::new(1));

        assert_eq!(
            coerce_field_value(&FieldValue::Float(3.9), &FieldType::Int, &enums),
            FieldValue::Int(3)
        );
        assert_eq!(
            coerce_field_value(
                &FieldValue::String("42".to_string()),
                &FieldType::Int,
                &enums
            ),
            FieldValue::Int(42)
        );
        assert_eq!(
            coerce_field_value(
                &FieldValue::String("nope".to_string()),
                &FieldType::Int,
                &enums
            ),
            FieldValue::Int(0)
        );
        assert_eq!(
            coerce_field_value(&FieldValue::Int(1), &FieldType::Bool, &enums),
            FieldValue::Bool(true)
        );
        assert_eq!(
            coerce_field_value(&FieldValue::String("Desert".to_string()), &enum_ty, &enums),
            FieldValue::Enum("Desert".to_string())
        );
        assert_eq!(
            coerce_field_value(
                &FieldValue::String("Mountain".to_string()),
                &enum_ty,
                &enums
            ),
            FieldValue::Enum("Forest".to_string())
        );
        assert_eq!(
            coerce_field_value(
                &FieldValue::String("4,7".to_string()),
                &FieldType::Point,
                &enums
            ),
            FieldValue::Point(IVec2::new(4, 7))
        );
        assert_eq!(
            coerce_field_value(
                &FieldValue::Int(5),
                &FieldType::Array(Box::new(FieldType::Int)),
                &enums
            ),
            FieldValue::Array(Vec::new())
        );
    }

    #[test]
    fn array_items_are_coerced_recursively() {
        let enums = enum_defs();
        let value = FieldValue::Array(vec![FieldValue::Float(1.2), FieldValue::Float(2.8)]);
        assert_eq!(
            coerce_field_value(&value, &FieldType::Array(Box::new(FieldType::Int)), &enums),
            FieldValue::Array(vec![FieldValue::Int(1), FieldValue::Int(2)])
        );
    }

    #[test]
    fn formatting_is_human_readable() {
        assert_eq!(format_field_value(&FieldValue::Int(3)), "3");
        assert_eq!(
            format_field_value(&FieldValue::Color([1, 2, 3, 255])),
            "#010203FF"
        );
        assert_eq!(
            format_field_value(&FieldValue::Point(IVec2::new(-2, 5))),
            "-2, 5"
        );
        assert_eq!(
            format_field_value(&FieldValue::Array(vec![
                FieldValue::Int(1),
                FieldValue::Int(2)
            ])),
            "1, 2"
        );
    }
}
