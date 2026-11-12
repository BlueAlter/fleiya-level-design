//! Combined Tileset panel that dynamically adapts to the selected layer type.
//!
//! Replaces separate Tiles, Entities, and Tileset tabs/panels with a unified docked panel.
//! - For Tiles layers: displays the tileset palette, zoom controls, and painting tools.
//! - For Entities layers: displays the entity definitions list and placement tools.
//! - For IntGrid layers: displays the IntGrid value selector and tools.
//! - For Auto layers: displays rule information and tileset reference.

use egui::{
    Color32, CornerRadius, DragValue, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui,
};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::{button, hud, ButtonKind};
use lc_core::command::DocKind;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::layer::LayerKind;
use lc_feature_entities::EntitiesPanel;
use lc_feature_tileset::{AddTilesetDefCommand, TilePalettePanel};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::{PanelId, PanelView};

/// Combined Tileset panel adapting to active layer kind.
pub struct CombinedTilesetPanel {
    tile_palette: TilePalettePanel,
    entities_panel: EntitiesPanel,
    show_add_tileset_form: bool,
    new_tileset_name: String,
    new_tileset_path: String,
    new_tileset_w: u32,
    new_tileset_h: u32,
}

impl Default for CombinedTilesetPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl CombinedTilesetPanel {
    /// Panel ID constant for the combined tileset panel.
    pub const PANEL_ID: &'static str = "tileset";

    /// Creates a new `CombinedTilesetPanel`.
    pub fn new() -> Self {
        Self {
            tile_palette: TilePalettePanel::new(),
            entities_panel: EntitiesPanel::new(),
            show_add_tileset_form: false,
            new_tileset_name: "Tileset_1".to_string(),
            new_tileset_path: "assets/tilesets/dungeon.png".to_string(),
            new_tileset_w: 16,
            new_tileset_h: 16,
        }
    }
}

impl PanelView for CombinedTilesetPanel {
    fn id(&self) -> PanelId {
        PanelId::new(Self::PANEL_ID)
    }

    fn title(&self) -> &str {
        "Tileset"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());

        // Resolve active layer definition (cloned to avoid holding borrow on cx)
        let active_layer_def = cx
            .active_level()
            .and_then(|lid| cx.project().level(lid))
            .and_then(|_lvl| {
                cx.active_layer()
                    .and_then(|lay_id| cx.project().defs.layer(lay_id).cloned())
            });

        // Top toolbar: Mode indicator & + Add Tileset button
        Frame::new()
            .fill(t.sunken)
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref layer_def) = active_layer_def {
                        let (icon_glyph, kind_name) = match layer_def.kind {
                            LayerKind::Tiles => (Icon::Tiles.glyph(), "Tiles"),
                            LayerKind::Entities => (Icon::Entity.glyph(), "Entities"),
                            LayerKind::IntGrid => (Icon::IntGrid.glyph(), "IntGrid"),
                            LayerKind::Auto => (Icon::AutoLayer.glyph(), "AutoLayer"),
                        };
                        ui.label(RichText::new(icon_glyph).color(t.accent_ink));
                        ui.label(
                            RichText::new(format!("{kind_name}: {}", layer_def.name))
                                .strong()
                                .color(t.ink_strong),
                        );
                    } else {
                        ui.label(RichText::new(Icon::Tileset.glyph()).color(t.accent_ink));
                        ui.label(RichText::new("TILESET").strong().color(t.ink_strong));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let add_btn_text = if self.show_add_tileset_form {
                            "Cancel"
                        } else {
                            "+ Add Tileset"
                        };
                        if ui.button(add_btn_text).clicked() {
                            self.show_add_tileset_form = !self.show_add_tileset_form;
                            let next_id = cx
                                .project()
                                .defs
                                .tilesets
                                .iter()
                                .map(|ts| ts.id.0)
                                .max()
                                .unwrap_or(0)
                                + 1;
                            self.new_tileset_name = format!("Tileset_{next_id}");
                        }

                        if let Some(ts) = cx.project().defs.tilesets.first() {
                            if ui
                                .button("Open Def")
                                .on_hover_text("Open Tileset window")
                                .clicked()
                            {
                                cx.emit(EditorEvent::OpenDocument(DocKind::TilesetDef(ts.id)));
                            }
                        }
                    });
                });
            });

        // Optional inline Add Tileset Form
        if self.show_add_tileset_form {
            Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.accent))
                .inner_margin(Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    hud(ui, "NEW TILESET DEFINITION", t.accent_ink);
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.add(TextEdit::singleline(&mut self.new_tileset_name));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.add(TextEdit::singleline(&mut self.new_tileset_path));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Tile Size:");
                        ui.add(
                            DragValue::new(&mut self.new_tileset_w)
                                .range(1..=512)
                                .prefix("W: "),
                        );
                        ui.add(
                            DragValue::new(&mut self.new_tileset_h)
                                .range(1..=512)
                                .prefix("H: "),
                        );
                    });
                    ui.horizontal(|ui| {
                        if button(ui, "Create Tileset", ButtonKind::Primary, true).clicked() {
                            let next_id = TilesetDefId(
                                cx.project()
                                    .defs
                                    .tilesets
                                    .iter()
                                    .map(|ts| ts.id.0)
                                    .max()
                                    .unwrap_or(0)
                                    + 1,
                            );
                            let def = TilesetDef {
                                id: next_id,
                                name: self.new_tileset_name.trim().to_string(),
                                path: self.new_tileset_path.trim().to_string(),
                                tile_size: UVec2::new(
                                    self.new_tileset_w.max(1),
                                    self.new_tileset_h.max(1),
                                ),
                                spacing: 0,
                                padding: 0,
                            };
                            cx.exec(AddTilesetDefCommand::new(def));
                            cx.emit(EditorEvent::OpenDocument(DocKind::TilesetDef(next_id)));
                            cx.status(format!("Created tileset {}", self.new_tileset_name));
                            self.show_add_tileset_form = false;
                        }
                    });
                });
            ui.add_space(4.0);
        }

        // Branch content based on active layer type
        match active_layer_def.as_ref().map(|l| l.kind) {
            Some(LayerKind::Entities) => {
                self.entities_panel.ui(ui, cx);
            }
            Some(LayerKind::IntGrid) => {
                self.render_intgrid_mode(ui, cx, active_layer_def.as_ref().unwrap());
            }
            Some(LayerKind::Auto) => {
                self.render_auto_mode(ui, cx, active_layer_def.as_ref().unwrap());
            }
            Some(LayerKind::Tiles) | None => {
                self.tile_palette.ui(ui, cx);
            }
        }
    }
}

impl CombinedTilesetPanel {
    fn render_intgrid_mode(
        &mut self,
        ui: &mut Ui,
        cx: &mut EditorCtx,
        layer_def: &lc_core::defs::LayerDef,
    ) {
        let t = Tokens::get(ui.ctx());

        // IntGrid Tools toolbar (borderless icons)
        Frame::new()
            .fill(t.panel)
            .inner_margin(Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let active_tool = cx.active_tool_name().to_string();
                    let tools = [
                        (Icon::Brush, "Paint", "Paint IntGrid (B)"),
                        (Icon::Eraser, "Erase", "Erase IntGrid (E)"),
                        (Icon::Rectangle, "Rectangle", "Rectangle Fill (R)"),
                        (Icon::Bucket, "Fill", "Flood Fill (F)"),
                    ];
                    for (icon, tool_name, tooltip) in tools {
                        let is_active = active_tool == tool_name;
                        let icon_color = if is_active { t.accent_ink } else { t.muted };
                        let bg_color = if is_active {
                            lc_canvas::theme::tint(t.accent, 0.15)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let resp = Frame::new()
                            .fill(bg_color)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(3))
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(RichText::new(icon.glyph()).size(14.0).color(icon_color));
                            })
                            .response
                            .interact(egui::Sense::click());
                        if resp.on_hover_text(tooltip).clicked() {
                            cx.emit(EditorEvent::SetActiveTool(tool_name.to_string()));
                        }
                    }
                });
            });

        ui.add_space(4.0);

        if layer_def.int_grid_values.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    RichText::new("No IntGrid values configured for this layer.").color(t.muted),
                );
                ui.label(RichText::new("Open the Layer Window to define values.").color(t.faint));
            });
            return;
        }

        hud(ui, "SELECT VALUE TO PAINT", t.faint);
        ScrollArea::vertical().show(ui, |ui| {
            for val in &layer_def.int_grid_values {
                let dot_color = Color32::from_rgb(val.color[0], val.color[1], val.color[2]);
                let tool_name = format!("IntGrid:{}", val.value);
                let is_active = cx.active_tool_name() == tool_name;

                let item_frame = Frame::new()
                    .fill(if is_active { t.sunken } else { t.panel })
                    .stroke(if is_active {
                        Stroke::new(1.0, t.accent)
                    } else {
                        t.hairline()
                    })
                    .inner_margin(Margin::symmetric(6, 4));

                item_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(dot_rect, egui::CornerRadius::ZERO, dot_color);
                        ui.label(RichText::new(format!("#{}", val.value)).strong());
                        let resp = ui.selectable_label(is_active, &val.name);
                        if resp.clicked() {
                            cx.emit(EditorEvent::SetActiveTool(tool_name));
                        }
                    });
                });
                ui.add_space(2.0);
            }
        });
    }

    fn render_auto_mode(
        &mut self,
        ui: &mut Ui,
        cx: &mut EditorCtx,
        layer_def: &lc_core::defs::LayerDef,
    ) {
        let t = Tokens::get(ui.ctx());

        Frame::new()
            .fill(t.panel)
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                hud(ui, "AUTO-LAYER CONFIGURATION", t.accent_ink);
                ui.label(RichText::new(format!("Layer: {}", layer_def.name)).strong());
                if let Some(ts_id) = layer_def.tileset_id {
                    if let Some(ts) = cx.project().defs.tileset(ts_id) {
                        ui.label(RichText::new(format!("Tileset: {}", ts.name)).color(t.muted));
                    }
                } else {
                    ui.label(RichText::new("No tileset assigned to this auto-layer").color(t.warn));
                }
            });

        ui.add_space(4.0);
        // Also render tile palette for reference
        self.tile_palette.ui(ui, cx);
    }
}
