//! LDtk-parity toggleable `ENTITY VALUES` 2-column fixed-height layout.

use egui::{
    vec2, Color32, CornerRadius, Frame, Id, Margin, Pos2, Rect, RichText, ScrollArea, Sense, Ui,
};
use lc_canvas::theme::{self, Tokens};
use lc_core::defs::EntityDef;
use lc_core::fields::{FieldDef, FieldType};
use lc_feature_entities::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, ReorderEntityFieldCommand,
};

use super::field_config_card::render_selected_field_config;
use super::field_type_grid::{field_type_color, render_add_field_buttons};
use super::state::EntityViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::reorder_list::{reorder_list, ListItem};
use crate::shared::ui::reorder_row::{
    GhostPreview, MoveDirection, Slot as BodySlot, Trailing, ROW_LAYOUT_CARD,
};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_COMPACT;

const MIN_CARD_H: f32 = 320.0;
const MAX_CARD_H: f32 = 560.0;

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

/// Renders the collapsible `ENTITY VALUES` section with LDtk's 2-column fixed-width split.
///
/// Column widths are determined strictly from the parent workspace width (never from child content),
/// and column heights track `app_height` so child widgets scroll cleanly inside without shifting panels.
pub fn render_entity_fields_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
    app_height: f32,
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
        let card_h = (app_height - 368.0).clamp(MIN_CARD_H, MAX_CARD_H);
        let list_body_h = (card_h - CONTROL_HEIGHT_COMPACT - 8.0).max(160.0);

        let (row_rect, _) = ui.allocate_exact_size(vec2(total_w, card_h), Sense::hover());
        let left_rect = Rect::from_min_size(row_rect.min, vec2(left_w, card_h));
        let right_rect = Rect::from_min_size(
            Pos2::new(left_rect.max.x + col_gap, row_rect.min.y),
            vec2(right_w, card_h),
        );

        // LEFT COLUMN: Equal-width + SINGLE VALUE / + ARRAY buttons + outline-only fixed card list
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(left_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.set_clip_rect(left_rect.intersect(ui.clip_rect()));
                ui.set_width(left_w);
                ui.set_max_width(left_w);
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
                        ui.set_max_width(inner_w);
                        ui.set_min_height(list_body_h - 14.0);
                        ui.set_max_height(list_body_h - 14.0);

                        ScrollArea::vertical()
                            .id_salt(("entity_field_list_scroll", def.id))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(inner_w);
                                ui.set_max_width(inner_w);
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

        // RIGHT COLUMN: Strict fixed-width card (`right_w`, never driven by child content) + internal ScrollArea
        let inner_rect = right_rect.shrink(12.0);
        let inner_w = inner_rect.width().max(160.0);
        ui.painter().rect(
            right_rect,
            CornerRadius::ZERO,
            t.panel,
            t.hairline(),
            egui::StrokeKind::Inside,
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.set_clip_rect(inner_rect.intersect(ui.clip_rect()));
                ui.set_width(inner_w);
                ui.set_max_width(inner_w);

                ScrollArea::vertical()
                    .id_salt(("entity_field_config_scroll", def.id))
                    .max_width(inner_w)
                    .max_height(inner_rect.height().max(80.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);
                        if let Some(sel_idx) = state.selected_field_idx {
                            if let Some(field) = def.fields.get(sel_idx) {
                                render_selected_field_config(
                                    app, ui, t, state, def, sel_idx, field,
                                );
                            }
                        } else {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("Select or add a field on the left to configure it.")
                                    .size(12.0)
                                    .color(t.muted),
                            );
                        }
                    });
            },
        );
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

    if let Some(name) = delete_name {
        app.execute_command(Box::new(RemoveEntityFieldCommand::new(def.id, name)));
        state.selected_field_idx = None;
    }

    if let Some((from, to)) = reorder_pair {
        app.execute_command(Box::new(ReorderEntityFieldCommand::new(def.id, from, to)));
        state.selected_field_idx = Some(to);
    }
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
                if pill_resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    state.selected_field_idx = Some(idx);
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

    ui.ctx().data_mut(|d| d.insert_temp(drag_id, drag_state));
}
