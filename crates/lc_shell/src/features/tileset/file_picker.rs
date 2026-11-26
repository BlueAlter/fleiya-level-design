//! File picker button widget for tileset source image selection.

use egui::{
    vec2, Align, CornerRadius, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::theme::Tokens;

use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Renders a file picker button with a label/filename, hover outline, and file status coloring.
pub fn file_picker_button(
    ui: &mut Ui,
    t: &Tokens,
    filename: &str,
    is_empty: bool,
    is_missing: bool,
    tooltip: &str,
    desired_width: f32,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(desired_width, CONTROL_HEIGHT_FORM), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = resp.hovered();
        painter.rect_filled(rect, CornerRadius::ZERO, t.sunken);
        let stroke = if hovered {
            Stroke::new(1.0, t.accent)
        } else {
            t.hairline()
        };
        painter.rect_stroke(rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

        let text_color = if is_empty {
            t.muted
        } else if is_missing {
            t.bad
        } else {
            t.ink_strong
        };

        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 8.0, rect.min.y),
            Pos2::new(rect.max.x - 8.0, rect.max.y),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        child.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(RichText::new(filename).color(text_color));
        });
    }

    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
}
