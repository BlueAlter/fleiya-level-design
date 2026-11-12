//! Floating definition windows for Layers, Entities, Tilesets, and Enums.
//!
//! Replaces central document tabs with movable floating windows, allowing users
//! to inspect and modify project definitions without cluttering the tab bar.

use std::collections::HashSet;

use egui::{
    Color32, CornerRadius, DragValue, Frame, Id, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui,
};
use egui_extras::{Column, TableBuilder};
use glam::{UVec2, Vec2};
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, ButtonKind};
use lc_core::defs::{EntityDef, EnumDef, EnumValueDef, LayerDef, TilesetDef};
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_core::id::{EntityDefId, EnumDefId, LayerDefId, TilesetDefId};
use lc_core::layer::LayerKind;
use lc_feature_entities::{
    AddEntityDefCommand, AddEntityFieldCommand, RemoveEntityDefCommand, RemoveEntityFieldCommand,
    UpdateEntityDefCommand,
};
use lc_feature_enums::{
    find_enum_def_references, AddEnumDefCommand, AddEnumValueCommand, EnumReferenceLocation,
    RemoveEnumDefCommand, RemoveEnumValueCommand, ReorderEnumDefsCommand, ReorderEnumValuesCommand,
    UpdateEnumDefCommand, UpdateEnumValueCommand,
};
use lc_feature_layers::{AddLayerDefCommand, RemoveLayerDefCommand, UpdateLayerDefCommand};
use lc_feature_tileset::{AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand};
use lc_ui_api::AssetService;

use crate::shell_app::ShellApp;

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

/// Renders the floating Entities window if open.
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

/// Renders the floating Tileset window if open.
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

#[derive(Clone, Debug, PartialEq)]
pub enum PendingDeleteModal {
    Enum {
        id: EnumDefId,
        name: String,
        entity_count: usize,
        field_count: usize,
    },
    EnumValue {
        enum_id: EnumDefId,
        val_id: String,
        name: String,
    },
}

/// State for the floating Enum window.
#[derive(Clone, Default)]
struct EnumWindowState {
    search_query: String,
    active_sub_tab: usize,
    display_title: String,
    selected_tileset_id: Option<TilesetDefId>,
    is_bitmask: bool,
    dragging_enum_idx: Option<usize>,
    drag_target_enum_idx: Option<usize>,
    grab_enum_offset: egui::Vec2,
    dragging_val_idx: Option<usize>,
    drag_target_val_idx: Option<usize>,
    grab_val_offset: egui::Vec2,
    tileset_dialog: crate::tileset_dialog::TilesetDialogState,
}

fn render_delete_confirmation_modal(app: &mut ShellApp, ctx: &egui::Context, t: &Tokens) {
    let Some(ref pending) = app.pending_enum_delete else {
        return;
    };

    let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    let mut confirmed = false;
    let mut cancelled = esc_pressed;

    let modal_rect = ctx.data(|d| d.get_temp::<egui::Rect>(Id::new("delete_modal_rect")));

    // Dimmed modal backdrop: clicking outside cancels
    egui::Area::new(Id::new("delete_modal_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let screen_rect = ctx.viewport_rect();
            let (bg_rect, bg_resp) =
                ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
            ui.painter()
                .rect_filled(bg_rect, CornerRadius::ZERO, Color32::from_black_alpha(150));
            if bg_resp.clicked() {
                if let Some(m_rect) = modal_rect {
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        if !m_rect.contains(pos) {
                            cancelled = true;
                        }
                    } else {
                        cancelled = true;
                    }
                } else {
                    cancelled = true;
                }
            }
        });

    if cancelled {
        app.pending_enum_delete = None;
        return;
    }

    // Centered modal dialog window
    let dialog_area_resp = egui::Area::new(Id::new("delete_modal_dialog"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.line_strong))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(20))
                .show(ui, |ui| {
                    ui.set_width(420.0);

                    ui.horizontal(|ui| {
                        ui.label(Icon::Warning.rich(ui.ctx(), 18.0, t.bad));
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Confirm Deletion")
                                .size(16.0)
                                .strong()
                                .color(t.ink_strong),
                        );
                    });

                    ui.add_space(14.0);

                    match pending {
                        PendingDeleteModal::Enum {
                            name,
                            entity_count,
                            field_count,
                            ..
                        } => {
                            ui.label(
                                RichText::new(format!(
                                    "Delete Enum '{}'? This will also remove references from entities.",
                                    name
                                ))
                                .size(13.0)
                                .color(t.ink_strong),
                            );
                            if *entity_count > 0 || *field_count > 0 {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(format!(
                                        "Currently referenced by {} Entity Def(s) & {} Field(s).",
                                        entity_count, field_count
                                    ))
                                    .size(12.0)
                                    .color(t.warn),
                                );
                            }
                        }
                        PendingDeleteModal::EnumValue { name, val_id, .. } => {
                            ui.label(
                                RichText::new(format!(
                                    "Delete Enum Value '{}' (Key: {})? This will also remove references from entities.",
                                    name, val_id
                                ))
                                .size(13.0)
                                .color(t.ink_strong),
                            );
                        }
                    }

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if button(ui, "Confirm", ButtonKind::Danger, true).clicked()
                                || enter_pressed
                            {
                                confirmed = true;
                            }
                            ui.add_space(8.0);
                            if button(ui, "Cancel", ButtonKind::Secondary, true).clicked() {
                                cancelled = true;
                            }
                        });
                    });
                })
        });

    ctx.data_mut(|d| d.insert_temp(Id::new("delete_modal_rect"), dialog_area_resp.response.rect));

    if cancelled {
        app.pending_enum_delete = None;
    } else if confirmed {
        if let Some(target) = app.pending_enum_delete.take() {
            match target {
                PendingDeleteModal::Enum { id, .. } => {
                    app.execute_command(Box::new(RemoveEnumDefCommand::cascade(id)));
                    if app.selected_enum_def == Some(id) {
                        app.selected_enum_def = None;
                    }
                }
                PendingDeleteModal::EnumValue {
                    enum_id, val_id, ..
                } => {
                    app.execute_command(Box::new(RemoveEnumValueCommand::cascade(
                        enum_id, &val_id,
                    )));
                }
            }
        }
    }
}

/// Renders the floating Enums window if open.
pub fn render_enum_window(app: &mut ShellApp, ctx: &egui::Context) {
    if app.show_enum_window {
        let mut open = app.show_enum_window;
        let t = Tokens::get(ctx);

        egui::Window::new("Enum Definitions")
            .id(Id::new("floating_enum_definitions_window"))
            .open(&mut open)
            .default_pos([80.0, 32.0])
            .fixed_size([980.0, 680.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                render_enum_window_contents(app, ui, &t);
            });

        app.show_enum_window = open;
    }

    if app.pending_enum_delete.is_some() {
        let t = Tokens::get(ctx);
        render_delete_confirmation_modal(app, ctx, &t);
    }
}

fn render_enum_window_contents(app: &mut ShellApp, ui: &mut Ui, t: &Tokens) {
    ui.set_min_size(egui::vec2(940.0, 620.0));

    let state_id = Id::new("enum_window_transient_state");
    let mut state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<EnumWindowState>(state_id).unwrap_or_default());

    let mut pending_enum_reorder: Option<Vec<EnumDefId>> = None;
    let mut pending_enum_update: Option<EnumDef> = None;
    let mut pending_value_update: Option<EnumValueDef> = None;
    let mut pending_value_reorder: Option<Vec<String>> = None;
    let mut pending_tileset_open: Option<(String, Option<TilesetDefId>)> = None;

    let enums = app.project.defs.enums.clone();

    if app.selected_enum_def.is_none() || !enums.iter().any(|e| Some(e.id) == app.selected_enum_def)
    {
        app.selected_enum_def = enums
            .iter()
            .find(|e| e.name == "ItemRarity")
            .map(|e| e.id)
            .or_else(|| enums.first().map(|e| e.id));
    }

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        // ==========================================
        // LEFT PANE: ENUM CARDS & DEFINITIONS (width 220)
        // ==========================================
        ui.allocate_ui_with_layout(
            egui::vec2(220.0, ui.available_height().max(560.0)),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(220.0);

                // 1. Top action buttons: + New Enum & Import (50% each with small gap)
                let gap = 6.0;
                let btn_w = ((ui.available_width() - gap) / 2.0).floor().max(1.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    let new_btn = ui.add_sized(
                        [btn_w, 26.0],
                        egui::Button::new(
                            RichText::new("+ New Enum")
                                .size(11.5)
                                .strong()
                                .color(t.invert_fg),
                        )
                        .fill(t.accent)
                        .corner_radius(CornerRadius::same(3)),
                    );
                    if new_btn.clicked() {
                        let next_id = EnumDefId(
                            app.project
                                .defs
                                .enums
                                .iter()
                                .map(|e| e.id.0)
                                .max()
                                .unwrap_or(0)
                                + 1,
                        );
                        let name = format!("Enum_{}", next_id.0);
                        let def = EnumDef {
                            id: next_id,
                            name,
                            tileset_id: None,
                            tags: Vec::new(),
                            values: vec![
                                EnumValueDef {
                                    id: "Common".to_string(),
                                    name: "Common Item".to_string(),
                                    color: [156, 163, 175, 255],
                                    tile: None,
                                },
                                EnumValueDef {
                                    id: "Uncommon".to_string(),
                                    name: "Uncommon Item".to_string(),
                                    color: [61, 214, 184, 255],
                                    tile: None,
                                },
                            ],
                        };
                        app.execute_command(Box::new(AddEnumDefCommand::new(def)));
                        app.selected_enum_def = Some(next_id);
                    }

                    let import_btn = ui.add_sized(
                        [btn_w, 26.0],
                        egui::Button::new(
                            RichText::new("Import")
                                .size(11.5)
                                .strong()
                                .color(t.ink),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, t.line_strong))
                        .corner_radius(CornerRadius::same(3)),
                    );
                    if import_btn.clicked() {
                        app.status_message = "Enum import triggered".to_string();
                    }
                });

                ui.add_space(6.0);

                // 2. Search input
                ui.horizontal(|ui| {
                    ui.label(RichText::new(Icon::Search.glyph()).color(t.faint));
                    ui.add(
                        TextEdit::singleline(&mut state.search_query)
                            .hint_text("Search enums...")
                            .desired_width(f32::INFINITY),
                    );
                });

                ui.add_space(8.0);

                // 3. Section header: ENUM DEFINITIONS (X)
                ui.label(
                    RichText::new(format!("ENUM DEFINITIONS ({})", enums.len()))
                        .size(11.0)
                        .strong()
                        .color(t.accent_ink),
                );

                ui.add_space(4.0);

                // 4. Cards list
                let query = state.search_query.trim().to_lowercase();
                let filtered: Vec<&EnumDef> = enums
                    .iter()
                    .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query))
                    .collect();

                let avail_h = ui.available_height().max(100.0);
                ScrollArea::vertical()
                    .id_salt("enum_cards_scroll")
                    .max_height(avail_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if filtered.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(16.0);
                                ui.label(RichText::new("No matching enums").color(t.muted));
                            });
                        } else {
                            for (idx, e) in filtered.iter().enumerate() {
                                let is_selected = app.selected_enum_def == Some(e.id);
                                let is_being_dragged = state.dragging_enum_idx == Some(idx);
                                let pill_color = match e.name.as_str() {
                                    "BiomeType" => Color32::from_rgb(255, 193, 77),
                                    "ItemRarity" => t.accent,
                                    "WeaponType" => Color32::from_rgb(61, 214, 184),
                                    _ => match e.id.0 % 3 {
                                        0 => Color32::from_rgb(255, 193, 77),
                                        1 => t.accent,
                                        _ => Color32::from_rgb(61, 214, 184),
                                    },
                                };

                                let card_fill = if is_being_dragged {
                                    t.canvas.linear_multiply(0.4)
                                } else if is_selected {
                                    t.sunken
                                } else {
                                    t.panel
                                };

                                let card_stroke = if is_being_dragged {
                                    Stroke::new(1.0, t.muted.linear_multiply(0.4))
                                } else if is_selected {
                                    Stroke::new(1.0, t.accent)
                                } else {
                                    t.hairline()
                                };

                                let card_frame = Frame::new()
                                    .fill(card_fill)
                                    .stroke(card_stroke)
                                    .corner_radius(CornerRadius::same(4))
                                    .inner_margin(Margin::symmetric(8, 6));

                                let mut trash_clicked = false;
                                let mut card_selected_by_click = false;

                                let frame_res = card_frame.show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let drag_resp = ui
                                            .add(
                                                egui::Label::new(
                                                    Icon::Drag.rich(ui.ctx(), 13.0, t.muted),
                                                )
                                                .sense(egui::Sense::drag()),
                                            )
                                            .on_hover_cursor(egui::CursorIcon::Grab)
                                            .on_hover_text("Drag to reorder");

                                        if drag_resp.drag_started() {
                                            state.dragging_enum_idx = Some(idx);
                                            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                                state.grab_enum_offset = pos - ui.max_rect().min;
                                            } else {
                                                state.grab_enum_offset = egui::vec2(10.0, 10.0);
                                            }
                                            ui.memory_mut(|m| m.surrender_focus(egui::Id::NULL));
                                        }

                                        ui.label(RichText::new("=").strong().color(pill_color));

                                        let name_resp = ui.add(
                                            egui::Label::new(
                                                RichText::new(&e.name).strong().color(if is_selected {
                                                    t.ink_strong
                                                } else {
                                                    t.ink
                                                }),
                                            )
                                            .sense(egui::Sense::click()),
                                        );
                                        if name_resp.clicked() {
                                            card_selected_by_click = true;
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let del_btn = ui
                                                    .add(
                                                        egui::Button::new(
                                                            Icon::Delete.rich(
                                                                ui.ctx(),
                                                                13.0,
                                                                t.muted,
                                                            ),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_cursor(
                                                        egui::CursorIcon::PointingHand,
                                                    )
                                                    .on_hover_text("Delete enum");

                                                if del_btn.clicked() {
                                                    trash_clicked = true;
                                                }
                                            },
                                        );
                                    });
                                });

                                let card_rect = frame_res.response.rect;

                                if trash_clicked {
                                    let refs = find_enum_def_references(
                                        &app.project,
                                        e.id,
                                    );
                                    let mut unique_ents = HashSet::new();
                                    for r in &refs {
                                        if let EnumReferenceLocation::EntityField {
                                            entity_id,
                                            ..
                                        } = r
                                        {
                                            unique_ents.insert(*entity_id);
                                        }
                                    }
                                    app.pending_enum_delete =
                                        Some(PendingDeleteModal::Enum {
                                            id: e.id,
                                            name: e.name.clone(),
                                            entity_count: unique_ents.len(),
                                            field_count: refs.len(),
                                        });
                                } else if card_selected_by_click {
                                    app.selected_enum_def = Some(e.id);
                                } else if ui.input(|i| i.pointer.primary_clicked()) {
                                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                        if card_rect.contains(pos) && state.dragging_enum_idx.is_none() {
                                            app.selected_enum_def = Some(e.id);
                                        }
                                    }
                                }

                                if state.dragging_enum_idx.is_some() {
                                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos())) {
                                        if pos.y >= card_rect.top() && pos.y <= card_rect.bottom() {
                                            state.drag_target_enum_idx = Some(idx);
                                        }
                                    }
                                }

                                if let (Some(drag_from), Some(target)) = (
                                    state.dragging_enum_idx,
                                    state.drag_target_enum_idx,
                                ) {
                                    if target == idx && drag_from != idx {
                                        let line_y = if drag_from < idx {
                                            card_rect.bottom() + 1.0
                                        } else {
                                            card_rect.top() - 1.0
                                        };
                                        ui.painter().hline(
                                            card_rect.x_range(),
                                            line_y,
                                            Stroke::new(2.0, t.accent),
                                        );
                                    }
                                }

                                ui.add_space(2.0);
                            }
                        }
                    });

                // Ghost row for dragged enum card
                if let Some(drag_idx) = state.dragging_enum_idx {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    if let Some(dragged_enum) = filtered.get(drag_idx) {
                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos())) {
                            let ghost_pos = pos - state.grab_enum_offset;
                            egui::Area::new(Id::new("enum_card_drag_ghost"))
                                .order(egui::Order::Tooltip)
                                .fixed_pos(ghost_pos)
                                .interactable(false)
                                .show(ui.ctx(), |ui| {
                                    Frame::new()
                                        .fill(t.sunken)
                                        .stroke(Stroke::new(1.5, t.accent))
                                        .shadow(egui::epaint::Shadow {
                                            offset: [0, 4],
                                            blur: 10,
                                            spread: 1,
                                            color: Color32::from_black_alpha(140),
                                        })
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(8, 6))
                                        .show(ui, |ui| {
                                            ui.set_width(200.0);
                                            ui.horizontal(|ui| {
                                                ui.label(Icon::Drag.rich(ui.ctx(), 13.0, t.accent_ink));
                                                ui.label(RichText::new(&dragged_enum.name).strong().color(t.ink_strong));
                                            });
                                        });
                                });
                        }
                    }
                }

                if ui.input(|i| i.pointer.any_released()) {
                    if let (Some(from), Some(to)) = (
                        state.dragging_enum_idx.take(),
                        state.drag_target_enum_idx.take(),
                    ) {
                        if from != to && from < filtered.len() && to < filtered.len() {
                            let from_id = filtered[from].id;
                            let to_id = filtered[to].id;
                            let mut ids: Vec<EnumDefId> =
                                enums.iter().map(|item| item.id).collect();
                            if let (Some(p_from), Some(p_to)) = (
                                ids.iter().position(|&x| x == from_id),
                                ids.iter().position(|&x| x == to_id),
                            ) {
                                let moved = ids.remove(p_from);
                                ids.insert(p_to, moved);
                                pending_enum_reorder = Some(ids);
                            }
                        }
                    }
                }
            },
        );

        ui.separator();

        // ==========================================
        // RIGHT PANE: ENUM DETAILS (matching picture-3)
        // ==========================================
        let right_rect = ui.available_rect_before_wrap();
        const FOOTER_HEIGHT: f32 = 48.0;

        let content_rect = egui::Rect::from_min_max(
            right_rect.min,
            egui::pos2(
                right_rect.max.x,
                (right_rect.max.y - FOOTER_HEIGHT).max(right_rect.min.y),
            ),
        );
        let footer_rect = egui::Rect::from_min_max(
            egui::pos2(
                right_rect.min.x,
                (right_rect.max.y - FOOTER_HEIGHT).max(right_rect.min.y),
            ),
            right_rect.max,
        );

        if let Some(selected_id) = app.selected_enum_def {
            if let Some(enum_def) = enums.iter().find(|e| e.id == selected_id).cloned() {
                let references = find_enum_def_references(&app.project, selected_id);
                let mut unique_entities = HashSet::new();
                for r in &references {
                    if let EnumReferenceLocation::EntityField { entity_id, .. } = r {
                        unique_entities.insert(*entity_id);
                    }
                }
                let entity_count = unique_entities.len();
                let field_count = references.len();

                if state.selected_tileset_id.is_none() {
                    state.selected_tileset_id =
                        app.project.defs.tilesets.first().map(|ts| ts.id);
                }

                let is_val_dragging = state.dragging_val_idx.is_some();

                // ------------------------------------------
                // MAIN CONTENT AREA (above footer)
                // ------------------------------------------
                ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    ui.set_width(ui.available_width());

                    // 1. Top Sub-tabs (ENUM DEF / DOCUMENT) + Project Enums button
                    ui.horizontal(|ui| {
                        let is_def = state.active_sub_tab == 0;
                        let btn_def = Frame::new()
                            .fill(if is_def { t.sunken } else { Color32::TRANSPARENT })
                            .stroke(if is_def {
                                Stroke::new(1.0, t.accent)
                            } else {
                                Stroke::NONE
                            })
                            .inner_margin(Margin::symmetric(10, 4))
                            .corner_radius(CornerRadius::same(2))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("ENUM DEF")
                                        .size(11.0)
                                        .strong()
                                        .color(if is_def { t.accent_ink } else { t.muted }),
                                );
                            });
                        if btn_def.response.interact(egui::Sense::click()).clicked() {
                            state.active_sub_tab = 0;
                        }

                        let is_doc = state.active_sub_tab == 1;
                        let btn_doc = Frame::new()
                            .fill(if is_doc { t.sunken } else { Color32::TRANSPARENT })
                            .stroke(if is_doc {
                                Stroke::new(1.0, t.accent)
                            } else {
                                Stroke::NONE
                            })
                            .inner_margin(Margin::symmetric(10, 4))
                            .corner_radius(CornerRadius::same(2))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("DOCUMENT")
                                        .size(11.0)
                                        .color(if is_doc { t.accent_ink } else { t.muted }),
                                );
                            });
                        if btn_doc.response.interact(egui::Sense::click()).clicked() {
                            state.active_sub_tab = 1;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = Frame::new()
                                .stroke(Stroke::new(1.0, t.accent))
                                .corner_radius(CornerRadius::same(2))
                                .inner_margin(Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("PROJECT ENUMS")
                                            .size(10.5)
                                            .strong()
                                            .color(t.accent_ink),
                                    );
                                });
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Heading
                    ui.heading(
                        RichText::new(format!("{} (Enum)", enum_def.name))
                            .size(22.0)
                            .strong()
                            .color(t.ink_strong),
                    );
                    ui.label(
                        RichText::new(
                            "Define enumeration keys, localized labels, badge colors, and optional bound tile graphics.",
                        )
                        .size(12.0)
                        .color(t.muted),
                    );

                    ui.add_space(10.0);

                    // 3. Section: ENUM IDENTIFIER & ATTRIBUTES
                    ui.label(
                        RichText::new("ENUM IDENTIFIER & ATTRIBUTES")
                            .size(11.0)
                            .strong()
                            .color(t.accent_ink),
                    );
                    ui.add_space(4.0);

                    Frame::new()
                        .fill(t.panel)
                        .stroke(t.hairline())
                        .inner_margin(Margin::same(12))
                        .corner_radius(CornerRadius::same(4))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let avail = ui.available_width();
                            let spacing = 16.0;
                            let col_w = ((avail - spacing) / 2.0).floor().max(100.0);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = spacing;

                                // Column 1: Enum Identifier
                                ui.allocate_ui_with_layout(
                                    egui::vec2(col_w, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new("Enum Identifier")
                                                .strong()
                                                .color(t.ink_strong),
                                        );
                                        let mut name_buf = enum_def.name.clone();
                                        let name_resp = ui.add_sized(
                                            [ui.available_width().max(40.0), 22.0],
                                            TextEdit::singleline(&mut name_buf)
                                                .hint_text("Identifier")
                                                .interactive(!is_val_dragging),
                                        );
                                        if name_resp.lost_focus()
                                            && name_buf.trim() != enum_def.name
                                            && !name_buf.trim().is_empty()
                                        {
                                            let mut updated = enum_def.clone();
                                            updated.name = name_buf.trim().to_string();
                                            pending_enum_update = Some(updated);
                                        }
                                    },
                                );

                                // Column 2: Display Title
                                ui.allocate_ui_with_layout(
                                    egui::vec2(col_w, 24.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new("Display Title")
                                                .strong()
                                                .color(t.ink_strong),
                                        );
                                        if state.display_title.is_empty() {
                                            state.display_title = format!("{} Tier", enum_def.name);
                                        }
                                        ui.add_sized(
                                            [ui.available_width().max(40.0), 22.0],
                                            TextEdit::singleline(&mut state.display_title)
                                                .hint_text("Display Title")
                                                .interactive(!is_val_dragging),
                                        );
                                    },
                                );
                            });

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Tileset").strong().color(t.ink_strong));
                                let tilesets = &app.project.defs.tilesets;
                                let current_tileset_name = state
                                    .selected_tileset_id
                                    .and_then(|id| tilesets.iter().find(|ts| ts.id == id))
                                    .map(|ts| ts.name.as_str())
                                    .unwrap_or("Select Tileset...");

                                egui::ComboBox::from_id_salt("enum_attr_tileset_selector")
                                    .selected_text(current_tileset_name)
                                    .show_ui(ui, |ui| {
                                        for ts in tilesets {
                                            let is_sel = state.selected_tileset_id == Some(ts.id);
                                            if ui.selectable_label(is_sel, &ts.name).clicked() {
                                                state.selected_tileset_id = Some(ts.id);
                                            }
                                        }
                                    });

                                ui.add_space(16.0);

                                ui.checkbox(
                                    &mut state.is_bitmask,
                                    RichText::new(
                                        "Export as bitflag / multiselect bitmask (allows combining multiple values)",
                                    )
                                    .color(t.muted)
                                    .size(11.5),
                                );
                            });
                        });

                    ui.add_space(10.0);

                    // 4. Section: ENUM VALUES (X DEFINED) with + Add Enum Value & Sort A-Z
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "ENUM VALUES ({} DEFINED)",
                                enum_def.values.len()
                            ))
                            .size(11.0)
                            .strong()
                            .color(t.accent_ink),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("+ Add Enum Value").clicked() {
                                let next_num = enum_def.values.len() + 1;
                                let new_id = format!("Value_{next_num}");
                                let new_name = format!("Value {next_num}");

                                let default_colors: [[u8; 4]; 6] = [
                                    [156, 163, 175, 255], // Gray #9ca3af
                                    [61, 214, 184, 255],  // Teal #3fd6b8
                                    [45, 129, 255, 255],  // Blue #2d81ff
                                    [168, 85, 247, 255],  // Purple #a855f7
                                    [255, 193, 77, 255],  // Amber #ffc14d
                                    [239, 68, 68, 255],   // Red #ef4444
                                ];
                                let color =
                                    default_colors[(next_num - 1) % default_colors.len()];

                                let new_val = EnumValueDef {
                                    id: new_id,
                                    name: new_name,
                                    color,
                                    tile: None,
                                };

                                app.execute_command(Box::new(AddEnumValueCommand::new(
                                    selected_id,
                                    new_val,
                                )));
                            }

                            if ui.button("Sort A-Z").clicked() {
                                let mut sorted_ids: Vec<String> = enum_def
                                    .values
                                    .iter()
                                    .map(|v| v.id.clone())
                                    .collect();
                                sorted_ids.sort();
                                app.execute_command(Box::new(ReorderEnumValuesCommand::new(
                                    selected_id,
                                    sorted_ids,
                                )));
                            }
                        });
                    });
                    ui.add_space(4.0);

                    // Table Card
                    Frame::new()
                        .fill(t.panel)
                        .stroke(t.hairline())
                        .inner_margin(Margin::same(8))
                        .corner_radius(CornerRadius::same(4))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let table_max_h = ui.available_height().max(140.0);
                            if is_val_dragging {
                                ui.style_mut().interaction.selectable_labels = false;
                            }

                            TableBuilder::new(ui)
                                .id_salt("enum_values_table_builder")
                                .striped(false)
                                .resizable(false)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::exact(28.0))                     // Drag handle
                                .column(Column::initial(180.0).range(120.0..=260.0)) // Key / ID
                                .column(Column::remainder().at_least(120.0).clip(true)) // Display Label
                                .column(Column::exact(120.0))                    // Badge Color
                                .column(Column::exact(120.0))                    // Bound Tile
                                .column(Column::exact(64.0))                     // Action
                                .min_scrolled_height(140.0)
                                .max_scroll_height(table_max_h)
                                .header(24.0, |mut header| {
                                    header.col(|_| {});
                                    header.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new("KEY / ID")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(t.muted),
                                            );
                                        });
                                    });
                                    header.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new("DISPLAY LABEL")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(t.muted),
                                            );
                                        });
                                    });
                                    header.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new("BADGE COLOR")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(t.muted),
                                            );
                                        });
                                    });
                                    header.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new("BOUND TILE")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(t.muted),
                                            );
                                        });
                                    });
                                    header.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new("ACTION")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(t.muted),
                                            );
                                        });
                                    });
                                })
                                .body(|mut body| {
                                    if enum_def.values.is_empty() {
                                        body.row(36.0, |mut row| {
                                            row.col(|_| {});
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new("No enum values defined yet.")
                                                        .color(t.muted)
                                                        .italics(),
                                                );
                                            });
                                            row.col(|_| {});
                                            row.col(|_| {});
                                            row.col(|_| {});
                                            row.col(|_| {});
                                        });
                                    } else {
                                        let num_values = enum_def.values.len();
                                        for idx in 0..num_values {
                                            let val = &enum_def.values[idx];
                                            let val_id = val.id.clone();
                                            let mut val_id_buf = val.id.clone();
                                            let mut val_name_buf = val.name.clone();
                                            let mut color_32 = Color32::from_rgba_unmultiplied(
                                                val.color[0],
                                                val.color[1],
                                                val.color[2],
                                                val.color[3],
                                            );

                                            body.row(30.0, |mut row| {
                                                // 0. Drag handle column (exact 28px)
                                                row.col(|ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        let drag_resp = ui
                                                            .add(
                                                                egui::Label::new(
                                                                    Icon::Drag.rich(
                                                                        ui.ctx(),
                                                                        13.0,
                                                                        t.muted,
                                                                    ),
                                                                )
                                                                .sense(egui::Sense::drag()),
                                                            )
                                                            .on_hover_cursor(
                                                                egui::CursorIcon::Grab,
                                                            )
                                                            .on_hover_text(
                                                                "Drag to reorder value",
                                                            );

                                                        if drag_resp.drag_started() {
                                                            state.dragging_val_idx = Some(idx);
                                                            if let Some(pos) =
                                                                ui.input(|i| i.pointer.interact_pos())
                                                            {
                                                                state.grab_val_offset =
                                                                    pos - ui.max_rect().min;
                                                            } else {
                                                                state.grab_val_offset =
                                                                    egui::vec2(10.0, 10.0);
                                                            }
                                                            ui.memory_mut(|m| {
                                                                m.surrender_focus(egui::Id::NULL)
                                                            });
                                                        }
                                                    });

                                                    if state.dragging_val_idx.is_some() {
                                                        if let Some(pos) =
                                                            ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()))
                                                        {
                                                            let cell = ui.max_rect();
                                                            if pos.y >= cell.top()
                                                                && pos.y <= cell.bottom()
                                                            {
                                                                state.drag_target_val_idx =
                                                                    Some(idx);
                                                            }
                                                        }
                                                    }
                                                });

                                                // 1. Key / ID column
                                                row.col(|ui| {
                                                    let key_resp = ui.add_sized(
                                                        [ui.available_width(), 22.0],
                                                        TextEdit::singleline(&mut val_id_buf)
                                                            .hint_text("Key")
                                                            .interactive(!is_val_dragging),
                                                    );
                                                    if key_resp.lost_focus()
                                                        && val_id_buf != val.id
                                                        && !val_id_buf.trim().is_empty()
                                                    {
                                                        let mut updated_val = val.clone();
                                                        updated_val.id =
                                                            val_id_buf.trim().to_string();
                                                        pending_value_update = Some(updated_val);
                                                    }
                                                });

                                                // 2. Display Label column
                                                row.col(|ui| {
                                                    let name_resp = ui.add_sized(
                                                        [ui.available_width(), 22.0],
                                                        TextEdit::singleline(&mut val_name_buf)
                                                            .hint_text("Display Label")
                                                            .interactive(!is_val_dragging),
                                                    );
                                                    if name_resp.lost_focus()
                                                        && val_name_buf != val.name
                                                    {
                                                        let mut updated_val = val.clone();
                                                        updated_val.name = val_name_buf;
                                                        pending_value_update = Some(updated_val);
                                                    }
                                                });

                                                // 3. Badge Color column
                                                row.col(|ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        if ui
                                                            .color_edit_button_srgba(
                                                                &mut color_32,
                                                            )
                                                            .changed()
                                                        {
                                                            let mut updated_val = val.clone();
                                                            updated_val.color = color_32.to_array();
                                                            pending_value_update =
                                                                Some(updated_val);
                                                        }
                                                    });
                                                });

                                                // 4. Bound Tile column
                                                row.col(|ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        if let Some(ref tile_ref) = val.tile {
                                                            let icon_resp =
                                                                crate::tileset_dialog::render_tile_icon(
                                                                    ui,
                                                                    &app.project,
                                                                    &*app.assets,
                                                                    tile_ref,
                                                                    22.0,
                                                                );
                                                            if icon_resp.clicked() {
                                                                pending_tileset_open = Some((
                                                                    val_id.clone(),
                                                                    state
                                                                        .selected_tileset_id
                                                                        .or(Some(
                                                                            tile_ref.tileset_id,
                                                                        )),
                                                                ));
                                                            }
                                                            if icon_resp.secondary_clicked() {
                                                                let mut updated_val = val.clone();
                                                                updated_val.tile = None;
                                                                pending_value_update =
                                                                    Some(updated_val);
                                                            }
                                                        } else {
                                                            if ui
                                                                .small_button("+ Tile")
                                                                .on_hover_text(
                                                                    "Assign bound tile from tileset",
                                                                )
                                                                .clicked()
                                                            {
                                                                pending_tileset_open = Some((
                                                                    val_id.clone(),
                                                                    state.selected_tileset_id,
                                                                ));
                                                            }
                                                        }
                                                    });
                                                });

                                                // 5. Action column (exact 64px)
                                                row.col(|ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        let del_btn = ui.add(
                                                            egui::Button::new(
                                                                Icon::Delete.rich(
                                                                    ui.ctx(),
                                                                    13.0,
                                                                    t.muted,
                                                                ),
                                                            )
                                                            .frame(false),
                                                        )
                                                        .on_hover_cursor(
                                                            egui::CursorIcon::PointingHand,
                                                        )
                                                        .on_hover_text("Delete value");

                                                        if del_btn.clicked() {
                                                            app.pending_enum_delete =
                                                                Some(PendingDeleteModal::EnumValue {
                                                                    enum_id: selected_id,
                                                                    val_id: val_id.clone(),
                                                                    name: val.name.clone(),
                                                                });
                                                        }
                                                    });
                                                });
                                            });
                                        }
                                    }
                                });
                        });

                    // Drop indicator for table rows while dragging
                    if let (Some(drag_from), Some(target)) =
                        (state.dragging_val_idx, state.drag_target_val_idx)
                    {
                        if drag_from != target {
                            let table_rect = ui.min_rect();
                            let y_pct = (target as f32 + 0.5) / (enum_def.values.len() as f32).max(1.0);
                            let line_y = table_rect.top() + table_rect.height() * y_pct;
                            ui.painter().hline(
                                table_rect.x_range(),
                                line_y,
                                Stroke::new(2.0, t.accent),
                            );
                        }
                    }

                    // Ghost row for table value dragging
                    if let Some(drag_idx) = state.dragging_val_idx {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        if let Some(dragged_val) = enum_def.values.get(drag_idx) {
                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos())) {
                                let ghost_pos = pos - state.grab_val_offset;
                                egui::Area::new(Id::new("val_row_drag_ghost"))
                                    .order(egui::Order::Tooltip)
                                    .fixed_pos(ghost_pos)
                                    .interactable(false)
                                    .show(ui.ctx(), |ui| {
                                        Frame::new()
                                            .fill(t.panel)
                                            .stroke(Stroke::new(1.5, t.accent))
                                            .shadow(egui::epaint::Shadow {
                                                offset: [0, 4],
                                                blur: 10,
                                                spread: 1,
                                                color: Color32::from_black_alpha(140),
                                            })
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::symmetric(12, 6))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(Icon::Drag.rich(ui.ctx(), 13.0, t.accent_ink));
                                                    ui.label(RichText::new(&dragged_val.id).strong().color(t.ink_strong));
                                                    ui.label(RichText::new(format!("({})", dragged_val.name)).color(t.muted));
                                                });
                                            });
                                    });
                            }
                        }
                    }

                    if ui.input(|i| i.pointer.any_released()) {
                        if let (Some(from), Some(to)) = (
                            state.dragging_val_idx.take(),
                            state.drag_target_val_idx.take(),
                        ) {
                            if from != to
                                && from < enum_def.values.len()
                                && to < enum_def.values.len()
                            {
                                let mut new_order: Vec<String> =
                                    enum_def.values.iter().map(|v| v.id.clone()).collect();
                                let moved = new_order.remove(from);
                                new_order.insert(to, moved);
                                pending_value_reorder = Some(new_order);
                            }
                        }
                    }
                });

                // ------------------------------------------
                // FOOTER BAR (fixed at bottom of right panel)
                // ------------------------------------------
                ui.painter().rect_filled(footer_rect, CornerRadius::ZERO, t.panel);
                ui.painter().hline(footer_rect.x_range(), footer_rect.top(), t.hairline());

                ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "ID: EnumDefId({}) • Used by {} Entity Def(s) & {} Field(s)",
                                selected_id.0, entity_count, field_count
                            ))
                            .size(11.5)
                            .color(t.muted),
                        );

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.add_space(8.0);
                                let save_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("Save Enum Def")
                                            .size(12.0)
                                            .strong()
                                            .color(t.invert_fg),
                                    )
                                    .fill(t.accent)
                                    .corner_radius(CornerRadius::same(3))
                                    .min_size(egui::vec2(100.0, 26.0)),
                                );
                                if save_btn.clicked() {
                                    app.status_message =
                                        format!("Saved enum '{}'", enum_def.name);
                                }

                                ui.add_space(6.0);

                                let revert_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("Revert")
                                            .size(12.0)
                                            .color(t.ink),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, t.line_strong))
                                    .corner_radius(CornerRadius::same(3))
                                    .min_size(egui::vec2(60.0, 26.0)),
                                );
                                if revert_btn.clicked() {
                                    app.status_message = format!(
                                        "Reverted changes for '{}'",
                                        enum_def.name
                                    );
                                }
                            },
                        );
                    });
                });
            }
        } else {
            ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("No enum definition selected.").color(t.muted));
                });
            });
        }

        ui.allocate_rect(right_rect, egui::Sense::hover());
    });

    // Execute state mutations after render loop finishes
    if let Some(new_order) = pending_enum_reorder {
        app.execute_command(Box::new(ReorderEnumDefsCommand::new(new_order)));
    }
    if let Some(updated_enum) = pending_enum_update {
        app.execute_command(Box::new(UpdateEnumDefCommand::new(updated_enum)));
    }
    if let Some(updated_val) = pending_value_update {
        if let Some(selected_id) = app.selected_enum_def {
            app.execute_command(Box::new(UpdateEnumValueCommand::new(
                selected_id,
                updated_val,
            )));
        }
    }
    if let Some(new_order) = pending_value_reorder {
        if let Some(selected_id) = app.selected_enum_def {
            app.execute_command(Box::new(ReorderEnumValuesCommand::new(
                selected_id,
                new_order,
            )));
        }
    }
    if let Some((target_val, ts_id)) = pending_tileset_open {
        state.tileset_dialog.open(target_val, ts_id, None);
    }

    if let Some((target_val_id, tile_ref)) = crate::tileset_dialog::show_tileset_dialog(
        ui.ctx(),
        &mut state.tileset_dialog,
        &app.project,
        &*app.assets,
    ) {
        if let Some(selected_id) = app.selected_enum_def {
            if let Some(mut val) = app
                .project
                .defs
                .enums
                .iter()
                .find(|e| e.id == selected_id)
                .and_then(|e| e.values.iter().find(|v| v.id == target_val_id))
                .cloned()
            {
                val.tile = tile_ref;
                app.execute_command(Box::new(UpdateEnumValueCommand::new(selected_id, val)));
            }
        }
    }

    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
}
