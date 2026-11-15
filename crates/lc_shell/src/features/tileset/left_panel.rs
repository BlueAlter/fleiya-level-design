//! Left panel list of tileset definitions with search, drag-reorder, and context actions.

use egui::{
    vec2, Button, Color32, CornerRadius, CursorIcon, Id, RichText, ScrollArea, Sense, Stroke,
    TextEdit, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_feature_tileset::{
    generate_unique_tileset_name, AddTilesetDefCommand, RemoveTilesetDefCommand,
    ReorderTilesetDefCommand,
};

use super::file_dialog::TilesetDialogManager;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::context_menu::{show_context_menu, MenuAnchor, MenuItem};
use crate::shared::ui::drag_list::{paint_placeholder, DragListState, DragOutcome, DragTheme};
use crate::shared::ui::reorder::Slot;

const CARD_HEIGHT: f32 = 34.0;
const CARD_GAP: f32 = 4.0;

/// Renders the left sidebar list of tilesets.
pub fn render_left_panel(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    dialogs: &TilesetDialogManager,
    drag_state: &mut DragListState,
) {
    let panel_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(panel_rect, CornerRadius::ZERO, t.panel);

    ui.scope_builder(egui::UiBuilder::new().max_rect(panel_rect), |ui| {
        ui.vertical(|ui| {
            ui.add_space(8.0);

            // + New Tileset button and ⋮ menu button
            let btn_h = 28.0;
            let gap = 6.0;
            let dots_btn_size = btn_h;
            let new_btn_w = (ui.available_width() - dots_btn_size - gap - 8.0).max(1.0);

            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.spacing_mut().item_spacing.x = gap;

                let btn = Button::new(
                    RichText::new(tr(keys::NEW_BUTTON))
                        .font(egui::FontId::proportional(13.0))
                        .color(t.ink_strong),
                )
                .fill(t.accent)
                .corner_radius(CornerRadius::same(4));

                if ui.add_sized(vec2(new_btn_w, btn_h), btn).clicked() {
                    dialogs.request_new_tileset();
                }

                let dots_btn = Button::new(
                    RichText::new("⋮")
                        .font(egui::FontId::proportional(14.0))
                        .color(t.ink_strong),
                )
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.line))
                .corner_radius(CornerRadius::same(4));

                let _dots_resp = ui.add_sized(vec2(dots_btn_size, btn_h), dots_btn);
            });

            ui.add_space(6.0);

            // Search field
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let search_width = (ui.available_width() - 8.0).max(10.0);
                ui.add_sized(
                    vec2(search_width, 24.0),
                    TextEdit::singleline(&mut app.tileset_view_state.search_query)
                        .hint_text(tr(keys::SEARCH_PLACEHOLDER)),
                );
            });

            ui.add_space(8.0);

            // Header: TILESET DEFINITIONS (N)
            let total_count = app.project.defs.tilesets.len();
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("{} ({})", tr(keys::HEADER), total_count))
                        .font(egui::FontId::proportional(11.0))
                        .color(t.muted),
                );
            });

            ui.add_space(4.0);

            // Filter items by search query
            let query = app.tileset_view_state.search_query.trim().to_lowercase();
            let tileset_indices: Vec<usize> = app
                .project
                .defs
                .tilesets
                .iter()
                .enumerate()
                .filter(|(_, ts)| query.is_empty() || ts.name.to_lowercase().contains(&query))
                .map(|(idx, _)| idx)
                .collect();

            // Reorderable list
            let drag_theme = DragTheme {
                accent: t.accent,
                placeholder_style: crate::shared::ui::drag_list::PlaceholderStyle::Solid,
                placeholder_fill: Color32::from_rgba_unmultiplied(61, 214, 184, 25),
                stroke_width: 1.5,
                dash_len: 4.0,
                gap_len: 2.0,
                auto_scroll_edge: 20.0,
                auto_scroll_speed: 4.0,
            };

            let is_filtered = !query.is_empty();
            let count = tileset_indices.len();
            let slots = if is_filtered {
                (0..count).map(Slot::Item).collect()
            } else {
                drag_state.layout(count)
            };

            let mut slot_rects = Vec::with_capacity(slots.len());
            let mut pending_action: Option<(String, TilesetDefId)> = None;

            ScrollArea::vertical()
                .id_salt("tileset_left_list_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(2.0);

                    for slot in slots {
                        match slot {
                            Slot::Placeholder => {
                                let (ph_rect, _) = ui.allocate_exact_size(
                                    vec2(ui.available_width() - 8.0, CARD_HEIGHT),
                                    Sense::hover(),
                                );
                                paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                                slot_rects.push(ph_rect);
                                ui.add_space(CARD_GAP);
                            }
                            Slot::Item(idx) => {
                                if idx >= tileset_indices.len() {
                                    continue;
                                }
                                let real_idx = tileset_indices[idx];
                                let ts = app.project.defs.tilesets[real_idx].clone();
                                let is_selected =
                                    app.tileset_view_state.selected_tileset_id == Some(ts.id);

                                let (card_rect, resp) = ui.allocate_exact_size(
                                    vec2(ui.available_width() - 8.0, CARD_HEIGHT),
                                    Sense::click(),
                                );
                                slot_rects.push(card_rect);

                                if resp.clicked() {
                                    app.tileset_view_state.select_tileset(ts.id);
                                    app.selected_tileset_def = Some(ts.id);
                                }

                                // Card background
                                let bg_color = if is_selected || resp.hovered() {
                                    t.sunken
                                } else {
                                    t.panel
                                };
                                ui.painter().rect_filled(
                                    card_rect,
                                    CornerRadius::same(4),
                                    bg_color,
                                );

                                let stroke_color = if is_selected { t.accent } else { t.line };
                                ui.painter().rect_stroke(
                                    card_rect,
                                    CornerRadius::same(4),
                                    Stroke::new(1.0, stroke_color),
                                    egui::StrokeKind::Inside,
                                );

                                // Inner card layout
                                let inner_rect = card_rect.shrink2(vec2(6.0, 4.0));
                                ui.scope_builder(
                                    egui::UiBuilder::new().max_rect(inner_rect),
                                    |ui| {
                                        ui.horizontal(|ui| {
                                            // Drag handle (only enabled when not searching)
                                            if !is_filtered {
                                                let (handle_rect, handle_resp) = ui
                                                    .allocate_exact_size(
                                                        vec2(14.0, 20.0),
                                                        Sense::drag(),
                                                    );
                                                if handle_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                                                }
                                                if handle_resp.drag_started() {
                                                    drag_state.begin(ui.ctx(), idx, card_rect);
                                                }

                                                // Draw 6 grip dots
                                                let dot_color = if handle_resp.hovered() {
                                                    t.ink_strong
                                                } else {
                                                    t.muted
                                                };
                                                for col in 0..2 {
                                                    for row in 0..3 {
                                                        let cx = handle_rect.min.x
                                                            + 3.0
                                                            + col as f32 * 6.0;
                                                        let cy = handle_rect.min.y
                                                            + 4.0
                                                            + row as f32 * 6.0;
                                                        ui.painter().circle_filled(
                                                            egui::pos2(cx, cy),
                                                            1.2,
                                                            dot_color,
                                                        );
                                                    }
                                                }
                                            }

                                            // Tileset Name
                                            let label_w = (ui.available_width() - 24.0).max(10.0);
                                            let name_text = RichText::new(&ts.name)
                                                .font(egui::FontId::proportional(12.5))
                                                .color(if is_selected {
                                                    t.ink_strong
                                                } else {
                                                    t.ink
                                                });
                                            ui.add_sized(
                                                vec2(label_w, 20.0),
                                                egui::Label::new(name_text)
                                                    .truncate()
                                                    .sense(Sense::click()),
                                            );

                                            // Context menu button (⋮)
                                            let (kebab_rect, kebab_resp) = ui.allocate_exact_size(
                                                vec2(18.0, 20.0),
                                                Sense::click(),
                                            );
                                            if kebab_resp.hovered() {
                                                ui.painter().rect_filled(
                                                    kebab_rect,
                                                    CornerRadius::same(3),
                                                    t.sunken,
                                                );
                                            }
                                            let dot_color = if kebab_resp.hovered() {
                                                t.ink_strong
                                            } else {
                                                t.muted
                                            };
                                            for row in 0..3 {
                                                let cx = kebab_rect.center().x;
                                                let cy = kebab_rect.min.y + 4.0 + row as f32 * 6.0;
                                                ui.painter().circle_filled(
                                                    egui::pos2(cx, cy),
                                                    1.4,
                                                    dot_color,
                                                );
                                            }

                                            let menu_items = vec![
                                                MenuItem::action(
                                                    "duplicate",
                                                    tr(keys::MENU_DUPLICATE),
                                                    Icon::Duplicate,
                                                ),
                                                MenuItem::destructive(
                                                    "delete",
                                                    tr(keys::MENU_DELETE),
                                                    Icon::Delete,
                                                ),
                                            ];

                                            let popup_id =
                                                Id::new(("tileset_row_menu_popup", ts.id));
                                            if let Some(action) = show_context_menu(
                                                ui,
                                                MenuAnchor::button_with_id(&kebab_resp, popup_id),
                                                &menu_items,
                                            ) {
                                                pending_action = Some((action.to_string(), ts.id));
                                            }
                                        });
                                    },
                                );

                                ui.add_space(CARD_GAP);
                            }
                        }
                    }
                });

            // Handle drag outcome
            if !is_filtered {
                drag_state.update(ui, &slot_rects, &drag_theme);
                let outcome = drag_state.finish(ui);
                if let DragOutcome::Dropped { from, to } = outcome {
                    if from < tileset_indices.len() && to < tileset_indices.len() {
                        let from_real = tileset_indices[from];
                        let to_real = tileset_indices[to];
                        let ts_id = app.project.defs.tilesets[from_real].id;
                        app.execute_command(Box::new(ReorderTilesetDefCommand::new(
                            ts_id, to_real,
                        )));
                    }
                }
            }

            // Handle context menu actions
            if let Some((action, ts_id)) = pending_action {
                match action.as_str() {
                    "duplicate" => {
                        if let Some(source) = app.project.defs.tileset(ts_id) {
                            let existing_names: Vec<String> = app
                                .project
                                .defs
                                .tilesets
                                .iter()
                                .map(|t| t.name.clone())
                                .collect();
                            let unique_name =
                                generate_unique_tileset_name(&existing_names, &source.name);
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
                            let dup = TilesetDef {
                                id: next_id,
                                name: unique_name,
                                path: source.path.clone(),
                                tile_size: source.tile_size,
                                spacing: source.spacing,
                                padding: source.padding,
                            };
                            app.execute_command(Box::new(AddTilesetDefCommand::new(dup)));
                            app.tileset_view_state.select_tileset(next_id);
                            app.selected_tileset_def = Some(next_id);
                        }
                    }
                    "delete" => {
                        if RemoveTilesetDefCommand::is_referenced(&app.project, ts_id) {
                            app.tileset_view_state.pending_delete_id = Some(ts_id);
                        } else {
                            app.execute_command(Box::new(RemoveTilesetDefCommand::new(ts_id)));
                            if app.tileset_view_state.selected_tileset_id == Some(ts_id) {
                                app.tileset_view_state.selected_tileset_id = None;
                                app.tileset_view_state.selected_tile = None;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Delete confirmation modal when referenced
            if let Some(del_id) = app.tileset_view_state.pending_delete_id {
                egui::Window::new(tr(keys::DELETE_CONFIRM_TITLE))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            ui.label(tr(keys::DELETE_CONFIRM_MESSAGE));
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                if ui.button(tr(keys::DELETE_CANCEL_BUTTON)).clicked() {
                                    app.tileset_view_state.pending_delete_id = None;
                                }
                                let del_btn = Button::new(
                                    RichText::new(tr(keys::DELETE_CONFIRM_BUTTON))
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(220, 38, 38));
                                if ui.add(del_btn).clicked() {
                                    app.execute_command(Box::new(RemoveTilesetDefCommand::new(
                                        del_id,
                                    )));
                                    if app.tileset_view_state.selected_tileset_id == Some(del_id) {
                                        app.tileset_view_state.selected_tileset_id = None;
                                        app.tileset_view_state.selected_tile = None;
                                    }
                                    app.tileset_view_state.pending_delete_id = None;
                                }
                            });
                        });
                    });
            }
        });
    });
}
