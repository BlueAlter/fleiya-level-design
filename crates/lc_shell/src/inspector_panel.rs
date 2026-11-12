//! Dockable Inspector panel on the right hosting entity inspector and level/layer properties.
//!
//! Replaces the old floating canvas Properties window with a dedicated docked inspector.

use egui::{CornerRadius, Frame, Margin, RichText, ScrollArea, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::hud;
use lc_feature_entities::EntityInspectorSection;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::view::{PanelId, PanelView};

/// Dockable panel view for inspecting active levels, layers, and entity instances.
pub struct InspectorPanel {
    sections: Vec<Box<dyn InspectorSection>>,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectorPanel {
    /// Panel ID constant for the inspector panel.
    pub const PANEL_ID: &'static str = "inspector";

    /// Constructs a new `InspectorPanel` seeded with entity inspection capabilities.
    pub fn new() -> Self {
        Self {
            sections: vec![Box::new(EntityInspectorSection::new())],
        }
    }
}

impl PanelView for InspectorPanel {
    fn id(&self) -> PanelId {
        PanelId::new(Self::PANEL_ID)
    }

    fn title(&self) -> &str {
        "Inspector"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());
        ui.set_min_width(240.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);

            // 1. Check if any specialized inspector section applies (e.g. Entity selected)
            let sel = cx.selection().clone();
            let mut any_rendered = false;

            for section in self.sections.iter_mut() {
                if section.applies(&sel) {
                    section.ui(ui, cx, &sel);
                    any_rendered = true;
                }
            }

            // 2. If no specialized section applies, display active Level and Layer properties
            if !any_rendered {
                let active_lvl_opt = cx.active_level().and_then(|id| cx.project().level(id)).cloned();

                // Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new(Icon::Sliders.glyph()).color(t.accent_ink));
                    ui.label(
                        RichText::new("PROPERTIES")
                            .font(egui::FontId::new(11.0, lc_canvas::theme::mono_bold_for(ui.ctx())))
                            .color(t.ink_strong),
                    );
                });
                ui.add_space(4.0);

                if let Some(level) = active_lvl_opt {
                    Frame::new()
                        .fill(t.sunken)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::same(3))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                hud(ui, "LEVEL", t.faint);
                                ui.label(RichText::new(&level.name).strong().color(t.ink_strong));
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                hud(ui, "POSITION", t.faint);
                                ui.label(format!("{}, {}", level.world_pos.x, level.world_pos.y));
                            });
                            ui.horizontal(|ui| {
                                hud(ui, "SIZE", t.faint);
                                ui.label(format!("{} × {} px", level.size_px.x, level.size_px.y));
                            });
                            ui.horizontal(|ui| {
                                hud(ui, "DEPTH", t.faint);
                                ui.label(format!("{}", level.world_depth));
                            });
                            ui.horizontal(|ui| {
                                hud(ui, "LAYERS", t.faint);
                                ui.label(format!("{}", level.layers.len()));
                            });
                        });

                    // Active Layer properties if any
                    if let Some(lay_id) = cx.active_layer() {
                        if let Some(layer_def) = cx.project().defs.layer(lay_id) {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(Icon::Layer.glyph()).color(t.accent_ink));
                                ui.label(
                                    RichText::new("ACTIVE LAYER")
                                        .font(egui::FontId::new(11.0, lc_canvas::theme::mono_bold_for(ui.ctx())))
                                        .color(t.ink_strong),
                                );
                            });
                            ui.add_space(4.0);

                            Frame::new()
                                .fill(t.sunken)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::same(3))
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        hud(ui, "NAME", t.faint);
                                        ui.label(RichText::new(&layer_def.name).strong().color(t.ink_strong));
                                    });
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        hud(ui, "KIND", t.faint);
                                        ui.label(format!("{:?}", layer_def.kind));
                                    });
                                    ui.horizontal(|ui| {
                                        hud(ui, "GRID SIZE", t.faint);
                                        ui.label(format!("{} × {}", layer_def.grid_size.x, layer_def.grid_size.y));
                                    });
                                });
                        }
                    }
                } else {
                    Frame::new()
                        .fill(t.sunken)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::same(3))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(8.0);
                                ui.label(RichText::new("No active level selected.").color(t.muted));
                                ui.label(
                                    RichText::new("Double-click a level in the Project panel to inspect properties.")
                                        .size(11.0)
                                        .color(t.faint),
                                );
                                ui.add_space(8.0);
                            });
                        });
                }
            }
        });
    }
}
