//! Application modes (Editor, Enums) and toolbar mode switcher.

use egui::{
    vec2, Align2, CornerRadius, CursorIcon, FontId, Response, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

/// Top-level application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Level and world editor with dock panels.
    #[default]
    Editor,
    /// Dedicated enum definitions management mode.
    Enums,
}

/// Renders the mode switcher segmented control in the center of the toolbar.
pub fn render_mode_segmented_control(
    ui: &mut Ui,
    active_mode: &mut AppMode,
    enum_unsaved: bool,
    t: &Tokens,
) {
    let button_size = 26.0;
    let gap = 4.0;
    let control_width = button_size * 2.0 + gap;

    let avail_width = ui.available_width();
    let offset = ((avail_width - control_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(offset);
        ui.spacing_mut().item_spacing.x = gap;

        // Editor mode button
        let is_editor = *active_mode == AppMode::Editor;
        let editor_resp = render_mode_button(
            ui,
            Icon::Level,
            is_editor,
            false,
            button_size,
            "Editor (Ctrl+1)",
            t,
        );
        if editor_resp.clicked() {
            *active_mode = AppMode::Editor;
        }

        // Enums mode button
        let is_enums = *active_mode == AppMode::Enums;
        let enums_resp = render_mode_button(
            ui,
            Icon::Enum,
            is_enums,
            enum_unsaved,
            button_size,
            "Enums (Ctrl+2)",
            t,
        );
        if enums_resp.clicked() {
            *active_mode = AppMode::Enums;
        }
    });
}

fn render_mode_button(
    ui: &mut Ui,
    icon: Icon,
    is_active: bool,
    show_unsaved: bool,
    size: f32,
    tooltip: &str,
    t: &Tokens,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let response = response
        .on_hover_text(tooltip)
        .on_hover_cursor(CursorIcon::PointingHand);

    let (bg, stroke, icon_color) = if is_active {
        (t.accent, Stroke::NONE, t.invert_fg)
    } else if response.hovered() {
        (t.sunken, t.hairline(), t.ink_strong)
    } else {
        (t.panel, t.hairline(), t.muted)
    };

    ui.painter()
        .rect(rect, CornerRadius::same(3), bg, stroke, StrokeKind::Inside);

    // Icon glyph
    let icon_pos = rect.center();
    let glyph = icon.glyph();
    let font_id = FontId::new(14.0, lc_canvas::icons::family(ui.ctx()));
    ui.painter()
        .text(icon_pos, Align2::CENTER_CENTER, glyph, font_id, icon_color);

    // Unsaved indicator dot
    if show_unsaved {
        let dot_pos = egui::pos2(rect.max.x - 5.0, rect.min.y + 5.0);
        let dot_color = if is_active { t.invert_fg } else { t.warn };
        ui.painter().circle_filled(dot_pos, 3.0, dot_color);
    }

    response
}
