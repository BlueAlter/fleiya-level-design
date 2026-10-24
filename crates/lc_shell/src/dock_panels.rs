//! Panel dock rendering for side slots and collapsible bottom tray.

use egui::{CornerRadius, Id, Panel, Stroke, StrokeKind, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{chevron, dock_tab, dock_tab_sized};
use lc_ui_api::context::EditorCtx;

use crate::dock_layout::{BottomDockState, DockSlotLocation};
use crate::shell_app::ShellApp;

/// Renders the collapsible bottom dock panel in the center column.
pub fn render_bottom_dock(app: &mut ShellApp, ui: &mut Ui) {
    if app.dock_layout.bottom.panels.is_empty() {
        return;
    }

    let t = Tokens::get(ui.ctx());
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

    let response = panel
        .frame(
            egui::Frame::new()
                .fill(t.panel)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ui, |ui| {
            // Drop zone detection for bottom dock
            if ui.rect_contains_pointer(ui.max_rect())
                && ui.input(|i| i.pointer.any_released())
                && app.dock_layout.dragging_panel.is_some()
            {
                app.dock_layout.drop_dragged(DockSlotLocation::Bottom);
                app.dock_layout.bottom.collapsed = false;
            }

            // Header bar
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), BottomDockState::COLLAPSED_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.x = 0.0;

                    let chev = chevron(ui, !is_collapsed, t.muted);
                    if chev.clicked() {
                        app.dock_layout.bottom.toggle_collapsed();
                    }
                    ui.add_space(4.0);

                    let panels = app.dock_layout.bottom.panels.clone();
                    let active = app.dock_layout.bottom.active.clone();

                    for panel_id in panels {
                        let is_selected = active.as_ref() == Some(&panel_id);
                        let title = app
                            .registry
                            .find_panel(&panel_id)
                            .map(|p| p.title())
                            .unwrap_or_else(|| panel_id.as_str());

                        let btn = dock_tab(ui, title, is_selected && !is_collapsed);
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
                    ui.add_space(8.0);
                },
            );

            // Divider under the header.
            let divider = ui.min_rect().bottom() - 0.5;
            ui.painter()
                .hline(ui.max_rect().x_range(), divider, t.hairline());

            // Content (only if expanded)
            if !is_collapsed {
                ui.spacing_mut().item_spacing.y = 0.0;
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(theme::PANEL_PAD, 8))
                    .show(ui, |ui| {
                        ui.set_max_height(
                            (height - BottomDockState::COLLAPSED_HEIGHT - 16.0).max(0.0),
                        );
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
                    });
            }

            // Drop target highlight.
            if app.dock_layout.dragging_panel.is_some() && ui.rect_contains_pointer(ui.max_rect()) {
                ui.painter().rect_stroke(
                    ui.max_rect().shrink(1.0),
                    CornerRadius::ZERO,
                    Stroke::new(1.0, t.accent),
                    StrokeKind::Inside,
                );
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
    let t = Tokens::get(ui.ctx());
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
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), theme::TAB_H),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let tab_width = ((ui.available_width() - 16.0) / panels.len().max(1) as f32).max(60.0);
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing.x = 0.0;
            for panel_id in &panels {
                let is_selected = active.as_ref() == Some(panel_id);
                let title = app
                    .registry
                    .find_panel(panel_id)
                    .map(|p| p.title())
                    .unwrap_or_else(|| panel_id.as_str());

                let btn = dock_tab_sized(ui, title, is_selected, tab_width);
                if btn.drag_started() {
                    app.dock_layout.start_drag(panel_id.clone());
                }
                if btn.clicked() {
                    match location {
                        DockSlotLocation::Left => app.dock_layout.left.set_active(panel_id.clone()),
                        DockSlotLocation::Right => {
                            app.dock_layout.right.set_active(panel_id.clone())
                        }
                        DockSlotLocation::Bottom => {
                            app.dock_layout.bottom.set_active(panel_id.clone())
                        }
                    }
                }

                // Context menu to move panel
                btn.context_menu(|ui| {
                    if location != DockSlotLocation::Left
                        && ui.button("Move to Left Dock").clicked()
                    {
                        app.dock_layout.move_panel(panel_id, DockSlotLocation::Left);
                        ui.close();
                    }
                    if location != DockSlotLocation::Right
                        && ui.button("Move to Right Dock").clicked()
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
            ui.add_space(8.0);
        },
    );

    // Divider under the tab strip.
    let divider = ui.min_rect().bottom() - 0.5;
    ui.painter()
        .hline(ui.max_rect().x_range(), divider, t.hairline());

    // Re-query current active panel ID after tab clicks and context relocations
    let current_active = match location {
        DockSlotLocation::Left => app.dock_layout.left.active.clone(),
        DockSlotLocation::Right => app.dock_layout.right.active.clone(),
        DockSlotLocation::Bottom => app.dock_layout.bottom.active.clone(),
    };

    // Render active panel content
    if let Some(active_id) = current_active {
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(theme::PANEL_PAD, 8))
            .show(ui, |ui| {
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
            });
    }

    // Drop target highlight.
    if app.dock_layout.dragging_panel.is_some() && ui.rect_contains_pointer(ui.max_rect()) {
        ui.painter().rect_stroke(
            ui.max_rect().shrink(1.0),
            CornerRadius::ZERO,
            Stroke::new(1.0, t.accent),
            StrokeKind::Inside,
        );
    }
}
