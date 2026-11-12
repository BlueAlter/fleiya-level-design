//! Universal Tile Picker modal component for tile selection in enums and layers.
//!
//! Provides an LDtk-style page-based workspace where the tileset sits centered
//! within a scrollable, pannable, and zoomable workspace. Supports single tile selection
//! via left click and rectangular multi-tile selection via right-click dragging.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{
    vec2, Color32, ColorImage, CornerRadius, CursorIcon, Frame, Id, Margin, Modifiers, Order,
    PointerButton, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;

use super::tile_bounds::{TileGrid, TileRect};

/// Neutral, seamlessly repeating background drawn behind the tileset page.
///
/// Embedded so the page is never transparent, even when the tileset image has
/// alpha pixels or the process working directory is not the repository root.
const PAGE_BG_BYTES: &[u8] =
    include_bytes!("../../../../../assets/Bg/tileset_picker_bg_netral.png");

/// Page-space edge length of one repetition of the page background pattern.
///
/// Expressed in tileset (page) pixels, so on screen one cell measures
/// `PAGE_BG_CELL * zoom`: the pattern keeps its size relative to the artwork
/// and never shrinks when the user zooms in.
pub const PAGE_BG_CELL: f32 = 16.0;

/// UV rectangle that repeats the page background across a page-space `rect`.
///
/// The texture is uploaded with [`TextureOptions::NEAREST_REPEAT`], so texture
/// coordinates beyond `1.0` simply start the pattern over again.
pub fn page_bg_uv(rect: Rect) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        vec2(
            (rect.width() / PAGE_BG_CELL).max(0.0),
            (rect.height() / PAGE_BG_CELL).max(0.0),
        ),
    )
}

/// Event emitted by the [`TilePicker`] component after user interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePickerEvent {
    /// No event occurred this frame.
    None,
    /// A tile or multi-tile rectangular region was selected.
    Selected(TileRect),
    /// The popup was closed via Escape, Close button, or clicking the backdrop.
    Closed,
}

/// Normalized tileset metadata consumed by the [`TilePicker`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilesetData {
    /// Unique tileset definition ID.
    pub id: TilesetDefId,
    /// Human-readable display name.
    pub name: String,
    /// Relative path to the image file from project root.
    pub path: String,
    /// Size of an individual tile in pixels.
    pub tile_size: glam::UVec2,
    /// Spacing between adjacent tiles in pixels.
    pub spacing: u32,
    /// Padding around the tileset border in pixels.
    pub padding: u32,
    /// Whether this is a placeholder or empty tileset representation.
    pub is_empty: bool,
}

impl TilesetData {
    /// Creates an empty tileset data placeholder.
    pub fn empty() -> Self {
        Self {
            id: TilesetDefId(0),
            name: String::new(),
            path: String::new(),
            tile_size: glam::UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            is_empty: true,
        }
    }

    /// Returns `true` if this represents a missing or unbound tileset.
    pub fn is_empty(&self) -> bool {
        self.is_empty || self.path.trim().is_empty()
    }
}

impl From<&TilesetDef> for TilesetData {
    fn from(def: &TilesetDef) -> Self {
        Self {
            id: def.id,
            name: def.name.clone(),
            path: def.path.clone(),
            tile_size: def.tile_size,
            spacing: def.spacing,
            padding: def.padding,
            is_empty: false,
        }
    }
}

/// Cached texture information including actual decoded dimensions and scale factor.
#[derive(Clone)]
pub struct CachedTextureInfo {
    pub handle: TextureHandle,
    pub real_size: (u32, u32),
    pub texture_size: (u32, u32),
    pub scale: f32,
}

/// View transformation between unscaled page coordinates and screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickerView {
    /// Zoom scale factor from page pixels to screen pixels.
    pub zoom: f32,
    /// Screen pixel position where page (0, 0) is drawn.
    pub offset: egui::Vec2,
}

impl Default for PickerView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
        }
    }
}

impl PickerView {
    /// Creates a new `PickerView`.
    pub fn new(zoom: f32, offset: egui::Vec2) -> Self {
        Self { zoom, offset }
    }

    /// Converts a point in page coordinates to screen pixels (snapped to whole pixels).
    pub fn page_to_screen(&self, page_pos: Pos2) -> Pos2 {
        Pos2::new(
            (page_pos.x * self.zoom + self.offset.x).round(),
            (page_pos.y * self.zoom + self.offset.y).round(),
        )
    }

    /// Converts a screen pixel position to page coordinates.
    pub fn screen_to_page(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.offset.x) / self.zoom,
            (screen_pos.y - self.offset.y) / self.zoom,
        )
    }

    /// Converts a page rectangle to a screen pixel rectangle.
    pub fn page_to_screen_rect(&self, page_rect: Rect) -> Rect {
        Rect::from_min_max(
            self.page_to_screen(page_rect.min),
            self.page_to_screen(page_rect.max),
        )
    }
}

/// Target identifying which enum and value opened the tile picker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerTarget {
    pub enum_id: lc_core::id::EnumDefId,
    pub value_id: String,
}

/// Dedicated Tile Picker modal popup state.
#[derive(Clone)]
pub struct TilePicker {
    /// Whether the modal popup is currently visible.
    pub is_open: bool,
    /// ID of the tileset currently loaded in the picker.
    pub tileset_id: Option<TilesetDefId>,
    /// Currently committed selection, if any.
    pub selected_tile_rect: Option<TileRect>,
    /// Tile ID currently hovered by the pointer, if any.
    pub hovered_tile: Option<u32>,
    /// View transformation (zoom level and screen offset).
    pub view: PickerView,
    /// Whether tile grid lines are drawn over the texture.
    pub show_grid: bool,
    /// Target enum and value that opened this picker, if applicable.
    pub target: Option<PickerTarget>,
    /// Target enum value ID that opened this picker, if applicable.
    pub target_val_id: Option<String>,
    /// Flag requesting initial fit zoom and centering calculation.
    pub needs_initial_fit: bool,
    /// Previously displayed tileset ID for detecting switches.
    pub last_tileset_id: Option<TilesetDefId>,
    /// In-memory texture cache keyed by tileset ID.
    pub textures: HashMap<TilesetDefId, Result<CachedTextureInfo, String>>,
    /// Cached neutral tiled background texture used behind the tileset page.
    pub page_bg: Option<TextureHandle>,
    /// Multi-select drag start cell, if right mouse button is dragging.
    pub drag_start_cell: Option<(u32, u32)>,
    /// Whether user is currently dragging with the right mouse button.
    pub is_right_dragging: bool,
    /// Active live selection during right-drag.
    pub live_drag_rect: Option<TileRect>,
}

impl Default for TilePicker {
    fn default() -> Self {
        Self {
            is_open: false,
            tileset_id: None,
            selected_tile_rect: None,
            hovered_tile: None,
            view: PickerView::default(),
            show_grid: true,
            target: None,
            target_val_id: None,
            needs_initial_fit: true,
            last_tileset_id: None,
            textures: HashMap::new(),
            page_bg: None,
            drag_start_cell: None,
            is_right_dragging: false,
            live_drag_rect: None,
        }
    }
}

impl TilePicker {
    /// Creates a new inactive `TilePicker` component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the picker modal for a target enum value and bound tile rect.
    pub fn open(
        &mut self,
        target_val_id: Option<String>,
        tileset_id: Option<TilesetDefId>,
        selected_rect: Option<TileRect>,
    ) {
        let target = target_val_id.map(|vid| PickerTarget {
            enum_id: lc_core::id::EnumDefId(0),
            value_id: vid,
        });
        self.open_target(target, tileset_id, selected_rect);
    }

    /// Opens the picker modal with a structured PickerTarget containing enum and value IDs.
    pub fn open_target(
        &mut self,
        target: Option<PickerTarget>,
        tileset_id: Option<TilesetDefId>,
        selected_rect: Option<TileRect>,
    ) {
        self.is_open = true;
        self.target_val_id = target.as_ref().map(|t| t.value_id.clone());
        self.target = target;
        if self.tileset_id != tileset_id {
            self.tileset_id = tileset_id;
            self.needs_initial_fit = true;
        }
        self.selected_tile_rect = selected_rect;
        self.hovered_tile = None;
        self.drag_start_cell = None;
        self.is_right_dragging = false;
        self.live_drag_rect = None;
    }

    /// Opens the picker modal with a value ID string for compatibility.
    pub fn open_legacy(
        &mut self,
        target_val_id: Option<String>,
        tileset_id: Option<TilesetDefId>,
        selected_rect: Option<TileRect>,
    ) {
        self.open(target_val_id, tileset_id, selected_rect);
    }

    /// Opens the picker modal from a legacy single tile ID.
    pub fn open_tile_id(
        &mut self,
        target_val_id: Option<String>,
        tileset_id: Option<TilesetDefId>,
        selected_tile: Option<u32>,
        cols: u32,
    ) {
        let rect = selected_tile.map(|id| {
            let col = id.checked_rem(cols).unwrap_or(0);
            let row = id.checked_div(cols).unwrap_or(0);
            TileRect::single(col, row)
        });
        self.open(target_val_id, tileset_id, rect);
    }

    /// Closes the picker modal and resets active interaction states.
    pub fn close(&mut self) {
        self.is_open = false;
        self.target = None;
        self.target_val_id = None;
        self.hovered_tile = None;
        self.drag_start_cell = None;
        self.is_right_dragging = false;
        self.live_drag_rect = None;
    }

    /// Returns `true` if the modal is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns top-left selected tile ID, if any.
    pub fn selected_tile_id(&self, grid: &TileGrid) -> Option<u32> {
        self.selected_tile_rect.map(|r| r.top_left_id(grid))
    }

    /// Renders the tile picker modal on the screen.
    ///
    /// Handles all navigation, clicks, right-drag multi-selection, hover, and keyboard shortcuts.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        tileset: &TilesetData,
        selected: Option<TileRect>,
    ) -> TilePickerEvent {
        if !self.is_open {
            return TilePickerEvent::None;
        }

        // Initialize selection from caller if provided and picker hasn't set one yet
        if selected.is_some() && self.selected_tile_rect.is_none() {
            self.selected_tile_rect = selected;
        }

        let t = Tokens::get(ctx);
        let screen_rect = ctx.viewport_rect();

        // 1. Consume keyboard shortcuts active while the modal is open
        let toggle_grid = ctx.input_mut(|i| {
            i.consume_key(Modifiers::CTRL, egui::Key::H)
                || i.consume_key(Modifiers::COMMAND, egui::Key::H)
        });
        if toggle_grid {
            self.show_grid = !self.show_grid;
        }

        let esc_pressed = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape));
        if esc_pressed {
            if self.is_right_dragging {
                // Esc cancels active drag first without closing the modal
                self.is_right_dragging = false;
                self.drag_start_cell = None;
                self.live_drag_rect = None;
            } else {
                self.close();
                return TilePickerEvent::Closed;
            }
        }

        let enter_pressed = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Enter));
        if enter_pressed {
            return if let Some(rect) = self.selected_tile_rect {
                self.is_open = false;
                TilePickerEvent::Selected(rect)
            } else {
                self.close();
                TilePickerEvent::Closed
            };
        }

        // 2. Fixed Modal Dimensions & Position (centered ~80% of screen)
        let popup_w = (screen_rect.width() * 0.8).clamp(380.0, 1400.0).round();
        let popup_h = (screen_rect.height() * 0.8).clamp(280.0, 900.0).round();
        let popup_size = vec2(popup_w, popup_h);

        let popup_min = Pos2::new(
            screen_rect.min.x + ((screen_rect.width() - popup_w) / 2.0).round(),
            screen_rect.min.y + ((screen_rect.height() - popup_h) / 2.0).round(),
        );
        let popup_rect = Rect::from_min_size(popup_min, popup_size);

        // 3. Subtle dark backdrop behind modal (Order::Foreground)
        egui::Area::new(Id::new("tile_picker_backdrop"))
            .order(Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen_rect,
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(140),
                );
            });

        // Detect click outside popup to close (only when not actively dragging)
        let mut closed_by_backdrop = false;
        if !self.is_right_dragging
            && ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
        {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if !popup_rect.contains(pos) {
                    closed_by_backdrop = true;
                }
            }
        }

        if closed_by_backdrop {
            self.close();
            return TilePickerEvent::Closed;
        }

        let mut event = TilePickerEvent::None;

        // 4. Modal Window with enforced fixed size
        egui::Area::new(Id::new("tile_picker_window"))
            .order(Order::Foreground)
            .fixed_pos(popup_min)
            .show(ctx, |ui| {
                ui.set_min_size(popup_size);
                ui.set_max_size(popup_size);

                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(Margin::ZERO)
                    .show(ui, |ui| {
                        ui.set_min_size(popup_size);
                        ui.set_max_size(popup_size);

                        if tileset.is_empty() {
                            self.render_empty_state(ui, &t, &mut event);
                        } else {
                            self.render_canvas_state(ui, &t, tileset, popup_rect, &mut event);
                        }
                    });
            });

        if event != TilePickerEvent::None && event != TilePickerEvent::Closed {
            self.is_open = false;
        }

        event
    }

    /// Renders the empty state when the enum has no tileset bound.
    fn render_empty_state(&mut self, ui: &mut egui::Ui, t: &Tokens, event: &mut TilePickerEvent) {
        ui.vertical_centered(|ui| {
            let total_h = ui.available_height();
            ui.add_space(total_h * 0.35);

            ui.label(
                RichText::new("No tileset bound to this enum")
                    .size(15.0)
                    .strong()
                    .color(t.ink_strong),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Assign a tileset in the Enum Identifier card to pick tile graphics.",
                )
                .size(12.0)
                .color(t.muted),
            );
            ui.add_space(20.0);

            let close_btn = ui.add(
                egui::Button::new(RichText::new("Close").size(12.0).color(t.ink_strong))
                    .fill(t.sunken)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::ZERO)
                    .min_size(vec2(100.0, 28.0)),
            );

            if close_btn.clicked() {
                self.close();
                *event = TilePickerEvent::Closed;
            }
        });
    }

    /// Renders the full interactive tileset canvas area.
    fn render_canvas_state(
        &mut self,
        ui: &mut egui::Ui,
        t: &Tokens,
        tileset: &TilesetData,
        popup_rect: Rect,
        event: &mut TilePickerEvent,
    ) {
        // Detect tileset switch to trigger auto-fit
        if self.last_tileset_id != Some(tileset.id) {
            self.last_tileset_id = Some(tileset.id);
            self.needs_initial_fit = true;
        }

        let texture_result = self.get_or_load_texture(ui.ctx(), tileset);
        let tex_info = match texture_result {
            Ok(ref info) => info.clone(),
            Err(ref err_msg) => {
                ui.vertical_centered(|ui| {
                    let total_h = ui.available_height();
                    ui.add_space(total_h * 0.35);
                    ui.label(
                        RichText::new("Failed to load tileset image")
                            .size(15.0)
                            .strong()
                            .color(Color32::from_rgb(239, 68, 68)),
                    );
                    ui.add_space(6.0);
                    ui.label(RichText::new(err_msg).size(11.5).color(t.muted));
                    ui.add_space(20.0);
                    let close_btn = ui.add(
                        egui::Button::new(RichText::new("Close").size(12.0).color(t.ink_strong))
                            .fill(t.sunken)
                            .stroke(Stroke::new(1.0, t.line_strong))
                            .corner_radius(CornerRadius::ZERO)
                            .min_size(vec2(100.0, 28.0)),
                    );
                    if close_btn.clicked() {
                        self.close();
                        *event = TilePickerEvent::Closed;
                    }
                });
                return;
            }
        };

        let (real_w, real_h) = tex_info.real_size;
        let img_w = real_w;
        let img_h = real_h;

        let tile_size = if tileset.tile_size.x == 0 || tileset.tile_size.y == 0 {
            Self::detect_tile_size(&tileset.path, img_w, img_h)
        } else {
            tileset.tile_size
        };

        let tile_w = tile_size.x.max(1);
        let tile_h = tile_size.y.max(1);
        let padding = tileset.padding;
        let spacing = tileset.spacing;

        let eff_w = img_w.saturating_sub(padding * 2);
        let eff_h = img_h.saturating_sub(padding * 2);
        let stride_x = tile_w + spacing;
        let stride_y = tile_h + spacing;

        let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(0);
        let rows = (eff_h + spacing).checked_div(stride_y).unwrap_or(0);
        let grid = TileGrid::new(tile_w, tile_h, spacing, padding, cols, rows);

        // 1. Allocate exact viewport rect (Canvas)
        let (viewport_rect, response) =
            ui.allocate_exact_size(popup_rect.size(), Sense::click_and_drag());
        let viewport_w = viewport_rect.width();
        let viewport_h = viewport_rect.height();

        // 2. Initial zoom & Centering on open (Section 1)
        if self.needs_initial_fit {
            let avail_w = (viewport_w - 48.0).max(10.0);
            let avail_h = (viewport_h - 48.0).max(10.0);
            let fit_zoom = if img_w > 0 && img_h > 0 {
                (avail_w / img_w as f32)
                    .min(avail_h / img_h as f32)
                    .clamp(0.25, 32.0)
            } else {
                1.0
            };
            self.view.zoom = fit_zoom;
            let zoomed_w = img_w as f32 * fit_zoom;
            let zoomed_h = img_h as f32 * fit_zoom;
            let center_x = viewport_rect.min.x + (viewport_w - zoomed_w) / 2.0;
            let center_y = viewport_rect.min.y + (viewport_h - zoomed_h) / 2.0;
            self.view.offset = vec2(center_x.round(), center_y.round());
            self.needs_initial_fit = false;
        }

        // 3. Navigation Controls (Section 1)
        // Space + left-drag or middle-drag to pan
        let space_down = ui.ctx().input(|i| i.key_down(egui::Key::Space));
        if space_down {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        if response.dragged_by(PointerButton::Middle)
            || (space_down && response.dragged_by(PointerButton::Primary))
        {
            if space_down {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            self.view.offset += response.drag_delta();
        }

        // Mouse wheel: scroll for vertical, Shift+scroll for horizontal
        let (modifiers, scroll_delta) = ui.ctx().input(|i| (i.modifiers, i.smooth_scroll_delta));
        if !modifiers.ctrl
            && !modifiers.command
            && (scroll_delta.x.abs() > 0.0 || scroll_delta.y.abs() > 0.0)
        {
            if let Some(cursor) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                if viewport_rect.contains(cursor) {
                    if modifiers.shift {
                        let dx = if scroll_delta.x.abs() > 0.0 {
                            scroll_delta.x
                        } else {
                            scroll_delta.y
                        };
                        self.view.offset.x += dx;
                    } else {
                        self.view.offset.y += scroll_delta.y;
                        if scroll_delta.x.abs() > 0.0 {
                            self.view.offset.x += scroll_delta.x;
                        }
                    }
                }
            }
        }

        // Ctrl+scroll (zoom_delta) to zoom toward the cursor
        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 {
            if let Some(cursor) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                if viewport_rect.contains(cursor) {
                    let page_cursor = self.view.screen_to_page(cursor);
                    let old_zoom = self.view.zoom;
                    let new_zoom = (old_zoom * zoom_delta).clamp(0.25, 32.0);
                    if (new_zoom - old_zoom).abs() > 0.0001 {
                        self.view.offset.x = (cursor.x - page_cursor.x * new_zoom).round();
                        self.view.offset.y = (cursor.y - page_cursor.y * new_zoom).round();
                        self.view.zoom = new_zoom;
                    }
                }
            }
        }

        // Workspace size = zoomed page size + margin on all 4 sides (Section 1)
        let zoomed_w = img_w as f32 * self.view.zoom;
        let zoomed_h = img_h as f32 * self.view.zoom;
        self.view.offset.x = self.view.offset.x.clamp(
            viewport_rect.min.x + 24.0 - zoomed_w,
            viewport_rect.max.x - 24.0,
        );
        self.view.offset.y = self.view.offset.y.clamp(
            viewport_rect.min.y + 24.0 - zoomed_h,
            viewport_rect.max.y - 24.0,
        );

        // 4. Multi-select with Right Click Drag (Section 3)
        let right_pressed = ui
            .ctx()
            .input(|i| i.pointer.button_pressed(PointerButton::Secondary));
        let right_down = ui
            .ctx()
            .input(|i| i.pointer.button_down(PointerButton::Secondary));
        let right_released = ui
            .ctx()
            .input(|i| i.pointer.button_released(PointerButton::Secondary));

        if right_pressed {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                if viewport_rect.contains(pos) && cols > 0 && rows > 0 {
                    let page_pos = self.view.screen_to_page(pos);
                    let stride_xf = stride_x.max(1) as f32;
                    let stride_yf = stride_y.max(1) as f32;
                    let rel_x = page_pos.x - padding as f32;
                    let rel_y = page_pos.y - padding as f32;
                    let col = (rel_x / stride_xf)
                        .floor()
                        .clamp(0.0, cols.saturating_sub(1) as f32)
                        as u32;
                    let row = (rel_y / stride_yf)
                        .floor()
                        .clamp(0.0, rows.saturating_sub(1) as f32)
                        as u32;

                    self.drag_start_cell = Some((col, row));
                    self.is_right_dragging = true;
                    let single_rect = TileRect::single(col, row);
                    self.live_drag_rect = Some(single_rect);
                    self.selected_tile_rect = Some(single_rect);
                }
            }
        }

        if self.is_right_dragging && right_down {
            if let Some(start_cell) = self.drag_start_cell {
                if let Some(curr_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    // Auto-scroll when nearing viewport edge (24 px)
                    let edge_margin = 24.0;
                    let scroll_step = 6.0;
                    let mut auto_scroll = vec2(0.0, 0.0);
                    if curr_pos.x < viewport_rect.min.x + edge_margin {
                        auto_scroll.x += scroll_step;
                    }
                    if curr_pos.x > viewport_rect.max.x - edge_margin {
                        auto_scroll.x -= scroll_step;
                    }
                    if curr_pos.y < viewport_rect.min.y + edge_margin {
                        auto_scroll.y += scroll_step;
                    }
                    if curr_pos.y > viewport_rect.max.y - edge_margin {
                        auto_scroll.y -= scroll_step;
                    }
                    if auto_scroll != vec2(0.0, 0.0) {
                        self.view.offset += auto_scroll;
                        ui.ctx().request_repaint();
                    }

                    let curr_page = self.view.screen_to_page(curr_pos);
                    let stride_xf = stride_x.max(1) as f32;
                    let stride_yf = stride_y.max(1) as f32;
                    let rel_x = curr_page.x - padding as f32;
                    let rel_y = curr_page.y - padding as f32;
                    let end_col = (rel_x / stride_xf)
                        .floor()
                        .clamp(0.0, cols.saturating_sub(1) as f32)
                        as u32;
                    let end_row = (rel_y / stride_yf)
                        .floor()
                        .clamp(0.0, rows.saturating_sub(1) as f32)
                        as u32;

                    let sel =
                        TileRect::from_cells(start_cell, (end_col, end_row)).clamp(cols, rows);
                    self.live_drag_rect = Some(sel);
                    self.selected_tile_rect = Some(sel);
                }
            }
        }

        if self.is_right_dragging && right_released {
            self.is_right_dragging = false;
            self.drag_start_cell = None;
            if let Some(sel) = self.live_drag_rect.take().or(self.selected_tile_rect) {
                self.selected_tile_rect = Some(sel);
                self.is_open = false;
                *event = TilePickerEvent::Selected(sel);
            }
        }

        // 5. Hover Detection & Left Click (Section 3 & 5)
        let mut hovered_c_r = None;
        let mut hovered_tid = None;
        if !self.is_right_dragging && !space_down {
            if let Some(mpos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                if viewport_rect.contains(mpos) {
                    let page_pos = self.view.screen_to_page(mpos);
                    if let Some(cell) = grid.cell_at(page_pos) {
                        hovered_c_r = Some(cell);
                        hovered_tid = Some(grid.tile_id(cell.0, cell.1));
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                }
            }
        }
        self.hovered_tile = hovered_tid;

        // Left click selects 1x1 TileRect and closes (Section 5)
        if !self.is_right_dragging && !space_down && response.clicked() {
            if let Some(cell) = hovered_c_r {
                let sel = TileRect::single(cell.0, cell.1);
                self.selected_tile_rect = Some(sel);
                self.is_open = false;
                *event = TilePickerEvent::Selected(sel);
            }
        }

        // 6. Rendering (Painter with viewport clip)
        let painter = ui.painter().with_clip_rect(viewport_rect);

        // Workspace background (Section 1)
        painter.rect_filled(viewport_rect, CornerRadius::ZERO, t.canvas);

        // Page panel bounds
        let page_rect = Rect::from_min_size(Pos2::ZERO, vec2(img_w as f32, img_h as f32));
        let screen_page_rect = self.view.page_to_screen_rect(page_rect);

        // Drop shadow behind the page — fixed in screen pixels so it never
        // grows or shrinks when the user zooms in or out.
        painter.add(
            egui::epaint::Shadow {
                offset: [4, 6],
                blur: 14,
                spread: 0,
                color: Color32::from_black_alpha(130),
            }
            .as_shape(screen_page_rect, CornerRadius::ZERO),
        );

        // Opaque page: flat fallback first, then the neutral tiled background.
        // This replaces the transparency checkerboard, so alpha tiles sit on a
        // readable, non-transparent surface that covers the whole tileset.
        // The pattern is tiled in page space, so it zooms together with the
        // tileset and keeps its size relative to the artwork.
        painter.rect_filled(screen_page_rect, CornerRadius::ZERO, t.panel);
        if let Some(bg) = self.page_background(ui.ctx()) {
            painter.image(
                bg.id(),
                screen_page_rect,
                page_bg_uv(page_rect),
                Color32::WHITE,
            );
        }

        // Render tileset texture with NEAREST filtering
        painter.image(
            tex_info.handle.id(),
            screen_page_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // Page outline — always 2 screen pixels regardless of zoom level.
        // Using StrokeKind::Outside so it frames the tileset without
        // covering edge tiles at any zoom.
        painter.rect_stroke(
            screen_page_rect,
            CornerRadius::ZERO,
            Stroke::new(2.0, Color32::BLACK),
            StrokeKind::Outside,
        );

        // 7. Grid Lines (Section 2: drawn strictly inside page bounds, culled)
        if self.show_grid && cols > 0 && rows > 0 {
            let grid_clip = screen_page_rect.intersect(viewport_rect);
            let grid_painter = painter.with_clip_rect(grid_clip);
            let step_x = (stride_x as f32 * self.view.zoom).round();
            let step_y = (stride_y as f32 * self.view.zoom).round();

            if step_x >= 4.0 && step_y >= 4.0 {
                let grid_stroke = Stroke::new(1.0, Color32::from_white_alpha(35));
                let mut shapes = Vec::new();

                for c in 0..=cols {
                    let page_x = if c < cols {
                        grid.cell_rect(c, 0).min.x
                    } else {
                        grid.cell_rect(cols - 1, 0).max.x
                    };
                    let screen_x = (page_x * self.view.zoom + self.view.offset.x).round();
                    if screen_x >= grid_clip.min.x && screen_x <= grid_clip.max.x {
                        shapes.push(egui::Shape::line_segment(
                            [
                                Pos2::new(screen_x, grid_clip.min.y),
                                Pos2::new(screen_x, grid_clip.max.y),
                            ],
                            grid_stroke,
                        ));
                    }
                }

                for r in 0..=rows {
                    let page_y = if r < rows {
                        grid.cell_rect(0, r).min.y
                    } else {
                        grid.cell_rect(0, rows - 1).max.y
                    };
                    let screen_y = (page_y * self.view.zoom + self.view.offset.y).round();
                    if screen_y >= grid_clip.min.y && screen_y <= grid_clip.max.y {
                        shapes.push(egui::Shape::line_segment(
                            [
                                Pos2::new(grid_clip.min.x, screen_y),
                                Pos2::new(grid_clip.max.x, screen_y),
                            ],
                            grid_stroke,
                        ));
                    }
                }

                grid_painter.extend(shapes);
            }
        }

        // 8. Selection Highlight (Section 3: live drag cyan or committed amber)
        let active_sel = self.live_drag_rect.or(self.selected_tile_rect);
        if let Some(sel) = active_sel {
            if cols > 0 && rows > 0 {
                let clamped = sel.clamp(cols, rows);
                let page_union = clamped.union_rect(&grid);
                let screen_box = self.view.page_to_screen_rect(page_union);

                if self.is_right_dragging {
                    // While dragging: cyan stroke 2px, subtle transparent cyan fill
                    painter.rect_filled(
                        screen_box,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(61, 214, 184, 45),
                    );
                    painter.rect_stroke(
                        screen_box,
                        CornerRadius::ZERO,
                        Stroke::new(2.0, t.accent),
                        StrokeKind::Inside,
                    );
                } else {
                    // Committed selection: amber/gold 2.5px
                    painter.rect_filled(
                        screen_box,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(255, 193, 77, 60),
                    );
                    painter.rect_stroke(
                        screen_box,
                        CornerRadius::ZERO,
                        Stroke::new(2.5, Color32::from_rgb(255, 193, 77)),
                        StrokeKind::Inside,
                    );
                }
            }
        }

        // 9. Single Tile Hover Highlight Box (when not dragging)
        if !self.is_right_dragging {
            if let Some(cell) = hovered_c_r {
                let cell_page = grid.cell_rect(cell.0, cell.1);
                let cell_screen = self.view.page_to_screen_rect(cell_page);
                painter.rect_filled(
                    cell_screen,
                    CornerRadius::ZERO,
                    Color32::from_rgba_unmultiplied(61, 214, 184, 45),
                );
                painter.rect_stroke(
                    cell_screen,
                    CornerRadius::ZERO,
                    Stroke::new(1.5, t.accent),
                    StrokeKind::Inside,
                );
            }
        }

        // 10. Tile ID / Dimensions Badge (Section 3: "#742 (3x2)" or "#742")
        let active_rect = self.live_drag_rect.or(self.selected_tile_rect);
        let badge_info = if let Some(r) = active_rect {
            if cols > 0 && rows > 0 {
                let c = r.clamp(cols, rows);
                let id = c.top_left_id(&grid);
                if c.w > 1 || c.h > 1 {
                    Some(format!("#{id} ({}x{})", c.w, c.h))
                } else {
                    Some(format!("#{id}"))
                }
            } else {
                None
            }
        } else {
            self.hovered_tile.map(|id| format!("#{id}"))
        };

        if let Some(badge_text) = badge_info {
            let font_id = egui::FontId::monospace(12.0);
            let galley = ui
                .painter()
                .layout_no_wrap(badge_text, font_id, t.ink_strong);
            let badge_padding = vec2(8.0, 5.0);
            let badge_size = galley.size() + badge_padding * 2.0;

            let badge_pos = Pos2::new(
                popup_rect.right() - badge_size.x - 12.0,
                popup_rect.top() + 12.0,
            );
            let badge_rect = Rect::from_min_size(badge_pos, badge_size);

            let badge_painter = ui.painter().with_clip_rect(popup_rect);
            badge_painter.rect_filled(
                badge_rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(220),
            );
            badge_painter.rect_stroke(
                badge_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, t.line_strong),
                StrokeKind::Inside,
            );
            badge_painter.galley(badge_pos + badge_padding, galley, t.ink_strong);
        }
    }

    /// Auto-detects tile size from image filename or dimensions without hardcoding.
    pub fn detect_tile_size(path: &str, img_w: u32, img_h: u32) -> glam::UVec2 {
        if let Some(stem) = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
        {
            if let Some(pos) = stem.rfind('_') {
                if let Ok(num) = stem[pos + 1..].parse::<u32>() {
                    if (4..=256).contains(&num) {
                        return glam::UVec2::new(num, num);
                    }
                }
            }
        }
        for candidate in [16, 32, 24, 8, 48, 64] {
            if img_w.is_multiple_of(candidate) && img_h.is_multiple_of(candidate) {
                return glam::UVec2::new(candidate, candidate);
            }
        }
        glam::UVec2::new(16, 16)
    }

    /// Loads (once) and returns the embedded neutral page background texture.
    ///
    /// The texture repeats via [`TextureOptions::NEAREST_REPEAT`], so a single
    /// image quad can tile it across the whole tileset page.
    fn page_background(&mut self, ctx: &egui::Context) -> Option<TextureHandle> {
        if let Some(handle) = &self.page_bg {
            return Some(handle.clone());
        }

        let img = image::load_from_memory(PAGE_BG_BYTES).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        let size = [w as usize, h as usize];
        let pixels = img.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let handle = ctx.load_texture(
            "tile_picker_page_bg",
            color_image,
            TextureOptions::NEAREST_REPEAT,
        );
        self.page_bg = Some(handle.clone());
        Some(handle)
    }

    /// Loads or fetches a cached TextureHandle for the specified tileset.
    fn get_or_load_texture(
        &mut self,
        ctx: &egui::Context,
        tileset: &TilesetData,
    ) -> Result<CachedTextureInfo, String> {
        if let Some(cached) = self.textures.get(&tileset.id) {
            return cached.clone();
        }

        let res = Self::load_image_to_texture(ctx, tileset);
        self.textures.insert(tileset.id, res.clone());
        res
    }

    /// Reads tileset image file bytes and uploads them as a NEAREST TextureHandle.
    fn load_image_to_texture(
        ctx: &egui::Context,
        tileset: &TilesetData,
    ) -> Result<CachedTextureInfo, String> {
        let candidate_paths = [
            PathBuf::from(&tileset.path),
            PathBuf::from("examples/sample_project").join(&tileset.path),
            PathBuf::from("examples").join(&tileset.path),
            PathBuf::from("../../").join(&tileset.path),
            PathBuf::from("../../examples/sample_project").join(&tileset.path),
            PathBuf::from("../../examples").join(&tileset.path),
        ];

        let mut loaded_bytes = None;
        let mut last_err = String::new();

        for p in &candidate_paths {
            match lc_io::read_bytes(p) {
                Ok(bytes) => {
                    loaded_bytes = Some(bytes);
                    break;
                }
                Err(e) => {
                    last_err = format!("{}: {e}", p.display());
                }
            }
        }

        let bytes = loaded_bytes.ok_or_else(|| {
            if last_err.is_empty() {
                format!("File not found: {}", tileset.path)
            } else {
                last_err
            }
        })?;

        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("Failed to decode image {}: {e}", tileset.path))?;
        let mut rgba = img.to_rgba8();
        let (real_w, real_h) = rgba.dimensions();

        let max_side = ctx.input(|i| i.max_texture_side) as u32;
        let scale = if real_w > max_side || real_h > max_side {
            let s_w = max_side as f32 / real_w as f32;
            let s_h = max_side as f32 / real_h as f32;
            s_w.min(s_h)
        } else {
            1.0
        };

        if scale < 1.0 {
            let target_w = (real_w as f32 * scale).round() as u32;
            let target_h = (real_h as f32 * scale).round() as u32;
            rgba = image::imageops::resize(
                &rgba,
                target_w,
                target_h,
                image::imageops::FilterType::Nearest,
            );
        }

        let (tex_w, tex_h) = rgba.dimensions();
        let size = [tex_w as usize, tex_h as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
        let handle = ctx.load_texture(
            format!("tile_picker_{:?}", tileset.id),
            color_image,
            TextureOptions::NEAREST,
        );

        Ok(CachedTextureInfo {
            handle,
            real_size: (real_w, real_h),
            texture_size: (tex_w, tex_h),
            scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_page_background_decodes() {
        let img = image::load_from_memory(PAGE_BG_BYTES).expect("embedded bg must decode");
        assert_eq!((img.width(), img.height()), (16, 16));
        assert_eq!(PAGE_BG_CELL, 16.0);
    }

    #[test]
    fn page_bg_uv_tiles_by_page_cell() {
        // Page-space rect: a 64x32 tileset repeats the 16 px pattern 4x2 times,
        // independent of the zoom used to draw it.
        let page = Rect::from_min_size(Pos2::new(40.0, 10.0), vec2(64.0, 32.0));
        let uv = page_bg_uv(page);
        assert_eq!(uv.min, Pos2::ZERO);
        assert_eq!(uv.width(), 4.0);
        assert_eq!(uv.height(), 2.0);
    }

    #[test]
    fn page_bg_uv_is_zero_for_empty_rect() {
        let uv = page_bg_uv(Rect::ZERO);
        assert_eq!(uv.width(), 0.0);
        assert_eq!(uv.height(), 0.0);
    }
}
