//! `GENERAL LAYER SETTINGS` collapsible section matching LDtk 1.5.3 `PROJECT LAYERS`.

use egui::{vec2, Button, ComboBox, CornerRadius, DragValue, RichText, Stroke, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::LayerDef;
use lc_core::layer::LayerKind;
use lc_feature_layers::{count_layer_def_usage, UpdateLayerDefCommand};

use super::left_panel::{layer_kind_color, layer_kind_icon, layer_kind_label};
use super::state::LayerViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::text_field::{text_field, validate_identifier, FieldOutline, FieldStyle};
use crate::shared::ui::tileset_select::{tileset_select_full_width, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::{
    card_label, CONTROL_HEIGHT_COMPACT, CONTROL_HEIGHT_FORM,
};

/// Shared aligned attribute row with `FORM_LABEL_COL_W` label column.
pub(crate) fn attr_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        card_label(ui, label, t);
        control(ui)
    })
    .inner
}

/// Renders Section 1 (`GENERAL LAYER SETTINGS`) for `def`.
pub fn render_general_layer_settings(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let mut draft = def.clone();
    let mut changed = false;
    let mut grid_stroke = false;
    let mut requested_kind: Option<LayerKind> = None;
    let mut collapse = state.collapse_settings;

    SectionContainer::new("GENERAL LAYER SETTINGS")
        .use_toggle_collapse(true)
        .show(
            ui,
            t,
            &mut collapse,
            |_| {},
            |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                // 1. Type row (Icon badge + Label + Change Type switcher)
                attr_row(ui, t, "Type", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let kc = layer_kind_color(draft.kind);
                        egui::Frame::new()
                            .fill(t.sunken)
                            .stroke(Stroke::new(1.0, kc.gamma_multiply(0.65)))
                            .corner_radius(CornerRadius::ZERO)
                            .inner_margin(egui::Margin::symmetric(6, 4))
                            .show(ui, |ui| {
                                ui.label(layer_kind_icon(draft.kind).rich(ui.ctx(), 16.0, kc));
                            });
                        ui.label(
                            RichText::new(layer_kind_label(draft.kind))
                                .size(13.0)
                                .strong()
                                .color(t.ink_strong),
                        );

                        ui.add_space(6.0);
                        ComboBox::from_id_salt(("layer_type_switch", def.id))
                            .selected_text("Change type...")
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for kind in [
                                    LayerKind::IntGrid,
                                    LayerKind::Tiles,
                                    LayerKind::Auto,
                                    LayerKind::Entities,
                                ] {
                                    let sel = draft.kind == kind;
                                    if ui.selectable_label(sel, layer_kind_label(kind)).clicked()
                                        && !sel
                                    {
                                        requested_kind = Some(kind);
                                    }
                                }
                            });
                    });
                });

                // 2. Layer identifier
                attr_row(ui, t, "Layer identifier", |ui| {
                    let mut name_buf = draft.name.clone();
                    let other_names: Vec<&str> = app
                        .project
                        .defs
                        .layers
                        .iter()
                        .filter(|other| other.id != draft.id)
                        .map(|other| other.name.as_str())
                        .collect();
                    let id_err_msg = validate_identifier(&name_buf, &other_names, Some(&def.name))
                        .err()
                        .map(|e| e.to_string());
                    let mut style = FieldStyle::default()
                        .with_hint("Layer identifier")
                        .with_outline_preset(FieldOutline::Hairline);
                    if let Some(ref msg) = id_err_msg {
                        style = style.with_error(msg);
                    }
                    let res = text_field(ui, &mut name_buf, style);
                    if (res.committed || res.changed)
                        && validate_identifier(&name_buf, &other_names, Some(&def.name)).is_ok()
                    {
                        let trimmed = name_buf.trim();
                        if trimmed != draft.name {
                            draft.name = trimmed.to_string();
                            changed = true;
                        }
                    }
                });

                // 3. Documentation
                attr_row(ui, t, "Documentation", |ui| {
                    let mut doc_buf = draft.doc.clone().unwrap_or_default();
                    let style = FieldStyle::default()
                        .with_hint("Optional designer note")
                        .with_outline_preset(FieldOutline::Hairline);
                    if text_field(ui, &mut doc_buf, style).changed {
                        let trimmed = doc_buf.trim().to_string();
                        draft.doc = (!trimmed.is_empty()).then_some(trimmed);
                        changed = true;
                    }
                });

                // 4. Default tileset (for Tiles layers, matching LDtk screenshot 7)
                if draft.kind == LayerKind::Tiles {
                    attr_row(ui, t, "Default tileset", |ui| {
                        let ts_items: Vec<TilesetSelectItem<_>> = app
                            .project
                            .defs
                            .tilesets
                            .iter()
                            .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
                            .collect();
                        let mut sel_ts = draft.tileset_id;
                        let combo_w = (ui.available_width() - 95.0).max(160.0);
                        ui.horizontal(|ui| {
                            ui.allocate_ui(vec2(combo_w, CONTROL_HEIGHT_FORM), |ui| {
                                if tileset_select_full_width(ui, &ts_items, &mut sel_ts, true) {
                                    draft.tileset_id = sel_ts;
                                    if let Some(ts_id) = sel_ts {
                                        if let Some(ts) = app.project.defs.tileset(ts_id) {
                                            let stats = count_layer_def_usage(&app.project, def.id);
                                            if stats.placed_items == 0 {
                                                draft.grid_size = ts.tile_size;
                                            }
                                        }
                                    }
                                    changed = true;
                                }
                            });
                            if let Some(ts) =
                                draft.tileset_id.and_then(|id| app.project.defs.tileset(id))
                            {
                                ui.label(
                                    RichText::new(format!("{}px tiles", ts.tile_size.x))
                                        .size(11.0)
                                        .color(t.muted),
                                );
                            }
                        });
                    });
                }

                // 5. Grid size (dimmed/locked when Auto-layer has an IntGrid source)
                let grid_locked =
                    draft.kind == LayerKind::Auto && draft.auto_source_layer_def_id.is_some();
                attr_row(ui, t, "Grid size", |ui| {
                    ui.add_enabled_ui(!grid_locked, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            let mut gx = draft.grid_size.x;
                            if ui
                                .add(
                                    DragValue::new(&mut gx)
                                        .range(1..=1024)
                                        .prefix("W: ")
                                        .suffix(" px"),
                                )
                                .changed()
                            {
                                draft.grid_size.x = gx.max(1);
                                changed = true;
                                grid_stroke = true;
                            }
                            let mut gy = draft.grid_size.y;
                            if ui
                                .add(
                                    DragValue::new(&mut gy)
                                        .range(1..=1024)
                                        .prefix("H: ")
                                        .suffix(" px"),
                                )
                                .changed()
                            {
                                draft.grid_size.y = gy.max(1);
                                changed = true;
                                grid_stroke = true;
                            }
                            for preset in [8u32, 16, 32, 64] {
                                let active =
                                    draft.grid_size.x == preset && draft.grid_size.y == preset;
                                let btn = Button::new(
                                    RichText::new(format!("{preset}"))
                                        .size(11.0)
                                        .color(if active { t.invert_fg } else { t.muted }),
                                )
                                .fill(if active { t.accent } else { t.sunken })
                                .stroke(if active { Stroke::NONE } else { t.hairline() })
                                .corner_radius(CornerRadius::ZERO);
                                if ui.add_sized([30.0, CONTROL_HEIGHT_COMPACT], btn).clicked()
                                    && !active
                                {
                                    draft.grid_size = glam::UVec2::splat(preset);
                                    changed = true;
                                }
                            }
                        });
                    });
                });

                // 6. Offsets
                attr_row(ui, t, "Offsets", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if ui
                            .add(
                                DragValue::new(&mut draft.px_offset.x)
                                    .range(-4096..=4096)
                                    .prefix("X= ")
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                DragValue::new(&mut draft.px_offset.y)
                                    .range(-4096..=4096)
                                    .prefix("Y= ")
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                });

                // 7. Opacity (Base % / Inactive %)
                attr_row(ui, t, "Opacity", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut base_pct = (draft.display_opacity * 100.0).round() as i32;
                        if ui
                            .add(
                                DragValue::new(&mut base_pct)
                                    .range(0..=100)
                                    .prefix("Base: ")
                                    .suffix("%"),
                            )
                            .changed()
                        {
                            draft.display_opacity = (base_pct.clamp(0, 100) as f32) / 100.0;
                            changed = true;
                        }
                        let mut inactive_pct = (draft.inactive_opacity * 100.0).round() as i32;
                        if ui
                            .add(
                                DragValue::new(&mut inactive_pct)
                                    .range(0..=100)
                                    .prefix("Inactive: ")
                                    .suffix("%"),
                            )
                            .changed()
                        {
                            draft.inactive_opacity = (inactive_pct.clamp(0, 100) as f32) / 100.0;
                            changed = true;
                        }
                    });
                });

                // 8. Parallax (X% / Y% + Scale?)
                attr_row(ui, t, "Parallax", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut px_pct = (draft.parallax_factor.x * 100.0).round() as i32;
                        if ui
                            .add(
                                DragValue::new(&mut px_pct)
                                    .range(-100..=100)
                                    .prefix("X= ")
                                    .suffix("%"),
                            )
                            .changed()
                        {
                            draft.parallax_factor.x = (px_pct.clamp(-100, 100) as f32) / 100.0;
                            changed = true;
                        }
                        let mut py_pct = (draft.parallax_factor.y * 100.0).round() as i32;
                        if ui
                            .add(
                                DragValue::new(&mut py_pct)
                                    .range(-100..=100)
                                    .prefix("Y= ")
                                    .suffix("%"),
                            )
                            .changed()
                        {
                            draft.parallax_factor.y = (py_pct.clamp(-100, 100) as f32) / 100.0;
                            changed = true;
                        }
                        if styled_checkbox(ui, t, &mut draft.parallax_scaling, "Scale ?").changed()
                        {
                            changed = true;
                        }
                    });
                });
            },
        );
    state.collapse_settings = collapse;

    // Handle layer kind conversion (confirm if placed data exists in levels)
    if let Some(target_kind) = requested_kind {
        let stats = count_layer_def_usage(&app.project, def.id);
        if stats.placed_items > 0 {
            state.pending_kind_change = Some((
                def.id,
                target_kind,
                stats.non_empty_levels,
                stats.placed_items,
            ));
        } else {
            apply_kind_conversion(&mut draft, target_kind);
            let stroke = def.id.0.wrapping_add(0x4C59_5203);
            app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(draft, stroke)));
            return;
        }
    }

    if let Some((layer_id, target_kind, level_count, item_count)) = state.pending_kind_change {
        if layer_id == def.id {
            #[derive(Clone, Copy, PartialEq, Eq)]
            enum KindModalAction {
                Cancel,
                Convert,
            }
            let buttons = [
                ModalButton::new(KindModalAction::Cancel, "Cancel", ModalButtonKind::Normal),
                ModalButton::new(
                    KindModalAction::Convert,
                    "Convert Layer",
                    ModalButtonKind::Danger,
                ),
            ];
            let res = modal(
                ui.ctx(),
                "layer_kind_change_confirm_modal",
                ModalSpec {
                    title: "Convert Layer Kind?".to_string(),
                    body: |ui| {
                        ui.label(format!(
                            "Changing '{}' from {} to {} will clear {item_count} placed item(s) across {level_count} level(s). You can undo this action.",
                            def.name,
                            layer_kind_label(def.kind),
                            layer_kind_label(target_kind),
                        ));
                    },
                    buttons: &buttons,
                    width: 390.0,
                    default_button: Some(KindModalAction::Cancel),
                },
            );
            match res {
                Some(KindModalAction::Cancel) => state.pending_kind_change = None,
                Some(KindModalAction::Convert) => {
                    state.pending_kind_change = None;
                    apply_kind_conversion(&mut draft, target_kind);
                    let stroke = def.id.0.wrapping_add(0x4C59_5203);
                    app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(
                        draft, stroke,
                    )));
                    return;
                }
                None => {}
            }
        }
    }

    if changed && draft != *def {
        let stroke = if grid_stroke {
            def.id.0.wrapping_add(0x4C59_5202)
        } else {
            def.id.0.wrapping_add(0x4C59_5204)
        };
        app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(draft, stroke)));
    }
}

fn apply_kind_conversion(draft: &mut LayerDef, target_kind: LayerKind) {
    draft.kind = target_kind;
    if target_kind == LayerKind::Entities {
        draft.tileset_id = None;
    }
}
