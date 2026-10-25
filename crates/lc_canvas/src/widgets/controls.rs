//! Interactive editor controls.

use egui::{vec2, Color32, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui};

use super::primitives::{caps, paint_centred};
use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};

/// Visual weight of a button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    /// Inverted primary action.
    Primary,
    /// Outlined secondary action.
    Secondary,
    /// Borderless action.
    Ghost,
    /// Destructive action.
    Danger,
}

/// A square, mono-labelled button.
pub fn button(ui: &mut Ui, text: &str, kind: ButtonKind, enabled: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_bold_for(ui.ctx()),
        theme::BUTTON_SPACING,
        t.ink,
    );
    let size = vec2(galley.size().x + 20.0, theme::CONTROL_H);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let hovered = enabled && resp.hovered();

    let (fill, stroke, ink) = match (kind, enabled, hovered) {
        (_, false, _) => (Color32::TRANSPARENT, t.line, t.faint),
        (ButtonKind::Primary, true, false) => (t.invert_bg, t.invert_bg, t.invert_fg),
        (ButtonKind::Primary, true, true) => (t.accent, t.accent, Color32::WHITE),
        (ButtonKind::Secondary, true, false) => (Color32::TRANSPARENT, t.line, t.ink),
        (ButtonKind::Secondary, true, true) => (t.sunken, t.line_strong, t.ink_strong),
        (ButtonKind::Ghost, true, false) => (Color32::TRANSPARENT, Color32::TRANSPARENT, t.muted),
        (ButtonKind::Ghost, true, true) => (t.sunken, Color32::TRANSPARENT, t.ink_strong),
        (ButtonKind::Danger, true, false) => (Color32::TRANSPARENT, t.bad, t.bad),
        (ButtonKind::Danger, true, true) => (t.bad, t.bad, Color32::WHITE),
    };

    {
        let p = ui.painter();
        if fill != Color32::TRANSPARENT {
            p.rect_filled(rect, CornerRadius::ZERO, fill);
        }
        if stroke != Color32::TRANSPARENT {
            p.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, stroke),
                StrokeKind::Inside,
            );
        }
    }

    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_bold_for(ui.ctx()),
        theme::BUTTON_SPACING,
        ink,
    );
    paint_centred(ui, rect, galley, ink);

    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text));
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// A square icon button with a tooltip.
pub fn icon_button(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) =
        ui.allocate_exact_size(vec2(theme::CONTROL_H, theme::CONTROL_H), Sense::click());
    let hovered = resp.hovered();
    {
        let p = ui.painter();
        if hovered {
            p.rect_filled(rect, CornerRadius::ZERO, t.sunken);
        }
        icons::paint_icon(
            p,
            rect.center(),
            icon,
            14.0,
            if hovered { t.ink_strong } else { t.muted },
        );
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    resp.on_hover_text(tip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A tool toggle with an accent underline when active.
pub fn tool_button(ui: &mut Ui, text: &str, active: bool, tip: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        t.ink,
    );
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 16.0, 26.0), Sense::click());
    let hovered = resp.hovered();
    let (fill, ink) = if active {
        (t.sunken, t.ink_strong)
    } else if hovered {
        (t.sunken, t.ink)
    } else {
        (Color32::TRANSPARENT, t.muted)
    };
    {
        let p = ui.painter();
        if fill != Color32::TRANSPARENT {
            p.rect_filled(rect, CornerRadius::ZERO, fill);
        }
        if active {
            p.hline(rect.x_range(), rect.max.y - 0.5, Stroke::new(1.0, t.accent));
        }
    }
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        ink,
    );
    paint_centred(ui, rect, galley, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    resp.on_hover_text(tip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A dock/document tab. Active tabs are inverted with an accent underline.
pub fn dock_tab(ui: &mut Ui, text: &str, active: bool) -> Response {
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        Tokens::get(ui.ctx()).ink,
    );
    dock_tab_sized(ui, text, active, galley.size().x + 20.0)
}

/// A dock/document tab with a fixed width, eliding long labels.
pub fn dock_tab_sized(ui: &mut Ui, text: &str, active: bool, width: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) =
        ui.allocate_exact_size(vec2(width.max(24.0), theme::TAB_H), Sense::click_and_drag());
    let hovered = resp.hovered();
    let (fill, ink) = if active {
        (t.bg, t.ink_strong)
    } else if hovered {
        (t.sunken, t.ink)
    } else {
        (t.panel, t.muted)
    };
    {
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::ZERO, fill);
        if active {
            p.hline(rect.x_range(), rect.max.y - 0.5, Stroke::new(1.0, t.accent));
        }
        p.vline(rect.max.x - 0.5, rect.y_range(), t.hairline());
    }

    let format = egui::TextFormat {
        font_id: FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())),
        color: ink,
        extra_letter_spacing: theme::HUD_SPACING,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::single_section(text.to_uppercase(), format);
    job.wrap = egui::text::TextWrapping::truncate_at_width((width - 16.0).max(0.0));
    let galley = ui.painter().layout_job(job);
    paint_centred(ui, rect, galley, ink);

    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A tree row with indentation and an inverted selected state.
pub fn tree_row(ui: &mut Ui, text: &str, selected: bool, depth: usize) -> Response {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, theme::ROW_H), Sense::click());
    let hovered = resp.hovered();
    let (fill, ink) = if selected {
        (t.invert_bg, t.invert_fg)
    } else if hovered {
        (t.sunken, t.ink_strong)
    } else {
        (Color32::TRANSPARENT, t.ink)
    };
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    }

    let text_x = rect.left() + 8.0 + depth as f32 * 12.0;
    let format = egui::TextFormat {
        font_id: FontId::new(12.0, theme::medium_for(ui.ctx())),
        color: ink,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::single_section(text.to_owned(), format);
    job.wrap = egui::text::TextWrapping::truncate_at_width((rect.right() - text_x - 8.0).max(0.0));
    let galley = ui.painter().layout_job(job);
    ui.painter().galley_with_override_text_color(
        egui::pos2(text_x, rect.center().y - galley.size().y / 2.0),
        galley,
        ink,
    );

    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A toggle chip used for filters and compact options.
pub fn toggle_chip(ui: &mut Ui, text: &str, active: bool, accent: Color32) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        accent,
    );
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 14.0, 22.0), Sense::click());
    let hovered = resp.hovered();
    let (fill, stroke, ink) = if active {
        (theme::tint(accent, 0.14), accent, accent)
    } else if hovered {
        (t.sunken, t.line_strong, t.ink)
    } else {
        (Color32::TRANSPARENT, t.line, t.muted)
    };
    {
        let p = ui.painter();
        if fill != Color32::TRANSPARENT {
            p.rect_filled(rect, CornerRadius::ZERO, fill);
        }
        p.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
    }
    let galley = caps(
        ui,
        text,
        theme::HUD_SIZE,
        theme::mono_for(ui.ctx()),
        theme::HUD_SPACING,
        ink,
    );
    paint_centred(ui, rect, galley, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A thin progress bar.
pub fn progress(ui: &mut Ui, fraction: f32) {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 4.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.sunken);
    let fill = egui::Rect::from_min_size(
        rect.min,
        vec2(rect.width() * fraction.clamp(0.0, 1.0), rect.height()),
    );
    ui.painter().rect_filled(fill, CornerRadius::ZERO, t.accent);
}
