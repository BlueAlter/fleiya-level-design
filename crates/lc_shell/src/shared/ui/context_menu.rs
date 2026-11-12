//! Unified context menu theme and component for Level Craft / Fleiya.
//!
//! Provides a single theme ([`ContextMenuTheme`]) and shared functions ([`show_context_menu`])
//! so that all context menus, dropdown frames, and action menus have consistent styling,
//! 0 corner radius (sharp rectangle), flat full-row hover states, content-derived widths,
//! and predictable placement.

use egui::{
    emath, Align2, Color32, CornerRadius, FontId, Frame, Id, Margin, Popup, Pos2, Response, Sense,
    Stroke, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

/// Minimum width for all context menus.
pub const MENU_MIN_WIDTH: f32 = 130.0;

/// Maximum width for all context menus.
pub const MENU_MAX_WIDTH: f32 = 240.0;

/// Theme definition for all context menus in the app.
///
/// Visual parameters are defined only here, with no hardcoded values at call sites.
#[derive(Clone, Debug)]
pub struct ContextMenuTheme {
    // Frame appearance
    pub frame_fill: Option<Color32>,
    pub frame_stroke: Option<Stroke>,
    pub corner_radius: CornerRadius,
    pub inner_margin: Margin,
    pub shadow: egui::epaint::Shadow,

    // Dimensions
    pub min_width: f32,
    pub max_width: f32,
    pub item_height: f32,
    pub separator_height: f32,

    // Item layout
    pub padding_x: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub font_size: f32,

    // Colors
    pub text_color: Option<Color32>,
    pub hover_fill: Option<Color32>,
    pub destructive_color: Color32,
    pub destructive_hover_fill: Color32,
}

impl Default for ContextMenuTheme {
    fn default() -> Self {
        Self {
            frame_fill: None,                  // Resolves to t.panel
            frame_stroke: None,                // Resolves to t.hairline()
            corner_radius: CornerRadius::ZERO, // Sharp rectangle (rounding 0)
            inner_margin: Margin::symmetric(0, 4),
            shadow: egui::epaint::Shadow {
                offset: [0, 4],
                blur: 8,
                spread: 1,
                color: Color32::from_black_alpha(100),
            },
            min_width: MENU_MIN_WIDTH,
            max_width: MENU_MAX_WIDTH,
            item_height: 26.0,
            separator_height: 5.0,
            padding_x: 10.0,
            icon_size: 13.0,
            icon_gap: 8.0,
            font_size: 12.0,
            text_color: None, // Resolves to t.ink_strong
            hover_fill: None, // Resolves to t.sunken
            destructive_color: Color32::from_rgb(239, 68, 68),
            destructive_hover_fill: Color32::from_rgba_unmultiplied(239, 68, 68, 32),
        }
    }
}

impl ContextMenuTheme {
    /// Creates an `egui::Frame` matching this context menu theme.
    ///
    /// Can also be used directly for popups that share the menu appearance (such as tag/tileset dropdowns).
    pub fn frame(&self, t: &Tokens) -> Frame {
        Frame::new()
            .fill(self.frame_fill.unwrap_or(t.panel))
            .stroke(self.frame_stroke.unwrap_or_else(|| t.hairline()))
            .corner_radius(self.corner_radius)
            .inner_margin(self.inner_margin)
            .shadow(self.shadow)
    }

    /// Computes the menu width clamped between `min_width` and `max_width`
    /// based on measuring the widest label text.
    pub fn compute_menu_width<T: Clone + PartialEq>(
        &self,
        ctx: &egui::Context,
        items: &[MenuItem<T>],
    ) -> f32 {
        let mut widest_label: f32 = 0.0;
        let font_size = self.font_size;
        for item in items {
            if let MenuItem::Action { label, .. } = item {
                let text_w = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    ctx.fonts_mut(|f| {
                        f.layout_no_wrap(
                            label.clone(),
                            FontId::proportional(font_size),
                            Color32::WHITE,
                        )
                        .size()
                        .x
                    })
                }))
                .unwrap_or_else(|_| label.chars().count() as f32 * font_size * 0.6);

                widest_label = widest_label.max(text_w);
            }
        }
        let content_w = widest_label + self.icon_size + self.icon_gap + 2.0 * self.padding_x;
        content_w.clamp(self.min_width, self.max_width)
    }
}

/// An item in a context menu.
#[derive(Clone, Debug)]
pub enum MenuItem<T: Clone + PartialEq> {
    /// Action item with an ID, text label, icon, and destructive flag.
    Action {
        id: T,
        label: String,
        icon: Icon,
        destructive: bool,
    },
    /// Visual separator line between items.
    Separator,
}

impl<T: Clone + PartialEq> MenuItem<T> {
    /// Creates a normal action menu item.
    pub fn action(id: T, label: impl Into<String>, icon: Icon) -> Self {
        Self::Action {
            id,
            label: label.into(),
            icon,
            destructive: false,
        }
    }

    /// Creates a destructive action menu item (styled in red).
    pub fn destructive(id: T, label: impl Into<String>, icon: Icon) -> Self {
        Self::Action {
            id,
            label: label.into(),
            icon,
            destructive: true,
        }
    }

    /// Creates a separator item.
    pub fn separator() -> Self {
        Self::Separator
    }
}

/// Anchor specification for context menus.
#[derive(Clone, Copy)]
pub enum MenuAnchor<'a> {
    /// Button trigger with toggle behavior: appears directly below the button,
    /// right-aligned to the button's right edge so it never overflows the panel.
    Button {
        response: &'a Response,
        popup_id: Option<Id>,
    },
    /// Right-click context menu on a widget: appears at the cursor position.
    ContextMenu {
        response: &'a Response,
        popup_id: Option<Id>,
    },
    /// Render directly inside an existing open menu popup UI.
    InsideUi,
}

impl<'a> MenuAnchor<'a> {
    pub fn button(response: &'a Response) -> Self {
        Self::Button {
            response,
            popup_id: None,
        }
    }

    pub fn button_with_id(response: &'a Response, popup_id: Id) -> Self {
        Self::Button {
            response,
            popup_id: Some(popup_id),
        }
    }

    pub fn context_menu(response: &'a Response) -> Self {
        Self::ContextMenu {
            response,
            popup_id: None,
        }
    }

    pub fn context_menu_with_id(response: &'a Response, popup_id: Id) -> Self {
        Self::ContextMenu {
            response,
            popup_id: Some(popup_id),
        }
    }

    pub fn inside_ui() -> Self {
        Self::InsideUi
    }
}

/// Shows a context menu with the default [`ContextMenuTheme`].
///
/// Returns `Some(action_id)` if an action was clicked, or `None` otherwise.
/// The action is returned as a value and should be executed AFTER drawing finishes.
pub fn show_context_menu<T: Clone + PartialEq>(
    ui: &mut Ui,
    anchor: MenuAnchor<'_>,
    items: &[MenuItem<T>],
) -> Option<T> {
    show_context_menu_with_theme(ui, anchor, &ContextMenuTheme::default(), items)
}

/// Shows a context menu with a custom [`ContextMenuTheme`].
pub fn show_context_menu_with_theme<T: Clone + PartialEq>(
    ui: &mut Ui,
    anchor: MenuAnchor<'_>,
    theme: &ContextMenuTheme,
    items: &[MenuItem<T>],
) -> Option<T> {
    let t = Tokens::get(ui.ctx());
    let menu_w = theme.compute_menu_width(ui.ctx(), items);

    match anchor {
        MenuAnchor::Button { response, popup_id } => {
            let id = popup_id.unwrap_or_else(|| response.id.with("context_menu_popup"));
            Popup::menu(response)
                .id(id)
                .align(emath::RectAlign::BOTTOM_END)
                .align_alternatives(&[
                    emath::RectAlign::TOP_END,
                    emath::RectAlign::BOTTOM_START,
                    emath::RectAlign::TOP_START,
                ])
                .frame(theme.frame(&t))
                .width(menu_w)
                .show(|popup_ui| show_context_menu_contents(popup_ui, theme, &t, menu_w, items))
                .and_then(|r| r.inner)
        }
        MenuAnchor::ContextMenu { response, popup_id } => {
            let id = popup_id.unwrap_or_else(|| response.id.with("context_menu_popup"));
            Popup::context_menu(response)
                .id(id)
                .align(emath::RectAlign::BOTTOM_START)
                .align_alternatives(&[
                    emath::RectAlign::TOP_START,
                    emath::RectAlign::BOTTOM_END,
                    emath::RectAlign::TOP_END,
                ])
                .frame(theme.frame(&t))
                .width(menu_w)
                .show(|popup_ui| show_context_menu_contents(popup_ui, theme, &t, menu_w, items))
                .and_then(|r| r.inner)
        }
        MenuAnchor::InsideUi => show_context_menu_contents(ui, theme, &t, menu_w, items),
    }
}

/// Renders the list of menu items inside an open popup or container.
pub fn show_context_menu_contents<T: Clone + PartialEq>(
    ui: &mut Ui,
    theme: &ContextMenuTheme,
    t: &Tokens,
    menu_w: f32,
    items: &[MenuItem<T>],
) -> Option<T> {
    ui.set_width(menu_w);
    ui.set_max_width(menu_w);
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

    let mut selected_action: Option<T> = None;

    for item in items {
        match item {
            MenuItem::Separator => {
                let (sep_rect, _) = ui.allocate_exact_size(
                    egui::vec2(menu_w, theme.separator_height),
                    Sense::hover(),
                );
                let y = sep_rect.center().y;
                ui.painter().hline(
                    (sep_rect.min.x + theme.padding_x)..=(sep_rect.max.x - theme.padding_x),
                    y,
                    Stroke::new(1.0, t.muted.linear_multiply(0.2)),
                );
            }
            MenuItem::Action {
                id,
                label,
                icon,
                destructive,
            } => {
                let (item_rect, item_resp) =
                    ui.allocate_exact_size(egui::vec2(menu_w, theme.item_height), Sense::click());
                let is_hovered = item_resp.hovered();

                // Hover highlight: flat fill across the full row, no per-item borders
                if is_hovered {
                    let fill = if *destructive {
                        theme.destructive_hover_fill
                    } else {
                        theme.hover_fill.unwrap_or(t.sunken)
                    };
                    ui.painter()
                        .rect_filled(item_rect, theme.corner_radius, fill);
                }

                // Determine colors
                let (fg_color, icon_color) = if *destructive {
                    (theme.destructive_color, theme.destructive_color)
                } else if is_hovered {
                    (t.ink_strong, t.ink_strong)
                } else {
                    (theme.text_color.unwrap_or(t.ink), t.muted)
                };

                let cy = item_rect.center().y;

                // Icon on left
                let icon_center = Pos2::new(
                    item_rect.min.x + theme.padding_x + theme.icon_size / 2.0,
                    cy,
                );
                paint_icon(
                    ui.painter(),
                    icon_center,
                    *icon,
                    theme.icon_size,
                    icon_color,
                );

                // Label
                let text_start_x =
                    item_rect.min.x + theme.padding_x + theme.icon_size + theme.icon_gap;
                ui.painter().text(
                    Pos2::new(text_start_x, cy),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(theme.font_size),
                    fg_color,
                );

                if item_resp.clicked() {
                    selected_action = Some(id.clone());
                    ui.close();
                }
            }
        }
    }

    selected_action
}

/// Checks whether a menu popup with the specified `Id` is currently open in egui memory.
pub fn is_menu_open(ctx: &egui::Context, popup_id: Id) -> bool {
    Popup::is_id_open(ctx, popup_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_theme_defaults() {
        let theme = ContextMenuTheme::default();
        assert_eq!(theme.corner_radius, CornerRadius::ZERO);
        assert_eq!(theme.min_width, MENU_MIN_WIDTH);
        assert_eq!(theme.max_width, MENU_MAX_WIDTH);
        assert_eq!(theme.item_height, 26.0);
        assert_eq!(theme.font_size, 12.0);
        assert_eq!(theme.icon_size, 13.0);
        assert_eq!(theme.icon_gap, 8.0);
        assert_eq!(theme.padding_x, 10.0);
    }

    #[test]
    fn test_menu_width_computation() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        let theme = ContextMenuTheme::default();

        // 1. Very short items are clamped to MENU_MIN_WIDTH (130px)
        let short_items = [
            MenuItem::action("act1", "A", Icon::Copy),
            MenuItem::action("act2", "B", Icon::Delete),
        ];
        let w_short = theme.compute_menu_width(&ctx, &short_items);
        assert_eq!(w_short, MENU_MIN_WIDTH);

        // 2. Normal items fit content
        let normal_items = [
            MenuItem::action("dup", "Duplicate", Icon::Copy),
            MenuItem::action("del", "Delete", Icon::Delete),
        ];
        let w_normal = theme.compute_menu_width(&ctx, &normal_items);
        assert!(w_normal >= MENU_MIN_WIDTH);
        assert!(w_normal <= MENU_MAX_WIDTH);

        // 3. Very long items are clamped to MENU_MAX_WIDTH (240px)
        let long_items = [
            MenuItem::action(
                "long",
                "This is an extraordinarily long action label that definitely exceeds the maximum menu width",
                Icon::File,
            ),
        ];
        let w_long = theme.compute_menu_width(&ctx, &long_items);
        assert_eq!(w_long, MENU_MAX_WIDTH);
    }

    #[test]
    fn test_menu_item_constructors() {
        let a = MenuItem::action(1, "Test Action", Icon::Copy);
        match a {
            MenuItem::Action {
                id,
                label,
                icon,
                destructive,
            } => {
                assert_eq!(id, 1);
                assert_eq!(label, "Test Action");
                assert_eq!(icon, Icon::Copy);
                assert!(!destructive);
            }
            MenuItem::Separator => panic!("Expected Action"),
        }

        let d = MenuItem::destructive(2, "Delete Item", Icon::Delete);
        match d {
            MenuItem::Action {
                id,
                label,
                icon,
                destructive,
            } => {
                assert_eq!(id, 2);
                assert_eq!(label, "Delete Item");
                assert_eq!(icon, Icon::Delete);
                assert!(destructive);
            }
            MenuItem::Separator => panic!("Expected Action"),
        }

        let s: MenuItem<i32> = MenuItem::separator();
        match s {
            MenuItem::Separator => {}
            MenuItem::Action { .. } => panic!("Expected Separator"),
        }
    }

    #[test]
    fn test_context_menu_frame_helper() {
        let ctx = egui::Context::default();
        let t = Tokens::get(&ctx);
        let theme = ContextMenuTheme::default();
        let frame = theme.frame(&t);
        assert_eq!(frame.corner_radius, CornerRadius::ZERO);
    }
}
