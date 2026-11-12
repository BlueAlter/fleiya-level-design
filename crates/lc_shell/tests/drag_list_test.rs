//! Headless tests for the reusable drag-to-reorder component.

use egui::{
    pos2, vec2, Color32, Context, Event, Key, Modifiers, PointerButton, RawInput, Rect, Sense,
};
use lc_shell::shared::ui::drag_list::{
    paint_placeholder, DragListState, DragOutcome, DragTheme, PlaceholderStyle,
};
use lc_shell::shared::ui::reorder::{reorder_item, Slot};

const ROWS: usize = 4;
const ROW_H: f32 = 40.0;
const GAP: f32 = 4.0;

fn screen() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0))
}

/// Runs one frame drawing `ROWS` rows according to the drag layout.
/// Returns the rects of the displayed slots and the drag outcome.
fn frame(
    ctx: &Context,
    state: &mut DragListState,
    theme: &DragTheme,
    events: Vec<Event>,
) -> (Vec<Rect>, Vec<Slot>, DragOutcome) {
    let mut rects = Vec::new();
    let mut slots_out = Vec::new();
    let mut outcome = DragOutcome::None;
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen()),
            events,
            ..Default::default()
        },
        |ui| {
            let slots = state.layout(ROWS);
            slots_out = slots.clone();
            for slot in slots {
                match slot {
                    Slot::Item(_) => {
                        let (rect, _) = ui.allocate_exact_size(vec2(200.0, ROW_H), Sense::hover());
                        rects.push(rect);
                    }
                    Slot::Placeholder => {
                        let size = state.row_size().unwrap();
                        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                        paint_placeholder(ui.painter(), rect, theme);
                        rects.push(rect);
                    }
                }
                ui.add_space(GAP);
            }
            state.update(ui, &rects, theme);
            outcome = state.finish(ui);
        },
    );
    output.textures_delta.clear();
    (rects, slots_out, outcome)
}

fn move_to(p: egui::Pos2) -> Event {
    Event::PointerMoved(p)
}

fn press(p: egui::Pos2) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    }
}

fn release(p: egui::Pos2) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }
}

fn theme() -> DragTheme {
    DragTheme::from_accent(Color32::from_rgb(0, 220, 255))
}

/// Starts a drag on row `from` (grab at its centre) and returns the idle rects.
fn start_drag(ctx: &Context, state: &mut DragListState, from: usize) -> Vec<Rect> {
    let t = theme();
    let (idle, _, _) = frame(ctx, state, &t, vec![]);
    let grab = idle[from].center();
    let (_, _, _) = frame(ctx, state, &t, vec![move_to(grab), press(grab)]);
    state.begin(ctx, from, idle[from]);
    idle
}

#[test]
fn idle_state_uses_normal_layout() {
    let state = DragListState::new();
    assert!(!state.is_active());
    assert_eq!(
        state.layout(3),
        vec![Slot::Item(0), Slot::Item(1), Slot::Item(2)]
    );
}

#[test]
fn layout_stays_normal_until_snapshot_is_captured() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let (idle, _, _) = frame(&ctx, &mut state, &t, vec![]);
    state.begin(&ctx, 2, idle[2]);

    // begin() alone must not change the layout (snapshot not captured yet)
    assert_eq!(
        state.layout(ROWS),
        (0..ROWS).map(Slot::Item).collect::<Vec<_>>()
    );

    // after the first update the snapshot exists and the placeholder takes row 2's slot
    let _ = frame(&ctx, &mut state, &t, vec![move_to(idle[2].center())]);
    assert_eq!(
        state.layout(ROWS),
        vec![
            Slot::Item(0),
            Slot::Item(1),
            Slot::Placeholder,
            Slot::Item(3)
        ]
    );
}

#[test]
fn ghost_slot_follows_pointer_up_and_down() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let idle = start_drag(&ctx, &mut state, 2);

    // pointer over row 2's own centre: no-op, placeholder stays at index 2
    let _ = frame(&ctx, &mut state, &t, vec![move_to(idle[2].center())]);
    assert_eq!(state.target_index(), Some(2));

    // drag up above row 1's centre -> insertion index 1, placeholder moves to index 1
    let up = pos2(idle[1].center().x, idle[1].center().y - 5.0);
    let _ = frame(&ctx, &mut state, &t, vec![move_to(up)]);
    assert_eq!(state.target_index(), Some(1));
    assert_eq!(
        state.layout(ROWS),
        vec![
            Slot::Item(0),
            Slot::Placeholder,
            Slot::Item(1),
            Slot::Item(3)
        ]
    );

    // all the way to the top
    let top = pos2(idle[0].center().x, idle[0].top());
    let _ = frame(&ctx, &mut state, &t, vec![move_to(top)]);
    assert_eq!(
        state.layout(ROWS),
        vec![
            Slot::Placeholder,
            Slot::Item(0),
            Slot::Item(1),
            Slot::Item(3)
        ]
    );

    // down past the last row
    let bottom = pos2(idle[3].center().x, idle[3].bottom() + 30.0);
    let _ = frame(&ctx, &mut state, &t, vec![move_to(bottom)]);
    assert_eq!(state.target_index(), Some(4));
    assert_eq!(
        state.layout(ROWS),
        vec![
            Slot::Item(0),
            Slot::Item(1),
            Slot::Item(3),
            Slot::Placeholder
        ]
    );
}

#[test]
fn placeholder_does_not_flicker_while_pointer_rests() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let idle = start_drag(&ctx, &mut state, 2);

    // rest the pointer slightly above row 1's centre for many frames
    let p = pos2(idle[1].center().x, idle[1].center().y - 2.0);
    let mut seen = Vec::new();
    for _ in 0..8 {
        let _ = frame(&ctx, &mut state, &t, vec![move_to(p)]);
        seen.push(state.target_index());
    }
    assert!(
        seen.iter().all(|t| *t == Some(1)),
        "target flickered: {seen:?}"
    );
}

#[test]
fn release_drops_and_matches_preview_order() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let idle = start_drag(&ctx, &mut state, 2);

    let up = pos2(idle[1].center().x, idle[1].center().y - 5.0);
    let _ = frame(&ctx, &mut state, &t, vec![move_to(up)]);
    let _ = frame(&ctx, &mut state, &t, vec![move_to(up)]);
    let (_, preview, _) = frame(&ctx, &mut state, &t, vec![move_to(up)]);

    let (_, _, outcome) = frame(&ctx, &mut state, &t, vec![release(up)]);
    assert_eq!(outcome, DragOutcome::Dropped { from: 2, to: 1 });
    assert!(!state.is_active());

    // order the user saw == order after applying the outcome
    let mut v: Vec<usize> = (0..ROWS).collect();
    reorder_item(&mut v, 2, 1);
    let shown: Vec<usize> = preview
        .iter()
        .map(|s| match s {
            Slot::Item(i) => *i,
            Slot::Placeholder => 2,
        })
        .collect();
    assert_eq!(shown, v);
}

#[test]
fn escape_cancels_and_is_consumed() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let idle = start_drag(&ctx, &mut state, 1);
    let _ = frame(&ctx, &mut state, &t, vec![move_to(idle[1].center())]);

    let (_, _, outcome) = frame(
        &ctx,
        &mut state,
        &t,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(outcome, DragOutcome::Cancelled);
    assert!(!state.is_active());
}

#[test]
fn escape_is_not_consumed_when_idle() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let mut seen_escape = false;
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen()),
            events: vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            let _ = state.finish(ui);
            seen_escape = ui.input(|i| i.key_pressed(Key::Escape));
        },
    );
    output.textures_delta.clear();
    assert!(seen_escape, "idle list must not swallow Escape");
    let _ = t;
}

#[test]
fn drop_on_same_place_is_a_noop_move() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let idle = start_drag(&ctx, &mut state, 1);
    let c = idle[1].center();
    let _ = frame(&ctx, &mut state, &t, vec![move_to(c)]);
    let (_, _, outcome) = frame(&ctx, &mut state, &t, vec![release(c)]);
    let DragOutcome::Dropped { from, to } = outcome else {
        panic!("expected Dropped, got {outcome:?}");
    };
    let mut v: Vec<usize> = (0..ROWS).collect();
    assert!(!reorder_item(&mut v, from, to));
}

#[test]
fn second_begin_is_ignored_while_dragging() {
    let ctx = Context::default();
    let mut state = DragListState::new();
    let t = theme();
    let (idle, _, _) = frame(&ctx, &mut state, &t, vec![]);
    state.begin(&ctx, 0, idle[0]);
    state.begin(&ctx, 3, idle[3]);
    assert_eq!(state.dragging_index(), Some(0));
}

// ---------------------------------------------------------------------------
// Ghost slot outline
// ---------------------------------------------------------------------------

fn flatten(shape: &egui::epaint::Shape, out: &mut Vec<egui::epaint::Shape>) {
    match shape {
        egui::epaint::Shape::Vec(v) => v.iter().for_each(|s| flatten(s, out)),
        other => out.push(other.clone()),
    }
}

fn painted_shapes(theme: &DragTheme, rect: Rect) -> Vec<egui::epaint::Shape> {
    let ctx = Context::default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen()),
            ..Default::default()
        },
        |ui| {
            paint_placeholder(ui.painter(), rect, theme);
        },
    );
    out.textures_delta.clear();
    let mut shapes = Vec::new();
    for clipped in &out.shapes {
        flatten(&clipped.shape, &mut shapes);
    }
    shapes
}

#[test]
fn solid_placeholder_is_one_pixel_snapped_outline() {
    use egui::epaint::Shape;
    let theme = theme();
    assert_eq!(theme.placeholder_style, PlaceholderStyle::Solid);
    // fractional position and size, as produced by scrolling and layout
    let rect = Rect::from_min_size(pos2(10.3, 20.7), vec2(231.4, 36.0));
    let shapes = painted_shapes(&theme, rect);

    let rects: Vec<_> = shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Rect(r) => Some(r.rect),
            _ => None,
        })
        .collect();
    // fill + outline, no dashes
    assert_eq!(
        rects.len(),
        2,
        "expected fill rect and outline rect: {shapes:?}"
    );
    assert!(!shapes
        .iter()
        .any(|s| matches!(s, Shape::LineSegment { .. } | Shape::Path(_))));
    for r in rects {
        for v in [r.min.x, r.min.y, r.max.x, r.max.y] {
            assert_eq!(v, v.round(), "edge not snapped to a whole pixel: {r:?}");
        }
    }
}

#[test]
fn dashed_placeholder_dashes_each_edge_independently() {
    use egui::epaint::Shape;
    let mut theme = theme();
    theme.placeholder_style = PlaceholderStyle::Dashed;
    let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(231.0, 36.0));
    let shapes = painted_shapes(&theme, rect);

    let mut dashes = 0;
    for s in &shapes {
        let points: Vec<egui::Pos2> = match s {
            Shape::LineSegment { points, .. } => points.to_vec(),
            Shape::Path(p) => p.points.clone(),
            _ => continue,
        };
        dashes += 1;
        let same_x = points.windows(2).all(|w| (w[0].x - w[1].x).abs() < 1e-3);
        let same_y = points.windows(2).all(|w| (w[0].y - w[1].y).abs() < 1e-3);
        assert!(same_x || same_y, "a dash bends around a corner: {points:?}");
    }
    assert!(dashes > 10, "expected many dashes, got {dashes}");
}
