#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! `lc_io`: Multi-file JSON serialization, migrations, and project I/O.
//! The ONLY crate using `std::fs`.

pub mod error;
pub mod format;
pub mod io;
pub mod log;
pub mod migrate;

pub use error::IoError;
pub use format::{LevelFile, LevelManifestEntry, ProjectFile, WorldFile, WorldManifestEntry};
pub use io::{load_project, save_project, SUPPORTED_VERSION};
pub use log::{RotatingLogConfig, RotatingLogWriter};
