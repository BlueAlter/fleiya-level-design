//! Floating inspector section for entity instances.
//!
//! Shown inside the level canvas' floating properties window whenever one or
//! more entities are selected. A single selection exposes position, size and
//! every field of its definition; a multi-selection exposes only the fields
//! that all selected definitions share (same name and type).
//!
//! Every edit goes through a command. Field editors keep one stroke id for the
//! whole focus session, so typing a value is a single undo step.

use egui::{DragValue, RichText, Ui};
use glam::IVec2;
use lc_canvas::theme::Tokens;
use lc_core::command::Command;
use lc_core::defs::{EntityDef, EnumDef};
use lc_core::fields::{FieldDef, FieldValue};
use lc_core::id::{EntityId, LevelId};
use lc_core::layer::EntityInstance;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::selection::Selection;
use lc_ui_api::session::CanvasPickMode;

use crate::field_defs::{coerce_for_field_def, default_field_value, default_value_for_field};
use crate::field_editors::{entity_choices, field_value_editor, FieldEditorContext};
use crate::field_instance_commands::{EntityFieldUpdate, SetEntityFieldValuesCommand};
use crate::geometry::clamp_position_for_def;
use crate::helpers::{filter_entity_choices, find_entity_location};
use crate::instance_commands::{
    RemoveEntityInstancesCommand, SetEntityPositionsCommand, SetEntitySizesCommand,
};
use crate::stroke::FocusStrokeTracker;

/// Inspector section rendering the selected entity or entities.
#[derive(Debug, Default)]
pub struct EntityInspectorSection {
    strokes: FocusStrokeTracker<String>,
    last_selection: Option<Vec<EntityId>>,
}

impl EntityInspectorSection {
    /// Creates a new inspector section.
    pub fn new() -> Self {
        Self::default()
    }
}

impl InspectorSection for EntityInspectorSection {
    fn applies(&self, sel: &Selection) -> bool {
        sel.entities().is_some_and(|ids| !ids.is_empty())
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx, sel: &Selection) {
        let Some(ids) = sel.entities().map(<[EntityId]>::to_vec) else {
            self.strokes.reset();
            self.last_selection = None;
            return;
        };
        if self.last_selection.as_ref() != Some(&ids) {
            self.strokes.reset();
            self.last_selection = Some(ids.clone());
        }

        self.strokes.begin_frame();

        // The selection is session-global, but this section is rendered by one
        // level tab: only ids that live in the active level may be shown or
        // edited here. The active level wins over the first selected id so a
        // stale cross-tab selection can never edit another level.
        let Some(level_id) = cx.active_level().or_else(|| {
            ids.iter()
                .find_map(|id| find_entity_location(cx.project(), *id).map(|loc| loc.level_id))
        }) else {
            self.strokes.end_frame();
            return;
        };
        let Some(level) = cx.project().level(level_id) else {
            self.strokes.end_frame();
            return;
        };

        let entities = entities_in_level(cx.project(), level_id, &ids);
        if entities.is_empty() {
            self.strokes.end_frame();
            return;
        }

        let defs: Vec<EntityDef> = entities
            .iter()
            .filter_map(|entity| cx.project().defs.entity(entity.def_id).cloned())
            .collect();
        let enum_defs = cx.project().defs.enums.clone();
        let choices = entity_choices(cx.project());

        let mut commands: Vec<Box<dyn Command>> = Vec::new();
        let mut events: Vec<EditorEvent> = Vec::new();

        render_header(ui, &entities, &defs, level_id);

        if entities.len() == 1 {
            let allow_oob = defs.first().is_some_and(|d| d.allow_out_of_bounds);
            let rx = defs.first().is_some_and(|d| d.resizable_x);
            let ry = defs.first().is_some_and(|d| d.resizable_y);
            render_transform(
                ui,
                LevelTransform {
                    level_id,
                    entity: &entities[0],
                    level_size: level.size_px,
                    allow_out_of_bounds: allow_oob,
                    resizable_x: rx,
                    resizable_y: ry,
                },
                &mut self.strokes,
                &mut commands,
            );
        }

        let common = common_fields(&defs);
        ui.add_space(6.0);
        if common.is_empty() {
            ui.label(RichText::new("No custom fields").color(Tokens::get(ui.ctx()).faint));
        } else {
            let all_same_def = defs.windows(2).all(|w| w[0].id == w[1].id);
            for field in common {
                let start = effective_value(&entities[0], &field, &enum_defs);
                let mixed = entities
                    .iter()
                    .skip(1)
                    .any(|entity| effective_value(entity, &field, &enum_defs) != start);

                let filtered_choices = if all_same_def && !defs.is_empty() {
                    filter_entity_choices(cx.project(), defs[0].id, &field)
                } else {
                    choices.clone()
                };
                let field_ctx = FieldEditorContext::new(&enum_defs, &filtered_choices);

                let mut scratch = start;
                let mut null_toggled = false;
                ui.horizontal(|ui| {
                    let label_resp =
                        ui.label(RichText::new(&field.name).color(Tokens::get(ui.ctx()).ink));
                    if let Some(ref doc) = field.doc {
                        label_resp.on_hover_text(doc);
                    }
                    if mixed {
                        ui.label(RichText::new("(mixed)").color(Tokens::get(ui.ctx()).faint));
                    }

                    if field.can_be_null && !field.is_array() {
                        let mut is_null = matches!(scratch, FieldValue::Null);
                        if ui
                            .checkbox(&mut is_null, "null")
                            .on_hover_text("Toggle null value")
                            .changed()
                        {
                            scratch = if is_null {
                                FieldValue::Null
                            } else {
                                default_field_value(&field.field_type, &enum_defs)
                            };
                            null_toggled = true;
                        }
                    }

                    if !matches!(scratch, FieldValue::Null) {
                        let response =
                            field_value_editor(ui, &mut scratch, &field.field_type, &field_ctx);
                        let held = response.keeps_stroke();
                        if let Some(stroke) =
                            self.strokes.id(field.name.clone(), held, response.changed)
                        {
                            if response.changed {
                                let clamped = coerce_for_field_def(&scratch, &field, &enum_defs);
                                let updates = entities
                                    .iter()
                                    .map(|entity| EntityFieldUpdate {
                                        entity_id: entity.id,
                                        field_name: field.name.clone(),
                                        value: clamped.clone(),
                                    })
                                    .collect();
                                commands.push(Box::new(SetEntityFieldValuesCommand::with_stroke(
                                    level_id, updates, stroke,
                                )));
                            }
                        }

                        if entities.len() == 1
                            && matches!(
                                field.field_type,
                                lc_core::fields::FieldType::Point
                                    | lc_core::fields::FieldType::EntityRef
                            )
                            && ui
                                .small_button("Pick")
                                .on_hover_text("Pick on canvas")
                                .clicked()
                        {
                            let pick_mode = match field.field_type {
                                lc_core::fields::FieldType::Point => CanvasPickMode::Point {
                                    level_id,
                                    entity_id: entities[0].id,
                                    field_name: field.name.clone(),
                                    array_index: None,
                                },
                                _ => CanvasPickMode::EntityRef {
                                    level_id,
                                    entity_id: entities[0].id,
                                    field_name: field.name.clone(),
                                    array_index: None,
                                },
                            };
                            events.push(EditorEvent::SetCanvasPickMode(Some(pick_mode)));
                            events.push(EditorEvent::Status(format!(
                                "Click on canvas to pick '{}' (Esc to cancel)",
                                field.name
                            )));
                        }
                    }

                    if null_toggled {
                        let updates = entities
                            .iter()
                            .map(|entity| EntityFieldUpdate {
                                entity_id: entity.id,
                                field_name: field.name.clone(),
                                value: scratch.clone(),
                            })
                            .collect();
                        commands.push(Box::new(SetEntityFieldValuesCommand::new(
                            level_id, updates,
                        )));
                    }
                });
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let t = Tokens::get(ui.ctx());
            if ui.button(RichText::new("Delete").color(t.bad)).clicked() {
                let targets: Vec<EntityId> = entities.iter().map(|entity| entity.id).collect();
                commands.push(Box::new(RemoveEntityInstancesCommand::new(
                    level_id, targets,
                )));
                events.push(EditorEvent::SelectionChanged(Selection::None));
                events.push(EditorEvent::Status("Deleted entity".to_string()));
            }
        });

        self.strokes.end_frame();

        for command in commands {
            cx.exec_boxed(command);
        }
        for event in events {
            cx.emit(event);
        }
    }
}

struct LevelTransform<'a> {
    level_id: LevelId,
    entity: &'a EntityInstance,
    level_size: glam::UVec2,
    allow_out_of_bounds: bool,
    resizable_x: bool,
    resizable_y: bool,
}

fn render_header(ui: &mut Ui, entities: &[EntityInstance], defs: &[EntityDef], level_id: LevelId) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(RichText::new("ENTITY").color(t.faint).strong());
        if entities.len() == 1 {
            let name = defs
                .first()
                .map(|def| def.name.as_str())
                .unwrap_or("Entity");
            ui.label(
                RichText::new(format!("{name} #{}", entities[0].id.0))
                    .color(t.ink_strong)
                    .strong(),
            );
        } else {
            ui.label(
                RichText::new(format!("{} entities", entities.len()))
                    .color(t.ink_strong)
                    .strong(),
            );
        }
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new("LEVEL").color(t.faint));
        ui.label(RichText::new(format!("Level #{}", level_id.0)).color(t.muted));
    });
}

fn render_transform(
    ui: &mut Ui,
    transform: LevelTransform<'_>,
    strokes: &mut FocusStrokeTracker<String>,
    commands: &mut Vec<Box<dyn Command>>,
) {
    let t = Tokens::get(ui.ctx());
    let entity = transform.entity;
    let mut position = entity.pos_px;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("POSITION").color(t.faint));
        let x = ui.add(DragValue::new(&mut position.x).speed(1.0));
        let y = ui.add(DragValue::new(&mut position.y).speed(1.0));
        let changed = x.changed() || y.changed();
        let held = x.has_focus()
            || y.has_focus()
            || x.gained_focus()
            || y.gained_focus()
            || x.dragged()
            || y.dragged();
        if let Some(stroke) = strokes.id("position".to_string(), held, changed) {
            if changed {
                let clamped = clamp_position_for_def(
                    position,
                    entity.size_px,
                    transform.level_size,
                    transform.allow_out_of_bounds,
                );
                commands.push(Box::new(SetEntityPositionsCommand::with_stroke(
                    transform.level_id,
                    vec![(entity.id, clamped)],
                    stroke,
                )));
            }
        }
    });

    if transform.resizable_x || transform.resizable_y {
        let mut size = IVec2::new(entity.size_px.x as i32, entity.size_px.y as i32);
        ui.horizontal(|ui| {
            ui.label(RichText::new("SIZE").color(t.faint));
            let w = ui.add_enabled(
                transform.resizable_x,
                DragValue::new(&mut size.x).speed(1.0),
            );
            let h = ui.add_enabled(
                transform.resizable_y,
                DragValue::new(&mut size.y).speed(1.0),
            );
            let changed = w.changed() || h.changed();
            let held = w.has_focus()
                || h.has_focus()
                || w.gained_focus()
                || h.gained_focus()
                || w.dragged()
                || h.dragged();
            if let Some(stroke) = strokes.id("size".to_string(), held, changed) {
                if changed {
                    let max_w = transform.level_size.x.max(1) as i32;
                    let max_h = transform.level_size.y.max(1) as i32;
                    let clamped = IVec2::new(size.x.clamp(1, max_w), size.y.clamp(1, max_h));
                    commands.push(Box::new(SetEntitySizesCommand::with_stroke(
                        transform.level_id,
                        vec![(
                            entity.id,
                            glam::UVec2::new(clamped.x as u32, clamped.y as u32),
                        )],
                        stroke,
                    )));
                }
            }
        });
    }
}

fn effective_value(entity: &EntityInstance, field: &FieldDef, enum_defs: &[EnumDef]) -> FieldValue {
    entity
        .fields
        .get(&field.name)
        .map(|value| coerce_for_field_def(value, field, enum_defs))
        .unwrap_or_else(|| default_value_for_field(field, enum_defs))
}

pub fn entities_in_level(
    project: &Project,
    level_id: LevelId,
    ids: &[EntityId],
) -> Vec<EntityInstance> {
    project
        .level(level_id)
        .map(|level| {
            ids.iter()
                .filter_map(|id| level.entity(*id).cloned())
                .collect()
        })
        .unwrap_or_default()
}

pub fn common_fields(defs: &[EntityDef]) -> Vec<FieldDef> {
    let Some(first) = defs.first() else {
        return Vec::new();
    };
    first
        .fields
        .iter()
        .filter(|field| {
            defs.iter().all(|def| {
                def.field_def(&field.name)
                    .is_some_and(|other| other.field_type == field.field_type)
            })
        })
        .cloned()
        .collect()
}
