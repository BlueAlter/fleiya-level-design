//! Headless interaction tests for the shared searchable picker popup.
//!
//! The popup is opened through egui's popup memory (`Popup::open_id`) instead of a
//! real click so the tests stay deterministic; keyboard, search and the row cap
//! are then driven through synthetic input events.

use egui::{pos2, vec2, Context, Event, Id, Key, Modifiers, Popup, RawInput, Rect, Sense};
use lc_shell::shared::ui::search_picker::{
    search_picker, PickerChoice, PickerItem, SearchPickerConfig, SearchPickerState,
    PICKER_ROW_HEIGHT,
};

fn screen() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 400.0))
}

const PICKER_ID: &str = "search_picker_test";

struct Harness {
    ctx: Context,
    items: Vec<PickerItem<u32>>,
    state: SearchPickerState,
    max_rows: usize,
}

impl Harness {
    fn new(items: Vec<PickerItem<u32>>) -> Self {
        Self {
            ctx: Context::default(),
            items,
            state: SearchPickerState::default(),
            max_rows: 5,
        }
    }

    fn picker_id() -> Id {
        Id::new(PICKER_ID)
    }

    fn open(&self) {
        Popup::open_id(&self.ctx, Self::picker_id());
    }

    fn frame(&mut self, events: Vec<Event>) -> Option<PickerChoice<u32>> {
        let ctx = self.ctx.clone();
        let mut out = None;
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen()),
                events,
                ..Default::default()
            },
            |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(120.0, 32.0), Sense::hover());
                let anchor = ui.interact(rect, Id::new("anchor"), Sense::click());
                let mut cfg = SearchPickerConfig::new("Search tags...");
                cfg.max_rows = self.max_rows;
                out = search_picker(
                    &anchor,
                    Self::picker_id(),
                    &self.items,
                    &mut self.state,
                    &cfg,
                );
            },
        );
        output.textures_delta.clear();
        out
    }

    fn focused_search(&self) -> bool {
        self.ctx.memory(|m| m.focused()) == Some(Self::picker_id().with("search_input"))
    }

    fn is_open(&self) -> bool {
        Popup::is_id_open(&self.ctx, Self::picker_id())
    }

    fn popup_rect(&self) -> Option<Rect> {
        self.ctx.memory(|m| m.area_rect(Self::picker_id()))
    }
}

fn tags() -> Vec<PickerItem<u32>> {
    vec![
        PickerItem::new(1, "Forest"),
        PickerItem::new(2, "Desert"),
        PickerItem::new(3, "Ocean"),
        PickerItem::new(4, "Tundra"),
        PickerItem::new(5, "Volcanic"),
        PickerItem::new(6, "Swamp"),
    ]
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

/// Opens the popup and lets one frame pass so the search field can take focus.
fn opened(mut h: Harness) -> Harness {
    h.frame(vec![]);
    h.open();
    h.frame(vec![]);
    h
}

#[test]
fn opens_with_focus_in_the_search_field() {
    let h = opened(Harness::new(tags()));
    assert!(h.is_open(), "popup must stay open");
    assert!(
        h.focused_search(),
        "the search field must be focused when the popup opens"
    );
}

#[test]
fn typing_filters_and_enter_selects_the_match() {
    let mut h = opened(Harness::new(tags()));
    h.frame(vec![Event::Text("des".to_owned())]);
    h.frame(vec![]);
    assert_eq!(
        h.state.search(),
        "des",
        "search text is owned by the caller"
    );

    let choice = h.frame(vec![key(Key::Enter)]);
    assert_eq!(
        choice,
        Some(PickerChoice::Item(2)),
        "Enter picks the only remaining match"
    );
    assert!(!h.is_open(), "picking an item closes the popup");
}

#[test]
fn arrow_keys_skip_rows_and_clamp() {
    let mut h = opened(Harness::new(tags()));

    // The first item row is preselected.
    let first = h.frame(vec![key(Key::ArrowUp)]);
    assert_eq!(first, None, "arrow up at the top only clamps");
    let second = h.frame(vec![key(Key::ArrowDown)]);
    assert_eq!(second, None);
    let choice = h.frame(vec![key(Key::Enter)]);
    assert_eq!(choice, Some(PickerChoice::Item(2)), "second row is Desert");
}

#[test]
fn escape_closes_without_a_choice() {
    let mut h = opened(Harness::new(tags()));
    let choice = h.frame(vec![key(Key::Escape)]);
    assert_eq!(choice, None);
    assert!(!h.is_open(), "Escape closes the popup");
}

#[test]
fn list_is_capped_and_scrolls() {
    let mut h = opened(Harness::new(tags()));
    h.max_rows = 3;
    h.frame(vec![]);
    let rect = h.popup_rect().expect("popup is open");
    // Three item rows plus the search field: far shorter than six rows.
    let capped = PICKER_ROW_HEIGHT * 3.0 + PICKER_ROW_HEIGHT * 2.0;
    assert!(
        rect.height() <= capped,
        "popup height {} must stay within the 3-row cap ({capped})",
        rect.height()
    );
    assert!(
        rect.height() >= PICKER_ROW_HEIGHT * 3.0,
        "the capped list still shows three rows"
    );
}

#[test]
fn search_without_matches_keeps_the_popup_open() {
    let mut h = opened(Harness::new(tags()));
    h.frame(vec![Event::Text("zzz".to_owned())]);
    let choice = h.frame(vec![key(Key::Enter)]);
    assert_eq!(choice, None, "an empty result list selects nothing");
    assert!(h.is_open(), "the popup stays open on an empty result list");
}
