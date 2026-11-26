//! Left sidebar list of entity definitions with search, drag-reorder, and context actions.

use egui::{Color32, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, EntityRenderMode, TilesetRect};
use lc_core::id::EntityDefId;
use lc_feature_entities::{
    count_entity_def_instances, generate_unique_entity_name, AddEntityDefCommand,
    RemoveEntityDefCommand, ReorderEntityDefCommand,
};
use lc_ui_api::AssetService;

use super::attributes_card::resolve_tile_icon;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::context_menu::{show_context_menu_contents, ContextMenuTheme, MenuItem};
use crate::shared::ui::def_workspace::{
    def_left_panel, DefItem, DefItemIcon, DefLeftPanelAction, DefLeftPanelConfig,
};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};

/// Creates and adds a new [`EntityDef`] with LDtk defaults, returning its assigned ID.
pub fn create_new_entity_def(app: &mut ShellApp) -> EntityDefId {
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
    let mut def = EntityDef::new(next_id, format!("Entity_{}", next_id.0));
    let palette: [[u8; 4]; 6] = [
        [148, 200, 120, 255],
        [240, 160, 60, 255],
        [96, 165, 250, 255],
        [192, 132, 252, 255],
        [244, 114, 182, 255],
        [52, 211, 153, 255],
    ];
    def.color = palette[(next_id.0 as usize).saturating_sub(1) % palette.len()];
    app.execute_command(Box::new(AddEntityDefCommand::new(def)));
    app.selected_entity_def = Some(next_id);
    app.entity_view_state.select_entity(next_id);
    next_id
}

/// Renders the left sidebar list of entity definitions using the shared `def_left_panel`.
pub fn render_left_panel(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    drag_state: &mut DragListState,
) {
    let query = app.entity_view_state.search_query.trim().to_lowercase();
    let tag_filter = query.strip_prefix('#').map(str::trim);

    let items: Vec<DefItem<EntityDefId>> = app
        .project
        .defs
        .entities
        .iter()
        .filter(|e| {
            if query.is_empty() {
                return true;
            }
            if let Some(tag_q) = tag_filter {
                e.tags.iter().any(|tag| tag.to_lowercase().contains(tag_q))
            } else {
                e.name.to_lowercase().contains(&query)
                    || e.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            }
        })
        .map(|e| {
            let icon = resolve_entity_card_icon(app, ui, e);
            DefItem::new(e.id, e.name.clone())
                .with_tags(e.tags.clone())
                .with_icon(icon)
        })
        .collect();
    let filtered_ids: Vec<EntityDefId> = items.iter().map(|i| i.id).collect();

    let config = DefLeftPanelConfig::new(tr(keys::NEW_BUTTON), tr(keys::HEADER)).with_menu(false);

    let actions = def_left_panel(
        ui,
        t,
        &config,
        &mut app.entity_view_state.search_query,
        &items,
        app.entity_view_state.selected_entity_id,
        drag_state,
    );

    let mut pending_action: Option<(String, EntityDefId)> = None;

    for action in actions {
        match action {
            DefLeftPanelAction::NewClicked => {
                create_new_entity_def(app);
            }
            DefLeftPanelAction::MenuClicked => {}
            DefLeftPanelAction::Selected(id) => {
                app.selected_entity_def = Some(id);
                app.entity_view_state.select_entity(id);
            }
            DefLeftPanelAction::ContextMenu { id, pos } => {
                app.entity_view_state.context_menu = Some((id, pos));
            }
            DefLeftPanelAction::Reorder { from, to } => {
                if let (Some(&from_id), Some(&target_id)) =
                    (filtered_ids.get(from), filtered_ids.get(to))
                {
                    if let Some(real_to) = app
                        .project
                        .defs
                        .entities
                        .iter()
                        .position(|e| e.id == target_id)
                    {
                        app.execute_command(Box::new(ReorderEntityDefCommand::new(
                            from_id, real_to,
                        )));
                    }
                }
            }
        }
    }

    // Context menu popup
    if let Some((menu_id, pos)) = app.entity_view_state.context_menu {
        let menu_items = [
            MenuItem::action("duplicate", tr(keys::MENU_DUPLICATE), Icon::Duplicate),
            MenuItem::destructive("delete", tr(keys::MENU_DELETE), Icon::Delete),
        ];

        let theme = ContextMenuTheme::default();
        let menu_w = theme.compute_menu_width(ui.ctx(), &menu_items);
        let popup_id = egui::Id::new(("entity_context_menu", menu_id));
        let mut closed = false;

        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                theme.frame(t).show(ui, |ui| {
                    if let Some(action_id) =
                        show_context_menu_contents(ui, &theme, t, menu_w, &menu_items)
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
            app.entity_view_state.context_menu = None;
        }
    }

    if let Some((action, entity_id)) = pending_action {
        match action.as_str() {
            "duplicate" => {
                if let Some(source) = app.project.defs.entity(entity_id).cloned() {
                    let existing_names: Vec<String> = app
                        .project
                        .defs
                        .entities
                        .iter()
                        .map(|e| e.name.clone())
                        .collect();
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
                    let mut dup = source;
                    dup.id = next_id;
                    dup.name = generate_unique_entity_name(&existing_names, &dup.name);
                    app.execute_command(Box::new(AddEntityDefCommand::new(dup)));
                    app.selected_entity_def = Some(next_id);
                    app.entity_view_state.select_entity(next_id);
                }
            }
            "delete" => {
                let usage = count_entity_def_instances(&app.project, entity_id);
                if usage > 0 {
                    let name = app
                        .project
                        .defs
                        .entity(entity_id)
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| "Entity".to_string());
                    app.entity_view_state.pending_delete = Some((entity_id, name, usage));
                } else {
                    app.execute_command(Box::new(RemoveEntityDefCommand::safe(entity_id)));
                    if app.entity_view_state.selected_entity_id == Some(entity_id) {
                        app.entity_view_state.selected_entity_id = None;
                        app.selected_entity_def = None;
                    }
                }
            }
            _ => {}
        }
    }

    // Cascade delete confirmation modal
    if let Some((del_id, del_name, usage)) = app.entity_view_state.pending_delete.clone() {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum DeleteModalAction {
            Cancel,
            Delete,
        }

        let buttons = [
            ModalButton::new(DeleteModalAction::Cancel, "Cancel", ModalButtonKind::Normal),
            ModalButton::new(
                DeleteModalAction::Delete,
                "Delete & Cascade",
                ModalButtonKind::Danger,
            ),
        ];

        let result = modal(
            ui.ctx(),
            "entity_delete_confirm_modal",
            ModalSpec {
                title: "Confirm Entity Deletion".to_string(),
                body: |ui| {
                    ui.label(format!(
                        "Delete entity definition '{del_name}'? This will also remove {usage} placed instance(s) and clear any EntityRef values pointing to them."
                    ));
                },
                buttons: &buttons,
                default_button: Some(DeleteModalAction::Cancel),
                width: 400.0,
            },
        );

        match result {
            Some(DeleteModalAction::Cancel) => {
                app.entity_view_state.pending_delete = None;
            }
            Some(DeleteModalAction::Delete) => {
                app.execute_command(Box::new(RemoveEntityDefCommand::cascade(del_id)));
                if app.entity_view_state.selected_entity_id == Some(del_id) {
                    app.entity_view_state.selected_entity_id = None;
                    app.selected_entity_def = None;
                }
                app.entity_view_state.pending_delete = None;
            }
            None => {}
        }
    }
}

fn resolve_entity_card_icon(app: &ShellApp, ui: &Ui, def: &EntityDef) -> DefItemIcon {
    let base_color = Color32::from_rgba_unmultiplied(def.color[0], def.color[1], def.color[2], 255);
    let tile_rect = def
        .ui_tile_rect
        .or_else(|| {
            if def.render_mode == EntityRenderMode::Tile || def.tile_rect.is_some() {
                def.tile_rect
            } else {
                None
            }
        })
        .or_else(|| {
            let legacy = def.tile?;
            let ts = app.project.defs.tileset(legacy.tileset_id)?;
            let tex = app.assets.load_texture(ui.ctx(), &ts.path).ok()?;
            let sz = tex.size_vec2();
            let tw = ts.tile_size.x.max(1);
            let th = ts.tile_size.y.max(1);
            let eff_w = (sz.x as u32).saturating_sub(ts.padding * 2);
            let cols = (eff_w + ts.spacing)
                .checked_div(tw + ts.spacing)
                .unwrap_or(1)
                .max(1);
            let col = legacy.tile_id % cols;
            let row = legacy.tile_id / cols;
            Some(TilesetRect::new(
                legacy.tileset_id,
                ts.padding + col * (tw + ts.spacing),
                ts.padding + row * (th + ts.spacing),
                tw,
                th,
            ))
        });

    let icon = resolve_tile_icon(app, ui, tile_rect);
    DefItemIcon::with_tile(base_color, def.hollow, icon)
}
