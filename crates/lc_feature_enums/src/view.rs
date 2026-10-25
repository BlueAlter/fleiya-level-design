//! Document view for editing `EnumDef` enumerations and their values.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{hud, meta_row, page, page_header};
use lc_core::command::{Command, DocKind};
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::DocumentView;

use crate::commands::{
    find_enum_def_references, find_enum_value_references, AddEnumValueCommand,
    EnumReferenceLocation, RemoveEnumDefCommand, RemoveEnumValueCommand, ReorderEnumValuesCommand,
    UpdateEnumDefCommand, UpdateEnumValueCommand,
};

/// Deferred actions emitted during UI traversal to prevent borrow conflicts with `cx.project()`.
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

/// Document tab view for inspecting and editing an `EnumDef` and its values.
pub struct EnumDefDocumentView {
    enum_def_id: EnumDefId,

    // Edit buffers
    name_buf: String,
    values_buf: Vec<EnumValueDef>,
    buffers_synced: bool,

    // Confirm dialog / state
    confirm_delete_enum: bool,
    cascade_delete_enum: bool,
}

impl EnumDefDocumentView {
    /// Creates a new document view for the given enum definition ID.
    pub fn new(enum_def_id: EnumDefId) -> Self {
        Self {
            enum_def_id,
            name_buf: String::new(),
            values_buf: Vec::new(),
            buffers_synced: false,
            confirm_delete_enum: false,
            cascade_delete_enum: false,
        }
    }

    /// Accessor for the target enum definition ID.
    pub fn enum_def_id(&self) -> EnumDefId {
        self.enum_def_id
    }

    /// Synchronizes local input buffers from the project definition.
    fn sync_buffers(&mut self, def: &EnumDef) {
        if !self.buffers_synced {
            self.name_buf = def.name.clone();
            self.values_buf = def.values.clone();
            self.buffers_synced = true;
        }
    }
}

impl DocumentView for EnumDefDocumentView {
    fn title(&self, p: &Project) -> String {
        p.defs
            .enum_def(self.enum_def_id)
            .map(|e| format!("Enum: {}", e.name))
            .unwrap_or_else(|| "Enum (deleted)".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.defs.enum_def(self.enum_def_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::EnumDef(self.enum_def_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let Some(enum_def) = cx.project().defs.enum_def(self.enum_def_id).cloned() else {
            ui.centered_and_justified(|ui| {
                ui.label("Enum definition does not exist in project.");
            });
            return;
        };

        self.sync_buffers(&enum_def);

        let t = Tokens::get(ui.ctx());
        let mut actions = ActionQueue::default();
        let references = find_enum_def_references(cx.project(), self.enum_def_id);

        page(ui, |ui| {
            // 1. Header: Name & ID
            page_header(
                ui,
                "ENUM",
                &format!("ENUM DEFINITION #{}", self.enum_def_id.0),
                "Configure enumeration values, badge colors, and optional tile representations.",
            );

            ui.add_space(8.0);

            // Metadata card
            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::ZERO)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Enum Name").color(t.ink_strong));
                        let name_resp = ui.add(
                            TextEdit::singleline(&mut self.name_buf)
                                .desired_width(220.0)
                                .hint_text("Enum Name"),
                        );

                        if name_resp.lost_focus() && self.name_buf.trim() != enum_def.name {
                            let mut updated = enum_def.clone();
                            updated.name = self.name_buf.trim().to_string();
                            actions.exec(UpdateEnumDefCommand::new(updated));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !self.confirm_delete_enum {
                                if ui
                                    .button(RichText::new("Delete Enum").color(t.bad))
                                    .clicked()
                                {
                                    self.confirm_delete_enum = true;
                                }
                            } else {
                                if ui.button("Cancel").clicked() {
                                    self.confirm_delete_enum = false;
                                }

                                if references.is_empty() {
                                    if ui
                                        .button(RichText::new("Confirm Delete").color(t.bad))
                                        .clicked()
                                    {
                                        actions.exec(RemoveEnumDefCommand::safe(self.enum_def_id));
                                        actions.emit(EditorEvent::CloseDocument(DocKind::EnumDef(
                                            self.enum_def_id,
                                        )));
                                    }
                                } else {
                                    ui.checkbox(&mut self.cascade_delete_enum, "Force Cascade");
                                    if self.cascade_delete_enum {
                                        if ui
                                            .button(RichText::new("Delete & Cascade").color(t.bad))
                                            .clicked()
                                        {
                                            actions.exec(RemoveEnumDefCommand::cascade(
                                                self.enum_def_id,
                                            ));
                                            actions.emit(EditorEvent::CloseDocument(
                                                DocKind::EnumDef(self.enum_def_id),
                                            ));
                                        }
                                    } else {
                                        ui.label(
                                            RichText::new("Referenced by entity fields")
                                                .color(t.warn),
                                        );
                                    }
                                }
                            }
                        });
                    });
                });

            // References Info Banner if used
            if !references.is_empty() {
                ui.add_space(8.0);
                Frame::new()
                    .fill(theme::tint(t.warn, 0.14))
                    .stroke(Stroke::new(1.0, t.warn))
                    .inner_margin(Margin::same(10))
                    .corner_radius(CornerRadius::ZERO)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Referenced by {} field(s):", references.len()))
                                .color(t.warn)
                                .strong(),
                        );
                        for r in &references {
                            if let EnumReferenceLocation::EntityField {
                                entity_name,
                                field_name,
                                ..
                            } = r
                            {
                                ui.label(
                                    RichText::new(format!(
                                        "  - Entity '{}' -> Field '{}'",
                                        entity_name, field_name
                                    ))
                                    .color(t.ink_strong),
                                );
                            }
                        }
                    });
            }

            ui.add_space(16.0);

            // 2. Enum Values Section
            meta_row(ui, "VALUES", &format!("{} value(s)", enum_def.values.len()));
            ui.add_space(8.0);

            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::ZERO)
                .show(ui, |ui| {
                    if enum_def.values.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            hud(ui, "NO VALUES DEFINED YET", t.muted);
                            ui.add_space(8.0);
                        });
                    } else {
                        ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                            let num_values = enum_def.values.len();
                            for idx in 0..num_values {
                                let val = &enum_def.values[idx];
                                let val_id = val.id.clone();
                                let mut val_name = val.name.clone();
                                let mut color_32 = Color32::from_rgba_unmultiplied(
                                    val.color[0],
                                    val.color[1],
                                    val.color[2],
                                    val.color[3],
                                );

                                ui.horizontal(|ui| {
                                    // Reorder controls: Up / Down
                                    if idx > 0 {
                                        if ui.small_button("▲").clicked() {
                                            let mut new_order: Vec<String> = enum_def
                                                .values
                                                .iter()
                                                .map(|v| v.id.clone())
                                                .collect();
                                            new_order.swap(idx, idx - 1);
                                            actions.exec(ReorderEnumValuesCommand::new(
                                                self.enum_def_id,
                                                new_order,
                                            ));
                                        }
                                    } else {
                                        ui.add_space(17.0);
                                    }

                                    if idx + 1 < num_values {
                                        if ui.small_button("▼").clicked() {
                                            let mut new_order: Vec<String> = enum_def
                                                .values
                                                .iter()
                                                .map(|v| v.id.clone())
                                                .collect();
                                            new_order.swap(idx, idx + 1);
                                            actions.exec(ReorderEnumValuesCommand::new(
                                                self.enum_def_id,
                                                new_order,
                                            ));
                                        }
                                    } else {
                                        ui.add_space(17.0);
                                    }

                                    // Badge index
                                    ui.label(
                                        RichText::new(format!("#{idx}")).color(t.muted).monospace(),
                                    );

                                    // Value ID key
                                    ui.label(
                                        RichText::new(&val.id)
                                            .color(t.accent_ink)
                                            .monospace()
                                            .strong(),
                                    );

                                    // Display Name TextEdit
                                    let name_resp = ui.add(
                                        TextEdit::singleline(&mut val_name)
                                            .desired_width(140.0)
                                            .hint_text("Display Name"),
                                    );
                                    if name_resp.lost_focus() && val_name != val.name {
                                        let mut updated_val = val.clone();
                                        updated_val.name = val_name;
                                        actions.exec(UpdateEnumValueCommand::new(
                                            self.enum_def_id,
                                            updated_val,
                                        ));
                                    }

                                    // Color picker button
                                    if ui.color_edit_button_srgba(&mut color_32).changed() {
                                        let mut updated_val = val.clone();
                                        updated_val.color = color_32.to_array();
                                        actions.exec(UpdateEnumValueCommand::new(
                                            self.enum_def_id,
                                            updated_val,
                                        ));
                                    }

                                    // Tile icon association
                                    if let Some(ref tile_ref) = val.tile {
                                        ui.label(
                                            RichText::new(format!(
                                                "Tile #{} (TS #{})",
                                                tile_ref.tile_id, tile_ref.tileset_id.0
                                            ))
                                            .color(t.muted),
                                        );
                                        if ui.small_button("Clear").clicked() {
                                            let mut updated_val = val.clone();
                                            updated_val.tile = None;
                                            actions.exec(UpdateEnumValueCommand::new(
                                                self.enum_def_id,
                                                updated_val,
                                            ));
                                        }
                                    } else {
                                        // Check if user has an active palette tile selection
                                        if let Some(ts) = cx.tile_selection() {
                                            if let Some(primary) = ts.primary_tile() {
                                                if ui.small_button("+ Use Palette Tile").clicked() {
                                                    let mut updated_val = val.clone();
                                                    updated_val.tile = Some(TileRef::new(
                                                        primary.tileset_id,
                                                        primary.tile_id,
                                                    ));
                                                    actions.exec(UpdateEnumValueCommand::new(
                                                        self.enum_def_id,
                                                        updated_val,
                                                    ));
                                                }
                                            }
                                        }
                                    }

                                    // Delete value button
                                    let val_refs = find_enum_value_references(
                                        cx.project(),
                                        self.enum_def_id,
                                        &val_id,
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button(RichText::new("X").color(t.bad)).clicked()
                                            {
                                                if val_refs.is_empty() {
                                                    actions.exec(RemoveEnumValueCommand::safe(
                                                        self.enum_def_id,
                                                        &val_id,
                                                    ));
                                                } else {
                                                    actions.exec(RemoveEnumValueCommand::cascade(
                                                        self.enum_def_id,
                                                        &val_id,
                                                    ));
                                                }
                                            }

                                            if !val_refs.is_empty() {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "Used in {}",
                                                        val_refs.len()
                                                    ))
                                                    .color(t.warn)
                                                    .font(egui::FontId::new(
                                                        11.0,
                                                        egui::FontFamily::Monospace,
                                                    )),
                                                );
                                            }
                                        },
                                    );
                                });

                                ui.add_space(4.0);
                            }
                        });
                    }

                    ui.add_space(8.0);

                    // Add Value row
                    ui.horizontal(|ui| {
                        if ui.button("+ Add Value").clicked() {
                            let next_num = enum_def.values.len() + 1;
                            let new_id = format!("Value_{next_num}");
                            let new_name = format!("Value {next_num}");

                            // Generate attractive default colors in a cycle
                            let default_colors: [[u8; 4]; 6] = [
                                [80, 160, 240, 255],  // Blue
                                [100, 200, 120, 255], // Green
                                [240, 160, 60, 255],  // Orange
                                [220, 80, 80, 255],   // Red
                                [180, 100, 220, 255], // Purple
                                [240, 220, 80, 255],  // Yellow
                            ];
                            let color = default_colors[(next_num - 1) % default_colors.len()];

                            let new_val = EnumValueDef {
                                id: new_id,
                                name: new_name,
                                color,
                                tile: None,
                            };

                            actions.exec(AddEnumValueCommand::new(self.enum_def_id, new_val));
                        }
                    });
                });
        });

        // Apply deferred actions
        for cmd in actions.commands {
            cx.exec_boxed(cmd);
            self.buffers_synced = false;
        }
        for event in actions.events {
            cx.emit(event);
        }
    }
}
