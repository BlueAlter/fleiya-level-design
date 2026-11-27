//! Fast inline custom properties editor for collision shapes and tiles.
//!
//! Stores typed custom properties (String, Int, Float, Boolean) into the
//! existing `custom_data` string representation with backward compatibility.

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, RichText, Sense, Stroke, StrokeKind,
    TextEdit, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};

/// Supported custom property types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomPropType {
    String,
    Int,
    Float,
    Bool,
}

impl CustomPropType {
    pub const ALL: &'static [Self] = &[Self::String, Self::Int, Self::Float, Self::Bool];

    pub fn label(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Bool => "Boolean",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::String => "str",
            Self::Int => "int",
            Self::Float => "flt",
            Self::Bool => "bool",
        }
    }

    pub fn from_badge(s: &str) -> Self {
        match s {
            "int" => Self::Int,
            "flt" | "float" => Self::Float,
            "bool" | "boolean" => Self::Bool,
            _ => Self::String,
        }
    }

    pub fn default_value(self) -> String {
        match self {
            Self::String => String::new(),
            Self::Int => "0".to_string(),
            Self::Float => "0.0".to_string(),
            Self::Bool => "false".to_string(),
        }
    }

    pub fn badge_color(self, t: &Tokens) -> Color32 {
        match self {
            Self::String => t.accent_ink,
            Self::Int => t.good,
            Self::Float => t.warn,
            Self::Bool => t.bad,
        }
    }
}

/// A typed key-value custom property item.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomProperty {
    pub name: String,
    pub prop_type: CustomPropType,
    pub value: String,
}

impl CustomProperty {
    pub fn new(name: impl Into<String>, prop_type: CustomPropType) -> Self {
        let val = prop_type.default_value();
        Self {
            name: name.into(),
            prop_type,
            value: val,
        }
    }

    /// Parses a single formatted line (`name:type=value` or fallback `key=val` / raw).
    pub fn parse_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((lhs, val)) = trimmed.split_once('=') {
            if let Some((name, t_str)) = lhs.split_once(':') {
                return Some(Self {
                    name: name.trim().to_string(),
                    prop_type: CustomPropType::from_badge(t_str.trim()),
                    value: val.trim().to_string(),
                });
            }
            return Some(Self {
                name: lhs.trim().to_string(),
                prop_type: CustomPropType::String,
                value: val.trim().to_string(),
            });
        }

        Some(Self {
            name: trimmed.to_string(),
            prop_type: CustomPropType::String,
            value: String::new(),
        })
    }

    /// Serializes to `name:type=value`.
    pub fn serialize(&self) -> String {
        format!("{}:{}={}", self.name, self.prop_type.badge(), self.value)
    }
}

/// Parses an entire custom data string into a list of typed properties.
pub fn parse_custom_properties(raw: &str) -> Vec<CustomProperty> {
    raw.lines().filter_map(CustomProperty::parse_line).collect()
}

/// Serializes a list of typed properties into a single string.
pub fn serialize_custom_properties(props: &[CustomProperty]) -> String {
    props
        .iter()
        .filter(|p| !p.name.trim().is_empty())
        .map(|p| p.serialize())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Renders the fast inline custom properties editor. Returns true if modified.
pub fn render_fast_custom_properties(
    ui: &mut Ui,
    t: &Tokens,
    props: &mut Vec<CustomProperty>,
) -> bool {
    let mut modified = false;

    // Header row: PROPERTIES (count) and [+ Add] dropdown
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("PROPERTIES ({})", props.len()))
                .size(9.5)
                .strong()
                .color(t.muted),
        );

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            ui.menu_button(
                RichText::new(format!("{} Add", Icon::Add.glyph()))
                    .font(FontId::new(10.0, lc_canvas::icons::family(ui.ctx())))
                    .color(t.accent_ink),
                |ui| {
                    ui.set_min_width(90.0);
                    for &pt in CustomPropType::ALL {
                        let btn = egui::Button::new(
                            RichText::new(format!("+ {}", pt.label())).size(11.0),
                        )
                        .corner_radius(CornerRadius::ZERO);
                        if ui.add(btn).clicked() {
                            let next_num = props.len() + 1;
                            props.push(CustomProperty::new(format!("prop_{next_num}"), pt));
                            modified = true;
                            ui.close();
                        }
                    }
                },
            );
        });
    });

    ui.add_space(2.0);

    if props.is_empty() {
        ui.label(
            RichText::new("No custom properties")
                .size(9.5)
                .italics()
                .color(t.muted.gamma_multiply(0.7)),
        );
        return modified;
    }

    let mut remove_idx = None;

    for (idx, prop) in props.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Name field
            let name_edit = ui.add(
                TextEdit::singleline(&mut prop.name)
                    .hint_text("name")
                    .desired_width(62.0)
                    .font(FontId::monospace(10.0)),
            );
            if name_edit.changed() {
                modified = true;
            }

            // Type badge
            let badge_color = prop.prop_type.badge_color(t);
            let badge_sz = vec2(28.0, 16.0);
            let (badge_rect, _) = ui.allocate_exact_size(badge_sz, Sense::hover());
            ui.painter().rect_filled(
                badge_rect,
                CornerRadius::ZERO,
                badge_color.gamma_multiply(0.2),
            );
            ui.painter().rect_stroke(
                badge_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, badge_color),
                StrokeKind::Inside,
            );
            ui.painter().text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                prop.prop_type.badge(),
                FontId::new(9.0, lc_canvas::theme::mono_for(ui.ctx())),
                badge_color,
            );

            // Value field based on type
            match prop.prop_type {
                CustomPropType::String => {
                    let val_edit = ui.add(
                        TextEdit::singleline(&mut prop.value)
                            .hint_text("value")
                            .desired_width(70.0)
                            .font(FontId::proportional(10.0)),
                    );
                    if val_edit.changed() {
                        modified = true;
                    }
                }
                CustomPropType::Int => {
                    let val_edit = ui.add(
                        TextEdit::singleline(&mut prop.value)
                            .hint_text("0")
                            .desired_width(55.0)
                            .font(FontId::monospace(10.0)),
                    );
                    if val_edit.changed() {
                        prop.value = sanitize_int_input(&prop.value);
                        modified = true;
                    }
                    if val_edit.lost_focus() {
                        let finalized = finalize_int_input(&prop.value);
                        if finalized != prop.value {
                            prop.value = finalized;
                            modified = true;
                        }
                    }
                }
                CustomPropType::Float => {
                    let val_edit = ui.add(
                        TextEdit::singleline(&mut prop.value)
                            .hint_text("0.0")
                            .desired_width(55.0)
                            .font(FontId::monospace(10.0)),
                    );
                    if val_edit.changed() {
                        prop.value = sanitize_float_input(&prop.value);
                        modified = true;
                    }
                    if val_edit.lost_focus() {
                        let finalized = finalize_float_input(&prop.value);
                        if finalized != prop.value {
                            prop.value = finalized;
                            modified = true;
                        }
                    }
                }
                CustomPropType::Bool => {
                    let mut is_true = prop.value == "true";
                    if ui.checkbox(&mut is_true, "").changed() {
                        prop.value = if is_true { "true" } else { "false" }.to_string();
                        modified = true;
                    }
                }
            }

            // Delete property button
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Icon::Close, 16.0, IconButtonStyle::bordered().square())
                    .on_hover_text("Delete property")
                    .clicked()
                {
                    remove_idx = Some(idx);
                }
            });
        });
        ui.add_space(1.0);
    }

    if let Some(idx) = remove_idx {
        props.remove(idx);
        modified = true;
    }

    modified
}

/// Filters an integer string to optional leading `-` and ASCII digits only.
pub fn sanitize_int_input(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for (idx, ch) in raw.chars().enumerate() {
        if ch.is_ascii_digit() || (ch == '-' && idx == 0) {
            out.push(ch);
        }
    }
    out
}

/// Normalizes a finalized integer string when editing ends.
pub fn finalize_int_input(raw: &str) -> String {
    let sanitized = sanitize_int_input(raw);
    if sanitized.is_empty() || sanitized == "-" {
        "0".to_string()
    } else if let Ok(v) = sanitized.parse::<i64>() {
        v.to_string()
    } else {
        sanitized
    }
}

/// Filters a decimal/float string to optional leading `-`, digits, and at most one `.`
/// (accepting `,` or `.` as the decimal separator and fixing `,` to `.`).
pub fn sanitize_float_input(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut has_dot = false;
    for (idx, ch) in raw.chars().enumerate() {
        if ch.is_ascii_digit() || (ch == '-' && idx == 0) {
            out.push(ch);
        } else if (ch == '.' || ch == ',') && !has_dot {
            has_dot = true;
            out.push('.');
        }
    }
    out
}

/// Normalizes a finalized float string when editing ends.
pub fn finalize_float_input(raw: &str) -> String {
    let sanitized = sanitize_float_input(raw);
    if sanitized.is_empty() || sanitized == "-" || sanitized == "." || sanitized == "-." {
        return "0.0".to_string();
    }
    if sanitized.starts_with('.') {
        format!("0{sanitized}")
    } else if let Some(rest) = sanitized.strip_prefix("-.") {
        format!("-0.{rest}")
    } else if sanitized.ends_with('.') {
        format!("{sanitized}0")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_property_parse_and_serialize() {
        let raw = "damage:int=50\nspeed:flt=1.5\nis_solid:bool=true\nname:str=ice_wall";
        let props = parse_custom_properties(raw);
        assert_eq!(props.len(), 4);
        assert_eq!(props[0].name, "damage");
        assert_eq!(props[0].prop_type, CustomPropType::Int);
        assert_eq!(props[0].value, "50");

        assert_eq!(props[1].name, "speed");
        assert_eq!(props[1].prop_type, CustomPropType::Float);
        assert_eq!(props[1].value, "1.5");

        assert_eq!(props[2].name, "is_solid");
        assert_eq!(props[2].prop_type, CustomPropType::Bool);
        assert_eq!(props[2].value, "true");

        assert_eq!(props[3].name, "name");
        assert_eq!(props[3].prop_type, CustomPropType::String);
        assert_eq!(props[3].value, "ice_wall");

        let serialized = serialize_custom_properties(&props);
        assert_eq!(serialized, raw);
    }

    #[test]
    fn test_legacy_fallback_parse() {
        let raw = "damage=50\nsolid";
        let props = parse_custom_properties(raw);
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].name, "damage");
        assert_eq!(props[0].prop_type, CustomPropType::String);
        assert_eq!(props[0].value, "50");

        assert_eq!(props[1].name, "solid");
        assert_eq!(props[1].prop_type, CustomPropType::String);
        assert_eq!(props[1].value, "");
    }

    #[test]
    fn test_sanitize_and_finalize_int_input() {
        assert_eq!(sanitize_int_input("abc123def"), "123");
        assert_eq!(sanitize_int_input("-42xyz"), "-42");
        assert_eq!(sanitize_int_input("12-3.5"), "1235");
        assert_eq!(finalize_int_input(""), "0");
        assert_eq!(finalize_int_input("-"), "0");
        assert_eq!(finalize_int_input("-007"), "-7");
    }

    #[test]
    fn test_sanitize_and_finalize_float_input() {
        assert_eq!(sanitize_float_input("1,5"), "1.5");
        assert_eq!(sanitize_float_input("1.5"), "1.5");
        assert_eq!(sanitize_float_input("-12,34abc"), "-12.34");
        assert_eq!(sanitize_float_input("1,5.2,9"), "1.529");
        assert_eq!(finalize_float_input(""), "0.0");
        assert_eq!(finalize_float_input("."), "0.0");
        assert_eq!(finalize_float_input(",5"), "0.5");
        assert_eq!(finalize_float_input("-.5"), "-0.5");
        assert_eq!(finalize_float_input("3,"), "3.0");
    }
}
