//! Standardized reorderable row component with handle, body slots, and pinned delete button.
//!
//! Provides [`reorder_row`] layout: `[handle][body slots...][delete]`.
//! Handles keyboard reordering (`Alt+Up` / `Alt+Down`), drag handle interaction,
//! strict slot dimension clipping, and deletion states.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::reorder_row::{reorder_row, ReorderRowSpec, Slot};
//!
//! let spec = ReorderRowSpec::new(ui.id().with("item_1"), "Item 1");
//! let slots = [Slot::Fixed(32.0), Slot::Flex];
//! let resp = reorder_row(ui, spec, &slots, |ui, body| {
//!     body.slot(ui, 0, |ui| ui.label("Icon"));
//!     body.slot(ui, 1, |ui| ui.text_edit_singleline(&mut name));
//! });
//! ```

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Id, Key, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::icon_button::{icon_button, IconButtonStyle};
pub use super::workspace_metrics::{
    ReorderRowLayout, CONTROL_HEIGHT_FORM, ICON_BUTTON_SIZE_DELETE, ROW_LAYOUT_CARD,
    ROW_LAYOUT_VALUES,
};

/// Direction for keyboard-driven reordering requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    /// Move row upwards (triggered by `Alt+ArrowUp`).
    Up,
    /// Move row downwards (triggered by `Alt+ArrowDown`).
    Down,
}

/// Drag handle capability flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragCapability {
    /// Dragging is enabled with grab cursor.
    Enabled,
    /// Dragging is disabled.
    Disabled,
}

/// Lightweight drag preview descriptor for ghost painting.
#[derive(Clone, Debug, PartialEq)]
pub struct GhostPreview {
    /// Display label for the dragged item.
    pub label: String,
    /// Accent or badge color.
    pub color: Option<Color32>,
    /// Optional leading icon glyph.
    pub icon: Option<Icon>,
}

impl GhostPreview {
    /// Creates a new ghost preview.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
            icon: None,
        }
    }
}

/// Specification for rendering a [`reorder_row`].
#[derive(Clone, Debug)]
pub struct ReorderRowSpec {
    /// Unique identifier for egui interaction state.
    pub id: Id,
    /// Height of the entire row in pixels.
    pub height: f32,
    /// Optional border outline color (e.g. value badge color).
    pub outline: Option<Color32>,
    /// Whether dragging via the handle is enabled.
    pub drag: DragCapability,
    /// Whether deletion is currently allowed.
    pub deletable: bool,
    /// Tooltip shown on the active delete button.
    pub delete_tooltip: Option<String>,
    /// Tooltip explaining why deletion is disabled.
    pub delete_disabled_reason: Option<String>,
    /// Row layout parameters (handle width, slot gap, row gap).
    pub layout: ReorderRowLayout,
    /// Lightweight preview for drag list integration.
    pub ghost: GhostPreview,
}

impl ReorderRowSpec {
    /// Creates a default reorder row specification.
    pub fn new(id: impl egui::AsId, label: impl Into<String>) -> Self {
        Self {
            id: Id::new(id),
            height: CONTROL_HEIGHT_FORM,
            outline: None,
            drag: DragCapability::Enabled,
            deletable: true,
            delete_tooltip: Some("Delete".to_string()),
            delete_disabled_reason: None,
            layout: ROW_LAYOUT_VALUES,
            ghost: GhostPreview::new(label),
        }
    }
}

/// Sizing mode for body slots in a [`reorder_row`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    /// Exact fixed width in pixels.
    Fixed(f32),
    /// Flexible slot that consumes proportional remaining width.
    Flex,
}

/// Computes horizontal bounds for slots within an allocated total width.
///
/// Flexible slots take the remaining width (`.max(0.0)` so it is never negative).
pub fn split_slots(total_width: f32, slots: &[Slot], gap: f32) -> Vec<Rect> {
    if slots.is_empty() {
        return Vec::new();
    }

    let gaps_total = (slots.len().saturating_sub(1)) as f32 * gap;
    let mut fixed_total = 0.0;
    let mut flex_count = 0usize;

    for &s in slots {
        match s {
            Slot::Fixed(w) => fixed_total += w,
            Slot::Flex => flex_count += 1,
        }
    }

    let remaining = (total_width - fixed_total - gaps_total).max(0.0);
    let flex_width = if flex_count > 0 {
        remaining / flex_count as f32
    } else {
        0.0
    };

    let mut result = Vec::with_capacity(slots.len());
    let mut cur_x = 0.0;

    for &s in slots {
        let w = match s {
            Slot::Fixed(fw) => fw,
            Slot::Flex => flex_width,
        };
        result.push(Rect::from_min_size(Pos2::new(cur_x, 0.0), vec2(w, 0.0)));
        cur_x += w + gap;
    }

    result
}

/// Scope passed to the row body closure to place widgets in strict slot bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct RowBody {
    slot_rects: Vec<Rect>,
}

impl RowBody {
    /// Creates a new [`RowBody`] from calculated slot rectangles.
    pub fn new(slot_rects: Vec<Rect>) -> Self {
        Self { slot_rects }
    }

    /// Number of configured slots in this row.
    pub fn len(&self) -> usize {
        self.slot_rects.len()
    }

    /// Whether there are no slots in this row.
    pub fn is_empty(&self) -> bool {
        self.slot_rects.is_empty()
    }

    /// Rectangle allocated for slot `index`.
    pub fn slot_rect(&self, index: usize) -> Option<Rect> {
        self.slot_rects.get(index).copied()
    }

    /// Runs a closure inside a child UI strictly clipped and bounded to slot `index`.
    pub fn slot<R>(&self, ui: &mut Ui, index: usize, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if let Some(&rect) = self.slot_rects.get(index) {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
            child.set_clip_rect(rect);
            child.set_max_width(rect.width());
            Some(add(&mut child))
        } else {
            None
        }
    }
}

/// Result returned from [`reorder_row`].
pub struct ReorderRowResponse<R> {
    /// Result value returned by the body closure.
    pub body: R,
    /// Drag handle interaction response.
    pub handle: Response,
    /// Whether the delete button was clicked this frame.
    pub delete_clicked: bool,
    /// Full bounding rectangle of the row.
    pub rect: Rect,
    /// Keyboard reorder request via `Alt+Up` or `Alt+Down`.
    pub move_request: Option<MoveDirection>,
}

/// Renders a row with layout `[handle][body slots...][delete]` and strict sizing contracts.
pub fn reorder_row<R>(
    ui: &mut Ui,
    spec: ReorderRowSpec,
    slots: &[Slot],
    body: impl FnOnce(&mut Ui, &mut RowBody) -> R,
) -> ReorderRowResponse<R> {
    let t = Tokens::get(ui.ctx());
    let available_w = ui.available_width();
    let (row_rect, row_resp) =
        ui.allocate_exact_size(vec2(available_w, spec.height), Sense::click());

    // 1. Paint row background and outline
    let is_hovered = row_resp.hovered();
    let bg = if is_hovered { t.sunken } else { t.panel };
    let border_color = if let Some(outline_c) = spec.outline {
        outline_c
    } else if is_hovered {
        t.line_strong
    } else {
        t.line
    };

    let painter = ui.painter();
    painter.rect_filled(row_rect, CornerRadius::same(3), bg);
    painter.rect_stroke(
        row_rect,
        CornerRadius::same(3),
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );

    // 2. Drag handle slot on the left
    let handle_rect = Rect::from_min_size(row_rect.min, vec2(spec.layout.handle_w, spec.height));
    let handle_resp = ui
        .interact(
            handle_rect,
            spec.id.with("handle"),
            if spec.drag == DragCapability::Enabled {
                Sense::drag()
            } else {
                Sense::hover()
            },
        )
        .on_hover_cursor(if spec.drag == DragCapability::Enabled {
            CursorIcon::Grab
        } else {
            CursorIcon::Default
        });

    let handle_color = if handle_resp.hovered() {
        t.ink_strong
    } else {
        t.muted
    };
    paint_icon(
        painter,
        handle_rect.center(),
        Icon::Drag,
        14.0,
        handle_color,
    );

    // 3. Delete button slot pinned to the right (icon_button at FORM size)
    let delete_rect = Rect::from_min_size(
        Pos2::new(row_rect.max.x - ICON_BUTTON_SIZE_DELETE, row_rect.min.y),
        vec2(ICON_BUTTON_SIZE_DELETE, spec.height),
    );

    let mut delete_clicked = false;
    let mut del_child = ui.new_child(egui::UiBuilder::new().max_rect(delete_rect));
    if spec.deletable {
        let del_resp = icon_button(
            &mut del_child,
            Icon::Delete,
            ICON_BUTTON_SIZE_DELETE,
            IconButtonStyle::ghost_danger(),
        );
        if let Some(ref tip) = spec.delete_tooltip {
            del_resp.clone().on_hover_text(tip);
        }
        if del_resp.clicked() {
            delete_clicked = true;
        }
    } else {
        let disabled_resp = icon_button(
            &mut del_child,
            Icon::Delete,
            ICON_BUTTON_SIZE_DELETE,
            IconButtonStyle::ghost(),
        );
        if let Some(ref reason) = spec.delete_disabled_reason {
            disabled_resp.on_hover_text(reason);
        }
    }

    // 4. Body slots in the remaining middle region
    let body_start_x = handle_rect.max.x + spec.layout.slot_gap;
    let body_end_x = delete_rect.min.x - spec.layout.slot_gap;
    let body_w = (body_end_x - body_start_x).max(0.0);

    let relative_rects = split_slots(body_w, slots, spec.layout.slot_gap);
    let absolute_rects: Vec<Rect> = relative_rects
        .into_iter()
        .map(|r| {
            Rect::from_min_size(
                Pos2::new(body_start_x + r.min.x, row_rect.min.y),
                vec2(r.width(), spec.height),
            )
        })
        .collect();

    let mut body_scope = RowBody {
        slot_rects: absolute_rects,
    };
    let body_res = body(ui, &mut body_scope);

    // 5. Alt+Up / Alt+Down move request
    let mut move_request = None;
    if row_resp.hovered() || ui.memory(|m| m.has_focus(spec.id)) {
        if ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowUp)) {
            move_request = Some(MoveDirection::Up);
        } else if ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowDown)) {
            move_request = Some(MoveDirection::Down);
        }
    }

    ReorderRowResponse {
        body: body_res,
        handle: handle_resp,
        delete_clicked,
        rect: row_rect,
        move_request,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Rect;

    #[test]
    fn test_split_slots_zero_slots() {
        assert_eq!(split_slots(100.0, &[], 6.0), Vec::<Rect>::new());
    }

    #[test]
    fn test_split_slots_one_flex() {
        let res = split_slots(100.0, &[Slot::Flex], 6.0);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].width(), 100.0);
    }

    #[test]
    fn test_split_slots_two_fixed_plus_flex() {
        let slots = [Slot::Fixed(20.0), Slot::Flex, Slot::Fixed(30.0)];
        let res = split_slots(100.0, &slots, 5.0);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].min.x, 0.0);
        assert_eq!(res[0].width(), 20.0);
        assert_eq!(res[1].min.x, 25.0);
        assert_eq!(res[1].width(), 40.0);
        assert_eq!(res[2].min.x, 70.0);
        assert_eq!(res[2].width(), 30.0);
    }

    #[test]
    fn test_split_slots_too_narrow_never_negative() {
        let slots = [Slot::Fixed(80.0), Slot::Flex, Slot::Fixed(40.0)];
        let res = split_slots(50.0, &slots, 6.0);
        assert_eq!(res.len(), 3);
        assert_eq!(res[1].width(), 0.0);
    }
}
