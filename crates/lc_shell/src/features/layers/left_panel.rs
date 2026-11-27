//! Left sidebar list of layer definitions with search, tag grouping, drag-reorder, and context actions.

use egui::{vec2, Button, Color32, CornerRadius, RichText, Stroke, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::id::LayerDefId;
use lc_core::layer::LayerKind;
use lc_feature_layers::{
    count_layer_def_usage, generate_unique_layer_name, AddLayerDefCommand, RemoveLayerDefCommand,
    ReorderLayerDefsCommand,
};

use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::context_menu::{show_context_menu_contents, ContextMenuTheme, MenuItem};
use crate::shared::ui::def_workspace::{
    def_left_panel, DefItem, DefItemIcon, DefLeftPanelAction, DefLeftPanelConfig,
};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};

/// Creates and adds a new [`LayerDef`] of a specific [`LayerKind`] with LDtk-compatible defaults.
pub fn create_layer_def_with_kind(app: &mut ShellApp, kind: LayerKind) -> LayerDefId {
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
    let default_grid = app.project.settings.default_grid_size;
    let default_tileset = app.project.defs.tilesets.first().map(|ts| ts.id);
    let int_grid_values = if kind == LayerKind::IntGrid {
        vec![IntGridValueDef {
            value: 1,
            name: "Solid".to_string(),
            color: [96, 165, 250, 255],
            ..Default::default()
        }]
    } else {
        Vec::new()
    };
    let auto_source_layer_def_id = if kind == LayerKind::Auto {
        app.project
            .defs
            .layers
            .iter()
            .find(|l| l.kind == LayerKind::IntGrid)
            .map(|l| l.id)
    } else {
        None
    };

    let def = LayerDef {
        id: next_id,
        name: format!("{}_{}", layer_kind_label(kind), next_id.0),
        kind,
        grid_size: default_grid,
        tileset_id: match kind {
            LayerKind::Tiles | LayerKind::Auto => default_tileset,
            LayerKind::IntGrid | LayerKind::Entities => None,
        },
        int_grid_values,
        auto_source_layer_def_id,
        ..Default::default()
    };
    app.execute_command(Box::new(AddLayerDefCommand::new(def)));
    app.selected_layer_def = Some(next_id);
    app.layer_view_state.select_layer(next_id);
    next_id
}

/// Creates and adds a new [`LayerDef`] with sensible defaults, returning its assigned ID.
pub fn create_new_layer_def(app: &mut ShellApp) -> LayerDefId {
    let default_tileset = app.project.defs.tilesets.first().map(|ts| ts.id);
    let kind = if default_tileset.is_some() {
        LayerKind::Tiles
    } else {
        LayerKind::IntGrid
    };
    create_layer_def_with_kind(app, kind)
}

/// Returns the semantic icon for a [`LayerKind`].
pub(crate) const fn layer_kind_icon(kind: LayerKind) -> Icon {
    match kind {
        LayerKind::IntGrid => Icon::IntGrid,
        LayerKind::Tiles => Icon::Tiles,
        LayerKind::Entities => Icon::Entity,
        LayerKind::Auto => Icon::AutoLayer,
    }
}

/// Returns the semantic badge color for a [`LayerKind`].
pub(crate) fn layer_kind_color(kind: LayerKind) -> Color32 {
    match kind {
        LayerKind::IntGrid => Color32::from_rgb(240, 160, 60),
        LayerKind::Tiles => Color32::from_rgb(96, 165, 250),
        LayerKind::Entities => Color32::from_rgb(148, 200, 120),
        LayerKind::Auto => Color32::from_rgb(192, 132, 252),
    }
}

/// Returns the short label for a [`LayerKind`].
pub(crate) const fn layer_kind_label(kind: LayerKind) -> &'static str {
    match kind {
        LayerKind::IntGrid => "IntGrid",
        LayerKind::Tiles => "Tiles",
        LayerKind::Entities => "Entities",
        LayerKind::Auto => "AutoLayer",
    }
}

/// Renders the left sidebar list of layer definitions using the shared `def_left_panel`.
pub fn render_left_panel(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    drag_state: &mut DragListState,
) {
    let query = app.layer_view_state.search_query.trim().to_lowercase();
    let tag_filter = query.strip_prefix('#').map(str::trim);

    let items: Vec<DefItem<LayerDefId>> = app
        .project
        .defs
        .layers
        .iter()
        .filter(|l| {
            if query.is_empty() {
                return true;
            }
            if let Some(tag_q) = tag_filter {
                l.tags.iter().any(|tag| tag.to_lowercase().contains(tag_q))
            } else {
                l.name.to_lowercase().contains(&query)
                    || layer_kind_label(l.kind).to_lowercase().contains(&query)
                    || l.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            }
        })
        .map(|l| {
            let color = l
                .ui_color
                .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], 255))
                .unwrap_or_else(|| layer_kind_color(l.kind));
            DefItem::new(l.id, l.name.clone())
                .with_tags(l.tags.clone())
                .with_icon(DefItemIcon::color(color, false))
        })
        .collect();
    let filtered_ids: Vec<LayerDefId> = items.iter().map(|i| i.id).collect();

    let config = DefLeftPanelConfig::new(tr(keys::NEW_BUTTON), tr(keys::HEADER)).with_menu(false);

    let actions = def_left_panel(
        ui,
        t,
        &config,
        &mut app.layer_view_state.search_query,
        &items,
        app.layer_view_state.selected_layer_id,
        drag_state,
    );

    let mut pending_action: Option<(String, LayerDefId)> = None;

    for action in actions {
        match action {
            DefLeftPanelAction::NewClicked => {
                app.layer_view_state.show_new_layer_modal = true;
            }
            DefLeftPanelAction::MenuClicked => {}
            DefLeftPanelAction::Selected(id) => {
                app.selected_layer_def = Some(id);
                app.layer_view_state.select_layer(id);
            }
            DefLeftPanelAction::ContextMenu { id, pos } => {
                app.layer_view_state.context_menu = Some((id, pos));
            }
            DefLeftPanelAction::Reorder { from, to } => {
                if let (Some(&from_id), Some(&target_id)) =
                    (filtered_ids.get(from), filtered_ids.get(to))
                {
                    let mut new_order: Vec<LayerDefId> =
                        app.project.defs.layers.iter().map(|l| l.id).collect();
                    if let (Some(real_from), Some(real_to)) = (
                        new_order.iter().position(|&id| id == from_id),
                        new_order.iter().position(|&id| id == target_id),
                    ) {
                        if real_from != real_to {
                            let moved = new_order.remove(real_from);
                            new_order.insert(real_to, moved);
                            app.execute_command(Box::new(ReorderLayerDefsCommand::new(new_order)));
                        }
                    }
                }
            }
        }
    }

    // New Layer kind picker modal (LDtk-style 4-kind creation flow)
    if app.layer_view_state.show_new_layer_modal {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum NewLayerModalAction {
            Cancel,
        }
        let mut chosen_kind: Option<LayerKind> = None;
        let buttons = [ModalButton::new(
            NewLayerModalAction::Cancel,
            "Cancel",
            ModalButtonKind::Normal,
        )];
        let modal_res = modal(
            ui.ctx(),
            "layer_new_kind_picker_modal",
            ModalSpec {
                title: "Create New Layer".to_string(),
                body: |ui| {
                    ui.label(
                        RichText::new("Choose the layer type (LDtk):")
                            .size(12.0)
                            .color(t.muted),
                    );
                    ui.add_space(8.0);
                    for (kind, desc) in [
                        (
                            LayerKind::IntGrid,
                            "Integer cell grid for collisions/logic (+ optional auto-tiles)",
                        ),
                        (
                            LayerKind::Tiles,
                            "Manual tilemap painted from a bound Tileset",
                        ),
                        (
                            LayerKind::Auto,
                            "Pure procedural Auto-Layer driven by an IntGrid source",
                        ),
                        (
                            LayerKind::Entities,
                            "Vector plane for entity instances and custom fields",
                        ),
                    ] {
                        let col = layer_kind_color(kind);
                        let btn = Button::new(
                            RichText::new(format!("{} — {}", layer_kind_label(kind), desc))
                                .size(12.0)
                                .color(t.ink_strong),
                        )
                        .fill(t.sunken)
                        .stroke(Stroke::new(1.0, col.gamma_multiply(0.7)))
                        .corner_radius(CornerRadius::ZERO);
                        if ui
                            .add_sized(vec2(ui.available_width(), 32.0), btn)
                            .clicked()
                        {
                            chosen_kind = Some(kind);
                        }
                        ui.add_space(4.0);
                    }
                },
                buttons: &buttons,
                width: 440.0,
                default_button: Some(NewLayerModalAction::Cancel),
            },
        );
        if let Some(kind) = chosen_kind {
            app.layer_view_state.show_new_layer_modal = false;
            create_layer_def_with_kind(app, kind);
        } else if matches!(modal_res, Some(NewLayerModalAction::Cancel)) {
            app.layer_view_state.show_new_layer_modal = false;
        }
    }

    // Context menu popup
    if let Some((menu_id, pos)) = app.layer_view_state.context_menu {
        let menu_items = [
            MenuItem::action("duplicate", tr(keys::MENU_DUPLICATE), Icon::Duplicate),
            MenuItem::destructive("delete", tr(keys::MENU_DELETE), Icon::Delete),
        ];

        let theme = ContextMenuTheme::default();
        let menu_w = theme.compute_menu_width(ui.ctx(), &menu_items);
        let popup_id = egui::Id::new(("layer_context_menu", menu_id));
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
            app.layer_view_state.context_menu = None;
        }
    }

    if let Some((action, layer_id)) = pending_action {
        match action.as_str() {
            "duplicate" => {
                if let Some(source) = app.project.defs.layer(layer_id).cloned() {
                    let existing_names: Vec<String> = app
                        .project
                        .defs
                        .layers
                        .iter()
                        .map(|l| l.name.clone())
                        .collect();
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
                    let mut dup = source;
                    dup.id = next_id;
                    dup.name = generate_unique_layer_name(&existing_names, &dup.name);
                    app.execute_command(Box::new(AddLayerDefCommand::new(dup)));
                    app.selected_layer_def = Some(next_id);
                    app.layer_view_state.select_layer(next_id);
                }
            }
            "delete" => {
                let stats = count_layer_def_usage(&app.project, layer_id);
                if stats.non_empty_levels > 0 {
                    let name = app
                        .project
                        .defs
                        .layer(layer_id)
                        .map(|l| l.name.clone())
                        .unwrap_or_else(|| "Layer".to_string());
                    app.layer_view_state.pending_delete =
                        Some((layer_id, name, stats.non_empty_levels, stats.placed_items));
                } else {
                    app.execute_command(Box::new(RemoveLayerDefCommand::with_cascade(
                        layer_id, false,
                    )));
                    if app.layer_view_state.selected_layer_id == Some(layer_id) {
                        app.layer_view_state.selected_layer_id = None;
                        app.selected_layer_def = None;
                    }
                }
            }
            _ => {}
        }
    }

    // Cascade delete confirmation modal
    if let Some((del_id, del_name, level_count, item_count)) =
        app.layer_view_state.pending_delete.clone()
    {
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
            "layer_delete_confirm_modal",
            ModalSpec {
                title: "Confirm Layer Deletion".to_string(),
                body: |ui| {
                    ui.label(format!(
                        "Delete layer definition '{del_name}'? This layer has {item_count} placed item(s) across {level_count} level(s), which will be removed."
                    ));
                },
                buttons: &buttons,
                width: 380.0,
                default_button: Some(DeleteModalAction::Cancel),
            },
        );

        match result {
            Some(DeleteModalAction::Cancel) => {
                app.layer_view_state.pending_delete = None;
            }
            Some(DeleteModalAction::Delete) => {
                app.layer_view_state.pending_delete = None;
                app.execute_command(Box::new(RemoveLayerDefCommand::new(del_id)));
                if app.layer_view_state.selected_layer_id == Some(del_id) {
                    app.layer_view_state.selected_layer_id = None;
                    app.selected_layer_def = None;
                }
            }
            None => {}
        }
    }
}
