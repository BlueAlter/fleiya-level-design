//! Dockable panel listing entity definitions and starting placement.

use egui::{Frame, Margin, RichText, ScrollArea, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::command::DocKind;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::{PanelId, PanelView};

use crate::helpers::count_entity_def_instances;

/// Dockable panel for entity definitions.
///
/// `Place` switches the canvas tool to `Place #<uid>`; the entity tool reads
/// that name and places the matching definition on the active Entities layer.
pub struct EntitiesPanel;

impl EntitiesPanel {
    /// Panel ID constant for the entity definitions panel.
    pub const PANEL_ID: &'static str = "entities";

    /// Creates a new `EntitiesPanel`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EntitiesPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelView for EntitiesPanel {
    fn id(&self) -> PanelId {
        PanelId::new(Self::PANEL_ID)
    }

    fn title(&self) -> &str {
        "Entities"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());

        Frame::new()
            .fill(t.sunken)
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(Icon::Entity.glyph()).color(t.muted));
                    ui.label(
                        RichText::new("ENTITY DEFINITIONS")
                            .strong()
                            .color(t.ink_strong),
                    );
                });
            });
        ui.add_space(4.0);

        if cx.project().defs.entities.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No entity definitions.").color(t.muted));
                ui.label(
                    RichText::new("Create one in the Project panel to start placing entities.")
                        .color(t.faint),
                );
            });
            return;
        }

        let defs = cx.project().defs.entities.clone();
        let active_tool = cx.active_tool_name().to_string();

        ScrollArea::vertical().show(ui, |ui| {
            for def in &defs {
                let place_tool = format!("Place #{}", def.id.0);
                let placing = active_tool == place_tool;
                let usage = count_entity_def_instances(cx.project(), def.id);

                Frame::new()
                    .fill(if placing { t.sunken } else { t.panel })
                    .stroke(t.hairline())
                    .inner_margin(Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Icon::Entity.glyph()).color(t.accent_ink));
                            let name = ui.add(
                                egui::Label::new(
                                    RichText::new(&def.name).color(t.ink_strong).strong(),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if name.clicked() {
                                cx.emit(EditorEvent::OpenDocument(DocKind::EntityDef(def.id)));
                            }
                            ui.label(RichText::new(format!("#{}", def.id.0)).color(t.faint));
                            ui.label(RichText::new(format!("{usage} placed")).color(t.muted));

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(if placing { "Placing" } else { "Place" })
                                        .on_hover_text(format!(
                                            "Place {} on the active Entities layer",
                                            def.name
                                        ))
                                        .clicked()
                                    {
                                        cx.emit(EditorEvent::SetActiveTool(place_tool.clone()));
                                        cx.status(format!("Place '{}' on the canvas", def.name));
                                    }
                                },
                            );
                        });
                    });

                ui.add_space(2.0);
            }
        });
    }
}
