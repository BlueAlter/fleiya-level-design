//! lc_shell: Main dock layout, menu bar, status bar, and central document docking.

#![forbid(unsafe_code)]

pub mod app_mode;
pub mod assets;
pub mod console;
pub mod def_windows;
pub mod doc_tab_viewer;
pub mod dock_layout;
pub mod dock_panels;
pub mod dock_style;
pub mod enum_file;
pub mod enum_view;
pub mod file_dialog;
pub mod inspector_panel;
pub mod menu;
pub mod project_panel;
pub mod shell_app;
pub mod status_bar;
pub mod tileset_dialog;
pub mod tileset_panel;
pub mod tool_router;
pub mod ui;
pub mod workspace;

pub use app_mode::AppMode;
pub use assets::ShellAssetService;
pub use console::{
    parse_doc_kind, BoundedRingBuffer, CollapsedLogEntry, ConsoleFilter, ConsolePanel,
    ConsoleSubscriberLayer, LogEntry, LogLevel, SharedRingBuffer,
};
pub use doc_tab_viewer::{DocTabViewer, PlaceholderDocumentView};
pub use dock_layout::{BottomDockState, DockSlotLocation, DockSlotState, ShellDockLayout};
pub use file_dialog::{
    FileDialogHandle, FileDialogOutcome, FileDialogService, NativeFileDialogService,
};
pub use inspector_panel::InspectorPanel;
pub use lc_canvas::theme;
pub use lc_core::command::DocKind;
pub use project_panel::ProjectPanel;
pub use shell_app::{ShellApp, TagDef, TagId};
pub use tileset_dialog::{render_tile_icon, show_tileset_dialog, TilesetDialogState};
pub use tileset_panel::CombinedTilesetPanel;
pub use ui::context_menu::{
    is_menu_open, show_context_menu, show_context_menu_contents, show_context_menu_with_theme,
    ContextMenuTheme, MenuAnchor, MenuItem, MENU_MAX_WIDTH, MENU_MIN_WIDTH,
};
pub use ui::tile_bounds::{TileGrid, TileRect};
pub use ui::tile_picker::{PickerTarget, PickerView, TilePicker, TilePickerEvent, TilesetData};

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
