//! Typography and hairline primitives.
//!
//! Upper-case text is drawn upper case but announced to screen readers as
//! written, matching the accessible label of the source string.

use std::sync::Arc;

use egui::{vec2, Color32, CornerRadius, Galley, Rect, Response, Sense, Stroke, Ui};

use crate::theme::{self, Tokens};

/// Lays out `text` upper case in `family`, with `spacing` points between letters.
pub fn caps(
    ui: &Ui,
    text: &str,
    size: f32,
    family: egui::FontFamily,
    spacing: f32,
    color: Color32,
) -> Arc<Galley> {
    let format = egui::TextFormat {
        font_id: egui::FontId::new(size, family),
        color,
        extra_letter_spacing: spacing,
        ..Default::default()
    };
    ui.painter()
        .layout_job(egui::text::LayoutJob::single_section(
            text.to_uppercase(),
            format,
        ))
}

/// A HUD label galley: 10 px Geist Mono 500, upper case, widely spaced.
pub fn hud_galley(ui: &Ui, text: &str, color: Color32) -> Arc<Galley> {
    caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        color,
    )
}

/// Paints `galley` at the left of `rect`, vertically centred.
pub(crate) fn paint_left(ui: &Ui, rect: Rect, galley: Arc<Galley>, color: Color32) {
    let pos = egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
    ui.painter()
        .galley_with_override_text_color(pos, galley, color);
}

/// Paints `galley` centred in `rect`.
pub(crate) fn paint_centred(ui: &Ui, rect: Rect, galley: Arc<Galley>, color: Color32) {
    ui.painter().galley_with_override_text_color(
        rect.center() - galley.size() / 2.0,
        galley,
        color,
    );
}

/// A mono HUD label.
pub fn hud(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let galley = hud_galley(ui, text, color);
    let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
    paint_left(ui, rect, galley, color);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    resp
}

/// A mono HUD label cut to one line with "..." when wider than `max_width`.
pub fn hud_elided(ui: &mut Ui, text: &str, color: Color32, max_width: f32) -> Response {
    let format = egui::TextFormat {
        font_id: egui::FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())),
        color,
        extra_letter_spacing: theme::HUD_SPACING,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::single_section(text.to_uppercase(), format);
    job.wrap =
        egui::text::TextWrapping::truncate_at_width(max_width.min(ui.available_width()).max(0.0));
    let galley = ui.painter().layout_job(job);
    let elided = galley.elided;
    let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
    paint_left(ui, rect, galley, color);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    if elided {
        resp.on_hover_text(text)
    } else {
        resp
    }
}

/// "01 / Features": a section eyebrow, the index faint.
pub fn eyebrow(ui: &mut Ui, index: &str, label: &str) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        hud(ui, index, t.faint);
        hud(ui, " / ", t.faint);
        hud(ui, label, t.muted);
    });
}

/// A vertical hairline `height` tall.
pub fn rule(ui: &mut Ui, height: f32) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, height), Sense::hover());
    ui.painter()
        .vline(rect.center().x, rect.y_range(), t.hairline());
}

/// A full-width horizontal hairline.
pub fn hline(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, t.hairline());
}

/// A solid chip with a bold mono label.
pub fn tag(ui: &mut Ui, text: &str, fill: Color32, ink: Color32) -> Response {
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_bold_for(ui.ctx()),
        theme::HUD_SPACING,
        ink,
    );
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(12.0, 6.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    paint_centred(ui, rect, galley, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    resp
}

/// A small status dot.
pub fn status_dot(ui: &mut Ui, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.0, color);
    resp
}

/// A small chevron; `open` points down, otherwise right.
pub fn chevron(ui: &mut Ui, open: bool, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
    let c = rect.center();
    let stroke = Stroke::new(1.2, color);
    if open {
        ui.painter()
            .line_segment([c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0)], stroke);
        ui.painter()
            .line_segment([c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)], stroke);
    } else {
        ui.painter()
            .line_segment([c + vec2(-2.0, -4.0), c + vec2(2.0, 0.0)], stroke);
        ui.painter()
            .line_segment([c + vec2(2.0, 0.0), c + vec2(-2.0, 4.0)], stroke);
    }
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            if open { "Collapse" } else { "Expand" },
        )
    });
    resp
}

/// A close glyph.
pub fn close_icon(ui: &mut Ui, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
    let c = rect.center();
    let stroke = Stroke::new(1.2, color);
    let s = 4.0;
    ui.painter()
        .line_segment([c + vec2(-s, -s), c + vec2(s, s)], stroke);
    ui.painter()
        .line_segment([c + vec2(-s, s), c + vec2(s, -s)], stroke);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close"));
    resp
}
