//! lc_feature_world: World overview tab with level placement, depth selection,
//! linear/grid layouts, and multi-world documents.

#![forbid(unsafe_code)]

pub mod canvas;
pub mod commands;
pub mod helpers;
pub mod layout;
pub mod view;

pub use canvas::{level_screen_rect, render_world_canvas};
pub use commands::{
    DuplicateLevelCommand, SetLevelPositionsCommand, SetWorldLayoutCommand, WorldScopedCommand,
};
pub use view::{LevelDrag, WorldDocumentView, WorldViewState};

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the world document factory with the application registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::World(world_id) => Some(Box::new(WorldDocumentView::new(world_id))),
        _ => None,
    });
}
