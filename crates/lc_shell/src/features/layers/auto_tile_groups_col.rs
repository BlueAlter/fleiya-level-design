//! Left `RULES` column (`278px`) for `AUTO TILE` matching LDtk Screenshots 9, 10, & Assistant #1.

use egui::{
    vec2, Button, Color32, CornerRadius, FontId, Frame, Margin, Popup, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextureHandle, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::{AutoLayerRuleDef, AutoLayerRuleGroup, IntGridValueGroupDef};
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::id::{AutoRuleGroupId, AutoRuleId};
use lc_feature_layers::{
    AddAutoRuleCommand, AddAutoRuleGroupCommand, RemoveAutoRuleGroupCommand,
    ReorderAutoRuleGroupsCommand, UpdateAutoRuleGroupCommand,
};

use super::auto_tile_rule_card::render_compact_rule_card;
use super::rule_pattern_widget::{
    condition_color_and_label, draw_tile_in_rect, draw_tileset_rect_in_rect,
};
use super::state::{CenterAutoMode, LayerViewState, RulesAssistantState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_COMPACT;

/// Renders the left `RULES` stack (`+ GROUP` Assistant/Empty menu, `RANDOMIZE`, `Render`, colored groups).
#[allow(clippy::too_many_arguments)]
pub fn render_groups_and_rules_col(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    state.hovered_rule_id = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let grp_btn = Button::new(
            RichText::new(format!("{} GROUP", Icon::Add.glyph()))
                .font(FontId::new(11.5, lc_canvas::icons::family(ui.ctx())))
                .strong()
                .color(t.invert_fg),
        )
        .fill(t.accent)
        .corner_radius(CornerRadius::ZERO);
        let grp_resp = ui.add_sized([86.0, CONTROL_HEIGHT_COMPACT], grp_btn);
        if interactive {
            Popup::menu(&grp_resp).show(|ui| {
                if ui
                    .button(format!(
                        "{} USE ASSISTANT (RECOMMENDED)",
                        Icon::AutoLayer.glyph()
                    ))
                    .clicked()
                {
                    let def_val = source_values.first().map(|v| v.value).unwrap_or(1);
                    let def_name = source_values
                        .first()
                        .map(|v| v.name.clone())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "AutoTerrain".to_string());
                    state.assistant = RulesAssistantState::for_layer(def.id, def_val, &def_name);
                    state.center_auto_mode = CenterAutoMode::RulesAssistant;
                    ui.close();
                }
                if ui
                    .button(format!("{} CREATE AN EMPTY GROUP", Icon::Folder.glyph()))
                    .clicked()
                {
                    let gid = next_group_id(def);
                    let grp = AutoLayerRuleGroup::new(gid, format!("Group_{}", gid.0));
                    app.execute_command(Box::new(AddAutoRuleGroupCommand::new(def.id, grp)));
                    state.selected_group_id = Some(gid);
                    state.selected_rule_id = None;
                    state.center_auto_mode = CenterAutoMode::RuleEditor;
                    ui.close();
                }
            });
        }

        let rand_btn = Button::new(
            RichText::new(format!("{} RANDOMIZE", Icon::Shuffle.glyph()))
                .font(FontId::new(11.0, lc_canvas::icons::family(ui.ctx())))
                .color(t.ink_strong),
        )
        .fill(t.accent.gamma_multiply(0.75))
        .corner_radius(CornerRadius::ZERO);
        if ui
            .add_sized([92.0, CONTROL_HEIGHT_COMPACT], rand_btn)
            .on_hover_text("Reroll auto-layer randomization seed")
            .clicked()
            && interactive
        {
            state.sandbox_seed = state.sandbox_seed.wrapping_add(1).max(1);
        }

        styled_checkbox(ui, t, &mut state.show_render_preview, "Render");
    });

    ui.add_space(4.0);
    let ts_label = tileset
        .map(|ts| format!("{} (DEFAULT)", ts.name))
        .unwrap_or_else(|| "No tileset bound".to_string());
    Frame::new()
        .fill(t.sunken)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(ts_label).size(11.0).color(t.muted));
        });
    ui.add_space(6.0);

    if def.auto_rule_groups.is_empty() {
        Frame::new()
            .fill(t.panel)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(
                        "No rule groups yet. Click '+ GROUP' -> 'USE ASSISTANT' or 'CREATE AN EMPTY GROUP'.",
                    )
                    .size(11.5)
                    .color(t.muted),
                );
            });
        return;
    }

    let max_h = (ui.available_height() - 8.0).max(260.0);
    ScrollArea::vertical()
        .id_salt(("auto_rules_stack_scroll", def.id))
        .max_height(max_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let group_count = def.auto_rule_groups.len();
            for (g_idx, grp) in def.auto_rule_groups.iter().enumerate() {
                render_colored_group_block(
                    app,
                    ui,
                    t,
                    state,
                    def,
                    grp,
                    g_idx,
                    group_count,
                    source_values,
                    source_groups,
                    tileset,
                    tileset_tex,
                    interactive,
                );
                ui.add_space(5.0);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn render_colored_group_block(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    grp: &AutoLayerRuleGroup,
    g_idx: usize,
    group_count: usize,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let is_grp_sel = state.selected_group_id == Some(grp.id);
    let is_collapsed = state.collapsed_rule_groups.contains(&grp.id);
    let grp_col = resolve_group_header_color(t, grp, source_values, source_groups);

    let hdr_fill = if is_grp_sel {
        grp_col.gamma_multiply(0.45)
    } else {
        grp_col.gamma_multiply(0.28)
    };
    Frame::new()
        .fill(hdr_fill)
        .stroke(Stroke::new(
            1.0,
            if is_grp_sel {
                grp_col
            } else {
                grp_col.gamma_multiply(0.55)
            },
        ))
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui
                    .add_enabled(
                        g_idx > 0,
                        Button::new(Icon::ChevronUp.rich(ui.ctx(), 11.0, t.ink_strong))
                            .frame(false),
                    )
                    .clicked()
                    && interactive
                {
                    let mut order: Vec<AutoRuleGroupId> =
                        def.auto_rule_groups.iter().map(|g| g.id).collect();
                    order.swap(g_idx, g_idx - 1);
                    app.execute_command(Box::new(ReorderAutoRuleGroupsCommand::new(def.id, order)));
                }

                if render_switch_pill(ui, t, grp.active, interactive) {
                    let mut upd = grp.clone();
                    upd.active = !grp.active;
                    app.execute_command(Box::new(UpdateAutoRuleGroupCommand::new(def.id, upd)));
                }

                let (thumb_r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                if let Some(tid) = grp
                    .rules
                    .first()
                    .and_then(|r| r.tile_rects.first())
                    .and_then(|tr| tr.tiles.first().copied())
                {
                    draw_tile_in_rect(
                        ui.painter(),
                        thumb_r,
                        tileset,
                        tileset_tex,
                        tid,
                        false,
                        false,
                        Color32::WHITE,
                    );
                } else {
                    draw_tileset_rect_in_rect(app, ui.ctx(), ui.painter(), thumb_r, None, grp_col);
                }

                let label_btn = Button::new(
                    RichText::new(&grp.name)
                        .size(12.0)
                        .strong()
                        .color(if grp.active { t.ink_strong } else { t.muted }),
                )
                .frame(false);
                if ui.add(label_btn).clicked() && interactive {
                    state.selected_group_id = Some(grp.id);
                    if is_collapsed {
                        state.collapsed_rule_groups.remove(&grp.id);
                    } else if is_grp_sel {
                        state.collapsed_rule_groups.insert(grp.id);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    let menu_btn = ui.add(
                        Button::new(Icon::DotsThreeVertical.rich(ui.ctx(), 13.0, t.ink_strong))
                            .frame(false),
                    );
                    if interactive {
                        Popup::menu(&menu_btn).show(|ui| {
                            if ui.button("Add Rule").clicked() {
                                add_rule_to_group(app, state, def, grp.id, source_values);
                                ui.close();
                            }
                            let opt_lbl = if grp.is_optional {
                                "Optional: ON (click to toggle)"
                            } else {
                                "Optional: OFF (click to toggle)"
                            };
                            if ui.button(opt_lbl).clicked() {
                                let mut upd = grp.clone();
                                upd.is_optional = !grp.is_optional;
                                app.execute_command(Box::new(UpdateAutoRuleGroupCommand::new(
                                    def.id, upd,
                                )));
                                ui.close();
                            }
                            if g_idx + 1 < group_count && ui.button("Move Group Down").clicked() {
                                let mut order: Vec<AutoRuleGroupId> =
                                    def.auto_rule_groups.iter().map(|g| g.id).collect();
                                order.swap(g_idx, g_idx + 1);
                                app.execute_command(Box::new(ReorderAutoRuleGroupsCommand::new(
                                    def.id, order,
                                )));
                                ui.close();
                            }
                            if ui.button("Delete Group").clicked() {
                                app.execute_command(Box::new(RemoveAutoRuleGroupCommand::new(
                                    def.id, grp.id,
                                )));
                                ui.close();
                            }
                        });
                    }

                    let add_rule_btn = Button::new(Icon::Add.rich(ui.ctx(), 12.0, t.invert_fg))
                        .fill(t.accent)
                        .corner_radius(CornerRadius::ZERO);
                    if ui
                        .add_sized([20.0, 20.0], add_rule_btn)
                        .on_hover_text("Add rule to this group")
                        .clicked()
                        && interactive
                    {
                        add_rule_to_group(app, state, def, grp.id, source_values);
                    }
                });
            });
        });

    if is_collapsed {
        return;
    }

    for (r_idx, rule) in grp.rules.iter().enumerate() {
        render_compact_rule_card(
            app,
            ui,
            t,
            state,
            def,
            grp,
            rule,
            r_idx,
            grp.rules.len(),
            tileset,
            tileset_tex,
            interactive,
        );
        ui.add_space(2.0);
    }
}

pub(crate) fn render_switch_pill(ui: &mut Ui, t: &Tokens, active: bool, interactive: bool) -> bool {
    let (r, resp) = ui.allocate_exact_size(
        vec2(22.0, 12.0),
        if interactive {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let fill = if active {
        Color32::from_rgb(76, 175, 80)
    } else {
        t.canvas
    };
    let p = ui.painter();
    p.rect_filled(r, CornerRadius::same(6), fill);
    p.rect_stroke(r, CornerRadius::same(6), t.hairline(), StrokeKind::Inside);
    let knob_x = if active { r.max.x - 6.0 } else { r.min.x + 6.0 };
    p.circle_filled(
        egui::pos2(knob_x, r.center().y),
        4.0,
        if active { Color32::WHITE } else { t.muted },
    );
    interactive && resp.clicked()
}

fn add_rule_to_group(
    app: &mut ShellApp,
    state: &mut LayerViewState,
    def: &LayerDef,
    gid: AutoRuleGroupId,
    source_values: &[IntGridValueDef],
) {
    let rid = AutoRuleId(
        def.auto_rule_groups
            .iter()
            .flat_map(|g| g.rules.iter().map(|r| r.id.0))
            .max()
            .unwrap_or(0)
            .saturating_add(1),
    );
    let mut rule = AutoLayerRuleDef::new(rid);
    let dv = source_values.first().map(|v| v.value).unwrap_or(1);
    rule.set_pattern_at(1, 1, dv);
    app.execute_command(Box::new(AddAutoRuleCommand::new(def.id, gid, rule)));
    state.selected_group_id = Some(gid);
    state.selected_rule_id = Some(rid);
    state.center_auto_mode = CenterAutoMode::RuleEditor;
}

fn resolve_group_header_color(
    t: &Tokens,
    grp: &AutoLayerRuleGroup,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
) -> Color32 {
    for r in &grp.rules {
        let half = r.size / 2;
        let cond = r.pattern_at(half, half);
        if cond != 0 {
            let (col, _) = condition_color_and_label(cond, source_values, source_groups);
            if col != Color32::TRANSPARENT {
                return col;
            }
        }
    }
    t.accent
}

fn next_group_id(def: &LayerDef) -> AutoRuleGroupId {
    AutoRuleGroupId(
        def.auto_rule_groups
            .iter()
            .map(|g| g.id.0)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
    )
}
