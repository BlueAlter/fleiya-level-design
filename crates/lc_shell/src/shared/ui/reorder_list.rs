//! Standardized reorderable list component wired with drag-and-drop ghost previews.
//!
//! Provides [`reorder_list`] on top of [`super::drag_list`], managing row rendering,
//! live placeholder slots, follow-cursor ghost card previews, and pointer release commits.
//! Never mutates caller data directly; emits [`ListEvents`] for the caller to handle.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::reorder_list::{reorder_list, ListItem, ListEvents};
//! use lc_shell::shared::ui::reorder_row::{GhostPreview, ROW_LAYOUT_VALUES};
//!
//! let items = [
//!     ListItem::new("item_1", GhostPreview::new("First Item")),
//!     ListItem::new("item_2", GhostPreview::new("Second Item")),
//! ];
//! let events = reorder_list(ui, &mut drag_state, &items, ROW_LAYOUT_VALUES, |ui, idx, body| {
//!     body.slot(ui, 0, |ui| ui.label(&items[idx].ghost.label));
//! });
//! ```

use egui::{vec2, CornerRadius, Id, Pos2, Rect, Sense, Stroke, Ui};
use lc_canvas::icons::paint_icon;
use lc_canvas::theme::Tokens;

use super::drag_list::{
    draw_drag_handle, drop_index as shared_drop_index, paint_placeholder, DragListState,
    DragOutcome, DragTheme, GhostCardStyle, Slot as DragSlot,
};
use super::reorder::final_index;
use super::reorder_row::{
    reorder_row, GhostPreview, MoveDirection, ReorderRowLayout, ReorderRowSpec, RowBody,
    SelectDirection, Slot as RowSlot, Trailing,
};

/// Item descriptor for a row in a [`reorder_list`].
#[derive(Clone, Debug)]
pub struct ListItem {
    /// Unique identifier for this item's row interaction state.
    pub id: Id,
    /// Lightweight preview descriptor used for ghost rendering.
    pub ghost: GhostPreview,
    /// Slot layout for the row body. Defaults to a single flexible slot.
    pub slots: Vec<RowSlot>,
    /// Whether this row can be deleted. Defaults to `true`.
    pub deletable: bool,
    /// Tooltip explaining why deletion is disabled.
    pub delete_disabled_reason: Option<String>,
    /// Whether the row is selected.
    pub selected: bool,
    /// Whether the row is clickable.
    pub clickable: bool,
    /// Trailing action button kind (delete, kebab, or none).
    pub trailing: Trailing,
    /// Whether card rows paint a filled background (`true`) or outline-only (`false`).
    pub use_custom_color: bool,
}

impl ListItem {
    /// Creates a new [`ListItem`] with a single flexible body slot.
    pub fn new(id: impl egui::AsId, ghost: GhostPreview) -> Self {
        Self {
            id: Id::new(id),
            ghost,
            slots: vec![RowSlot::Flex],
            deletable: true,
            delete_disabled_reason: None,
            selected: false,
            clickable: false,
            trailing: Trailing::Delete {
                tooltip: Some("Delete".into()),
                disabled_reason: None,
            },
            use_custom_color: true,
        }
    }

    /// Sets the body slots layout for this item.
    pub fn with_slots(mut self, slots: impl IntoIterator<Item = RowSlot>) -> Self {
        self.slots = slots.into_iter().collect();
        self
    }

    /// Sets whether the item is selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets whether the item is clickable.
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Sets the trailing action button kind.
    pub fn trailing(mut self, trailing: Trailing) -> Self {
        self.trailing = trailing;
        self
    }

    /// Sets whether card rows paint a filled background (`true`) or outline-only (`false`).
    pub fn use_custom_color(mut self, use_custom_color: bool) -> Self {
        self.use_custom_color = use_custom_color;
        self
    }
}

/// Events emitted during a [`reorder_list`] frame.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ListEvents {
    /// An item was dropped to a new position `(from_index, to_index)`.
    pub moved: Option<(usize, usize)>,
    /// User clicked or pressed Enter on the item at index `usize`.
    pub selected: Option<usize>,
    /// User clicked delete on the item at index `usize`.
    pub delete_requested: Option<usize>,
    /// User requested reordering via keyboard shortcut `Alt+Up` or `Alt+Down`.
    pub move_requested: Option<(usize, MoveDirection)>,
    /// User requested selection navigation via `Up` or `Down`.
    pub select_requested: Option<(usize, SelectDirection)>,
    /// User requested context menu at position `Pos2` on item at index `usize`.
    pub context_menu_requested: Option<(usize, Pos2)>,
}

/// Evaluates the target drop index relative to row bounds.
pub fn calculate_drop_index(row_rects: &[Rect], pointer_y: f32, dragged_index: usize) -> usize {
    shared_drop_index(row_rects, pointer_y, dragged_index)
}

/// Renders a reorderable list of items with drag handles, placeholder slots, and follow-cursor ghost.
pub fn reorder_list<F>(
    ui: &mut Ui,
    state: &mut DragListState,
    items: &[ListItem],
    layout: ReorderRowLayout,
    mut row: F,
) -> ListEvents
where
    F: FnMut(&mut Ui, usize, &mut RowBody),
{
    let t = Tokens::get(ui.ctx());
    let drag_theme = DragTheme::from_accent(t.accent);

    // Ensure list pitch is independent of the surrounding theme item spacing
    ui.spacing_mut().item_spacing.y = 0.0;

    let mut slot_rects = Vec::with_capacity(items.len());
    let mut selected = None;
    let mut delete_requested = None;
    let mut move_requested = None;
    let mut select_requested = None;
    let mut context_menu_requested = None;

    let available_w = ui.available_width();
    let display_slots = state.layout(items.len());

    for slot in display_slots {
        match slot {
            DragSlot::Placeholder => {
                let ph_h = state.row_size().map_or(layout.row_gap.max(32.0), |s| s.y);
                let (ph_rect, _) = ui.allocate_exact_size(vec2(available_w, ph_h), Sense::hover());
                paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                slot_rects.push(ph_rect);
                ui.add_space(layout.row_gap);
            }
            DragSlot::Item(idx) => {
                let Some(item) = items.get(idx) else {
                    continue;
                };

                let mut spec = ReorderRowSpec::new(item.id, item.ghost.label.clone());
                spec.layout = layout;
                spec.outline = item.ghost.color;
                spec.use_custom_color = item.use_custom_color;
                spec.deletable = item.deletable;
                spec.delete_disabled_reason = item.delete_disabled_reason.clone();
                spec.ghost = item.ghost.clone();
                spec.selected = item.selected;
                spec.clickable = item.clickable;
                spec.trailing = item.trailing.clone();

                let slots = if item.slots.is_empty() {
                    &[RowSlot::Flex]
                } else {
                    item.slots.as_slice()
                };

                let row_resp = reorder_row(ui, spec, slots, |ui, body| {
                    row(ui, idx, body);
                });

                slot_rects.push(row_resp.rect);

                if row_resp.handle.drag_started() {
                    state.begin(ui.ctx(), idx, row_resp.rect);
                    ui.memory_mut(|m| {
                        if let Some(fid) = m.focused() {
                            m.surrender_focus(fid);
                        }
                        m.stop_text_input();
                    });
                }

                if row_resp.clicked && selected.is_none() {
                    selected = Some(idx);
                }

                if row_resp.delete_clicked && delete_requested.is_none() {
                    delete_requested = Some(idx);
                }

                if let Some(dir) = row_resp.move_request {
                    if move_requested.is_none() {
                        move_requested = Some((idx, dir));
                    }
                }

                if let Some(dir) = row_resp.select_request {
                    if select_requested.is_none() {
                        select_requested = Some((idx, dir));
                    }
                }

                if let Some(pos) = row_resp.menu_requested {
                    if context_menu_requested.is_none() {
                        context_menu_requested = Some((idx, pos));
                    }
                }

                ui.add_space(layout.row_gap);
            }
        }
    }

    // Update drag state, target calculations, cursor icon, and autoscroll
    state.update(ui, &slot_rects, &drag_theme);

    // Render follow-cursor ghost card if an item is currently being dragged
    if let Some(drag_idx) = state.dragging_index() {
        if let Some(item) = items.get(drag_idx) {
            let ghost_style =
                GhostCardStyle::new(t.panel, Stroke::new(1.5, t.accent), CornerRadius::same(4));
            let card_w = slot_rects.first().map_or(available_w, |r| r.width());

            state.show_ghost_card(
                ui,
                Id::new("reorder_list_ghost_card"),
                card_w,
                ghost_style,
                |ui, _from, rect| {
                    // 1. Drag handle
                    let handle_rect =
                        Rect::from_min_size(rect.min, vec2(layout.handle_w, rect.height()));
                    draw_drag_handle(ui.painter(), handle_rect, t.ink_strong);

                    let body_start_x = handle_rect.max.x + layout.handle_gap;
                    let mut cur_x = body_start_x;

                    // 2. Ghost preview optional icon
                    if let Some(icon) = item.ghost.icon {
                        let icon_rect = Rect::from_min_size(
                            Pos2::new(cur_x, rect.min.y),
                            vec2(16.0, rect.height()),
                        );
                        paint_icon(ui.painter(), icon_rect.center(), icon, 14.0, t.ink_strong);
                        cur_x += 16.0 + layout.slot_gap;
                    }

                    // 3. Ghost preview optional color badge
                    if let Some(color) = item.ghost.color {
                        let badge_rect = Rect::from_min_size(
                            Pos2::new(cur_x, rect.center().y - 6.0),
                            vec2(12.0, 12.0),
                        );
                        ui.painter()
                            .rect_filled(badge_rect, CornerRadius::same(2), color);
                        cur_x += 12.0 + layout.slot_gap;
                    }

                    // 4. Ghost preview label text
                    let text_pos = Pos2::new(cur_x, rect.center().y);
                    ui.painter().text(
                        text_pos,
                        egui::Align2::LEFT_CENTER,
                        &item.ghost.label,
                        egui::FontId::proportional(13.0),
                        t.ink_strong,
                    );
                },
            );
        }
    }

    // Resolve drag outcome on release or cancellation
    let moved = match state.finish(ui) {
        DragOutcome::Dropped { from, to } => {
            let final_to = final_index(from, to);
            if from != final_to {
                Some((from, final_to))
            } else {
                None
            }
        }
        _ => None,
    };

    ListEvents {
        moved,
        selected,
        delete_requested,
        move_requested,
        select_requested,
        context_menu_requested,
    }
}

#[cfg(test)]
mod tests {
    use super::super::workspace_metrics::ROW_LAYOUT_VALUES;
    use super::*;
    use egui::epaint::TextShape;
    use egui::{vec2, Color32, Context, FullOutput, Pos2, RawInput, Shape};
    use lc_canvas::icons::Icon;
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_calculate_drop_index_pure_logic() {
        let row_rects = [
            Rect::from_min_size(Pos2::new(0.0, 0.0), vec2(200.0, 32.0)),
            Rect::from_min_size(Pos2::new(0.0, 40.0), vec2(200.0, 32.0)),
            Rect::from_min_size(Pos2::new(0.0, 80.0), vec2(200.0, 32.0)),
        ];

        // Before first row -> 0
        assert_eq!(calculate_drop_index(&row_rects, -10.0, 1), 0);

        // Between first and second row -> 0 or 1
        assert_eq!(calculate_drop_index(&row_rects, 36.0, 2), 1);

        // After last row -> last index (2)
        assert_eq!(calculate_drop_index(&row_rects, 150.0, 0), 2);

        // Same position as dragged item -> unchanged
        assert_eq!(calculate_drop_index(&row_rects, 16.0, 0), 0);
        assert_eq!(calculate_drop_index(&row_rects, 56.0, 1), 1);
    }

    fn collect_texts(shapes: &[egui::epaint::ClippedShape], out: &mut Vec<TextShape>) {
        for s in shapes {
            match &s.shape {
                Shape::Text(t) => out.push(t.clone()),
                Shape::Vec(vec) => {
                    for inner in vec {
                        if let Shape::Text(t) = inner {
                            out.push(t.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn shape_contains_color(shape: &Shape, target: Color32) -> bool {
        match shape {
            Shape::Rect(r) => r.fill == target || r.stroke.color == target,
            Shape::Circle(c) => c.fill == target || c.stroke.color == target,
            Shape::Vec(vec) => vec.iter().any(|s| shape_contains_color(s, target)),
            _ => false,
        }
    }

    fn any_shape_contains_color(shapes: &[egui::epaint::ClippedShape], target: Color32) -> bool {
        shapes
            .iter()
            .any(|s| shape_contains_color(&s.shape, target))
    }

    #[test]
    fn test_reorder_list_drag_and_ghost_preview_headless() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let mut drag_state = DragListState::new();

        let items = vec![
            ListItem::new(
                "item_0",
                GhostPreview {
                    label: "Item Alpha".to_string(),
                    color: Some(Color32::from_rgb(255, 0, 0)),
                    icon: Some(Icon::Add),
                },
            ),
            ListItem::new("item_1", GhostPreview::new("Item Beta")),
            ListItem::new("item_2", GhostPreview::new("Item Gamma")),
        ];

        // Frame 1: Simulate drag beginning on row 0
        let rect0 = Rect::from_min_size(Pos2::new(10.0, 10.0), vec2(200.0, 32.0));
        drag_state.begin(&ctx, 0, rect0);
        assert!(drag_state.is_active());
        assert_eq!(drag_state.dragging_index(), Some(0));

        // Frame 2: Dragging active over row 2 (y around 100.0)
        let raw_input2 = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            events: vec![egui::Event::PointerMoved(Pos2::new(100.0, 100.0))],
            ..Default::default()
        };

        let mut output2_pass1: FullOutput = ctx.run_ui(raw_input2.clone(), |ui| {
            reorder_list(
                ui,
                &mut drag_state,
                &items,
                ROW_LAYOUT_VALUES,
                |ui, idx, body| {
                    body.slot(ui, 0, |ui| ui.label(&items[idx].ghost.label));
                },
            );
        });
        output2_pass1.textures_delta.clear();

        let mut output2: FullOutput = ctx.run_ui(raw_input2, |ui| {
            let events = reorder_list(
                ui,
                &mut drag_state,
                &items,
                ROW_LAYOUT_VALUES,
                |ui, idx, body| {
                    body.slot(ui, 0, |ui| ui.label(&items[idx].ghost.label));
                },
            );
            // Still dragging, so not yet dropped
            assert_eq!(events.moved, None);
        });

        // Verify GhostPreview text reached the painted ghost shapes
        let mut texts = Vec::new();
        collect_texts(&output2.shapes, &mut texts);
        let ghost_text_exists = texts.iter().any(|t| t.galley.text().contains("Item Alpha"));
        assert!(
            ghost_text_exists,
            "GhostPreview label 'Item Alpha' must exist in painted output shapes"
        );
        let target_color = Color32::from_rgb(255, 0, 0);
        assert!(
            any_shape_contains_color(&output2.shapes, target_color),
            "GhostPreview color {target_color:?} must exist in painted output shapes"
        );
        output2.textures_delta.clear();

        // Frame 3: Pointer released below row 2 -> yields moved == Some((0, 2))
        let raw_input3 = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            events: vec![
                egui::Event::PointerMoved(Pos2::new(100.0, 100.0)),
                egui::Event::PointerButton {
                    pos: Pos2::new(100.0, 100.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        };

        let mut output3: FullOutput = ctx.run_ui(raw_input3, |ui| {
            let events = reorder_list(
                ui,
                &mut drag_state,
                &items,
                ROW_LAYOUT_VALUES,
                |ui, idx, body| {
                    body.slot(ui, 0, |ui| ui.label(&items[idx].ghost.label));
                },
            );
            assert_eq!(events.moved, Some((0, 2)));
        });

        output3.textures_delta.clear();
    }
}
