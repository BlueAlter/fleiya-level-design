//! Tileset Definitions full-window mode view and detail editor.

use std::path::Path;
use std::sync::Arc;

use egui::{vec2, Button, CornerRadius, Id, RichText, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_feature_tileset::{
    generate_unique_tileset_name, tileset_usage_counts, AddTilesetDefCommand,
    UpdateTilesetDefCommand,
};
use lc_ui_api::AssetService;

use super::atlas_view::render_atlas_view;
use super::attributes_card::render_attributes_card;
use super::file_dialog::{TilesetDialogManager, TilesetDialogMessage};
use super::i18n::{self, keys};
use super::left_panel::render_left_panel;
use super::right_panel::render_right_panel;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::{doc, tr};
use crate::shared::ui::def_workspace::{def_footer, def_panel_width};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::render_document_page;
use crate::shared::ui::sub_tabs::sub_tabs;

/// Sub-tab index for the tileset definition editor.
pub const TILESET_DEF_SUB_TAB: usize = 0;
/// Sub-tab index for the documentation page.
pub const DOCUMENT_SUB_TAB: usize = 1;

/// Chrome sections shown in the detail panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailSections {
    /// Whether to draw the title row above content.
    pub title: bool,
    /// Whether to draw the pinned footer bar at bottom.
    pub footer: bool,
}

/// Which chrome sections `tab` shows.
pub const fn sections_for(tab: usize) -> DetailSections {
    match tab {
        DOCUMENT_SUB_TAB => DetailSections {
            title: false,
            footer: false,
        },
        _ => DetailSections {
            title: true,
            footer: true,
        },
    }
}

/// Renders the complete Tileset workspace page.
pub fn render_tileset_view(app: &mut ShellApp, ui: &mut Ui) {
    i18n::register();
    let t = Tokens::get(ui.ctx());

    // Persistent dialog manager
    let dialog_mgr_id = Id::new("tileset_file_dialog_manager");
    let dialog_mgr = ui.ctx().data_mut(|d| {
        d.get_temp::<Arc<TilesetDialogManager>>(dialog_mgr_id)
            .unwrap_or_else(|| {
                let m = Arc::new(TilesetDialogManager::new());
                d.insert_temp(dialog_mgr_id, m.clone());
                m
            })
    });

    // Poll completed dialog messages
    while let Some(msg) = dialog_mgr.poll() {
        match msg {
            TilesetDialogMessage::NewTileset(path) => {
                let project_root = app
                    .project_path
                    .as_deref()
                    .and_then(|p| p.parent())
                    .unwrap_or(Path::new("."));
                let canonical_root = project_root
                    .canonicalize()
                    .unwrap_or_else(|_| project_root.to_path_buf());
                let canonical_file = path.canonicalize().unwrap_or_else(|_| path.clone());

                if let Ok(rel) = canonical_file.strip_prefix(&canonical_root) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("tileset");
                    let existing: Vec<String> = app
                        .project
                        .defs
                        .tilesets
                        .iter()
                        .map(|t| t.name.clone())
                        .collect();
                    let unique_name = generate_unique_tileset_name(&existing, stem);
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
                        name: unique_name,
                        path: rel_str,
                        tile_size: glam::UVec2::splat(16),
                        spacing: 0,
                        padding: 0,
                        tags: Vec::new(),
                        marking: lc_core::defs::TileMarking::EMPTY,
                    };
                    app.execute_command(Box::new(AddTilesetDefCommand::new(def)));
                    app.selected_tileset_def = Some(next_id);
                    app.tileset_view_state.select_tileset(next_id);
                } else {
                    app.status_message = tr(keys::ERROR_OUTSIDE_ROOT).to_string();
                    app.tileset_view_state.error_message =
                        Some(tr(keys::ERROR_OUTSIDE_ROOT).to_string());
                }
            }
            TilesetDialogMessage::BrowseSourceImage(ts_id, path) => {
                let project_root = app
                    .project_path
                    .as_deref()
                    .and_then(|p| p.parent())
                    .unwrap_or(Path::new("."));
                let canonical_root = project_root
                    .canonicalize()
                    .unwrap_or_else(|_| project_root.to_path_buf());
                let canonical_file = path.canonicalize().unwrap_or_else(|_| path.clone());

                if let Ok(rel) = canonical_file.strip_prefix(&canonical_root) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if let Some(existing) = app.project.defs.tileset(ts_id) {
                        let mut updated = existing.clone();
                        updated.path = rel_str;
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }
                } else {
                    app.status_message = tr(keys::ERROR_OUTSIDE_ROOT).to_string();
                    app.tileset_view_state.error_message =
                        Some(tr(keys::ERROR_OUTSIDE_ROOT).to_string());
                }
            }
        }
    }

    // Sync selected tileset definition
    let tilesets_count = app.project.defs.tilesets.len();
    if tilesets_count > 0 {
        let valid_selected = app
            .tileset_view_state
            .selected_tileset_id
            .map(|id| app.project.defs.tileset(id).is_some())
            .unwrap_or(false);

        if !valid_selected {
            let fallback_id = app
                .selected_tileset_def
                .filter(|id| app.project.defs.tileset(*id).is_some())
                .unwrap_or_else(|| app.project.defs.tilesets[0].id);
            app.tileset_view_state.select_tileset(fallback_id);
            app.selected_tileset_def = Some(fallback_id);
        } else if let Some(ext_id) = app.selected_tileset_def {
            if ext_id != app.tileset_view_state.selected_tileset_id.unwrap()
                && app.project.defs.tileset(ext_id).is_some()
            {
                app.tileset_view_state.select_tileset(ext_id);
            }
        }
    } else {
        app.tileset_view_state.selected_tileset_id = None;
        app.selected_tileset_def = None;
    }

    // Outer workspace background
    let full_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(full_rect, CornerRadius::ZERO, t.bg);

    // Left panel drag list state
    let drag_state_id = Id::new("tileset_left_panel_drag_state");
    let mut drag_state = ui.ctx().data_mut(|d| {
        d.get_temp::<DragListState>(drag_state_id)
            .unwrap_or_default()
    });

    let left_w = def_panel_width();

    ui.horizontal(|ui| {
        // Left Panel
        ui.allocate_ui(vec2(left_w, full_rect.height()), |ui| {
            render_left_panel(app, ui, &t, dialog_mgr.as_ref(), &mut drag_state);
        });

        // Detail / Workspace Area
        let detail_w = ui.available_width().max(0.0);
        ui.allocate_ui(vec2(detail_w, full_rect.height()), |ui| {
            render_detail_area(app, ui, &t, &dialog_mgr);
        });
    });

    ui.ctx()
        .data_mut(|d| d.insert_temp(drag_state_id, drag_state));
}

/// Renders the main content area to the right of the tileset list.
fn render_detail_area(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    dialogs: &Arc<TilesetDialogManager>,
) {
    let tileset = app
        .tileset_view_state
        .selected_tileset_id
        .and_then(|id| app.project.defs.tileset(id).cloned());

    if tileset.is_none() {
        // Empty State: No tilesets
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(tr(keys::EMPTY_MESSAGE))
                        .font(egui::FontId::proportional(16.0))
                        .color(t.ink_strong),
                );
                ui.add_space(10.0);
                let btn = Button::new(
                    RichText::new(tr(keys::EMPTY_CREATE_BUTTON))
                        .font(egui::FontId::proportional(13.0))
                        .color(t.invert_fg),
                )
                .fill(t.accent)
                .corner_radius(CornerRadius::same(4));
                if ui.add(btn).clicked() {
                    dialogs.request_new_tileset();
                }
            });
        });
        return;
    }

    let tileset = tileset.unwrap();
    let sections = sections_for(app.tileset_view_state.active_sub_tab);

    ui.vertical(|ui| {
        // Sub-tabs on the very top: [TILESET DEF | DOCUMENT]
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            let labels = [tr(keys::TAB_DEFINITION), tr(keys::TAB_DOCUMENT)];
            sub_tabs(ui, t, &mut app.tileset_view_state.active_sub_tab, &labels);
        });

        ui.add_space(6.0);

        // Header title row & description (only for TILESET DEF tab, below sub-tabs)
        if sections.title {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    RichText::new(format!("{} (Tileset)", tileset.name))
                        .font(egui::FontId::proportional(18.0))
                        .color(t.ink_strong),
                );
            });
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    RichText::new(tr(keys::DESCRIPTION))
                        .font(egui::FontId::proportional(12.0))
                        .color(t.muted),
                );
            });
            ui.add_space(6.0);
        }

        // Content dispatch based on sub-tab
        if app.tileset_view_state.active_sub_tab == DOCUMENT_SUB_TAB {
            render_document_page(
                ui,
                t,
                &app.tileset_view_state.doc_cache,
                &mut app.tileset_view_state.doc_scroll,
                "tileset_document_scroll",
                doc("tileset", "overview"),
            );
        } else {
            // TILESET DEF Tab
            let tex = app.assets.load_texture(ui.ctx(), &tileset.path).ok();

            // Attribute Card + Image info
            ui.scope(|ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        render_attributes_card(app, ui, t, &tileset, dialogs, tex.as_ref());
                    });
                    ui.add_space(16.0);
                });
            });

            ui.add_space(6.0);

            // Center area: Atlas view on left, Right panel shell on right
            let footer_height = if sections.footer { 32.0 } else { 0.0 };
            let center_h = (ui.available_height() - footer_height - 4.0).max(120.0);
            let right_panel_w = 240.0;
            let atlas_w = (ui.available_width() - right_panel_w).max(100.0);

            ui.allocate_ui(vec2(ui.available_width(), center_h), |ui| {
                ui.horizontal(|ui| {
                    // Atlas View (fills space between card/info line and footer)
                    ui.allocate_ui(vec2(atlas_w, center_h), |ui| {
                        render_atlas_view(
                            ui,
                            t,
                            &tileset,
                            tex.as_ref(),
                            &mut app.tileset_view_state,
                        );
                    });

                    // Right Panel Shell (docked with divider line on left)
                    ui.allocate_ui(vec2(right_panel_w, center_h), |ui| {
                        render_right_panel(
                            ui,
                            t,
                            &tileset,
                            &app.project,
                            &mut app.tileset_view_state,
                        );
                    });
                });
            });

            // Pinned Footer Bar
            if sections.footer {
                let (layers_count, enums_count) = tileset_usage_counts(&app.project, tileset.id);
                let left_text = format!(
                    "ID: TilesetDefId({}) • Used by {} Layer(s) & {} Enum(s)",
                    tileset.id.0, layers_count, enums_count
                );
                def_footer(ui, t, &left_text, |_| {});
            }
        }
    });
}
