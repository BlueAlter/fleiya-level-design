//! Typed I/O errors with path context.

use std::path::PathBuf;

/// Error conditions encountered during filesystem serialization or deserialization.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    /// Standard filesystem I/O error.
    #[error("Filesystem I/O error at {path}: {source}")]
    Fs {
        /// Associated file path.
        path: PathBuf,
        /// Underlying std::io::Error.
        #[source]
        source: std::io::Error,
    },

    /// JSON serialization or deserialization failure.
    #[error("JSON error at {path}: {source}")]
    Json {
        /// Associated file path.
        path: PathBuf,
        /// Underlying serde_json::Error.
        #[source]
        source: serde_json::Error,
    },

    /// The file schema version is newer than this version of Level Craft can handle.
    #[error("Unsupported project version {version} in {path} (maximum supported version: {max_supported})")]
    UnsupportedVersion {
        /// Version found in file.
        version: u32,
        /// Maximum supported version by this binary.
        max_supported: u32,
        /// File path where version check failed.
        path: PathBuf,
    },

    /// Expected file or directory is missing.
    #[error("Missing project component at {path}")]
    MissingComponent {
        /// Missing file path.
        path: PathBuf,
    },

    /// Corrupt file missing critical data.
    #[error("Corrupt file at {path}: {details}")]
    Corrupt {
        /// Diagnostic description of the corruption.
        details: String,
        /// File path.
        path: PathBuf,
    },
}
