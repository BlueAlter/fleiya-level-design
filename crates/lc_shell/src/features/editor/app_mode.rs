//! Application modes (Editor, Enums, Tilesets) and toolbar mode switcher.

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
    /// Dedicated tileset definitions and atlas management mode.
    Tilesets,
}

/// Specification for a mode switch button in the top toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeButton {
    pub mode: AppMode,
    pub icon: Icon,
    pub tooltip: &'static str,
}

/// Available mode buttons in toolbar order.
pub const MODE_BUTTONS: [ModeButton; 3] = [
    ModeButton {
        mode: AppMode::Editor,
        icon: Icon::Level,
        tooltip: "Editor (Ctrl+1)",
    },
    ModeButton {
        mode: AppMode::Enums,
        icon: Icon::Enum,
        tooltip: "Enums (Ctrl+2)",
    },
    ModeButton {
        mode: AppMode::Tilesets,
        icon: Icon::Tileset,
        tooltip: "Tilesets (Ctrl+3)",
    },
];

/// Computes the total width required by a segmented control of N buttons and gaps.
pub fn control_width(button_count: usize, button_size: f32, gap: f32) -> f32 {
    if button_count == 0 {
        0.0
    } else {
        (button_count as f32) * button_size + (button_count - 1) as f32 * gap
    }
}

/// Returns true if the unsaved indicator dot should be shown for the given mode.
pub fn shows_unsaved(mode: AppMode, enum_unsaved: bool) -> bool {
    matches!(mode, AppMode::Enums) && enum_unsaved
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
    let total_width = control_width(MODE_BUTTONS.len(), button_size, gap);

    let avail_width = ui.available_width();
    let offset = ((avail_width - total_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(offset);
        ui.spacing_mut().item_spacing.x = gap;

        for btn in MODE_BUTTONS {
            let is_active = *active_mode == btn.mode;
            let show_dot = shows_unsaved(btn.mode, enum_unsaved);
            let resp = render_mode_button(
                ui,
                btn.icon,
                is_active,
                show_dot,
                button_size,
                btn.tooltip,
                t,
            );
            if resp.clicked() {
                *active_mode = btn.mode;
            }
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
