//! `INTGRID SETTINGS` section matching LDtk Screenshot 6 (`Values interface icons`, framed `Grid values`, tile icons, groups, and Auto-layer bindings).

use egui::{pos2, vec2, Button, Color32, ComboBox, CornerRadius, Id, Rect, RichText, Stroke, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::IntGridValueGroupDef;
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetRect};
use lc_feature_layers::{next_int_grid_value_id, UpdateLayerDefCommand};
use lc_ui_api::AssetService;

use super::attributes_card::attr_row;
use super::state::LayerViewState;
use super::type_settings_card::render_layer_ops_modals;
use super::visuals_card::render_autolayer_bindings;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::reorder_list::{reorder_list, ListItem};
use crate::shared::ui::reorder_row::{GhostPreview, Slot as BodySlot, Trailing, ROW_LAYOUT_VALUES};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::swatch::{color_swatch, tile_icon_button, TileIconData};
use crate::shared::ui::text_field::{text_field, FieldOutline, FieldStyle};
use crate::shared::ui::tile_picker::{TilePicker, TilePickerEvent, TilesetData};
use crate::shared::ui::tileset_select::{tileset_select_full_width, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, CONTROL_HEIGHT_FORM};

const INTGRID_PALETTE: [[u8; 4]; 6] = [
    [96, 165, 250, 255],
    [240, 160, 60, 255],
    [148, 200, 120, 255],
    [239, 68, 68, 255],
    [168, 85, 247, 255],
    [61, 214, 184, 255],
];

/// Resolves a [`TilesetRect`] into a [`TileIconData`] texture + UV rect for swatches.
pub(crate) fn resolve_tile_icon_data(
    app: &ShellApp,
    ctx: &egui::Context,
    rect: Option<TilesetRect>,
) -> Option<TileIconData> {
    let r = rect?;
    let ts = app.project.defs.tileset(r.tileset_id)?;
    let tex = app.assets.load_texture(ctx, &ts.path).ok()?;
    let sz = tex.size_vec2();
    if sz.x <= 0.0 || sz.y <= 0.0 {
        return None;
    }
    let uv = Rect::from_min_max(
        pos2(r.x as f32 / sz.x, r.y as f32 / sz.y),
        pos2((r.x + r.w) as f32 / sz.x, (r.y + r.h) as f32 / sz.y),
    );
    Some(TileIconData::new(tex.id(), uv))
}

/// Renders Section 3 (`INTGRID SETTINGS`) for an `IntGrid` layer definition.
pub fn render_intgrid_settings_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let mut draft = def.clone();
    let mut changed = false;
    let mut add_clicked = false;
    let mut add_group_clicked = false;
    let mut collapse = state.collapse_kind_config;

    SectionContainer::new("INTGRID SETTINGS")
        .use_toggle_collapse(true)
        .show(
            ui,
            t,
            &mut collapse,
            |_| {},
            |ui| {
                // 1. Values interface icons tileset
                attr_row(ui, t, "Values interface icons", |ui| {
                    let ts_items: Vec<TilesetSelectItem<_>> = app
                        .project
                        .defs
                        .tilesets
                        .iter()
                        .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
                        .collect();
                    let mut sel_icons = draft.int_grid_icons_tileset_id;
                    if tileset_select_full_width(ui, &ts_items, &mut sel_icons, true) {
                        draft.int_grid_icons_tileset_id = sel_icons;
                        changed = true;
                    }
                });
                ui.add_space(6.0);

                // 2. Framed Grid values table (LDtk Screenshot 6)
                let icon_ts_id = draft.effective_int_grid_icons_tileset();
                attr_row(ui, t, "Grid values", |ui| {
                    egui::Frame::new()
                        .fill(t.canvas)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width());
                                render_grid_values_inner(
                                    app,
                                    ui,
                                    t,
                                    state,
                                    def,
                                    &mut draft,
                                    &mut changed,
                                    &mut add_clicked,
                                    &mut add_group_clicked,
                                    icon_ts_id,
                                );
                            });
                        });
                });
                ui.add_space(8.0);
                render_autolayer_bindings(app, ui, t, state, &mut draft, &mut changed, false);
            },
        );
    state.collapse_kind_config = collapse;

    if add_group_clicked {
        let next_uid = draft
            .int_grid_value_groups
            .iter()
            .map(|g| g.uid)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let color = INTGRID_PALETTE[(next_uid as usize) % INTGRID_PALETTE.len()];
        let mut grp = IntGridValueGroupDef::new(next_uid, format!("Group_{next_uid}"));
        grp.color = Some(color);
        draft.int_grid_value_groups.push(grp);
        changed = true;
    }

    if add_clicked {
        let next_val = next_int_grid_value_id(&draft.int_grid_values);
        let color = INTGRID_PALETTE[((next_val - 1) as usize) % INTGRID_PALETTE.len()];
        draft.int_grid_values.push(IntGridValueDef {
            value: next_val,
            name: format!("Value_{next_val}"),
            color,
            ..Default::default()
        });
        changed = true;
    }

    if changed && draft != *def {
        let stroke = def.id.0.wrapping_add(0x4C59_5205);
        app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(draft, stroke)));
    }

    render_intgrid_tile_picker(app, ui, state, def);
    render_layer_ops_modals(app, ui, state, def);
}

#[allow(clippy::too_many_arguments)]
fn render_grid_values_inner(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    draft: &mut LayerDef,
    changed: &mut bool,
    add_clicked: &mut bool,
    add_group_clicked: &mut bool,
    icon_ts_id: Option<lc_core::id::TilesetDefId>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let folder_w = 40.0;
        let plus_w = (ui.available_width() - folder_w - 6.0).max(80.0);
        let top_plus = Button::new(Icon::Add.rich(ui.ctx(), 14.0, t.ink_strong))
            .fill(t.sunken)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO);
        if ui
            .add_sized([plus_w, CONTROL_HEIGHT_COMPACT], top_plus)
            .on_hover_text("Add IntGrid value")
            .clicked()
        {
            *add_clicked = true;
        }
        let grp_btn = Button::new(Icon::Folder.rich(ui.ctx(), 14.0, t.ink_strong))
            .fill(t.sunken)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO);
        if ui
            .add_sized([folder_w, CONTROL_HEIGHT_COMPACT], grp_btn)
            .on_hover_text("Create IntGrid value group")
            .clicked()
        {
            *add_group_clicked = true;
        }
    });

    if !draft.int_grid_value_groups.is_empty() {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let mut remove_group_uid: Option<u32> = None;
            for grp in &mut draft.int_grid_value_groups {
                let gcol = grp
                    .color
                    .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], 255))
                    .unwrap_or(t.accent);
                egui::Frame::new()
                    .fill(t.sunken)
                    .stroke(Stroke::new(1.0, gcol.gamma_multiply(0.6)))
                    .corner_radius(CornerRadius::same(3))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let mut c = gcol;
                            if color_swatch(
                                ui,
                                Id::new(("intgrid_grp_col", def.id, grp.uid)),
                                &mut c,
                            )
                            .changed()
                            {
                                grp.color = Some([c.r(), c.g(), c.b(), 255]);
                                *changed = true;
                            }
                            if ui
                                .add(egui::TextEdit::singleline(&mut grp.name).desired_width(78.0))
                                .changed()
                            {
                                *changed = true;
                            }
                            if ui.small_button("×").clicked() {
                                remove_group_uid = Some(grp.uid);
                            }
                        });
                    });
            }
            if let Some(uid) = remove_group_uid {
                draft.int_grid_value_groups.retain(|g| g.uid != uid);
                for v in &mut draft.int_grid_values {
                    if v.group_uid == Some(uid) {
                        v.group_uid = None;
                    }
                }
                *changed = true;
            }
        });
    }

    ui.add_space(6.0);

    if draft.int_grid_values.is_empty() {
        ui.label(
            RichText::new("No IntGrid values defined yet.")
                .size(11.5)
                .color(t.muted),
        );
    } else {
        let group_snapshot = draft.int_grid_value_groups.clone();
        let icon_previews: Vec<Option<TileIconData>> = draft
            .int_grid_values
            .iter()
            .map(|v| resolve_tile_icon_data(app, ui.ctx(), v.tile))
            .collect();

        let list_items: Vec<ListItem> = draft
            .int_grid_values
            .iter()
            .map(|val| {
                let col =
                    Color32::from_rgba_unmultiplied(val.color[0], val.color[1], val.color[2], 255);
                let mut ghost = GhostPreview::new(&val.name);
                ghost.color = Some(col);
                let item = ListItem::new(("intgrid_val", def.id, val.value), ghost);
                let item = if group_snapshot.is_empty() {
                    item.with_slots([
                        BodySlot::Fixed(40.0),
                        BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                        BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                        BodySlot::Flex,
                    ])
                } else {
                    item.with_slots([
                        BodySlot::Fixed(40.0),
                        BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                        BodySlot::Fixed(CONTROL_HEIGHT_FORM),
                        BodySlot::Flex,
                        BodySlot::Fixed(96.0),
                    ])
                };
                item.trailing(Trailing::Delete {
                    tooltip: Some("Delete IntGrid value".to_string()),
                    disabled_reason: None,
                })
            })
            .collect();

        let mut open_tile_idx: Option<usize> = None;
        let events = reorder_list(
            ui,
            &mut state.intgrid_drag,
            &list_items,
            ROW_LAYOUT_VALUES,
            |ui, idx, body| {
                let Some(val) = draft.int_grid_values.get_mut(idx) else {
                    return;
                };
                let val_color =
                    Color32::from_rgba_unmultiplied(val.color[0], val.color[1], val.color[2], 255);

                // Slot 0: Value number badge
                body.slot(ui, 0, |ui| {
                    let (badge_rect, _) = ui
                        .allocate_exact_size(vec2(40.0, CONTROL_HEIGHT_FORM), egui::Sense::hover());
                    if ui.is_rect_visible(badge_rect) {
                        let p = ui.painter();
                        p.rect_filled(
                            badge_rect,
                            CornerRadius::same(3),
                            val_color.gamma_multiply(0.18),
                        );
                        p.rect_stroke(
                            badge_rect,
                            CornerRadius::same(3),
                            Stroke::new(1.0, val_color.gamma_multiply(0.65)),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", val.value),
                            egui::FontId::monospace(11.5),
                            t.ink_strong,
                        );
                    }
                });

                // Slot 1: Color swatch
                body.slot(ui, 1, |ui| {
                    let mut col = val_color;
                    if color_swatch(ui, Id::new(("intgrid_swatch", def.id, val.value)), &mut col)
                        .changed()
                    {
                        val.color = [col.r(), col.g(), col.b(), 255];
                        *changed = true;
                    }
                });

                // Slot 2: Interface tile icon button
                body.slot(ui, 2, |ui| {
                    let preview = icon_previews.get(idx).copied().flatten();
                    let resp = tile_icon_button(ui, preview);
                    if resp.clicked() && icon_ts_id.is_some() {
                        open_tile_idx = Some(idx);
                    } else if resp.secondary_clicked() && val.tile.is_some() {
                        val.tile = None;
                        *changed = true;
                    }
                });

                // Slot 3: Value name field tinted with value color
                body.slot(ui, 3, |ui| {
                    let mut name_buf = val.name.clone();
                    let style = FieldStyle::default()
                        .with_hint("IntGrid value name")
                        .with_outline_preset(FieldOutline::Color(val_color));
                    if text_field(ui, &mut name_buf, style).changed {
                        val.name = name_buf;
                        *changed = true;
                    }
                });

                // Optional Slot 4: Group assignment
                if !group_snapshot.is_empty() {
                    body.slot(ui, 4, |ui| {
                        let grp_label = val
                            .group_uid
                            .and_then(|uid| group_snapshot.iter().find(|g| g.uid == uid))
                            .map(|g| g.name.as_str())
                            .unwrap_or("No group");
                        ComboBox::from_id_salt(("intgrid_val_grp", def.id, val.value))
                            .selected_text(grp_label)
                            .width(92.0)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(val.group_uid.is_none(), "No group")
                                    .clicked()
                                {
                                    val.group_uid = None;
                                    *changed = true;
                                }
                                for g in &group_snapshot {
                                    if ui
                                        .selectable_label(val.group_uid == Some(g.uid), &g.name)
                                        .clicked()
                                    {
                                        val.group_uid = Some(g.uid);
                                        *changed = true;
                                    }
                                }
                            });
                    });
                }
            },
        );

        if let Some(idx) = open_tile_idx {
            if let Some(ts_id) = icon_ts_id {
                let mut picker = TilePicker::new();
                picker.open(None, Some(ts_id), None);
                state.intgrid_tile_picker = Some((def.id, idx, picker));
            }
        }

        if let Some((from, to)) = events.moved {
            if from < draft.int_grid_values.len() && to < draft.int_grid_values.len() && from != to
            {
                let moved = draft.int_grid_values.remove(from);
                draft.int_grid_values.insert(to, moved);
                *changed = true;
            }
        }

        if let Some(idx) = events.delete_requested {
            if idx < draft.int_grid_values.len() {
                draft.int_grid_values.remove(idx);
                *changed = true;
            }
        }
    }

    ui.add_space(6.0);
    let bottom_plus = Button::new(Icon::Add.rich(ui.ctx(), 14.0, t.invert_fg))
        .fill(t.accent)
        .corner_radius(CornerRadius::ZERO);
    if ui
        .add_sized(
            [ui.available_width().max(80.0), CONTROL_HEIGHT_COMPACT],
            bottom_plus,
        )
        .on_hover_text("Add IntGrid value")
        .clicked()
    {
        *add_clicked = true;
    }
}

fn render_intgrid_tile_picker(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let Some((layer_id, val_idx, mut picker)) = state.intgrid_tile_picker.take() else {
        return;
    };
    if layer_id != def.id {
        return;
    }
    let Some(ts_id) = picker.tileset_id else {
        return;
    };
    let Some(ts) = app.project.defs.tileset(ts_id).cloned() else {
        return;
    };
    let ts_data = TilesetData::from(&ts);
    let current_sel = picker.selected_tile_rect;
    let mut keep_open = true;

    match picker.show(ui.ctx(), &ts_data, current_sel) {
        TilePickerEvent::Selected(rect) => {
            if let Some(val) = def.int_grid_values.get(val_idx) {
                let mut draft = def.clone();
                if let Some(slot) = draft.int_grid_values.get_mut(val_idx) {
                    let tw = ts.tile_size.x.max(1);
                    let th = ts.tile_size.y.max(1);
                    let px = ts.padding + rect.col * (tw + ts.spacing);
                    let py = ts.padding + rect.row * (th + ts.spacing);
                    slot.tile = Some(TilesetRect::new(
                        ts_id,
                        px,
                        py,
                        tw * rect.w.max(1),
                        th * rect.h.max(1),
                    ));
                    let _ = val;
                    app.execute_command(Box::new(UpdateLayerDefCommand::new(draft)));
                }
            }
            keep_open = false;
        }
        TilePickerEvent::Closed => keep_open = false,
        TilePickerEvent::None => {}
    }

    if keep_open && picker.is_open {
        state.intgrid_tile_picker = Some((layer_id, val_idx, picker));
    }
}
