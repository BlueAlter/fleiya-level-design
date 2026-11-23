//! Tileset Definitions full-window mode view and detail editor.

use std::path::Path;
use std::sync::Arc;

use egui::{vec2, Button, Color32, CornerRadius, Frame, Id, Pos2, Rect, RichText, Sense, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_feature_tileset::{
    generate_unique_tileset_name, AddTilesetDefCommand, UpdateTilesetDefCommand,
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
use crate::shared::ui::def_workspace::def_panel_width;
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::render_document_page;
use crate::shared::ui::sub_tabs::sub_tabs;
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

/// Sub-tab index for the tileset definition editor.
pub const TILESET_DEF_SUB_TAB: usize = 0;
/// Sub-tab index for the documentation page.
pub const DOCUMENT_SUB_TAB: usize = 1;

/// Chrome sections shown in the detail panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailSections {
    /// Whether to draw the title row above content.
    pub title: bool,
    /// Whether to draw the pinned footer bar at bottom (false for tileset workspace).
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
            footer: false,
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
    let left_rect = Rect::from_min_max(
        full_rect.min,
        Pos2::new(full_rect.min.x + left_w, full_rect.max.y),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        render_left_panel(app, ui, &t, dialog_mgr.as_ref(), &mut drag_state);
    });

    ui.ctx()
        .data_mut(|d| d.insert_temp(drag_state_id, drag_state));

    let tileset = app
        .tileset_view_state
        .selected_tileset_id
        .and_then(|id| app.project.defs.tileset(id).cloned());

    let sections = sections_for(app.tileset_view_state.active_sub_tab);

    let content_bottom = full_rect.max.y - WORKSPACE_PADDING;

    let content_rect = Rect::from_min_max(
        Pos2::new(
            left_rect.max.x + WORKSPACE_PADDING,
            full_rect.min.y + WORKSPACE_PADDING,
        ),
        Pos2::new(full_rect.max.x - WORKSPACE_PADDING, content_bottom),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        render_detail_area(
            app,
            ui,
            &t,
            &dialog_mgr,
            tileset.as_ref(),
            sections,
            content_rect,
        );
    });
}

/// Renders the main content area to the right of the tileset list.
fn render_detail_area(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    dialogs: &Arc<TilesetDialogManager>,
    tileset: Option<&TilesetDef>,
    sections: DetailSections,
    content_rect: Rect,
) {
    let Some(tileset) = tileset else {
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
    };

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Sub-tabs at top: [TILESET DEF | DOCUMENT]
        Frame::new().fill(Color32::TRANSPARENT).show(ui, |ui| {
            ui.set_min_height(CONTROL_HEIGHT_COMPACT);
            ui.set_max_height(CONTROL_HEIGHT_COMPACT);
            let labels = [tr(keys::TAB_DEFINITION), tr(keys::TAB_DOCUMENT)];
            let mut active = app.tileset_view_state.active_sub_tab;
            if sub_tabs(ui, t, &mut active, &labels) {
                app.tileset_view_state.set_sub_tab(active);
            }
        });

        // Content dispatch based on sub-tab with separate id_salt
        if app.tileset_view_state.active_sub_tab == DOCUMENT_SUB_TAB {
            ui.scope_builder(
                egui::UiBuilder::new().id_salt("tileset_tab_document"),
                |ui| {
                    ui.add_space(12.0);
                    render_document_page(
                        ui,
                        t,
                        &app.tileset_view_state.doc_cache,
                        &mut app.tileset_view_state.doc_scroll,
                        "tileset_document_scroll",
                        doc("tileset", "overview"),
                    );
                },
            );
        } else {
            ui.scope_builder(
                egui::UiBuilder::new().id_salt("tileset_tab_definition"),
                |ui| {
                    // Header title row & description (only for TILESET DEF tab, below sub-tabs)
                    if sections.title {
                        ui.add_space(12.0);
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!("{} (Tileset)", tileset.name))
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
                    }

                    // TILESET DEF Tab
                    let mut tex = app.assets.load_texture(ui.ctx(), &tileset.path).ok();

                    // Attribute Card + Image info
                    render_attributes_card(app, ui, t, tileset, dialogs.as_ref(), tex.as_mut());

                    ui.add_space(10.0);

                    // Center area: Atlas card on left (flex width), Tools card on right (240px)
                    let mode_card_w = 240.0;
                    let gap = WORKSPACE_PADDING; // 12.0
                    let atlas_card_w = (content_rect.width() - mode_card_w - gap).max(100.0);

                    let (img_w, img_h) = if let Some(ref t) = tex {
                        let sz = t.size();
                        (sz[0] as u32, sz[1] as u32)
                    } else {
                        (0, 0)
                    };

                    let atlas_tooltip = if img_w > 0 && img_h > 0 {
                        let metrics = lc_feature_tileset::compute_tileset_grid(
                            [img_w, img_h],
                            tileset.tile_size,
                            tileset.spacing,
                            tileset.padding,
                        );
                        format!(
                            "Atlas {}x{} px, {} x {} tiles ({})",
                            img_w, img_h, metrics.cols, metrics.rows, metrics.total_tiles
                        )
                    } else if !tileset.path.is_empty() {
                        "Source image file could not be loaded.".to_string()
                    } else {
                        "No image assigned.".to_string()
                    };

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let (atlas_label_rect, _) =
                            ui.allocate_exact_size(vec2(atlas_card_w, 18.0), Sense::hover());
                        let mut child =
                            ui.new_child(egui::UiBuilder::new().max_rect(atlas_label_rect));
                        child
                            .add(
                                egui::Label::new(
                                    RichText::new("TILE ATLAS")
                                        .size(11.0)
                                        .strong()
                                        .color(t.accent_ink),
                                )
                                .sense(Sense::hover()),
                            )
                            .on_hover_text(atlas_tooltip);

                        ui.add_space(gap);

                        let (tools_label_rect, _) =
                            ui.allocate_exact_size(vec2(mode_card_w, 18.0), Sense::hover());
                        let mut child =
                            ui.new_child(egui::UiBuilder::new().max_rect(tools_label_rect));
                        child.label(
                            RichText::new("TOOLS")
                                .size(11.0)
                                .strong()
                                .color(t.accent_ink),
                        );
                    });

                    ui.add_space(4.0);

                    let card_top = ui.cursor().min.y;
                    let card_bottom = content_rect.max.y;
                    let card_h = (card_bottom - card_top).max(60.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;

                        // Bordered Atlas Card
                        ui.allocate_ui(vec2(atlas_card_w, card_h), |ui| {
                            Frame::new()
                                .fill(t.canvas)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::same(4))
                                .show(ui, |ui| {
                                    render_atlas_view(
                                        ui,
                                        t,
                                        tileset,
                                        tex.as_ref(),
                                        &mut app.tileset_view_state,
                                    );
                                });
                        });

                        ui.add_space(gap);

                        // Bordered Tools / Mode Card
                        ui.allocate_ui(vec2(mode_card_w, card_h), |ui| {
                            Frame::new()
                                .fill(t.panel)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::same(4))
                                .show(ui, |ui| {
                                    render_right_panel(
                                        ui,
                                        t,
                                        tileset,
                                        &app.project,
                                        &mut app.tileset_view_state,
                                    );
                                });
                        });
                    });
                },
            );
        }
    });
}
