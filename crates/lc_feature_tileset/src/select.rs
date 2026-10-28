//! Canvas tool for selecting rectangular regions of tile or IntGrid cells.

use glam::UVec2;
use lc_core::layer::LayerKind;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::selection::Selection;

use crate::brush::rect_cells;

/// Canvas tool for selecting a rectangular region of tile or IntGrid cells.
#[derive(Debug)]
pub struct SelectTool {
    kind: LayerKind,
    start: Option<UVec2>,
}

impl SelectTool {
    /// Constructs a new `SelectTool` for the given fallback layer kind.
    pub fn new(kind: LayerKind) -> Self {
        Self { kind, start: None }
    }
}

impl CanvasTool for SelectTool {
    fn name(&self) -> &str {
        "Select"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let Some(active_layer_id) = cv.active_layer_id.or_else(|| {
            cx.project()
                .level(cv.active_level_id)
                .and_then(|l| l.layers.first().map(|layer| layer.def_id))
        }) else {
            return;
        };

        let layer_kind = cx
            .project()
            .defs
            .layers
            .iter()
            .find(|l| l.id == active_layer_id)
            .map(|l| l.kind)
            .unwrap_or(self.kind);
        if !matches!(layer_kind, LayerKind::Tiles | LayerKind::IntGrid) {
            return;
        }

        if (cv.response.is_pointer_button_down_on() || cv.response.drag_started())
            && self.start.is_none()
        {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    self.start = Some(UVec2::new(pos.x as u32, pos.y as u32));
                }
            }
        }

        let clicked = cv.response.clicked_by(egui::PointerButton::Primary);
        let dragged = cv.response.dragged_by(egui::PointerButton::Primary);
        if clicked || dragged {
            if let Some(pos) = cv.grid_pos {
                if pos.x >= 0 && pos.y >= 0 {
                    let current = UVec2::new(pos.x as u32, pos.y as u32);
                    let start = if clicked {
                        current
                    } else {
                        self.start.unwrap_or(current)
                    };
                    let cells = rect_cells(start, current);
                    cx.emit(EditorEvent::SelectionChanged(Selection::Tiles {
                        layer_id: active_layer_id,
                        cells,
                    }));
                }
            }
        }

        if cv.response.drag_stopped() {
            self.start = None;
        }
    }
}
