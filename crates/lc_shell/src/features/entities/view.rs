//! Primary full-window Entity Definitions Workspace (`AppMode::Entities`, Ctrl+3).

use egui::{
    Button, Color32, CornerRadius, Frame, Id, Pos2, Rect, RichText, ScrollArea, Stroke, Ui,
};
use lc_canvas::theme::{self, Tokens};
use lc_core::defs::{EntityRenderMode, TilesetRect};
use lc_core::fields::FieldValue;
use lc_core::layer::TileRef;
use lc_feature_entities::{UpdateEntityDefCommand, UpdateEntityFieldCommand};
use lc_ui_api::AssetService;

use super::attributes_card::{render_entity_attributes, render_entity_identifier_bar};
use super::fields_editor::render_entity_fields_section;
use super::i18n::{self, keys};
use super::left_panel::{create_new_entity_def, render_left_panel};
use super::preview_panel::render_right_preview_panel;
use super::state::{EntityTileSlot, EntityViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::{doc, tr};
use crate::shared::ui::def_workspace::def_panel_width;
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::render_document_page;
use crate::shared::ui::sub_tabs::sub_tabs;
use crate::shared::ui::tile_picker::{TilePickerEvent, TilesetData};
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

/// Sub-tab index for the entity definition editor.
pub const ENTITY_DEF_SUB_TAB: usize = 0;
/// Sub-tab index for the entity documentation page.
pub const DOCUMENT_SUB_TAB: usize = 1;

/// Renders the full Entities Workspace page (`AppMode::Entities`).
pub fn render_entity_view(app: &mut ShellApp, ui: &mut Ui) {
    i18n::register();
    let t = Tokens::get(ui.ctx());

    // Keep selected entity valid
    if !app.project.defs.entities.is_empty() {
        let valid = app
            .entity_view_state
            .selected_entity_id
            .is_some_and(|id| app.project.defs.entity(id).is_some());
        if !valid {
            let fallback = app
                .selected_entity_def
                .filter(|id| app.project.defs.entity(*id).is_some())
                .unwrap_or(app.project.defs.entities[0].id);
            app.entity_view_state.select_entity(fallback);
            app.selected_entity_def = Some(fallback);
        }
    } else {
        app.entity_view_state.selected_entity_id = None;
        app.selected_entity_def = None;
    }

    // Take state out temporarily so sub-panels can borrow `app` mutably.
    let mut state = std::mem::take(&mut app.entity_view_state);

    let full_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(full_rect, CornerRadius::ZERO, t.bg);

    let drag_state_id = Id::new("entity_left_panel_drag_state");
    let mut drag_state = ui.ctx().data_mut(|d| {
        d.get_temp::<DragListState>(drag_state_id)
            .unwrap_or_default()
    });

    let left_w = def_panel_width();
    let left_rect = Rect::from_min_max(
        full_rect.min,
        Pos2::new(full_rect.min.x + left_w, full_rect.max.y),
    );
    app.entity_view_state = state.clone();
    ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        render_left_panel(app, ui, &t, &mut drag_state);
    });
    state = std::mem::take(&mut app.entity_view_state);
    ui.ctx()
        .data_mut(|d| d.insert_temp(drag_state_id, drag_state));

    let selected_def = state
        .selected_entity_id
        .and_then(|id| app.project.defs.entity(id).cloned());

    let content_rect = Rect::from_min_max(
        Pos2::new(
            left_rect.max.x + WORKSPACE_PADDING,
            full_rect.min.y + WORKSPACE_PADDING,
        ),
        Pos2::new(
            full_rect.max.x - WORKSPACE_PADDING,
            (full_rect.max.y - WORKSPACE_PADDING).max(full_rect.min.y + WORKSPACE_PADDING),
        ),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        render_detail_area(app, ui, &t, &mut state, selected_def.as_ref(), content_rect);
    });

    // Tile picker modal (if open)
    render_entity_tile_picker(app, ui, &t, &mut state);

    app.entity_view_state = state;
}

fn render_detail_area(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: Option<&lc_core::defs::EntityDef>,
    content_rect: Rect,
) {
    if let Some(err) = state.error_message.clone() {
        let mut dismiss = false;
        Frame::new()
            .fill(theme::tint(t.bad, 0.15))
            .stroke(Stroke::new(1.0, t.bad))
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&err).color(t.bad).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Dismiss").clicked() {
                            dismiss = true;
                        }
                    });
                });
            });
        if dismiss {
            state.error_message = None;
        }
        ui.add_space(8.0);
    }

    let Some(def) = def else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(tr(keys::EMPTY_MESSAGE))
                        .size(16.0)
                        .color(t.ink_strong),
                );
                ui.add_space(10.0);
                let btn = Button::new(
                    RichText::new(tr(keys::EMPTY_CREATE))
                        .size(13.0)
                        .color(t.invert_fg),
                )
                .fill(t.accent)
                .corner_radius(CornerRadius::ZERO);
                if ui.add(btn).clicked() {
                    let id = create_new_entity_def(app);
                    state.select_entity(id);
                }
            });
        });
        return;
    };

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Top Sub-tabs: [ENTITY DEF | DOCUMENT]
        Frame::new().fill(Color32::TRANSPARENT).show(ui, |ui| {
            ui.set_min_height(CONTROL_HEIGHT_COMPACT);
            ui.set_max_height(CONTROL_HEIGHT_COMPACT);
            let labels = [tr(keys::TAB_DEFINITION), tr(keys::TAB_DOCUMENT)];
            sub_tabs(ui, t, &mut state.active_sub_tab, &labels);
        });

        if state.active_sub_tab == DOCUMENT_SUB_TAB {
            ui.add_space(12.0 - 15.0);
            render_document_page(
                ui,
                t,
                &state.doc_cache,
                &mut state.doc_scroll,
                "entity_document_scroll",
                doc("entity", "overview"),
            );
            return;
        }

        ui.add_space(12.0);
        ui.add(
            egui::Label::new(
                RichText::new(format!("{} (Entity)", def.name))
                    .size(20.0)
                    .strong()
                    .color(t.ink_strong),
            )
            .truncate(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(tr(keys::DESCRIPTION))
                .size(11.5)
                .color(t.muted),
        );
        ui.add_space(10.0);

        const RIGHT_PREVIEW_W: f32 = 240.0;
        const COL_GAP: f32 = WORKSPACE_PADDING;
        let split_rect = ui.available_rect_before_wrap();
        let total_w = split_rect.width();
        let app_height = content_rect.height();

        if total_w >= 760.0 {
            let center_w = (total_w - RIGHT_PREVIEW_W - COL_GAP).max(320.0);
            let center_rect = Rect::from_min_max(
                split_rect.min,
                Pos2::new(split_rect.min.x + center_w, split_rect.max.y),
            );
            let preview_rect = Rect::from_min_max(
                Pos2::new(split_rect.max.x - RIGHT_PREVIEW_W, split_rect.min.y),
                split_rect.max,
            );

            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(center_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(center_rect.intersect(ui.clip_rect()));
                    ui.set_width(center_w);
                    ui.set_max_width(center_w);
                    ScrollArea::vertical()
                        .id_salt(("entity_detail_scroll", def.id))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(center_w);
                            ui.set_max_width(center_w);
                            render_entity_identifier_bar(app, ui, t, def);
                            ui.add_space(10.0);
                            render_entity_attributes(app, ui, t, state, def);
                            ui.add_space(10.0);
                            render_entity_fields_section(app, ui, t, state, def, app_height);
                            ui.add_space(16.0);
                        });
                },
            );

            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(preview_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(preview_rect.intersect(ui.clip_rect()));
                    ui.set_width(RIGHT_PREVIEW_W);
                    ui.set_max_width(RIGHT_PREVIEW_W);
                    render_right_preview_panel(
                        app,
                        ui,
                        t,
                        state,
                        def,
                        RIGHT_PREVIEW_W,
                        preview_rect.height(),
                    );
                },
            );
        } else {
            let narrow_w = total_w.max(240.0);
            ScrollArea::vertical()
                .id_salt(("entity_detail_scroll", def.id))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(narrow_w);
                    ui.set_max_width(narrow_w);
                    render_entity_identifier_bar(app, ui, t, def);
                    ui.add_space(10.0);
                    render_right_preview_panel(app, ui, t, state, def, narrow_w, 300.0);
                    ui.add_space(10.0);
                    render_entity_attributes(app, ui, t, state, def);
                    ui.add_space(10.0);
                    render_entity_fields_section(app, ui, t, state, def, app_height);
                    ui.add_space(16.0);
                });
        }
    });
}

fn render_entity_tile_picker(
    app: &mut ShellApp,
    ui: &mut Ui,
    _t: &Tokens,
    state: &mut EntityViewState,
) {
    let Some((entity_id, slot, mut picker)) = state.tile_picker.take() else {
        return;
    };

    let Some(ts_id) = picker.tileset_id else {
        return;
    };
    let Some(ts) = app.project.defs.tileset(ts_id).cloned() else {
        return;
    };
    let ts_data = TilesetData::from(&ts);
    let tex_size = app
        .assets
        .load_texture(ui.ctx(), &ts.path)
        .ok()
        .map(|tex| tex.size_vec2());

    let mut close_picker = false;
    let current_sel = picker.selected_tile_rect;
    match picker.show(ui.ctx(), &ts_data, current_sel) {
        TilePickerEvent::Selected(tile_rect) => {
            if let Some(def) = app.project.defs.entity(entity_id).cloned() {
                let tw = ts.tile_size.x.max(1);
                let th = ts.tile_size.y.max(1);
                let px_x = ts.padding + tile_rect.col * (tw + ts.spacing);
                let px_y = ts.padding + tile_rect.row * (th + ts.spacing);
                let px_w = tile_rect.w * tw + tile_rect.w.saturating_sub(1) * ts.spacing;
                let px_h = tile_rect.h * th + tile_rect.h.saturating_sub(1) * ts.spacing;
                let rect = TilesetRect::new(ts_id, px_x, px_y, px_w, px_h);

                match slot {
                    EntityTileSlot::MainTile => {
                        let mut updated = def;
                        updated.render_mode = EntityRenderMode::Tile;
                        updated.tileset_id = Some(ts_id);
                        updated.tile_rect = Some(rect);
                        if let Some(sz) = tex_size {
                            let eff_w = (sz.x as u32).saturating_sub(ts.padding * 2);
                            let cols = (eff_w + ts.spacing)
                                .checked_div(tw + ts.spacing)
                                .unwrap_or(1)
                                .max(1);
                            let tile_id = tile_rect.row * cols + tile_rect.col;
                            updated.tile = Some(TileRef::new(ts_id, tile_id));
                        }
                        app.execute_command(Box::new(UpdateEntityDefCommand::new(updated)));
                    }
                    EntityTileSlot::UiTile => {
                        let mut updated = def;
                        updated.ui_tile_rect = Some(rect);
                        app.execute_command(Box::new(UpdateEntityDefCommand::new(updated)));
                    }
                    EntityTileSlot::FieldDefaultTile(field_idx) => {
                        if let Some(orig_field) = def.fields.get(field_idx).cloned() {
                            let mut updated_field = orig_field.clone();
                            updated_field.default_value = Some(FieldValue::Tile(Some(rect)));
                            app.execute_command(Box::new(UpdateEntityFieldCommand::new(
                                entity_id,
                                orig_field.name,
                                updated_field,
                            )));
                        }
                    }
                }
            }
            close_picker = true;
        }
        TilePickerEvent::Closed => {
            close_picker = true;
        }
        TilePickerEvent::None => {}
    }

    if !close_picker {
        state.tile_picker = Some((entity_id, slot, picker));
    }
}
