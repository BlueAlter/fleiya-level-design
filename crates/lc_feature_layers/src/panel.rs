//! Dockable panel for managing layers in card view, matching the World and Asset card panel styling.

use egui::{
    Color32, CornerRadius, FontId, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui,
};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, ButtonKind};
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

/// Dockable panel view for inspecting, ordering, and selecting layers in a modern card panel layout.
#[derive(Default)]
pub struct LayersPanel {
    search_query: String,
}

impl LayersPanel {
    /// Identifier constant for the Layers dockable panel.
    pub const PANEL_ID: &'static str = "layers";

    /// Creates a new `LayersPanel`.
    pub fn new() -> Self {
        Self::default()
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
        let project = cx.project();
        let level_opt = cx.active_level().and_then(|lid| project.level(lid));
        let active_layer_id = cx.active_layer();
        let mut actions = ActionQueue::default();

        // Layer defs from project
        let all_layer_defs = project.defs.layers.clone();
        let total_count = all_layer_defs.len();

        // 1. Top Header: Icon + Title + Count + "+ Add Layer" Menu
        ui.horizontal(|ui| {
            ui.label(RichText::new(Icon::Layer.glyph()).color(t.accent_ink));
            ui.label(
                RichText::new("LAYERS")
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_bold_for(ui.ctx())))
                    .color(t.ink_strong),
            );
            ui.label(
                RichText::new(format!("{total_count}"))
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                    .color(t.muted),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button("+ LAYER", |ui| {
                    if ui.button("Tiles Layer").clicked() {
                        let next_id = LayerDefId(
                            all_layer_defs.iter().map(|l| l.id.0).max().unwrap_or(0) + 1,
                        );
                        let default_tileset = project.defs.tilesets.first().map(|ts| ts.id);
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
                            all_layer_defs.iter().map(|l| l.id.0).max().unwrap_or(0) + 1,
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
                            all_layer_defs.iter().map(|l| l.id.0).max().unwrap_or(0) + 1,
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
            });
        });

        hline(ui);

        // 2. Search input
        ui.horizontal(|ui| {
            ui.label(RichText::new(Icon::Search.glyph()).color(t.faint));
            ui.add(
                TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search layers...")
                    .desired_width(f32::INFINITY),
            );
        });

        ui.add_space(4.0);

        // Section header
        let query = self.search_query.trim().to_lowercase();
        let filtered_layers: Vec<LayerDef> = all_layer_defs
            .into_iter()
            .filter(|def| {
                if query.is_empty() {
                    true
                } else {
                    def.name.to_lowercase().contains(&query)
                }
            })
            .collect();

        ui.label(
            RichText::new(format!("LAYERS IN PROJECT ({})", filtered_layers.len()))
                .font(FontId::new(10.0, theme::mono_for(ui.ctx())))
                .color(t.faint),
        );
        ui.add_space(4.0);

        // 3. Card list
        ScrollArea::vertical()
            .max_height(ui.available_height() - 36.0)
            .show(ui, |ui| {
                if filtered_layers.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(RichText::new("No layers match the search.").color(t.muted));
                    });
                } else {
                    for (idx, def) in filtered_layers.iter().enumerate() {
                        let is_selected = active_layer_id == Some(def.id);

                        let card_frame = Frame::new()
                            .fill(if is_selected { t.sunken } else { t.panel })
                            .stroke(if is_selected {
                                Stroke::new(1.0, t.accent)
                            } else {
                                t.hairline()
                            })
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 6));

                        let card_resp = card_frame
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Visibility toggle if level active
                                    if let Some(lvl) = level_opt {
                                        let is_visible = lvl
                                            .layers
                                            .iter()
                                            .find(|l| l.def_id == def.id)
                                            .map(|l| l.visible)
                                            .unwrap_or(true);
                                        let vis_btn = ui
                                            .small_button(if is_visible { "👁" } else { "—" })
                                            .on_hover_text(if is_visible {
                                                "Hide layer"
                                            } else {
                                                "Show layer"
                                            });
                                        if vis_btn.clicked() {
                                            actions.exec(ToggleLayerVisibilityCommand::new(
                                                lvl.id, def.id,
                                            ));
                                        }
                                    }

                                    // Layer kind icon
                                    let (icon_glyph, badge_text, badge_color) = match def.kind {
                                        LayerKind::Tiles => (
                                            Icon::Tiles.glyph(),
                                            format!(
                                                "Tiles {}x{}",
                                                def.grid_size.x, def.grid_size.y
                                            ),
                                            t.accent,
                                        ),
                                        LayerKind::IntGrid => (
                                            Icon::IntGrid.glyph(),
                                            "IntGrid".to_string(),
                                            Color32::from_rgb(100, 180, 240),
                                        ),
                                        LayerKind::Entities => (
                                            Icon::Entity.glyph(),
                                            "Entities".to_string(),
                                            Color32::from_rgb(240, 160, 80),
                                        ),
                                        LayerKind::Auto => (
                                            Icon::AutoLayer.glyph(),
                                            "Auto".to_string(),
                                            Color32::from_rgb(180, 120, 240),
                                        ),
                                    };

                                    ui.label(RichText::new(icon_glyph).color(t.accent_ink));

                                    // Layer name
                                    ui.label(
                                        RichText::new(&def.name).strong().color(if is_selected {
                                            t.ink_strong
                                        } else {
                                            t.ink
                                        }),
                                    );

                                    // Right pill badge and more action
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(RichText::new("⋮").color(t.muted));

                                            Frame::new()
                                                .fill(theme::tint(badge_color, 0.12))
                                                .stroke(Stroke::new(1.0, badge_color))
                                                .corner_radius(CornerRadius::same(3))
                                                .inner_margin(Margin::symmetric(6, 2))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new(badge_text)
                                                            .size(10.5)
                                                            .color(badge_color),
                                                    );
                                                });
                                        },
                                    );
                                });
                            })
                            .response
                            .interact(egui::Sense::click());

                        if card_resp.clicked() {
                            actions.emit(EditorEvent::SetActiveLayer(Some(def.id)));
                        }
                        if card_resp.double_clicked() {
                            actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(def.id)));
                        }

                        // Context menu
                        card_resp.context_menu(|ui| {
                            if ui.button("Select Layer").clicked() {
                                actions.emit(EditorEvent::SetActiveLayer(Some(def.id)));
                                ui.close();
                            }
                            if ui.button("Edit Definition").clicked() {
                                actions.emit(EditorEvent::OpenDocument(DocKind::LayerDef(def.id)));
                                ui.close();
                            }
                            if let Some(lvl) = level_opt {
                                if ui.button("Toggle Visibility").clicked() {
                                    actions.exec(ToggleLayerVisibilityCommand::new(lvl.id, def.id));
                                    ui.close();
                                }
                                if idx > 0 && ui.button("Move Layer Up").clicked() {
                                    let mut new_order: Vec<LayerDefId> =
                                        lvl.layers.iter().map(|l| l.def_id).collect();
                                    if let Some(pos) = new_order.iter().position(|&id| id == def.id)
                                    {
                                        if pos + 1 < new_order.len() {
                                            new_order.swap(pos, pos + 1);
                                            actions.exec(ReorderLayerDefsCommand::new(new_order));
                                        }
                                    }
                                    ui.close();
                                }
                                if ui.button("Move Layer Down").clicked() {
                                    let mut new_order: Vec<LayerDefId> =
                                        lvl.layers.iter().map(|l| l.def_id).collect();
                                    if let Some(pos) = new_order.iter().position(|&id| id == def.id)
                                    {
                                        if pos > 0 {
                                            new_order.swap(pos, pos - 1);
                                            actions.exec(ReorderLayerDefsCommand::new(new_order));
                                        }
                                    }
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button("Delete Layer").clicked() {
                                actions.exec(RemoveLayerDefCommand::new(def.id));
                                if is_selected {
                                    actions.emit(EditorEvent::SetActiveLayer(None));
                                }
                                ui.close();
                            }
                        });

                        ui.add_space(4.0);
                    }
                }
            });

        // 4. Bottom action button
        ui.add_space(4.0);
        if button(ui, "+ Add Layer Def", ButtonKind::Primary, true).clicked() {
            let next_id = LayerDefId(
                project
                    .defs
                    .layers
                    .iter()
                    .map(|l| l.id.0)
                    .max()
                    .unwrap_or(0)
                    + 1,
            );
            let default_tileset = project.defs.tilesets.first().map(|ts| ts.id);
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
        }

        // 5. Apply deferred actions
        for event in actions.events {
            cx.emit(event);
        }
        for cmd in actions.commands {
            cx.exec_boxed(cmd);
        }
    }
}
