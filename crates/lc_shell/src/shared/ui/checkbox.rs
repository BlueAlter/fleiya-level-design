//! Standardized square-cornered theme-aware checkbox (`CornerRadius::ZERO`).

use egui::{
    vec2, Align, CornerRadius, CursorIcon, Layout, Pos2, Response, RichText, Sense, Stroke,
    StrokeKind, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Size in pixels of the square checkbox indicator box.
pub const CHECKBOX_BOX_SIZE: f32 = 18.0;

/// Renders a crisp square checkbox matching PhotoCraft theme `Tokens`, vertically centered to
/// [`CONTROL_HEIGHT_FORM`] (`32.0 px`).
///
/// Both clicking the box and clicking the label toggle `checked`.
pub fn styled_checkbox(ui: &mut Ui, t: &Tokens, checked: &mut bool, label: &str) -> Response {
    let mut inner_resp: Option<Response> = None;
    let row_resp = ui
        .horizontal(|ui| {
            ui.set_min_height(CONTROL_HEIGHT_FORM);
            ui.spacing_mut().item_spacing.x = 8.0;
            let resp = ui.allocate_ui_with_layout(
                vec2(CHECKBOX_BOX_SIZE, CONTROL_HEIGHT_FORM),
                Layout::left_to_right(Align::Center),
                |ui| paint_checkbox_box(ui, t, checked),
            );
            let mut combined = resp.inner;
            if !label.is_empty() {
                let lbl_resp = ui
                    .add(egui::Label::new(RichText::new(label).color(t.ink)).sense(Sense::click()))
                    .on_hover_cursor(CursorIcon::PointingHand);
                if lbl_resp.clicked() && !combined.clicked() {
                    *checked = !*checked;
                    combined.mark_changed();
                }
                combined = combined.union(lbl_resp);
            }
            inner_resp = Some(combined);
        })
        .response;
    inner_resp.unwrap_or(row_resp)
}

/// Renders a standalone square checkbox without label text inside a [`CONTROL_HEIGHT_FORM`] slot.
pub fn styled_checkbox_square(ui: &mut Ui, t: &Tokens, checked: &mut bool) -> Response {
    ui.allocate_ui_with_layout(
        vec2(CHECKBOX_BOX_SIZE, CONTROL_HEIGHT_FORM),
        Layout::left_to_right(Align::Center),
        |ui| paint_checkbox_box(ui, t, checked),
    )
    .inner
}

fn paint_checkbox_box(ui: &mut Ui, t: &Tokens, checked: &mut bool) -> Response {
    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(CHECKBOX_BOX_SIZE, CHECKBOX_BOX_SIZE), Sense::click());
    resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if resp.clicked() {
        *checked = !*checked;
        resp.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let fill = if *checked {
            t.accent
        } else if resp.hovered() {
            t.panel
        } else {
            t.sunken
        };
        let border_col = if *checked {
            t.accent
        } else if resp.hovered() || resp.has_focus() {
            t.line_strong
        } else {
            t.line
        };
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::ZERO, fill);
        p.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, border_col),
            StrokeKind::Inside,
        );
        if *checked {
            paint_icon(
                p,
                Pos2::new(rect.center().x, rect.center().y),
                Icon::Check,
                12.0,
                t.invert_fg,
            );
        }
    }

    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn styled_checkbox_toggles_on_click_and_uses_zero_radius() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);
        let t = Tokens::get(&ctx);

        let mut val = false;
        let full = Rect::from_min_size(Pos2::ZERO, vec2(240.0, 64.0));

        // Initial render to determine center of checkbox box
        let mut out0 = ctx.run_ui(
            RawInput {
                screen_rect: Some(full),
                ..Default::default()
            },
            |ui| {
                styled_checkbox(ui, &t, &mut val, "Active");
            },
        );
        out0.textures_delta.clear();
        let box_rect = out0
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Rect(r)
                    if (r.rect.width() - CHECKBOX_BOX_SIZE).abs() < 0.5
                        && (r.rect.height() - CHECKBOX_BOX_SIZE).abs() < 0.5 =>
                {
                    assert_eq!(r.corner_radius, CornerRadius::ZERO);
                    Some(r.rect)
                }
                _ => None,
            })
            .expect("checkbox rect must render at 18x18");

        let click = box_rect.center();
        for pressed in [true, false] {
            let mut out = ctx.run_ui(
                RawInput {
                    screen_rect: Some(full),
                    events: vec![
                        egui::Event::PointerMoved(click),
                        egui::Event::PointerButton {
                            pos: click,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ui| {
                    styled_checkbox(ui, &t, &mut val, "Active");
                },
            );
            out.textures_delta.clear();
        }
        assert!(val, "clicking styled_checkbox must toggle value to true");
    }
}
