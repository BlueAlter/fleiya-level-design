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
    vec2, Align, Color32, CornerRadius, CursorIcon, Id, Key, Layout, Pos2, Rect, Response, Sense,
    Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::drag_list::draw_drag_handle;
use super::icon_button::{icon_button, IconButtonStyle};
pub use super::workspace_metrics::{
    ReorderRowLayout, RowFrame, CONTROL_HEIGHT_FORM, ICON_BUTTON_SIZE_DELETE, ROW_LAYOUT_CARD,
    ROW_LAYOUT_VALUES,
};

/// Trailing action button in a reorderable row.
#[derive(Clone, Debug, PartialEq)]
pub enum Trailing {
    /// Delete action button with standard trash icon.
    Delete {
        /// Tooltip shown when delete is enabled.
        tooltip: Option<String>,
        /// Tooltip reason shown when delete is disabled.
        disabled_reason: Option<String>,
    },
    /// Kebab action button ("⋮") for context menus.
    Kebab,
    /// No trailing button.
    None,
}

/// Direction for keyboard-driven selection requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectDirection {
    /// Select previous row (triggered by `ArrowUp`).
    Previous,
    /// Select next row (triggered by `ArrowDown`).
    Next,
}

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
    /// Row layout parameters (height, padding, handle, gaps, trailing, row gap).
    pub layout: ReorderRowLayout,
    /// Lightweight preview for drag list integration.
    pub ghost: GhostPreview,
    /// Whether the row is currently marked as selected.
    pub selected: bool,
    /// Whether the row responds to clicks with pointer cursor.
    pub clickable: bool,
    /// Trailing action button kind (delete, kebab, or none).
    pub trailing: Trailing,
}

impl ReorderRowSpec {
    /// Creates a default reorder row specification.
    pub fn new(id: impl egui::AsId, label: impl Into<String>) -> Self {
        Self {
            id: Id::new(id),
            height: ROW_LAYOUT_VALUES.height,
            outline: None,
            drag: DragCapability::Enabled,
            deletable: true,
            delete_tooltip: Some("Delete".to_string()),
            delete_disabled_reason: None,
            layout: ROW_LAYOUT_VALUES,
            ghost: GhostPreview::new(label),
            selected: false,
            clickable: false,
            trailing: Trailing::Delete {
                tooltip: Some("Delete".to_string()),
                disabled_reason: None,
            },
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
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
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
    /// Response of the entire row container.
    pub row_response: Response,
    /// Whether the row was activated (via mouse click or keyboard Enter).
    pub clicked: bool,
    /// Whether the delete button was clicked this frame.
    pub delete_clicked: bool,
    /// Trailing action button interaction response (if present).
    pub trailing_response: Option<Response>,
    /// Full bounding rectangle of the row.
    pub rect: Rect,
    /// Keyboard reorder request via `Alt+Up` or `Alt+Down`.
    pub move_request: Option<MoveDirection>,
    /// Keyboard selection change request via `Up` or `Down`.
    pub select_request: Option<SelectDirection>,
    /// Position where a context menu was requested (via kebab click or right-click).
    pub menu_requested: Option<Pos2>,
}

/// Renders a row with layout `[handle][body slots...][trailing]` and strict sizing contracts.
pub fn reorder_row<R>(
    ui: &mut Ui,
    spec: ReorderRowSpec,
    slots: &[Slot],
    body: impl FnOnce(&mut Ui, &mut RowBody) -> R,
) -> ReorderRowResponse<R> {
    let t = Tokens::get(ui.ctx());
    let available_w = ui.available_width();
    let row_h = spec.layout.height;
    let (row_rect, mut row_resp) = ui.allocate_exact_size(vec2(available_w, row_h), Sense::click());

    if spec.clickable {
        row_resp = row_resp.on_hover_cursor(CursorIcon::PointingHand);
    }

    let painter = ui.painter();

    // 1. Paint row background and 1px outline (Card preset only)
    if spec.layout.frame == RowFrame::Card {
        let is_hovered = row_resp.hovered();
        let bg = if spec.selected || is_hovered {
            t.sunken
        } else {
            t.panel
        };

        let border_color = if spec.selected {
            t.accent
        } else if is_hovered {
            t.line_strong
        } else if let Some(outline_c) = spec.outline {
            outline_c
        } else {
            t.hairline().color
        };

        painter.rect_filled(row_rect, CornerRadius::same(4), bg);
        painter.rect_stroke(
            row_rect,
            CornerRadius::same(4),
            Stroke::new(1.0, border_color),
            StrokeKind::Inside,
        );
    }

    // 2. Drag handle slot on the left
    let content_h = (row_h - 2.0 * spec.layout.padding_y).max(1.0);
    let cy = row_rect.center().y;
    let handle_min_x = row_rect.min.x + spec.layout.padding_x;
    let handle_rect = Rect::from_min_size(
        Pos2::new(handle_min_x, cy - content_h / 2.0),
        vec2(spec.layout.handle_w, content_h),
    );
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
    draw_drag_handle(painter, handle_rect, handle_color);

    // 3. Trailing action slot on the right
    let mut delete_clicked = false;
    let mut trailing_response = None;
    let mut trailing_rect_opt = None;
    let tr_w = spec.layout.trailing_w;
    let tr_rect = Rect::from_min_size(
        Pos2::new(
            row_rect.max.x - spec.layout.padding_x - tr_w,
            cy - tr_w / 2.0,
        ),
        vec2(tr_w, tr_w),
    );

    let trailing_mode = match spec.trailing {
        Trailing::Delete {
            tooltip,
            disabled_reason,
        } => Trailing::Delete {
            tooltip: tooltip.or(spec.delete_tooltip),
            disabled_reason: disabled_reason.or(if !spec.deletable {
                spec.delete_disabled_reason.or(Some("Disabled".into()))
            } else {
                spec.delete_disabled_reason
            }),
        },
        other => other,
    };

    match trailing_mode {
        Trailing::Delete {
            tooltip,
            disabled_reason,
        } => {
            trailing_rect_opt = Some(tr_rect);
            let mut del_child = ui.new_child(egui::UiBuilder::new().max_rect(tr_rect));
            let style = IconButtonStyle::bordered();
            let resp = icon_button(&mut del_child, Icon::Delete, tr_w, style);
            if let Some(ref reason) = disabled_reason {
                resp.clone().on_hover_text(reason);
            } else if let Some(ref tip) = tooltip {
                resp.clone().on_hover_text(tip);
            }
            if disabled_reason.is_none() && resp.clicked() {
                delete_clicked = true;
            }
            trailing_response = Some(resp);
        }
        Trailing::Kebab => {
            trailing_rect_opt = Some(tr_rect);
            let resp = ui
                .interact(tr_rect, spec.id.with("kebab"), Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Actions");
            let kebab_color = if resp.hovered() {
                t.ink_strong
            } else {
                t.muted
            };
            paint_icon(
                painter,
                tr_rect.center(),
                Icon::DotsThreeVertical,
                13.0,
                kebab_color,
            );
            trailing_response = Some(resp);
        }
        Trailing::None => {}
    }

    // 4. Body slots in the remaining middle region
    let body_start_x = handle_rect.max.x + spec.layout.handle_gap;
    let body_end_x = if let Some(trailing_rect) = trailing_rect_opt {
        trailing_rect.min.x - spec.layout.trailing_gap
    } else {
        row_rect.max.x - spec.layout.padding_x
    };
    let body_w = (body_end_x - body_start_x).max(0.0);

    let relative_rects = split_slots(body_w, slots, spec.layout.slot_gap);
    let absolute_rects: Vec<Rect> = relative_rects
        .into_iter()
        .map(|r| {
            Rect::from_min_size(
                Pos2::new(body_start_x + r.min.x, cy - content_h / 2.0),
                vec2(r.width(), content_h),
            )
        })
        .collect();

    let mut body_scope = RowBody {
        slot_rects: absolute_rects,
    };
    let body_res = body(ui, &mut body_scope);

    // 5. Context menu trigger from Kebab click or row secondary click
    let mut menu_requested = None;
    if let Some(ref tr_resp) = trailing_response {
        if tr_resp.clicked() {
            if let Some(tr_rect) = trailing_rect_opt {
                menu_requested = Some(tr_rect.left_bottom());
            }
        }
    }
    if row_resp.secondary_clicked() {
        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
            menu_requested = Some(pos);
        } else {
            menu_requested = Some(row_rect.left_bottom());
        }
    }

    // 6. Keyboard navigation: Alt+Up / Alt+Down (move), Up / Down (select), Enter (activate)
    let mut move_request = None;
    let mut select_request = None;
    let wants_kbd = ui.ctx().egui_wants_keyboard_input();
    let has_key_interest =
        !wants_kbd && (row_resp.hovered() || ui.memory(|m| m.has_focus(spec.id)) || spec.selected);
    if has_key_interest {
        if ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowUp)) {
            move_request = Some(MoveDirection::Up);
        } else if ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowDown)) {
            move_request = Some(MoveDirection::Down);
        } else if ui.input(|i| !i.modifiers.any() && i.key_pressed(Key::ArrowUp)) {
            select_request = Some(SelectDirection::Previous);
        } else if ui.input(|i| !i.modifiers.any() && i.key_pressed(Key::ArrowDown)) {
            select_request = Some(SelectDirection::Next);
        }
    }

    let enter_activated =
        has_key_interest && ui.input(|i| !i.modifiers.any() && i.key_pressed(Key::Enter));
    let clicked = row_resp.clicked() || enter_activated;

    ReorderRowResponse {
        body: body_res,
        handle: handle_resp,
        row_response: row_resp,
        clicked,
        delete_clicked,
        trailing_response,
        rect: row_rect,
        move_request,
        select_request,
        menu_requested,
    }
}
