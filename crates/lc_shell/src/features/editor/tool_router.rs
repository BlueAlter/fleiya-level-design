//! Canvas tool routing shared by the shell's level tabs.
//!
//! The level document view owns a single active `CanvasTool`, while painting
//! tiles and editing entities need different implementations. This router
//! picks the tool that matches the active layer kind each frame, so palette
//! tool names (`Paint`, `Erase`, `Place #<uid>`, ...) keep working on both
//! layer families without a contract change.

use lc_core::layer::LayerKind;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;

/// Dispatches canvas input to the tile tool or the entity tool.
pub struct LevelCanvasToolRouter {
    tiles: Box<dyn CanvasTool>,
    entities: Box<dyn CanvasTool>,
}

impl LevelCanvasToolRouter {
    /// Creates a router with the default tile and entity tools.
    pub fn new() -> Self {
        Self {
            tiles: Box::new(lc_feature_tileset::TileCanvasTool::new(LayerKind::Tiles)),
            entities: Box::new(lc_feature_entities::EntityCanvasTool::new()),
        }
    }

    fn active_kind(
        &self,
        cx: &EditorCtx,
        active_layer: Option<lc_core::id::LayerDefId>,
    ) -> Option<LayerKind> {
        active_layer
            .and_then(|id| cx.project().defs.layer(id))
            .map(|def| def.kind)
    }
}

impl Default for LevelCanvasToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasTool for LevelCanvasToolRouter {
    fn name(&self) -> &str {
        "Level Canvas"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let kind = self.active_kind(cx, cv.active_layer_id);
        match kind {
            Some(LayerKind::Entities) => self.entities.on_input(cv, cx),
            _ => self.tiles.on_input(cv, cx),
        }
    }
}
