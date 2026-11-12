//! lc_shell: main dock layout, menu bar, status bar, and central document docking.
//!
//! Source layout:
//!
//! - [`shared`] — feature-agnostic widgets and services, owned by the
//!   `shared/ui-kit` branch (drag ghost, tile bounds, reorder lists, tag input,
//!   context menus, tile picker, file dialogs, asset service).
//! - [`features`] — one folder per editor domain, each owned by its own
//!   `feature/<name>` branch (editor, entity, layer, tileset, enum, project,
//!   world, settings).

#![forbid(unsafe_code)]

pub mod features;
pub mod shared;

pub use features::editor::app_mode::AppMode;
pub use features::editor::console::{
    parse_doc_kind, BoundedRingBuffer, CollapsedLogEntry, ConsoleFilter, ConsolePanel,
    ConsoleSubscriberLayer, LogEntry, LogLevel, SharedRingBuffer,
};
pub use features::editor::dock_layout::{
    BottomDockState, DockSlotLocation, DockSlotState, ShellDockLayout,
};
pub use features::editor::inspector_panel::InspectorPanel;
pub use features::editor::shell_app::{ShellApp, TagDef, TagId};
pub use features::project::ProjectPanel;
pub use features::tileset::dialog::{render_tile_icon, show_tileset_dialog, TilesetDialogState};
pub use features::tileset::panel::CombinedTilesetPanel;
pub use features::world::doc_tab_viewer::{DocTabViewer, PlaceholderDocumentView};
pub use lc_canvas::theme;
pub use lc_core::command::DocKind;
pub use shared::assets::ShellAssetService;
pub use shared::file_dialog::{
    FileDialogHandle, FileDialogOutcome, FileDialogService, NativeFileDialogService,
};
pub use shared::ui::context_menu::{
    is_menu_open, show_context_menu, show_context_menu_contents, show_context_menu_with_theme,
    ContextMenuTheme, MenuAnchor, MenuItem, MENU_MAX_WIDTH, MENU_MIN_WIDTH,
};
pub use shared::ui::tile_bounds::{TileGrid, TileRect};
pub use shared::ui::tile_picker::{
    PickerTarget, PickerView, TilePicker, TilePickerEvent, TilesetData,
};

use lc_io::RotatingLogWriter;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initializes the global tracing subscriber with the console subscriber layer and rotating file log.
///
/// Returns the shared ring buffer and rotating file log writer handles.
pub fn init_shell_logging() -> (SharedRingBuffer, Arc<RotatingLogWriter>) {
    let buffer = SharedRingBuffer::new(BoundedRingBuffer::DEFAULT_CAPACITY);
    let writer = Arc::new(RotatingLogWriter::with_default_path());
    let layer = ConsoleSubscriberLayer::new(buffer.clone()).with_file_writer(writer.clone());

    let _ = tracing_subscriber::registry().with(layer).try_init();

    (buffer, writer)
}
