//! Floating overlay window widget for canvases, hosting inspector sections and properties.

use egui::{pos2, vec2, Frame, Id, Margin, Pos2, Rect, ScrollArea, Sense, Ui, Vec2};

use super::controls::icon_button;
use super::primitives::{hline, hud};
use crate::icons::Icon;
use crate::theme::{self, Tokens};

/// State for a floating properties window, persisting across frames in view state.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingWindowState {
    /// Screen-space position of top-left corner. If `None`, calculated from parent bounds.
    pub pos: Option<Pos2>,
    /// Whether the window body is collapsed, showing only the header bar.
    pub collapsed: bool,
    /// Whether the window is pinned in place (ignoring drags).
    pub pinned: bool,
    /// Whether the window is visible.
    pub visible: bool,
    /// Default width of the window in pixels.
    pub width: f32,
}

impl Default for FloatingWindowState {
    fn default() -> Self {
        Self {
            pos: None,
            collapsed: false,
            pinned: false,
            visible: true,
            width: 260.0,
        }
    }
}

impl FloatingWindowState {
    /// Returns the active position, defaulting to the top-right corner of `parent_rect` with margins.
    pub fn resolve_pos(&mut self, parent_rect: Rect, window_size: Vec2) -> Pos2 {
        if let Some(p) = self.pos {
            return p;
        }
        let default_x = (parent_rect.right() - window_size.x - 16.0).max(parent_rect.left() + 8.0);
        let default_y = parent_rect.top() + 16.0;
        let p = pos2(default_x, default_y);
        self.pos = Some(p);
        p
    }

    /// Clamps the window position so it stays entirely within `parent_rect`.
    pub fn clamp_to_rect(&mut self, parent_rect: Rect, window_size: Vec2) {
        if let Some(pos) = &mut self.pos {
            let max_x = (parent_rect.right() - window_size.x).max(parent_rect.left());
            let max_y = (parent_rect.bottom() - window_size.y).max(parent_rect.top());
            pos.x = pos.x.clamp(parent_rect.left(), max_x);
            pos.y = pos.y.clamp(parent_rect.top(), max_y);
        }
    }

    /// Drags the window by `delta` if not pinned, and clamps to `parent_rect`.
    pub fn drag(&mut self, delta: Vec2, parent_rect: Rect, window_size: Vec2) {
        if self.pinned || delta == Vec2::ZERO {
            return;
        }
        let pos = self.resolve_pos(parent_rect, window_size);
        self.pos = Some(pos + delta);
        self.clamp_to_rect(parent_rect, window_size);
    }
}

/// Renders a floating properties window constrained inside `parent_rect`.
///
/// Returns the inner closure output if visible and not collapsed, or `None` if hidden.
pub fn floating_window<R>(
    ui: &mut Ui,
    id_source: impl std::hash::Hash + std::fmt::Debug,
    title: &str,
    parent_rect: Rect,
    state: &mut FloatingWindowState,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if !state.visible {
        return None;
    }

    let t = Tokens::get(ui.ctx());
    let width = state.width.max(180.0);
    let approx_size = vec2(width, if state.collapsed { 28.0 } else { 180.0 });
    let pos = state.resolve_pos(parent_rect, approx_size);

    let area_id = Id::new("floating_window").with(id_source);
    let mut inner_result = None;

    let area_response = egui::Area::new(area_id)
        .order(egui::Order::Middle)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::ZERO)
                .show(ui, |ui| {
                    ui.set_width(width);

                    // Header bar
                    let (header_rect, header_response) =
                        ui.allocate_exact_size(vec2(width, 28.0), Sense::click_and_drag());

                    let header_painter = ui.painter().with_clip_rect(header_rect);
                    header_painter.rect_filled(header_rect, 0.0, t.sunken);

                    let mut header_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(header_rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );

                    // Collapse toggle
                    let collapse_icon = if state.collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    };
                    let collapse_tip = if state.collapsed {
                        "Expand window"
                    } else {
                        "Collapse window"
                    };
                    if icon_button(&mut header_ui, collapse_icon, collapse_tip).clicked() {
                        state.collapsed = !state.collapsed;
                    }

                    // Title
                    hud(&mut header_ui, title, t.ink_strong);

                    // Right-aligned controls
                    header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        if icon_button(ui, Icon::Close, "Close window").clicked() {
                            state.visible = false;
                        }

                        // Pin button
                        let pin_icon = if state.pinned {
                            Icon::Lock
                        } else {
                            Icon::Unlock
                        };
                        let pin_tip = if state.pinned {
                            "Unpin window (allow dragging)"
                        } else {
                            "Pin window position"
                        };
                        if icon_button(ui, pin_icon, pin_tip).clicked() {
                            state.pinned = !state.pinned;
                        }
                    });

                    // Window drag interaction
                    if header_response.dragged() && !state.pinned {
                        let delta = header_response.drag_delta();
                        state.drag(delta, parent_rect, approx_size);
                    }

                    // Body contents if not collapsed
                    if !state.collapsed {
                        hline(ui);

                        let max_body_h = (parent_rect.height() - 48.0).max(60.0);
                        ScrollArea::vertical()
                            .max_height(max_body_h)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.add_space(8.0);
                                Frame::new()
                                    .inner_margin(Margin::symmetric(theme::PANEL_PAD, 0))
                                    .show(ui, |ui| {
                                        inner_result = Some(add_contents(ui));
                                    });
                                ui.add_space(8.0);
                            });
                    }
                });
        });

    let actual_size = area_response.response.rect.size();
    state.clamp_to_rect(parent_rect, actual_size);

    inner_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_initializes_cleanly() {
        let state = FloatingWindowState::default();
        assert_eq!(state.pos, None);
        assert!(!state.collapsed);
        assert!(!state.pinned);
        assert!(state.visible);
    }

    #[test]
    fn resolve_pos_and_clamp_within_parent() {
        let mut state = FloatingWindowState::default();
        let parent = Rect::from_min_size(pos2(100.0, 50.0), vec2(800.0, 600.0));
        let win_size = vec2(200.0, 150.0);

        let initial_pos = state.resolve_pos(parent, win_size);
        assert!(initial_pos.x >= parent.left());
        assert!(initial_pos.x + win_size.x <= parent.right());
        assert!(initial_pos.y >= parent.top());

        // Drag out of bounds to the right
        state.drag(vec2(1000.0, 1000.0), parent, win_size);
        assert_eq!(state.pos.unwrap().x, parent.right() - win_size.x);
        assert_eq!(state.pos.unwrap().y, parent.bottom() - win_size.y);

        // Drag out of bounds to top-left
        state.drag(vec2(-2000.0, -2000.0), parent, win_size);
        assert_eq!(state.pos.unwrap().x, parent.left());
        assert_eq!(state.pos.unwrap().y, parent.top());
    }

    #[test]
    fn pinned_window_ignores_drags() {
        let mut state = FloatingWindowState::default();
        let parent = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0));
        let win_size = vec2(100.0, 100.0);

        let initial_pos = state.resolve_pos(parent, win_size);
        state.pinned = true;
        state.drag(vec2(50.0, 50.0), parent, win_size);
        assert_eq!(state.pos.unwrap(), initial_pos);
    }

    #[test]
    fn floating_window_renders_without_panicking() {
        let ctx = egui::Context::default();
        let mut state = FloatingWindowState::default();
        let parent = Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 400.0));

        let mut output = ctx.run_ui(Default::default(), |ui| {
            let res = floating_window(ui, "test_props", "PROPERTIES", parent, &mut state, |ui| {
                ui.label("Field 1");
                42
            });
            assert_eq!(res, Some(42));
        });
        output.textures_delta.clear();
    }
}
