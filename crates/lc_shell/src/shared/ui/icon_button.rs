//! Standardized square icon button widget without padding expansion traps.
//!
//! Unlike standard `egui::Button`, [`icon_button`] avoids the project theme's
//! horizontal button padding (`style.spacing.button_padding.x = 10.0`), guaranteeing
//! exact requested square dimensions.
//!
//! # Usage Example
//! ```ignore
//! use lc_canvas::icons::Icon;
//! use lc_shell::shared::ui::icon_button::{icon_button, IconButtonStyle};
//!
//! let resp = icon_button(ui, Icon::DotsThreeVertical, 28.0, IconButtonStyle::ghost());
//! if resp.clicked() {
//!     // handle click
//! }
//! ```

use egui::{vec2, Color32, CornerRadius, CursorIcon, Response, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

/// Style configuration for [`icon_button`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconButtonStyle {
    /// Destructive action flag (colours red on hover/press).
    pub danger: bool,
    /// Whether the button is actively selected or toggled on.
    pub selected: bool,
    /// Whether the button has a visible border and background fill when idle.
    pub bordered: bool,
    /// Corner radius for the button background and border.
    pub corner_radius: CornerRadius,
}

const DEFAULT_CORNER_RADIUS: CornerRadius = CornerRadius {
    nw: 3,
    ne: 3,
    sw: 3,
    se: 3,
};

impl Default for IconButtonStyle {
    fn default() -> Self {
        Self {
            danger: false,
            selected: false,
            bordered: true,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }
}

impl IconButtonStyle {
    /// Creates a standard bordered icon button style.
    pub const fn bordered() -> Self {
        Self {
            danger: false,
            selected: false,
            bordered: true,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }

    /// Creates a ghost (transparent when idle) icon button style.
    pub const fn ghost() -> Self {
        Self {
            danger: false,
            selected: false,
            bordered: false,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }

    /// Creates a danger (red hover) icon button style.
    pub const fn danger() -> Self {
        Self {
            danger: true,
            selected: false,
            bordered: true,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }

    /// Creates a ghost danger icon button style (transparent idle, red hover).
    pub const fn ghost_danger() -> Self {
        Self {
            danger: true,
            selected: false,
            bordered: false,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }

    /// Sets whether the button is selected.
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides corner radius to zero (sharp rectangle).
    pub const fn square(mut self) -> Self {
        self.corner_radius = CornerRadius::ZERO;
        self
    }

    /// Sets a custom corner radius.
    pub const fn with_corner_radius(mut self, radius: CornerRadius) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// Renders a square icon button of exact dimensions `size x size`.
///
/// Bypasses egui `button_padding` using `allocate_exact_size` and custom painting.
pub fn icon_button(ui: &mut Ui, glyph: Icon, size: f32, style: IconButtonStyle) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let t = Tokens::get(ui.ctx());
        let hovered = response.hovered();
        let pressed = response.is_pointer_button_down_on();

        let (bg, stroke_color, icon_color) = if style.danger {
            if pressed || hovered {
                (t.bad, t.bad, t.invert_fg)
            } else if style.bordered {
                (t.sunken, t.line, t.bad)
            } else {
                (Color32::TRANSPARENT, Color32::TRANSPARENT, t.muted)
            }
        } else if style.selected {
            (t.accent, t.line_strong, t.invert_fg)
        } else if pressed || hovered {
            (t.sunken, t.line_strong, t.ink_strong)
        } else if style.bordered {
            (t.sunken, t.line, t.ink)
        } else {
            (Color32::TRANSPARENT, Color32::TRANSPARENT, t.muted)
        };

        let painter = ui.painter();
        if bg != Color32::TRANSPARENT {
            painter.rect_filled(rect, style.corner_radius, bg);
        }
        if stroke_color != Color32::TRANSPARENT {
            painter.rect_stroke(
                rect,
                style.corner_radius,
                Stroke::new(1.0, stroke_color),
                StrokeKind::Inside,
            );
        }

        let icon_glyph_size = if size >= 28.0 {
            14.0
        } else if size >= 20.0 {
            13.0
        } else {
            11.0
        };
        paint_icon(painter, rect.center(), glyph, icon_glyph_size, icon_color);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_icon_button_style_constructors() {
        let b = IconButtonStyle::bordered();
        assert!(b.bordered && !b.danger && !b.selected);

        let g = IconButtonStyle::ghost();
        assert!(!g.bordered && !g.danger && !g.selected);

        let d = IconButtonStyle::danger();
        assert!(d.bordered && d.danger && !d.selected);

        let gd = IconButtonStyle::ghost_danger();
        assert!(!gd.bordered && gd.danger && !gd.selected);
    }

    #[test]
    fn test_icon_button_exact_rect_sizes_under_theme() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let sizes = [20.0, 24.0, 28.0, 32.0];
        let mut measured_sizes = Vec::new();

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            for &size in &sizes {
                let resp = icon_button(
                    ui,
                    Icon::DotsThreeVertical,
                    size,
                    IconButtonStyle::bordered(),
                );
                measured_sizes.push(resp.rect.size());
            }
        });

        output.textures_delta.clear();

        for (i, &expected_size) in sizes.iter().enumerate() {
            let actual = measured_sizes[i];
            assert!(
                (actual.x - expected_size).abs() < 0.01,
                "Width for size {} was {}, expected {}",
                expected_size,
                actual.x,
                expected_size
            );
            assert!(
                (actual.y - expected_size).abs() < 0.01,
                "Height for size {} was {}, expected {}",
                expected_size,
                actual.y,
                expected_size
            );
        }
    }
}
