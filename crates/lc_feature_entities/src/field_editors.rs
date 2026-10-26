//! Reusable field value editors.
//!
//! These editors are plain functions that mutate a scratch [`FieldValue`] and
//! report what happened. Callers decide which command to execute, so the same
//! code serves the entity definition tab and the floating inspector.

use egui::{Color32, ComboBox, DragValue, Response, RichText, TextEdit, Ui};
use glam::IVec2;
use lc_core::defs::EnumDef;
use lc_core::fields::{FieldType, FieldValue};
use lc_core::id::{EntityDefId, EntityId, EnumDefId};
use lc_core::layer::LayerData;
use lc_core::project::Project;

use crate::field_defs::default_field_value;

/// One selectable entity reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityChoice {
    /// Referenced entity instance.
    pub id: EntityId,
    /// Human-readable label, e.g. `Player #3`.
    pub label: String,
}

/// Ambient data a field editor needs but does not own.
pub struct FieldEditorContext<'a> {
    /// Enum definitions available to `Enum` fields.
    pub enum_defs: &'a [EnumDef],
    /// Entity instances available to `EntityRef` fields.
    pub entities: &'a [EntityChoice],
    /// Preferred width for text inputs.
    pub width: f32,
}

impl<'a> FieldEditorContext<'a> {
    /// Creates a context with the default input width.
    pub fn new(enum_defs: &'a [EnumDef], entities: &'a [EntityChoice]) -> Self {
        Self {
            enum_defs,
            entities,
            width: 150.0,
        }
    }
}

/// What happened while an editor was drawn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FieldEditorResponse {
    /// The value changed and should be committed.
    pub changed: bool,
    /// A widget in the editor has keyboard focus right now.
    pub has_focus: bool,
    /// A widget in the editor gained keyboard focus this frame.
    pub gained_focus: bool,
    /// A widget in the editor lost keyboard focus this frame.
    pub lost_focus: bool,
    /// A widget in the editor is being dragged right now.
    ///
    /// `DragValue` edits with the pointer and never takes keyboard focus, so
    /// this is the only signal that a numeric drag is still in progress.
    pub dragging: bool,
}

impl FieldEditorResponse {
    fn observe(&mut self, response: &Response) {
        self.changed |= response.changed();
        self.has_focus |= response.has_focus();
        self.gained_focus |= response.gained_focus();
        self.lost_focus |= response.lost_focus();
        self.dragging |= response.dragged();
    }

    /// Whether the editor is in an active interaction that should keep a
    /// stroke id alive across frames without changes (typing pauses, drag
    /// pauses).
    pub fn keeps_stroke(&self) -> bool {
        self.has_focus || self.gained_focus || self.dragging
    }
}

/// Draws an editor for `value` according to `field_type`.
pub fn field_value_editor(
    ui: &mut Ui,
    value: &mut FieldValue,
    field_type: &FieldType,
    ctx: &FieldEditorContext<'_>,
) -> FieldEditorResponse {
    let mut out = FieldEditorResponse::default();

    match field_type {
        FieldType::Int => {
            let mut current = match value {
                FieldValue::Int(v) => *v,
                _ => 0,
            };
            let response = ui.add(DragValue::new(&mut current).speed(1.0));
            if response.changed() {
                *value = FieldValue::Int(current);
            }
            out.observe(&response);
        }
        FieldType::Float => {
            let mut current = match value {
                FieldValue::Float(v) => *v,
                _ => 0.0,
            };
            let response = ui.add(DragValue::new(&mut current).speed(0.05));
            if response.changed() {
                *value = FieldValue::Float(current);
            }
            out.observe(&response);
        }
        FieldType::Bool => {
            let mut current = match value {
                FieldValue::Bool(v) => *v,
                _ => false,
            };
            let response = ui.checkbox(&mut current, "");
            if response.changed() {
                *value = FieldValue::Bool(current);
            }
            out.observe(&response);
        }
        FieldType::String => {
            let mut current = match value {
                FieldValue::String(v) => v.clone(),
                _ => String::new(),
            };
            let response = ui.add(TextEdit::singleline(&mut current).desired_width(ctx.width));
            if response.changed() {
                *value = FieldValue::String(current);
            }
            out.observe(&response);
        }
        FieldType::Color => {
            let mut current = match value {
                FieldValue::Color(v) => Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3]),
                _ => Color32::WHITE,
            };
            let response = ui.color_edit_button_srgba(&mut current);
            if response.changed() {
                *value = FieldValue::Color(current.to_array());
            }
            out.observe(&response);
        }
        FieldType::Enum(enum_id) => {
            let mut current = match value {
                FieldValue::Enum(v) => v.clone(),
                _ => String::new(),
            };
            let mut picked = false;
            let response = ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(enum_label(ctx.enum_defs, *enum_id, &current))
                .show_ui(ui, |ui| {
                    if let Some(def) = ctx.enum_defs.iter().find(|e| e.id == *enum_id) {
                        for option in &def.values {
                            if ui
                                .selectable_value(&mut current, option.id.clone(), &option.name)
                                .clicked()
                            {
                                picked = true;
                            }
                        }
                    } else {
                        ui.label(RichText::new("Enum definition missing").weak());
                    }
                });
            if picked {
                *value = FieldValue::Enum(current);
            }
            out.observe(&response.response);
        }
        FieldType::Point => {
            let mut current = match value {
                FieldValue::Point(v) => *v,
                _ => IVec2::ZERO,
            };
            ui.horizontal(|ui| {
                let x = ui.add(DragValue::new(&mut current.x).speed(1.0));
                let y = ui.add(DragValue::new(&mut current.y).speed(1.0));
                if x.changed() || y.changed() {
                    *value = FieldValue::Point(current);
                }
                out.observe(&x);
                out.observe(&y);
            });
        }
        FieldType::EntityRef => {
            let mut current = match value {
                FieldValue::EntityRef(v) => *v,
                _ => None,
            };
            let selected = current
                .and_then(|id| ctx.entities.iter().find(|e| e.id == id))
                .map(|e| e.label.as_str())
                .unwrap_or("None");
            let mut picked = false;
            let response = ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut current, None, "None").clicked() {
                        picked = true;
                    }
                    for choice in ctx.entities {
                        if ui
                            .selectable_value(&mut current, Some(choice.id), &choice.label)
                            .clicked()
                        {
                            picked = true;
                        }
                    }
                });
            if picked {
                *value = FieldValue::EntityRef(current);
            }
            out.observe(&response.response);
        }
        FieldType::Array(inner) => {
            let mut items = match value {
                FieldValue::Array(items) => std::mem::take(items),
                _ => Vec::new(),
            };
            let mut removed = None;
            ui.vertical(|ui| {
                for (index, item) in items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let response = field_value_editor(ui, item, inner, ctx);
                        out.changed |= response.changed;
                        out.has_focus |= response.has_focus;
                        out.gained_focus |= response.gained_focus;
                        out.lost_focus |= response.lost_focus;
                        out.dragging |= response.dragging;
                        if ui.small_button("-").on_hover_text("Remove item").clicked() {
                            removed = Some(index);
                        }
                    });
                }
                if ui.small_button("+").on_hover_text("Add item").clicked() {
                    items.push(default_field_value(inner, ctx.enum_defs));
                    out.changed = true;
                }
            });
            if let Some(index) = removed {
                if index < items.len() {
                    items.remove(index);
                    out.changed = true;
                }
            }
            if out.changed {
                *value = FieldValue::Array(items);
            }
        }
    }

    out
}

/// Draws a type picker for a field definition.
///
/// Returns the new type only when the user changed it. Enum and array types
/// keep their parameters where possible.
pub fn field_type_editor(
    ui: &mut Ui,
    current: &FieldType,
    enum_defs: &[EnumDef],
) -> Option<FieldType> {
    let mut selected = current.clone();

    ComboBox::from_id_salt(ui.next_auto_id())
        .selected_text(field_type_label(&selected))
        .show_ui(ui, |ui| {
            for candidate in [
                FieldType::Int,
                FieldType::Float,
                FieldType::Bool,
                FieldType::String,
                FieldType::Color,
                FieldType::Point,
                FieldType::EntityRef,
            ] {
                if ui
                    .selectable_label(selected == candidate, field_type_label(&candidate))
                    .clicked()
                {
                    selected = candidate;
                }
            }

            let enum_selected = matches!(selected, FieldType::Enum(_));
            if ui.selectable_label(enum_selected, "Enum").clicked() && !enum_selected {
                selected = enum_defs
                    .first()
                    .map(|e| FieldType::Enum(e.id))
                    .unwrap_or(FieldType::Enum(EnumDefId(0)));
            }

            let array_selected = matches!(selected, FieldType::Array(_));
            if ui.selectable_label(array_selected, "Array").clicked() && !array_selected {
                selected = FieldType::Array(Box::new(FieldType::Int));
            }
        });

    match &mut selected {
        FieldType::Enum(enum_id) => {
            let current_name = enum_defs
                .iter()
                .find(|e| e.id == *enum_id)
                .map(|e| e.name.as_str())
                .unwrap_or("Missing enum");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    for def in enum_defs {
                        ui.selectable_value(enum_id, def.id, &def.name);
                    }
                });
        }
        FieldType::Array(inner) => {
            ui.horizontal(|ui| {
                ui.label("of");
                if let Some(new_inner) = field_type_editor(ui, inner, enum_defs) {
                    **inner = new_inner;
                }
            });
        }
        _ => {}
    }

    (selected != *current).then_some(selected)
}

/// Short label for a field type, used in pickers and field rows.
pub fn field_type_label(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Int => "Int",
        FieldType::Float => "Float",
        FieldType::Bool => "Bool",
        FieldType::String => "String",
        FieldType::Color => "Color",
        FieldType::Enum(_) => "Enum",
        FieldType::Point => "Point",
        FieldType::EntityRef => "EntityRef",
        FieldType::Array(_) => "Array",
    }
}

fn enum_label(enum_defs: &[EnumDef], id: EnumDefId, value: &str) -> String {
    let Some(def) = enum_defs.iter().find(|e| e.id == id) else {
        return format!("Missing enum ({value})");
    };
    def.value(value)
        .map(|v| v.name.clone())
        .unwrap_or_else(|| value.to_string())
}

/// Ensures `value` matches `field_type` before it is displayed.
///
/// Used by the definition tab and inspector so legacy or hand-edited files do
/// not render an empty editor.
pub fn normalized_value(
    value: &FieldValue,
    field_type: &FieldType,
    enums: &[EnumDef],
) -> FieldValue {
    crate::field_defs::coerce_field_value(value, field_type, enums)
}

/// Returns a stable identifier for an entity choice list entry.
pub fn entity_choice_label(def_name: &str, id: EntityId) -> String {
    format!("{def_name} #{}", id.0)
}

/// Collects every entity instance in the project as a reference choice.
///
/// Used by `EntityRef` editors in the definition tab and the floating
/// inspector.
pub fn entity_choices(p: &Project) -> Vec<EntityChoice> {
    let mut out = Vec::new();
    for world in &p.worlds {
        for level in &world.levels {
            for layer in &level.layers {
                if let LayerData::Entities(entities) = &layer.data {
                    for entity in entities {
                        let def_name = p
                            .defs
                            .entity(entity.def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("Entity");
                        out.push(EntityChoice {
                            id: entity.id,
                            label: entity_choice_label(def_name, entity.id),
                        });
                    }
                }
            }
        }
    }
    out
}

/// Formats an entity definition reference for labels.
pub fn entity_def_label(def_name: &str, id: EntityDefId) -> String {
    format!("{def_name} (#{})", id.0)
}
