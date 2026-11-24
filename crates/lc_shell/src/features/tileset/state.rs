//! View state for the Tilesets workspace.

use egui::{Pos2, Vec2};
use lc_core::id::TilesetDefId;
use std::collections::HashMap;

use crate::shared::ui::atlas_canvas::AtlasCanvasState;
use crate::shared::ui::custom_data_dialog::CustomDataDialogState;
use crate::shared::ui::markdown_document::{new_document_cache, DocumentCache};
use crate::shared::ui::search_picker::SearchPickerState;

/// Special value ID used when the "Custom data" item is selected in marking mode.
pub const CUSTOM_DATA_VALUE_ID: &str = "__custom_data__";

/// Active tool for editing tile collisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CollisionTool {
    #[default]
    Select,
    Box,
    Polygon,
}

/// Grid snapping mode for collision editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CollisionSnapMode {
    #[default]
    Pixel,
    TileSize,
    None,
}

impl CollisionSnapMode {
    /// Cycles to the next snap mode (Pixel -> TileSize -> None -> Pixel).
    pub fn cycle(&mut self) {
        *self = match self {
            Self::Pixel => Self::TileSize,
            Self::TileSize => Self::None,
            Self::None => Self::Pixel,
        };
    }

    /// Label to display on the toolbar button.
    pub fn label(self) -> &'static str {
        match self {
            Self::Pixel => "Pixel",
            Self::TileSize => "Tile",
            Self::None => "Off",
        }
    }
}

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
    /// Currently selected enum value ID for tile marking mode.
    pub selected_marking_value: Option<String>,
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
    /// Image path pending confirmation modal to copy into project assets.
    pub pending_outside_copy: Option<std::path::PathBuf>,
    /// Cached neutral repeating page background texture handle.
    pub page_bg: Option<egui::TextureHandle>,
    /// Modal dialog state for editing tile custom data.
    pub custom_data_dialog: CustomDataDialogState,
    /// Whether the enum for tile marking dropdown (or tool mode dropdown) is currently open.
    pub is_enum_dropdown_open: bool,
    /// Currently selected animation anchor tile ID in Animation tool mode.
    pub selected_animation_anchor: Option<u32>,
    /// Whether animations are actively playing in the viewport/preview.
    pub is_animation_playing: bool,
    /// Whether the floating bottom-left animation preview panel is visible.
    pub show_animation_preview: bool,
    /// Timestamp when animation playback started.
    pub animation_start_time: std::time::Instant,
    /// Active tool in Collision mode (Select, Box, Polygon).
    pub collision_tool: CollisionTool,
    /// Snapping mode in Collision mode (Pixel, Tile, Off).
    pub collision_snap: CollisionSnapMode,
    /// Currently isolated tile ID in Collision focus mode (None if normal atlas view).
    pub collision_focus_tile: Option<u32>,
    /// Currently selected collision shape ID within the active tile (if any).
    pub selected_collision_shape_id: Option<u32>,
    /// Clipboard for copying and pasting tile collision shapes.
    pub collision_clipboard: Option<lc_core::collision::TileCollisionDef>,
    /// Whether collision shapes are rendered as overlay across the atlas canvas.
    pub show_collision_overlay: bool,
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
            selected_marking_value: None,
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
            pending_outside_copy: None,
            page_bg: None,
            custom_data_dialog: CustomDataDialogState::default(),
            is_enum_dropdown_open: false,
            selected_animation_anchor: None,
            is_animation_playing: true,
            show_animation_preview: true,
            animation_start_time: std::time::Instant::now(),
            collision_tool: CollisionTool::Select,
            collision_snap: CollisionSnapMode::Pixel,
            collision_focus_tile: None,
            selected_collision_shape_id: None,
            collision_clipboard: None,
            show_collision_overlay: true,
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
            self.collision_focus_tile = None;
            self.selected_collision_shape_id = None;
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

    /// Returns the active tile frame for an anchor tile at the current elapsed time.
    pub fn active_animation_frame(
        &self,
        tileset: &lc_core::defs::TilesetDef,
        anchor_id: u32,
    ) -> u32 {
        if let Some(anim) = tileset.animations.get(&anchor_id) {
            if !self.is_animation_playing {
                return anim.frames.first().copied().unwrap_or(anchor_id);
            }
            let elapsed_ms = self.animation_start_time.elapsed().as_millis() as u64;
            anim.active_frame(elapsed_ms)
        } else {
            anchor_id
        }
    }
}
