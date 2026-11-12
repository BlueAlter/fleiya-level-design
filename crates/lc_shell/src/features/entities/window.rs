//! Floating Entity Definitions window.

use egui::{Color32, DragValue, Frame, Id, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};
use glam::{UVec2, Vec2};
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, ButtonKind};
use lc_core::defs::EntityDef;
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_core::id::EntityDefId;
use lc_feature_entities::{
    AddEntityDefCommand, AddEntityFieldCommand, RemoveEntityDefCommand, RemoveEntityFieldCommand,
    UpdateEntityDefCommand,
};

use crate::features::editor::shell_app::ShellApp;

pub fn render_entities_window(app: &mut ShellApp, ctx: &egui::Context) {
    if !app.show_entities_window {
        return;
    }

    let mut open = app.show_entities_window;
    let t = Tokens::get(ctx);

    egui::Window::new("Entity Definitions")
        .id(Id::new("floating_entity_definitions_window"))
        .open(&mut open)
        .default_size([540.0, 560.0])
        .min_size([400.0, 360.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            render_entities_window_contents(app, ui, &t);
        });

    app.show_entities_window = open;
}

fn render_entities_window_contents(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(Icon::Entity.glyph()).color(t.accent_ink));
        ui.label(
            RichText::new("ENTITIES")
                .font(egui::FontId::new(
                    theme::HUD_SIZE,
                    theme::mono_bold_for(ui.ctx()),
                ))
                .color(t.ink_strong),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if button(ui, "+ ADD ENTITY", ButtonKind::Ghost, true).clicked() {
                let next_id = EntityDefId(
                    app.project
                        .defs
                        .entities
                        .iter()
                        .map(|e| e.id.0)
                        .max()
                        .unwrap_or(0)
                        + 1,
                );
                let def = EntityDef {
                    id: next_id,
                    name: format!("Entity_{}", next_id.0),
                    size_px: UVec2::new(16, 16),
                    color: [240, 160, 60, 255],
                    pivot: Vec2::new(0.5, 0.5),
                    tile: None,
                    fields: Vec::new(),
                };
                app.execute_command(Box::new(AddEntityDefCommand::new(def)));
                app.selected_entity_def = Some(next_id);
            }
        });
    });

    hline(ui);

    if app.project.defs.entities.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("No entity definitions in project.").color(t.muted));
        });
        return;
    }

    let entities = app.project.defs.entities.clone();

    if app.selected_entity_def.is_none()
        || !entities
            .iter()
            .any(|e| Some(e.id) == app.selected_entity_def)
    {
        app.selected_entity_def = entities.first().map(|e| e.id);
    }

    ui.columns(2, |columns| {
        // Left column: Entities list
        ScrollArea::vertical().show(&mut columns[0], |ui| {
            for e in &entities {
                let is_selected = app.selected_entity_def == Some(e.id);

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
                        let color = Color32::from_rgb(e.color[0], e.color[1], e.color[2]);
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(dot_rect, egui::CornerRadius::ZERO, color);

                        let resp = ui.selectable_label(is_selected, &e.name);
                        if resp.clicked() {
                            app.selected_entity_def = Some(e.id);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        });

        // Right column: Selected Entity editor
        if let Some(selected_id) = app.selected_entity_def {
            if let Some(mut e) = entities.into_iter().find(|e| e.id == selected_id) {
                let orig = e.clone();
                let ui = &mut columns[1];

                ScrollArea::vertical().show(ui, |ui| {
                    let mut changed = false;
                    hud(ui, "ENTITY NAME", t.faint);
                    let resp1 =
                        ui.add(TextEdit::singleline(&mut e.name).desired_width(f32::INFINITY));
                    if resp1.lost_focus() {
                        changed = true;
                    }
                    ui.add_space(6.0);

                    hud(ui, "DEFAULT SIZE (PX)", t.faint);
                    ui.horizontal(|ui| {
                        let resp2 = ui.add(
                            DragValue::new(&mut e.size_px.x)
                                .range(4..=512)
                                .prefix("W: "),
                        );
                        let resp3 = ui.add(
                            DragValue::new(&mut e.size_px.y)
                                .range(4..=512)
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

                    hud(ui, "COLOR", t.faint);
                    let mut col = Color32::from_rgb(e.color[0], e.color[1], e.color[2]);
                    if ui.color_edit_button_srgba(&mut col).changed() {
                        e.color = [col.r(), col.g(), col.b(), 255];
                        changed = true;
                    }
                    ui.add_space(10.0);

                    // Field definitions
                    hud(ui, "FIELDS", t.faint);
                    let mut field_to_delete = None;
                    for (f_idx, field) in e.fields.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&field.name).strong());
                            ui.label(format!("({:?})", field.field_type));
                            if ui.button("x").clicked() {
                                field_to_delete = Some(f_idx);
                            }
                        });
                    }

                    if let Some(del_idx) = field_to_delete {
                        let field_name = e.fields[del_idx].name.clone();
                        app.execute_command(Box::new(RemoveEntityFieldCommand::new(
                            selected_id,
                            field_name,
                        )));
                    }

                    if ui.button("+ Add Field").clicked() {
                        let next_num = e.fields.len() + 1;
                        let field_def = FieldDef {
                            name: format!("field_{next_num}"),
                            field_type: FieldType::String,
                            default_value: Some(FieldValue::String("".to_string())),
                        };
                        app.execute_command(Box::new(AddEntityFieldCommand::new(
                            selected_id,
                            field_def,
                        )));
                    }

                    if changed && e != orig {
                        app.execute_command(Box::new(UpdateEntityDefCommand::new(e.clone())));
                    }

                    ui.add_space(12.0);
                    if button(ui, "DELETE ENTITY", ButtonKind::Danger, true).clicked() {
                        app.execute_command(Box::new(RemoveEntityDefCommand::cascade(selected_id)));
                        app.selected_entity_def = None;
                    }
                });
            }
        }
    });
}
