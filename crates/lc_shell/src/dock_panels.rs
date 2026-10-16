//! Panel dock rendering for side slots and collapsible bottom tray.

use egui::{Id, Panel, Ui};
use lc_ui_api::context::EditorCtx;

use crate::dock_layout::{BottomDockState, DockSlotLocation};
use crate::shell_app::ShellApp;

/// Renders the collapsible bottom dock panel in the center column.
pub fn render_bottom_dock(app: &mut ShellApp, ui: &mut Ui) {
    if app.dock_layout.bottom.panels.is_empty() {
        return;
    }

    let is_collapsed = app.dock_layout.bottom.collapsed;
    let height = if is_collapsed {
        BottomDockState::COLLAPSED_HEIGHT
    } else {
        app.dock_layout.bottom.height
    };

    let bar_id = Id::new("bottom_dock");
    if !is_collapsed {
        if let Some(mut state) = egui::panel::PanelState::load(ui.ctx(), bar_id) {
            if state.outer_rect.height() < BottomDockState::COLLAPSED_HEIGHT + 2.0 {
                state.outer_rect.min.y = state.outer_rect.max.y - app.dock_layout.bottom.height;
                ui.ctx().data_mut(|d| d.insert_persisted(bar_id, state));
            }
        }
    }

    let panel = if is_collapsed {
        Panel::bottom("bottom_dock")
            .resizable(false)
            .exact_size(BottomDockState::COLLAPSED_HEIGHT)
    } else {
        Panel::bottom("bottom_dock")
            .resizable(true)
            .default_size(height)
    };

    let response = panel.show(ui, |ui| {
        // Drop zone detection for bottom dock
        if ui.rect_contains_pointer(ui.max_rect())
            && ui.input(|i| i.pointer.any_released())
            && app.dock_layout.dragging_panel.is_some()
        {
            app.dock_layout.drop_dragged(DockSlotLocation::Bottom);
            app.dock_layout.bottom.collapsed = false;
        }

        // Header bar
        ui.horizontal(|ui| {
            let collapse_icon = if is_collapsed { "▲" } else { "▼" };
            if ui.button(collapse_icon).clicked() {
                app.dock_layout.bottom.toggle_collapsed();
            }

            let panels = app.dock_layout.bottom.panels.clone();
            let active = app.dock_layout.bottom.active.clone();

            for panel_id in panels {
                let is_selected = active.as_ref() == Some(&panel_id);
                let title = app
                    .registry
                    .find_panel(&panel_id)
                    .map(|p| p.title())
                    .unwrap_or_else(|| panel_id.as_str());

                let btn = ui.selectable_label(is_selected && !is_collapsed, title);
                if btn.drag_started() {
                    app.dock_layout.start_drag(panel_id.clone());
                }
                if btn.clicked() {
                    if is_selected {
                        // Clicking active tab toggles collapse
                        app.dock_layout.bottom.toggle_collapsed();
                    } else {
                        // Clicking inactive tab selects it and expands
                        app.dock_layout.bottom.active = Some(panel_id.clone());
                        app.dock_layout.bottom.collapsed = false;
                    }
                }

                // Context menu to move dock
                btn.context_menu(|ui| {
                    if ui.button("Move to Left Dock").clicked() {
                        app.dock_layout
                            .move_panel(&panel_id, DockSlotLocation::Left);
                        ui.close();
                    }
                    if ui.button("Move to Right Dock").clicked() {
                        app.dock_layout
                            .move_panel(&panel_id, DockSlotLocation::Right);
                        ui.close();
                    }
                });
            }
        });

        // Content (only if expanded)
        if !is_collapsed {
            ui.separator();
            if let Some(active_id) = app.dock_layout.bottom.active.clone() {
                if let Some(panel) = app.registry.find_panel_mut(&active_id) {
                    let mut cx = EditorCtx::new(
                        &app.project,
                        &app.undo_stack,
                        &app.session,
                        &mut app.events,
                        &mut app.pending_commands,
                        &*app.assets,
                    );
                    panel.ui(ui, &mut cx);
                }
            }
        }
    });

    // Update remembered expanded height
    if !is_collapsed {
        let actual_h = response.response.rect.height();
        if actual_h > BottomDockState::COLLAPSED_HEIGHT + 10.0 {
            app.dock_layout.bottom.height = actual_h;
        }
    }
}

/// Renders a side dock slot (Left or Right).
pub fn render_dock_slot(app: &mut ShellApp, ui: &mut Ui, location: DockSlotLocation) {
    let panels = match location {
        DockSlotLocation::Left => app.dock_layout.left.panels.clone(),
        DockSlotLocation::Right => app.dock_layout.right.panels.clone(),
        DockSlotLocation::Bottom => app.dock_layout.bottom.panels.clone(),
    };

    let active = match location {
        DockSlotLocation::Left => app.dock_layout.left.active.clone(),
        DockSlotLocation::Right => app.dock_layout.right.active.clone(),
        DockSlotLocation::Bottom => app.dock_layout.bottom.active.clone(),
    };

    // Drop zone detection for this dock slot
    if ui.rect_contains_pointer(ui.max_rect())
        && ui.input(|i| i.pointer.any_released())
        && app.dock_layout.dragging_panel.is_some()
    {
        app.dock_layout.drop_dragged(location);
    }

    // Tab strip
    ui.horizontal(|ui| {
        for panel_id in &panels {
            let is_selected = active.as_ref() == Some(panel_id);
            let title = app
                .registry
                .find_panel(panel_id)
                .map(|p| p.title())
                .unwrap_or_else(|| panel_id.as_str());

            let btn = ui.selectable_label(is_selected, title);
            if btn.drag_started() {
                app.dock_layout.start_drag(panel_id.clone());
            }
            if btn.clicked() {
                match location {
                    DockSlotLocation::Left => app.dock_layout.left.set_active(panel_id.clone()),
                    DockSlotLocation::Right => app.dock_layout.right.set_active(panel_id.clone()),
                    DockSlotLocation::Bottom => app.dock_layout.bottom.set_active(panel_id.clone()),
                }
            }

            // Context menu to move panel
            btn.context_menu(|ui| {
                if location != DockSlotLocation::Left && ui.button("Move to Left Dock").clicked() {
                    app.dock_layout.move_panel(panel_id, DockSlotLocation::Left);
                    ui.close();
                }
                if location != DockSlotLocation::Right && ui.button("Move to Right Dock").clicked()
                {
                    app.dock_layout
                        .move_panel(panel_id, DockSlotLocation::Right);
                    ui.close();
                }
                if location != DockSlotLocation::Bottom
                    && ui.button("Move to Bottom Dock").clicked()
                {
                    app.dock_layout
                        .move_panel(panel_id, DockSlotLocation::Bottom);
                    ui.close();
                }
            });
        }
    });

    ui.separator();

    // Re-query current active panel ID after tab clicks and context relocations
    let current_active = match location {
        DockSlotLocation::Left => app.dock_layout.left.active.clone(),
        DockSlotLocation::Right => app.dock_layout.right.active.clone(),
        DockSlotLocation::Bottom => app.dock_layout.bottom.active.clone(),
    };

    // Render active panel content
    if let Some(active_id) = current_active {
        if let Some(panel) = app.registry.find_panel_mut(&active_id) {
            let mut cx = EditorCtx::new(
                &app.project,
                &app.undo_stack,
                &app.session,
                &mut app.events,
                &mut app.pending_commands,
                &*app.assets,
            );
            panel.ui(ui, &mut cx);
        }
    }
}
