use egui::{pos2, vec2, Color32, Context, CornerRadius, Pos2, RawInput, Rect, Shape};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_shell::shared::ui::section_container::SectionContainer;

fn collect_rects(shape: &Shape, out: &mut Vec<egui::epaint::RectShape>) {
    match shape {
        Shape::Rect(r) => out.push(r.clone()),
        Shape::Vec(children) => {
            for c in children {
                collect_rects(c, out);
            }
        }
        _ => {}
    }
}

#[test]
fn section_container_uses_zero_corner_radius_and_respects_outline_and_selection() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    // 1. Outlined & unselected
    let mut collapsed = false;
    let mut out1 = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            let resp = SectionContainer::new("SECTION A")
                .use_outline(true)
                .use_toggle_collapse(false)
                .selected(false)
                .show(
                    ui,
                    &t,
                    &mut collapsed,
                    |_| {},
                    |ui| {
                        ui.label("Body A");
                    },
                );
            assert!(resp.inner.is_some());
            assert!(!resp.toggled);
        },
    );
    out1.textures_delta.clear();

    let mut rects1 = Vec::new();
    for clipped in &out1.shapes {
        collect_rects(&clipped.shape, &mut rects1);
    }
    assert!(rects1
        .iter()
        .any(|r| r.corner_radius == CornerRadius::ZERO && r.stroke == t.hairline()));

    // 2. Outlined & selected -> accent outline
    let mut out2 = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            SectionContainer::new("SECTION B")
                .use_outline(true)
                .selected(true)
                .show(
                    ui,
                    &t,
                    &mut collapsed,
                    |_| {},
                    |ui| {
                        ui.label("Body B");
                    },
                );
        },
    );
    out2.textures_delta.clear();

    let mut rects2 = Vec::new();
    for clipped in &out2.shapes {
        collect_rects(&clipped.shape, &mut rects2);
    }
    assert!(rects2
        .iter()
        .any(|r| r.corner_radius == CornerRadius::ZERO && r.stroke.color == t.accent));

    // 3. No outline -> Stroke::NONE, and use_custom_color(false) -> transparent fill
    let mut out3 = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            SectionContainer::new("SECTION C")
                .use_outline(true)
                .use_custom_color(false)
                .show(
                    ui,
                    &t,
                    &mut collapsed,
                    |_| {},
                    |ui| {
                        ui.label("Body C");
                    },
                );
        },
    );
    out3.textures_delta.clear();

    let mut rects3 = Vec::new();
    for clipped in &out3.shapes {
        collect_rects(&clipped.shape, &mut rects3);
    }
    assert!(rects3
        .iter()
        .any(|r| r.fill == Color32::TRANSPARENT && r.stroke == t.hairline()));
}

#[test]
fn section_container_collapse_hides_body_and_supports_header_extra() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let mut collapsed = true;
    let mut body_called = false;
    let mut extra_called = false;

    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            let resp = SectionContainer::new("COLLAPSIBLE")
                .use_toggle_collapse(true)
                .show(
                    ui,
                    &t,
                    &mut collapsed,
                    |ui| {
                        extra_called = true;
                        ui.label("Extra");
                    },
                    |_ui| {
                        body_called = true;
                    },
                );
            assert!(resp.inner.is_none());
            assert!(resp.frame_rect.is_none());
        },
    );
    out.textures_delta.clear();
    assert!(extra_called);
    assert!(!body_called);
}

#[test]
fn section_container_detects_pointer_press_inside_for_focus() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let mut collapsed = false;
    let mut clicked = false;

    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            events: vec![egui::Event::PointerButton {
                pos: pos2(60.0, 60.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            let resp = SectionContainer::new("FOCUSABLE")
                .size(vec2(200.0, 120.0))
                .show(
                    ui,
                    &t,
                    &mut collapsed,
                    |_| {},
                    |ui| {
                        ui.label("Inside");
                    },
                );
            clicked = resp.clicked_inside;
        },
    );
    out.textures_delta.clear();
    assert!(
        clicked,
        "Pointer press inside container must set clicked_inside"
    );
}
