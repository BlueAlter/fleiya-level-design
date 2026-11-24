//! lc_feature_tileset: Tileset definition tab, tile palette panel, tile commands, and layer handlers.

#![forbid(unsafe_code)]

pub mod animation_commands;
mod brush;
pub mod clipboard;
pub mod commands;
pub mod custom_data_commands;
mod fill_tools;
pub mod handler;
pub mod helpers;
pub mod marking_command;
pub mod marking_commands;
pub mod paint_commands;
pub mod palette;
pub mod reorder_command;
mod select;
pub mod tag_commands;
pub mod tools;
pub mod view;

pub use animation_commands::{
    AddAnimationFrameCommand, RemoveAnimationFrameCommand, RemoveTileAnimationCommand,
    SetTileAnimationCommand,
};
pub use clipboard::TileClipboard;
pub use commands::{
    AddTilesetDefCommand, RemoveTilesetDefCommand, ReorderTilesetDefCommand,
    SetTileCustomDataCommand, UpdateTilesetDefCommand,
};
pub use handler::{IntGridLayerHandler, TilesLayerHandler};
pub use helpers::{
    compute_tileset_grid, generate_unique_tileset_name, mode_availability, tileset_usage_counts,
    GridMetrics, ModeEntry,
};
pub use marking_commands::{
    compute_tile_count, AddTilesetTagCommand, PaintTileTagsCommand, PruneTilesetMarkingsCommand,
    RemoveTilesetTagCommand, RenameTilesetTagCommand, SetMarkingSourceCommand,
    SetTileMarkingCommand, SetTilesetSourceEnumCommand,
};
pub use paint_commands::{
    FillCommand, FillValue, PaintIntGridCommand, PaintTilesCommand, RectCommand,
};
pub use palette::TilePalettePanel;
pub use tools::{
    EraseTool, EyedropperTool, FillTool, PaintTool, RectTool, SelectTool, TileCanvasTool,
};
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
