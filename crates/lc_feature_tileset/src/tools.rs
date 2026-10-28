//! Interactive canvas tools for painting, erasing, rectangle fill, flood fill, and eyedropper on Tiles and IntGrid layers.

use egui::{Key, PointerButton};
use glam::UVec2;
use lc_canvas::ids::next_stroke_id;
use lc_core::layer::LayerKind;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;

use crate::brush::{pattern_tiles, stamp_cells};
use crate::clipboard::CanvasClipboard;
pub use crate::fill_tools::{EyedropperTool, FillTool};
pub use crate::select::SelectTool;

use crate::paint_commands::{FillValue, PaintIntGridCommand, PaintTilesCommand, RectCommand};

/// Canvas tool for painting tiles or integer values.
#[derive(Debug)]
pub struct PaintTool {
    kind: LayerKind,
    active_stroke_id: Option<u64>,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl PaintTool {
    /// Constructs a new `PaintTool` for the given layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self {
            kind,
            active_stroke_id: None,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Handles keyboard shortcut toggles (X for Flip X, Y for Flip Y).
    pub fn handle_shortcuts(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        if cv.response.hovered() {
            let (flip_x_pressed, flip_y_pressed) = cv
                .response
                .ctx
                .input(|i| (i.key_pressed(Key::X), i.key_pressed(Key::Y)));

            if flip_x_pressed {
                self.flip_x = !self.flip_x;
                cx.status(format!("Tile Flip X: {}", self.flip_x));
            }
            if flip_y_pressed {
                self.flip_y = !self.flip_y;
                cx.status(format!("Tile Flip Y: {}", self.flip_y));
            }
        }
    }
}

impl CanvasTool for PaintTool {
    fn name(&self) -> &str {
        "Paint"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        self.handle_shortcuts(cv, cx);

        let active_layer_id = cv.active_layer_id.or_else(|| {
            cx.project()
                .level(cv.active_level_id)
                .and_then(|l| l.layers.first().map(|layer| layer.def_id))
        });
        let active_layer_id = match active_layer_id {
            Some(id) => id,
            None => return,
        };

        let is_primary_drag = cv.response.dragged_by(PointerButton::Primary);
        let is_primary_click = cv.response.clicked_by(PointerButton::Primary);

        if is_primary_drag || is_primary_click {
            let grid_pos = match cv.grid_pos {
                Some(pos) if pos.x >= 0 && pos.y >= 0 => pos,
                _ => return,
            };

            let stroke_id = *self.active_stroke_id.get_or_insert_with(next_stroke_id);

            let layer_kind = cx
                .project()
                .defs
                .layers
                .iter()
                .find(|l| l.id == active_layer_id)
                .map(|l| l.kind)
                .unwrap_or(self.kind);

            match layer_kind {
                LayerKind::Tiles => {
                    if let Some(ts) = cx.tile_selection() {
                        let cells = stamp_cells(
                            ts,
                            UVec2::new(grid_pos.x as u32, grid_pos.y as u32),
                            self.flip_x,
                            self.flip_y,
                        );

                        let cmd = PaintTilesCommand::with_stroke(
                            cv.active_level_id,
                            active_layer_id,
                            cells,
                            stroke_id,
                        );
                        cx.exec(cmd);
                    }
                }
                LayerKind::IntGrid => {
                    let cmd = PaintIntGridCommand::with_stroke(
                        cv.active_level_id,
                        active_layer_id,
                        vec![(grid_pos.x as u32, grid_pos.y as u32, 1)],
                        stroke_id,
                    );
                    cx.exec(cmd);
                }
                _ => {}
            }
        }

        if !is_primary_drag && self.active_stroke_id.is_some() {
            self.active_stroke_id = None;
        }
    }
}

/// Canvas tool for erasing tiles or resetting integer values.
#[derive(Debug)]
pub struct EraseTool {
    kind: LayerKind,
    active_stroke_id: Option<u64>,
}

impl EraseTool {
    /// Constructs a new `EraseTool` for the given layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self {
            kind,
            active_stroke_id: None,
        }
    }
}

impl CanvasTool for EraseTool {
    fn name(&self) -> &str {
        "Erase"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let active_layer_id = cv.active_layer_id.or_else(|| {
            cx.project()
                .level(cv.active_level_id)
                .and_then(|l| l.layers.first().map(|layer| layer.def_id))
        });
        let active_layer_id = match active_layer_id {
            Some(id) => id,
            None => return,
        };

        let is_primary_drag = cv.response.dragged_by(PointerButton::Primary);
        let is_primary_click = cv.response.clicked_by(PointerButton::Primary);

        if is_primary_drag || is_primary_click {
            let grid_pos = match cv.grid_pos {
                Some(pos) if pos.x >= 0 && pos.y >= 0 => pos,
                _ => return,
            };

            let stroke_id = *self.active_stroke_id.get_or_insert_with(next_stroke_id);

            let layer_kind = cx
                .project()
                .defs
                .layers
                .iter()
                .find(|l| l.id == active_layer_id)
                .map(|l| l.kind)
                .unwrap_or(self.kind);

            match layer_kind {
                LayerKind::Tiles => {
                    let cmd = PaintTilesCommand::with_stroke(
                        cv.active_level_id,
                        active_layer_id,
                        vec![(grid_pos.x as u32, grid_pos.y as u32, None)],
                        stroke_id,
                    );
                    cx.exec(cmd);
                }
                LayerKind::IntGrid => {
                    let cmd = PaintIntGridCommand::with_stroke(
                        cv.active_level_id,
                        active_layer_id,
                        vec![(grid_pos.x as u32, grid_pos.y as u32, 0)],
                        stroke_id,
                    );
                    cx.exec(cmd);
                }
                _ => {}
            }
        }

        if !is_primary_drag && self.active_stroke_id.is_some() {
            self.active_stroke_id = None;
        }
    }
}

/// Canvas tool for dragging rectangular tile or integer fills.
#[derive(Debug)]
pub struct RectTool {
    kind: LayerKind,
    start_grid: Option<UVec2>,
    active_stroke_id: Option<u64>,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl RectTool {
    /// Constructs a new `RectTool` for the given layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self {
            kind,
            start_grid: None,
            active_stroke_id: None,
            flip_x: false,
            flip_y: false,
        }
    }
}

impl CanvasTool for RectTool {
    fn name(&self) -> &str {
        "Rectangle"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let active_layer_id = cv.active_layer_id.or_else(|| {
            cx.project()
                .level(cv.active_level_id)
                .and_then(|l| l.layers.first().map(|layer| layer.def_id))
        });
        let active_layer_id = match active_layer_id {
            Some(id) => id,
            None => return,
        };

        let layer_kind = cx
            .project()
            .defs
            .layers
            .iter()
            .find(|l| l.id == active_layer_id)
            .map(|l| l.kind)
            .unwrap_or(self.kind);

        if cv.response.drag_started() {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    self.start_grid = Some(UVec2::new(pos.x as u32, pos.y as u32));
                    self.active_stroke_id = Some(next_stroke_id());
                }
            }
        }

        if cv.response.dragged_by(PointerButton::Primary) {
            if let (Some(start), Some(stroke_id), Some(pos)) =
                (self.start_grid, self.active_stroke_id, cv.grid_pos)
            {
                if pos.x >= 0 && pos.y >= 0 {
                    let current = UVec2::new(pos.x as u32, pos.y as u32);
                    let fill_val = match layer_kind {
                        LayerKind::Tiles => match cx.tile_selection() {
                            Some(ts) if ts.tiles.len() > 1 => FillValue::TilePattern {
                                tiles: pattern_tiles(ts, self.flip_x, self.flip_y),
                                width: ts.width.max(1),
                            },
                            Some(ts) => {
                                let tile = ts.primary_tile().map(|mut t| {
                                    t.flip_x = self.flip_x;
                                    t.flip_y = self.flip_y;
                                    t
                                });
                                FillValue::Tile(tile)
                            }
                            None => FillValue::Tile(None),
                        },
                        LayerKind::IntGrid => FillValue::Int(1),
                        _ => return,
                    };

                    let cmd = RectCommand::with_stroke(
                        cv.active_level_id,
                        active_layer_id,
                        start,
                        current,
                        fill_val,
                        stroke_id,
                    );
                    cx.exec(cmd);
                }
            }
        }

        if cv.response.drag_stopped() {
            self.start_grid = None;
            self.active_stroke_id = None;
        }
    }
}

/// Unified composite canvas tool that dynamically delegates to sub-tools based on `cx.active_tool_name()`.
#[derive(Debug)]
pub struct TileCanvasTool {
    paint: PaintTool,
    erase: EraseTool,
    rect: RectTool,
    fill: FillTool,
    eyedropper: EyedropperTool,
    select: crate::select::SelectTool,
    clipboard: CanvasClipboard,
}

impl TileCanvasTool {
    /// Constructs a unified tile canvas tool.
    pub fn new(kind: LayerKind) -> Self {
        Self {
            paint: PaintTool::new(kind),
            erase: EraseTool::new(kind),
            rect: RectTool::new(kind),
            fill: FillTool::new(kind),
            eyedropper: EyedropperTool::new(kind),
            select: crate::select::SelectTool::new(kind),
            clipboard: CanvasClipboard::new(),
        }
    }
}

impl Default for TileCanvasTool {
    fn default() -> Self {
        Self::new(LayerKind::Tiles)
    }
}

impl CanvasTool for TileCanvasTool {
    fn name(&self) -> &str {
        "Tiles"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        if self.clipboard.handle_shortcuts(cv, cx) {
            return;
        }
        match cx.active_tool_name() {
            "Erase" => self.erase.on_input(cv, cx),
            "Rectangle" | "Rect" => self.rect.on_input(cv, cx),
            "Fill" | "FloodFill" => self.fill.on_input(cv, cx),
            "Eyedropper" | "Picker" => self.eyedropper.on_input(cv, cx),
            "Select" => self.select.on_input(cv, cx),
            _ => self.paint.on_input(cv, cx),
        }
    }
}
