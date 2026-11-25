//! Standardized informative callout / banner component.
//!
//! Renders an info callout card with a prominent info/help badge
//! on the left, multi-line guidance text in the middle, and an optional custom
//! preview slot on the right, top-aligned and matching project tokens.

use egui::{
    vec2, Align, CornerRadius, Frame, Layout, Margin, Response, RichText, Sense, StrokeKind, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
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
    let target_w = ui.available_width();
    ui.allocate_ui_with_layout(vec2(target_w, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_min_width(target_w);
        let frame = Frame::new()
            .fill(t.sunken)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(Margin::symmetric(12, 10));

        let resp = frame.show(ui, |ui| {
            let inner_w = ui.available_width();
            ui.set_min_width(inner_w);
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.set_min_width(inner_w);
                // 1. Left info icon badge (top-aligned, rectangular, project theme)
                let icon_size = vec2(22.0, 22.0);
                let (rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

                ui.painter().rect_filled(rect, CornerRadius::ZERO, t.panel);
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::ZERO,
                    t.hairline(),
                    StrokeKind::Inside,
                );
                paint_icon(ui.painter(), rect.center(), Icon::Info, 13.0, t.accent_ink);

                ui.add_space(10.0);

                // 2. Middle guidance text (top-aligned)
                let text_layout_width = if preview.is_some() {
                    inner_w - icon_size.x - 10.0 - 56.0
                } else {
                    inner_w - icon_size.x - 10.0
                };

                ui.allocate_ui_with_layout(
                    vec2(text_layout_width.max(100.0), 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_min_width(text_layout_width.max(100.0));
                        ui.label(RichText::new(text).size(11.5).color(t.ink));
                    },
                );

                // 3. Optional right preview (top-aligned)
                if let Some(render_preview) = preview {
                    ui.add_space(8.0);
                    let (preview_rect, _) =
                        ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                    ui.painter()
                        .rect_filled(preview_rect, CornerRadius::ZERO, t.canvas);
                    ui.painter().rect_stroke(
                        preview_rect,
                        CornerRadius::ZERO,
                        t.hairline(),
                        StrokeKind::Inside,
                    );
                    ui.scope_builder(egui::UiBuilder::new().max_rect(preview_rect), |ui| {
                        render_preview(ui);
                    });
                }
            });
        });
        resp.response
    })
    .inner
}
