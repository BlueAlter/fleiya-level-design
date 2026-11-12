//! Enum Definitions full-window mode view, detail editor, and modal dialogs.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use egui::{
    vec2, Align, Align2, Area, Color32, CornerRadius, CursorIcon, Frame, Id, Key, Layout, Margin,
    Order, Panel, PointerButton, Popup, Pos2, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Ui,
};

use lc_canvas::grid::draw_checkerboard;
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;
use lc_core::layer::TileRef;
use lc_feature_enums::{find_enum_def_references, EnumReferenceLocation};
use lc_ui_api::AssetService;

use crate::enum_file::{
    compute_lost_keys, convert_file_enum_to_def, convert_project_enums_to_file,
    convert_single_enum_to_file, generate_unique_identifier, read_enum_file,
    validate_enum_file_data, write_enum_file_atomic, ConflictResolutionChoice, EnumConflict,
    EnumFileDialogManager, EnumFileDialogMessage, FleiyaEnumFile, ENUM_FILE_EXTENSION,
};
use crate::shell_app::ShellApp;
use crate::ui::context_menu::{
    is_menu_open, show_context_menu, ContextMenuTheme, MenuAnchor, MenuItem,
};
use crate::ui::drag_list::{paint_placeholder, DragListState, DragOutcome, DragTheme};
pub use crate::ui::reorder::{drop_index, reorder_item};
use crate::ui::reorder::Slot;
use crate::ui::tile_bounds::{TileGrid, TileRect};
use crate::ui::tile_picker::{PickerTarget, TilePicker, TilePickerEvent, TilesetData};

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

/// State for the card context menu popup (Duplicate / Delete).
#[derive(Clone, Debug)]
pub struct CardContextMenuState {
    pub target_enum_id: EnumDefId,
    pub pos: Pos2,
}

/// Action to perform on project tags.
#[derive(Clone, Debug)]
pub enum TagAction {
    Remove(crate::shell_app::TagId),
    Rename(crate::shell_app::TagId, String),
    CreateAndAssign(String),
    Assign(crate::shell_app::TagId),
}

/// Pure layout geometry for an enum value row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowRects {
    pub row: Rect,
    pub handle: Rect,
    pub bound_tile: Rect,
    pub color: Rect,
    pub input: Rect,
    pub delete_btn: Rect,
}

/// Pure deterministic layout function for value rows.
pub fn row_layout(rect: Rect) -> RowRects {
    let y = rect.min.y;
    let h = 32.0;
    let gap = 8.0;

    let handle_w = 20.0;
    let square_size = 32.0;

    let handle = Rect::from_min_size(rect.min, vec2(handle_w, h));
    let mut x = rect.min.x + handle_w + gap;

    let bound_tile = Rect::from_min_size(Pos2::new(x, y), vec2(square_size, square_size));
    x += square_size + gap;

    let color = Rect::from_min_size(Pos2::new(x, y), vec2(square_size, square_size));
    x += square_size + gap;

    let del_x = rect.max.x - square_size;
    let delete_btn = Rect::from_min_size(Pos2::new(del_x, y), vec2(square_size, square_size));

    let input_min_x = x;
    let input_max_x = (del_x - gap).max(input_min_x);
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

/// Draws a 6-dot drag handle (2 columns x 3 rows) using the Painter.
pub fn draw_six_dot_handle(painter: &egui::Painter, rect: Rect, hovered: bool, t: &Tokens) {
    let center = rect.center();
    let dot_color = if hovered { t.ink_strong } else { t.muted };
    let radius = 1.5;
    let col_xs = [center.x - 2.5, center.x + 2.5];
    let row_ys = [center.y - 5.0, center.y, center.y + 5.0];

    for &x in &col_xs {
        for &y in &row_ys {
            painter.circle_filled(Pos2::new(x, y), radius, dot_color);
        }
    }
}

/// Target item for pending deletion confirmation.
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

/// State for the conflict resolution modal during import.
#[derive(Clone, Debug)]
pub struct ConflictModalState {
    pub file_data: FleiyaEnumFile,
    pub conflicts: Vec<EnumConflict>,
    pub understand_checked: bool,
}

/// Transient UI and interaction state for the enum mode view.
#[derive(Clone)]
pub struct EnumViewState {
    pub search_query: String,
    pub active_sub_tab: usize,
    pub tile_picker: TilePicker,
    /// Drag state of the enum definition list (live ghost drop slot).
    pub enum_drag: DragListState,
    /// Drag state of the enum value rows (live ghost drop slot).
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
    pub cached_usage: Option<(EnumDefId, usize, usize, usize)>, // (enum_id, val_count, entity_count, field_count)
    pub more_menu_open: bool,
    pub card_context_menu: Option<CardContextMenuState>,
    pub tag_inline_new_active: bool,
    pub tag_inline_new_text: String,
    pub tag_editing_id: Option<crate::shell_app::TagId>,
    pub tag_editing_text: String,
    pub tag_recall_open: bool,
    pub tag_recall_search: String,
    pub tag_recall_selected_idx: usize,
    pub tag_recall_was_open: bool,
    pub previous_selected_enum: Option<EnumDefId>,
    pub card_rects: Vec<egui::Rect>,
    pub delete_btn_rects: Vec<egui::Rect>,
}

impl EnumViewState {
    pub fn ensure_value_uids(&mut self, enum_id: EnumDefId, count: usize) -> &[u64] {
        let uids = self.value_uids.entry(enum_id).or_default();
        while uids.len() < count {
            self.next_uid += 1;
            uids.push(self.next_uid);
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
            active_sub_tab: 0,
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
            tag_recall_open: false,
            tag_recall_search: String::new(),
            tag_recall_selected_idx: 0,
            tag_recall_was_open: false,
            previous_selected_enum: None,
            card_rects: Vec::new(),
            delete_btn_rects: Vec::new(),
        }
    }
}

/// Action produced by the enum card context menu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumListAction {
    Duplicate(EnumDefId),
    Delete(EnumDefId),
}

/// Action produced by the top left ⋮ menu next to "+ New Enum".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoreMenuAction {
    Import,
    ExportAll,
}

/// Shared item list for enum card context menus.
pub fn enum_card_menu_items(enum_id: EnumDefId) -> [MenuItem<EnumListAction>; 2] {
    [
        MenuItem::action(EnumListAction::Duplicate(enum_id), "Duplicate", Icon::Copy),
        MenuItem::destructive(EnumListAction::Delete(enum_id), "Delete", Icon::Delete),
    ]
}

/// Shared context menu for enum cards in the left panel (used by right-click and ⋮ kebab button).
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

/// Renders the complete Enum Mode view filling the window body below the toolbar.
pub fn render_enum_view(app: &mut ShellApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let state_id = Id::new("enum_mode_view_transient_state");
    let mut state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<EnumViewState>(state_id).unwrap_or_default());

    let dialog_mgr = Id::new("enum_file_dialog_mgr");
    let dialog_manager = ui.ctx().data_mut(|d| {
        d.get_temp::<std::sync::Arc<EnumFileDialogManager>>(dialog_mgr)
            .unwrap_or_else(|| {
                let m = std::sync::Arc::new(EnumFileDialogManager::new());
                d.insert_temp(dialog_mgr, m.clone());
                m
            })
    });

    // 0. Poll background file dialog results
    if let Some(msg) = dialog_manager.poll() {
        handle_file_dialog_message(app, &mut state, msg);
    }

    // Ensure a valid enum is selected from draft
    let enums = &app.enum_draft;
    if app.selected_enum_def.is_none() || !enums.iter().any(|e| Some(e.id) == app.selected_enum_def)
    {
        app.selected_enum_def = enums
            .iter()
            .find(|e| e.name == "ItemRarity")
            .map(|e| e.id)
            .or_else(|| enums.first().map(|e| e.id));
    }

    let mut pending_delete_execute: Option<DeleteTarget> = None;

    // Reinforce text selection disabling on all styles
    ui.ctx().all_styles_mut(|s| {
        s.interaction.selectable_labels = false;
        s.interaction.multi_widget_text_select = false;
    });

    // =========================================================================
    // 1. LEFT PANEL: Enum cards list & quick actions (Fixed width: 260px, padding: 12px)
    // =========================================================================
    Panel::left("enum_mode_left_panel")
        .exact_size(260.0)
        .resizable(false)
        .frame(Frame::new().fill(t.panel).inner_margin(Margin::same(12)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().min(236.0));
            ui.set_max_width(236.0);

            // 1. Top action row: "+ New Enum" (stretches) + square 1:1 "..." button
            let btn_h = 28.0;
            let gap = 6.0;
            let dots_btn_size = btn_h;
            let new_btn_w = (ui.available_width() - dots_btn_size - gap).max(1.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;

                let new_btn = ui.add_sized(
                    [new_btn_w, btn_h],
                    egui::Button::new(
                        RichText::new("+ New Enum")
                            .size(12.0)
                            .strong()
                            .color(t.invert_fg),
                    )
                    .fill(t.accent)
                    .corner_radius(CornerRadius::same(3)),
                );
                if new_btn.clicked() {
                    let next_id =
                        EnumDefId(app.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1);
                    let name = format!("Enum_{}", next_id.0);
                    let def = EnumDef {
                        id: next_id,
                        name,
                        tileset_id: None,
                        tags: Vec::new(),
                        values: vec![
                            EnumValueDef {
                                id: "Common".to_string(),
                                name: "Common".to_string(),
                                color: [156, 163, 175, 255],
                                tile: None,
                            },
                            EnumValueDef {
                                id: "Uncommon".to_string(),
                                name: "Uncommon".to_string(),
                                color: [61, 214, 184, 255],
                                tile: None,
                            },
                        ],
                    };
                    state.ensure_value_uids(next_id, 2);
                    app.enum_drafts
                        .insert(next_id, crate::shell_app::EnumDraft::new(def.clone()));
                    app.enum_draft.push(def);
                    app.selected_enum_def = Some(next_id);
                    app.enum_draft_revision += 1;
                }

                // Three-dot 1:1 square button
                let dots_resp = ui
                    .add_sized(
                        [dots_btn_size, btn_h],
                        egui::Button::new(Icon::DotsThreeVertical.rich(
                            ui.ctx(),
                            14.0,
                            t.ink_strong,
                        ))
                        .fill(t.sunken)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::same(3)),
                    )
                    .on_hover_text("More actions (Import / Export All)");

                let more_menu_items = [
                    MenuItem::action(MoreMenuAction::Import, "Import...", Icon::Import),
                    MenuItem::action(MoreMenuAction::ExportAll, "Export All...", Icon::Export),
                ];

                if let Some(act) = show_context_menu(
                    ui,
                    MenuAnchor::button_with_id(&dots_resp, Id::new("enum_left_more_menu_popup")),
                    &more_menu_items,
                ) {
                    match act {
                        MoreMenuAction::Import => dialog_manager.request_import(),
                        MoreMenuAction::ExportAll => {
                            dialog_manager.request_export_all(&app.project.name);
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Search box (Section C & Item 7: centered text, 8px margin, 28px height)
            ui.add_sized(
                [ui.available_width(), 28.0],
                TextEdit::singleline(&mut state.search_query)
                    .id(Id::new("enum_search_query"))
                    .hint_text("Search enums...")
                    .vertical_align(Align::Center)
                    .margin(Margin::symmetric(8, 0)),
            );

            ui.add_space(12.0);

            // 3. Header: ENUM DEFINITIONS (N)
            let draft_enums = &app.enum_draft;
            ui.label(
                RichText::new(format!("ENUM DEFINITIONS ({})", draft_enums.len()))
                    .size(11.0)
                    .strong()
                    .color(t.accent_ink),
            );

            ui.add_space(10.0);

            // 4. Cards list
            let query = state.search_query.trim().to_lowercase();
            let filtered_indices: Vec<usize> = draft_enums
                .iter()
                .enumerate()
                .filter(|(_, e)| query.is_empty() || e.name.to_lowercase().contains(&query))
                .map(|(idx, _)| idx)
                .collect();

            let mut card_rects = Vec::with_capacity(filtered_indices.len());
            let mut delete_btn_rects = Vec::with_capacity(filtered_indices.len());
            let drag_theme = DragTheme::from_accent(t.accent);
            let mut pending_list_action: Option<EnumListAction> = None;
            let selected_changed = state.previous_selected_enum != app.selected_enum_def;

            ui.style_mut().spacing.scroll = egui::style::ScrollStyle::thin();

            ScrollArea::vertical()
                .id_salt("enum_cards_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.set_max_width(ui.available_width());
                    if filtered_indices.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(RichText::new("No matching enums").color(t.muted));
                        });
                    } else {
                        for slot in state.enum_drag.layout(filtered_indices.len()) {
                            let order_idx = match slot {
                                Slot::Item(i) => i,
                                Slot::Placeholder => {
                                    // Ghost drop slot: marks where the dragged card will land
                                    let ph_h = state
                                        .enum_drag
                                        .row_size()
                                        .map_or(ENUM_CARD_HEIGHT, |size| size.y);
                                    let (ph_rect, _) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), ph_h),
                                        Sense::hover(),
                                    );
                                    paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                                    card_rects.push(ph_rect);
                                    delete_btn_rects.push(ph_rect);
                                    ui.add_space(ENUM_CARD_GAP);
                                    continue;
                                }
                            };
                            let Some(&real_idx) = filtered_indices.get(order_idx) else {
                                continue;
                            };
                            let e = match draft_enums.get(real_idx) {
                                Some(e) => e,
                                None => continue,
                            };
                            let is_selected = app.selected_enum_def == Some(e.id);
                            let card_w = ui.available_width();
                            let content_h = ENUM_CARD_HEIGHT - 2.0 * ENUM_CARD_PADDING_Y - 2.0;

                            let (row_rect, row_resp) = ui.allocate_exact_size(
                                vec2(card_w, ENUM_CARD_HEIGHT),
                                Sense::click(),
                            );
                            card_rects.push(row_rect);

                            let del_btn_rect = Rect::from_min_size(
                                Pos2::new(
                                    row_rect.max.x - ENUM_CARD_PADDING_X - 1.0 - KEBAB_ICON_WIDTH,
                                    row_rect.min.y + ENUM_CARD_PADDING_Y + 1.0,
                                ),
                                vec2(KEBAB_ICON_WIDTH, content_h),
                            );
                            delete_btn_rects.push(del_btn_rect);

                            let row_id = Id::new(("enum_row", e.id));
                            let kebab_id = row_id.with("kebab");
                            let kebab_popup_id = row_id.with("kebab_popup");
                            let context_popup_id = row_id.with("context_popup");

                            let is_kebab_open = is_menu_open(ui.ctx(), kebab_popup_id);
                            let is_context_open = is_menu_open(ui.ctx(), context_popup_id);
                            let is_menu_open = is_kebab_open || is_context_open;

                            let card_fill = if is_selected {
                                t.sunken
                            } else if is_menu_open {
                                t.sunken.linear_multiply(0.8)
                            } else {
                                t.panel
                            };

                            let stroke_color = if is_selected {
                                t.accent
                            } else if is_menu_open {
                                t.accent.linear_multiply(0.6)
                            } else if row_resp.hovered() {
                                t.muted.linear_multiply(0.5)
                            } else {
                                t.hairline().color
                            };
                            let card_stroke = Stroke::new(1.0, stroke_color);

                            // 1. Draw card background & border
                            ui.painter().rect(
                                row_rect,
                                CornerRadius::same(4),
                                card_fill,
                                card_stroke,
                                StrokeKind::Inside,
                            );

                            // 2. Drag handle on left: Sense::drag() only on the handle!
                            let handle_rect = Rect::from_min_size(
                                Pos2::new(
                                    row_rect.min.x + ENUM_CARD_PADDING_X + 1.0,
                                    row_rect.min.y + ENUM_CARD_PADDING_Y + 1.0,
                                ),
                                vec2(DRAG_HANDLE_WIDTH, content_h),
                            );
                            let handle_resp =
                                ui.interact(handle_rect, row_id.with("handle"), Sense::drag());
                            if handle_resp.hovered() {
                                ui.ctx().set_cursor_icon(CursorIcon::Grab);
                            }
                            if handle_resp.drag_started() {
                                state.enum_drag.begin(ui.ctx(), order_idx, row_rect);
                                ui.memory_mut(|m| {
                                    if let Some(fid) = m.focused() {
                                        m.surrender_focus(fid);
                                    }
                                    m.stop_text_input();
                                });
                            }
                            let handle_painter = ui.painter().with_clip_rect(handle_rect);
                            draw_six_dot_handle(
                                &handle_painter,
                                handle_rect,
                                handle_resp.hovered(),
                                &t,
                            );

                            // 3. Truncated label for enum name: left-aligned, vertically centered
                            let text_start_x = handle_rect.max.x + DRAG_HANDLE_GAP;
                            let text_end_x = del_btn_rect.min.x - KEBAB_ICON_GAP;
                            let text_rect = Rect::from_min_max(
                                Pos2::new(text_start_x, row_rect.min.y + ENUM_CARD_PADDING_Y + 1.0),
                                Pos2::new(
                                    text_end_x.max(text_start_x + 10.0),
                                    row_rect.max.y - ENUM_CARD_PADDING_Y - 1.0,
                                ),
                            );
                            ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(text_rect)
                                    .layout(Layout::left_to_right(Align::Center)),
                            )
                            .add(
                                egui::Label::new(
                                    RichText::new(&e.name).strong().color(if is_selected {
                                        t.ink_strong
                                    } else {
                                        t.ink
                                    }),
                                )
                                .truncate()
                                .selectable(false),
                            );

                            // 4. Kebab button (⋮): separate widget allocated AFTER row and on top of it
                            let kebab_btn = ui
                                .interact(del_btn_rect, kebab_id, Sense::click())
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .on_hover_text("Enum actions");

                            let kebab_color = if kebab_btn.hovered() || is_menu_open {
                                t.ink_strong
                            } else {
                                t.muted
                            };
                            lc_canvas::icons::paint_icon(
                                ui.painter(),
                                del_btn_rect.center(),
                                Icon::DotsThreeVertical,
                                13.0,
                                kebab_color,
                            );

                            let card_items = enum_card_menu_items(e.id);

                            // Kebab button menu with toggle behavior (right-aligned to button)
                            if let Some(act) = show_context_menu(
                                ui,
                                MenuAnchor::button_with_id(&kebab_btn, kebab_popup_id),
                                &card_items,
                            ) {
                                pending_list_action = Some(act);
                            }

                            // Main row context menu (right click anywhere on row at pointer position)
                            if let Some(act) = show_context_menu(
                                ui,
                                MenuAnchor::context_menu_with_id(&row_resp, context_popup_id),
                                &card_items,
                            ) {
                                pending_list_action = Some(act);
                            }

                            // Left click selects row (unless kebab was clicked or dragging started)
                            if row_resp.clicked() && !kebab_btn.clicked() && !handle_resp.dragged()
                            {
                                app.selected_enum_def = Some(e.id);
                            }

                            // When active enum changes (not every frame), scroll_to_me
                            if selected_changed && is_selected {
                                row_resp.scroll_to_me(Some(Align::Center));
                            }

                            ui.add_space(ENUM_CARD_GAP);
                        }

                        state.card_rects = card_rects.clone();
                        state.delete_btn_rects = delete_btn_rects.clone();

                        // Updates the drop target, Grabbing cursor and auto-scroll
                        state.enum_drag.update(ui, &card_rects, &drag_theme);
                    }
                });

            // Preview for dragged enum card on Order::Tooltip at exact original size
            if let Some(drag_idx) = state.enum_drag.dragging_index() {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                if let Some(&real_idx) = filtered_indices.get(drag_idx) {
                    if let Some(dragged_enum) = draft_enums.get(real_idx) {
                        if let Some(pos) =
                            ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()))
                        {
                            let ghost_pos = pos - state.enum_drag.grab_offset();
                            let card_w = if !card_rects.is_empty() {
                                card_rects[0].width()
                            } else {
                                236.0
                            };
                            Area::new(Id::new("enum_card_drag_preview"))
                                .order(Order::Tooltip)
                                .fixed_pos(ghost_pos)
                                .interactable(false)
                                .show(ui.ctx(), |ui| {
                                    ui.set_min_size(vec2(card_w, ENUM_CARD_HEIGHT));
                                    ui.set_max_size(vec2(card_w, ENUM_CARD_HEIGHT));
                                    let preview_rect = Rect::from_min_size(
                                        ui.cursor().min,
                                        vec2(card_w, ENUM_CARD_HEIGHT),
                                    );
                                    ui.allocate_rect(preview_rect, Sense::hover());
                                    // Active style: cyan stroke, slightly lighter fill
                                    ui.painter().rect_filled(
                                        preview_rect,
                                        CornerRadius::same(4),
                                        t.sunken,
                                    );
                                    ui.painter().rect_stroke(
                                        preview_rect,
                                        CornerRadius::same(4),
                                        Stroke::new(1.5, t.accent),
                                        StrokeKind::Inside,
                                    );

                                    let content_h =
                                        ENUM_CARD_HEIGHT - 2.0 * ENUM_CARD_PADDING_Y - 2.0;
                                    let cy = preview_rect.center().y;

                                    // Six-dot drag handle
                                    let hr = Rect::from_min_size(
                                        Pos2::new(
                                            preview_rect.min.x + ENUM_CARD_PADDING_X + 1.0,
                                            cy - content_h / 2.0,
                                        ),
                                        vec2(DRAG_HANDLE_WIDTH, content_h),
                                    );
                                    draw_six_dot_handle(ui.painter(), hr, true, &t);

                                    // Truncated label for enum name
                                    let text_start_x = hr.max.x + DRAG_HANDLE_GAP;
                                    let text_end_x = preview_rect.max.x
                                        - ENUM_CARD_PADDING_X
                                        - 1.0
                                        - KEBAB_ICON_WIDTH
                                        - KEBAB_ICON_GAP;
                                    let text_w = (text_end_x - text_start_x).max(10.0);
                                    let name_rect = Rect::from_min_size(
                                        Pos2::new(text_start_x, cy - content_h / 2.0),
                                        vec2(text_w, content_h),
                                    );
                                    ui.new_child(
                                        egui::UiBuilder::new()
                                            .max_rect(name_rect)
                                            .layout(Layout::left_to_right(Align::Center)),
                                    )
                                    .add(
                                        egui::Label::new(
                                            RichText::new(&dragged_enum.name)
                                                .strong()
                                                .color(t.ink_strong),
                                        )
                                        .truncate(),
                                    );

                                    // Kebab icon on right
                                    let kebab_center = Pos2::new(
                                        preview_rect.max.x
                                            - ENUM_CARD_PADDING_X
                                            - 1.0
                                            - KEBAB_ICON_WIDTH / 2.0,
                                        cy,
                                    );
                                    lc_canvas::icons::paint_icon(
                                        ui.painter(),
                                        kebab_center,
                                        Icon::DotsThreeVertical,
                                        13.0,
                                        t.muted,
                                    );
                                });
                        }
                    }
                }
            }

            // Resolve the card drag: drop on release, cancel on Escape
            if let DragOutcome::Dropped { from, to } = state.enum_drag.finish(ui) {
                if from < filtered_indices.len() && to <= filtered_indices.len() {
                    let from_real = filtered_indices[from];
                    let to_real = if to < filtered_indices.len() {
                        filtered_indices[to]
                    } else {
                        app.enum_draft.len()
                    };
                    if reorder_item(&mut app.enum_draft, from_real, to_real) {
                        app.enum_draft_revision += 1;
                    }
                }
            }

            // Apply pending list action (Duplicate / Delete) after drawing loop
            if let Some(action) = pending_list_action {
                match action {
                    EnumListAction::Duplicate(id) => {
                        if let Some(new_id) = app.duplicate_enum(id) {
                            app.selected_enum_def = Some(new_id);
                        }
                    }
                    EnumListAction::Delete(id) => {
                        let (entity_count, field_count) = compute_enum_usage(&app.project, id);
                        let target_name = app
                            .enum_draft
                            .iter()
                            .find(|e| e.id == id)
                            .map(|e| e.name.clone())
                            .unwrap_or_default();
                        state.pending_delete = Some(DeleteTarget::Enum {
                            id,
                            name: target_name,
                            entity_count,
                            field_count,
                        });
                    }
                }
            }
        });

    // =========================================================================
    // 2. DETAIL PANEL: Pinned footer via TopBottomPanel::bottom + CentralPanel
    // =========================================================================
    let selected_enum_info = app
        .selected_enum_def
        .and_then(|id| app.enum_draft.iter().find(|e| e.id == id))
        .map(|e| (e.id, e.name.clone(), e.values.len()));

    let (entity_count, field_count) = if let Some(ref info) = selected_enum_info {
        match state.cached_usage {
            Some((cid, vcnt, ec, fc)) if cid == info.0 && vcnt == info.2 => (ec, fc),
            _ => {
                let (ec, fc) = compute_enum_usage(&app.project, info.0);
                state.cached_usage = Some((info.0, info.2, ec, fc));
                (ec, fc)
            }
        }
    } else {
        (0, 0)
    };

    // A. Footer docked at the bottom of the detail area (declared BEFORE CentralPanel)
    Panel::bottom("enum_mode_detail_footer")
        .exact_size(48.0)
        .frame(Frame::new().fill(t.panel))
        .show_separator_line(true)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(20.0);
                if let Some(ref e) = selected_enum_info {
                    let mut info = format!(
                        "ID: EnumDefId({}) • Used by {} Entity Def(s) & {} Field(s)",
                        e.0 .0, entity_count, field_count
                    );
                    if app.enum_is_dirty() {
                        info.push_str(" • Unsaved changes");
                    }
                    ui.label(RichText::new(info).size(11.5).color(t.muted));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(12.0);

                    let is_dirty = app.enum_is_dirty();

                    // "Save Enum Def" button (solid blue accent, Title Case)
                    let save_btn = ui.add_enabled(
                        is_dirty,
                        egui::Button::new(
                            RichText::new("Save Enum Def")
                                .size(12.0)
                                .strong()
                                .color(if is_dirty { t.invert_fg } else { t.muted }),
                        )
                        .fill(if is_dirty { t.accent } else { t.sunken })
                        .corner_radius(CornerRadius::same(3))
                        .min_size(vec2(110.0, 26.0)),
                    );
                    if save_btn.clicked() {
                        if let Some(open_id) = app.selected_enum_def {
                            if let Some(open_def) = app.enum_draft.iter().find(|e| e.id == open_id).cloned() {
                                if let Some(pos) = app.project.defs.enums.iter().position(|e| e.id == open_id) {
                                    app.project.defs.enums[pos] = open_def.clone();
                                } else {
                                    app.project.defs.enums.push(open_def.clone());
                                }
                                app.enum_saved_defs.insert(open_id, open_def.clone());
                                if let Some(pos) = app.enum_saved.iter().position(|e| e.id == open_id) {
                                    app.enum_saved[pos] = open_def.clone();
                                } else {
                                    app.enum_saved.push(open_def.clone());
                                }
                                if let Some(draft) = app.enum_drafts.get_mut(&open_id) {
                                    draft.saved_revision = draft.revision;
                                }
                                for ((eid, vid), rect) in &app.enum_value_tile_rects {
                                    if *eid == open_id {
                                        app.enum_saved_tile_rects.insert((*eid, vid.clone()), *rect);
                                    }
                                }
                            }
                        }

                        let save_path = get_default_enum_file_path(app);
                        let resolver = |tid| app.resolve_tileset_image_size(tid);
                        let file_data = convert_project_enums_to_file(
                            &app.project.defs.enums,
                            &app.project.defs.tilesets,
                            Some(&app.enum_tileset_map),
                            &resolver,
                        );

                        match validate_enum_file_data(&file_data) {
                            Ok(()) => match write_enum_file_atomic(&save_path, &file_data) {
                                Ok(()) => {
                                    app.enum_unsaved = false;
                                    app.status_message = format!(
                                        "Saved enum definition to '{}'",
                                        save_path.display()
                                    );
                                    state.toast = Some((
                                        "Enum definition saved successfully".to_string(),
                                        Instant::now(),
                                    ));
                                }
                                Err(e) => {
                                    state.error_modal =
                                        Some(vec![format!("Failed to save enum definition: {e}")]);
                                }
                            },
                            Err(errs) => {
                                state.error_modal = Some(errs);
                            }
                        }
                    }

                    ui.add_space(6.0);

                    // "Revert" button (outline, Title Case)
                    let revert_btn = ui.add_enabled(
                        is_dirty,
                        egui::Button::new(RichText::new("Revert").size(12.0).color(t.ink))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(
                                1.0,
                                if is_dirty { t.line_strong } else { t.line },
                            ))
                            .corner_radius(CornerRadius::same(3))
                            .min_size(vec2(70.0, 26.0)),
                    );
                    if revert_btn.clicked() {
                        state.revert_confirm_open = true;
                    }
                });
            });
        });

    // B. Central content area filling the space above the footer
    let sel_id = app.selected_enum_def;
    let selected_idx = sel_id.and_then(|id| app.enum_draft.iter().position(|e| e.id == id));

    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(t.bg)
                .inner_margin(Margin {
                    left: 20,
                    right: 12,
                    top: 12,
                    bottom: 12,
                }),
        )
        .show(ui, |ui| {
            ui.set_min_width(0.0);
            ui.set_max_width(ui.available_width());

            if let Some(enum_idx) = selected_idx {
                ScrollArea::vertical()
                    .id_salt("enum_right_panel_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let current_enum_id = app.enum_draft[enum_idx].id;
                        let is_val_dragging = state.value_drag.is_active();

                // 1. Sub-tabs: ENUM DEF / DOCUMENT
                ui.horizontal(|ui| {
                    let is_def = state.active_sub_tab == 0;
                    let btn_def = Frame::new()
                        .fill(if is_def { t.sunken } else { Color32::TRANSPARENT })
                        .stroke(if is_def {
                            Stroke::new(1.0, t.accent)
                        } else {
                            Stroke::NONE
                        })
                        .inner_margin(Margin::symmetric(10, 4))
                        .corner_radius(CornerRadius::same(2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("ENUM DEF")
                                     .size(11.0)
                                     .strong()
                                     .color(if is_def { t.accent_ink } else { t.muted }),
                            );
                        });
                    if btn_def.response.interact(Sense::click()).clicked() {
                        state.active_sub_tab = 0;
                    }

                    let is_doc = state.active_sub_tab == 1;
                    let btn_doc = Frame::new()
                        .fill(if is_doc { t.sunken } else { Color32::TRANSPARENT })
                        .stroke(if is_doc {
                            Stroke::new(1.0, t.accent)
                        } else {
                            Stroke::NONE
                        })
                        .inner_margin(Margin::symmetric(10, 4))
                        .corner_radius(CornerRadius::same(2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("DOCUMENT")
                                    .size(11.0)
                                    .color(if is_doc { t.accent_ink } else { t.muted }),
                            );
                        });
                    if btn_doc.response.interact(Sense::click()).clicked() {
                        state.active_sub_tab = 1;
                    }
                });

                ui.add_space(8.0);

                // 2. Title & Description (truncated with ellipsis, max-width: 100%)
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{} (Enum)", app.enum_draft[enum_idx].name))
                            .size(20.0)
                            .strong()
                            .color(t.ink_strong),
                    )
                    .truncate(),
                );
                ui.label(
                    RichText::new(
                        "Define enumeration keys, localized labels, badge colors, and optional bound tile graphics.",
                    )
                    .size(11.5)
                    .color(t.muted),
                );

                ui.add_space(10.0);

                // 3. Section: ENUM IDENTIFIER & ATTRIBUTES (Frozen top card)
                ui.label(
                    RichText::new("ENUM IDENTIFIER & ATTRIBUTES")
                        .size(11.0)
                        .strong()
                        .color(t.accent_ink),
                );
                ui.add_space(4.0);

                Frame::new()
                    .fill(t.panel)
                    .stroke(t.hairline())
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::same(4))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        // Row 1: Enum Identifier (full width) + Export this enum icon
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Enum Identifier").strong().color(t.ink_strong));
                            let export_btn_w = CONTROL_HEIGHT;
                            let text_w = (ui.available_width() - export_btn_w - 8.0).max(60.0);
                            let mut name_text = app.enum_draft[enum_idx].name.clone();
                            let name_resp = ui.add_sized(
                                [text_w, CONTROL_HEIGHT],
                                TextEdit::singleline(&mut name_text)
                                    .char_limit(48)
                                    .id(egui::Id::new(("enum_identifier", current_enum_id)))
                                    .hint_text("Identifier")
                                    .vertical_align(Align::Center)
                                    .margin(Margin::symmetric(8, 0))
                                    .interactive(!is_val_dragging),
                            );
                            if name_resp.changed() {
                                if name_text.chars().count() > 48 {
                                    name_text = name_text.chars().take(48).collect();
                                }
                                app.enum_draft[enum_idx].name = name_text;
                                app.enum_draft_revision += 1;
                            }

                            // Top-right "Export this enum" button (1:1 square: CONTROL_HEIGHT x CONTROL_HEIGHT)
                            let export_btn = ui
                                .add_sized(
                                    [CONTROL_HEIGHT, CONTROL_HEIGHT],
                                    egui::Button::new(Icon::Export.rich(ui.ctx(), 14.0, t.muted))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(t.hairline())
                                        .corner_radius(CornerRadius::same(3)),
                                )
                                .on_hover_text("Export this enum (.fleiyaenum)");

                            if export_btn.clicked() {
                                let name_snapshot = app.enum_draft[enum_idx].name.clone();
                                dialog_manager.request_export_one(&name_snapshot, current_enum_id);
                            }
                        });

                        ui.add_space(8.0);

                        // Row 2: Tileset dropdown (= iconTilesetUid)
                        ui.horizontal(|ui| {
                            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;
                            ui.label(RichText::new("Tileset:").strong().color(t.ink_strong));
                            let tilesets = &app.project.defs.tilesets;
                            let prev_ts = app.enum_draft[enum_idx].tileset_id;
                            let current_tileset_name = prev_ts
                                .and_then(|id| tilesets.iter().find(|ts| ts.id == id))
                                .map(|ts| ts.name.as_str())
                                .unwrap_or("Select Tileset...");

                            let mut chosen_ts = prev_ts;

                            let combo_theme = ContextMenuTheme::default();
                            let t_combo = t;

                            egui::ComboBox::from_id_salt("enum_mode_tileset_selector")
                                .selected_text(current_tileset_name)
                                .popup_style(egui::style::StyleModifier::new(move |style: &mut egui::Style| {
                                    style.visuals.menu_corner_radius = combo_theme.corner_radius;
                                    style.visuals.window_fill =
                                        combo_theme.frame_fill.unwrap_or(t_combo.panel);
                                    style.visuals.window_stroke =
                                        combo_theme.frame_stroke.unwrap_or_else(|| t_combo.hairline());
                                    style.visuals.popup_shadow = combo_theme.shadow;
                                }))
                                .icon(|ui, rect, visuals, is_open| {
                                    let icon = if is_open {
                                        Icon::ChevronUp
                                    } else {
                                        Icon::ChevronDown
                                    };
                                    lc_canvas::icons::paint_icon(
                                        ui.painter(),
                                        rect.center(),
                                        icon,
                                        12.0,
                                        visuals.text_color(),
                                    );
                                })
                                .show_ui(ui, |ui| {
                                    for ts in tilesets {
                                        let is_sel = prev_ts == Some(ts.id);
                                        if ui.selectable_label(is_sel, &ts.name).clicked() {
                                            chosen_ts = Some(ts.id);
                                        }
                                    }
                                });

                            if chosen_ts != prev_ts {
                                let has_bound_tiles =
                                    app.enum_draft[enum_idx].values.iter().any(|v| v.tile.is_some());
                                if has_bound_tiles {
                                    state.pending_tileset_change = Some(chosen_ts);
                                } else {
                                    tracing::info!(
                                        "[DIAGNOSE] Changing tileset for open enum {:?}: writing field enum_def.tileset_id from {:?} to {:?}",
                                        current_enum_id, prev_ts, chosen_ts
                                    );
                                    app.enum_draft[enum_idx].tileset_id = chosen_ts;
                                    if let Some(ed) = app.enum_drafts.get_mut(&current_enum_id) {
                                        ed.def.tileset_id = chosen_ts;
                                        ed.revision += 1;
                                    }
                                    if let Some(tid) = chosen_ts {
                                        app.enum_tileset_map.insert(current_enum_id, tid);
                                    } else {
                                        app.enum_tileset_map.remove(&current_enum_id);
                                    }
                                    app.enum_draft_revision += 1;
                                }
                            }
                        });

                        ui.add_space(8.0);

                        // Row 3: Tags row
                        ui.horizontal(|ui| {
                            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;
                            ui.label(RichText::new("Tags:").strong().color(t.ink_strong));

                            let assigned_tag_ids: Vec<crate::shell_app::TagId> = app
                                .enum_tags_map
                                .get(&current_enum_id)
                                .cloned()
                                .unwrap_or_default();

                            let mut remove_tag: Option<crate::shell_app::TagId> = None;
                            let mut rename_tag_action: Option<(crate::shell_app::TagId, String)> =
                                None;
                            let mut create_tag_action: Option<String> = None;
                            let mut assign_tag_choice: Option<crate::shell_app::TagId> = None;
                            let mut recall_clicked = false;
                            let mut recall_btn_rect = Rect::ZERO;
                            let mut recall_btn_resp: Option<egui::Response> = None;
                            let unused_tags = app.unused_tags_for_enum(current_enum_id);
                            let has_unused = !unused_tags.is_empty();

                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                                ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;

                                if assigned_tag_ids.is_empty() && !state.tag_inline_new_active {
                                    let (no_tag_rect, no_tag_resp) = ui.allocate_exact_size(
                                        vec2(76.0, CONTROL_HEIGHT),
                                        Sense::click(),
                                    );
                                    let painter = ui.painter();
                                    let border_stroke = if no_tag_resp.hovered() {
                                        Stroke::new(1.0, t.muted)
                                    } else {
                                        Stroke::new(1.0, t.muted.linear_multiply(0.35))
                                    };
                                    painter.rect_stroke(
                                        no_tag_rect,
                                        CornerRadius::same(3),
                                        border_stroke,
                                        StrokeKind::Inside,
                                    );
                                    painter.text(
                                        no_tag_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "(No tag)",
                                        egui::FontId::proportional(11.5),
                                        t.muted,
                                    );
                                    if no_tag_resp.clicked() {
                                        state.tag_inline_new_active = true;
                                        state.tag_inline_new_text.clear();
                                    }
                                    no_tag_resp.on_hover_text("Click to add tag");
                                }

                                // Render assigned tag chips
                                for &tag_id in &assigned_tag_ids {
                                    let tag_name = app
                                        .tag_registry
                                        .iter()
                                        .find(|t| t.id == tag_id)
                                        .map(|t| t.name.clone())
                                        .unwrap_or_else(|| format!("Tag_{}", tag_id.0));

                                    if state.tag_editing_id == Some(tag_id) {
                                        let text_edit_id = Id::new(("tag_edit_input", tag_id));
                                        let resp = ui.add_sized(
                                            [84.0, CONTROL_HEIGHT],
                                            TextEdit::singleline(&mut state.tag_editing_text)
                                                .id(text_edit_id)
                                                .vertical_align(Align::Center)
                                                .margin(Margin::symmetric(8, 0)),
                                        );
                                        resp.request_focus();
                                        if ui.input(|i| i.key_pressed(Key::Enter))
                                            || resp.lost_focus()
                                        {
                                            rename_tag_action = Some((
                                                tag_id,
                                                state.tag_editing_text.clone(),
                                            ));
                                            state.tag_editing_id = None;
                                        } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                                            state.tag_editing_id = None;
                                        }
                                    } else {
                                        let name_galley = ui.painter().layout_no_wrap(
                                            tag_name.clone(),
                                            egui::FontId::proportional(12.0),
                                            t.ink_strong,
                                        );
                                        let close_btn_w = 16.0;
                                        let chip_w =
                                            (name_galley.size().x + close_btn_w + 22.0).max(44.0);
                                        let (chip_rect, chip_resp) = ui.allocate_exact_size(
                                            vec2(chip_w, CONTROL_HEIGHT),
                                            Sense::click(),
                                        );

                                        let painter = ui.painter();
                                        let chip_fill =
                                            if chip_resp.hovered() { t.canvas } else { t.sunken };
                                        painter.rect_filled(
                                            chip_rect,
                                            CornerRadius::same(3),
                                            chip_fill,
                                        );
                                        painter.rect_stroke(
                                            chip_rect,
                                            CornerRadius::same(3),
                                            t.hairline(),
                                            StrokeKind::Inside,
                                        );

                                        let text_pos =
                                            Pos2::new(chip_rect.min.x + 8.0, chip_rect.center().y);
                                        painter.galley(
                                            Pos2::new(
                                                text_pos.x,
                                                text_pos.y - name_galley.size().y / 2.0,
                                            ),
                                            name_galley,
                                            t.ink_strong,
                                        );

                                        let close_rect = Rect::from_min_size(
                                            Pos2::new(
                                                chip_rect.max.x - 20.0,
                                                chip_rect.min.y + (CONTROL_HEIGHT - 16.0) / 2.0,
                                            ),
                                            vec2(16.0, 16.0),
                                        );
                                        let close_resp = ui.interact(
                                            close_rect,
                                            Id::new(("tag_chip_close", tag_id)),
                                            Sense::click(),
                                        ).on_hover_text("Remove tag");
                                        let close_color =
                                            if close_resp.hovered() { t.ink_strong } else { t.muted };
                                        lc_canvas::icons::paint_icon(
                                            painter,
                                            close_rect.center(),
                                            Icon::Close,
                                            11.0,
                                            close_color,
                                        );

                                        if close_resp.clicked() {
                                            remove_tag = Some(tag_id);
                                        } else if chip_resp.double_clicked() {
                                            state.tag_editing_id = Some(tag_id);
                                            state.tag_editing_text = tag_name;
                                        } else {
                                            chip_resp.on_hover_text("Double-click to rename tag");
                                        }
                                    }
                                }

                                if state.tag_inline_new_active {
                                    let text_edit_id = Id::new("tag_inline_new_input");
                                    let resp = ui.add_sized(
                                        [90.0, CONTROL_HEIGHT],
                                        TextEdit::singleline(&mut state.tag_inline_new_text)
                                            .id(text_edit_id)
                                            .hint_text("Tag name...")
                                            .vertical_align(Align::Center)
                                            .margin(Margin::symmetric(8, 0)),
                                    );
                                    resp.request_focus();
                                    if ui.input(|i| i.key_pressed(Key::Enter)) || resp.lost_focus()
                                    {
                                        create_tag_action = Some(state.tag_inline_new_text.clone());
                                        state.tag_inline_new_active = false;
                                    } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                                        state.tag_inline_new_active = false;
                                    }
                                }

                                // 1:1 Square "+" button (CONTROL_HEIGHT x CONTROL_HEIGHT)
                                let plus_btn = ui.add_sized(
                                    [CONTROL_HEIGHT, CONTROL_HEIGHT],
                                    egui::Button::new(Icon::Add.rich(ui.ctx(), 13.0, t.ink_strong))
                                        .fill(t.sunken)
                                        .stroke(t.hairline())
                                        .corner_radius(CornerRadius::same(3)),
                                )
                                .on_hover_text("Add tag");
                                if plus_btn.clicked() {
                                    state.tag_inline_new_active = true;
                                    state.tag_inline_new_text.clear();
                                }

                                // 1:1 Square "⌄" button (CONTROL_HEIGHT x CONTROL_HEIGHT)
                                let recall_icon_color = if has_unused {
                                    t.ink_strong
                                } else {
                                    t.muted.linear_multiply(0.4)
                                };

                                let recall_btn = ui.add_enabled(
                                    has_unused,
                                    egui::Button::new(Icon::ChevronDown.rich(
                                        ui.ctx(),
                                        13.0,
                                        recall_icon_color,
                                    ))
                                    .fill(t.sunken)
                                    .stroke(t.hairline())
                                    .corner_radius(CornerRadius::same(3)),
                                )
                                .on_hover_text(if has_unused {
                                    "Assign existing tag"
                                } else {
                                    "No other tags available"
                                });

                                recall_btn_rect = recall_btn.rect;
                                if recall_btn.clicked() {
                                    recall_clicked = true;
                                    state.tag_recall_open = !state.tag_recall_open;
                                }
                                recall_btn_resp = Some(recall_btn);
                            });

                            if let Some(tag_id) = remove_tag {
                                app.remove_tag_from_enum(current_enum_id, tag_id);
                            }
                            if let Some((tag_id, new_name)) = rename_tag_action {
                                if let Err(e) = app.rename_tag(tag_id, &new_name) {
                                    state.toast = Some((e, Instant::now()));
                                }
                            }
                            if let Some(name) = create_tag_action {
                                let trimmed = name.trim();
                                if !trimmed.is_empty() {
                                    let tag_id = app.get_or_create_tag(trimmed);
                                    app.assign_tag_to_enum(current_enum_id, tag_id);
                                }
                            }

                            let tag_popup_id = Id::new("tag_recall_popup");
                            let is_open_now = Popup::is_id_open(ui.ctx(), tag_popup_id);
                            let just_opened = is_open_now && !state.tag_recall_was_open;
                            state.tag_recall_was_open = is_open_now;
                            if just_opened {
                                state.tag_recall_search.clear();
                                state.tag_recall_selected_idx = 0;
                            }

                            if let Some(ref recall_btn) = recall_btn_resp {
                                if has_unused {
                                    let tag_popup_theme = ContextMenuTheme::default();
                                    Popup::from_toggle_button_response(recall_btn)
                                        .id(tag_popup_id)
                                        .frame(tag_popup_theme.frame(&t))
                                        .width(200.0)
                                        .show(|ui| {
                                        ui.set_width(200.0);
                                        let q = state.tag_recall_search.trim().to_lowercase();
                                        let filtered_unused: Vec<&crate::shell_app::TagDef> = unused_tags
                                            .iter()
                                            .filter(|tag| q.is_empty() || tag.name.to_lowercase().contains(&q))
                                            .collect();

                                        // Search bar on top (outside ScrollArea, does not scroll)
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;
                                            let icon_rect = Rect::from_min_size(
                                                ui.cursor().min,
                                                vec2(16.0, CONTROL_HEIGHT),
                                            );
                                            lc_canvas::icons::paint_icon(
                                                ui.painter(),
                                                icon_rect.center(),
                                                Icon::Search,
                                                12.0,
                                                t.muted,
                                            );
                                            ui.add_space(20.0);
                                            let search_w = ui.available_width().max(60.0);
                                            let search_resp = ui.add_sized(
                                                [search_w, CONTROL_HEIGHT],
                                                TextEdit::singleline(&mut state.tag_recall_search)
                                                    .id(Id::new("tag_recall_search_input"))
                                                    .hint_text("Search tags...")
                                                    .vertical_align(Align::Center)
                                                    .margin(Margin::symmetric(6, 0)),
                                            );
                                            if just_opened {
                                                search_resp.request_focus();
                                            }
                                        });

                                        ui.add_space(4.0);

                                        // Keyboard navigation
                                        if ui.input(|i| i.key_pressed(Key::ArrowDown)) && !filtered_unused.is_empty() {
                                            state.tag_recall_selected_idx =
                                                (state.tag_recall_selected_idx + 1).min(filtered_unused.len() - 1);
                                        }
                                        if ui.input(|i| i.key_pressed(Key::ArrowUp)) && state.tag_recall_selected_idx > 0 {
                                            state.tag_recall_selected_idx -= 1;
                                        }
                                        if ui.input(|i| i.key_pressed(Key::Enter)) {
                                            if let Some(tag) = filtered_unused.get(state.tag_recall_selected_idx) {
                                                assign_tag_choice = Some(tag.id);
                                                ui.close();
                                            }
                                        }
                                        if ui.input(|i| i.key_pressed(Key::Escape)) {
                                            ui.close();
                                        }

                                        // ScrollArea for items: max_height(5.0 * CONTROL_HEIGHT)
                                        ScrollArea::vertical()
                                            .id_salt("tag_recall_scroll")
                                            .max_height(5.0 * CONTROL_HEIGHT)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                if filtered_unused.is_empty() {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(8.0);
                                                        ui.label(RichText::new("No tags found").size(12.0).color(t.muted));
                                                        ui.add_space(8.0);
                                                    });
                                                } else {
                                                    for (idx, tag) in filtered_unused.iter().enumerate() {
                                                        let is_kb_sel = idx == state.tag_recall_selected_idx;
                                                        let (item_rect, item_resp) = ui.allocate_exact_size(
                                                            vec2(ui.available_width(), CONTROL_HEIGHT),
                                                            Sense::click(),
                                                        );
                                                        if item_resp.hovered() {
                                                            state.tag_recall_selected_idx = idx;
                                                        }
                                                        let item_fill = if item_resp.hovered() || is_kb_sel {
                                                            t.sunken
                                                        } else {
                                                            Color32::TRANSPARENT
                                                        };
                                                        ui.painter().rect_filled(item_rect, CornerRadius::ZERO, item_fill);

                                                        let text_rect = item_rect.shrink2(vec2(8.0, 0.0));
                                                        ui.new_child(
                                                            egui::UiBuilder::new()
                                                                .max_rect(text_rect)
                                                                .layout(Layout::left_to_right(Align::Center)),
                                                        )
                                                        .add(
                                                            egui::Label::new(
                                                                RichText::new(&tag.name)
                                                                    .size(12.0)
                                                                    .color(t.ink_strong),
                                                            )
                                                            .truncate()
                                                            .selectable(false),
                                                        );

                                                        let item_resp = item_resp.on_hover_text(&tag.name);
                                                        if item_resp.clicked() {
                                                            assign_tag_choice = Some(tag.id);
                                                            ui.close();
                                                        }
                                                    }
                                                }
                                            });
                                    });
                                }
                            }

                            if let Some(tag_id) = assign_tag_choice {
                                app.assign_tag_to_enum(current_enum_id, tag_id);
                            }
                        });
                    });

                ui.add_space(10.0);

                let enum_def = &mut app.enum_draft[enum_idx];
                let current_enum_id = enum_def.id;

                // 4. Section: ENUM VALUES Toolbar
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "ENUM VALUES ({} DEFINED)",
                            enum_def.values.len()
                        ))
                        .size(11.0)
                        .strong()
                        .color(t.accent_ink),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let add_btn = ui.add_sized(
                            [110.0, CONTROL_HEIGHT],
                            egui::Button::new(
                                RichText::new("+ Add Value")
                                    .size(12.0)
                                    .strong()
                                    .color(t.invert_fg),
                            )
                            .fill(t.accent)
                            .corner_radius(CornerRadius::same(3)),
                        )
                        .on_hover_text("Add a new value to this enum");

                        if add_btn.clicked() {
                            let next_num = enum_def.values.len() + 1;
                            let new_id = format!("Value_{next_num}");

                            let default_colors: [[u8; 4]; 6] = [
                                [156, 163, 175, 255], // Gray
                                [61, 214, 184, 255],  // Teal
                                [45, 129, 255, 255],  // Blue
                                [168, 85, 247, 255],  // Purple
                                [255, 193, 77, 255],  // Amber
                                [239, 68, 68, 255],   // Red
                            ];
                            let color = default_colors[(next_num - 1) % default_colors.len()];

                            let new_val = EnumValueDef {
                                id: new_id.clone(),
                                name: new_id,
                                color,
                                tile: None,
                            };

                            enum_def.values.push(new_val);
                            state.ensure_value_uids(enum_def.id, enum_def.values.len());
                            app.enum_draft_revision += 1;
                        }
                    });
                });

                ui.add_space(4.0);

                // 5. Section: Value rows (pure deterministic row_layout)
                ui.vertical(|ui| {
                        let row_w = ui.available_width().max(280.0);
                        let uids = state.ensure_value_uids(enum_def.id, enum_def.values.len()).to_vec();
                        let mut val_row_rects = Vec::with_capacity(enum_def.values.len());

                        let drag_theme = DragTheme::from_accent(t.accent);

                        for slot in state.value_drag.layout(enum_def.values.len()) {
                            let idx = match slot {
                                Slot::Item(i) => i,
                                Slot::Placeholder => {
                                    // Ghost drop slot: marks where the dragged value will land
                                    let ph_h = state.value_drag.row_size().map_or(32.0, |size| size.y);
                                    let (ph_rect, _) = ui.allocate_exact_size(vec2(row_w, ph_h), Sense::hover());
                                    paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                                    val_row_rects.push(ph_rect);
                                    ui.add_space(4.0);
                                    continue;
                                }
                            };
                            let Some(val) = enum_def.values.get_mut(idx) else {
                                continue;
                            };
                            let (row_rect, _) = ui.allocate_exact_size(vec2(row_w, 32.0), Sense::hover());
                            val_row_rects.push(row_rect);

                            let r = row_layout(row_rect);

                            // 1. Drag handle (20px wide, 32px height)
                            let handle_resp = ui.interact(r.handle, Id::new(uids[idx]).with("handle"), Sense::drag());
                            if handle_resp.hovered() {
                                ui.ctx().set_cursor_icon(CursorIcon::Grab);
                            }
                            if handle_resp.drag_started() {
                                state.value_drag.begin(ui.ctx(), idx, r.row);
                                ui.memory_mut(|m| {
                                    if let Some(fid) = m.focused() {
                                        m.surrender_focus(fid);
                                    }
                                    m.stop_text_input();
                                });
                            }
                            let handle_painter = ui.painter().with_clip_rect(r.handle);
                            draw_six_dot_handle(&handle_painter, r.handle, handle_resp.hovered(), &t);

                            // 2. Bound Tile 32x32, 1:1 SQUARE
                            let tile_resp = ui.interact(r.bound_tile, Id::new(uids[idx]).with("tile"), Sense::click());
                            let tile_painter = ui.painter().with_clip_rect(r.bound_tile);
                            tile_painter.rect_filled(r.bound_tile, CornerRadius::same(3), t.sunken);

                            let mut drew_tile = false;
                            let enum_ts_id = enum_def.tileset_id.or_else(|| val.tile.map(|t| t.tileset_id));
                            if let Some(ref tr) = val.tile {
                                if let Some(ts_id) = enum_ts_id {
                                    if let Some(tileset) = app.project.defs.tileset(ts_id) {
                                        if let Ok(tex) = app.assets.load_texture(ui.ctx(), &tileset.path) {
                                        let sz = tex.size_vec2();
                                        let img_w = sz.x as u32;
                                        let img_h = sz.y as u32;
                                        let tile_w = tileset.tile_size.x.max(1);
                                        let tile_h = tileset.tile_size.y.max(1);
                                        let padding = tileset.padding;
                                        let spacing = tileset.spacing;
                                        let eff_w = img_w.saturating_sub(padding * 2);
                                        let eff_h = img_h.saturating_sub(padding * 2);
                                        let stride_x = tile_w + spacing;
                                        let stride_y = tile_h + spacing;
                                        let cols = (eff_w + spacing)
                                            .checked_div(stride_x)
                                            .unwrap_or(1)
                                            .max(1);
                                        let rows = (eff_h + spacing)
                                            .checked_div(stride_y)
                                            .unwrap_or(1)
                                            .max(1);
                                        let grid = TileGrid::new(tile_w, tile_h, spacing, padding, cols, rows);

                                        let tile_rect = app
                                            .enum_value_tile_rects
                                            .get(&(current_enum_id, val.id.clone()))
                                            .copied()
                                            .unwrap_or_else(|| TileRect::single(tr.tile_id % cols, tr.tile_id / cols));

                                        let region_page = tile_rect.union_rect(&grid);
                                        let u0 = region_page.min.x / img_w as f32;
                                        let v0 = region_page.min.y / img_h as f32;
                                        let u1 = region_page.max.x / img_w as f32;
                                        let v1 = region_page.max.y / img_h as f32;

                                        let slot_rect = r.bound_tile.shrink(1.0);
                                        let region_w = region_page.width().max(1.0);
                                        let region_h = region_page.height().max(1.0);
                                        let scale = (slot_rect.width() / region_w).min(slot_rect.height() / region_h);
                                        let fitted_w = (region_w * scale).round();
                                        let fitted_h = (region_h * scale).round();
                                        let draw_min = Pos2::new(
                                            (slot_rect.center().x - fitted_w / 2.0).round(),
                                            (slot_rect.center().y - fitted_h / 2.0).round(),
                                        );
                                        let draw_rect = Rect::from_min_size(draw_min, vec2(fitted_w, fitted_h));

                                        draw_checkerboard(&tile_painter, draw_rect, 4.0, &t);
                                        tile_painter.image(
                                            tex.id(),
                                            draw_rect,
                                            Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                                            Color32::WHITE,
                                        );
                                        drew_tile = true;
                                    }
                                }
                            }
                                if !drew_tile {
                                    let col = Color32::from_rgb(
                                        ((tr.tile_id * 53 + 90) % 200 + 40) as u8,
                                        ((tr.tile_id * 97 + 120) % 200 + 40) as u8,
                                        ((tr.tile_id * 163 + 150) % 200 + 40) as u8,
                                    );
                                    tile_painter.rect_filled(r.bound_tile.shrink(2.0), CornerRadius::same(2), col);
                                    tile_painter.text(
                                        r.bound_tile.center(),
                                        Align2::CENTER_CENTER,
                                        format!("{}", tr.tile_id),
                                        egui::FontId::monospace(11.0),
                                        Color32::WHITE,
                                    );
                                }
                            } else {
                                tile_painter.rect_stroke(
                                    r.bound_tile.shrink(1.0),
                                    CornerRadius::same(3),
                                    Stroke::new(1.0, t.muted.linear_multiply(0.4)),
                                    StrokeKind::Inside,
                                );
                            }

                            let tile_border = if tile_resp.hovered() {
                                Stroke::new(1.5, t.accent)
                            } else {
                                t.hairline()
                            };
                            tile_painter.rect_stroke(r.bound_tile, CornerRadius::same(3), tile_border, StrokeKind::Inside);

                            let is_right_clicked = tile_resp.secondary_clicked()
                                || (tile_resp.hovered()
                                    && ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary)));

                            if is_right_clicked {
                                if val.tile.is_some() {
                                    val.tile = None;
                                    app.enum_value_tile_rects
                                        .remove(&(current_enum_id, val.id.clone()));
                                    app.enum_draft_revision += 1;
                                }
                            } else if tile_resp.clicked() {
                                let active_ts = enum_def
                                    .tileset_id
                                    .or_else(|| val.tile.map(|t| t.tileset_id));
                                let current_rect = app
                                    .enum_value_tile_rects
                                    .get(&(current_enum_id, val.id.clone()))
                                    .copied()
                                    .or_else(|| val.tile.map(|t| TileRect::single(t.tile_id % 16, t.tile_id / 16)));
                                state.tile_picker.open_target(
                                    Some(PickerTarget {
                                        enum_id: current_enum_id,
                                        value_id: val.id.clone(),
                                    }),
                                    active_ts,
                                    current_rect,
                                );
                            }

                            // 3. Color swatch 32x32, 1:1 SQUARE
                            let color_resp = ui.interact(r.color, Id::new(uids[idx]).with("color"), Sense::click());
                            let color_painter = ui.painter().with_clip_rect(r.color);
                            let val_color = Color32::from_rgba_premultiplied(val.color[0], val.color[1], val.color[2], 255);
                            color_painter.rect_filled(r.color, CornerRadius::same(3), val_color);
                            let color_border = if color_resp.hovered() {
                                Stroke::new(1.5, t.ink_strong)
                            } else {
                                Stroke::new(1.0, Color32::from_black_alpha(80))
                            };
                            color_painter.rect_stroke(r.color, CornerRadius::same(3), color_border, StrokeKind::Inside);

                            // Built-in egui color picker popup anchored to the swatch
                            egui::Popup::from_toggle_button_response(&color_resp).show(|ui| {
                                ui.spacing_mut().slider_width = 180.0;
                                let mut c32 = val_color;
                                if egui::color_picker::color_picker_color32(ui, &mut c32, egui::color_picker::Alpha::Opaque) {
                                    val.color = [c32.r(), c32.g(), c32.b(), 255];
                                    if !state.color_picker_dragging {
                                        state.color_picker_dragging = true;
                                        app.enum_draft_revision += 1;
                                    }
                                }
                            });

                            // 4. Value TextEdit in r.input with stable UID
                            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(r.input));
                            let id_resp = child_ui.add_sized(
                                [r.input.width(), 32.0],
                                TextEdit::singleline(&mut val.id)
                                    .id(egui::Id::new(uids[idx]))
                                    .hint_text("Value Identifier")
                                    .vertical_align(Align::Center)
                                    .margin(Margin::symmetric(8, 0))
                                    .interactive(!is_val_dragging),
                            );
                            if id_resp.changed() {
                                val.name = val.id.clone();
                                app.enum_draft_revision += 1;
                            }

                            // 1.5px outline stroke in value's color
                            ui.painter().rect_stroke(
                                r.input,
                                CornerRadius::same(3),
                                Stroke::new(1.5, val_color),
                                StrokeKind::Inside,
                            );

                            // 5. Delete button 32x32 pinned to right edge
                            let del_resp = ui.interact(r.delete_btn, Id::new(uids[idx]).with("delete"), Sense::click());
                            let del_painter = ui.painter().with_clip_rect(r.delete_btn);
                            let del_fill = if del_resp.hovered() { t.canvas } else { t.sunken };
                            del_painter.rect_filled(r.delete_btn, CornerRadius::same(3), del_fill);
                            del_painter.rect_stroke(r.delete_btn, CornerRadius::same(3), t.hairline(), StrokeKind::Inside);
                            let del_icon_color = if del_resp.hovered() { t.ink_strong } else { t.muted };
                            lc_canvas::icons::paint_icon(&del_painter, r.delete_btn.center(), Icon::Delete, 14.0, del_icon_color);

                            if del_resp.clicked() {
                                state.pending_delete = Some(DeleteTarget::EnumValue {
                                    enum_id: enum_def.id,
                                    val_id: val.id.clone(),
                                    name: val.id.clone(),
                                });
                            }

                            ui.add_space(4.0);
                        }

                        // Updates the drop target, Grabbing cursor and auto-scroll
                        state.value_drag.update(ui, &val_row_rects, &drag_theme);

                        // Ghost for dragged value row
                        if let Some(drag_idx) = state.value_drag.dragging_index() {
                            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                            if let Some(dragged_val) = enum_def.values.get(drag_idx) {
                                if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos())) {
                                    let ghost_pos = pos - state.value_drag.grab_offset();
                                    Area::new(Id::new("enum_val_drag_ghost"))
                                        .order(Order::Tooltip)
                                        .fixed_pos(ghost_pos)
                                        .interactable(false)
                                        .show(ui.ctx(), |ui| {
                                            Frame::new()
                                                .fill(t.sunken)
                                                .stroke(Stroke::new(1.5, t.accent))
                                                .shadow(egui::epaint::Shadow {
                                                    offset: [0, 4],
                                                    blur: 10,
                                                    spread: 1,
                                                    color: Color32::from_black_alpha(140),
                                                })
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::ZERO)
                                                .show(ui, |ui| {
                                                    let ghost_rect = Rect::from_min_size(ui.cursor().min, vec2(row_w, 32.0));
                                                    ui.allocate_rect(ghost_rect, Sense::hover());
                                                    let gr = row_layout(ghost_rect);
                                                    let gp = ui.painter();

                                                    draw_six_dot_handle(gp, gr.handle, true, &t);

                                                    gp.rect_filled(gr.bound_tile, CornerRadius::same(3), t.sunken);
                                                    let mut drew_tile = false;
                                                    if let Some(ref tr) = dragged_val.tile {
                                                        if let Some(tileset) = app.project.defs.tileset(tr.tileset_id) {
                                                            if let Ok(tex) = app.assets.load_texture(ui.ctx(), &tileset.path) {
                                                                let sz = tex.size_vec2();
                                                                let img_w = sz.x as u32;
                                                                let img_h = sz.y as u32;
                                                                let tile_w = tileset.tile_size.x.max(1);
                                                                let tile_h = tileset.tile_size.y.max(1);
                                                                let padding = tileset.padding;
                                                                let spacing = tileset.spacing;
                                                                let eff_w = img_w.saturating_sub(padding * 2);
                                                                let stride_x = tile_w + spacing;
                                                                let stride_y = tile_h + spacing;
                                                                let cols = (eff_w + spacing)
                                                                    .checked_div(stride_x)
                                                                    .unwrap_or(1)
                                                                    .max(1);
                                                                let c = tr.tile_id % cols;
                                                                let r_tile = tr.tile_id / cols;
                                                                let px = padding + c * stride_x;
                                                                let py = padding + r_tile * stride_y;
                                                                let u0 = px as f32 / img_w as f32;
                                                                let v0 = py as f32 / img_h as f32;
                                                                let u1 = (px + tile_w) as f32 / img_w as f32;
                                                                let v1 = (py + tile_h) as f32 / img_h as f32;
                                                                draw_checkerboard(gp, gr.bound_tile, 4.0, &t);
                                                                gp.image(
                                                                    tex.id(),
                                                                    gr.bound_tile.shrink(1.0),
                                                                    Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                                                                    Color32::WHITE,
                                                                );
                                                                drew_tile = true;
                                                            }
                                                        }
                                                        if !drew_tile {
                                                            let col = Color32::from_rgb(
                                                                ((tr.tile_id * 53 + 90) % 200 + 40) as u8,
                                                                ((tr.tile_id * 97 + 120) % 200 + 40) as u8,
                                                                ((tr.tile_id * 163 + 150) % 200 + 40) as u8,
                                                            );
                                                            gp.rect_filled(gr.bound_tile.shrink(2.0), CornerRadius::same(2), col);
                                                            gp.text(
                                                                gr.bound_tile.center(),
                                                                Align2::CENTER_CENTER,
                                                                format!("{}", tr.tile_id),
                                                                egui::FontId::monospace(11.0),
                                                                Color32::WHITE,
                                                            );
                                                        }
                                                    } else {
                                                        gp.rect_stroke(
                                                            gr.bound_tile.shrink(1.0),
                                                            CornerRadius::same(3),
                                                            Stroke::new(1.0, t.muted.linear_multiply(0.4)),
                                                            StrokeKind::Inside,
                                                        );
                                                    }
                                                    gp.rect_stroke(gr.bound_tile, CornerRadius::same(3), t.hairline(), StrokeKind::Inside);

                                                    let gc = Color32::from_rgba_premultiplied(
                                                        dragged_val.color[0],
                                                        dragged_val.color[1],
                                                        dragged_val.color[2],
                                                        255,
                                                    );
                                                    gp.rect_filled(gr.color, CornerRadius::same(3), gc);
                                                    gp.rect_stroke(gr.color, CornerRadius::same(3), Stroke::new(1.0, Color32::from_black_alpha(80)), StrokeKind::Inside);

                                                    gp.rect_filled(gr.input, CornerRadius::same(3), t.canvas);
                                                    gp.rect_stroke(gr.input, CornerRadius::same(3), Stroke::new(1.5, gc), StrokeKind::Inside);
                                                    gp.text(
                                                        Pos2::new(gr.input.min.x + 8.0, gr.input.center().y),
                                                        Align2::LEFT_CENTER,
                                                        &dragged_val.id,
                                                        egui::FontId::proportional(13.0),
                                                        t.ink_strong,
                                                    );

                                                    gp.rect_filled(gr.delete_btn, CornerRadius::same(3), t.sunken);
                                                    gp.rect_stroke(gr.delete_btn, CornerRadius::same(3), t.hairline(), StrokeKind::Inside);
                                                    lc_canvas::icons::paint_icon(gp, gr.delete_btn.center(), Icon::Delete, 14.0, t.muted);
                                                });
                                        });
                                }
                            }
                        }
                    });

                // Resolve the value drag: drop on release, cancel on Escape
                if let DragOutcome::Dropped { from, to } = state.value_drag.finish(ui) {
                    if reorder_item(&mut enum_def.values, from, to) {
                        if let Some(uids) = state.value_uids.get_mut(&enum_def.id) {
                            reorder_item(uids, from, to);
                        }
                        app.enum_draft_revision += 1;
                    }
                }
                    });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("Select an enum definition from the left panel to edit.")
                            .color(t.muted),
                    );
                });
            }
        });

    // Reset color picker drag tracking on mouse release
    if ui.input(|i| i.pointer.any_released()) {
        state.color_picker_dragging = false;
    }

    // =========================================================================
    // 5. TILE PICKER POP-UP MODAL (Section D & E)
    // =========================================================================
    let target_enum_id = state
        .tile_picker
        .target
        .as_ref()
        .map(|t| t.enum_id)
        .or(app.selected_enum_def);

    let active_ts_id = target_enum_id.and_then(|eid| {
        app.enum_draft
            .iter()
            .find(|e| e.id == eid)
            .and_then(|e| e.tileset_id)
    });

    let active_tileset_data = active_ts_id
        .and_then(|id| app.project.defs.tileset(id))
        .map(TilesetData::from)
        .unwrap_or_else(TilesetData::empty);

    let current_bound_tile_rect = if let Some(target) = state.tile_picker.target.as_ref() {
        app.enum_draft
            .iter()
            .find(|e| e.id == target.enum_id)
            .and_then(|ed| ed.values.iter().find(|v| v.id == target.value_id))
            .and_then(|v| {
                let img_w = app
                    .resolve_tileset_image_size(active_tileset_data.id)
                    .map(|(w, _)| w)
                    .unwrap_or(256);
                let tile_w = active_tileset_data.tile_size.x.max(1);
                let padding = active_tileset_data.padding;
                let spacing = active_tileset_data.spacing;
                let eff_w = img_w.saturating_sub(padding * 2);
                let stride_x = tile_w + spacing;
                let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                app.get_enum_value_tile_rect(target.enum_id, &v.id, v.tile.as_ref(), cols)
            })
    } else {
        None
    };

    match state
        .tile_picker
        .show(ui.ctx(), &active_tileset_data, current_bound_tile_rect)
    {
        TilePickerEvent::Selected(picked_rect) => {
            if let Some(target) = state.tile_picker.target.take() {
                let target_enum_id = target.enum_id;
                let target_val_id = target.value_id;

                tracing::info!(
                    "[DIAGNOSE] TilePicker Selected: enum_id={:?}, value_id={}, picked_rect={:?}",
                    target_enum_id, target_val_id, picked_rect
                );

                let target_ts_id = app
                    .enum_draft
                    .iter()
                    .find(|e| e.id == target_enum_id)
                    .and_then(|e| e.tileset_id)
                    .or(active_ts_id);

                if let Some(ts_id) = target_ts_id {
                    let img_w = app
                        .resolve_tileset_image_size(ts_id)
                        .map(|(w, _)| w)
                        .unwrap_or(256);
                    let tile_w = active_tileset_data.tile_size.x.max(1);
                    let tile_h = active_tileset_data.tile_size.y.max(1);
                    let padding = active_tileset_data.padding;
                    let spacing = active_tileset_data.spacing;
                    let eff_w = img_w.saturating_sub(padding * 2);
                    let stride_x = tile_w + spacing;
                    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                    let grid = TileGrid::new(tile_w, tile_h, spacing, padding, cols, 1000);
                    let top_left_id = picked_rect.top_left_id(&grid);

                    if let Some(ed) = app.enum_draft.iter_mut().find(|e| e.id == target_enum_id) {
                        if let Some(v) = ed.values.iter_mut().find(|v| v.id == target_val_id) {
                            v.tile = Some(TileRef::new(ts_id, top_left_id));
                        }
                    }
                    if let Some(draft) = app.enum_drafts.get_mut(&target_enum_id) {
                        if let Some(v) = draft.def.values.iter_mut().find(|v| v.id == target_val_id) {
                            v.tile = Some(TileRef::new(ts_id, top_left_id));
                        }
                        draft.tile_rects.insert(target_val_id.clone(), picked_rect);
                        draft.revision += 1;
                    }
                    app.enum_value_tile_rects
                        .insert((target_enum_id, target_val_id), picked_rect);
                    app.enum_draft_revision += 1;
                }
                ui.ctx().request_repaint();
            }
        }
        TilePickerEvent::Closed => {
            state.tile_picker.target = None;
            state.tile_picker.target_val_id = None;
        }
        TilePickerEvent::None => {}
    }

    // =========================================================================
    // 6. CONFIRMATION MODALS
    // =========================================================================
    // Revert confirmation modal
    if state.revert_confirm_open {
        Area::new(Id::new("revert_enum_confirm_modal"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.heading("Discard Changes?");
                        ui.add_space(8.0);
                        ui.label("Discard all unsaved enum changes? This cannot be undone.");
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Discard").color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(220, 38, 38)),
                                )
                                .clicked()
                            {
                                if let Some(open_id) = app.selected_enum_def {
                                    if let Some(saved) = app.enum_saved_defs.get(&open_id).cloned().or_else(|| {
                                        app.project.defs.enums.iter().find(|e| e.id == open_id).cloned()
                                    }) {
                                        if let Some(slot) = app.enum_draft.iter_mut().find(|e| e.id == open_id) {
                                            *slot = saved.clone();
                                        }
                                        if let Some(draft) = app.enum_drafts.get_mut(&open_id) {
                                            draft.def = saved;
                                            draft.revision = draft.saved_revision;
                                        }
                                    }
                                    app.enum_value_tile_rects.retain(|(eid, _), _| *eid != open_id);
                                    for ((eid, vid), rect) in &app.enum_saved_tile_rects {
                                        if *eid == open_id {
                                            app.enum_value_tile_rects.insert((*eid, vid.clone()), *rect);
                                        }
                                    }
                                    state.toast = Some((
                                        "Reverted enum to last saved state".to_string(),
                                        Instant::now(),
                                    ));
                                }
                                state.revert_confirm_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                state.revert_confirm_open = false;
                            }
                        });
                    });
            });
    }

    // Change tileset confirmation modal
    if let Some(new_ts) = state.pending_tileset_change {
        Area::new(Id::new("change_tileset_confirm_modal"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(340.0);
                        ui.heading("Change Tileset?");
                        ui.add_space(8.0);
                        ui.label(
                            "Changing the tileset will clear all bound tiles in this enum. Continue?",
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Change Tileset").color(t.invert_fg),
                                    )
                                    .fill(t.accent),
                                )
                                .clicked()
                            {
                                if let Some(sel_id) = app.selected_enum_def {
                                    tracing::info!(
                                        "[DIAGNOSE] Confirmed tileset change for open enum {:?}: writing field enum_def.tileset_id to {:?}",
                                        sel_id, new_ts
                                    );
                                    if let Some(ed) =
                                        app.enum_draft.iter_mut().find(|e| e.id == sel_id)
                                    {
                                        ed.tileset_id = new_ts;
                                        for v in &mut ed.values {
                                            v.tile = None;
                                        }
                                    }
                                    if let Some(draft) = app.enum_drafts.get_mut(&sel_id) {
                                        draft.def.tileset_id = new_ts;
                                        for v in &mut draft.def.values {
                                            v.tile = None;
                                        }
                                        draft.tile_rects.clear();
                                        draft.revision += 1;
                                    }
                                    if let Some(tid) = new_ts {
                                        app.enum_tileset_map.insert(sel_id, tid);
                                    } else {
                                        app.enum_tileset_map.remove(&sel_id);
                                    }
                                    app.enum_value_tile_rects.retain(|(eid, _), _| *eid != sel_id);
                                    app.enum_draft_revision += 1;
                                }
                                state.pending_tileset_change = None;
                            }
                            if ui.button("Cancel").clicked() {
                                state.pending_tileset_change = None;
                            }
                        });
                    });
            });
    }

    // Delete confirmation modal
    if let Some(ref target) = state.pending_delete {
        let (title, desc) = match target {
            DeleteTarget::Enum {
                name,
                entity_count,
                field_count,
                ..
            } => {
                let usage_info = if *entity_count > 0 || *field_count > 0 {
                    format!(
                        "\n\nWARNING: Used by {} Entity Def(s) & {} Field(s).",
                        entity_count, field_count
                    )
                } else {
                    String::new()
                };
                (
                    "Delete Enum Definition?".to_string(),
                    format!("Are you sure you want to delete enum '{name}'?{usage_info}"),
                )
            }
            DeleteTarget::EnumValue { val_id, .. } => (
                "Delete Enum Value?".to_string(),
                format!("Are you sure you want to delete value '{val_id}'?"),
            ),
        };

        Area::new(Id::new("enum_delete_confirm_modal"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.heading(title);
                        ui.add_space(8.0);
                        ui.label(desc);
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Delete").color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(220, 38, 38)),
                                )
                                .clicked()
                            {
                                pending_delete_execute = state.pending_delete.take();
                            }
                            if ui.button("Cancel").clicked() {
                                state.pending_delete = None;
                            }
                        });
                    });
            });
    }

    if let Some(target) = pending_delete_execute {
        match target {
            DeleteTarget::Enum { id, .. } => {
                if let Some(pos) = app.enum_draft.iter().position(|e| e.id == id) {
                    app.enum_draft.remove(pos);
                    state.value_uids.remove(&id);
                    app.enum_tags_map.remove(&id);
                    app.cleanup_unused_tags();
                    app.selected_enum_def = app.enum_draft.first().map(|e| e.id);
                    app.enum_draft_revision += 1;
                }
            }
            DeleteTarget::EnumValue {
                enum_id, val_id, ..
            } => {
                if let Some(ed) = app.enum_draft.iter_mut().find(|e| e.id == enum_id) {
                    if let Some(pos) = ed.values.iter().position(|v| v.id == val_id) {
                        ed.values.remove(pos);
                        if let Some(uids) = state.value_uids.get_mut(&enum_id) {
                            if pos < uids.len() {
                                uids.remove(pos);
                            }
                        }
                        app.enum_draft_revision += 1;
                    }
                }
            }
        }
    }

    // Conflict Resolution Modal
    if let Some(ref mut cstate) = state.conflict_modal {
        let mut apply_clicked = false;
        let mut cancel_clicked = false;

        Area::new(Id::new("enum_conflict_resolution_modal"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, t.line_strong))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(440.0);
                        ui.heading("Enum Identifier Conflicts");
                        ui.add_space(8.0);
                        ui.label(
                            "One or more imported enums match existing enum identifiers in your draft.",
                        );
                        ui.add_space(8.0);

                        for c in &mut cstate.conflicts {
                            ui.group(|ui| {
                                ui.label(
                                    RichText::new(&c.file_enum.identifier).strong().color(t.accent_ink),
                                );
                                if c.entity_count > 0 || c.field_count > 0 {
                                    ui.label(
                                        RichText::new(format!(
                                            "Used by {} Entity Def(s) & {} Field(s)",
                                            c.entity_count, c.field_count
                                        ))
                                        .size(11.0)
                                        .color(t.muted),
                                    );
                                }
                                let lost = compute_lost_keys(&c.existing_values, &c.file_enum.values);
                                if !lost.is_empty() {
                                    ui.label(
                                        RichText::new(format!(
                                            "Warning: Values to be lost: {}",
                                            lost.join(", ")
                                        ))
                                        .size(11.0)
                                        .color(Color32::from_rgb(255, 193, 77)),
                                    );
                                }

                                ui.horizontal(|ui| {
                                    ui.radio_value(
                                        &mut c.choice,
                                        ConflictResolutionChoice::Replace,
                                        "Replace",
                                    );
                                    ui.radio_value(
                                        &mut c.choice,
                                        ConflictResolutionChoice::KeepBoth,
                                        "Keep Both (Rename)",
                                    );
                                    ui.radio_value(
                                        &mut c.choice,
                                        ConflictResolutionChoice::Skip,
                                        "Skip",
                                    );
                                });
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(8.0);
                        ui.checkbox(
                            &mut cstate.understand_checked,
                            "I understand that replacing an enum may break existing entity fields.",
                        );

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let can_apply = cstate.understand_checked
                                || !cstate
                                    .conflicts
                                    .iter()
                                    .any(|c| c.choice == ConflictResolutionChoice::Replace);
                            if ui
                                .add_enabled(
                                    can_apply,
                                    egui::Button::new(
                                        RichText::new("Apply Import").color(t.invert_fg),
                                    )
                                    .fill(t.accent),
                                )
                                .clicked()
                            {
                                apply_clicked = true;
                            }
                            if ui.button("Cancel").clicked() {
                                cancel_clicked = true;
                            }
                        });
                    });
            });

        if apply_clicked {
            let conflict_state_clone = cstate.clone();
            state.conflict_modal = None;
            apply_import_with_conflicts(app, &conflict_state_clone, &mut state);
        } else if cancel_clicked {
            state.conflict_modal = None;
        }
    }

    // Error Modal
    if let Some(ref errs) = state.error_modal {
        let mut close_clicked = false;
        Area::new(Id::new("enum_error_modal"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(t.panel)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(220, 38, 38)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(420.0);
                        ui.heading(
                            RichText::new("Validation Errors")
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                        ui.add_space(8.0);
                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for err in errs {
                                ui.label(RichText::new(format!("• {err}")).size(12.0).color(t.ink));
                            }
                        });
                        ui.add_space(12.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_clicked = true;
                            }
                        });
                    });
            });
        if close_clicked {
            state.error_modal = None;
        }
    }

    // If any drag is active, enforce Grabbing cursor at the end
    if state.value_drag.is_active() || state.enum_drag.is_active() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }

    state.previous_selected_enum = app.selected_enum_def;

    // Save transient state
    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
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

/// Resolves default file path for saving/reverting enum definitions.
pub fn get_default_enum_file_path(app: &ShellApp) -> PathBuf {
    if let Some(ref path) = app.project_path {
        if let Some(parent) = path.parent() {
            return parent.join(format!("{}.{}", app.project.name, ENUM_FILE_EXTENSION));
        }
    }
    PathBuf::from(format!("{}.{}", app.project.name, ENUM_FILE_EXTENSION))
}

fn handle_file_dialog_message(
    app: &mut ShellApp,
    state: &mut EnumViewState,
    msg: EnumFileDialogMessage,
) {
    match msg {
        EnumFileDialogMessage::SaveEnums { path } => {
            let resolver = |tid| app.resolve_tileset_image_size(tid);
            let file_data = convert_project_enums_to_file(
                &app.enum_draft,
                &app.project.defs.tilesets,
                Some(&app.enum_tileset_map),
                &resolver,
            );
            match write_enum_file_atomic(&path, &file_data) {
                Ok(()) => {
                    app.enum_saved = app.enum_draft.clone();
                    app.project.defs.enums = app.enum_draft.clone();
                    app.enum_saved_revision = app.enum_draft_revision;
                    app.enum_saved_tile_rects = app.enum_value_tile_rects.clone();
                    app.enum_unsaved = false;
                    state.toast = Some((
                        format!("Saved all enums to '{}'", path.display()),
                        Instant::now(),
                    ));
                }
                Err(e) => {
                    state.error_modal = Some(vec![format!("Save error: {e}")]);
                }
            }
        }
        EnumFileDialogMessage::ExportAll { path } => {
            let resolver = |tid| app.resolve_tileset_image_size(tid);
            let file_data = convert_project_enums_to_file(
                &app.enum_draft,
                &app.project.defs.tilesets,
                Some(&app.enum_tileset_map),
                &resolver,
            );
            match validate_enum_file_data(&file_data) {
                Ok(_) => match write_enum_file_atomic(&path, &file_data) {
                    Ok(()) => {
                        state.toast = Some((
                            format!("Exported all enums to '{}'", path.display()),
                            Instant::now(),
                        ));
                    }
                    Err(e) => {
                        state.error_modal = Some(vec![format!("Export write error: {e}")]);
                    }
                },
                Err(errs) => {
                    state.error_modal = Some(errs);
                }
            }
        }
        EnumFileDialogMessage::ExportOne { path, enum_id } => {
            if let Some(enum_def) = app.enum_draft.iter().find(|e| e.id == enum_id) {
                let resolver = |tid| app.resolve_tileset_image_size(tid);
                let icon_ts = app.enum_tileset_map.get(&enum_id).copied();
                let file_data = convert_single_enum_to_file(
                    enum_def,
                    &app.project.defs.tilesets,
                    icon_ts,
                    &resolver,
                );
                match validate_enum_file_data(&file_data) {
                    Ok(_) => match write_enum_file_atomic(&path, &file_data) {
                        Ok(()) => {
                            state.toast = Some((
                                format!(
                                    "Exported enum '{}' to '{}'",
                                    enum_def.name,
                                    path.display()
                                ),
                                Instant::now(),
                            ));
                        }
                        Err(e) => {
                            state.error_modal = Some(vec![format!("Export write error: {e}")]);
                        }
                    },
                    Err(errs) => {
                        state.error_modal = Some(errs);
                    }
                }
            }
        }
        EnumFileDialogMessage::Import { path } => {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let is_valid_ext = Path::new(file_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
                .unwrap_or(false);

            if !is_valid_ext {
                state.error_modal = Some(vec![format!(
                    "Unsupported extension: '{file_name}'. Only .fleiyaenum is accepted."
                )]);
                return;
            }

            match read_enum_file(&path) {
                Ok(file_data) => {
                    // Check for identifier conflicts with draft
                    let mut conflicts = Vec::new();
                    for fe in &file_data.enums {
                        if let Some(existing) =
                            app.enum_draft.iter().find(|e| e.name == fe.identifier)
                        {
                            let (entity_count, field_count) =
                                compute_enum_usage(&app.project, existing.id);
                            conflicts.push(EnumConflict {
                                file_enum: fe.clone(),
                                existing_id: existing.id,
                                existing_name: existing.name.clone(),
                                existing_values: existing.values.clone(),
                                entity_count,
                                field_count,
                                choice: ConflictResolutionChoice::KeepBoth,
                            });
                        }
                    }

                    if !conflicts.is_empty() {
                        state.conflict_modal = Some(ConflictModalState {
                            file_data,
                            conflicts,
                            understand_checked: false,
                        });
                    } else {
                        apply_direct_import(app, &file_data, state);
                    }
                }
                Err(errs) => {
                    state.error_modal = Some(errs);
                }
            }
        }
    }
}

fn apply_direct_import(app: &mut ShellApp, file_data: &FleiyaEnumFile, state: &mut EnumViewState) {
    let start_raw_id = app.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1;

    let mut first_new_id = None;
    let tileset_sizes: std::collections::HashMap<lc_core::id::TilesetDefId, (u32, u32)> = app
        .project
        .defs
        .tilesets
        .iter()
        .filter_map(|ts| app.resolve_tileset_image_size(ts.id).map(|sz| (ts.id, sz)))
        .collect();
    let resolver = |tid| tileset_sizes.get(&tid).copied();

    for (next_raw_id, fe) in (start_raw_id..).zip(file_data.enums.iter()) {
        let new_id = EnumDefId(next_raw_id);
        if first_new_id.is_none() {
            first_new_id = Some(new_id);
        }
        let def = convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
        if let Some(uid) = fe.icon_tileset_uid {
            app.enum_tileset_map
                .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
        }
        app.enum_draft.push(def);
    }

    if let Some(sel) = first_new_id {
        app.selected_enum_def = Some(sel);
    }
    app.enum_draft_revision += 1;
    state.toast = Some((
        format!("Imported {} enum(s) into draft", file_data.enums.len()),
        Instant::now(),
    ));
}

fn apply_import_with_conflicts(
    app: &mut ShellApp,
    conflict_state: &ConflictModalState,
    state: &mut EnumViewState,
) {
    let mut existing_names: HashSet<String> =
        app.enum_draft.iter().map(|e| e.name.clone()).collect();
    let mut next_raw_id = app.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1;

    let mut replaced_count = 0;
    let mut renamed_count = 0;
    let mut added_count = 0;
    let mut first_imported_id = None;
    let tileset_sizes: std::collections::HashMap<lc_core::id::TilesetDefId, (u32, u32)> = app
        .project
        .defs
        .tilesets
        .iter()
        .filter_map(|ts| app.resolve_tileset_image_size(ts.id).map(|sz| (ts.id, sz)))
        .collect();
    let resolver = |tid| tileset_sizes.get(&tid).copied();

    for fe in &conflict_state.file_data.enums {
        if let Some(c) = conflict_state
            .conflicts
            .iter()
            .find(|c| c.file_enum.identifier == fe.identifier)
        {
            match c.choice {
                ConflictResolutionChoice::Replace => {
                    let updated_def = convert_file_enum_to_def(
                        fe,
                        c.existing_id,
                        &app.project.defs.tilesets,
                        &resolver,
                    );
                    if let Some(pos) = app.enum_draft.iter().position(|e| e.id == c.existing_id) {
                        if let Some(slot) = app.enum_draft.get_mut(pos) {
                            *slot = updated_def;
                        }
                    }
                    if let Some(uid) = fe.icon_tileset_uid {
                        app.enum_tileset_map
                            .insert(c.existing_id, lc_core::id::TilesetDefId(uid as u64));
                    } else {
                        app.enum_tileset_map.remove(&c.existing_id);
                    }
                    if first_imported_id.is_none() {
                        first_imported_id = Some(c.existing_id);
                    }
                    replaced_count += 1;
                }
                ConflictResolutionChoice::KeepBoth => {
                    let unique_name = generate_unique_identifier(&fe.identifier, &existing_names);
                    existing_names.insert(unique_name.clone());
                    let new_id = EnumDefId(next_raw_id);
                    next_raw_id += 1;
                    let mut def =
                        convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
                    def.name = unique_name;
                    if let Some(uid) = fe.icon_tileset_uid {
                        app.enum_tileset_map
                            .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
                    }
                    app.enum_draft.push(def);
                    if first_imported_id.is_none() {
                        first_imported_id = Some(new_id);
                    }
                    renamed_count += 1;
                }
                ConflictResolutionChoice::Skip => {}
            }
        } else {
            let new_id = EnumDefId(next_raw_id);
            next_raw_id += 1;
            let def = convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
            existing_names.insert(def.name.clone());
            if let Some(uid) = fe.icon_tileset_uid {
                app.enum_tileset_map
                    .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
            }
            app.enum_draft.push(def);
            if first_imported_id.is_none() {
                first_imported_id = Some(new_id);
            }
            added_count += 1;
        }
    }

    if let Some(sel) = first_imported_id {
        app.selected_enum_def = Some(sel);
    }
    app.enum_draft_revision += 1;

    let total = replaced_count + renamed_count + added_count;
    state.toast = Some((
        format!("Imported {total} enums into draft ({replaced_count} replaced, {renamed_count} renamed)"),
        Instant::now(),
    ));
}
