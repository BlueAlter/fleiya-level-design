#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! `lc_io`: Multi-file JSON serialization, migrations, and project I/O.
//! The ONLY crate using `std::fs`.

pub mod enum_file;
pub mod error;
pub mod format;
pub mod io;
pub mod log;
pub mod migrate;
pub mod preferences;
pub mod shortcuts;

pub use enum_file::*;

pub use error::IoError;
pub use format::{LevelFile, LevelManifestEntry, ProjectFile, WorldFile, WorldManifestEntry};
pub use io::{load_project, read_bytes, save_project, SUPPORTED_VERSION};
pub use log::{RotatingLogConfig, RotatingLogWriter};
pub use preferences::{
    default_preferences_path, load_preferences, load_preferences_or_default, save_preferences,
    AppPreferences, AppearancePreferences, CanvasPreferences, GeneralPreferences,
    SystemPreferences, ThemePreference, WorkspaceStartupMode, PREFERENCES_VERSION,
};
pub use shortcuts::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutMap};
