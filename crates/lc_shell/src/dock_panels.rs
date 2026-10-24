//! Panel dock rendering for side slots and collapsible bottom tray.

use egui::{CornerRadius, Id, LayerId, Order, Panel, Pos2, Rect, Stroke, StrokeKind, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{chevron, dock_tab, dock_tab_sized};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::view::PanelId;

use crate::dock_layout::{BottomDockState, DockSlotLocation};
use crate::shell_app::ShellApp;

/// Geometry of one custom dock slot for drag/drop hit-testing.
#[derive(Debug, Clone)]
pub struct DockSlotGeometry {
    /// Full outer rect of the slot.
    pub slot_rect: Rect,
    /// Rect of the horizontal tab strip.
    pub tab_strip_rect: Rect,
    /// Panel id and rect for each tab, in display order.
    pub tab_rects: Vec<(PanelId, Rect)>,
}

impl Default for DockSlotGeometry {
    fn default() -> Self {
        Self {
            slot_rect: Rect::NOTHING,
            tab_strip_rect: Rect::NOTHING,
            tab_rects: Vec::new(),
        }
    }
}

/// Geometry of all custom dock slots for the current frame.
#[derive(Debug, Clone, Default)]
pub struct DockGeometry {
    /// Left dock geometry, if visible.
    pub left: Option<DockSlotGeometry>,
    /// Right dock geometry, if visible.
    pub right: Option<DockSlotGeometry>,
    /// Bottom dock geometry, if visible.
    pub bottom: Option<DockSlotGeometry>,
}

impl DockGeometry {
    fn slot(&self, location: DockSlotLocation) -> Option<&DockSlotGeometry> {
        match location {
            DockSlotLocation::Left => self.left.as_ref(),
            DockSlotLocation::Right => self.right.as_ref(),
            DockSlotLocation::Bottom => self.bottom.as_ref(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = (DockSlotLocation, &DockSlotGeometry)> {
        [
            (DockSlotLocation::Left, self.left.as_ref()),
            (DockSlotLocation::Right, self.right.as_ref()),
            (DockSlotLocation::Bottom, self.bottom.as_ref()),
        ]
        .into_iter()
        .filter_map(|(location, geometry)| geometry.map(|geometry| (location, geometry)))
    }
}

/// Renders the collapsible bottom dock panel in the center column.
pub fn render_bottom_dock(app: &mut ShellApp, ui: &mut Ui) -> Option<DockSlotGeometry> {
    if app.dock_layout.bottom.panels.is_empty() {
        return None;
    }

    let t = Tokens::get(ui.ctx());
    let is_collapsed = app.dock_layout.bottom.collapsed;
    let available_height = ui.available_height();
    let max_height = BottomDockState::max_expanded_height(available_height);
    let min_height = BottomDockState::MIN_EXPANDED_HEIGHT.min(max_height);
    let desired_height = app
        .dock_layout
        .bottom
        .desired_expanded_height(available_height);

    let bar_id = Id::new("bottom_dock");
    if !is_collapsed {
        // `PanelState` is a rendering cache. It can be stale after a collapse
        // or after the window was resized, so normalise it before showing the
        // panel instead of trusting an old full-height rect.
        if let Some(mut state) = egui::panel::PanelState::load(ui.ctx(), bar_id) {
            let stored_height = state.outer_rect.height();
            let collapsed = BottomDockState::COLLAPSED_HEIGHT + 2.0;
            if stored_height < collapsed || stored_height > max_height + 0.5 {
                state.outer_rect.min.y = state.outer_rect.max.y - desired_height;
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
            .default_size(desired_height)
            .min_size(min_height)
            .max_size(max_height)
    };

    let response = panel
        .frame(
            egui::Frame::new()
                .fill(t.panel)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ui, |ui| {
            let panel_height = ui.max_rect().height();
            // Header bar
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut tab_rects = Vec::new();
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
                        tab_rects.push((panel_id.clone(), btn.rect));
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
                        // Use the panel's actual height rather than the
                        // remembered height so the console can grow while the
                        // user drags the divider.
                        ui.set_max_height(
                            (panel_height - BottomDockState::COLLAPSED_HEIGHT - 16.0).max(0.0),
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

            tab_rects
        });

    let slot_rect = response.response.rect;
    let tab_strip_rect = Rect::from_min_size(
        slot_rect.min,
        egui::vec2(slot_rect.width(), BottomDockState::COLLAPSED_HEIGHT),
    );

    // Update remembered expanded height, clamped to the current window.
    if !is_collapsed {
        let actual_height = response.response.rect.height();
        app.dock_layout.bottom.height = actual_height.clamp(min_height, max_height);
    }

    Some(DockSlotGeometry {
        slot_rect,
        tab_strip_rect,
        tab_rects: response.inner,
    })
}

/// Renders a side dock slot (Left or Right).
pub fn render_dock_slot(
    app: &mut ShellApp,
    ui: &mut Ui,
    location: DockSlotLocation,
) -> DockSlotGeometry {
    let t = Tokens::get(ui.ctx());
    let slot_rect = ui.max_rect();
    let tab_strip_rect =
        Rect::from_min_size(slot_rect.min, egui::vec2(slot_rect.width(), theme::TAB_H));
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

    // Tab strip
    let tab_rects = ui
        .allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::TAB_H),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let mut tab_rects = Vec::new();
                let tab_width =
                    ((ui.available_width() - 16.0) / panels.len().max(1) as f32).max(60.0);
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
                    tab_rects.push((panel_id.clone(), btn.rect));
                    if btn.drag_started() {
                        app.dock_layout.start_drag(panel_id.clone());
                    }
                    if btn.clicked() {
                        match location {
                            DockSlotLocation::Left => {
                                app.dock_layout.left.set_active(panel_id.clone())
                            }
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
                tab_rects
            },
        )
        .inner;

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

    DockSlotGeometry {
        slot_rect,
        tab_strip_rect,
        tab_rects,
    }
}

/// Handles the drag preview, the drop indicator and the final drop.
pub fn handle_drag_and_drop(app: &mut ShellApp, ui: &mut Ui, geometry: &DockGeometry) {
    let Some(panel_id) = app.dock_layout.dragging_panel.clone() else {
        return;
    };

    let ctx = ui.ctx();
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.dock_layout.cancel_drag();
        return;
    }

    let Some(pointer) = ctx.pointer_interact_pos() else {
        if ctx.input(|i| i.pointer.any_released()) {
            app.dock_layout.cancel_drag();
        }
        return;
    };

    let target = drop_target(geometry, pointer);
    paint_drag_ghost(ui, &panel_id, pointer, app);

    if let Some((location, index)) = target {
        if let Some(slot) = geometry.slot(location) {
            paint_drop_indicator(ui, slot, index);
        }
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
    } else {
        ctx.set_cursor_icon(egui::CursorIcon::NoDrop);
    }

    if ctx.input(|i| i.pointer.any_released()) {
        if let Some((location, index)) = target {
            app.dock_layout.drop_dragged_at(location, index);
            app.status_message = format!("Moved panel to {location:?} dock");
        } else {
            app.dock_layout.cancel_drag();
        }
    }
}

fn drop_target(geometry: &DockGeometry, pointer: Pos2) -> Option<(DockSlotLocation, usize)> {
    geometry.iter().find_map(|(location, slot)| {
        slot.slot_rect
            .contains(pointer)
            .then(|| (location, insertion_index(slot, pointer)))
    })
}

fn insertion_index(slot: &DockSlotGeometry, pointer: Pos2) -> usize {
    if !slot.tab_strip_rect.y_range().contains(pointer.y) {
        return slot.tab_rects.len();
    }

    for (index, (_, rect)) in slot.tab_rects.iter().enumerate() {
        if pointer.x < rect.center().x {
            return index;
        }
    }

    slot.tab_rects.len()
}

fn paint_drag_ghost(ui: &Ui, panel_id: &PanelId, pointer: Pos2, app: &ShellApp) {
    let t = Tokens::get(ui.ctx());
    let title = app
        .registry
        .find_panel(panel_id)
        .map(|panel| panel.title())
        .unwrap_or_else(|| panel_id.as_str());

    egui::Area::new(Id::new("dock_drag_ghost"))
        .order(Order::Tooltip)
        .fixed_pos(pointer + egui::vec2(14.0, 14.0))
        .interactable(false)
        .constrain_to(ui.ctx().content_rect())
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.accent))
                .corner_radius(CornerRadius::ZERO)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.set_max_width(220.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(title.to_uppercase())
                                .font(egui::FontId::new(
                                    theme::HUD_SIZE,
                                    theme::mono_for(ui.ctx()),
                                ))
                                .color(t.ink_strong),
                        )
                        .truncate(),
                    );
                });
        });
}

fn paint_drop_indicator(ui: &Ui, slot: &DockSlotGeometry, index: usize) {
    let t = Tokens::get(ui.ctx());
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Foreground,
        Id::new("dock_drop_indicator"),
    ));

    painter.rect(
        slot.slot_rect.shrink(1.0),
        CornerRadius::ZERO,
        theme::tint(t.accent, 0.08),
        Stroke::new(1.0, t.accent),
        StrokeKind::Inside,
    );

    let x = if slot.tab_rects.is_empty() {
        slot.tab_strip_rect.left() + 8.0
    } else if index >= slot.tab_rects.len() {
        slot.tab_rects
            .last()
            .map(|(_, rect)| rect.right() + 2.0)
            .unwrap_or(slot.tab_strip_rect.right() - 8.0)
    } else {
        slot.tab_rects[index].1.left()
    };
    let y0 = slot.tab_strip_rect.top() + 3.0;
    let y1 = slot.tab_strip_rect.bottom() - 3.0;
    let stroke = Stroke::new(2.0, t.accent);
    painter.line_segment([Pos2::new(x, y0), Pos2::new(x, y1)], stroke);
    // Small end caps make the insertion point readable even when it lands on
    // a tab boundary or the right edge of the strip.
    painter.line_segment([Pos2::new(x - 3.0, y0), Pos2::new(x + 3.0, y0)], stroke);
    painter.line_segment([Pos2::new(x - 3.0, y1), Pos2::new(x + 3.0, y1)], stroke);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn slot() -> DockSlotGeometry {
        DockSlotGeometry {
            slot_rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 200.0)),
            tab_strip_rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 28.0)),
            tab_rects: vec![
                (
                    PanelId::new("a"),
                    Rect::from_min_size(pos2(8.0, 0.0), vec2(90.0, 28.0)),
                ),
                (
                    PanelId::new("b"),
                    Rect::from_min_size(pos2(98.0, 0.0), vec2(90.0, 28.0)),
                ),
                (
                    PanelId::new("c"),
                    Rect::from_min_size(pos2(188.0, 0.0), vec2(90.0, 28.0)),
                ),
            ],
        }
    }

    #[test]
    fn insertion_index_uses_tab_centres_and_appends_outside_the_strip() {
        let slot = slot();
        assert_eq!(insertion_index(&slot, pos2(50.0, 10.0)), 0);
        assert_eq!(insertion_index(&slot, pos2(100.0, 10.0)), 1);
        assert_eq!(insertion_index(&slot, pos2(150.0, 10.0)), 2);
        assert_eq!(insertion_index(&slot, pos2(250.0, 10.0)), 3);
        assert_eq!(insertion_index(&slot, pos2(290.0, 10.0)), 3);
        assert_eq!(insertion_index(&slot, pos2(150.0, 100.0)), 3);
    }

    #[test]
    fn drop_target_finds_the_hovered_slot_and_rejects_the_central_area() {
        let geometry = DockGeometry {
            left: Some(slot()),
            right: None,
            bottom: None,
        };

        assert_eq!(
            drop_target(&geometry, pos2(150.0, 100.0)),
            Some((DockSlotLocation::Left, 3))
        );
        assert_eq!(drop_target(&geometry, pos2(500.0, 100.0)), None);
    }
}
