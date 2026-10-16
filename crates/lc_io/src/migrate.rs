//! Migration framework for evolving project schema versions over time.

use crate::error::IoError;
use std::path::Path;

/// Trait representing an atomic schema migration from one version to the next.
pub trait MigrationStep: Send + Sync {
    /// Source schema version this step migrates from.
    fn source_version(&self) -> u32;

    /// Target schema version this step migrates to.
    fn target_version(&self) -> u32;

    /// Performs the in-place file format migration on disk.
    fn migrate(&self, project_dir: &Path) -> Result<(), IoError>;
}

/// Dispatches sequential migration steps to bring an older project up to date.
pub fn migrate_project(
    project_dir: &Path,
    current_version: u32,
    target_version: u32,
) -> Result<u32, IoError> {
    if current_version == target_version {
        return Ok(current_version);
    }

    if current_version > target_version {
        return Err(IoError::UnsupportedVersion {
            version: current_version,
            max_supported: target_version,
            path: project_dir.to_path_buf(),
        });
    }

    // Step-by-step migration runner (skeleton for future version steps)
    let mut version = current_version;
    while version < target_version {
        // When v2 is introduced, register Step_1_to_2 here
        tracing::info!(
            from = version,
            to = version + 1,
            "Migrating project version"
        );
        version += 1;
    }

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_skeleton_same_version() {
        let temp = std::env::temp_dir().join("migrate_test");
        let result = migrate_project(&temp, 1, 1);
        assert!(result.is_ok());
        if let Ok(v) = result {
            assert_eq!(v, 1);
        }
    }

    #[test]
    fn test_migrate_rejects_newer_version() {
        let temp = std::env::temp_dir().join("migrate_test_future");
        let result = migrate_project(&temp, 999, 1);
        assert!(result.is_err());
    }
}
