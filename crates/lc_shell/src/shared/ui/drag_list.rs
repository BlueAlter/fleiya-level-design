//! Reusable egui component for drag-to-reorder lists with a live ghost drop slot.
//!
//! While an item is dragged, the original row is not drawn. Instead a dashed
//! **placeholder slot** (the ghost drop slot) follows the pointer through the list
//! and the remaining rows shift to make room, so the order the user sees is exactly
//! the order that results from dropping.
//!
//! # Frame flow
//! 1. `let slots = state.layout(len);` and draw the list following `slots`
//!    (`Slot::Item(i)` -> the row for original index `i`, `Slot::Placeholder` ->
//!    [`paint_placeholder`] in a rect of the dragged row's size).
//! 2. When a row's drag handle reports `drag_started()`, call [`DragListState::begin`].
//! 3. After the list is drawn, call [`DragListState::update`] with the rects of all
//!    displayed slots (in display order), then [`DragListState::finish`]; apply a
//!    returned [`DragOutcome::Dropped`] with [`super::reorder::reorder_item`].
//!
//! The component owns no row painting: callers draw rows and the floating ghost card
//! themselves, using [`DragListState::dragging_index`] and [`DragListState::grab_offset`].

use egui::emath::GuiRounding;
use egui::{
    Color32, Context, CursorIcon, Key, Modifiers, Painter, Rect, Shape, Stroke, StrokeKind, Ui,
    Vec2,
};

use super::reorder::{auto_scroll_delta, preview_layout, relative_to_top, snapshot_target, Slot};

/// Outline style of the ghost drop slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaceholderStyle {
    /// Continuous outline snapped to whole pixels. Crisp on every display scale.
    #[default]
    Solid,
    /// Dashed outline, dashed independently per edge so no dash wraps a corner.
    Dashed,
}

/// Visual and behavioural constants for a drag-to-reorder list.
#[derive(Debug, Clone, Copy)]
pub struct DragTheme {
    /// Accent colour of the placeholder outline.
    pub accent: Color32,
    /// Outline style of the placeholder slot.
    pub placeholder_style: PlaceholderStyle,
    /// Translucent fill of the placeholder slot.
    pub placeholder_fill: Color32,
    /// Stroke width of the dashed placeholder outline.
    pub stroke_width: f32,
    /// Length of one dash.
    pub dash_len: f32,
    /// Gap between dashes.
    pub gap_len: f32,
    /// Distance from the scroll area edge that triggers auto-scroll.
    pub auto_scroll_edge: f32,
    /// Auto-scroll step per frame. `0.0` disables auto-scroll.
    pub auto_scroll_speed: f32,
}

impl DragTheme {
    /// Builds the default theme from the application's accent colour.
    pub fn from_accent(accent: Color32) -> Self {
        Self {
            accent,
            placeholder_style: PlaceholderStyle::Solid,
            placeholder_fill: Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                28,
            ),
            stroke_width: 1.5,
            dash_len: 6.0,
            gap_len: 4.0,
            auto_scroll_edge: 24.0,
            auto_scroll_speed: 8.0,
        }
    }
}

/// Result of [`DragListState::finish`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragOutcome {
    /// Nothing happened this frame (no drag, or the drag is still running).
    None,
    /// The item at `from` was dropped on insertion index `to`.
    ///
    /// Both are in the original order; feed them to [`super::reorder::reorder_item`].
    Dropped {
        /// Original index of the dragged item.
        from: usize,
        /// Insertion index in the original order.
        to: usize,
    },
    /// The drag was cancelled (Escape) and the order must stay unchanged.
    Cancelled,
}

#[derive(Debug, Clone)]
struct ActiveDrag {
    from: usize,
    row_size: Vec2,
    grab_offset: Vec2,
    target: usize,
    /// Row rects relative to the list top, captured once at the start of the drag.
    snapshot: Option<Vec<Rect>>,
}

/// Drag state of one reorderable list. Keep one instance per list.
#[derive(Debug, Clone, Default)]
pub struct DragListState {
    active: Option<ActiveDrag>,
}

impl DragListState {
    /// Creates an idle state.
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` while an item is being dragged.
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Index (in the original order) of the item being dragged.
    pub fn dragging_index(&self) -> Option<usize> {
        self.active.as_ref().map(|a| a.from)
    }

    /// Current insertion index the item would be dropped at.
    pub fn target_index(&self) -> Option<usize> {
        self.active.as_ref().map(|a| a.target)
    }

    /// Offset between the pointer and the top-left of the dragged row at grab time.
    pub fn grab_offset(&self) -> Vec2 {
        self.active.as_ref().map_or(Vec2::ZERO, |a| a.grab_offset)
    }

    /// Size of the dragged row (also the size of the placeholder slot).
    pub fn row_size(&self) -> Option<Vec2> {
        self.active.as_ref().map(|a| a.row_size)
    }

    /// Starts dragging the item at original index `from`, whose row occupies `row_rect`.
    ///
    /// Ignored if a drag is already running.
    pub fn begin(&mut self, ctx: &Context, from: usize, row_rect: Rect) {
        if self.active.is_some() {
            return;
        }
        let pointer = ctx
            .input(|i| i.pointer.interact_pos().or_else(|| i.pointer.hover_pos()))
            .unwrap_or(row_rect.min);
        self.active = Some(ActiveDrag {
            from,
            row_size: row_rect.size(),
            grab_offset: pointer - row_rect.min,
            target: from,
            snapshot: None,
        });
        egui::DragAndDrop::set_payload(ctx, from);
    }

    /// Display order for this frame.
    ///
    /// The normal order is returned until the geometry snapshot has been captured
    /// (the first frame of a drag), so the snapshot always reflects the unshifted list.
    pub fn layout(&self, len: usize) -> Vec<Slot> {
        match &self.active {
            Some(a) if a.snapshot.is_some() => preview_layout(len, a.from, a.target),
            _ => (0..len).map(Slot::Item).collect(),
        }
    }

    /// Updates the drop target, cursor and auto-scroll. Call once after drawing the list.
    ///
    /// `rects` are the rects of every displayed slot, in display order (including
    /// the placeholder's rect when one was drawn).
    pub fn update(&mut self, ui: &Ui, rects: &[Rect], theme: &DragTheme) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

        if active.snapshot.is_none() && !rects.is_empty() {
            active.snapshot = Some(relative_to_top(rects));
        }

        let pointer = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()));
        let (Some(pointer), Some(first), Some(snapshot)) =
            (pointer, rects.first(), active.snapshot.as_ref())
        else {
            return;
        };

        active.target = snapshot_target(pointer.y - first.min.y, snapshot);

        if theme.auto_scroll_speed > 0.0 {
            let delta = auto_scroll_delta(
                pointer,
                ui.clip_rect(),
                theme.auto_scroll_edge,
                theme.auto_scroll_speed,
            );
            if delta != 0.0 {
                ui.scroll_with_delta(Vec2::new(0.0, delta));
                ui.ctx().request_repaint();
            }
        }
    }

    /// Resolves the drag: Escape cancels, releasing the pointer drops.
    pub fn finish(&mut self, ui: &Ui) -> DragOutcome {
        if self.active.is_none() {
            return DragOutcome::None;
        }
        let released = ui.input(|i| i.pointer.any_released());
        // egui's own DragAndDrop plugin consumes Escape at the start of the pass whenever a
        // payload exists and clears that payload. So Escape is detected both directly (in
        // case the key is still unconsumed) and by the payload having vanished while the
        // pointer is still held down.
        let escape = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        let payload_vanished = !egui::DragAndDrop::has_any_payload(ui.ctx()) && !released;
        if escape || payload_vanished {
            self.cancel(ui.ctx());
            return DragOutcome::Cancelled;
        }
        if released {
            let active = self.active.take();
            egui::DragAndDrop::clear_payload(ui.ctx());
            return match active {
                Some(a) if a.snapshot.is_some() => DragOutcome::Dropped {
                    from: a.from,
                    to: a.target,
                },
                _ => DragOutcome::Cancelled,
            };
        }
        DragOutcome::None
    }

    /// Aborts any running drag without producing a result.
    pub fn cancel(&mut self, ctx: &Context) {
        if self.active.take().is_some() {
            egui::DragAndDrop::clear_payload(ctx);
        }
    }
}

/// Paints the ghost drop slot: a translucent fill with an accent outline.
///
/// The rect is snapped to whole physical pixels so the outline is never anti-aliased
/// across two pixel rows (which makes thin lines look wavy or slanted).
pub fn paint_placeholder(painter: &Painter, rect: Rect, theme: &DragTheme) {
    let rect = rect.round_to_pixels(painter.pixels_per_point());
    painter.rect_filled(rect, 0.0, theme.placeholder_fill);
    let stroke = Stroke::new(theme.stroke_width, theme.accent.linear_multiply(0.75));
    match theme.placeholder_style {
        PlaceholderStyle::Solid => {
            painter.rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
        }
        PlaceholderStyle::Dashed => {
            let r = rect.shrink(theme.stroke_width * 0.5);
            let edges = [
                [r.left_top(), r.right_top()],
                [r.right_top(), r.right_bottom()],
                [r.right_bottom(), r.left_bottom()],
                [r.left_bottom(), r.left_top()],
            ];
            for edge in edges {
                painter.extend(Shape::dashed_line(
                    &edge,
                    stroke,
                    theme.dash_len,
                    theme.gap_len,
                ));
            }
        }
    }
}
