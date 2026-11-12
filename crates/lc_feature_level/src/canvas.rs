//! Canvas host for level documents, providing interactive pan/zoom, grid, layer rendering and tool dispatch.

use egui::{vec2, CursorIcon, Frame, Key, Margin, PointerButton, Rect, Sense, StrokeKind, Ui};
use glam::IVec2;
use lc_canvas::grid::{draw_checkerboard, draw_grid};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::controls::icon_button;
use lc_canvas::widgets::floating_window::{floating_window, FloatingWindowState};
use lc_canvas::widgets::primitives::hud;
use lc_core::level::Level;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool, LayerHandler, LayerRenderCtx};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::viewport::Viewport;

/// Transient state for level canvas view, managing viewport and floating properties.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelCanvasState {
    /// 2D coordinate viewport.
    pub viewport: Viewport,
    /// Floating inspector window state.
    pub floating_window: FloatingWindowState,
    /// Whether the floating properties panel is rendered on the canvas (defaults to false).
    pub show_floating_properties: bool,
    /// Whether grid snapping is enabled for tools.
    pub grid_snap: bool,
    /// Whether the visual grid is displayed.
    pub grid_visible: bool,
    /// Whether initial auto-fit has completed.
    pub initialized: bool,
}

impl Default for LevelCanvasState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            floating_window: FloatingWindowState::default(),
            show_floating_properties: false,
            grid_snap: true,
            grid_visible: true,
            initialized: false,
        }
    }
}

impl LevelCanvasState {
    /// Fits the viewport to frame the given level within `canvas_rect`.
    pub fn fit_to_level(&mut self, level: &Level, canvas_rect: Rect) {
        if canvas_rect.width() <= 0.0 || canvas_rect.height() <= 0.0 {
            return;
        }

        let level_w = (level.size_px.x as f32).max(1.0);
        let level_h = (level.size_px.y as f32).max(1.0);

        let pad_w = (canvas_rect.width() - 64.0).max(32.0);
        let pad_h = (canvas_rect.height() - 64.0).max(32.0);

        let scale_x = pad_w / level_w;
        let scale_y = pad_h / level_h;
        let target_zoom = scale_x.min(scale_y).clamp(self.viewport.min_zoom, 2.0);

        self.viewport.set_zoom(target_zoom);

        // Center level bounding box inside canvas rect
        let level_center_world = glam::vec2(
            level.world_pos.x as f32 + level_w * 0.5,
            level.world_pos.y as f32 + level_h * 0.5,
        );
        let center_offset = (canvas_rect.center() - canvas_rect.min)
            - vec2(level_center_world.x, level_center_world.y) * target_zoom;
        self.viewport.pan = center_offset;
        self.viewport.screen_rect = canvas_rect;
        self.initialized = true;
    }
}

/// Renders the level canvas host.
pub fn render_level_canvas(
    ui: &mut Ui,
    level: &Level,
    state: &mut LevelCanvasState,
    layer_handlers: &[Box<dyn LayerHandler>],
    inspector_sections: &mut [Box<dyn InspectorSection>],
    active_tool: &mut Option<Box<dyn CanvasTool>>,
    cx: &mut EditorCtx,
) {
    let t = Tokens::get(ui.ctx());
    let available_rect = ui.available_rect_before_wrap();
    state.viewport.screen_rect = available_rect;

    if !state.initialized {
        state.fit_to_level(level, available_rect);
    }

    // Allocate interactive canvas surface
    let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
    let painter = ui.painter().with_clip_rect(available_rect);

    // Fill background
    painter.rect_filled(available_rect, 0.0, t.canvas);

    // 1. Pan & Zoom Input Handling
    let is_space = ui.input(|i| i.key_down(Key::Space));
    let middle_down = response.dragged_by(PointerButton::Middle);
    let primary_drag = response.dragged_by(PointerButton::Primary);
    let is_panning = middle_down || (is_space && primary_drag);

    if is_panning {
        let delta = response.drag_delta();
        state.viewport.pan_by(delta);
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if is_space && response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    // Zooming via scroll wheel focused at cursor position
    if response.hovered() {
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_y != 0.0 {
            let factor = if scroll_y > 0.0 { 1.15 } else { 1.0 / 1.15 };
            if let Some(mouse_pos) = response.hover_pos() {
                state.viewport.zoom_at(factor, mouse_pos);
            }
        }
    }

    // Keyboard shortcuts for zoom
    if response.hovered() || response.has_focus() {
        if ui.input(|i| i.key_pressed(Key::Equals)) {
            state.viewport.zoom_at(1.2, available_rect.center());
        }
        if ui.input(|i| i.key_pressed(Key::Minus)) {
            state.viewport.zoom_at(1.0 / 1.2, available_rect.center());
        }
        if ui.input(|i| {
            i.key_pressed(Key::Num0) || (i.modifiers.command && i.key_pressed(Key::Num0))
        }) {
            state.viewport.set_zoom(1.0);
        }
    }

    // 2. Draw Level Bounds & Checkerboard
    let level_min_world = level.world_pos;
    let level_max_world =
        level.world_pos + IVec2::new(level.size_px.x as i32, level.size_px.y as i32);
    let level_screen_min = state.viewport.world_to_screen(level_min_world);
    let level_screen_max = state.viewport.world_to_screen(level_max_world);
    let level_screen_rect = Rect::from_two_pos(level_screen_min, level_screen_max);

    // Checkerboard backdrop inside level boundary
    let clipped_level_rect = level_screen_rect.intersect(available_rect);
    if clipped_level_rect.is_positive() {
        let check_size = (16.0 * state.viewport.zoom).clamp(4.0, 64.0);
        draw_checkerboard(&painter, clipped_level_rect, check_size, &t);
    }

    // Level border hairline
    painter.rect_stroke(level_screen_rect, 0.0, t.hairline(), StrokeKind::Outside);

    // 3. Grid Drawing
    if state.grid_visible {
        let grid_size = 16;
        draw_grid(&painter, available_rect, &state.viewport, grid_size, &t);
    }

    // 4. Layer Render Loop with Viewport Culling
    let (vis_min, vis_max) = state.viewport.visible_world_bounds();
    let level_offscreen = level_max_world.x < vis_min.x
        || level_min_world.x > vis_max.x
        || level_max_world.y < vis_min.y
        || level_min_world.y > vis_max.y;

    if !level_offscreen {
        for layer in &level.layers {
            if !layer.visible {
                continue;
            }

            let kind = layer.kind();
            if let Some(handler) = layer_handlers.iter().find(|h| h.kind() == kind) {
                let is_active = cx.active_layer() == Some(layer.def_id);
                let mut layer_ctx = LayerRenderCtx {
                    painter: &painter,
                    viewport: &state.viewport,
                    layer,
                    level,
                    project: cx.project(),
                    is_active,
                    opacity: 1.0,
                };
                handler.render(&mut layer_ctx);
            }
        }
    }

    // 5. Tool Input Dispatch
    if !is_panning && response.hovered() {
        if let Some(tool) = active_tool {
            let pointer_pos = response.hover_pos();
            let world_pos = pointer_pos.map(|p| state.viewport.screen_to_world(p));
            let grid_pos = pointer_pos.map(|p| {
                state
                    .viewport
                    .screen_to_grid(p, level.world_pos, IVec2::splat(16))
            });

            let mut cv = CanvasCtx {
                viewport: &state.viewport,
                pointer_pos,
                world_pos,
                grid_pos,
                modifiers: ui.input(|i| i.modifiers),
                response: &response,
                active_layer_id: cx.active_layer(),
                active_level_id: level.id,
            };

            tool.on_input(&mut cv, cx);
        }
    }

    // 6. Mini Overlay Canvas Toolbar (top-left)
    render_canvas_toolbar(ui, available_rect, level, state);

    // 7. Floating Properties Window (if enabled)
    if state.show_floating_properties {
        render_floating_inspector(ui, available_rect, level, state, inspector_sections, cx);
    }
}

/// Renders the mini overlay toolbar in the top-left corner of the canvas.
fn render_canvas_toolbar(
    ui: &mut Ui,
    canvas_rect: Rect,
    level: &Level,
    state: &mut LevelCanvasState,
) {
    let t = Tokens::get(ui.ctx());
    let toolbar_pos = canvas_rect.min + vec2(12.0, 12.0);

    egui::Area::new(ui.id().with("canvas_toolbar"))
        .order(egui::Order::Middle)
        .fixed_pos(toolbar_pos)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Zoom in
                        if icon_button(ui, Icon::ZoomIn, "Zoom In (+ key)").clicked() {
                            state.viewport.zoom_at(1.25, canvas_rect.center());
                        }

                        // Zoom out
                        if icon_button(ui, Icon::ZoomOut, "Zoom Out (- key)").clicked() {
                            state.viewport.zoom_at(1.0 / 1.25, canvas_rect.center());
                        }

                        // Fit to level
                        if icon_button(ui, Icon::Fit, "Fit level to canvas").clicked() {
                            state.fit_to_level(level, canvas_rect);
                        }

                        // Zoom percentage label
                        let zoom_pct = format!("{:.0}%", state.viewport.zoom * 100.0);
                        hud(ui, &zoom_pct, t.ink_strong);

                        ui.add_space(4.0);

                        // Grid toggle
                        let grid_tip = if state.grid_visible {
                            "Hide visual grid"
                        } else {
                            "Show visual grid"
                        };
                        if icon_button(ui, Icon::Grid, grid_tip).clicked() {
                            state.grid_visible = !state.grid_visible;
                        }

                        // Grid snap toggle
                        let snap_tip = if state.grid_snap {
                            "Disable grid snap"
                        } else {
                            "Enable grid snap"
                        };
                        if icon_button(ui, Icon::Snap, snap_tip).clicked() {
                            state.grid_snap = !state.grid_snap;
                        }
                    });
                });
        });
}

/// Renders the floating properties window hosting applicable inspector sections.
fn render_floating_inspector(
    ui: &mut Ui,
    canvas_rect: Rect,
    level: &Level,
    state: &mut LevelCanvasState,
    inspector_sections: &mut [Box<dyn InspectorSection>],
    cx: &mut EditorCtx,
) {
    let t = Tokens::get(ui.ctx());
    floating_window(
        ui,
        level.id,
        "PROPERTIES",
        canvas_rect,
        &mut state.floating_window,
        |ui| {
            let sel = cx.selection().clone();
            let mut any_rendered = false;

            for section in inspector_sections.iter_mut() {
                if section.applies(&sel) {
                    section.ui(ui, cx, &sel);
                    any_rendered = true;
                }
            }

            if !any_rendered {
                // Fallback: Default level summary properties
                ui.horizontal(|ui| {
                    hud(ui, "LEVEL", t.faint);
                    ui.label(
                        egui::RichText::new(&level.name)
                            .strong()
                            .color(t.ink_strong),
                    );
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    hud(ui, "POSITION", t.faint);
                    ui.label(format!("{}, {}", level.world_pos.x, level.world_pos.y));
                });
                ui.horizontal(|ui| {
                    hud(ui, "SIZE", t.faint);
                    ui.label(format!("{} x {} px", level.size_px.x, level.size_px.y));
                });
                ui.horizontal(|ui| {
                    hud(ui, "DEPTH", t.faint);
                    ui.label(format!("{}", level.world_depth));
                });
                ui.horizontal(|ui| {
                    hud(ui, "LAYERS", t.faint);
                    ui.label(format!("{}", level.layers.len()));
                });
            }
        },
    );
}
