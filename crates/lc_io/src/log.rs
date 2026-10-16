//! Rotating file logger utility for Level Craft.
//!
//! Provides a thread-safe rotating file writer that appends to `logs/editor.log`
//! and rotates when the file exceeds a maximum byte limit.
//!
//! Conforms strictly to `lc_io` constraints: no unwraps, no panics, no unchecked indexing.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Configuration for the rotating file logger.
#[derive(Debug, Clone)]
pub struct RotatingLogConfig {
    /// Base log file path (e.g. `logs/editor.log`).
    pub log_path: PathBuf,
    /// Maximum file size in bytes before rotating (default: 5 MB).
    pub max_bytes: u64,
    /// Maximum number of rotated backup files to retain (default: 3).
    pub max_files: usize,
}

impl Default for RotatingLogConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("logs/editor.log"),
            max_bytes: 5 * 1024 * 1024,
            max_files: 3,
        }
    }
}

struct RotatingState {
    file: File,
    current_size: u64,
}

/// Thread-safe rotating file log writer implementing `std::io::Write`.
pub struct RotatingLogWriter {
    config: RotatingLogConfig,
    state: Mutex<Option<RotatingState>>,
}

impl RotatingLogWriter {
    /// Creates a new rotating file log writer with the given configuration.
    pub fn new(config: RotatingLogConfig) -> Self {
        let state = Self::init_file(&config);
        Self {
            config,
            state: Mutex::new(state),
        }
    }

    /// Convenience constructor using default configuration (`logs/editor.log`).
    pub fn with_default_path() -> Self {
        Self::new(RotatingLogConfig::default())
    }

    fn init_file(config: &RotatingLogConfig) -> Option<RotatingState> {
        if let Some(parent) = config.log_path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_path)
            .ok()?;

        let current_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        Some(RotatingState { file, current_size })
    }

    fn rotate_files(&self) {
        if self.config.max_files == 0 {
            let _ = fs::remove_file(&self.config.log_path);
            return;
        }

        // Shift backup files down: e.g. editor.2.log -> editor.3.log (removed), editor.1.log -> editor.2.log
        for i in (1..self.config.max_files).rev() {
            let src = self.rotated_path(i);
            let dst = self.rotated_path(i.saturating_add(1));
            if src.exists() {
                let _ = fs::rename(&src, &dst);
            }
        }

        // Rotate primary file to editor.1.log
        let first_backup = self.rotated_path(1);
        let _ = fs::rename(&self.config.log_path, &first_backup);
    }

    fn rotated_path(&self, index: usize) -> PathBuf {
        let parent = self
            .config
            .log_path
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let stem = self
            .config
            .log_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("editor");
        let ext = self
            .config
            .log_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        parent.join(format!("{stem}.{index}.{ext}"))
    }

    /// Writes a line or byte buffer to the log file, rotating if needed.
    pub fn write_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return Ok(buf.len()),
        };

        if guard.is_none() {
            *guard = Self::init_file(&self.config);
        }

        let current_len = guard.as_ref().map(|s| s.current_size).unwrap_or(0);

        if current_len.saturating_add(buf.len() as u64) > self.config.max_bytes {
            // Close active file handle before rotation to release OS locks, while keeping mutex held
            *guard = None;
            self.rotate_files();
            *guard = Self::init_file(&self.config);
        }

        if let Some(state) = guard.as_mut() {
            state.file.write_all(buf)?;
            state.current_size = state.current_size.saturating_add(buf.len() as u64);
            Ok(buf.len())
        } else {
            Ok(buf.len())
        }
    }
}

impl Write for RotatingLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return Ok(()),
        };
        if let Some(state) = guard.as_mut() {
            state.file.flush()
        } else {
            Ok(())
        }
    }
}

impl Write for &RotatingLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return Ok(()),
        };
        if let Some(state) = guard.as_mut() {
            state.file.flush()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotating_log_writer_basic_and_rotation() {
        let temp_dir = std::env::temp_dir().join(format!("lc_log_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let log_path = temp_dir.join("test.log");
        let config = RotatingLogConfig {
            log_path: log_path.clone(),
            max_bytes: 60,
            max_files: 2,
        };

        let writer = RotatingLogWriter::new(config);
        let line = b"12345678901234567890\n"; // 21 bytes

        assert!(writer.write_bytes(line).is_ok());
        assert!(writer.write_bytes(line).is_ok());
        assert!(writer.write_bytes(line).is_ok());
        assert!(writer.write_bytes(line).is_ok());

        assert!(log_path.exists());
        let rot1 = temp_dir.join("test.1.log");
        assert!(rot1.exists(), "test.1.log should exist after rotation");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
