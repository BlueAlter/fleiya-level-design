//! Document view for editing `LayerDef` template definitions.

use egui::{Color32, ComboBox, CornerRadius, DragValue, Frame, Margin, RichText, TextEdit, Ui};
use glam::UVec2;
use lc_core::command::DocKind;
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::id::{LayerDefId, TilesetDefId};
use lc_core::layer::LayerKind;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::view::DocumentView;

use crate::commands::UpdateLayerDefCommand;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::{hud, meta_row, page, page_header};

/// Document tab view for editing a `LayerDef` template.
pub struct LayerDefDocumentView {
    layer_def_id: LayerDefId,

    // Edit buffers
    name_buf: String,
    kind: LayerKind,
    grid_w: u32,
    grid_h: u32,
    tileset_id: Option<TilesetDefId>,
    int_grid_values: Vec<IntGridValueDef>,
    buffers_synced: bool,
}

impl LayerDefDocumentView {
    /// Creates a new document view for the specified layer definition.
    pub fn new(layer_def_id: LayerDefId) -> Self {
        Self {
            layer_def_id,
            name_buf: String::new(),
            kind: LayerKind::Tiles,
            grid_w: 16,
            grid_h: 16,
            tileset_id: None,
            int_grid_values: Vec::new(),
            buffers_synced: false,
        }
    }

    /// Accessor for the layer definition ID.
    pub fn layer_def_id(&self) -> LayerDefId {
        self.layer_def_id
    }

    /// Syncs local input buffers from project definition.
    fn sync_buffers(&mut self, def: &LayerDef) {
        if !self.buffers_synced {
            self.name_buf = def.name.clone();
            self.kind = def.kind;
            self.grid_w = def.grid_size.x;
            self.grid_h = def.grid_size.y;
            self.tileset_id = def.tileset_id;
            self.int_grid_values = def.int_grid_values.clone();
            self.buffers_synced = true;
        }
    }

    /// Commits local input buffers to a command if differences exist.
    fn commit_changes(&mut self, def: &LayerDef, cx: &mut EditorCtx) {
        let updated = LayerDef {
            id: self.layer_def_id,
            name: self.name_buf.trim().to_string(),
            kind: self.kind,
            grid_size: UVec2::new(self.grid_w.max(1), self.grid_h.max(1)),
            tileset_id: self.tileset_id,
            int_grid_values: self.int_grid_values.clone(),
        };

        if *def != updated {
            cx.exec(UpdateLayerDefCommand::new(updated));
        }
    }
}

impl DocumentView for LayerDefDocumentView {
    fn title(&self, p: &Project) -> String {
        p.defs
            .layer(self.layer_def_id)
            .map(|l| format!("Layer: {}", l.name))
            .unwrap_or_else(|| "Layer (deleted)".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.defs.layer(self.layer_def_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::LayerDef(self.layer_def_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let Some(def) = cx.project().defs.layer(self.layer_def_id).cloned() else {
            ui.label("Layer definition does not exist.");
            return;
        };

        self.sync_buffers(&def);

        let t = Tokens::get(ui.ctx());

        page(ui, |ui| {
            page_header(
                ui,
                "LAYER DEF",
                &def.name,
                "Configure layer type, cell grid dimensions, tileset binding, and IntGrid value definitions.",
            );

            ui.add_space(16.0);

            // 1. Properties Section
            hud(ui, "PROPERTIES", t.accent_ink);
            ui.add_space(8.0);

            let mut commit_needed = false;

            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::ZERO)
                .show(ui, |ui| {
                    egui::Grid::new("layer_props_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            // Name
                            ui.label(RichText::new("Name").color(t.ink_strong));
                            let name_resp = ui
                                .add(TextEdit::singleline(&mut self.name_buf).desired_width(260.0));
                            if name_resp.lost_focus() {
                                commit_needed = true;
                            }
                            ui.end_row();

                            // Kind
                            ui.label(RichText::new("Layer Kind").color(t.ink_strong));
                            let current_kind_label = match self.kind {
                                LayerKind::IntGrid => "IntGrid (Integer collision/logic)",
                                LayerKind::Tiles => "Tiles (Tilemap)",
                                LayerKind::Entities => "Entities (Vector placements)",
                                LayerKind::Auto => "Auto (Rule-based automatic)",
                            };

                            ComboBox::from_id_salt("layer_kind_combo")
                                .selected_text(current_kind_label)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            self.kind == LayerKind::IntGrid,
                                            "IntGrid (Integer collision/logic)",
                                        )
                                        .clicked()
                                    {
                                        self.kind = LayerKind::IntGrid;
                                        commit_needed = true;
                                    }
                                    if ui
                                        .selectable_label(
                                            self.kind == LayerKind::Tiles,
                                            "Tiles (Tilemap)",
                                        )
                                        .clicked()
                                    {
                                        self.kind = LayerKind::Tiles;
                                        commit_needed = true;
                                    }
                                    if ui
                                        .selectable_label(
                                            self.kind == LayerKind::Entities,
                                            "Entities (Vector placements)",
                                        )
                                        .clicked()
                                    {
                                        self.kind = LayerKind::Entities;
                                        commit_needed = true;
                                    }
                                    if ui
                                        .selectable_label(
                                            self.kind == LayerKind::Auto,
                                            "Auto (Rule-based automatic)",
                                        )
                                        .clicked()
                                    {
                                        self.kind = LayerKind::Auto;
                                        commit_needed = true;
                                    }
                                });
                            ui.end_row();

                            // Grid Size
                            ui.label(RichText::new("Grid Size").color(t.ink_strong));
                            ui.horizontal(|ui| {
                                let w_resp = ui.add(
                                    DragValue::new(&mut self.grid_w)
                                        .range(1..=1024)
                                        .prefix("W: ")
                                        .suffix(" px"),
                                );
                                let h_resp = ui.add(
                                    DragValue::new(&mut self.grid_h)
                                        .range(1..=1024)
                                        .prefix("H: ")
                                        .suffix(" px"),
                                );
                                if w_resp.changed() || h_resp.changed() {
                                    commit_needed = true;
                                }
                            });
                            ui.end_row();

                            // Tileset Link
                            ui.label(RichText::new("Tileset Link").color(t.ink_strong));
                            let tileset_label = self
                                .tileset_id
                                .and_then(|id| cx.project().defs.tileset(id))
                                .map(|t| t.name.as_str())
                                .unwrap_or("None (Unlinked)");

                            ComboBox::from_id_salt("layer_tileset_link_combo")
                                .selected_text(tileset_label)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            self.tileset_id.is_none(),
                                            "None (Unlinked)",
                                        )
                                        .clicked()
                                    {
                                        self.tileset_id = None;
                                        commit_needed = true;
                                    }
                                    for t_def in &cx.project().defs.tilesets {
                                        let selected = self.tileset_id == Some(t_def.id);
                                        if ui.selectable_label(selected, &t_def.name).clicked() {
                                            self.tileset_id = Some(t_def.id);
                                            commit_needed = true;
                                        }
                                    }
                                });
                            ui.end_row();
                        });
                });

            ui.add_space(16.0);

            // 2. IntGrid Value Definitions (if IntGrid layer)
            if self.kind == LayerKind::IntGrid {
                hud(ui, "INTGRID VALUES", t.accent_ink);
                ui.add_space(8.0);

                Frame::new()
                    .fill(t.sunken)
                    .stroke(t.hairline())
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::ZERO)
                    .show(ui, |ui| {
                        let mut remove_idx = None;

                        for (idx, val_def) in self.int_grid_values.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{}", idx + 1)).color(t.muted));

                                let v_resp = ui.add(
                                    DragValue::new(&mut val_def.value)
                                        .range(-1000..=1000)
                                        .prefix("Val: "),
                                );
                                if v_resp.changed() {
                                    commit_needed = true;
                                }

                                let n_resp = ui.add(
                                    TextEdit::singleline(&mut val_def.name)
                                        .desired_width(120.0)
                                        .hint_text("Name"),
                                );
                                if n_resp.lost_focus() {
                                    commit_needed = true;
                                }

                                // Color button
                                let mut color = Color32::from_rgba_unmultiplied(
                                    val_def.color[0],
                                    val_def.color[1],
                                    val_def.color[2],
                                    val_def.color[3],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    val_def.color = color.to_array();
                                    commit_needed = true;
                                }

                                if ui.button("X").clicked() {
                                    remove_idx = Some(idx);
                                }
                            });
                            ui.add_space(4.0);
                        }

                        if let Some(idx) = remove_idx {
                            self.int_grid_values.remove(idx);
                            commit_needed = true;
                        }

                        ui.add_space(8.0);
                        if ui.button("+ Add Value").clicked() {
                            let next_val = (self.int_grid_values.len() + 1) as i32;
                            self.int_grid_values.push(IntGridValueDef {
                                value: next_val,
                                name: format!("Value_{next_val}"),
                                color: [220, 100, 50, 255],
                            });
                            commit_needed = true;
                        }
                    });

                ui.add_space(16.0);
            }

            if commit_needed {
                self.commit_changes(&def, cx);
            }

            // Metadata rows
            meta_row(ui, "LAYER DEF ID", &format!("#{}", self.layer_def_id.0));
            meta_row(ui, "KIND", &format!("{:?}", def.kind));
            meta_row(
                ui,
                "GRID DIMENSIONS",
                &format!("{} x {} px", def.grid_size.x, def.grid_size.y),
            );
        });
    }
}
