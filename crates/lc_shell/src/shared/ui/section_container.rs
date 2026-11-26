//! Shared section container with accent title, optional toolbar slot, optional collapse toggle,
//! flexible outline, mandatory square corners (`CornerRadius::ZERO`), and focus selection state.

use egui::{
    vec2, Align, Color32, CornerRadius, Frame, Layout, Margin, Rect, RichText, Sense, Stroke, Ui,
    Vec2,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

use super::icon_button::{icon_button, IconButtonStyle};

/// Default height of the section header row when no collapse toggle is present.
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;

/// Default height of the section header row when a 28x28 collapse toggle is present.
pub const SECTION_HEADER_HEIGHT_COLLAPSIBLE: f32 = 28.0;

/// Header state returned by [`SectionContainer::show_header`].
#[derive(Debug, Clone, Copy)]
pub struct SectionHeaderResponse {
    /// Bounding rectangle of the header row.
    pub header_rect: Rect,
    /// Bounding rectangle of the collapse toggle button if rendered.
    pub toggle_rect: Option<Rect>,
    /// `true` if the collapse toggle button was clicked this frame.
    pub toggled: bool,
    /// `true` if the section is currently collapsed.
    pub collapsed: bool,
}

/// Response returned by [`SectionContainer::show`] and [`SectionContainer::show_body`].
#[derive(Debug)]
pub struct SectionResponse<R> {
    /// Inner return value from the body closure, or `None` when collapsed.
    pub inner: Option<R>,
    /// `true` if the user pressed the pointer inside this section's frame or header (excluding the toggle button).
    pub clicked_inside: bool,
    /// `true` if the user clicked the collapse toggle button this frame.
    pub toggled: bool,
    /// Bounding rectangle of the body frame when expanded.
    pub frame_rect: Option<Rect>,
}

/// Reusable square-cornered (`CornerRadius::ZERO`) section container with header and optional body outline.
pub struct SectionContainer<'a> {
    title: &'a str,
    title_tooltip: Option<&'a str>,
    use_outline: bool,
    use_toggle_collapse: bool,
    use_custom_color: bool,
    selected: bool,
    fill: Option<Color32>,
    margin: Margin,
    header_height: Option<f32>,
    header_gap: f32,
    size: Option<Vec2>,
}

impl<'a> SectionContainer<'a> {
    /// Creates a new section container with `CornerRadius::ZERO` and default card styling.
    ///
    /// Defaults:
    /// - `use_outline = true`
    /// - `use_toggle_collapse = false`
    /// - `selected = false`
    /// - `fill = None` (uses `t.panel`)
    /// - `margin = Margin::same(12)`
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            title_tooltip: None,
            use_outline: true,
            use_toggle_collapse: false,
            use_custom_color: true,
            selected: false,
            fill: None,
            margin: Margin::same(12),
            header_height: None,
            header_gap: 4.0,
            size: None,
        }
    }

    /// Sets an optional hover tooltip on the section title label.
    pub fn title_tooltip(mut self, tooltip: Option<&'a str>) -> Self {
        self.title_tooltip = tooltip;
        self
    }

    /// Controls whether the body container draws a 1 px border (`true`) or no border (`false`).
    pub fn use_outline(mut self, use_outline: bool) -> Self {
        self.use_outline = use_outline;
        self
    }

    /// Controls whether the collapse/expand toggle button is shown on the right of the header.
    ///
    /// When `false`, the toggle button is hidden.
    pub fn use_toggle_collapse(mut self, use_toggle: bool) -> Self {
        self.use_toggle_collapse = use_toggle;
        self
    }

    /// Marks the container as selected/focused, highlighting its outline in `t.accent` when `use_outline` is `true`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides the container background fill (defaults to `t.panel` when `use_custom_color` is `true`).
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Controls whether the container paints its default/custom background fill (`true`)
    /// or stays transparent outline-only (`false`, unless `.fill(...)` is explicitly set).
    pub fn use_custom_color(mut self, use_custom_color: bool) -> Self {
        self.use_custom_color = use_custom_color;
        self
    }

    /// Sets inner padding inside the body container (defaults to `Margin::same(12)`).
    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    /// Overrides the header row height (defaults to `28.0` when collapsible, `24.0` otherwise).
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = Some(height);
        self
    }

    /// Sets the vertical gap between the header row and the body frame (defaults to `4.0`).
    pub fn header_gap(mut self, gap: f32) -> Self {
        self.header_gap = gap;
        self
    }

    /// Constrains the body container to an explicit size (`width`, `height`).
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Renders only the header row so callers can borrow mutable state separately in header and body.
    pub fn show_header(
        &self,
        ui: &mut Ui,
        t: &Tokens,
        collapsed: &mut bool,
        header_extra: impl FnOnce(&mut Ui),
    ) -> SectionHeaderResponse {
        let header_h = self.header_height.unwrap_or(if self.use_toggle_collapse {
            SECTION_HEADER_HEIGHT_COLLAPSIBLE
        } else {
            SECTION_HEADER_HEIGHT
        });

        let avail_w = self
            .size
            .map_or_else(|| ui.available_width(), |s| s.x)
            .max(1.0);
        let (header_rect, _) = ui.allocate_exact_size(vec2(avail_w, header_h), Sense::hover());

        let mut toggled = false;
        let mut toggle_rect: Option<Rect> = None;

        let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));
        header_ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let title_resp = ui.add(
                egui::Label::new(
                    RichText::new(self.title)
                        .size(11.0)
                        .strong()
                        .color(t.accent_ink),
                )
                .sense(Sense::hover()),
            );
            if let Some(tip) = self.title_tooltip {
                title_resp.on_hover_text(tip);
            }

            if self.use_toggle_collapse {
                // Reserve the right 28x28 slot for the collapse toggle button.
                let toggle_slot_w = 28.0;
                let extra_avail = (ui.available_width() - toggle_slot_w - 6.0).max(0.0);
                if extra_avail > 0.0 {
                    ui.allocate_ui_with_layout(
                        vec2(extra_avail, header_h),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            header_extra(ui);
                        },
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let eye_icon = if *collapsed { Icon::EyeOff } else { Icon::Eye };
                    let tooltip = if *collapsed {
                        "Show section"
                    } else {
                        "Hide section"
                    };
                    let btn = icon_button(ui, eye_icon, 28.0, IconButtonStyle::bordered())
                        .on_hover_text(tooltip);
                    toggle_rect = Some(btn.rect);
                    if btn.clicked() {
                        *collapsed = !*collapsed;
                        toggled = true;
                    }
                });
            } else {
                header_extra(ui);
            }
        });

        SectionHeaderResponse {
            header_rect,
            toggle_rect,
            toggled,
            collapsed: self.use_toggle_collapse && *collapsed,
        }
    }

    /// Renders the body frame after [`Self::show_header`].
    pub fn show_body<R>(
        &self,
        ui: &mut Ui,
        t: &Tokens,
        header: &SectionHeaderResponse,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> SectionResponse<R> {
        let pointer_pressed_pos = if ui.input(|i| i.pointer.any_pressed()) {
            ui.input(|i| i.pointer.interact_pos())
        } else {
            None
        };

        if header.collapsed {
            let clicked_inside = pointer_pressed_pos.is_some_and(|pos| {
                header.header_rect.contains(pos)
                    && !header.toggle_rect.is_some_and(|tr| tr.contains(pos))
            });
            return SectionResponse {
                inner: None,
                clicked_inside,
                toggled: header.toggled,
                frame_rect: None,
            };
        }

        if self.header_gap > 0.0 {
            ui.add_space(self.header_gap);
        }

        let stroke = if !self.use_outline {
            Stroke::NONE
        } else if self.selected {
            Stroke::new(1.0, t.accent)
        } else {
            t.hairline()
        };

        let default_fill = if self.use_custom_color {
            t.panel
        } else {
            Color32::TRANSPARENT
        };
        let fill = self.fill.unwrap_or(default_fill);
        let frame = Frame::new()
            .fill(fill)
            .stroke(stroke)
            .inner_margin(self.margin)
            .corner_radius(CornerRadius::ZERO);

        let (inner, frame_rect) = if let Some(target_size) = self.size {
            let stroke_pad = stroke.width * 2.0;
            let resp = ui.allocate_ui(target_size, |ui| {
                let frame_resp = frame.show(ui, |ui| {
                    ui.set_min_size(vec2(
                        ui.available_width().max(0.0),
                        (target_size.y - stroke_pad).max(0.0),
                    ));
                    body(ui)
                });
                (frame_resp.inner, frame_resp.response.rect)
            });
            resp.inner
        } else {
            let frame_resp = frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                body(ui)
            });
            (frame_resp.inner, frame_resp.response.rect)
        };

        let clicked_inside = pointer_pressed_pos.is_some_and(|pos| {
            (frame_rect.contains(pos) || header.header_rect.contains(pos))
                && !header.toggle_rect.is_some_and(|tr| tr.contains(pos))
        });

        SectionResponse {
            inner: Some(inner),
            clicked_inside,
            toggled: header.toggled,
            frame_rect: Some(frame_rect),
        }
    }

    /// Renders the section header (with optional `header_extra` slot next to the title) and body.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        t: &Tokens,
        collapsed: &mut bool,
        header_extra: impl FnOnce(&mut Ui),
        body: impl FnOnce(&mut Ui) -> R,
    ) -> SectionResponse<R> {
        let header = self.show_header(ui, t, collapsed, header_extra);
        self.show_body(ui, t, &header, body)
    }
}
