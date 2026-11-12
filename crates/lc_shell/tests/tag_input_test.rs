//! Headless tests for the modal tag text input and its click blocker.

use egui::{
    Context, Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Sense, pos2, vec2,
};
use lc_shell::ui::tag_input::{TagInputEvent, input_blocker, tag_text_input};

fn screen() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 400.0))
}

fn slot() -> Rect {
    Rect::from_min_size(pos2(100.0, 100.0), vec2(90.0, 32.0))
}

struct Harness {
    ctx: Context,
    text: String,
    focus_now: bool,
    button_rect: Rect,
    button_clicks: u32,
    block: bool,
}

impl Harness {
    fn new() -> Self {
        Self {
            ctx: Context::default(),
            text: String::new(),
            focus_now: true,
            button_rect: Rect::from_min_size(pos2(300.0, 200.0), vec2(80.0, 30.0)),
            button_clicks: 0,
            block: true,
        }
    }

    fn frame(&mut self, events: Vec<Event>) -> TagInputEvent {
        let mut result = TagInputEvent::Editing;
        let ctx = self.ctx.clone();
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen()),
                events,
                ..Default::default()
            },
            |ui| {
                // A "panel" widget living on the background layer, underneath everything.
                let btn = ui.interact(self.button_rect, Id::new("panel_button"), Sense::click());
                if btn.clicked() {
                    self.button_clicks += 1;
                }
                let focus_now = std::mem::take(&mut self.focus_now);
                result = tag_text_input(
                    ui.ctx(),
                    Id::new("tag_field"),
                    slot(),
                    &mut self.text,
                    "Tag name...",
                    focus_now,
                );
                if self.block {
                    input_blocker(ui.ctx());
                }
            },
        );
        out.textures_delta.clear();
        result
    }
}

fn press(p: Pos2) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    }
}

fn release(p: Pos2) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }
}

fn key(k: Key) -> Event {
    Event::Key {
        key: k,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn field_takes_focus_once_and_accepts_typing() {
    let mut h = Harness::new();
    assert_eq!(h.frame(vec![]), TagInputEvent::Editing);
    assert_eq!(h.frame(vec![]), TagInputEvent::Editing);
    assert_eq!(
        h.frame(vec![Event::Text("abc".to_owned())]),
        TagInputEvent::Editing
    );
    assert_eq!(h.text, "abc");
}

#[test]
fn typing_still_works_above_the_blocker() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    // click inside the field (the blocker must not swallow it), then type
    let inside = slot().center();
    h.frame(vec![Event::PointerMoved(inside)]);
    assert_eq!(h.frame(vec![press(inside)]), TagInputEvent::Editing);
    h.frame(vec![release(inside)]);
    h.frame(vec![Event::Text("xy".to_owned())]);
    assert_eq!(h.text, "xy");
}

#[test]
fn enter_commits() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(h.frame(vec![key(Key::Enter)]), TagInputEvent::Commit);
}

#[test]
fn escape_cancels_instead_of_committing() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(h.frame(vec![key(Key::Escape)]), TagInputEvent::Cancel);
}

#[test]
fn pressing_outside_the_field_commits() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    let outside = pos2(450.0, 300.0);
    h.frame(vec![Event::PointerMoved(outside)]);
    assert_eq!(h.frame(vec![press(outside)]), TagInputEvent::Commit);
}

#[test]
fn blocker_swallows_clicks_on_widgets_underneath() {
    let mut h = Harness::new();
    // frames 1-2 register the blocker layer (egui hit-tests against the previous frame)
    h.frame(vec![]);
    h.frame(vec![]);
    let on_button = h.button_rect.center();
    h.frame(vec![Event::PointerMoved(on_button)]);
    h.frame(vec![press(on_button)]);
    h.frame(vec![release(on_button)]);
    h.frame(vec![]);
    assert_eq!(
        h.button_clicks, 0,
        "a click on the panel widget must be swallowed while the tag input is open"
    );
}

#[test]
fn without_the_blocker_the_same_click_reaches_the_widget() {
    // control experiment: proves the previous test is meaningful
    let mut h = Harness::new();
    h.block = false;
    h.frame(vec![]);
    h.frame(vec![]);
    let on_button = h.button_rect.center();
    h.frame(vec![Event::PointerMoved(on_button)]);
    h.frame(vec![press(on_button)]);
    h.frame(vec![release(on_button)]);
    h.frame(vec![]);
    assert_eq!(h.button_clicks, 1);
}
