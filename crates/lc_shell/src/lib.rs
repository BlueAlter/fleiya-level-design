//! lc_shell: Main dock layout, menu bar, status bar, and central document docking.

#![forbid(unsafe_code)]

pub mod doc_tab_viewer;
pub mod dock_layout;
pub mod shell_app;

pub use doc_tab_viewer::DocTabViewer;
pub use dock_layout::{BottomDockState, DockSlotLocation, DockSlotState, ShellDockLayout};
pub use shell_app::ShellApp;
