//! Dockable panel for managing layers in the active level tab.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, Ui};
use glam::UVec2;
use lc_canvas::theme::Tokens;
use lc_core::command::{Command, DocKind};
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::id::LayerDefId;
use lc_core::layer::LayerKind;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::{PanelId, PanelView};

use crate::commands::{
    AddLayerDefCommand, RemoveLayerDefCommand, ReorderLayerDefsCommand,
    ToggleLayerVisibilityCommand,
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

/// Dockable panel view for inspecting, ordering, toggling visibility, and selecting layers of the active level.
#[derive(Default)]
pub struct LayersPanel;

impl LayersPanel {
    /// Identifier constant for the Layers dockable panel.
    pub const PANEL_ID: &'static str = "layers";

    /// Creates a new `LayersPanel`.
    pub fn new() -> Self {
        Self
    }
}

impl PanelView for LayersPanel {
    fn id(&self) -> PanelId {
        PanelId::new(Self::PANEL_ID)
    }

    fn title(&self) -> &str {
        "Layers"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());

        // 1. Check active level tab
        let Some(level_id) = cx.active_level() else {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("No active level.").color(t.muted));
                ui.label(
                    RichText::new("Open a level tab to inspect and manage its layers.")
                        .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(t.faint),
                );
            });
            return;
        };

        let Some(level) = cx.project().level(level_id) else {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("Active level not found.").color(t.muted));
            });
            return;
        };

        let mut actions = ActionQueue::default();
        let layers = level.layers.clone();
        let total_layers = layers.len();
        let active_layer_id = cx.active_layer();
        let active_layer_index =
            active_layer_id.and_then(|id| layers.iter().position(|l| l.def_id == id));

        // 2. Top Toolbar
        Frame::new()
            .fill(t.sunken)
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("+ Add Layer", |ui| {
                        if ui.button("Tiles Layer").clicked() {
                            let next_id = LayerDefId(
                                cx.project()
                                    .defs
                                    .layers
                                    .iter()
                                    .map(|l| l.id.0)
                                    .max()
                                    .unwrap_or(0)
                                    + 1,
                            );
                            let default_tileset =
                                cx.project().defs.tilesets.first().map(|ts| ts.id);
                            let def = LayerDef {
                                id: next_id,
                                name: format!("Layer_{}", next_id.0),
                                kind: LayerKind::Tiles,
                                grid_size: UVec2::new(16, 16),
                                tileset_id: default_tileset,
                                int_grid_values: Vec::new(),
                            };
                            actions.exec(AddLayerDefCommand::new(def));
                            actions.emit(EditorEvent::SetActiveLayer(Some(next_id)));
                            actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(next_id)));
                            ui.close();
                        }

                        if ui.button("IntGrid Layer").clicked() {
                            let next_id = LayerDefId(
                                cx.project()
                                    .defs
                                    .layers
                                    .iter()
                                    .map(|l| l.id.0)
                                    .max()
                                    .unwrap_or(0)
                                    + 1,
                            );
                            let def = LayerDef {
                                id: next_id,
                                name: format!("Layer_{}", next_id.0),
                                kind: LayerKind::IntGrid,
                                grid_size: UVec2::new(16, 16),
                                tileset_id: None,
                                int_grid_values: vec![IntGridValueDef {
                                    value: 1,
                                    name: "Solid".to_string(),
                                    color: [220, 50, 50, 255],
                                }],
                            };
                            actions.exec(AddLayerDefCommand::new(def));
                            actions.emit(EditorEvent::SetActiveLayer(Some(next_id)));
                            actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(next_id)));
                            ui.close();
                        }

                        if ui.button("Entities Layer").clicked() {
                            let next_id = LayerDefId(
                                cx.project()
                                    .defs
                                    .layers
                                    .iter()
                                    .map(|l| l.id.0)
                                    .max()
                                    .unwrap_or(0)
                                    + 1,
                            );
                            let def = LayerDef {
                                id: next_id,
                                name: format!("Layer_{}", next_id.0),
                                kind: LayerKind::Entities,
                                grid_size: UVec2::new(16, 16),
                                tileset_id: None,
                                int_grid_values: Vec::new(),
                            };
                            actions.exec(AddLayerDefCommand::new(def));
                            actions.emit(EditorEvent::SetActiveLayer(Some(next_id)));
                            actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(next_id)));
                            ui.close();
                        }
                    });

                    ui.separator();

                    // Move Up (higher render order)
                    let can_move_up = active_layer_index.is_some_and(|idx| idx + 1 < total_layers);
                    let up_btn = ui.add_enabled(can_move_up, egui::Button::new("▲").small());
                    if up_btn
                        .on_hover_text("Move Layer Up (render over lower layers)")
                        .clicked()
                    {
                        if let Some(idx) = active_layer_index {
                            let mut new_order: Vec<LayerDefId> =
                                layers.iter().map(|l| l.def_id).collect();
                            new_order.swap(idx, idx + 1);
                            actions.exec(ReorderLayerDefsCommand::new(new_order));
                        }
                    }

                    // Move Down (lower render order)
                    let can_move_down = active_layer_index.is_some_and(|idx| idx > 0);
                    let down_btn = ui.add_enabled(can_move_down, egui::Button::new("▼").small());
                    if down_btn
                        .on_hover_text("Move Layer Down (render under higher layers)")
                        .clicked()
                    {
                        if let Some(idx) = active_layer_index {
                            let mut new_order: Vec<LayerDefId> =
                                layers.iter().map(|l| l.def_id).collect();
                            new_order.swap(idx, idx - 1);
                            actions.exec(ReorderLayerDefsCommand::new(new_order));
                        }
                    }

                    ui.separator();

                    // Delete
                    let can_delete = active_layer_id.is_some();
                    let del_btn = ui.add_enabled(can_delete, egui::Button::new("🗑").small());
                    if del_btn.on_hover_text("Delete selected layer").clicked() {
                        if let Some(id) = active_layer_id {
                            actions.exec(RemoveLayerDefCommand::new(id));
                            actions.emit(EditorEvent::SetActiveLayer(None));
                        }
                    }

                    // Edit def
                    let edit_btn =
                        ui.add_enabled(can_delete, egui::Button::new("Edit Def").small());
                    if edit_btn
                        .on_hover_text("Open Layer Definition document")
                        .clicked()
                    {
                        if let Some(id) = active_layer_id {
                            actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(id)));
                        }
                    }
                });
            });

        ui.add_space(4.0);

        // 3. Layers List
        if layers.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No layers in this level.").color(t.muted));
                ui.label(
                    RichText::new("Click '+ Add Layer' above to add one.")
                        .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(t.faint),
                );
            });
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                // Top-to-bottom visual order (highest render index at the top)
                for (idx, layer) in layers.iter().enumerate().rev() {
                    let is_selected = active_layer_id == Some(layer.def_id);
                    let def_opt = cx.project().defs.layer(layer.def_id);
                    let layer_name = def_opt.map(|d| d.name.as_str()).unwrap_or("Unnamed Layer");
                    let kind = def_opt.map(|d| d.kind).unwrap_or_else(|| layer.kind());

                    let bg_color = if is_selected {
                        t.selection_fill
                    } else {
                        Color32::TRANSPARENT
                    };

                    let stroke = if is_selected {
                        Stroke::new(1.0, t.accent)
                    } else {
                        Stroke::NONE
                    };

                    Frame::new()
                        .fill(bg_color)
                        .stroke(stroke)
                        .inner_margin(Margin::symmetric(6, 4))
                        .corner_radius(CornerRadius::ZERO)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 1. Visibility toggle
                                let vis_text = if layer.visible { "👁" } else { "—" };
                                let vis_tip = if layer.visible {
                                    "Hide layer"
                                } else {
                                    "Show layer"
                                };
                                let vis_btn = ui.small_button(vis_text).on_hover_text(vis_tip);
                                if vis_btn.clicked() {
                                    actions.exec(ToggleLayerVisibilityCommand::new(
                                        level_id,
                                        layer.def_id,
                                    ));
                                }

                                // 2. Order badge
                                ui.label(
                                    RichText::new(format!("#{idx}")).color(t.muted).size(11.0),
                                );

                                // 3. Kind badge
                                let (badge_text, badge_color) = match kind {
                                    LayerKind::Tiles => ("Tiles", t.accent),
                                    LayerKind::IntGrid => {
                                        ("IntGrid", Color32::from_rgb(100, 180, 240))
                                    }
                                    LayerKind::Entities => {
                                        ("Entities", Color32::from_rgb(240, 160, 80))
                                    }
                                    LayerKind::Auto => ("Auto", Color32::from_rgb(180, 120, 240)),
                                };
                                ui.label(
                                    RichText::new(format!("[{badge_text}]"))
                                        .color(badge_color)
                                        .size(11.0),
                                );

                                // 4. Layer name / row selection
                                let name_label = RichText::new(layer_name)
                                    .color(if is_selected { t.ink_strong } else { t.ink })
                                    .size(13.0);

                                let row_resp = ui.selectable_label(is_selected, name_label);

                                if row_resp.clicked() {
                                    actions.emit(EditorEvent::SetActiveLayer(Some(layer.def_id)));
                                }
                                if row_resp.double_clicked() {
                                    actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(
                                        layer.def_id,
                                    )));
                                }

                                row_resp.context_menu(|ui| {
                                    if ui.button("Select Layer").clicked() {
                                        actions
                                            .emit(EditorEvent::SetActiveLayer(Some(layer.def_id)));
                                        ui.close();
                                    }
                                    if ui.button("Toggle Visibility").clicked() {
                                        actions.exec(ToggleLayerVisibilityCommand::new(
                                            level_id,
                                            layer.def_id,
                                        ));
                                        ui.close();
                                    }
                                    if ui.button("Edit Definition").clicked() {
                                        actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(
                                            layer.def_id,
                                        )));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("Delete Layer").clicked() {
                                        actions.exec(RemoveLayerDefCommand::new(layer.def_id));
                                        if is_selected {
                                            actions.emit(EditorEvent::SetActiveLayer(None));
                                        }
                                        ui.close();
                                    }
                                });
                            });
                        });

                    ui.add_space(2.0);
                }
            });
        }

        // 4. Apply deferred actions
        for event in actions.events {
            cx.emit(event);
        }
        for cmd in actions.commands {
            cx.exec_boxed(cmd);
        }
    }
}
