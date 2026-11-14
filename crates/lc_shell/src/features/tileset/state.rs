//! View state for the Tilesets workspace.

use egui::Vec2;
use lc_core::id::TilesetDefId;
use std::collections::HashMap;

use crate::shared::ui::markdown_document::{new_document_cache, DocumentCache};

/// Stored pan and zoom per tileset for the editing session.
#[derive(Debug, Clone)]
pub struct TilesetPanZoom {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for TilesetPanZoom {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

/// View state for the Tilesets workspace page.
#[derive(Debug, Clone)]
pub struct TilesetViewState {
    /// Active sub-tab (0 = TILESET DEF, 1 = DOCUMENT).
    pub active_sub_tab: usize,
    /// Currently selected tileset definition.
    pub selected_tileset_id: Option<TilesetDefId>,
    /// Currently selected tile index in the atlas (None if no tile is selected).
    pub selected_tile: Option<u32>,
    /// Active tool/mode in the right panel shell (e.g. "None", "Marking", "Animation").
    pub active_mode: String,
    /// Search query string in the left panel list.
    pub search_query: String,
    /// Pinned pan and zoom settings per tileset ID.
    pub pan_zoom: HashMap<TilesetDefId, TilesetPanZoom>,
    /// Tileset ID pending delete confirmation dialog (if referenced).
    pub pending_delete_id: Option<TilesetDefId>,
    /// Document scroll offset for the DOCUMENT tab.
    pub doc_scroll: f32,
    /// Markdown document cache.
    pub doc_cache: DocumentCache,
    /// Error banner or notification message (e.g. file outside project root).
    pub error_message: Option<String>,
}

impl Default for TilesetViewState {
    fn default() -> Self {
        Self {
            active_sub_tab: 0,
            selected_tileset_id: None,
            selected_tile: None,
            active_mode: "None".to_string(),
            search_query: String::new(),
            pan_zoom: HashMap::new(),
            pending_delete_id: None,
            doc_scroll: 0.0,
            doc_cache: new_document_cache(),
            error_message: None,
        }
    }
}

impl TilesetViewState {
    /// Selects a new tileset, resetting tile selection and mode to "None",
    /// while preserving pan and zoom per tileset ID.
    pub fn select_tileset(&mut self, id: TilesetDefId) {
        if self.selected_tileset_id != Some(id) {
            self.selected_tileset_id = Some(id);
            self.selected_tile = None;
            self.active_mode = "None".to_string();
        }
    }

    /// Gets or creates the pan and zoom state for a given tileset.
    pub fn pan_zoom_mut(&mut self, id: TilesetDefId) -> &mut TilesetPanZoom {
        self.pan_zoom.entry(id).or_default()
    }
}
