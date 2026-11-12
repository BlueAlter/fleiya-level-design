//! Modal inline text input used for creating and renaming tags.
//!
//! While the input is open the rest of the UI is frozen: [`input_blocker`] covers the
//! whole viewport with an invisible layer that swallows every pointer interaction
//! (clicks, drags, drag handles), and the text field itself is drawn in a higher layer
//! so it stays interactive. A press anywhere outside the field commits the text.
//!
//! # Frame flow
//! 1. Reserve the field's slot inline: `let (slot, _) = ui.allocate_exact_size(size, Sense::hover());`
//! 2. Call [`tag_text_input`] with that slot. Pass `focus_now = true` only on the first
//!    frame after the mode was entered; requesting focus every frame would make it
//!    impossible to ever lose focus (and would hide `lost_focus()` from the caller).
//! 3. At the end of the frame, if any tag input is open, call [`input_blocker`].

use egui::{Align, Area, Context, Id, Key, Margin, Order, Rect, Sense, TextEdit};

/// Outcome of one frame of an inline tag text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagInputEvent {
    /// Still being edited.
    Editing,
    /// The text should be applied (Enter, click outside, or focus lost).
    Commit,
    /// The edit was abandoned with Escape.
    Cancel,
}

/// Draws the text field in its own foreground layer at `slot` and reports what happened.
///
/// Escape always wins over the implicit focus loss it causes, so it cancels instead of
/// committing.
pub fn tag_text_input(
    ctx: &Context,
    id: Id,
    slot: Rect,
    text: &mut String,
    hint: &str,
    focus_now: bool,
) -> TagInputEvent {
    let resp = Area::new(id.with("layer"))
        .order(Order::Foreground)
        .fixed_pos(slot.min)
        .show(ctx, |ui| {
            ui.add_sized(
                slot.size(),
                TextEdit::singleline(text)
                    .id(id)
                    .hint_text(hint)
                    .vertical_align(Align::Center)
                    .margin(Margin::symmetric(8, 0)),
            )
        })
        .inner;

    if focus_now {
        resp.request_focus();
    }

    let (escape, enter, pressed_outside) = ctx.input(|i| {
        (
            i.key_pressed(Key::Escape),
            i.key_pressed(Key::Enter),
            i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !slot.contains(p)),
        )
    });

    if escape {
        TagInputEvent::Cancel
    } else if enter || pressed_outside || resp.lost_focus() {
        TagInputEvent::Commit
    } else {
        TagInputEvent::Editing
    }
}

/// Covers the whole viewport with an invisible layer that swallows pointer input.
///
/// It sits above every panel but below foreground layers, so the tag field (drawn at
/// foreground order) remains interactive. Call it once per frame while an input is open.
pub fn input_blocker(ctx: &Context) {
    let screen = ctx.viewport_rect();
    Area::new(Id::new("tag_input_blocker"))
        .order(Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.allocate_rect(screen, Sense::click_and_drag());
        });
}
