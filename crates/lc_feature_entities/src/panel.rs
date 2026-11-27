//! Dockable panel listing entity definitions grouped by tags and starting placement.

use egui::{vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Sense, TextEdit, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::command::DocKind;
use lc_core::defs::EntityDef;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::{PanelId, PanelView};
use std::collections::BTreeMap;

use crate::helpers::count_entity_def_instances;

/// Dockable panel for entity definitions, grouped by LDtk tags with quick placement.
#[derive(Default)]
pub struct EntitiesPanel {
    filter: String,
}

impl EntitiesPanel {
    /// Panel ID constant for the entity definitions panel.
    pub const PANEL_ID: &'static str = "entities";

    /// Creates a new `EntitiesPanel`.
    pub fn new() -> Self {
        Self::default()
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
                    RichText::new("Open Entities Workspace (Ctrl+3) to create entities.")
                        .color(t.faint),
                );
            });
            return;
        }

        ui.add(
            TextEdit::singleline(&mut self.filter)
                .hint_text("Filter entities or #tag...")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);

        let q = self.filter.trim().to_lowercase();
        let defs: Vec<EntityDef> = cx
            .project()
            .defs
            .entities
            .iter()
            .filter(|e| {
                q.is_empty()
                    || e.name.to_lowercase().contains(&q)
                    || e.tags.iter().any(|tag| tag.to_lowercase().contains(&q))
            })
            .cloned()
            .collect();

        let mut grouped: BTreeMap<String, Vec<EntityDef>> = BTreeMap::new();
        for def in defs {
            if def.tags.is_empty() {
                grouped.entry("Untagged".to_string()).or_default().push(def);
            } else {
                for tag in &def.tags {
                    grouped.entry(tag.clone()).or_default().push(def.clone());
                }
            }
        }

        let active_tool = cx.active_tool_name().to_string();

        ScrollArea::vertical().show(ui, |ui| {
            for (tag_name, items) in grouped {
                ui.label(
                    RichText::new(format!("{} ({})", tag_name.to_uppercase(), items.len()))
                        .size(10.5)
                        .strong()
                        .color(t.accent_ink),
                );
                ui.add_space(2.0);
                for def in items {
                    let place_tool = format!("Place #{}", def.id.0);
                    let placing = active_tool == place_tool;
                    let usage = count_entity_def_instances(cx.project(), def.id);
                    let swatch_col = Color32::from_rgba_unmultiplied(
                        def.color[0],
                        def.color[1],
                        def.color[2],
                        def.color[3],
                    );

                    Frame::new()
                        .fill(if placing { t.sunken } else { t.panel })
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(Margin::symmetric(6, 4))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (swatch_rect, _) =
                                    ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                ui.painter().rect_filled(
                                    swatch_rect,
                                    CornerRadius::ZERO,
                                    swatch_col,
                                );

                                let name = ui.add(
                                    egui::Label::new(
                                        RichText::new(&def.name).color(t.ink_strong).strong(),
                                    )
                                    .sense(Sense::click()),
                                );
                                if name.clicked() {
                                    cx.emit(EditorEvent::OpenDocument(DocKind::EntityDef(def.id)));
                                }
                                ui.label(RichText::new(format!("#{}", def.id.0)).color(t.faint));
                                ui.label(RichText::new(format!("{usage}")).color(t.muted));

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
                                            cx.status(format!(
                                                "Place '{}' on the canvas",
                                                def.name
                                            ));
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(2.0);
                }
                ui.add_space(6.0);
            }
        });
    }
}
