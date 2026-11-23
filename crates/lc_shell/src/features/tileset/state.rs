//! View state for the Tilesets workspace.

use egui::{Pos2, Vec2};
use lc_core::id::TilesetDefId;
use std::collections::HashMap;

use crate::shared::ui::atlas_canvas::AtlasCanvasState;
use crate::shared::ui::markdown_document::{new_document_cache, DocumentCache};
use crate::shared::ui::search_picker::SearchPickerState;

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
#[derive(Clone)]
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
    /// Atlas canvas navigation and selection state per tileset ID.
    pub canvas_states: HashMap<TilesetDefId, AtlasCanvasState>,
    /// Tileset ID pending delete confirmation dialog (if referenced).
    pub pending_delete_id: Option<TilesetDefId>,
    /// Document scroll offset for the DOCUMENT tab.
    pub doc_scroll: f32,
    /// Markdown document cache.
    pub doc_cache: DocumentCache,
    /// Error banner or notification message (e.g. file outside project root).
    pub error_message: Option<String>,
    /// Tags per tileset definition (view state).
    pub tags: HashMap<TilesetDefId, Vec<String>>,
    /// Selected enum for tile marking per tileset definition (view state).
    pub marking_enum: HashMap<TilesetDefId, Option<lc_core::id::EnumDefId>>,
    /// Search picker state for tile marking enum selector.
    pub marking_picker: SearchPickerState,
    /// Search picker state for tag recall dropdown.
    pub tag_recall_picker: SearchPickerState,
    /// Whether inline tag creation input is open.
    pub tag_inline_new_active: bool,
    /// Buffer for new tag input.
    pub tag_inline_new_text: String,
    /// Whether tag input needs immediate focus.
    pub tag_focus_pending: bool,
    /// Tag name being edited inline.
    pub tag_editing_name: Option<String>,
    /// Buffer for tag name editing.
    pub tag_editing_text: String,
    /// Context menu anchor (item ID and pos) if open.
    pub context_menu: Option<(TilesetDefId, Pos2)>,
    /// Whether the definition attributes card is collapsed (default false).
    pub is_attributes_collapsed: bool,
    /// Cached neutral repeating page background texture handle.
    pub page_bg: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for TilesetViewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TilesetViewState")
            .field("active_sub_tab", &self.active_sub_tab)
            .field("selected_tileset_id", &self.selected_tileset_id)
            .field("selected_tile", &self.selected_tile)
            .field("active_mode", &self.active_mode)
            .field("search_query", &self.search_query)
            .field("has_page_bg", &self.page_bg.is_some())
            .finish()
    }
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
            canvas_states: HashMap::new(),
            pending_delete_id: None,
            doc_scroll: 0.0,
            doc_cache: new_document_cache(),
            error_message: None,
            tags: HashMap::new(),
            marking_enum: HashMap::new(),
            marking_picker: SearchPickerState::default(),
            tag_recall_picker: SearchPickerState::default(),
            tag_inline_new_active: false,
            tag_inline_new_text: String::new(),
            tag_focus_pending: false,
            tag_editing_name: None,
            tag_editing_text: String::new(),
            context_menu: None,
            is_attributes_collapsed: false,
            page_bg: None,
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
            self.canvas_state_mut(id).needs_initial_fit = true;
        }
    }

    /// Marks the atlas canvas for a given tileset to re-fit synchronously on next render.
    pub fn reset_atlas_fit(&mut self, id: TilesetDefId) {
        self.canvas_state_mut(id).needs_initial_fit = true;
    }

    /// Sets the active sub-tab, resetting document scroll offset on tab change.
    pub fn set_sub_tab(&mut self, tab: usize) {
        if self.active_sub_tab != tab {
            self.active_sub_tab = tab;
            self.doc_scroll = 0.0;
        }
    }

    /// Gets or creates the pan and zoom state for a given tileset.
    pub fn pan_zoom_mut(&mut self, id: TilesetDefId) -> &mut TilesetPanZoom {
        self.pan_zoom.entry(id).or_default()
    }

    /// Gets or creates the atlas canvas state for a given tileset.
    pub fn canvas_state_mut(&mut self, id: TilesetDefId) -> &mut AtlasCanvasState {
        self.canvas_states.entry(id).or_default()
    }
}
