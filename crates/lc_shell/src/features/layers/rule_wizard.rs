//! Center Area Mode 2: `AUTO-LAYER RULES ASSISTANT` (Batch 2 Screenshots 2–4).
//! Displays IntGrid value pickers + Auto-Rendered Tiles slot template grids on the left,
//! and an interactive Tileset Atlas picker occupying the wide Center canvas on the right.

use egui::{
    pos2, vec2, Button, Color32, ComboBox, CornerRadius, Frame, Margin, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, TextureHandle, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::id::AutoRuleGroupId;
use lc_feature_layers::{
    build_wizard_group, ApplyAutoRuleWizardCommand, AutoRuleWizardPreset, AutoRuleWizardSpec,
    AUTOTILE_ASSISTANT_SLOTS,
};

use super::rule_pattern_widget::draw_tile_in_rect;
use super::state::{CenterAutoMode, LayerViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_COMPACT;

/// Renders the `AUTO-LAYER RULES ASSISTANT` inside the Center Area (`CenterAutoMode::RulesAssistant`).
#[allow(clippy::too_many_arguments)]
pub fn render_center_rules_assistant(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_values: &[IntGridValueDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    if state.assistant.layer_id != def.id {
        state.assistant.layer_id = def.id;
    }

    let mut dummy = false;
    SectionContainer::new("AUTO-LAYER RULES ASSISTANT")
        .use_toggle_collapse(false)
        .show(
            ui,
            t,
            &mut dummy,
            |_| {},
            |ui| {
                let total_w = ui.available_width();
                let left_w = 278.0f32.min((total_w * 0.44).max(240.0));
                let gap = 12.0;
                let atlas_w = (total_w - left_w - gap).max(220.0);

                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    // Left Assistant Form (1. INTGRID VALUES, 2. GROUP NAME, 3. AUTO-RENDERED TILES, 4. OPTIONS)
                    ui.allocate_ui_with_layout(
                        vec2(left_w, ui.available_height().max(420.0)),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(left_w);
                            ui.set_max_width(left_w);
                            render_assistant_left_form(
                                app,
                                ui,
                                t,
                                state,
                                def,
                                source_values,
                                tileset,
                                tileset_tex,
                                interactive,
                            );
                        },
                    );

                    ui.add_space(gap);

                    // Right Wide Tileset Atlas Viewer
                    ui.allocate_ui_with_layout(
                        vec2(atlas_w, ui.available_height().max(420.0)),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(atlas_w);
                            ui.set_max_width(atlas_w);
                            render_assistant_tileset_atlas(
                                ui,
                                t,
                                state,
                                tileset,
                                tileset_tex,
                                interactive,
                            );
                        },
                    );
                });
            },
        );
}

#[allow(clippy::too_many_arguments)]
fn render_assistant_left_form(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_values: &[IntGridValueDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    // 1. INTGRID VALUES
    ui.label(
        RichText::new("1. INTGRID VALUES")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(235, 185, 70)),
    );
    ui.separator();
    ui.label(
        RichText::new("Main IntGrid value")
            .size(11.0)
            .color(t.muted),
    );

    let main_val_obj = source_values
        .iter()
        .find(|v| v.value == state.assistant.int_grid_value);
    let main_lbl = main_val_obj
        .map(|v| format!("{} ({})", v.name, v.value))
        .unwrap_or_else(|| format!("Value ({})", state.assistant.int_grid_value));

    ComboBox::from_id_salt(("asst_main_val", def.id))
        .selected_text(main_lbl)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for v in source_values {
                let c = Color32::from_rgba_unmultiplied(v.color[0], v.color[1], v.color[2], 255);
                let sel = state.assistant.int_grid_value == v.value;
                let row_btn = Button::new(
                    RichText::new(format!("{} ({})", v.name, v.value))
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(c.gamma_multiply(if sel { 0.9 } else { 0.6 }))
                .stroke(if sel {
                    Stroke::new(1.5, Color32::WHITE)
                } else {
                    Stroke::NONE
                })
                .corner_radius(CornerRadius::ZERO);
                if ui.add_sized([220.0, 26.0], row_btn).clicked() {
                    state.assistant.int_grid_value = v.value;
                    if !v.name.is_empty() {
                        state.assistant.group_name = v.name.clone();
                    }
                    ui.close();
                }
            }
        });

    ui.add_space(4.0);
    ui.label(
        RichText::new("Outer IntGrid value")
            .size(11.0)
            .color(t.muted),
    );
    let outer_lbl = state
        .assistant
        .outer_int_grid_value
        .and_then(|ov| source_values.iter().find(|v| v.value == ov))
        .map(|v| format!("{} ({})", v.name, v.value))
        .unwrap_or_else(|| "< Any other IntGrid value >".to_string());
    ComboBox::from_id_salt(("asst_outer_val", def.id))
        .selected_text(outer_lbl)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    state.assistant.outer_int_grid_value.is_none(),
                    "< Any other IntGrid value >",
                )
                .clicked()
            {
                state.assistant.outer_int_grid_value = None;
            }
            for v in source_values {
                let c = Color32::from_rgba_unmultiplied(v.color[0], v.color[1], v.color[2], 255);
                let sel = state.assistant.outer_int_grid_value == Some(v.value);
                let row_btn = Button::new(
                    RichText::new(format!("{} ({})", v.name, v.value))
                        .size(12.0)
                        .color(Color32::WHITE),
                )
                .fill(c.gamma_multiply(if sel { 0.9 } else { 0.6 }))
                .corner_radius(CornerRadius::ZERO);
                if ui.add_sized([220.0, 24.0], row_btn).clicked() {
                    state.assistant.outer_int_grid_value = Some(v.value);
                    ui.close();
                }
            }
        });

    ui.add_space(8.0);
    // 2. GROUP OF RULES NAME
    ui.label(
        RichText::new("2. GROUP OF RULES NAME")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(235, 185, 70)),
    );
    ui.separator();
    ui.add(
        egui::TextEdit::singleline(&mut state.assistant.group_name)
            .desired_width(ui.available_width()),
    );

    ui.add_space(8.0);
    // 3. AUTO-RENDERED TILES
    ui.label(
        RichText::new("3. AUTO-RENDERED TILES")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(235, 185, 70)),
    );
    ui.separator();

    ComboBox::from_id_salt(("asst_preset", def.id))
        .selected_text(state.assistant.preset.label())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for p in [
                AutoRuleWizardPreset::AssistantSlots,
                AutoRuleWizardPreset::SolidBorders3x3,
                AutoRuleWizardPreset::PlatformHorizontal,
                AutoRuleWizardPreset::SimpleFill,
            ] {
                ui.selectable_value(&mut state.assistant.preset, p, p.label());
            }
        });
    ui.add_space(6.0);

    // Side-by-side 4x4 + 2x2 slot grids matching LDtk Batch 2 Screenshot 2
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        render_slot_panel(ui, t, state, 0, 4, 4, tileset, tileset_tex, interactive);
        render_slot_panel(ui, t, state, 1, 2, 2, tileset, tileset_tex, interactive);
    });
    ui.label(
        RichText::new("Click a slot, then click a tile on the Tileset to map it.")
            .size(10.0)
            .color(t.muted),
    );

    ui.add_space(8.0);
    // 4. OPTIONS
    ui.label(
        RichText::new("4. OPTIONS")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(235, 185, 70)),
    );
    ui.separator();
    styled_checkbox(
        ui,
        t,
        &mut state.assistant.allow_stack,
        "Allow rules to stack",
    );

    ui.add_space(10.0);
    // CREATE RULES / CANCEL bar
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let can_create = state.assistant.preset != AutoRuleWizardPreset::AssistantSlots
            || !state.assistant.slot_tiles.is_empty();
        let half_w = ((ui.available_width() - 6.0) * 0.5).max(90.0);
        let create_btn = Button::new(
            RichText::new("CREATE RULES")
                .size(11.5)
                .strong()
                .color(if can_create { t.invert_fg } else { t.muted }),
        )
        .fill(if can_create { t.accent } else { t.sunken })
        .corner_radius(CornerRadius::ZERO);
        if ui
            .add_enabled(
                can_create,
                create_btn.min_size(vec2(half_w, CONTROL_HEIGHT_COMPACT)),
            )
            .clicked()
            && interactive
        {
            let next_gid = AutoRuleGroupId(
                def.auto_rule_groups
                    .iter()
                    .map(|g| g.id.0)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1),
            );
            let next_rid = def
                .auto_rule_groups
                .iter()
                .flat_map(|g| g.rules.iter().map(|r| r.id.0))
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            let spec = AutoRuleWizardSpec {
                group_name: if state.assistant.group_name.trim().is_empty() {
                    "AutoTerrain".to_string()
                } else {
                    state.assistant.group_name.trim().to_string()
                },
                int_grid_value: state.assistant.int_grid_value,
                outer_int_grid_value: state.assistant.outer_int_grid_value,
                preset: state.assistant.preset,
                sample_tiles: state.assistant.sample_tiles.clone(),
                slot_tiles: state.assistant.slot_tiles.clone(),
                allow_stack: state.assistant.allow_stack,
            };
            let grp = build_wizard_group(next_gid, next_rid, &spec);
            let first_rid = grp.rules.first().map(|r| r.id);
            app.execute_command(Box::new(ApplyAutoRuleWizardCommand::new(def.id, grp)));
            state.selected_group_id = Some(next_gid);
            state.selected_rule_id = first_rid;
            state.center_auto_mode = CenterAutoMode::RuleEditor;
        }

        let cancel_btn = Button::new(RichText::new("CANCEL").size(11.5).color(t.muted))
            .fill(t.sunken)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO);
        if ui
            .add_sized([half_w, CONTROL_HEIGHT_COMPACT], cancel_btn)
            .clicked()
            && interactive
        {
            state.center_auto_mode = CenterAutoMode::RuleEditor;
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_slot_panel(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    panel_id: u8,
    cols: u8,
    rows: u8,
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let cell = 28.0;
    let gap = 2.0;
    let total_w = cols as f32 * cell + (cols.saturating_sub(1)) as f32 * gap;
    let total_h = rows as f32 * cell + (rows.saturating_sub(1)) as f32 * gap;
    let (rect, _) = ui.allocate_exact_size(vec2(total_w, total_h), Sense::hover());
    ui.painter()
        .rect_filled(rect.expand(2.0), CornerRadius::ZERO, t.canvas);

    for (idx, slot) in AUTOTILE_ASSISTANT_SLOTS.iter().enumerate() {
        if slot.panel != panel_id {
            continue;
        }
        let r = egui::Rect::from_min_size(
            pos2(
                rect.min.x + slot.col as f32 * (cell + gap),
                rect.min.y + slot.row as f32 * (cell + gap),
            ),
            vec2(cell, cell),
        );
        let resp = ui.interact(
            r,
            egui::Id::new(("asst_slot", panel_id, idx)),
            Sense::click(),
        );
        let is_sel = state.assistant.selected_slot_idx == Some(idx);
        let mapped_tid = state.assistant.slot_tiles.get(&idx).copied();

        {
            let p = ui.painter().with_clip_rect(r);
            p.rect_filled(r, CornerRadius::ZERO, t.sunken);
            if let Some(tid) = mapped_tid {
                draw_tile_in_rect(
                    &p,
                    r.shrink(1.0),
                    tileset,
                    tileset_tex,
                    tid,
                    false,
                    false,
                    Color32::WHITE,
                );
            } else {
                // Draw mini 3x3 neighbor diagram
                let sub = (cell - 6.0) / 3.0;
                for &(px, py, is_inner) in slot.pattern {
                    let sr = egui::Rect::from_min_size(
                        pos2(
                            r.min.x + 3.0 + px as f32 * sub,
                            r.min.y + 3.0 + py as f32 * sub,
                        ),
                        vec2(sub, sub),
                    );
                    p.rect_filled(
                        sr.shrink(0.4),
                        CornerRadius::ZERO,
                        if is_inner {
                            Color32::from_gray(150)
                        } else {
                            Color32::from_rgb(180, 70, 70)
                        },
                    );
                }
            }
            let stroke = if is_sel {
                Stroke::new(2.0, Color32::from_rgb(100, 220, 120))
            } else if resp.hovered() {
                Stroke::new(1.0, t.ink_strong)
            } else {
                t.hairline()
            };
            p.rect_stroke(r, CornerRadius::ZERO, stroke, StrokeKind::Inside);
        }

        if interactive {
            if resp.clicked() {
                state.assistant.selected_slot_idx = Some(idx);
                state.assistant.preset = AutoRuleWizardPreset::AssistantSlots;
            } else if resp.secondary_clicked() {
                state.assistant.slot_tiles.remove(&idx);
            }
        }
        resp.on_hover_text(slot.label);
    }
}

fn render_assistant_tileset_atlas(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let avail = ui.available_size();
    Frame::new()
        .fill(t.canvas)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            let Some(ts) = tileset else {
                ui.label(
                    RichText::new("No tileset bound to this layer.")
                        .size(12.0)
                        .color(t.muted),
                );
                return;
            };
            ui.label(
                RichText::new(format!(
                    "TILESET ATLAS: {} (Click tile to map active slot)",
                    ts.name
                ))
                .size(11.0)
                .strong()
                .color(t.muted),
            );
            ui.add_space(6.0);

            ScrollArea::both()
                .id_salt(("asst_atlas_scroll", ts.id))
                .max_height((avail.y - 38.0).max(300.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let tw = ts.tile_size.x.max(1);
                    let th = ts.tile_size.y.max(1);
                    let (cols, rows) = if let Some(tex) = tileset_tex {
                        let sz = tex.size_vec2();
                        let ew = (sz.x as u32).saturating_sub(ts.padding * 2);
                        let eh = (sz.y as u32).saturating_sub(ts.padding * 2);
                        let c = (ew + ts.spacing)
                            .checked_div(tw + ts.spacing)
                            .unwrap_or(8)
                            .max(1);
                        let r = (eh + ts.spacing)
                            .checked_div(th + ts.spacing)
                            .unwrap_or(8)
                            .max(1);
                        (c, r)
                    } else {
                        (8, 8)
                    };

                    let draw_cell = 28.0f32;
                    let total_sz = vec2(cols as f32 * draw_cell, rows as f32 * draw_cell);
                    let (atlas_rect, _) = ui.allocate_exact_size(total_sz, Sense::hover());
                    draw_checkerboard(ui.painter(), atlas_rect, 12.0, t);

                    for r in 0..rows {
                        for c in 0..cols {
                            let tid = r * cols + c;
                            let cell_r = egui::Rect::from_min_size(
                                pos2(
                                    atlas_rect.min.x + c as f32 * draw_cell,
                                    atlas_rect.min.y + r as f32 * draw_cell,
                                ),
                                vec2(draw_cell, draw_cell),
                            );
                            let resp = ui.interact(
                                cell_r,
                                egui::Id::new(("asst_atlas_tile", ts.id, tid)),
                                Sense::click(),
                            );
                            if ui.is_rect_visible(cell_r) {
                                draw_tile_in_rect(
                                    ui.painter(),
                                    cell_r,
                                    Some(ts),
                                    tileset_tex,
                                    tid,
                                    false,
                                    false,
                                    Color32::WHITE,
                                );
                                let is_mapped =
                                    state.assistant.slot_tiles.values().any(|&v| v == tid);
                                let stroke = if resp.hovered() {
                                    Stroke::new(1.5, Color32::from_rgb(245, 190, 60))
                                } else if is_mapped {
                                    Stroke::new(1.5, Color32::from_rgb(100, 220, 120))
                                } else {
                                    Stroke::new(0.5, Color32::from_white_alpha(20))
                                };
                                ui.painter().rect_stroke(
                                    cell_r,
                                    CornerRadius::ZERO,
                                    stroke,
                                    StrokeKind::Inside,
                                );
                            }
                            if resp.clicked() && interactive {
                                if let Some(slot_idx) = state.assistant.selected_slot_idx {
                                    state.assistant.preset = AutoRuleWizardPreset::AssistantSlots;
                                    state.assistant.slot_tiles.insert(slot_idx, tid);
                                }
                            }
                        }
                    }
                });
        });
}
