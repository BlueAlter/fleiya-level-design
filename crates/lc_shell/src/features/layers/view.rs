//! Primary full-window Layer Definitions Workspace (`AppMode::Layers`, Ctrl+2).

use egui::{
    Button, Color32, CornerRadius, Frame, Id, Pos2, Rect, RichText, ScrollArea, Stroke, Ui,
};
use lc_canvas::theme::Tokens;
use lc_core::defs::LayerDef;
use lc_core::layer::LayerKind;

use super::attributes_card::render_general_layer_settings;
use super::auto_tile_tab::render_auto_tile_tab;
use super::i18n::{self, keys};
use super::int_grid_editor::render_intgrid_settings_section;
use super::left_panel::{layer_kind_label, render_left_panel};
use super::preview::render_layer_preview_panel;
use super::state::LayerViewState;
use super::type_settings_card::render_type_specific_section;
use super::visuals_card::render_display_options_section;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::{doc, tr};
use crate::shared::ui::def_workspace::def_panel_width;
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::render_document_page;
use crate::shared::ui::sub_tabs::sub_tabs;
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

/// Sub-tab index for the layer definition editor (`LAYERS DEF`).
pub const LAYER_DEF_SUB_TAB: usize = 0;
/// Sub-tab index for the auto-layer rules editor (`AUTO TILE`).
pub const AUTO_TILE_SUB_TAB: usize = 1;
/// Sub-tab index for the layer documentation page (`DOCUMENT`).
pub const DOCUMENT_SUB_TAB: usize = 2;

/// Renders the full Layers Workspace page (`AppMode::Layers`).
pub fn render_layer_view(app: &mut ShellApp, ui: &mut Ui) {
    i18n::register();
    let t = Tokens::get(ui.ctx());

    // Keep selected layer valid
    if !app.project.defs.layers.is_empty() {
        let valid = app
            .layer_view_state
            .selected_layer_id
            .is_some_and(|id| app.project.defs.layer(id).is_some());
        if !valid {
            let fallback = app
                .selected_layer_def
                .filter(|id| app.project.defs.layer(*id).is_some())
                .unwrap_or(app.project.defs.layers[0].id);
            app.layer_view_state.select_layer(fallback);
            app.selected_layer_def = Some(fallback);
        }
    } else {
        app.layer_view_state.selected_layer_id = None;
        app.selected_layer_def = None;
    }

    // Take state out temporarily so sub-panels can borrow `app` mutably.
    let mut state = std::mem::take(&mut app.layer_view_state);

    let full_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(full_rect, CornerRadius::ZERO, t.bg);

    let drag_state_id = Id::new("layer_left_panel_drag_state");
    let mut drag_state = ui.ctx().data_mut(|d| {
        d.get_temp::<DragListState>(drag_state_id)
            .unwrap_or_default()
    });

    let left_w = def_panel_width();
    let left_rect = Rect::from_min_max(
        full_rect.min,
        Pos2::new(full_rect.min.x + left_w, full_rect.max.y),
    );
    app.layer_view_state = state.clone();
    ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        render_left_panel(app, ui, &t, &mut drag_state);
    });
    state = std::mem::take(&mut app.layer_view_state);
    ui.ctx()
        .data_mut(|d| d.insert_temp(drag_state_id, drag_state));

    let selected_def = state
        .selected_layer_id
        .and_then(|id| app.project.defs.layer(id).cloned());

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
        render_detail_area(app, ui, &t, &mut state, selected_def.as_ref());
    });

    app.layer_view_state = state;
}

fn render_detail_area(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: Option<&LayerDef>,
) {
    if let Some(err) = state.error_message.clone() {
        let mut dismiss = false;
        Frame::new()
            .fill(t.bad.gamma_multiply(0.15))
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
                    state.show_new_layer_modal = true;
                }
            });
        });
        return;
    };

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Top Sub-tabs: [LAYERS DEF | AUTO TILE | DOCUMENT] (always interactive)
        Frame::new().fill(Color32::TRANSPARENT).show(ui, |ui| {
            ui.set_min_height(CONTROL_HEIGHT_COMPACT);
            ui.set_max_height(CONTROL_HEIGHT_COMPACT);
            let labels = [
                tr(keys::TAB_DEFINITION),
                tr(keys::TAB_AUTO_TILE),
                tr(keys::TAB_DOCUMENT),
            ];
            sub_tabs(ui, t, &mut state.active_sub_tab, &labels);
        });

        if state.active_sub_tab == DOCUMENT_SUB_TAB {
            ui.add_space(12.0 - 15.0);
            render_document_page(
                ui,
                t,
                &state.doc_cache,
                &mut state.doc_scroll,
                "layer_document_scroll",
                doc("layer", "overview"),
            );
            return;
        }

        ui.add_space(12.0);
        ui.add(
            egui::Label::new(
                RichText::new(format!("{} ({})", def.name, layer_kind_label(def.kind)))
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

        if state.active_sub_tab == AUTO_TILE_SUB_TAB {
            render_auto_tile_tab(app, ui, t, state, def);
            return;
        }

        const RIGHT_PREVIEW_W: f32 = 240.0;
        const COL_GAP: f32 = 12.0;
        let total_w = ui.available_width();
        let avail_h = ui.available_height().max(240.0);

        if total_w >= 760.0 {
            let center_w = (total_w - RIGHT_PREVIEW_W - COL_GAP).max(400.0);
            let row_top_rect = ui.available_rect_before_wrap();
            let center_rect = Rect::from_min_max(
                row_top_rect.min,
                Pos2::new(row_top_rect.min.x + center_w, row_top_rect.max.y),
            );
            let right_rect = Rect::from_min_max(
                Pos2::new(center_rect.max.x + COL_GAP, row_top_rect.min.y),
                Pos2::new(
                    (center_rect.max.x + COL_GAP + RIGHT_PREVIEW_W).min(row_top_rect.max.x),
                    row_top_rect.max.y,
                ),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(center_rect), |ui| {
                ui.set_width(center_w);
                ui.set_max_width(center_w);
                ScrollArea::vertical()
                    .id_salt(("layer_detail_scroll", def.id))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        render_general_layer_settings(app, ui, t, state, def);
                        ui.add_space(10.0);
                        render_display_options_section(app, ui, t, state, def);
                        ui.add_space(10.0);
                        if def.kind == LayerKind::IntGrid {
                            render_intgrid_settings_section(app, ui, t, state, def);
                        } else {
                            render_type_specific_section(app, ui, t, state, def);
                        }
                        ui.add_space(16.0);
                    });
            });
            ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
                ui.set_width(right_rect.width());
                ui.set_max_width(right_rect.width());
                render_layer_preview_panel(app, ui, t, def);
            });
            let _ = avail_h;
        } else {
            ScrollArea::vertical()
                .id_salt(("layer_detail_scroll", def.id))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    render_layer_preview_panel(app, ui, t, def);
                    ui.add_space(10.0);
                    render_general_layer_settings(app, ui, t, state, def);
                    ui.add_space(10.0);
                    render_display_options_section(app, ui, t, state, def);
                    ui.add_space(10.0);
                    if def.kind == LayerKind::IntGrid {
                        render_intgrid_settings_section(app, ui, t, state, def);
                    } else {
                        render_type_specific_section(app, ui, t, state, def);
                    }
                    ui.add_space(16.0);
                });
        }
    });
}
