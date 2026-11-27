//! LDtk-parity toggleable `ENTITY VALUES` 2-column fixed-height layout.

use egui::{vec2, Color32, CornerRadius, Frame, Id, Margin, Pos2, RichText, ScrollArea, Sense, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_core::defs::EntityDef;
use lc_core::fields::{FieldDef, FieldType};
use lc_feature_entities::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, ReorderEntityFieldCommand,
    UpdateEntityFieldCommand,
};

use super::field_config_card::render_selected_field_config;
use super::field_type_grid::{
    clear_type_grid_selector_state, field_type_color, render_add_field_buttons,
    render_type_grid_selector,
};
use super::state::EntityViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::context_menu::{show_context_menu_contents, ContextMenuTheme, MenuItem};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::reorder_list::{reorder_list, ListItem};
use crate::shared::ui::reorder_row::{
    GhostPreview, MoveDirection, Slot as BodySlot, Trailing, ROW_LAYOUT_CARD,
};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_COMPACT;

const LIST_BODY_H: f32 = 340.0;
const RIGHT_CARD_H: f32 = CONTROL_HEIGHT_COMPACT + 8.0 + LIST_BODY_H;

fn compact_type_badge(field: &FieldDef) -> String {
    let base = match field.base_type() {
        FieldType::Int => "123",
        FieldType::Float => "1.2",
        FieldType::Bool => "Bit",
        FieldType::String => "Str",
        FieldType::Multilines => "Txt",
        FieldType::Color => "Clr",
        FieldType::Enum(_) => "Enu",
        FieldType::Point => "Pt",
        FieldType::EntityRef => "Ref",
        FieldType::FilePath => "File",
        FieldType::Tile => "Tile",
        FieldType::Array(_) => "Arr",
    };
    if field.is_array() {
        format!("[{base}]")
    } else {
        base.to_string()
    }
}

/// Renders the collapsible `ENTITY VALUES` section with LDtk's 2-column fixed-height split.
pub fn render_entity_fields_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
) {
    if def.fields.is_empty() {
        state.selected_field_idx = None;
    } else if state
        .selected_field_idx
        .is_none_or(|idx| idx >= def.fields.len())
    {
        state.selected_field_idx = Some(0);
    }

    let field_count = def.fields.len();
    let header_title = format!("ENTITY VALUES ({field_count} DEFINED)");

    let mut add_type: Option<FieldType> = None;
    let mut delete_name: Option<String> = None;
    let mut reorder_pair: Option<(usize, usize)> = None;

    let mut collapsed = state.collapse_fields;
    let container = SectionContainer::new(&header_title).use_toggle_collapse(true);
    let header = container.show_header(ui, t, &mut collapsed, |_| {});
    state.collapse_fields = collapsed;

    container.show_body(ui, t, &header, |ui| {
        let total_w = ui.available_width().max(320.0);
        let col_gap = 12.0;
        let left_w = (total_w * 0.36).clamp(220.0, 270.0);
        let right_w = (total_w - left_w - col_gap).max(200.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            // LEFT COLUMN: Equal-width + SINGLE VALUE / + ARRAY buttons + outline-only fixed-height card list
            ui.allocate_ui_with_layout(
                vec2(left_w, RIGHT_CARD_H),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left_w);
                    if let Some(ft) = render_add_field_buttons(ui, app, t, left_w, def.id) {
                        add_type = Some(ft);
                    }
                    ui.add_space(8.0);

                    // Outline-only container (`use_custom_color = false` -> transparent fill + 1px hairline border)
                    Frame::new()
                        .fill(Color32::TRANSPARENT)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            let inner_w = (left_w - 14.0).max(160.0);
                            ui.set_width(inner_w);
                            ui.set_min_height(LIST_BODY_H - 14.0);
                            ui.set_max_height(LIST_BODY_H - 14.0);

                            ScrollArea::vertical()
                                .id_salt(("entity_field_list_scroll", def.id))
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    if def.fields.is_empty() {
                                        ui.add_space(12.0);
                                        ui.label(
                                            RichText::new(
                                                "No fields yet.\nUse + SINGLE VALUE or + ARRAY.",
                                            )
                                            .size(11.5)
                                            .color(t.muted),
                                        );
                                    } else {
                                        render_field_list(
                                            ui,
                                            t,
                                            state,
                                            def,
                                            &mut delete_name,
                                            &mut reorder_pair,
                                        );
                                    }
                                });
                        });
                },
            );

            ui.add_space(col_gap);

            // RIGHT COLUMN: Fixed-height card with default theme fill (`t.panel`) + internal ScrollArea
            ui.allocate_ui_with_layout(
                vec2(right_w, RIGHT_CARD_H),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right_w);
                    Frame::new()
                        .fill(t.panel)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            let inner_w = (right_w - 26.0).max(180.0);
                            let content_w = (inner_w - 8.0).max(170.0);
                            ui.set_width(inner_w);
                            ui.set_max_width(inner_w);
                            ui.set_min_height(RIGHT_CARD_H - 26.0);
                            ui.set_max_height(RIGHT_CARD_H - 26.0);

                            ScrollArea::vertical()
                                .id_salt(("entity_field_config_scroll", def.id))
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(content_w);
                                    ui.set_max_width(content_w);
                                    if let Some(sel_idx) = state.selected_field_idx {
                                        if let Some(field) = def.fields.get(sel_idx) {
                                            render_selected_field_config(
                                                app, ui, t, state, def, sel_idx, field,
                                            );
                                        }
                                    } else {
                                        ui.add_space(12.0);
                                        ui.label(
                                            RichText::new(
                                                "Select or add a field on the left to configure it.",
                                            )
                                            .size(12.0)
                                            .color(t.muted),
                                        );
                                    }
                                });
                        });
                },
            );
        });
    });

    if let Some(ty) = add_type {
        let mut idx = def.fields.len() + 1;
        let mut candidate = format!("field_{idx}");
        while def.field_def(&candidate).is_some() {
            idx += 1;
            candidate = format!("field_{idx}");
        }
        app.execute_command(Box::new(AddEntityFieldCommand::new(
            def.id,
            FieldDef::new(candidate, ty),
        )));
        state.selected_field_idx = Some(def.fields.len());
    }

    if let Some(name) = delete_name.clone() {
        app.execute_command(Box::new(RemoveEntityFieldCommand::new(def.id, name)));
        state.selected_field_idx = None;
        state.field_context_menu = None;
        state.field_context_convert = false;
    }

    if let Some((from, to)) = reorder_pair {
        app.execute_command(Box::new(ReorderEntityFieldCommand::new(def.id, from, to)));
        state.selected_field_idx = Some(to);
        state.field_context_menu = None;
    }

    render_field_context_menu(app, ui, t, state, def);
}

fn render_field_list(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
    delete_name: &mut Option<String>,
    reorder_pair: &mut Option<(usize, usize)>,
) {
    let drag_id = Id::new(("entity_fields_drag", def.id));
    let mut drag_state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<DragListState>(drag_id).unwrap_or_default());

    let items: Vec<ListItem> = def
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let is_selected = state.selected_field_idx == Some(idx);
            ListItem::new(
                format!("field_{}_{}", def.id.0, field.name),
                GhostPreview {
                    label: field.name.clone(),
                    color: is_selected.then_some(t.accent),
                    icon: None,
                },
            )
            .with_slots([BodySlot::Flex, BodySlot::Fixed(54.0)])
            .clickable(true)
            .selected(is_selected)
            .use_custom_color(false)
            .trailing(Trailing::Delete {
                tooltip: Some("Delete field".into()),
                disabled_reason: None,
            })
        })
        .collect();

    let mut child_context_req: Option<(usize, Pos2)> = None;

    let events = reorder_list(
        ui,
        &mut drag_state,
        &items,
        ROW_LAYOUT_CARD,
        |ui, idx, body| {
            let Some(field) = def.fields.get(idx) else {
                return;
            };
            let is_selected = state.selected_field_idx == Some(idx);
            let badge = compact_type_badge(field);

            body.slot(ui, 0, |ui| {
                let text = if field.can_be_null {
                    format!("{}?", field.name)
                } else {
                    field.name.clone()
                };
                let label_resp = ui.add(
                    egui::Label::new(RichText::new(text).strong().color(if is_selected {
                        t.ink_strong
                    } else {
                        t.ink
                    }))
                    .truncate()
                    .sense(Sense::click()),
                );
                if label_resp.clicked() {
                    state.selected_field_idx = Some(idx);
                }
                if label_resp.secondary_clicked() && child_context_req.is_none() {
                    let pos = ui
                        .ctx()
                        .input(|i| i.pointer.interact_pos())
                        .unwrap_or_else(|| label_resp.rect.left_bottom());
                    child_context_req = Some((idx, pos));
                }
            });

            body.slot(ui, 1, |ui| {
                let type_color = field_type_color(field.base_type());
                let (pill_rect, pill_resp) =
                    ui.allocate_exact_size(vec2(50.0, 20.0), Sense::click());
                if ui.is_rect_visible(pill_rect) {
                    let pill_fill = theme::tint(type_color, if is_selected { 0.85 } else { 0.62 });
                    let p = ui.painter();
                    p.rect_filled(pill_rect, CornerRadius::ZERO, pill_fill);
                    p.rect_stroke(
                        pill_rect,
                        CornerRadius::ZERO,
                        egui::Stroke::new(1.0, type_color),
                        egui::StrokeKind::Inside,
                    );
                    let mut child =
                        ui.new_child(egui::UiBuilder::new().max_rect(pill_rect).layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        ));
                    child.add(
                        egui::Label::new(
                            RichText::new(&badge)
                                .size(10.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .selectable(false),
                    );
                }
                let pill_resp = pill_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                if pill_resp.clicked() {
                    state.selected_field_idx = Some(idx);
                }
                if pill_resp.secondary_clicked() && child_context_req.is_none() {
                    let pos = ui
                        .ctx()
                        .input(|i| i.pointer.interact_pos())
                        .unwrap_or_else(|| pill_rect.left_bottom());
                    child_context_req = Some((idx, pos));
                }
            });
        },
    );

    if let Some(sel) = events.selected {
        state.selected_field_idx = Some(sel);
    }
    if let Some(del_idx) = events.delete_requested {
        if let Some(field) = def.fields.get(del_idx) {
            *delete_name = Some(field.name.clone());
        }
    }
    if let Some((from, to)) = events.moved {
        *reorder_pair = Some((from, to));
    }
    if let Some((idx, dir)) = events.move_requested {
        let target = match dir {
            MoveDirection::Up => idx.saturating_sub(1),
            MoveDirection::Down => (idx + 1).min(def.fields.len().saturating_sub(1)),
        };
        if target != idx {
            *reorder_pair = Some((idx, target));
        }
    }
    if let Some((idx, pos)) = events.context_menu_requested.or(child_context_req) {
        state.selected_field_idx = Some(idx);
        state.field_context_menu = Some((def.id, idx, pos));
        state.field_context_convert = false;
        clear_type_grid_selector_state(ui.ctx(), Id::new(("entity_field_ctx_menu", def.id, idx)));
    }

    ui.ctx().data_mut(|d| d.insert_temp(drag_id, drag_state));
}

fn render_field_context_menu(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
) {
    let Some((menu_def_id, field_idx, pos)) = state.field_context_menu else {
        return;
    };
    if menu_def_id != def.id {
        return;
    }
    let Some(field) = def.fields.get(field_idx).cloned() else {
        state.field_context_menu = None;
        state.field_context_convert = false;
        return;
    };

    let popup_id = Id::new(("entity_field_ctx_menu", def.id, field_idx));
    let theme = ContextMenuTheme::default();
    let mut closed = false;
    let mut pending_action: Option<&'static str> = None;
    let mut pending_convert_base: Option<FieldType> = None;

    let area_resp = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            theme.frame(t).show(ui, |ui| {
                if state.field_context_convert {
                    ui.label(
                        RichText::new("CONVERT FIELD TYPE")
                            .size(11.0)
                            .strong()
                            .color(t.accent_ink),
                    );
                    ui.add_space(4.0);
                    let cur_enum = match field.base_type() {
                        FieldType::Enum(id) => Some(*id),
                        _ => None,
                    };
                    let (picked, should_close) =
                        render_type_grid_selector(ui, app, t, popup_id, cur_enum, false);
                    if let Some(new_base) = picked {
                        pending_convert_base = Some(new_base);
                        closed = true;
                    } else if should_close {
                        closed = true;
                    }
                    ui.add_space(4.0);
                    if ui.button("<- Back").clicked() {
                        state.field_context_convert = false;
                        clear_type_grid_selector_state(ui.ctx(), popup_id);
                    }
                } else {
                    let array_label = if field.is_array() {
                        "Convert to Single value"
                    } else {
                        "Convert to Array"
                    };
                    let items = [
                        MenuItem::action("toggle_array", array_label, Icon::Layer),
                        MenuItem::action("convert_type", "Convert type...", Icon::Settings),
                        MenuItem::separator(),
                        MenuItem::action("duplicate", "Duplicate", Icon::Duplicate),
                        MenuItem::destructive("delete", "Delete", Icon::Delete),
                    ];
                    let menu_w = theme.compute_menu_width(ui.ctx(), &items);
                    if let Some(action) = show_context_menu_contents(ui, &theme, t, menu_w, &items)
                    {
                        match action {
                            "convert_type" => {
                                state.field_context_convert = true;
                                clear_type_grid_selector_state(ui.ctx(), popup_id);
                            }
                            "toggle_array" => {
                                pending_action = Some("toggle_array");
                                closed = true;
                            }
                            "duplicate" => {
                                pending_action = Some("duplicate");
                                closed = true;
                            }
                            "delete" => {
                                pending_action = Some("delete");
                                closed = true;
                            }
                            _ => {}
                        }
                    }
                }
            });
        });

    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
        if ui.input(|i| i.pointer.any_pressed()) && !area_resp.response.rect.contains(pointer_pos) {
            closed = true;
        }
    }

    if let Some(new_base) = pending_convert_base {
        let mut updated = field.clone();
        let clean_base = match new_base {
            FieldType::Array(inner) => *inner,
            other => other,
        };
        updated.field_type = if field.is_array() {
            FieldType::Array(Box::new(clean_base))
        } else {
            clean_base
        };
        app.execute_command(Box::new(UpdateEntityFieldCommand::new(
            def.id,
            field.name.clone(),
            updated,
        )));
    } else if let Some(action) = pending_action {
        match action {
            "toggle_array" => {
                let mut updated = field.clone();
                let base = field.base_type().clone();
                updated.field_type = if field.is_array() {
                    base
                } else {
                    FieldType::Array(Box::new(base))
                };
                app.execute_command(Box::new(UpdateEntityFieldCommand::new(
                    def.id,
                    field.name.clone(),
                    updated,
                )));
            }
            "duplicate" => {
                let mut suffix_idx = 2usize;
                let mut candidate = format!("{}_copy", field.name);
                while def.field_def(&candidate).is_some() {
                    candidate = format!("{}_copy_{suffix_idx}", field.name);
                    suffix_idx += 1;
                }
                let mut dup = field.clone();
                dup.name = candidate;
                app.execute_command(Box::new(AddEntityFieldCommand::new(def.id, dup)));
                state.selected_field_idx = Some(def.fields.len());
            }
            "delete" => {
                app.execute_command(Box::new(RemoveEntityFieldCommand::new(
                    def.id,
                    field.name.clone(),
                )));
                state.selected_field_idx = None;
            }
            _ => {}
        }
    }

    if closed {
        state.field_context_menu = None;
        state.field_context_convert = false;
        clear_type_grid_selector_state(ui.ctx(), popup_id);
    }
}
