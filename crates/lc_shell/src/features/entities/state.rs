//! Transient UI state for the Entities Workspace.

use egui::Pos2;
use lc_core::id::EntityDefId;

use crate::shared::ui::markdown_document::{new_document_cache, DocumentCache};
use crate::shared::ui::tile_picker::TilePicker;

/// Which tile slot on an [`EntityDef`] the `TilePicker` popup is targeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTileSlot {
    /// Primary display `tile_rect`.
    MainTile,
    /// Optional UI icon `ui_tile_rect`.
    UiTile,
    /// `default_value` of a `FieldType::Tile` field at `field_index`.
    FieldDefaultTile(usize),
}

/// Transient view state for the Entities Workspace (never persisted to `Project`).
#[derive(Clone)]
pub struct EntityViewState {
    /// Currently selected entity definition in the left list.
    pub selected_entity_id: Option<EntityDefId>,
    /// Left panel search filter (matches name or `#tag`).
    pub search_query: String,
    /// Active detail sub-tab (`0 = ENTITY DEF`, `1 = DOCUMENT`).
    pub active_sub_tab: usize,
    /// Index of the selected field in `EntityDef::fields`.
    pub selected_field_idx: Option<usize>,
    /// Open context menu `(entity_id, popup_pos)`.
    pub context_menu: Option<(EntityDefId, Pos2)>,
    /// Pending delete confirmation target `(id, name, instance_count)`.
    pub pending_delete: Option<(EntityDefId, String, usize)>,
    /// Active tile picker popup, if open.
    pub tile_picker: Option<(EntityDefId, EntityTileSlot, TilePicker)>,
    /// Error banner message, if any.
    pub error_message: Option<String>,
    /// Cached markdown state for the `DOCUMENT` sub-tab.
    pub doc_cache: DocumentCache,
    /// Vertical scroll offset for `DOCUMENT` sub-tab.
    pub doc_scroll: f32,
    /// Collapse state of the unified Entity Settings section.
    pub collapse_settings: bool,
    /// Collapse state of the Entity Values / Fields section.
    pub collapse_fields: bool,
    /// Selection highlight state of the right-hand Live Preview card.
    pub focused_preview: bool,
}

impl Default for EntityViewState {
    fn default() -> Self {
        Self {
            selected_entity_id: None,
            search_query: String::new(),
            active_sub_tab: 0,
            selected_field_idx: None,
            context_menu: None,
            pending_delete: None,
            tile_picker: None,
            error_message: None,
            doc_cache: new_document_cache(),
            doc_scroll: 0.0,
            collapse_settings: false,
            collapse_fields: false,
            focused_preview: false,
        }
    }
}

impl EntityViewState {
    /// Selects an entity definition and resets field selection if it changed.
    pub fn select_entity(&mut self, id: EntityDefId) {
        if self.selected_entity_id != Some(id) {
            self.selected_entity_id = Some(id);
            self.selected_field_idx = None;
        }
    }
}
