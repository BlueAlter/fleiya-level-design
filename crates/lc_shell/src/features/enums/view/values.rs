//! Values toolbar, reorderable value rows, bound tile button, color swatch, and text fields.

use egui::{
    vec2, Align, Color32, CornerRadius, CursorIcon, Id, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui,
};

use lc_canvas::theme::Tokens;
use lc_core::defs::EnumValueDef;
use lc_ui_api::AssetService;

use super::types::{reorder_item, DeleteTarget, EnumViewState, CONTROL_HEIGHT};
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::metrics::*;
use crate::shared::ui::drag_list::{paint_placeholder, DragOutcome, DragTheme};
use crate::shared::ui::reorder::Slot as ReorderSlot;
use crate::shared::ui::reorder_row::{
    reorder_row, GhostPreview, ReorderRowSpec, Slot as BodySlot, ROW_LAYOUT_VALUES,
};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::swatch::{color_swatch, tile_icon_button, TileIconData};
use crate::shared::ui::text_field::{text_field, FieldStyle};
use crate::shared::ui::tile_bounds::{TileGrid, TileRect};
use crate::shared::ui::tile_picker::PickerTarget;

pub fn render_values(app: &mut ShellApp, ui: &mut Ui, state: &mut EnumViewState, enum_idx: usize) {
    let t = Tokens::get(ui.ctx());
    let current_enum_id = app.enum_draft[enum_idx].id;
    let tag_input_active = state.tag_editing_id.is_some() || state.tag_inline_new_active;

    // 4. Section: ENUM VALUES Toolbar & Container
    let header_title = format!(
        "ENUM VALUES ({} DEFINED)",
        app.enum_draft[enum_idx].values.len()
    );

    let mut collapse_values = state.collapse_values;
    let container = SectionContainer::new(&header_title)
        .use_toggle_collapse(false)
        .header_height(28.0);
    let header = container.show_header(ui, &t, &mut collapse_values, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add_btn = ui
                .add_sized(
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
                let enum_def = &mut app.enum_draft[enum_idx];
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
    state.collapse_values = collapse_values;

    // 5. Section: Value rows inside the container body
    container.show_body(ui, &t, &header, |ui| {
        ui.set_width(ui.available_width());

        let val_count = app.enum_draft[enum_idx].values.len();
        let uids = state.ensure_value_uids(current_enum_id, val_count).to_vec();
        let mut val_row_rects = Vec::with_capacity(val_count);
        let drag_theme = DragTheme::from_accent(t.accent);
        let row_w = ui.available_width().max(280.0);

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            for slot in state.value_drag.layout(val_count) {
                let idx = match slot {
                    ReorderSlot::Item(i) => i,
                    ReorderSlot::Placeholder => {
                        let ph_h = state.value_drag.row_size().map_or(32.0, |size| size.y);
                        let (ph_rect, _) =
                            ui.allocate_exact_size(vec2(row_w, ph_h), Sense::hover());
                        paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                        val_row_rects.push(ph_rect);
                        ui.add_space(ROW_LAYOUT_VALUES.row_gap);
                        continue;
                    }
                };

                let enum_def = &mut app.enum_draft[enum_idx];
                let Some(val) = enum_def.values.get_mut(idx) else {
                    continue;
                };

                let row_start_y = ui.cursor().min.y;
                let val_color =
                    Color32::from_rgba_premultiplied(val.color[0], val.color[1], val.color[2], 255);

                let mut spec = ReorderRowSpec::new(uids[idx], &val.id);
                spec.height = ROW_LAYOUT_VALUES.height;
                spec.layout = ROW_LAYOUT_VALUES;
                spec.outline = Some(val_color);
                spec.ghost = GhostPreview {
                    label: val.id.clone(),
                    color: Some(val_color),
                    icon: None,
                };

                let enum_ts_id = enum_def
                    .tileset_id
                    .or_else(|| val.tile.map(|t| t.tileset_id));
                let tile_data = if let Some(ref tr) = val.tile {
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
                                let cols =
                                    (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                                let rows =
                                    (eff_h + spacing).checked_div(stride_y).unwrap_or(1).max(1);
                                let grid =
                                    TileGrid::new(tile_w, tile_h, spacing, padding, cols, rows);

                                let tile_rect = app
                                    .enum_value_tile_rects
                                    .get(&(current_enum_id, val.id.clone()))
                                    .copied()
                                    .unwrap_or_else(|| {
                                        TileRect::single(tr.tile_id % cols, tr.tile_id / cols)
                                    });

                                let region_page = tile_rect.union_rect(&grid);
                                let u0 = region_page.min.x / img_w as f32;
                                let v0 = region_page.min.y / img_h as f32;
                                let u1 = region_page.max.x / img_w as f32;
                                let v1 = region_page.max.y / img_h as f32;
                                Some(TileIconData::new(
                                    tex.id(),
                                    Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let slots = [
                    BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                    BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                    BodySlot::Flex,
                ];

                let mut tile_clicked = false;
                let mut tile_secondary_clicked = false;
                let mut color_changed = false;
                let mut name_changed = false;
                let mut new_color = val_color;
                let mut new_name = val.id.clone();
                let val_id_clone = val.id.clone();

                let row_resp = reorder_row(ui, spec, &slots, |ui, body| {
                    // Slot 0: Tile icon button
                    body.slot(ui, 0, |ui| {
                        let tile_btn = tile_icon_button(ui, tile_data);
                        if tile_btn.secondary_clicked()
                            || (tile_btn.hovered()
                                && ui.input(|i| {
                                    i.pointer.button_clicked(egui::PointerButton::Secondary)
                                }))
                        {
                            tile_secondary_clicked = true;
                        } else if tile_btn.clicked() {
                            tile_clicked = true;
                        }
                    });

                    // Slot 1: Color swatch
                    body.slot(ui, 1, |ui| {
                        let swatch_resp =
                            color_swatch(ui, Id::new(uids[idx]).with("color"), &mut new_color);
                        if swatch_resp.changed() {
                            color_changed = true;
                        }
                    });

                    // Slot 2: Identifier text field with outline colored in val_color
                    body.slot(ui, 2, |ui| {
                        let style = FieldStyle::default()
                            .with_hint("Value Identifier")
                            .with_outline(false);
                        let resp = text_field(ui, &mut new_name, style);
                        if resp.changed {
                            name_changed = true;
                        }
                        ui.painter().rect_stroke(
                            resp.response.rect,
                            CornerRadius::same(3),
                            Stroke::new(1.5, val_color),
                            StrokeKind::Inside,
                        );
                    });
                });

                val_row_rects.push(Rect::from_min_size(
                    Pos2::new(row_resp.rect.min.x, row_start_y),
                    vec2(row_resp.rect.width(), ROW_LAYOUT_VALUES.height),
                ));

                if row_resp.handle.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
                if row_resp.handle.drag_started() && !tag_input_active {
                    state.value_drag.begin(ui.ctx(), idx, row_resp.rect);
                    ui.memory_mut(|m| {
                        if let Some(fid) = m.focused() {
                            m.surrender_focus(fid);
                        }
                        m.stop_text_input();
                    });
                }

                if row_resp.delete_clicked {
                    state.pending_delete = Some(DeleteTarget::EnumValue {
                        enum_id: current_enum_id,
                        val_id: val_id_clone.clone(),
                        name: val_id_clone.clone(),
                    });
                }

                if tile_secondary_clicked {
                    let val_mut = &mut app.enum_draft[enum_idx].values[idx];
                    if val_mut.tile.is_some() {
                        val_mut.tile = None;
                        app.enum_value_tile_rects
                            .remove(&(current_enum_id, val_id_clone.clone()));
                        app.enum_draft_revision += 1;
                    }
                } else if tile_clicked {
                    let active_ts = app.enum_draft[enum_idx].tileset_id.or_else(|| {
                        app.enum_draft[enum_idx].values[idx]
                            .tile
                            .map(|t| t.tileset_id)
                    });
                    let current_rect = app
                        .enum_value_tile_rects
                        .get(&(current_enum_id, val_id_clone.clone()))
                        .copied()
                        .or_else(|| {
                            app.enum_draft[enum_idx].values[idx]
                                .tile
                                .map(|t| TileRect::single(t.tile_id % 16, t.tile_id / 16))
                        });
                    state.tile_picker.open_target(
                        Some(PickerTarget {
                            enum_id: current_enum_id,
                            value_id: val_id_clone.clone(),
                        }),
                        active_ts,
                        current_rect,
                    );
                }

                if color_changed {
                    let val_mut = &mut app.enum_draft[enum_idx].values[idx];
                    val_mut.color = [new_color.r(), new_color.g(), new_color.b(), 255];
                    app.enum_draft_revision += 1;
                }

                if name_changed {
                    let val_mut = &mut app.enum_draft[enum_idx].values[idx];
                    val_mut.id = new_name.clone();
                    val_mut.name = new_name;
                    app.enum_draft_revision += 1;
                }

                ui.add_space(ROW_LAYOUT_VALUES.row_gap);
            }

            state.value_drag.update(ui, &val_row_rects, &drag_theme);
        });
    });

    // Resolve the value drag: drop on release, cancel on Escape
    if let DragOutcome::Dropped { from, to } = state.value_drag.finish(ui) {
        let enum_def = &mut app.enum_draft[enum_idx];
        if reorder_item(&mut enum_def.values, from, to) {
            if let Some(uids) = state.value_uids.get_mut(&enum_def.id) {
                reorder_item(uids, from, to);
            }
            app.enum_draft_revision += 1;
        }
    }
}
