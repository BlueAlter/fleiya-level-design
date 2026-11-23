//! Standardized informative callout / banner component.
//!
//! Renders an info callout card with a prominent question/help badge
//! on the left, multi-line guidance text in the middle, and an optional custom
//! preview slot on the right.

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Response, RichText,
    Sense, Stroke, Ui,
};
use lc_canvas::theme::Tokens;

/// Renders a standardized informative callout banner.
///
/// # Arguments
/// - `ui`: Target egui Ui.
/// - `t`: Semantic design tokens.
/// - `text`: Guidance message displayed in the center.
/// - `preview`: Optional closure rendering a preview widget on the right.
pub fn info_banner<F>(ui: &mut Ui, t: &Tokens, text: &str, preview: Option<F>) -> Response
where
    F: FnOnce(&mut Ui),
{
    let frame = Frame::new()
        .fill(Color32::from_rgba_unmultiplied(20, 34, 28, 230))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 95, 65)))
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::symmetric(12, 10));

    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 1. Left help icon (rectangular badge matching project theme)
                let icon_size = vec2(24.0, 24.0);
                let (rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(40, 120, 65));
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "?",
                    FontId::monospace(14.0),
                    Color32::WHITE,
                );

                ui.add_space(8.0);

                // 2. Middle guidance text
                let text_layout_width = if preview.is_some() {
                    ui.available_width() - 56.0
                } else {
                    ui.available_width()
                };

                ui.allocate_ui_with_layout(
                    vec2(text_layout_width.max(100.0), 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.label(
                            RichText::new(text)
                                .size(11.5)
                                .color(Color32::from_rgb(205, 225, 215)),
                        );
                    },
                );

                // 3. Optional right preview
                if let Some(render_preview) = preview {
                    ui.add_space(8.0);
                    let (preview_rect, _) =
                        ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                    ui.painter().rect_filled(
                        preview_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(10, 18, 24, 200),
                    );
                    ui.painter().rect_stroke(
                        preview_rect,
                        CornerRadius::ZERO,
                        t.hairline(),
                        egui::StrokeKind::Inside,
                    );
                    ui.scope_builder(egui::UiBuilder::new().max_rect(preview_rect), |ui| {
                        render_preview(ui);
                    });
                }
            });
        })
        .response
}
