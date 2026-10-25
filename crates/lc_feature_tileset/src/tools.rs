//! Interactive canvas tools for painting, erasing, rectangle fill, flood fill, and eyedropper on Tiles and IntGrid layers.

use egui::{Key, PointerButton};
use glam::UVec2;
use lc_core::layer::{LayerData, LayerKind, TileRef};
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::TileSelection;

use crate::paint_commands::{
    FillCommand, FillValue, PaintIntGridCommand, PaintTilesCommand, RectCommand,
};

static STROKE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_stroke_id() -> u64 {
    STROKE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

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

        let active_layer_id = match cv.active_layer_id {
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

            match self.kind {
                LayerKind::Tiles => {
                    if let Some(ts) = cx.tile_selection() {
                        let mut cells = Vec::new();
                        for (i, &tile_id) in ts.tiles.iter().enumerate() {
                            let dx = (i as u32) % ts.width;
                            let dy = (i as u32) / ts.width;
                            let cell_x = grid_pos.x as u32 + dx;
                            let cell_y = grid_pos.y as u32 + dy;

                            let mut tile_ref = TileRef::new(ts.tileset_id, tile_id);
                            tile_ref.flip_x = self.flip_x;
                            tile_ref.flip_y = self.flip_y;
                            cells.push((cell_x, cell_y, Some(tile_ref)));
                        }

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
        let active_layer_id = match cv.active_layer_id {
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

            match self.kind {
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
        let active_layer_id = match cv.active_layer_id {
            Some(id) => id,
            None => return,
        };

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
                    let fill_val = match self.kind {
                        LayerKind::Tiles => {
                            let tile = cx.tile_selection().and_then(|ts| {
                                ts.primary_tile().map(|mut t| {
                                    t.flip_x = self.flip_x;
                                    t.flip_y = self.flip_y;
                                    t
                                })
                            });
                            FillValue::Tile(tile)
                        }
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

/// Canvas tool for flood filling contiguous areas.
#[derive(Debug)]
pub struct FillTool {
    kind: LayerKind,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl FillTool {
    /// Constructs a new `FillTool` for the given layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self {
            kind,
            flip_x: false,
            flip_y: false,
        }
    }
}

impl CanvasTool for FillTool {
    fn name(&self) -> &str {
        "Fill"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let active_layer_id = match cv.active_layer_id {
            Some(id) => id,
            None => return,
        };

        if cv.response.clicked_by(PointerButton::Primary) {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    let start = UVec2::new(pos.x as u32, pos.y as u32);
                    match self.kind {
                        LayerKind::Tiles => {
                            let target_tile = cx.tile_selection().and_then(|ts| {
                                ts.primary_tile().map(|mut t| {
                                    t.flip_x = self.flip_x;
                                    t.flip_y = self.flip_y;
                                    t
                                })
                            });
                            let cmd = FillCommand::tiles(
                                cv.active_level_id,
                                active_layer_id,
                                start,
                                target_tile,
                            );
                            cx.exec(cmd);
                        }
                        LayerKind::IntGrid => {
                            let cmd = FillCommand::int_grid(
                                cv.active_level_id,
                                active_layer_id,
                                start,
                                1,
                            );
                            cx.exec(cmd);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Canvas tool for picking placed tiles or integer values from the canvas.
#[derive(Debug)]
pub struct EyedropperTool {
    kind: LayerKind,
}

impl EyedropperTool {
    /// Constructs a new `EyedropperTool` for the given layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self { kind }
    }
}

impl CanvasTool for EyedropperTool {
    fn name(&self) -> &str {
        "Eyedropper"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let active_layer_id = match cv.active_layer_id {
            Some(id) => id,
            None => return,
        };

        if cv.response.clicked_by(PointerButton::Primary) {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    let (x, y) = (pos.x as u32, pos.y as u32);
                    let mut tile_to_pick = None;
                    let mut int_to_pick = None;

                    if let Some(level) = cx.project().level(cv.active_level_id) {
                        if let Some(layer) = level.layer(active_layer_id) {
                            match (&self.kind, &layer.data) {
                                (LayerKind::Tiles, LayerData::Tiles(grid)) => {
                                    if let Some(Some(tile_ref)) = grid.get(x, y) {
                                        tile_to_pick = Some(*tile_ref);
                                    }
                                }
                                (LayerKind::IntGrid, LayerData::IntGrid(grid)) => {
                                    if let Some(&val) = grid.get(x, y) {
                                        int_to_pick = Some(val);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(tile_ref) = tile_to_pick {
                        let sel = TileSelection::single(tile_ref.tileset_id, tile_ref.tile_id);
                        cx.emit(EditorEvent::SetTileSelection(Some(sel)));
                        cx.status(format!(
                            "Picked tile #{} from tileset {:?}",
                            tile_ref.tile_id, tile_ref.tileset_id
                        ));
                    } else if let Some(val) = int_to_pick {
                        cx.status(format!("Picked IntGrid value {val}"));
                    }
                }
            }
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
        }
    }
}

impl CanvasTool for TileCanvasTool {
    fn name(&self) -> &str {
        "Tiles"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        match cx.active_tool_name() {
            "Erase" => self.erase.on_input(cv, cx),
            "Rectangle" | "Rect" => self.rect.on_input(cv, cx),
            "Fill" | "FloodFill" => self.fill.on_input(cv, cx),
            "Eyedropper" | "Picker" => self.eyedropper.on_input(cv, cx),
            _ => self.paint.on_input(cv, cx),
        }
    }
}
