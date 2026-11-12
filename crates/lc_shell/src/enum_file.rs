//! Enum file management, background rfd dialogs, and import/export coordination.

pub use lc_io::enum_file::*;

use lc_core::id::EnumDefId;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

/// Result message from a completed asynchronous file dialog.
#[derive(Debug, Clone)]
pub enum EnumFileDialogMessage {
    SaveEnums { path: PathBuf },
    ExportAll { path: PathBuf },
    ExportOne { path: PathBuf, enum_id: EnumDefId },
    Import { path: PathBuf },
}

/// Asynchronous file dialog manager for enum imports and exports.
pub struct EnumFileDialogManager {
    sender: Sender<EnumFileDialogMessage>,
    receiver: Mutex<Receiver<EnumFileDialogMessage>>,
}

impl Default for EnumFileDialogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnumFileDialogManager {
    /// Creates a new manager with internal communication channels.
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// Spawns a background thread to pick a save destination for all enums.
    pub fn request_save_all(&self, default_name: &str) {
        let sender = self.sender.clone();
        let file_name = format!("{}.fleiyaenum", default_name);
        std::thread::spawn(move || {
            if let Some(mut path) = rfd::FileDialog::new()
                .add_filter("Fleiya Enum", &[ENUM_FILE_EXTENSION])
                .set_file_name(&file_name)
                .save_file()
            {
                if !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
                {
                    path.set_extension(ENUM_FILE_EXTENSION);
                }
                let _ = sender.send(EnumFileDialogMessage::SaveEnums { path });
            }
        });
    }

    /// Spawns a background thread to export all enums.
    pub fn request_export_all(&self, default_name: &str) {
        let sender = self.sender.clone();
        let file_name = format!("{}.fleiyaenum", default_name);
        std::thread::spawn(move || {
            if let Some(mut path) = rfd::FileDialog::new()
                .add_filter("Fleiya Enum", &[ENUM_FILE_EXTENSION])
                .set_file_name(&file_name)
                .save_file()
            {
                if !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
                {
                    path.set_extension(ENUM_FILE_EXTENSION);
                }
                let _ = sender.send(EnumFileDialogMessage::ExportAll { path });
            }
        });
    }

    /// Spawns a background thread to export a single enum.
    pub fn request_export_one(&self, identifier: &str, enum_id: EnumDefId) {
        let sender = self.sender.clone();
        let file_name = format!("{}.fleiyaenum", identifier);
        std::thread::spawn(move || {
            if let Some(mut path) = rfd::FileDialog::new()
                .add_filter("Fleiya Enum", &[ENUM_FILE_EXTENSION])
                .set_file_name(&file_name)
                .save_file()
            {
                if !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
                {
                    path.set_extension(ENUM_FILE_EXTENSION);
                }
                let _ = sender.send(EnumFileDialogMessage::ExportOne { path, enum_id });
            }
        });
    }

    /// Spawns a background thread to pick a file to import.
    pub fn request_import(&self) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Fleiya Enum", &[ENUM_FILE_EXTENSION])
                .pick_file()
            {
                let _ = sender.send(EnumFileDialogMessage::Import { path });
            }
        });
    }

    /// Polls for completed dialog actions without blocking.
    pub fn poll(&self) -> Option<EnumFileDialogMessage> {
        if let Ok(rx) = self.receiver.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }
}
