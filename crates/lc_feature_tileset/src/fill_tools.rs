//! Flood-fill and eyedropper canvas tools for Tiles and IntGrid layers.

use egui::PointerButton;
use glam::UVec2;
use lc_core::layer::{LayerData, LayerKind};
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::TileSelection;

use crate::paint_commands::FillCommand;

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

        if cv.response.clicked_by(PointerButton::Primary) {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    let start = UVec2::new(pos.x as u32, pos.y as u32);
                    match layer_kind {
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

    /// Default target layer kind.
    pub fn kind(&self) -> LayerKind {
        self.kind
    }
}

impl CanvasTool for EyedropperTool {
    fn name(&self) -> &str {
        "Eyedropper"
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

        if cv.response.clicked_by(PointerButton::Primary) {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    let (x, y) = (pos.x as u32, pos.y as u32);
                    let mut tile_to_pick = None;
                    let mut int_to_pick = None;

                    if let Some(level) = cx.project().level(cv.active_level_id) {
                        if let Some(layer) = level.layer(active_layer_id) {
                            match &layer.data {
                                LayerData::Tiles(grid) => {
                                    if let Some(Some(tile_ref)) = grid.get(x, y) {
                                        tile_to_pick = Some(*tile_ref);
                                    }
                                }
                                LayerData::IntGrid(grid) => {
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
