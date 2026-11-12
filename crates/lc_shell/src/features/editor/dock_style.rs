//! `egui_dock` styling derived from the editor design tokens.

use egui::CornerRadius;
use egui_dock::Style;

use lc_canvas::theme::{self, Tokens};

/// Builds the dock style for the current appearance.
pub fn style(t: &Tokens) -> Style {
    let mut s = Style {
        dock_area_padding: Some(egui::Margin::same(0)),
        main_surface_border_stroke: egui::Stroke::new(1.0, t.line),
        main_surface_border_rounding: CornerRadius::ZERO,
        ..Default::default()
    };

    s.separator.width = 1.0;
    s.separator.extra_interact_width = 4.0;
    s.separator.color_idle = t.line;
    s.separator.color_hovered = t.accent;
    s.separator.color_dragged = t.accent;

    s.tab_bar.bg_fill = t.panel;
    s.tab_bar.height = theme::TAB_H;
    s.tab_bar.inner_margin = egui::Margin::symmetric(8, 0);
    s.tab_bar.corner_radius = CornerRadius::ZERO;
    s.tab_bar.hline_color = t.line;
    s.tab_bar.fill_tab_bar = false;
    s.tab_bar.show_scroll_bar_on_overflow = false;

    s.tab.active.bg_fill = t.bg;
    s.tab.active.text_color = t.ink_strong;
    s.tab.active.outline_color = t.accent;
    s.tab.inactive.bg_fill = t.panel;
    s.tab.inactive.text_color = t.muted;
    s.tab.inactive.outline_color = t.line;
    s.tab.hovered.bg_fill = t.sunken;
    s.tab.hovered.text_color = t.ink;
    s.tab.hovered.outline_color = t.line_strong;
    s.tab.focused.bg_fill = t.sunken;
    s.tab.focused.text_color = t.ink_strong;
    s.tab.focused.outline_color = t.accent;
    s.tab.hline_below_active_tab_name = true;
    s.tab.spacing = 0.0;
    s.tab.minimum_width = Some(84.0);
    s.tab.tab_body.bg_fill = t.bg;
    s.tab.tab_body.stroke = egui::Stroke::new(1.0, t.line);
    s.tab.tab_body.inner_margin = egui::Margin::same(0);
    s.tab.tab_body.corner_radius = CornerRadius::ZERO;

    s.overlay.selection_color = theme::tint(t.accent, 0.18);
    s.overlay.selection_stroke_width = 1.0;
    s.overlay.button_spacing = 4.0;
    s.overlay.max_button_size = 28.0;
    s.overlay.button_color = t.bg;
    s.overlay.button_border_stroke = egui::Stroke::new(1.0, t.line);
    s.overlay.hovered_leaf_highlight.color = theme::tint(t.accent, 0.08);
    s.overlay.hovered_leaf_highlight.stroke = egui::Stroke::new(1.0, t.accent);
    s.overlay.hovered_leaf_highlight.corner_radius = CornerRadius::ZERO;
    s.overlay.hovered_leaf_highlight.expansion = 0.0;
    s.overlay.surface_fade_opacity = 0.2;

    s.buttons.close_tab_color = t.muted;
    s.buttons.close_tab_active_color = t.ink_strong;
    s.buttons.close_tab_bg_fill = egui::Color32::TRANSPARENT;
    s.buttons.add_tab_color = t.accent;
    s.buttons.add_tab_active_color = t.accent_ink;
    s.buttons.add_tab_bg_fill = egui::Color32::TRANSPARENT;
    s.buttons.add_tab_border_color = t.line;
    s.buttons.close_all_tabs_color = t.muted;
    s.buttons.close_all_tabs_active_color = t.ink_strong;
    s.buttons.close_all_tabs_bg_fill = egui::Color32::TRANSPARENT;
    s.buttons.close_all_tabs_border_color = t.line;
    s.buttons.close_all_tabs_disabled_color = t.faint;
    s.buttons.collapse_tabs_color = t.muted;
    s.buttons.collapse_tabs_active_color = t.ink_strong;
    s.buttons.collapse_tabs_bg_fill = egui::Color32::TRANSPARENT;
    s.buttons.collapse_tabs_border_color = t.line;
    s.buttons.minimize_window_color = t.muted;
    s.buttons.minimize_window_active_color = t.ink_strong;
    s.buttons.minimize_window_bg_fill = egui::Color32::TRANSPARENT;
    s.buttons.minimize_window_border_color = t.line;
    s.buttons.show_tab_bar_color = t.muted;
    s.buttons.show_tab_bar_active_color = t.ink_strong;
    s.buttons.show_tab_bar_size = 12.0;
    s.buttons.show_tab_bar_hover_expand = 1.0;

    s
}
