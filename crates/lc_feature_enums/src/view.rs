//! Document view for editing `EnumDef` enumerations and their values matching reference picture 2.

use std::collections::HashSet;

use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, ButtonKind};
use lc_core::command::{Command, DocKind};
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::i18n::{keys, tr};
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
    active_sub_tab: usize, // 0 = ENUM DEF, 1 = DOCUMENT

    // Edit buffers
    name_buf: String,
    display_title_buf: String,
    is_bitmask: bool,
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
            active_sub_tab: 0,
            name_buf: String::new(),
            display_title_buf: String::new(),
            is_bitmask: false,
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
            if self.display_title_buf.is_empty() {
                self.display_title_buf = format!("{} Tier", def.name);
            }
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

        let mut unique_entities = HashSet::new();
        for r in &references {
            if let EnumReferenceLocation::EntityField { entity_id, .. } = r {
                unique_entities.insert(*entity_id);
            }
        }
        let entity_count = unique_entities.len();
        let field_count = references.len();

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);

            // 1. Top Sub-tabs (ENUM DEF / DOCUMENT) + Project Enums button
            ui.horizontal(|ui| {
                let is_def = self.active_sub_tab == 0;
                let btn_def = Frame::new()
                    .fill(if is_def { t.sunken } else { Color32::TRANSPARENT })
                    .stroke(if is_def { Stroke::new(1.0, t.accent) } else { Stroke::NONE })
                    .inner_margin(Margin::symmetric(10, 4))
                    .corner_radius(CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(tr(keys::ENUMS_TAB_DEFINITION))
                                .size(11.0)
                                .strong()
                                .color(if is_def { t.accent_ink } else { t.muted }),
                        );
                    });
                if btn_def.response.interact(egui::Sense::click()).clicked() {
                    self.active_sub_tab = 0;
                }

                let is_doc = self.active_sub_tab == 1;
                let btn_doc = Frame::new()
                    .fill(if is_doc { t.sunken } else { Color32::TRANSPARENT })
                    .stroke(if is_doc { Stroke::new(1.0, t.accent) } else { Stroke::NONE })
                    .inner_margin(Margin::symmetric(10, 4))
                    .corner_radius(CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(tr(keys::ENUMS_TAB_DOCUMENT))
                                .size(11.0)
                                .color(if is_doc { t.accent_ink } else { t.muted }),
                        );
                    });
                if btn_doc.response.interact(egui::Sense::click()).clicked() {
                    self.active_sub_tab = 1;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = Frame::new()
                        .stroke(Stroke::new(1.0, t.accent))
                        .corner_radius(CornerRadius::same(2))
                        .inner_margin(Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("PROJECT ENUMS")
                                    .size(10.5)
                                    .strong()
                                    .color(t.accent_ink),
                            );
                        });
                });
            });

            ui.add_space(10.0);

            // 2. Heading
            ui.heading(
                RichText::new(format!("{} (Enum)", enum_def.name))
                    .size(22.0)
                    .strong()
                    .color(t.ink_strong),
            );
            ui.label(
                RichText::new(tr(keys::ENUMS_DESCRIPTION))
                    .size(12.0)
                    .color(t.muted),
            );

            ui.add_space(14.0);

            // 3. Section: ENUM IDENTIFIER & ATTRIBUTES
            ui.label(
                RichText::new("ENUM IDENTIFIER & ATTRIBUTES")
                    .size(11.0)
                    .strong()
                    .color(t.accent_ink),
            );
            ui.add_space(4.0);

            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Enum Identifier").strong().color(t.ink_strong));
                        let name_resp = ui.add(
                            TextEdit::singleline(&mut self.name_buf)
                                .desired_width(180.0)
                                .hint_text("Identifier"),
                        );
                        if name_resp.lost_focus() && self.name_buf.trim() != enum_def.name && !self.name_buf.trim().is_empty() {
                            let mut updated = enum_def.clone();
                            updated.name = self.name_buf.trim().to_string();
                            actions.exec(UpdateEnumDefCommand::new(updated));
                        }

                        ui.add_space(16.0);

                        ui.label(RichText::new("Display Title").strong().color(t.ink_strong));
                        ui.add(
                            TextEdit::singleline(&mut self.display_title_buf)
                                .desired_width(180.0)
                                .hint_text("Display Title"),
                        );

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
                                    if self.cascade_delete_enum
                                        && ui
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
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.is_bitmask,
                            RichText::new(
                                "Export as bitflag / multiselect bitmask (allows combining multiple values)",
                            )
                            .color(t.muted)
                            .size(11.5),
                        );
                    });
                });

            // References alert if any
            if !references.is_empty() {
                ui.add_space(8.0);
                Frame::new()
                    .fill(theme::tint(t.warn, 0.12))
                    .stroke(Stroke::new(1.0, t.warn))
                    .inner_margin(Margin::same(8))
                    .corner_radius(CornerRadius::same(3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Referenced by {} entity def(s) across {} field(s)",
                                entity_count, field_count
                            ))
                            .color(t.warn)
                            .strong()
                            .size(11.0),
                        );
                    });
            }

            ui.add_space(14.0);

            // 4. Section: ENUM VALUES (X DEFINED)
            ui.label(
                RichText::new(format!("ENUM VALUES ({} DEFINED)", enum_def.values.len()))
                    .size(11.0)
                    .strong()
                    .color(t.accent_ink),
            );
            ui.add_space(4.0);

            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::same(4))
                .show(ui, |ui| {
                    // Table Header
                    ui.horizontal(|ui| {
                        ui.set_min_height(24.0);
                        ui.add_sized(
                            [140.0, 20.0],
                            egui::Label::new(
                                RichText::new("KEY / ID")
                                    .size(11.0)
                                    .strong()
                                    .color(t.muted),
                            ),
                        );
                        ui.add_sized(
                            [160.0, 20.0],
                            egui::Label::new(
                                RichText::new("DISPLAY LABEL")
                                    .size(11.0)
                                    .strong()
                                    .color(t.muted),
                            ),
                        );
                        ui.add_sized(
                            [120.0, 20.0],
                            egui::Label::new(
                                RichText::new("BADGE COLOR")
                                    .size(11.0)
                                    .strong()
                                    .color(t.muted),
                            ),
                        );
                        ui.add_sized(
                            [140.0, 20.0],
                            egui::Label::new(
                                RichText::new("BOUND TILE")
                                    .size(11.0)
                                    .strong()
                                    .color(t.muted),
                            ),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("ACTION")
                                    .size(11.0)
                                    .strong()
                                    .color(t.muted),
                            );
                        });
                    });

                    ui.separator();
                    ui.add_space(4.0);

                    if enum_def.values.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("No enum values defined yet.")
                                    .color(t.muted)
                                    .italics(),
                            );
                            ui.add_space(12.0);
                        });
                    } else {
                        let num_values = enum_def.values.len();
                        for idx in 0..num_values {
                            let val = &enum_def.values[idx];
                            let val_id = val.id.clone();
                            let mut val_id_buf = val.id.clone();
                            let mut val_name_buf = val.name.clone();
                            let mut color_32 = Color32::from_rgba_unmultiplied(
                                val.color[0],
                                val.color[1],
                                val.color[2],
                                val.color[3],
                            );

                            ui.horizontal(|ui| {
                                ui.set_min_height(28.0);

                                // Key / ID column
                                ui.allocate_ui_with_layout(
                                    egui::vec2(140.0, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        let key_resp = ui.add(
                                            TextEdit::singleline(&mut val_id_buf)
                                                .desired_width(130.0)
                                                .hint_text("Key"),
                                        );
                                        if key_resp.lost_focus() && val_id_buf != val.id && !val_id_buf.trim().is_empty() {
                                            let mut updated_val = val.clone();
                                            updated_val.id = val_id_buf.trim().to_string();
                                            actions.exec(UpdateEnumValueCommand::new(
                                                self.enum_def_id,
                                                updated_val,
                                            ));
                                        }
                                    },
                                );

                                // Display Label column
                                ui.allocate_ui_with_layout(
                                    egui::vec2(160.0, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        let name_resp = ui.add(
                                            TextEdit::singleline(&mut val_name_buf)
                                                .desired_width(150.0)
                                                .hint_text("Display Label"),
                                        );
                                        if name_resp.lost_focus() && val_name_buf != val.name {
                                            let mut updated_val = val.clone();
                                            updated_val.name = val_name_buf;
                                            actions.exec(UpdateEnumValueCommand::new(
                                                self.enum_def_id,
                                                updated_val,
                                            ));
                                        }
                                    },
                                );

                                // Badge Color column
                                ui.allocate_ui_with_layout(
                                    egui::vec2(120.0, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if ui.color_edit_button_srgba(&mut color_32).changed() {
                                            let mut updated_val = val.clone();
                                            updated_val.color = color_32.to_array();
                                            actions.exec(UpdateEnumValueCommand::new(
                                                self.enum_def_id,
                                                updated_val,
                                            ));
                                        }
                                        let hex = format!(
                                            "#{:02x}{:02x}{:02x}",
                                            val.color[0], val.color[1], val.color[2]
                                        );
                                        ui.label(
                                            RichText::new(hex)
                                                .monospace()
                                                .size(11.0)
                                                .color(t.muted),
                                        );
                                    },
                                );

                                // Bound Tile column
                                ui.allocate_ui_with_layout(
                                    egui::vec2(140.0, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if let Some(ref tile_ref) = val.tile {
                                            ui.label(
                                                RichText::new("📦")
                                                    .color(Color32::from_rgb(240, 180, 40)),
                                            );
                                            ui.label(
                                                RichText::new(format!("tile_{}", tile_ref.tile_id))
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(t.ink_strong),
                                            );
                                            if ui.small_button("×").clicked() {
                                                let mut updated_val = val.clone();
                                                updated_val.tile = None;
                                                actions.exec(UpdateEnumValueCommand::new(
                                                    self.enum_def_id,
                                                    updated_val,
                                                ));
                                            }
                                        } else if let Some(ts) = cx.tile_selection() {
                                            if let Some(primary) = ts.primary_tile() {
                                                if ui.small_button("+ Bind Tile").clicked() {
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
                                            } else {
                                                ui.label(RichText::new("—").color(t.muted));
                                            }
                                        } else {
                                            ui.label(RichText::new("—").color(t.muted));
                                        }
                                    },
                                );

                                // Action column: Delete (red text)
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let val_refs = find_enum_value_references(
                                            cx.project(),
                                            self.enum_def_id,
                                            &val_id,
                                        );

                                        if ui
                                            .button(RichText::new("Delete").color(t.bad))
                                            .clicked()
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
                                                RichText::new(format!("{} refs", val_refs.len()))
                                                    .color(t.warn)
                                                    .size(10.5),
                                            );
                                        }
                                    },
                                );
                            });

                            ui.separator();
                        }
                    }

                    ui.add_space(8.0);

                    // Add Enum Value & Sort A-Z buttons
                    ui.horizontal(|ui| {
                        if button(ui, "+ Add Enum Value", ButtonKind::Primary, true).clicked() {
                            let next_num = enum_def.values.len() + 1;
                            let new_id = format!("Value_{next_num}");
                            let new_name = format!("Value {next_num}");

                            let default_colors: [[u8; 4]; 6] = [
                                [156, 163, 175, 255], // Gray #9ca3af
                                [61, 214, 184, 255],  // Teal #3fd6b8
                                [45, 129, 255, 255],  // Blue #2d81ff
                                [168, 85, 247, 255],  // Purple #a855f7
                                [255, 193, 77, 255],  // Amber #ffc14d
                                [239, 68, 68, 255],   // Red #ef4444
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

                        if ui.button("Sort A-Z").clicked() {
                            let mut sorted_ids: Vec<String> = enum_def
                                .values
                                .iter()
                                .map(|v| v.id.clone())
                                .collect();
                            sorted_ids.sort();
                            actions.exec(ReorderEnumValuesCommand::new(
                                self.enum_def_id,
                                sorted_ids,
                            ));
                        }
                    });
                });

            ui.add_space(14.0);

            // 5. Bottom Action Bar (matching picture-2.png)
            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::symmetric(14, 10))
                .corner_radius(CornerRadius::same(3))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "ID: EnumDefId({}) • Used by {} Entity Def(s) & {} Field(s)",
                                self.enum_def_id.0, entity_count, field_count
                            ))
                            .size(11.5)
                            .color(t.muted),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if button(ui, "Save Enum Def", ButtonKind::Primary, true).clicked() {
                                let mut updated = enum_def.clone();
                                updated.name = self.name_buf.trim().to_string();
                                actions.exec(UpdateEnumDefCommand::new(updated));
                                cx.status(format!("Saved enum definition {}", self.name_buf));
                            }

                            if ui.button("Revert").clicked() {
                                self.buffers_synced = false;
                                self.sync_buffers(&enum_def);
                                cx.status("Reverted enum definition edits");
                            }
                        });
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
