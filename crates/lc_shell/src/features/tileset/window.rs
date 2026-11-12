//! Floating Tileset Definitions window.

use egui::{Color32, DragValue, Frame, Id, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, ButtonKind};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_feature_tileset::{AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand};
use lc_ui_api::AssetService;

use crate::features::editor::shell_app::ShellApp;

pub fn render_tileset_window(app: &mut ShellApp, ctx: &egui::Context) {
    if !app.show_tileset_window {
        return;
    }

    let mut open = app.show_tileset_window;
    let t = Tokens::get(ctx);

    egui::Window::new("Tileset Definitions")
        .id(Id::new("floating_tileset_definitions_window"))
        .open(&mut open)
        .default_size([580.0, 560.0])
        .min_size([420.0, 360.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            render_tileset_window_contents(app, ui, &t);
        });

    app.show_tileset_window = open;
}

fn render_tileset_window_contents(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(Icon::Tileset.glyph()).color(t.accent_ink));
        ui.label(
            RichText::new("TILESETS")
                .font(egui::FontId::new(
                    theme::HUD_SIZE,
                    theme::mono_bold_for(ui.ctx()),
                ))
                .color(t.ink_strong),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if button(ui, "+ ADD TILESET", ButtonKind::Ghost, true).clicked() {
                let next_id = TilesetDefId(
                    app.project
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
                    name: format!("Tileset_{}", next_id.0),
                    path: "assets/tiles/dungeon.png".to_string(),
                    tile_size: UVec2::splat(16),
                    spacing: 0,
                    padding: 0,
                };
                app.execute_command(Box::new(AddTilesetDefCommand::new(def)));
                app.selected_tileset_def = Some(next_id);
            }
        });
    });

    hline(ui);

    if app.project.defs.tilesets.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("No tileset definitions in project.").color(t.muted));
        });
        return;
    }

    let tilesets = app.project.defs.tilesets.clone();

    if app.selected_tileset_def.is_none()
        || !tilesets
            .iter()
            .any(|ts| Some(ts.id) == app.selected_tileset_def)
    {
        app.selected_tileset_def = tilesets.first().map(|ts| ts.id);
    }

    ui.columns(2, |columns| {
        // Left column: Tilesets list
        ScrollArea::vertical().show(&mut columns[0], |ui| {
            for ts in &tilesets {
                let is_selected = app.selected_tileset_def == Some(ts.id);

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
                        ui.label(RichText::new(Icon::Tileset.glyph()).color(t.accent_ink));
                        let resp = ui.selectable_label(is_selected, &ts.name);
                        if resp.clicked() {
                            app.selected_tileset_def = Some(ts.id);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        });

        // Right column: Selected Tileset editor and preview
        if let Some(selected_id) = app.selected_tileset_def {
            if let Some(mut def) = tilesets.into_iter().find(|ts| ts.id == selected_id) {
                let orig = def.clone();
                let ui = &mut columns[1];

                ScrollArea::vertical().show(ui, |ui| {
                    let mut changed = false;
                    hud(ui, "TILESET NAME", t.faint);
                    let resp1 =
                        ui.add(TextEdit::singleline(&mut def.name).desired_width(f32::INFINITY));
                    if resp1.lost_focus() {
                        changed = true;
                    }
                    ui.add_space(6.0);

                    hud(ui, "IMAGE PATH (RELATIVE TO PROJECT ROOT)", t.faint);
                    let resp2 =
                        ui.add(TextEdit::singleline(&mut def.path).desired_width(f32::INFINITY));
                    if resp2.lost_focus() {
                        changed = true;
                    }
                    ui.add_space(6.0);

                    hud(ui, "TILE SIZE (PX)", t.faint);
                    ui.horizontal(|ui| {
                        let resp3 = ui.add(
                            DragValue::new(&mut def.tile_size.x)
                                .range(1..=512)
                                .prefix("W: "),
                        );
                        let resp4 = ui.add(
                            DragValue::new(&mut def.tile_size.y)
                                .range(1..=512)
                                .prefix("H: "),
                        );
                        if resp3.drag_stopped()
                            || resp3.lost_focus()
                            || resp4.drag_stopped()
                            || resp4.lost_focus()
                        {
                            changed = true;
                        }
                    });
                    ui.add_space(6.0);

                    hud(ui, "SPACING & PADDING (PX)", t.faint);
                    ui.horizontal(|ui| {
                        let resp5 = ui.add(
                            DragValue::new(&mut def.spacing)
                                .range(0..=64)
                                .prefix("Spacing: "),
                        );
                        let resp6 = ui.add(
                            DragValue::new(&mut def.padding)
                                .range(0..=64)
                                .prefix("Padding: "),
                        );
                        if resp5.drag_stopped()
                            || resp5.lost_focus()
                            || resp6.drag_stopped()
                            || resp6.lost_focus()
                        {
                            changed = true;
                        }
                    });
                    ui.add_space(8.0);

                    // Atlas Preview Texture
                    hud(ui, "PREVIEW", t.faint);
                    match app.assets.load_texture(ui.ctx(), &def.path) {
                        Ok(texture) => {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(160.0, 160.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().image(
                                texture.id(),
                                rect,
                                egui::Rect::from_min_max(
                                    egui::Pos2::ZERO,
                                    egui::Pos2::new(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                        }
                        Err(err) => {
                            ui.label(
                                RichText::new(format!("Image not loaded: {err}")).color(t.warn),
                            );
                        }
                    }

                    if changed && def != orig {
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(def.clone())));
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if button(ui, "DELETE TILESET", ButtonKind::Danger, true).clicked() {
                            app.execute_command(Box::new(RemoveTilesetDefCommand::new(
                                selected_id,
                            )));
                            app.selected_tileset_def = None;
                        }
                    });
                });
            }
        }
    });
}
