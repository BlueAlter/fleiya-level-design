//! Universal tileset tile selection dialog and tile icon rendering.

use egui::{
    vec2, Align2, Area, Color32, ComboBox, CornerRadius, Frame, Id, Key, Margin, Order, Pos2, Rect,
    RichText, Sense, Shape, Stroke, StrokeKind, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::id::TilesetDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::AssetService;

/// State for the universal tileset selection pop-up dialog.
#[derive(Debug, Clone)]
pub struct TilesetDialogState {
    pub is_open: bool,
    pub target_val_id: Option<String>,
    pub current_bound_tile: Option<TileRef>,
    pub selected_tileset_id: Option<TilesetDefId>,
    pub zoom: f32,
    pub pan: Pos2,
    pub hovered_tile: Option<u32>,
    pub show_grid: bool,
    pub needs_fit: bool,
    pub show_debug_overlay: bool,
    pub last_frame_time: std::time::Instant,
    pub frame_time_ms: f32,
    pub fps: f32,
}

impl Default for TilesetDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            target_val_id: None,
            current_bound_tile: None,
            selected_tileset_id: None,
            zoom: 2.0,
            pan: Pos2::new(20.0, 20.0),
            hovered_tile: None,
            show_grid: true,
            needs_fit: true,
            show_debug_overlay: false,
            last_frame_time: std::time::Instant::now(),
            frame_time_ms: 16.6,
            fps: 60.0,
        }
    }
}

impl TilesetDialogState {
    /// Creates a new inactive tileset dialog state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the dialog to select a tile for a specific value ID.
    pub fn open(
        &mut self,
        target_val_id: String,
        tileset_id: Option<TilesetDefId>,
        bound_tile: Option<TileRef>,
    ) {
        self.is_open = true;
        self.target_val_id = Some(target_val_id);
        if tileset_id.is_some() {
            self.selected_tileset_id = tileset_id;
        }
        self.current_bound_tile = bound_tile;
        self.needs_fit = true;
        self.hovered_tile = None;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
        self.target_val_id = None;
        self.current_bound_tile = None;
        self.hovered_tile = None;
    }
}

/// Renders a tile graphic icon inside a UI rect.
///
/// Returns the `egui::Response` to allow handling clicks, right-clicks, and hover states.
pub fn render_tile_icon(
    ui: &mut Ui,
    project: &Project,
    assets: &dyn AssetService,
    tile_ref: &TileRef,
    size: f32,
) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::same(3), t.panel);
    draw_checkerboard(&painter, rect, 4.0, &t);

    let mut drew_image = false;
    if let Some(tileset) = project.defs.tileset(tile_ref.tileset_id) {
        if let Ok(texture) = assets.load_texture(ui.ctx(), &tileset.path) {
            let img_size = texture.size_vec2();
            let tile_w = tileset.tile_size.x.max(1);
            let tile_h = tileset.tile_size.y.max(1);
            let padding = tileset.padding;
            let spacing = tileset.spacing;

            let img_w = img_size.x as u32;
            let img_h = img_size.y as u32;
            let eff_w = img_w.saturating_sub(padding * 2);
            let stride_x = tile_w + spacing;
            let stride_y = tile_h + spacing;
            let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);

            let col = tile_ref.tile_id % cols;
            let row = tile_ref.tile_id / cols;

            let px = padding + col * stride_x;
            let py = padding + row * stride_y;

            let mut u0 = px as f32 / img_w as f32;
            let mut v0 = py as f32 / img_h as f32;
            let mut u1 = (px + tile_w) as f32 / img_w as f32;
            let mut v1 = (py + tile_h) as f32 / img_h as f32;

            if tile_ref.flip_x {
                std::mem::swap(&mut u0, &mut u1);
            }
            if tile_ref.flip_y {
                std::mem::swap(&mut v0, &mut v1);
            }

            painter.image(
                texture.id(),
                rect.shrink(1.0),
                Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                Color32::WHITE,
            );
            drew_image = true;
        }
    }

    if !drew_image {
        // Fallback visual tile icon when no texture is loaded
        let color = Color32::from_rgb(
            ((tile_ref.tile_id * 53 + 90) % 200 + 40) as u8,
            ((tile_ref.tile_id * 97 + 120) % 200 + 40) as u8,
            ((tile_ref.tile_id * 163 + 150) % 200 + 40) as u8,
        );
        painter.rect_filled(rect.shrink(2.0), CornerRadius::same(2), color);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("{}", tile_ref.tile_id),
            egui::FontId::monospace(size * 0.4),
            Color32::WHITE,
        );
    }

    let border_stroke = if resp.hovered() {
        Stroke::new(1.0, t.accent)
    } else {
        Stroke::new(1.0, t.line_strong)
    };
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        border_stroke,
        StrokeKind::Inside,
    );

    resp.on_hover_text(format!(
        "Tile #{} (Tileset {:?})\nClick to change, right-click to delete",
        tile_ref.tile_id, tile_ref.tileset_id.0
    ))
}

/// Renders the universal tileset selection pop-up dialog.
///
/// Returns `Some((target_val_id, selected_tile))` if a tile was picked or cleared.
pub fn show_tileset_dialog(
    ctx: &egui::Context,
    state: &mut TilesetDialogState,
    project: &Project,
    assets: &dyn AssetService,
) -> Option<(String, Option<TileRef>)> {
    if !state.is_open {
        return None;
    }

    // Measure frame time for debug overlay
    let now = std::time::Instant::now();
    let dt = now.duration_since(state.last_frame_time).as_secs_f32();
    state.last_frame_time = now;
    if dt > 0.0 && dt < 1.0 {
        state.fps = 0.9 * state.fps + 0.1 * (1.0 / dt);
        state.frame_time_ms = 0.9 * state.frame_time_ms + 0.1 * (dt * 1000.0);
    }

    // ESC closes the pop-up immediately
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.close();
        return None;
    }

    // F key triggers auto fit and center
    if ctx.input(|i| i.key_pressed(Key::F)) {
        state.needs_fit = true;
    }

    let t = Tokens::get(ctx);
    let mut chosen_action: Option<(String, Option<TileRef>)> = None;

    // Resolve active tileset
    if state.selected_tileset_id.is_none()
        || !project
            .defs
            .tilesets
            .iter()
            .any(|ts| Some(ts.id) == state.selected_tileset_id)
    {
        state.selected_tileset_id = project.defs.tilesets.first().map(|ts| ts.id);
        state.needs_fit = true;
    }

    let target_val_id = state.target_val_id.clone().unwrap_or_default();

    // Dimmed background shield to capture outside clicks and close pop-up
    Area::new(Id::new("tileset_dialog_backdrop"))
        .order(Order::Foreground)
        .fixed_pos(Pos2::ZERO)
        .show(ctx, |ui| {
            let screen_rect = ctx.viewport_rect();
            let (bg_rect, bg_resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
            ui.painter()
                .rect_filled(bg_rect, CornerRadius::ZERO, Color32::from_black_alpha(140));
            if bg_resp.clicked() {
                state.close();
            }
        });

    if !state.is_open {
        return None;
    }

    // Center pop-up modal
    Area::new(Id::new("tileset_dialog_window"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.line_strong))
                .corner_radius(CornerRadius::ZERO)
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(540.0);
                    ui.set_height(460.0);

                    // Top toolbar: Tileset dropdown, Grid toggle, Fit button, Clear button, Close
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tileset:").strong().color(t.ink_strong));
                        let prev_ts = state.selected_tileset_id;
                        ComboBox::from_id_salt("dialog_tileset_combo")
                            .selected_text(
                                state
                                    .selected_tileset_id
                                    .and_then(|id| project.defs.tileset(id))
                                    .map(|ts| ts.name.as_str())
                                    .unwrap_or("Select Tileset"),
                            )
                            .show_ui(ui, |ui| {
                                for ts in &project.defs.tilesets {
                                    ui.selectable_value(
                                        &mut state.selected_tileset_id,
                                        Some(ts.id),
                                        &ts.name,
                                    );
                                }
                            });

                        if prev_ts != state.selected_tileset_id {
                            state.needs_fit = true;
                        }

                        ui.add_space(8.0);

                        // Grid icon toggle
                        let grid_active = state.show_grid;
                        let grid_btn = ui
                            .add(
                                egui::Button::new(Icon::Grid.rich(
                                    ui.ctx(),
                                    14.0,
                                    if grid_active { t.accent_ink } else { t.muted },
                                ))
                                .fill(if grid_active {
                                    t.sunken
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(if grid_active {
                                    Stroke::new(1.0, t.accent)
                                } else {
                                    Stroke::NONE
                                })
                                .corner_radius(CornerRadius::same(3)),
                            )
                            .on_hover_text("Toggle grid");

                        if grid_btn.clicked() {
                            state.show_grid = !state.show_grid;
                        }

                        ui.add_space(4.0);

                        // Auto-Fit button
                        let fit_btn = ui
                            .add(
                                egui::Button::new(Icon::Fit.rich(ui.ctx(), 14.0, t.ink))
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .corner_radius(CornerRadius::same(3)),
                            )
                            .on_hover_text("Fit and Center (F)");
                        if fit_btn.clicked() {
                            state.needs_fit = true;
                        }

                        ui.add_space(4.0);

                        // "Clear" button to empty the bound tile
                        let clear_btn = ui
                            .add(
                                egui::Button::new(RichText::new("Clear").size(11.5).color(t.ink))
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(t.hairline())
                                    .corner_radius(CornerRadius::same(3)),
                            )
                            .on_hover_text("Clear bound tile");
                        if clear_btn.clicked() {
                            chosen_action = Some((target_val_id.clone(), None));
                        }

                        ui.add_space(4.0);

                        // Debug overlay toggle
                        let dbg_active = state.show_debug_overlay;
                        let dbg_btn = ui
                            .add(
                                egui::Button::new(
                                    RichText::new("FPS").size(10.5).color(if dbg_active {
                                        t.accent_ink
                                    } else {
                                        t.faint
                                    }),
                                )
                                .fill(if dbg_active {
                                    t.sunken
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(if dbg_active {
                                    Stroke::new(1.0, t.accent)
                                } else {
                                    Stroke::NONE
                                }),
                            )
                            .on_hover_text("Toggle FPS overlay");
                        if dbg_btn.clicked() {
                            state.show_debug_overlay = !state.show_debug_overlay;
                        }

                        // Close button on right with X icon and tooltip
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = ui
                                .add(
                                    egui::Button::new(Icon::Close.rich(ui.ctx(), 14.0, t.ink))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE)
                                        .corner_radius(CornerRadius::same(3)),
                                )
                                .on_hover_text("Close (ESC)");

                            if close_btn.clicked() {
                                state.close();
                            }

                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!("{:.0}%", state.zoom * 100.0))
                                    .size(11.0)
                                    .color(t.muted),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Interactive Tileset Canvas
                    let tileset = state
                        .selected_tileset_id
                        .and_then(|id| project.defs.tileset(id));
                    if let Some(ts) = tileset {
                        let tile_w = ts.tile_size.x.max(1);
                        let tile_h = ts.tile_size.y.max(1);
                        let padding = ts.padding;
                        let spacing = ts.spacing;

                        let (canvas_rect, response) =
                            ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                        let painter = ui.painter().with_clip_rect(canvas_rect);
                        painter.rect_filled(canvas_rect, CornerRadius::ZERO, t.canvas);

                        // Pan: middle button, or primary drag on non-tile area
                        if response.dragged_by(egui::PointerButton::Middle)
                            || (response.dragged_by(egui::PointerButton::Primary)
                                && state.hovered_tile.is_none())
                        {
                            state.pan += response.drag_delta();
                        }

                        // Zoom: scroll wheel centered on cursor position
                        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                        if scroll != 0.0 && response.hovered() {
                            if let Some(cursor) = response.hover_pos() {
                                let zoom_factor = if scroll > 0.0 { 1.15 } else { 0.87 };
                                let new_zoom = (state.zoom * zoom_factor).clamp(0.5, 8.0);
                                state.pan = cursor - (cursor - state.pan) * (new_zoom / state.zoom);
                                state.zoom = new_zoom;
                            }
                        }

                        // Check image texture
                        let texture_res = assets.load_texture(ui.ctx(), &ts.path);
                        let (img_w, img_h) = if let Ok(ref tex) = texture_res {
                            let sz = tex.size_vec2();
                            (sz.x as u32, sz.y as u32)
                        } else {
                            (256, 256)
                        };

                        let eff_w = img_w.saturating_sub(padding * 2);
                        let eff_h = img_h.saturating_sub(padding * 2);
                        let stride_x = tile_w + spacing;
                        let stride_y = tile_h + spacing;
                        let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                        let rows = (eff_h + spacing).checked_div(stride_y).unwrap_or(1).max(1);

                        // Handle Auto-Fit
                        if state.needs_fit {
                            let view_w = canvas_rect.width();
                            let view_h = canvas_rect.height();
                            let margin = 20.0f32;

                            if let Some(ref bound) = state.current_bound_tile {
                                if bound.tileset_id == ts.id {
                                    state.zoom = 2.5;
                                    let col = bound.tile_id % cols;
                                    let row = bound.tile_id / cols;
                                    let center_x = padding as f32
                                        + (col * stride_x) as f32
                                        + (tile_w as f32 / 2.0);
                                    let center_y = padding as f32
                                        + (row * stride_y) as f32
                                        + (tile_h as f32 / 2.0);
                                    state.pan = canvas_rect.center()
                                        - vec2(center_x * state.zoom, center_y * state.zoom);
                                    state.needs_fit = false;
                                } else {
                                    let zoom_w = (view_w - 2.0 * margin) / img_w as f32;
                                    let zoom_h = (view_h - 2.0 * margin) / img_h as f32;
                                    state.zoom = zoom_w.min(zoom_h).clamp(0.5, 8.0);
                                    let drawn_w = img_w as f32 * state.zoom;
                                    let drawn_h = img_h as f32 * state.zoom;
                                    state.pan = Pos2::new(
                                        canvas_rect.min.x + (view_w - drawn_w) / 2.0,
                                        canvas_rect.min.y + (view_h - drawn_h) / 2.0,
                                    );
                                    state.needs_fit = false;
                                }
                            } else {
                                let zoom_w = (view_w - 2.0 * margin) / img_w as f32;
                                let zoom_h = (view_h - 2.0 * margin) / img_h as f32;
                                state.zoom = zoom_w.min(zoom_h).clamp(0.5, 8.0);
                                let drawn_w = img_w as f32 * state.zoom;
                                let drawn_h = img_h as f32 * state.zoom;
                                state.pan = Pos2::new(
                                    canvas_rect.min.x + (view_w - drawn_w) / 2.0,
                                    canvas_rect.min.y + (view_h - drawn_h) / 2.0,
                                );
                                state.needs_fit = false;
                            }
                        }

                        // Grid lines batched via painter.extend(Vec<Shape>), hidden if on-screen cell < 6px
                        if state.show_grid {
                            let step_x = tile_w as f32 * state.zoom;
                            let step_y = tile_h as f32 * state.zoom;
                            if step_x >= 6.0 && step_y >= 6.0 {
                                let mut shapes = Vec::new();
                                let grid_stroke = Stroke::new(1.0, t.grid_minor);
                                let first_k_x =
                                    ((canvas_rect.min.x - state.pan.x) / step_x).floor() as i32;
                                let last_k_x =
                                    ((canvas_rect.max.x - state.pan.x) / step_x).ceil() as i32;
                                for k in first_k_x..=last_k_x {
                                    let x = state.pan.x + k as f32 * step_x;
                                    if x >= canvas_rect.min.x && x <= canvas_rect.max.x {
                                        shapes.push(Shape::line_segment(
                                            [
                                                Pos2::new(x, canvas_rect.top()),
                                                Pos2::new(x, canvas_rect.bottom()),
                                            ],
                                            grid_stroke,
                                        ));
                                    }
                                }

                                let first_k_y =
                                    ((canvas_rect.min.y - state.pan.y) / step_y).floor() as i32;
                                let last_k_y =
                                    ((canvas_rect.max.y - state.pan.y) / step_y).ceil() as i32;
                                for k in first_k_y..=last_k_y {
                                    let y = state.pan.y + k as f32 * step_y;
                                    if y >= canvas_rect.min.y && y <= canvas_rect.max.y {
                                        shapes.push(Shape::line_segment(
                                            [
                                                Pos2::new(canvas_rect.left(), y),
                                                Pos2::new(canvas_rect.right(), y),
                                            ],
                                            grid_stroke,
                                        ));
                                    }
                                }
                                painter.extend(shapes);
                            }
                        }

                        // Draw tileset image
                        if let Ok(texture) = texture_res {
                            let img_size = texture.size_vec2();
                            let tex_rect = Rect::from_min_size(state.pan, img_size * state.zoom);

                            // Checkerboard under tileset image
                            draw_checkerboard(&painter, tex_rect, 16.0, &t);

                            // Texture image
                            painter.image(
                                texture.id(),
                                tex_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            painter.rect_stroke(
                                tex_rect,
                                CornerRadius::ZERO,
                                Stroke::new(1.0, t.line_strong),
                                StrokeKind::Inside,
                            );

                            // Hover detection
                            let mut hovered_c_r = None;
                            if let Some(mpos) = response.hover_pos() {
                                let rel_x = (mpos.x - state.pan.x) / state.zoom;
                                let rel_y = (mpos.y - state.pan.y) / state.zoom;
                                if rel_x >= padding as f32 && rel_y >= padding as f32 {
                                    let c =
                                        ((rel_x - padding as f32) / stride_x as f32).floor() as u32;
                                    let r =
                                        ((rel_y - padding as f32) / stride_y as f32).floor() as u32;
                                    if c < cols && r < rows {
                                        hovered_c_r = Some((c, r));
                                        let tid = r * cols + c;
                                        state.hovered_tile = Some(tid);
                                    } else {
                                        state.hovered_tile = None;
                                    }
                                } else {
                                    state.hovered_tile = None;
                                }
                            } else {
                                state.hovered_tile = None;
                            }

                            // Highlight existing bound tile if present
                            if let Some(ref bound) = state.current_bound_tile {
                                if bound.tileset_id == ts.id {
                                    let c = bound.tile_id % cols;
                                    let r = bound.tile_id / cols;
                                    let bx = state.pan.x
                                        + (padding as f32 + (c * stride_x) as f32) * state.zoom;
                                    let by = state.pan.y
                                        + (padding as f32 + (r * stride_y) as f32) * state.zoom;
                                    let bound_rect = Rect::from_min_size(
                                        Pos2::new(bx, by),
                                        vec2(
                                            tile_w as f32 * state.zoom,
                                            tile_h as f32 * state.zoom,
                                        ),
                                    );
                                    painter.rect_stroke(
                                        bound_rect,
                                        CornerRadius::ZERO,
                                        Stroke::new(2.5, Color32::from_rgb(255, 193, 77)),
                                        StrokeKind::Inside,
                                    );
                                }
                            }

                            // Highlight hovered tile
                            if let Some((c, r)) = hovered_c_r {
                                let cx = state.pan.x
                                    + (padding as f32 + (c * stride_x) as f32) * state.zoom;
                                let cy = state.pan.y
                                    + (padding as f32 + (r * stride_y) as f32) * state.zoom;
                                let cell_rect = Rect::from_min_size(
                                    Pos2::new(cx, cy),
                                    vec2(tile_w as f32 * state.zoom, tile_h as f32 * state.zoom),
                                );
                                painter.rect_stroke(
                                    cell_rect,
                                    CornerRadius::ZERO,
                                    Stroke::new(2.0, t.accent),
                                    StrokeKind::Inside,
                                );

                                if response.clicked() {
                                    let tile_id = r * cols + c;
                                    chosen_action = Some((
                                        target_val_id.clone(),
                                        Some(TileRef::new(ts.id, tile_id)),
                                    ));
                                }
                            }
                        } else {
                            // Fallback procedural tiles
                            let cols = 8u32;
                            let rows = 8u32;
                            let cell_size = 40.0 * (state.zoom / 2.0);

                            let mouse_pos = response.hover_pos();
                            for r in 0..rows {
                                for c in 0..cols {
                                    let cell_rect = Rect::from_min_size(
                                        state.pan
                                            + vec2(c as f32 * cell_size, r as f32 * cell_size),
                                        vec2(cell_size, cell_size),
                                    );
                                    let tile_id = r * cols + c;
                                    let is_hovered =
                                        mouse_pos.is_some_and(|p| cell_rect.contains(p));

                                    let col = Color32::from_rgb(
                                        ((tile_id * 53 + 90) % 200 + 40) as u8,
                                        ((tile_id * 97 + 120) % 200 + 40) as u8,
                                        ((tile_id * 163 + 150) % 200 + 40) as u8,
                                    );
                                    painter.rect_filled(
                                        cell_rect.shrink(1.0),
                                        CornerRadius::ZERO,
                                        col,
                                    );
                                    painter.text(
                                        cell_rect.center(),
                                        Align2::CENTER_CENTER,
                                        format!("{tile_id}"),
                                        egui::FontId::monospace(11.0),
                                        Color32::WHITE,
                                    );

                                    let border = if is_hovered {
                                        Stroke::new(2.0, t.accent)
                                    } else {
                                        t.hairline()
                                    };
                                    painter.rect_stroke(
                                        cell_rect,
                                        CornerRadius::ZERO,
                                        border,
                                        StrokeKind::Inside,
                                    );

                                    if is_hovered && response.clicked() {
                                        chosen_action = Some((
                                            target_val_id.clone(),
                                            Some(TileRef::new(ts.id, tile_id)),
                                        ));
                                    }
                                }
                            }
                        }

                        // Debug overlay chip
                        if state.show_debug_overlay {
                            let dbg_rect = Rect::from_min_size(
                                canvas_rect.min + vec2(8.0, 8.0),
                                vec2(140.0, 24.0),
                            );
                            painter.rect_filled(
                                dbg_rect,
                                CornerRadius::same(3),
                                Color32::from_black_alpha(180),
                            );
                            painter.text(
                                dbg_rect.center(),
                                Align2::CENTER_CENTER,
                                format!("{:.1} FPS ({:.2} ms)", state.fps, state.frame_time_ms),
                                egui::FontId::monospace(11.0),
                                Color32::WHITE,
                            );
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(
                                RichText::new("No tileset available in project.").color(t.muted),
                            );
                        });
                    }
                });
        });

    if chosen_action.is_some() {
        state.close();
    }

    chosen_action
}
