//! lc_feature_tileset: Tileset definition tab, tile palette panel, tile commands, and layer handlers.

#![forbid(unsafe_code)]

pub mod commands;
pub mod handler;
pub mod paint_commands;
pub mod palette;
pub mod tools;
pub mod view;

pub use commands::{AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand};
pub use handler::{IntGridLayerHandler, TilesLayerHandler};
pub use paint_commands::{
    FillCommand, FillValue, PaintIntGridCommand, PaintTilesCommand, RectCommand,
};
pub use palette::TilePalettePanel;
pub use tools::{EraseTool, EyedropperTool, FillTool, PaintTool, RectTool, TileCanvasTool};
pub use view::TilesetDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the tileset document view factory, tile palette panel, and layer handlers with the application registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::TilesetDef(tileset_id) => Some(Box::new(TilesetDocumentView::new(tileset_id))),
        _ => None,
    });

    registry.register_panel(Box::new(TilePalettePanel::new()));

    registry.register_layer_handler(Box::new(TilesLayerHandler::new()));
    registry.register_layer_handler(Box::new(IntGridLayerHandler::new()));
}
