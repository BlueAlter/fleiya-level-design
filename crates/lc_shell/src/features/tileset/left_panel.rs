//! Left panel list of tileset definitions with search, drag-reorder, and context actions.

use egui::Ui;
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
use crate::shared::ui::context_menu::{show_context_menu_contents, ContextMenuTheme, MenuItem};
use crate::shared::ui::def_workspace::{
    def_left_panel, DefItem, DefLeftPanelAction, DefLeftPanelConfig,
};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};

/// Renders the left sidebar list of tilesets using the shared definition workspace kit.
pub fn render_left_panel(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    dialogs: &TilesetDialogManager,
    drag_state: &mut DragListState,
) {
    let items: Vec<DefItem<TilesetDefId>> = app
        .project
        .defs
        .tilesets
        .iter()
        .map(|ts| DefItem::new(ts.id, ts.name.clone()))
        .collect();

    let config = DefLeftPanelConfig::new(tr(keys::NEW_BUTTON), tr(keys::HEADER)).with_menu(true);

    let actions = def_left_panel(
        ui,
        t,
        &config,
        &mut app.tileset_view_state.search_query,
        &items,
        app.tileset_view_state.selected_tileset_id,
        drag_state,
    );

    let mut pending_action: Option<(String, TilesetDefId)> = None;

    for action in actions {
        match action {
            DefLeftPanelAction::NewClicked => {
                dialogs.request_new_tileset();
            }
            DefLeftPanelAction::MenuClicked => {
                // Top ⋮ menu button: could open a general menu if needed
            }
            DefLeftPanelAction::Selected(id) => {
                app.selected_tileset_def = Some(id);
                app.tileset_view_state.select_tileset(id);
                app.tileset_view_state.reset_atlas_fit(id);
            }
            DefLeftPanelAction::ContextMenu { id, pos } => {
                app.tileset_view_state.context_menu = Some((id, pos));
            }
            DefLeftPanelAction::Reorder { from, to } => {
                if let Some(item) = app.project.defs.tilesets.get(from) {
                    let ts_id = item.id;
                    app.execute_command(Box::new(ReorderTilesetDefCommand::new(ts_id, to)));
                }
            }
        }
    }

    // Context menu popup
    if let Some((menu_id, pos)) = app.tileset_view_state.context_menu {
        let items = [
            MenuItem::action("duplicate", tr(keys::MENU_DUPLICATE), Icon::Duplicate),
            MenuItem::destructive("delete", tr(keys::MENU_DELETE), Icon::Delete),
        ];

        let theme = ContextMenuTheme::default();
        let menu_w = theme.compute_menu_width(ui.ctx(), &items);
        let popup_id = egui::Id::new(("tileset_context_menu", menu_id));
        let mut closed = false;

        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                theme.frame(t).show(ui, |ui| {
                    if let Some(action_id) =
                        show_context_menu_contents(ui, &theme, t, menu_w, &items)
                    {
                        pending_action = Some((action_id.to_string(), menu_id));
                        closed = true;
                    }
                });
            });

        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
            if ui.input(|i| i.pointer.any_pressed())
                && !area_resp.response.rect.contains(pointer_pos)
            {
                closed = true;
            }
        }

        if closed {
            app.tileset_view_state.context_menu = None;
        }
    }

    // Handle context menu action
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
                    let unique_name = generate_unique_tileset_name(&existing_names, &source.name);
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
                        tags: source.tags.clone(),
                        marking: source.marking.clone(),
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
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum DeleteModalAction {
            Cancel,
            Delete,
        }

        let (layers_count, enums_count) =
            lc_feature_tileset::tileset_usage_counts(&app.project, del_id);
        let mut level_tiles_count = 0;
        for world in &app.project.worlds {
            for level in &world.levels {
                for layer in &level.layers {
                    if let lc_core::layer::LayerData::Tiles(grid) = &layer.data {
                        level_tiles_count += grid
                            .cells()
                            .iter()
                            .filter(|cell| cell.as_ref().map(|t| t.tileset_id) == Some(del_id))
                            .count();
                    }
                }
            }
        }

        let buttons = [
            ModalButton::new(
                DeleteModalAction::Cancel,
                tr(keys::DELETE_CANCEL_BUTTON),
                ModalButtonKind::Normal,
            ),
            ModalButton::new(
                DeleteModalAction::Delete,
                tr(keys::DELETE_CONFIRM_BUTTON),
                ModalButtonKind::Danger,
            ),
        ];

        let result = modal(
            ui.ctx(),
            "tileset_delete_confirm_modal",
            ModalSpec {
                title: tr(keys::DELETE_CONFIRM_TITLE).to_string(),
                body: |ui| {
                    ui.label(tr(keys::DELETE_CONFIRM_MESSAGE));
                    ui.add_space(8.0);
                    if layers_count > 0 {
                        ui.label(format!("• {layers_count} Layer definition(s)"));
                    }
                    if enums_count > 0 {
                        ui.label(format!("• {enums_count} Enum value icon(s)"));
                    }
                    if level_tiles_count > 0 {
                        ui.label(format!("• {level_tiles_count} Level tile cell(s)"));
                    }
                },
                buttons: &buttons,
                width: 380.0,
                default_button: Some(DeleteModalAction::Cancel),
            },
        );

        match result {
            Some(DeleteModalAction::Delete) => {
                app.execute_command(Box::new(RemoveTilesetDefCommand::new(del_id)));
                if app.tileset_view_state.selected_tileset_id == Some(del_id) {
                    app.tileset_view_state.selected_tileset_id = None;
                    app.tileset_view_state.selected_tile = None;
                }
                app.tileset_view_state.pending_delete_id = None;
            }
            Some(DeleteModalAction::Cancel) => {
                app.tileset_view_state.pending_delete_id = None;
            }
            None => {}
        }
    }
}
