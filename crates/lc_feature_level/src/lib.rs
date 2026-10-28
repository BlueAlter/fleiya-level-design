//! lc_feature_level: Level editor document view, canvas host, pan/zoom, grid, and inspector.

#![forbid(unsafe_code)]

pub mod canvas;
pub mod view;

pub use canvas::{render_level_canvas, LevelCanvasState};
pub use view::LevelDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the level document factory with the application registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::Level(level_id) => Some(Box::new(LevelDocumentView::new(level_id))),
        _ => None,
    });
}
