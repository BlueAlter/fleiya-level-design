//! Core types, state definitions, constants, and layout helpers for Enum workspace.

use std::collections::HashSet;
use std::time::Instant;

use egui::{vec2, Color32, Frame, Pos2, Rect, RichText, Ui};

use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::id::EnumDefId;
use lc_feature_enums::{find_enum_def_references, EnumReferenceLocation};
use lc_io::enum_file::{EnumConflict, FleiyaEnumFile};

use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::metrics::*;
use crate::shared::i18n::doc;
use crate::shared::ui::context_menu::{show_context_menu, MenuAnchor, MenuItem};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::render_document_page;
pub use crate::shared::ui::markdown_document::DocumentCache;
pub use crate::shared::ui::reorder::{drop_index, reorder_item};
use crate::shared::ui::search_picker::SearchPickerState;
use crate::shared::ui::tile_picker::TilePicker;

/// State for the conflict resolution modal during import.
#[derive(Clone, Debug)]
pub struct ConflictModalState {
    pub file_data: FleiyaEnumFile,
    pub conflicts: Vec<EnumConflict>,
    pub understand_checked: bool,
}

/// Single uniform control height variable matching enum value rows.
pub const CONTROL_HEIGHT: f32 = 32.0;

/// Shared metrics for Enum Definition cards matching Figure 2.
pub const ENUM_CARD_HEIGHT: f32 = 36.0;
pub const ENUM_CARD_GAP: f32 = 5.0;
pub const ENUM_CARD_PADDING_X: f32 = 10.0;
pub const ENUM_CARD_PADDING_Y: f32 = 6.0;
pub const DRAG_HANDLE_WIDTH: f32 = 16.0;
pub const DRAG_HANDLE_GAP: f32 = 12.0;
pub const KEBAB_ICON_WIDTH: f32 = 20.0;
pub const KEBAB_ICON_GAP: f32 = 8.0;
pub const TRASH_ICON_WIDTH: f32 = KEBAB_ICON_WIDTH;
pub const TRASH_ICON_GAP: f32 = KEBAB_ICON_GAP;

/// Sub-tab index of the enum definition form.
pub const ENUM_DEF_SUB_TAB: usize = 0;
/// Sub-tab index of the generic Markdown document page.
pub const DOCUMENT_SUB_TAB: usize = 1;

/// Chrome the enum detail panel draws around its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailSections {
    /// Draw the enum name/description title row above the content.
    pub title: bool,
    /// Draw the pinned footer with ID, usage counts, Save and Revert.
    pub footer: bool,
}

/// Which chrome sections `tab` shows.
pub const fn sections_for(tab: usize) -> DetailSections {
    match tab {
        DOCUMENT_SUB_TAB => DetailSections {
            title: false,
            footer: false,
        },
        _ => DetailSections {
            title: true,
            footer: true,
        },
    }
}

/// State for the card context menu popup (Duplicate / Delete).
#[derive(Clone, Debug)]
pub struct CardContextMenuState {
    pub target_enum_id: EnumDefId,
}

#[derive(Clone, Debug)]
pub enum TagAction {
    Select(crate::features::editor::shell_app::TagId),
    Remove(crate::features::editor::shell_app::TagId),
}

/// Computed geometry of an enum value row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowRects {
    pub row: Rect,
    pub handle: Rect,
    pub bound_tile: Rect,
    pub color: Rect,
    pub input: Rect,
    pub delete_btn: Rect,
}

/// Pure deterministic layout for an enum value row of any width.
pub fn row_layout(rect: Rect) -> RowRects {
    let y = rect.min.y;
    let h = 32.0;
    let handle_w = 20.0;
    let tile_w = 32.0;
    let color_w = 32.0;
    let del_w = 32.0;
    let gap = 8.0;

    let handle = Rect::from_min_size(rect.min, vec2(handle_w, h));
    let bound_tile = Rect::from_min_size(Pos2::new(handle.max.x + gap, y), vec2(tile_w, h));
    let color = Rect::from_min_size(Pos2::new(bound_tile.max.x + gap, y), vec2(color_w, h));
    let delete_btn = Rect::from_min_size(Pos2::new(rect.max.x - del_w, y), vec2(del_w, h));
    let input_min_x = color.max.x + gap;
    let input_max_x = delete_btn.min.x - gap;
    let input = Rect::from_min_max(Pos2::new(input_min_x, y), Pos2::new(input_max_x, y + h));

    RowRects {
        row: rect,
        handle,
        bound_tile,
        color,
        input,
        delete_btn,
    }
}

/// Renders a standardized six-dot drag handle indicator.
pub fn draw_six_dot_handle(painter: &egui::Painter, rect: Rect, hovered: bool, t: &Tokens) {
    let dot_radius = 1.5;
    let dot_color = if hovered {
        t.ink_strong
    } else {
        t.muted.linear_multiply(0.4)
    };
    let cx = rect.center().x;
    let cy = rect.center().y;
    let dx = 3.0;
    let dy = 5.0;

    for col in [-dx, dx] {
        for row in [-dy, 0.0, dy] {
            painter.circle_filled(Pos2::new(cx + col, cy + row), dot_radius, dot_color);
        }
    }
}

/// Target to delete pending confirmation.
#[derive(Clone, Debug)]
pub enum DeleteTarget {
    Enum {
        id: EnumDefId,
        name: String,
        entity_count: usize,
        field_count: usize,
    },
    EnumValue {
        enum_id: EnumDefId,
        val_id: String,
        name: String,
    },
}

/// Transient UI and interaction state for the enum mode view.
#[derive(Clone)]
pub struct EnumViewState {
    pub search_query: String,
    pub active_sub_tab: usize,
    pub tile_picker: TilePicker,
    pub enum_drag: DragListState,
    pub value_drag: DragListState,
    pub pending_delete: Option<DeleteTarget>,
    pub conflict_modal: Option<ConflictModalState>,
    pub error_modal: Option<Vec<String>>,
    pub revert_confirm_open: bool,
    pub pending_tileset_change: Option<Option<lc_core::id::TilesetDefId>>,
    pub color_picker_val_id: Option<String>,
    pub color_picker_dragging: bool,
    pub value_uids: std::collections::HashMap<EnumDefId, Vec<u64>>,
    pub next_uid: u64,
    pub toast: Option<(String, Instant)>,
    pub cached_usage: Option<(EnumDefId, usize, usize, usize)>,
    pub more_menu_open: bool,
    pub card_context_menu: Option<CardContextMenuState>,
    pub tag_inline_new_active: bool,
    pub tag_inline_new_text: String,
    pub tag_editing_id: Option<crate::features::editor::shell_app::TagId>,
    pub tag_editing_text: String,
    pub tag_focus_pending: bool,
    pub tag_recall_picker: SearchPickerState,
    pub previous_selected_enum: Option<EnumDefId>,
    pub doc_cache: DocumentCache,
    pub doc_scroll: f32,
    pub card_rects: Vec<egui::Rect>,
    pub delete_btn_rects: Vec<egui::Rect>,
}

impl EnumViewState {
    pub fn ensure_value_uids(&mut self, enum_id: EnumDefId, count: usize) -> &[u64] {
        let next_uid = &mut self.next_uid;
        let uids = self.value_uids.entry(enum_id).or_default();
        while uids.len() < count {
            *next_uid += 1;
            uids.push(*next_uid);
        }
        if uids.len() > count {
            uids.truncate(count);
        }
        uids
    }
}

impl Default for EnumViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_sub_tab: ENUM_DEF_SUB_TAB,
            tile_picker: TilePicker::new(),
            enum_drag: DragListState::new(),
            value_drag: DragListState::new(),
            pending_delete: None,
            conflict_modal: None,
            error_modal: None,
            revert_confirm_open: false,
            pending_tileset_change: None,
            color_picker_val_id: None,
            color_picker_dragging: false,
            value_uids: std::collections::HashMap::new(),
            next_uid: 1000,
            toast: None,
            cached_usage: None,
            more_menu_open: false,
            card_context_menu: None,
            tag_inline_new_active: false,
            tag_inline_new_text: String::new(),
            tag_editing_id: None,
            tag_editing_text: String::new(),
            tag_focus_pending: false,
            tag_recall_picker: SearchPickerState::default(),
            previous_selected_enum: None,
            doc_cache: DocumentCache::default(),
            doc_scroll: 0.0,
            card_rects: Vec::new(),
            delete_btn_rects: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumListAction {
    Duplicate(EnumDefId),
    Delete(EnumDefId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoreMenuAction {
    Import,
    ExportAll,
}

pub fn enum_card_menu_items(enum_id: EnumDefId) -> [MenuItem<EnumListAction>; 2] {
    [
        MenuItem::action(EnumListAction::Duplicate(enum_id), "Duplicate", Icon::Copy),
        MenuItem::destructive(EnumListAction::Delete(enum_id), "Delete", Icon::Delete),
    ]
}

pub fn draw_enum_menu(
    ui: &mut egui::Ui,
    enum_id: EnumDefId,
    action_out: &mut Option<EnumListAction>,
) {
    let items = enum_card_menu_items(enum_id);
    if let Some(act) = show_context_menu(ui, MenuAnchor::inside_ui(), &items) {
        *action_out = Some(act);
    }
}

/// Aligns shadow drafts so Save / Revert returns to clean.
pub fn sync_enum_draft_shadows(app: &mut ShellApp) {
    for draft_enum in &app.enum_draft {
        if let Some(shadow) = app.enum_drafts.get_mut(&draft_enum.id) {
            shadow.def = draft_enum.clone();
        }
    }
}

pub fn render_enum_document(ui: &mut Ui, t: &Tokens, cache: &DocumentCache, scroll: &mut f32) {
    render_document_page(
        ui,
        t,
        cache,
        scroll,
        "enum_document_scroll",
        doc("enum", "overview"),
    );
}

pub fn enum_sub_tabs(ui: &mut Ui, t: &Tokens, active: &mut usize, labels: &[&str]) -> bool {
    let mut changed = false;
    Frame::new().fill(Color32::TRANSPARENT).show(ui, |ui| {
        ui.set_min_height(CONTROL_HEIGHT_COMPACT);
        ui.set_max_height(CONTROL_HEIGHT_COMPACT);
        changed = crate::shared::ui::sub_tabs::sub_tabs(ui, t, active, labels);
    });
    changed
}

pub fn card_label(ui: &mut Ui, text: &str, t: &Tokens) {
    ui.add_sized(
        [100.0, CONTROL_HEIGHT_FORM],
        egui::Label::new(RichText::new(text).color(t.ink).size(12.0)),
    );
}

/// Computes the number of entity defs and field instances using an enum.
pub fn compute_enum_usage(
    project: &lc_core::project::Project,
    enum_id: EnumDefId,
) -> (usize, usize) {
    let refs = find_enum_def_references(project, enum_id);
    let mut entity_defs = HashSet::new();
    let mut field_count = 0;

    for loc in refs {
        match loc {
            EnumReferenceLocation::EntityField { entity_id, .. } => {
                entity_defs.insert(entity_id);
                field_count += 1;
            }
            EnumReferenceLocation::EntityInstanceValue { .. } => {
                field_count += 1;
            }
        }
    }

    (entity_defs.len(), field_count)
}

#[cfg(test)]
mod sections_tests {
    use super::*;

    #[test]
    fn document_tab_draws_no_definition_chrome() {
        let sections = sections_for(DOCUMENT_SUB_TAB);
        assert!(!sections.title, "DOCUMENT must not draw the title row");
        assert!(!sections.footer, "DOCUMENT must not draw the footer");
    }

    #[test]
    fn definition_tab_draws_title_and_footer() {
        let sections = sections_for(ENUM_DEF_SUB_TAB);
        assert!(sections.title, "ENUM DEF draws the title row");
        assert!(sections.footer, "ENUM DEF draws the footer");
        assert_eq!(sections_for(7), sections);
    }
}
