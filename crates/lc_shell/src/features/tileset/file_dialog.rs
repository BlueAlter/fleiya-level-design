//! Asynchronous file dialog manager for tileset image picking.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use lc_core::id::TilesetDefId;

/// Messages emitted by background file dialogs for tilesets.
#[derive(Debug)]
pub enum TilesetDialogMessage {
    /// User picked a PNG image to create a new tileset definition.
    NewTileset(PathBuf),
    /// User picked a PNG image to update the source image of an existing tileset.
    BrowseSourceImage(TilesetDefId, PathBuf),
}

/// Manages asynchronous file dialog requests for tileset operations.
pub struct TilesetDialogManager {
    sender: Sender<TilesetDialogMessage>,
    receiver: Mutex<Receiver<TilesetDialogMessage>>,
}

impl Default for TilesetDialogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TilesetDialogManager {
    /// Creates a new dialog manager with internal communication channels.
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// Spawns a background thread to pick a PNG file for a new tileset definition.
    pub fn request_new_tileset(&self) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png"])
                .pick_file()
            {
                let _ = sender.send(TilesetDialogMessage::NewTileset(path));
            }
        });
    }

    /// Spawns a background thread to pick a PNG file for an existing tileset definition.
    pub fn request_browse_image(&self, tileset_id: TilesetDefId) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png"])
                .pick_file()
            {
                let _ = sender.send(TilesetDialogMessage::BrowseSourceImage(tileset_id, path));
            }
        });
    }

    /// Polls for completed dialog actions without blocking.
    pub fn poll(&self) -> Option<TilesetDialogMessage> {
        if let Ok(rx) = self.receiver.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }
}
