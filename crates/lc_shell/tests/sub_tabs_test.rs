//! Headless geometry tests for the shared sub-tab strip.
//!
//! The strip must not move or resize when the active tab changes, so the rects
//! of every tab are compared for two different active indices.

use egui::{pos2, vec2, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect};
use lc_canvas::theme::Tokens;
use lc_shell::shared::ui::sub_tabs::{sub_tab_id, sub_tabs};

const LABELS: [&str; 2] = ["ENUM DEF", "DOCUMENT"];

fn screen() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 400.0))
}

/// Draws the strip once with `active` selected and returns both tab rects.
fn tab_rects(active: usize) -> [Rect; 2] {
    let ctx = Context::default();
    let mut selected = active;
    let mut ids = [Id::NULL; 2];

    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen()),
            ..Default::default()
        },
        |ui| {
            let tokens = Tokens::get(ui.ctx());
            sub_tabs(ui, &tokens, &mut selected, &LABELS);
            ids = [sub_tab_id(ui, 0), sub_tab_id(ui, 1)];
        },
    );
    output.textures_delta.clear();

    assert_eq!(
        selected, active,
        "drawing the strip must not change the active index"
    );

    ids.map(|id| {
        ctx.read_response(id)
            .expect("every tab must register a response")
            .rect
    })
}

#[test]
fn switching_tabs_moves_and_resizes_nothing() {
    let first_active = tab_rects(0);
    let second_active = tab_rects(1);
    assert_eq!(
        first_active, second_active,
        "tab rects must be identical for active_sub_tab 0 and 1"
    );
}

#[test]
fn clicking_a_tab_selects_it() {
    let ctx = Context::default();
    let mut selected = 0usize;
    let mut ids = [Id::NULL; 2];

    draw_frame(&ctx, &mut selected, &mut ids, Vec::new());
    let target = ctx
        .read_response(ids[1])
        .expect("second tab registers a response")
        .rect;
    let pos: Pos2 = target.center();
    let button = |pressed: bool| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    draw_frame(
        &ctx,
        &mut selected,
        &mut ids,
        vec![Event::PointerMoved(pos), button(true)],
    );
    draw_frame(&ctx, &mut selected, &mut ids, vec![button(false)]);

    assert_eq!(selected, 1, "clicking DOCUMENT must activate it");
}

fn draw_frame(ctx: &Context, selected: &mut usize, ids: &mut [Id; 2], events: Vec<Event>) {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen()),
            events,
            ..Default::default()
        },
        |ui| {
            let tokens = Tokens::get(ui.ctx());
            sub_tabs(ui, &tokens, selected, &LABELS);
            *ids = [sub_tab_id(ui, 0), sub_tab_id(ui, 1)];
        },
    );
    output.textures_delta.clear();
}

#[test]
fn tabs_have_a_stable_layout() {
    let [def, document] = tab_rects(0);
    assert_eq!(
        def.height(),
        document.height(),
        "both tabs share the same height"
    );
    // Same font, padding and stroke: only the label text differs.
    assert!(def.width() > 0.0 && document.width() > 0.0);
    assert!(
        def.right() <= document.left(),
        "tabs must not overlap ({:?} / {:?})",
        def,
        document
    );
    assert_eq!(
        def.top(),
        document.top(),
        "tabs must share one baseline row"
    );
}
