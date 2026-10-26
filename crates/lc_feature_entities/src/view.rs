//! Document view for editing an [`EntityDef`] and its field schema.

use egui::{Color32, CornerRadius, DragValue, Frame, Margin, RichText, TextEdit, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::controls::icon_button;
use lc_canvas::widgets::{hud, meta_row, page, page_header};
use lc_core::command::{Command, DocKind};
use lc_core::defs::EntityDef;
use lc_core::fields::{FieldDef, FieldType};
use lc_core::id::EntityDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::DocumentView;

use crate::commands::{RemoveEntityDefCommand, UpdateEntityDefCommand};
use crate::field_commands::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, UpdateEntityFieldCommand,
};
use crate::field_defs::{coerce_field_value, default_field_value};
use crate::field_editors::{
    entity_choices, field_type_editor, field_value_editor, FieldEditorContext,
};
use crate::helpers::count_entity_def_instances;
use crate::stroke::StrokeTracker;

/// Edit action that owns a merge stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetaField {
    /// Definition-level metadata (name, size, colour, pivot, tile).
    Metadata,
    /// A field row, including its name, type and default value.
    Field(usize),
}

#[derive(Default)]
struct ActionQueue {
    commands: Vec<Box<dyn Command>>,
    events: Vec<EditorEvent>,
}

impl ActionQueue {
    fn exec(&mut self, cmd: impl Command + 'static) {
        self.commands.push(Box::new(cmd));
    }

    fn emit(&mut self, event: EditorEvent) {
        self.events.push(event);
    }
}

/// Document tab for an entity definition.
pub struct EntityDefDocumentView {
    def_id: EntityDefId,
    new_field_name: String,
    new_field_type: FieldType,
    confirm_delete: bool,
    cascade_delete: bool,
    strokes: StrokeTracker<MetaField>,
}

impl EntityDefDocumentView {
    /// Creates a view for `def_id`.
    pub fn new(def_id: EntityDefId) -> Self {
        Self {
            def_id,
            new_field_name: String::new(),
            new_field_type: FieldType::Int,
            confirm_delete: false,
            cascade_delete: false,
            strokes: StrokeTracker::default(),
        }
    }

    /// The definition shown by this tab.
    pub fn def_id(&self) -> EntityDefId {
        self.def_id
    }
}

impl DocumentView for EntityDefDocumentView {
    fn title(&self, p: &Project) -> String {
        p.defs
            .entity(self.def_id)
            .map(|e| format!("Entity: {}", e.name))
            .unwrap_or_else(|| "Entity (deleted)".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.defs.entity(self.def_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::EntityDef(self.def_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let Some(def) = cx.project().defs.entity(self.def_id).cloned() else {
            ui.centered_and_justified(|ui| {
                ui.label("Entity definition does not exist in project.");
            });
            return;
        };

        let enums = cx.project().defs.enums.clone();
        let choices = entity_choices(cx.project());
        let editor_ctx = FieldEditorContext::new(&enums, &choices);
        let palette_tile = cx
            .tile_selection()
            .and_then(|selection| selection.primary_tile());
        let usage = count_entity_def_instances(cx.project(), self.def_id);

        let mut actions = ActionQueue::default();
        self.strokes.begin_frame();

        page(ui, |ui| {
            page_header(
                ui,
                "ENTITY",
                &format!("ENTITY DEFINITION #{}", self.def_id.0),
                "Configure the entity footprint, placeholder colour, pivot and typed field schema.",
            );

            self.render_metadata(ui, &def, usage, palette_tile, &mut actions);

            ui.add_space(16.0);
            meta_row(ui, "FIELDS", &format!("{} field(s)", def.fields.len()));
            ui.add_space(8.0);

            Frame::new()
                .fill(Tokens::get(ui.ctx()).sunken)
                .stroke(Tokens::get(ui.ctx()).hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::ZERO)
                .show(ui, |ui| {
                    if def.fields.is_empty() {
                        hud(ui, "NO FIELDS DEFINED YET", Tokens::get(ui.ctx()).muted);
                    } else {
                        for (index, field) in def.fields.iter().enumerate() {
                            render_field_row(
                                ui,
                                index,
                                field,
                                &def,
                                &editor_ctx,
                                &mut actions,
                                &mut self.strokes,
                            );
                            ui.add_space(4.0);
                        }
                    }
                    ui.add_space(8.0);
                    render_new_field(
                        ui,
                        &def,
                        &mut self.new_field_name,
                        &mut self.new_field_type,
                        &editor_ctx,
                        &mut actions,
                    );
                });

            if usage > 0 {
                ui.add_space(12.0);
                hud(
                    ui,
                    &format!("USED BY {usage} INSTANCE(S)"),
                    Tokens::get(ui.ctx()).warn,
                );
            }
        });

        self.strokes.end_frame();
        for cmd in actions.commands {
            cx.exec_boxed(cmd);
        }
        for event in actions.events {
            cx.emit(event);
        }
    }
}

impl EntityDefDocumentView {
    fn render_metadata(
        &mut self,
        ui: &mut Ui,
        def: &EntityDef,
        usage: usize,
        palette_tile: Option<TileRef>,
        actions: &mut ActionQueue,
    ) {
        let confirm_delete = &mut self.confirm_delete;
        let cascade_delete = &mut self.cascade_delete;
        let strokes = &mut self.strokes;
        let t = Tokens::get(ui.ctx());
        let mut draft = def.clone();

        Frame::new()
            .fill(t.sunken)
            .stroke(t.hairline())
            .inner_margin(Margin::same(12))
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Name").color(t.ink_strong));
                    ui.add(TextEdit::singleline(&mut draft.name).desired_width(200.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Colour").color(t.ink_strong));
                    let mut color = Color32::from_rgba_unmultiplied(
                        draft.color[0],
                        draft.color[1],
                        draft.color[2],
                        draft.color[3],
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        draft.color = color.to_array();
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Size").color(t.ink_strong));
                    ui.add(DragValue::new(&mut draft.size_px.x).range(1..=4096));
                    ui.label("x");
                    ui.add(DragValue::new(&mut draft.size_px.y).range(1..=4096));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Pivot").color(t.ink_strong));
                    ui.add(
                        DragValue::new(&mut draft.pivot.x)
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    ui.add(
                        DragValue::new(&mut draft.pivot.y)
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tile").color(t.ink_strong));
                    if let Some(tile) = &draft.tile {
                        ui.label(format!(
                            "Tile #{} (TS #{})",
                            tile.tile_id, tile.tileset_id.0
                        ));
                        if ui.small_button("Clear").clicked() {
                            draft.tile = None;
                        }
                    } else {
                        ui.label(RichText::new("none").color(t.faint));
                        if let Some(tile) = palette_tile {
                            if ui.small_button("Use Palette Tile").clicked() {
                                draft.tile = Some(tile);
                            }
                        }
                    }
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if !*confirm_delete {
                        if ui
                            .button(RichText::new("Delete Entity").color(t.bad))
                            .clicked()
                        {
                            *confirm_delete = true;
                        }
                    } else {
                        if ui.button("Cancel").clicked() {
                            *confirm_delete = false;
                        }
                        if usage == 0 {
                            if ui
                                .button(RichText::new("Confirm Delete").color(t.bad))
                                .clicked()
                            {
                                actions.exec(RemoveEntityDefCommand::safe(def.id));
                                actions
                                    .emit(EditorEvent::CloseDocument(DocKind::EntityDef(def.id)));
                            }
                        } else {
                            ui.checkbox(cascade_delete, "Force cascade");
                            if *cascade_delete
                                && ui
                                    .button(RichText::new("Delete & Cascade").color(t.bad))
                                    .clicked()
                            {
                                actions.exec(RemoveEntityDefCommand::cascade(def.id));
                                actions
                                    .emit(EditorEvent::CloseDocument(DocKind::EntityDef(def.id)));
                            }
                            if !*cascade_delete {
                                ui.label(
                                    RichText::new(format!("Referenced by {usage} instance(s)"))
                                        .color(t.warn),
                                );
                            }
                        }
                    }
                });
            });

        if draft != *def {
            actions.exec(UpdateEntityDefCommand::with_stroke(
                draft,
                strokes.id(MetaField::Metadata),
            ));
        }
    }
}

fn render_field_row(
    ui: &mut Ui,
    index: usize,
    field: &FieldDef,
    def: &EntityDef,
    editor_ctx: &FieldEditorContext<'_>,
    actions: &mut ActionQueue,
    strokes: &mut StrokeTracker<MetaField>,
) {
    let t = Tokens::get(ui.ctx());
    let mut updated = field.clone();
    let mut changed = false;

    ui.horizontal(|ui| {
        let mut name = field.name.clone();
        if ui
            .add(TextEdit::singleline(&mut name).desired_width(120.0))
            .changed()
            && !name.trim().is_empty()
        {
            updated.name = name;
            changed = true;
        }

        if let Some(field_type) = field_type_editor(ui, &field.field_type, editor_ctx.enum_defs) {
            updated.field_type = field_type;
            changed = true;
        }

        ui.label(RichText::new("default").color(t.faint));
        let mut default = field
            .default_value
            .clone()
            .map(|value| coerce_field_value(&value, &field.field_type, editor_ctx.enum_defs))
            .unwrap_or_else(|| default_field_value(&field.field_type, editor_ctx.enum_defs));
        let response = field_value_editor(ui, &mut default, &field.field_type, editor_ctx);
        if response.changed {
            updated.default_value = Some(default);
            changed = true;
        }

        if icon_button(ui, Icon::Delete, "Delete field").clicked() {
            actions.exec(RemoveEntityFieldCommand::new(def.id, field.name.clone()));
        }
    });

    if changed {
        actions.exec(UpdateEntityFieldCommand::with_stroke(
            def.id,
            field.name.clone(),
            updated,
            strokes.id(MetaField::Field(index)),
        ));
    }
}

fn render_new_field(
    ui: &mut Ui,
    def: &EntityDef,
    new_field_name: &mut String,
    new_field_type: &mut FieldType,
    editor_ctx: &FieldEditorContext<'_>,
    actions: &mut ActionQueue,
) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(RichText::new("NEW FIELD").color(t.accent_ink));
        ui.add(
            TextEdit::singleline(new_field_name)
                .desired_width(140.0)
                .hint_text("field name"),
        );
        if let Some(field_type) = field_type_editor(ui, new_field_type, editor_ctx.enum_defs) {
            *new_field_type = field_type;
        }
        let name = new_field_name.trim();
        let duplicate = def.field_def(name).is_some();
        if duplicate {
            ui.label(RichText::new("name in use").color(t.warn));
        }
        if ui.button("Add Field").clicked() && !name.is_empty() && !duplicate {
            actions.exec(AddEntityFieldCommand::new(
                def.id,
                FieldDef {
                    name: name.to_string(),
                    field_type: new_field_type.clone(),
                    default_value: None,
                },
            ));
            new_field_name.clear();
        }
    });
}
