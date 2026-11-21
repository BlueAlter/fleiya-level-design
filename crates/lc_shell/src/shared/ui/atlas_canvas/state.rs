//! Interactive state and events for the atlas canvas.

use egui::{Modifiers, Painter, PointerButton, Rect};

use super::super::tile_bounds::TileRect;
use super::view::{compute_fit_view, PickerView};

/// Callback for rendering custom per-tile overlays on top of the tile grid.
pub type TileOverlayCallback<'a> = dyn FnMut(&Painter, (u32, u32), Rect) + 'a;

/// Predicate deciding whether a given tile coordinate `(col, row)` should be dimmed.
pub type TileDimPredicate<'a> = dyn Fn((u32, u32)) -> bool + 'a;

/// Events produced during user interaction with the atlas canvas.
#[derive(Debug, Clone, PartialEq)]
pub enum AtlasEvent {
    /// Currently hovered tile ID (or `None` when hovering empty space / outside).
    Hovered(Option<u32>),
    /// A tile was clicked with pointer button and keyboard modifiers.
    Clicked {
        /// Tile column and row in grid coordinates.
        cell: (u32, u32),
        /// Tile ID in grid coordinates.
        tile_id: u32,
        /// Pointer button used.
        button: PointerButton,
        /// Keyboard modifiers held during click.
        modifiers: Modifiers,
    },
    /// A multi-selection rectangular drag started in tile coordinates.
    DragRectStart(TileRect),
    /// A multi-selection rectangular drag updated in tile coordinates.
    DragRectUpdate(TileRect),
    /// A multi-selection rectangular drag finished with the final selection.
    DragRectEnd(TileRect),
}

/// Interactive state of the atlas canvas viewport.
#[derive(Debug, Clone)]
pub struct AtlasCanvasState {
    /// Viewport zoom and pan offset.
    pub view: PickerView,
    /// Currently hovered tile ID.
    pub hovered_tile: Option<u32>,
    /// Active committed tile selection.
    pub selected_tile_rect: Option<TileRect>,
    /// In-progress drag selection.
    pub live_drag_rect: Option<TileRect>,
    /// Starting cell for drag selection.
    pub drag_start_cell: Option<(u32, u32)>,
    /// Whether right-mouse drag multi-selection is actively occurring.
    pub is_right_dragging: bool,
    /// Whether to draw tile grid lines.
    pub show_grid: bool,
    /// Whether initial auto-fit is pending.
    pub needs_initial_fit: bool,
}

impl Default for AtlasCanvasState {
    fn default() -> Self {
        Self {
            view: PickerView::default(),
            hovered_tile: None,
            selected_tile_rect: None,
            live_drag_rect: None,
            drag_start_cell: None,
            is_right_dragging: false,
            show_grid: true,
            needs_initial_fit: true,
        }
    }
}

impl AtlasCanvasState {
    /// Creates a new canvas state with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fits the atlas page into the given viewport rect.
    pub fn fit_to_viewport(&mut self, viewport_rect: Rect, img_w: f32, img_h: f32) {
        let (zoom, offset) = compute_fit_view(viewport_rect, img_w, img_h);
        self.view.zoom = zoom;
        self.view.offset = offset;
    }
}
