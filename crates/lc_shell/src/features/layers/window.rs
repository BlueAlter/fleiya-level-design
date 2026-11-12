//! Floating Layer Definitions window.

use egui::{DragValue, Frame, Id, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, ButtonKind};
use lc_core::defs::LayerDef;
use lc_core::id::LayerDefId;
use lc_core::layer::LayerKind;
use lc_feature_layers::{AddLayerDefCommand, RemoveLayerDefCommand, UpdateLayerDefCommand};

use crate::features::editor::shell_app::ShellApp;

/// Renders the floating Layers window if open.
pub fn render_layer_window(app: &mut ShellApp, ctx: &egui::Context) {
    if !app.show_layer_window {
        return;
    }

    let mut open = app.show_layer_window;
    let t = Tokens::get(ctx);

    egui::Window::new("Layer Definitions")
        .id(Id::new("floating_layer_definitions_window"))
        .open(&mut open)
        .default_size([500.0, 520.0])
        .min_size([380.0, 340.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            render_layer_window_contents(app, ui, &t);
        });

    app.show_layer_window = open;
}

fn render_layer_window_contents(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(Icon::Layer.glyph()).color(t.accent_ink));
        ui.label(
            RichText::new("LAYERS")
                .font(egui::FontId::new(
                    theme::HUD_SIZE,
                    theme::mono_bold_for(ui.ctx()),
                ))
                .color(t.ink_strong),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if button(ui, "+ ADD LAYER", ButtonKind::Ghost, true).clicked() {
                let next_id = LayerDefId(
                    app.project
                        .defs
                        .layers
                        .iter()
                        .map(|l| l.id.0)
                        .max()
                        .unwrap_or(0)
                        + 1,
                );
                let default_tileset = app.project.defs.tilesets.first().map(|ts| ts.id);
                let def = LayerDef {
                    id: next_id,
                    name: format!("Layer_{}", next_id.0),
                    kind: LayerKind::Tiles,
                    grid_size: UVec2::splat(16),
                    tileset_id: default_tileset,
                    int_grid_values: Vec::new(),
                };
                app.execute_command(Box::new(AddLayerDefCommand::new(def)));
                app.selected_layer_def = Some(next_id);
            }
        });
    });

    hline(ui);

    if app.project.defs.layers.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("No layer definitions in project.").color(t.muted));
        });
        return;
    }

    let layers = app.project.defs.layers.clone();

    if app.selected_layer_def.is_none()
        || !layers.iter().any(|l| Some(l.id) == app.selected_layer_def)
    {
        app.selected_layer_def = layers.first().map(|l| l.id);
    }

    ui.columns(2, |columns| {
        // Left column: Layers list
        ScrollArea::vertical().show(&mut columns[0], |ui| {
            for l in &layers {
                let is_selected = app.selected_layer_def == Some(l.id);

                let item_frame = Frame::new()
                    .fill(if is_selected { t.sunken } else { t.panel })
                    .stroke(if is_selected {
                        Stroke::new(1.0, t.accent)
                    } else {
                        t.hairline()
                    })
                    .inner_margin(Margin::symmetric(6, 4));

                item_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let icon_str = match l.kind {
                            LayerKind::Tiles => Icon::Tiles.glyph(),
                            LayerKind::IntGrid => Icon::IntGrid.glyph(),
                            LayerKind::Entities => Icon::Entity.glyph(),
                            LayerKind::Auto => Icon::AutoLayer.glyph(),
                        };
                        ui.label(RichText::new(icon_str).color(t.accent_ink));
                        let resp = ui.selectable_label(is_selected, &l.name);
                        if resp.clicked() {
                            app.selected_layer_def = Some(l.id);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        });

        // Right column: Selected Layer editor
        if let Some(selected_id) = app.selected_layer_def {
            if let Some(mut l) = layers.into_iter().find(|l| l.id == selected_id) {
                let orig = l.clone();
                let ui = &mut columns[1];

                ScrollArea::vertical().show(ui, |ui| {
                    let mut changed = false;
                    hud(ui, "LAYER NAME", t.faint);
                    let resp1 =
                        ui.add(TextEdit::singleline(&mut l.name).desired_width(f32::INFINITY));
                    if resp1.lost_focus() {
                        changed = true;
                    }
                    ui.add_space(6.0);

                    hud(ui, "GRID SIZE (PX)", t.faint);
                    ui.horizontal(|ui| {
                        let resp2 = ui.add(
                            DragValue::new(&mut l.grid_size.x)
                                .range(4..=256)
                                .prefix("W: "),
                        );
                        let resp3 = ui.add(
                            DragValue::new(&mut l.grid_size.y)
                                .range(4..=256)
                                .prefix("H: "),
                        );
                        if resp2.drag_stopped()
                            || resp2.lost_focus()
                            || resp3.drag_stopped()
                            || resp3.lost_focus()
                        {
                            changed = true;
                        }
                    });
                    ui.add_space(6.0);

                    hud(ui, "LAYER KIND", t.faint);
                    ui.label(RichText::new(format!("{:?}", l.kind)).color(t.ink_strong));
                    ui.add_space(6.0);

                    if l.kind == LayerKind::Tiles || l.kind == LayerKind::Auto {
                        hud(ui, "BOUND TILESET", t.faint);
                        let tilesets = app.project.defs.tilesets.clone();
                        egui::ComboBox::from_id_salt("layer_def_tileset_select")
                            .selected_text(
                                l.tileset_id
                                    .and_then(|id| tilesets.iter().find(|ts| ts.id == id))
                                    .map(|ts| ts.name.as_str())
                                    .unwrap_or("None"),
                            )
                            .show_ui(ui, |ui| {
                                for ts in &tilesets {
                                    if ui
                                        .selectable_value(&mut l.tileset_id, Some(ts.id), &ts.name)
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                        ui.add_space(6.0);
                    }

                    if changed && l != orig {
                        app.execute_command(Box::new(UpdateLayerDefCommand::new(l.clone())));
                    }

                    ui.add_space(12.0);
                    if button(ui, "DELETE LAYER", ButtonKind::Danger, true).clicked() {
                        app.execute_command(Box::new(RemoveLayerDefCommand::new(selected_id)));
                        app.selected_layer_def = None;
                    }
                });
            }
        }
    });
}
