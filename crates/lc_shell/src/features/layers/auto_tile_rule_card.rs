//! Compact LDtk Rule Card (`36x36` thumbnail + Phosphor-icon micro-badges `🎲`, `💧`, `✋`, `[X]`, `[Y]`, `[❖]`, `[1-1]`, `[P]`).

use egui::{
    vec2, Button, Color32, CornerRadius, DragValue, FontId, Frame, Margin, Popup, RichText, Sense,
    Stroke, StrokeKind, TextureHandle, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::{AutoLayerRuleDef, AutoLayerRuleGroup, AutoRuleCheckerMode};
use lc_core::defs::{LayerDef, TilesetDef};
use lc_core::id::AutoRuleId;
use lc_feature_layers::{RemoveAutoRuleCommand, ReorderAutoRulesCommand, UpdateAutoRuleCommand};

use super::auto_tile_groups_col::render_switch_pill;
use super::rule_pattern_widget::draw_tile_in_rect;
use super::state::{CenterAutoMode, LayerViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::tile_picker::TilePicker;

/// Renders one compact LDtk-style Rule Card inside an expanded rule group.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_compact_rule_card(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    grp: &AutoLayerRuleGroup,
    rule: &AutoLayerRuleDef,
    r_idx: usize,
    rule_count: usize,
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let is_sel = state.selected_group_id == Some(grp.id) && state.selected_rule_id == Some(rule.id);
    let card_bg = if is_sel { t.panel } else { t.sunken };
    let card_stroke = if is_sel {
        Stroke::new(1.5, Color32::from_rgb(245, 190, 60))
    } else {
        t.hairline()
    };

    let card_resp = Frame::new()
        .fill(card_bg)
        .stroke(card_stroke)
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::symmetric(5, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    if ui
                        .add_enabled(
                            r_idx > 0,
                            Button::new(Icon::ChevronUp.rich(ui.ctx(), 9.0, t.muted)).frame(false),
                        )
                        .clicked()
                        && interactive
                    {
                        let mut order: Vec<AutoRuleId> = grp.rules.iter().map(|r| r.id).collect();
                        order.swap(r_idx, r_idx - 1);
                        app.execute_command(Box::new(ReorderAutoRulesCommand::new(
                            def.id, grp.id, order,
                        )));
                    }
                    if render_switch_pill(ui, t, rule.active, interactive) {
                        let mut upd = rule.clone();
                        upd.active = !rule.active;
                        app.execute_command(Box::new(UpdateAutoRuleCommand::new(
                            def.id, grp.id, upd,
                        )));
                    }
                });

                // Square output tile thumbnail (`36x36`)
                let (thumb_rect, thumb_resp) =
                    ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());
                {
                    let p = ui.painter().with_clip_rect(thumb_rect);
                    p.rect_filled(thumb_rect, CornerRadius::ZERO, t.canvas);
                    if let Some(tid) = rule.tile_rects.first().and_then(|tr| tr.tiles.first()) {
                        draw_tile_in_rect(
                            &p,
                            thumb_rect.shrink(1.0),
                            tileset,
                            tileset_tex,
                            *tid,
                            rule.flip_x,
                            rule.flip_y,
                            if rule.active {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(120)
                            },
                        );
                    }
                    let tb = if is_sel {
                        Stroke::new(1.5, Color32::from_rgb(245, 190, 60))
                    } else {
                        t.hairline()
                    };
                    p.rect_stroke(thumb_rect, CornerRadius::ZERO, tb, StrokeKind::Inside);
                }
                if thumb_resp.clicked() && interactive {
                    state.selected_group_id = Some(grp.id);
                    state.selected_rule_id = Some(rule.id);
                    state.center_auto_mode = CenterAutoMode::RuleEditor;
                }
                if thumb_resp.double_clicked() && interactive {
                    if let Some(ts_id) = def.tileset_id {
                        let mut picker = TilePicker::new();
                        picker.open(None, Some(ts_id), None);
                        state.auto_tile_picker = Some((def.id, grp.id, rule.id, picker));
                    }
                }

                // 2-row micro-badges strip (Phosphor Icons)
                let mut draft = rule.clone();
                let mut changed = false;
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.label(Icon::Dice.rich(ui.ctx(), 12.0, t.muted));
                        let mut pct = (draft.chance * 100.0).round() as i32;
                        if ui
                            .add(DragValue::new(&mut pct).range(0..=100).speed(1.0))
                            .on_hover_text("Chance (%)")
                            .changed()
                        {
                            draft.chance = (pct.clamp(0, 100) as f32) / 100.0;
                            changed = true;
                        }

                        ui.label(Icon::Drop.rich(ui.ctx(), 11.5, t.muted));
                        if ui
                            .add(
                                DragValue::new(&mut draft.alpha)
                                    .range(0.0..=1.0)
                                    .speed(0.02),
                            )
                            .on_hover_text("Tile opacity (alpha)")
                            .changed()
                        {
                            draft.alpha = draft.alpha.clamp(0.0, 1.0);
                            changed = true;
                        }

                        let brk_fill = if draft.break_on_match {
                            Color32::from_rgb(70, 130, 210)
                        } else {
                            t.canvas
                        };
                        let brk_fg = if draft.break_on_match {
                            Color32::WHITE
                        } else {
                            t.faint
                        };
                        let brk_btn = Button::new(Icon::HandPalm.rich(ui.ctx(), 11.5, brk_fg))
                            .fill(brk_fill)
                            .stroke(t.hairline())
                            .corner_radius(CornerRadius::ZERO);
                        if ui
                            .add_sized([22.0, 18.0], brk_btn)
                            .on_hover_text("Break on match (stop cascade if matched)")
                            .clicked()
                            && interactive
                        {
                            draft.break_on_match = !draft.break_on_match;
                            changed = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        if micro_icon_toggle(
                            ui,
                            t,
                            Icon::FlipX,
                            "X",
                            draft.flip_x,
                            false,
                            "Flip X symmetry",
                        ) && interactive
                        {
                            draft.flip_x = !draft.flip_x;
                            changed = true;
                        }
                        if micro_icon_toggle(
                            ui,
                            t,
                            Icon::FlipY,
                            "Y",
                            draft.flip_y,
                            false,
                            "Flip Y symmetry",
                        ) && interactive
                        {
                            draft.flip_y = !draft.flip_y;
                            changed = true;
                        }
                        let checker_on = draft.checker != AutoRuleCheckerMode::None;
                        if micro_icon_toggle(
                            ui,
                            t,
                            Icon::Checker,
                            "",
                            checker_on,
                            false,
                            "Cycle Checkerboard mode (None / H / V)",
                        ) && interactive
                        {
                            draft.checker = match draft.checker {
                                AutoRuleCheckerMode::None => AutoRuleCheckerMode::Horizontal,
                                AutoRuleCheckerMode::Horizontal => AutoRuleCheckerMode::Vertical,
                                AutoRuleCheckerMode::Vertical => AutoRuleCheckerMode::None,
                            };
                            changed = true;
                        }
                        let mod_lbl = format!("{}-{}", draft.x_modulo, draft.y_modulo);
                        let mod_on = draft.x_modulo > 1 || draft.y_modulo > 1;
                        if micro_text_badge(
                            ui,
                            t,
                            &mod_lbl,
                            mod_on,
                            "Modulo filter (edit X/Y in right panel)",
                        ) && interactive
                        {
                            state.selected_group_id = Some(grp.id);
                            state.selected_rule_id = Some(rule.id);
                        }
                        if micro_icon_toggle(
                            ui,
                            t,
                            Icon::Perlin,
                            "P",
                            draft.perlin_active,
                            true,
                            "Toggle Perlin noise filter",
                        ) && interactive
                        {
                            draft.perlin_active = !draft.perlin_active;
                            changed = true;
                        }
                    });
                });

                if changed && draft != *rule {
                    let stroke = rule.id.0.wrapping_add(0x4155_5409);
                    app.execute_command(Box::new(UpdateAutoRuleCommand::with_stroke(
                        def.id, grp.id, draft, stroke,
                    )));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let m = ui.add(
                        Button::new(Icon::DotsThreeVertical.rich(ui.ctx(), 12.0, t.muted))
                            .frame(false),
                    );
                    if interactive {
                        Popup::menu(&m).show(|ui| {
                            if ui.button("Edit Rule Pattern").clicked() {
                                state.selected_group_id = Some(grp.id);
                                state.selected_rule_id = Some(rule.id);
                                state.center_auto_mode = CenterAutoMode::RuleEditor;
                                ui.close();
                            }
                            if ui.button("Pick Output Tile").clicked() {
                                if let Some(ts_id) = def.tileset_id {
                                    let mut picker = TilePicker::new();
                                    picker.open(None, Some(ts_id), None);
                                    state.auto_tile_picker =
                                        Some((def.id, grp.id, rule.id, picker));
                                }
                                ui.close();
                            }
                            if r_idx + 1 < rule_count && ui.button("Move Down").clicked() {
                                let mut order: Vec<AutoRuleId> =
                                    grp.rules.iter().map(|r| r.id).collect();
                                order.swap(r_idx, r_idx + 1);
                                app.execute_command(Box::new(ReorderAutoRulesCommand::new(
                                    def.id, grp.id, order,
                                )));
                                ui.close();
                            }
                            if ui.button("Delete Rule").clicked() {
                                app.execute_command(Box::new(RemoveAutoRuleCommand::new(
                                    def.id, grp.id, rule.id,
                                )));
                                ui.close();
                            }
                        });
                    }
                });
            });
        })
        .response;

    if card_resp.hovered() {
        state.hovered_rule_id = Some(rule.id);
    }
    if card_resp.clicked() && interactive {
        state.selected_group_id = Some(grp.id);
        state.selected_rule_id = Some(rule.id);
        state.center_auto_mode = CenterAutoMode::RuleEditor;
    }
}

fn micro_icon_toggle(
    ui: &mut Ui,
    t: &Tokens,
    icon: Icon,
    suffix: &str,
    active: bool,
    yellow_when_on: bool,
    tooltip: &str,
) -> bool {
    let fill = if active {
        if yellow_when_on {
            Color32::from_rgb(205, 175, 45)
        } else {
            t.accent.gamma_multiply(0.85)
        }
    } else {
        Color32::from_rgb(75, 32, 32).gamma_multiply(0.65)
    };
    let fg = if active { Color32::BLACK } else { t.muted };
    let text = if suffix.is_empty() {
        icon.glyph().to_string()
    } else {
        format!("{}{suffix}", icon.glyph())
    };
    let btn = Button::new(
        RichText::new(text)
            .font(FontId::new(10.0, lc_canvas::icons::family(ui.ctx())))
            .color(fg),
    )
    .fill(fill)
    .stroke(t.hairline())
    .corner_radius(CornerRadius::ZERO);
    ui.add_sized([28.0, 16.0], btn)
        .on_hover_text(tooltip)
        .clicked()
}

fn micro_text_badge(ui: &mut Ui, t: &Tokens, label: &str, active: bool, tooltip: &str) -> bool {
    let fill = if active {
        Color32::from_rgb(205, 175, 45)
    } else {
        Color32::from_rgb(75, 32, 32).gamma_multiply(0.65)
    };
    let fg = if active { Color32::BLACK } else { t.muted };
    let btn = Button::new(RichText::new(label).size(10.0).color(fg))
        .fill(fill)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO);
    ui.add_sized([30.0, 16.0], btn)
        .on_hover_text(tooltip)
        .clicked()
}
