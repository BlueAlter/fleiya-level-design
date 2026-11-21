//! Left panel showing enum definitions list, search filtering, and quick actions.

use egui::{
    vec2, Align, CornerRadius, CursorIcon, Frame, Id, Layout, Margin, Panel, Pos2, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui,
};

use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;

use super::left_panel_preview::render_card_drag_preview;
use super::types::{
    compute_enum_usage, draw_six_dot_handle, enum_card_menu_items, reorder_item, DeleteTarget,
    EnumListAction, EnumViewState, MoreMenuAction, DRAG_HANDLE_GAP, DRAG_HANDLE_WIDTH,
    ENUM_CARD_GAP, ENUM_CARD_HEIGHT, ENUM_CARD_PADDING_X, ENUM_CARD_PADDING_Y, KEBAB_ICON_GAP,
    KEBAB_ICON_WIDTH,
};
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::EnumFileDialogManager;
use crate::features::enums::metrics::*;
use crate::shared::ui::context_menu::{is_menu_open, show_context_menu, MenuAnchor, MenuItem};
use crate::shared::ui::drag_list::{paint_placeholder, DragOutcome, DragTheme};
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::reorder::Slot;

pub fn render_left_panel(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut EnumViewState,
    dialog_manager: &std::sync::Arc<EnumFileDialogManager>,
) {
    let t = Tokens::get(ui.ctx());

    Panel::left("enum_mode_left_panel")
        .exact_size(LEFT_PANEL_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(t.panel)
                .inner_margin(Margin::same(WORKSPACE_PADDING as i8)),
        )
        .show(ui, |ui| {
            let inner_w = LEFT_PANEL_WIDTH - 2.0 * WORKSPACE_PADDING;
            ui.set_width(ui.available_width().min(inner_w));
            ui.set_max_width(inner_w);

            // 1. Top action row: "+ New Enum" (stretches) + square 1:1 "..." button
            let btn_h = CONTROL_HEIGHT_COMPACT;
            let gap = 6.0;
            let dots_btn_size = ICON_BUTTON_SIZE_COMPACT;
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
                    app.enum_drafts.insert(
                        next_id,
                        crate::features::editor::shell_app::EnumDraft::new(def.clone()),
                    );
                    app.enum_draft.push(def);
                    app.selected_enum_def = Some(next_id);
                    app.enum_draft_revision += 1;
                }

                // Standardized 1:1 square button (exact 28x28)
                let dots_resp = icon_button(
                    ui,
                    Icon::DotsThreeVertical,
                    dots_btn_size,
                    IconButtonStyle::bordered(),
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
            let tag_input_active = state.tag_editing_id.is_some() || state.tag_inline_new_active;
            let mut pending_list_action: Option<EnumListAction> = None;
            let selected_changed = state.previous_selected_enum != app.selected_enum_def;

            let mut scroll = egui::style::ScrollStyle::thin();
            scroll.floating_allocated_width = 0.0;
            ui.style_mut().spacing.scroll = scroll;

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
                            if handle_resp.drag_started() && !tag_input_active {
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

                            // 3. Truncated label for enum name
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

                            // 4. Kebab button (⋮): separate widget allocated on top of card
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

                            if let Some(act) = show_context_menu(
                                ui,
                                MenuAnchor::button_with_id(&kebab_btn, kebab_popup_id),
                                &card_items,
                            ) {
                                pending_list_action = Some(act);
                            }

                            if let Some(act) = show_context_menu(
                                ui,
                                MenuAnchor::context_menu_with_id(&row_resp, context_popup_id),
                                &card_items,
                            ) {
                                pending_list_action = Some(act);
                            }

                            if row_resp.clicked() && !kebab_btn.clicked() && !handle_resp.dragged()
                            {
                                app.selected_enum_def = Some(e.id);
                            }

                            if selected_changed && is_selected {
                                row_resp.scroll_to_me(Some(Align::Center));
                            }

                            ui.add_space(ENUM_CARD_GAP);
                        }

                        state.card_rects = card_rects.clone();
                        state.delete_btn_rects = delete_btn_rects.clone();
                        state.enum_drag.update(ui, &card_rects, &drag_theme);
                    }
                });

            // Preview for dragged enum card on Order::Tooltip at exact original size
            if let Some(drag_idx) = state.enum_drag.dragging_index() {
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
                            render_card_drag_preview(ui, &t, dragged_enum, ghost_pos, card_w);
                        }
                    }
                }
            }

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
}
