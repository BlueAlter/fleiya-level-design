//! lc_feature_tileset: Tileset definition tab, tile palette panel, and tile commands.

#![forbid(unsafe_code)]

pub mod commands;
pub mod palette;
pub mod view;

pub use commands::{AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand};
pub use palette::TilePalettePanel;
pub use view::TilesetDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the tileset document view factory and tile palette panel with the application registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::TilesetDef(tileset_id) => Some(Box::new(TilesetDocumentView::new(tileset_id))),
        _ => None,
    });

    registry.register_panel(Box::new(TilePalettePanel::new()));
}
