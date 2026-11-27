//! Center Area Mode 1: Embedded LDtk Rule Pattern Editor (Screenshots 11–15).

use egui::{
    pos2, vec2, Align2, Button, Color32, ComboBox, CornerRadius, FontId, Frame, Margin, Rect,
    RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::{
    AutoRuleTileMode, BiomeRequirementMode, IntGridValueGroupDef, ANY_NON_ZERO_PATTERN,
    GROUP_PATTERN_BASE,
};
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_feature_layers::{UpdateAutoRuleCommand, UpdateAutoRuleGroupCommand};

use super::rule_pattern_widget::{
    draw_diagonal_hatch, draw_tile_in_rect, paint_intgrid_value_visual, render_pattern_matrix,
};
use super::state::LayerViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::info_banner::info_banner;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::tile_picker::TilePicker;

/// Renders the embedded LDtk Rule Pattern Editor inside the Center Area (`RuleEditor` mode).
#[allow(clippy::too_many_arguments)]
pub fn render_pattern_and_output_col(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    icon_tileset: Option<&TilesetDef>,
    icon_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let mut dummy = false;
    SectionContainer::new("PATTERN & OUTPUT")
        .use_toggle_collapse(false)
        .show(
            ui,
            t,
            &mut dummy,
            |_| {},
            |ui| {
                let Some(group) = state
                    .selected_group_id
                    .and_then(|gid| def.rule_group(gid))
                    .cloned()
                else {
                    ui.label(
                        RichText::new(
                            "Select a rule on the left or click '+ GROUP' -> 'USE ASSISTANT'.",
                        )
                        .size(12.0)
                        .color(t.muted),
                    );
                    return;
                };

                // Compact Group settings bar
                let mut grp_draft = group.clone();
                let mut grp_changed = false;
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(RichText::new("Group:").size(11.5).strong().color(t.muted));
                    if ui
                        .add(egui::TextEdit::singleline(&mut grp_draft.name).desired_width(120.0))
                        .changed()
                    {
                        grp_changed = true;
                    }
                    if styled_checkbox(ui, t, &mut grp_draft.is_optional, "Optional").changed() {
                        grp_changed = true;
                    }
                    let mut bio_buf = grp_draft.required_biome_values.join(", ");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut bio_buf)
                                .hint_text("Biome values...")
                                .desired_width(105.0),
                        )
                        .changed()
                    {
                        grp_draft.required_biome_values = bio_buf
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(ToString::to_string)
                            .collect();
                        grp_changed = true;
                    }
                    if !grp_draft.required_biome_values.is_empty() {
                        let mode_lbl = match grp_draft.biome_requirement_mode {
                            BiomeRequirementMode::Require => "Require",
                            BiomeRequirementMode::Exclude => "Exclude",
                        };
                        if ui.button(mode_lbl).clicked() && interactive {
                            grp_draft.biome_requirement_mode =
                                match grp_draft.biome_requirement_mode {
                                    BiomeRequirementMode::Require => BiomeRequirementMode::Exclude,
                                    BiomeRequirementMode::Exclude => BiomeRequirementMode::Require,
                                };
                            grp_changed = true;
                        }
                    }
                });
                if grp_changed && grp_draft != group {
                    let stroke = group.id.0.wrapping_add(0x4155_4701);
                    app.execute_command(Box::new(UpdateAutoRuleGroupCommand::with_stroke(
                        def.id, grp_draft, stroke,
                    )));
                }

                ui.add_space(6.0);
                let Some(rule) = state
                    .selected_rule_id
                    .and_then(|rid| group.rule(rid))
                    .cloned()
                else {
                    ui.label(
                        RichText::new("Select a rule or click '+' on a group to edit its pattern.")
                            .size(12.0)
                            .color(t.muted),
                    );
                    return;
                };

                let mut draft = rule.clone();
                let mut changed = false;

                // 1. Top Strip (Screenshots 11-13: Individual tiles / Rectangles of tiles + Tile Preview Slot)
                Frame::new()
                    .fill(t.sunken)
                    .stroke(t.hairline())
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            let mode_lbl = match draft.tile_mode {
                                AutoRuleTileMode::Single => "Individual tiles",
                                AutoRuleTileMode::Stamp => "Rectangles of tiles",
                            };
                            ComboBox::from_id_salt(("auto_rule_tile_mode", rule.id))
                                .selected_text(mode_lbl)
                                .width(142.0)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            draft.tile_mode == AutoRuleTileMode::Single,
                                            "Individual tiles",
                                        )
                                        .clicked()
                                    {
                                        draft.tile_mode = AutoRuleTileMode::Single;
                                        changed = true;
                                    }
                                    if ui
                                        .selectable_label(
                                            draft.tile_mode == AutoRuleTileMode::Stamp,
                                            "Rectangles of tiles",
                                        )
                                        .clicked()
                                    {
                                        draft.tile_mode = AutoRuleTileMode::Stamp;
                                        changed = true;
                                    }
                                });

                            // Selected output tiles slot (click -> full Tileset Picker Modal Screenshot 14!)
                            let slot_w = (ui.available_width() - 8.0).max(120.0);
                            let (slot_rect, slot_resp) =
                                ui.allocate_exact_size(vec2(slot_w, 38.0), Sense::click());
                            {
                                let p = ui.painter().with_clip_rect(slot_rect);
                                p.rect_filled(slot_rect, CornerRadius::ZERO, t.canvas);
                                if draft.tile_rects.is_empty() {
                                    p.text(
                                        slot_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "Click to pick output tile(s)...",
                                        FontId::proportional(11.0),
                                        t.muted,
                                    );
                                } else {
                                    let mut cur_x = slot_rect.min.x + 4.0;
                                    let tile_sz = 30.0;
                                    for tr in &draft.tile_rects {
                                        for &tid in &tr.tiles {
                                            if cur_x + tile_sz > slot_rect.max.x - 4.0 {
                                                break;
                                            }
                                            let r = egui::Rect::from_min_size(
                                                pos2(cur_x, slot_rect.min.y + 4.0),
                                                vec2(tile_sz, tile_sz),
                                            );
                                            draw_tile_in_rect(
                                                &p,
                                                r,
                                                tileset,
                                                tileset_tex,
                                                tid,
                                                false,
                                                false,
                                                Color32::WHITE,
                                            );
                                            cur_x += tile_sz + 3.0;
                                        }
                                    }
                                }
                                let border = if slot_resp.hovered() && interactive {
                                    Stroke::new(1.5, Color32::from_rgb(245, 190, 60))
                                } else {
                                    t.hairline()
                                };
                                p.rect_stroke(
                                    slot_rect,
                                    CornerRadius::ZERO,
                                    border,
                                    StrokeKind::Inside,
                                );
                            }
                            if slot_resp.clicked() && interactive {
                                if let Some(ts_id) = def.tileset_id {
                                    let mut picker = TilePicker::new();
                                    picker.open(None, Some(ts_id), None);
                                    state.auto_tile_picker =
                                        Some((def.id, group.id, rule.id, picker));
                                }
                            } else if slot_resp.secondary_clicked()
                                && interactive
                                && !draft.tile_rects.is_empty()
                            {
                                draft.tile_rects.pop();
                                changed = true;
                            }
                            slot_resp.on_hover_text(
                                "Left-click: open Tileset Picker | Right-click: remove last tile",
                            );
                        });
                    });

                ui.add_space(8.0);

                // 2. Split Workbench: Left Visual IntGrid Palette + Right Interactive Hatched Matrix
                let pal_w = 148.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    // Left IntGrid Palette (Screenshots 11-13)
                    ui.allocate_ui_with_layout(
                        vec2(pal_w, 240.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(pal_w);
                            // Palette header with 3x3 / 5x5 / 7x7 Grid Toggle Icon
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("INTGRID PALETTE")
                                        .size(10.5)
                                        .strong()
                                        .color(t.muted),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let sz_btn = Button::new(
                                            RichText::new(format!(
                                                "{} {}x{}",
                                                Icon::Grid.glyph(),
                                                draft.size,
                                                draft.size
                                            ))
                                            .font(FontId::new(
                                                11.0,
                                                lc_canvas::icons::family(ui.ctx()),
                                            ))
                                            .color(t.ink_strong),
                                        )
                                        .fill(t.sunken)
                                        .stroke(t.hairline())
                                        .corner_radius(CornerRadius::ZERO);
                                        if ui
                                            .add_sized([54.0, 22.0], sz_btn)
                                            .on_hover_text("Cycle pattern size (3x3 / 5x5 / 7x7)")
                                            .clicked()
                                            && interactive
                                        {
                                            let next = match draft.size {
                                                3 => 5,
                                                5 => 7,
                                                _ => 3,
                                            };
                                            draft.resize_pattern(next);
                                            changed = true;
                                        }
                                    },
                                );
                            });
                            ui.add_space(6.0);

                            // Visual IntGrid value cards (tileset thumbnail + colored name banner)
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
                                for val in source_values {
                                    let active = state.pattern_brush_value == val.value;
                                    let vc = Color32::from_rgba_unmultiplied(
                                        val.color[0],
                                        val.color[1],
                                        val.color[2],
                                        255,
                                    );
                                    let card_sz = vec2(66.0, 62.0);
                                    let (cr, resp) = ui.allocate_exact_size(card_sz, Sense::click());
                                    if ui.is_rect_visible(cr) {
                                        let cp = ui.painter().with_clip_rect(cr);
                                        cp.rect_filled(cr, CornerRadius::ZERO, t.sunken);
                                        let top_r = Rect::from_min_max(
                                            cr.min,
                                            pos2(cr.max.x, cr.max.y - 18.0),
                                        );
                                        paint_intgrid_value_visual(
                                            &cp,
                                            top_r,
                                            val.value,
                                            source_values,
                                            source_groups,
                                            icon_tileset,
                                            icon_tex,
                                            false,
                                        );
                                        let bot_r = Rect::from_min_max(
                                            pos2(cr.min.x, cr.max.y - 18.0),
                                            cr.max,
                                        );
                                        cp.rect_filled(
                                            bot_r,
                                            CornerRadius::ZERO,
                                            vc.gamma_multiply(0.85),
                                        );
                                        cp.text(
                                            bot_r.center(),
                                            Align2::CENTER_CENTER,
                                            if val.name.is_empty() {
                                                format!("#{}", val.value)
                                            } else {
                                                val.name.chars().take(9).collect()
                                            },
                                            FontId::proportional(10.0),
                                            Color32::WHITE,
                                        );
                                        let b = if active {
                                            Stroke::new(2.0, t.ink_strong)
                                        } else {
                                            t.hairline()
                                        };
                                        cp.rect_stroke(cr, CornerRadius::ZERO, b, StrokeKind::Inside);
                                    }
                                    if resp.clicked() && interactive {
                                        state.pattern_brush_value = val.value;
                                    }
                                }
                            });

                            ui.add_space(6.0);
                            // "Any value" / "No value" hatched swatch strip (Screenshot 11)
                            let active_any = state.pattern_brush_value == ANY_NON_ZERO_PATTERN;
                            let (any_r, any_resp) = ui.allocate_exact_size(
                                vec2(pal_w, 34.0),
                                Sense::click(),
                            );
                            if ui.is_rect_visible(any_r) {
                                let ap = ui.painter().with_clip_rect(any_r);
                                ap.rect_filled(any_r, CornerRadius::ZERO, t.canvas);
                                draw_diagonal_hatch(&ap, any_r, Color32::from_white_alpha(18));
                                ap.text(
                                    any_r.center(),
                                    Align2::CENTER_CENTER,
                                    "\"Any value\" / \"No value\"",
                                    FontId::proportional(10.5),
                                    t.ink_strong,
                                );
                                ap.rect_stroke(
                                    any_r,
                                    CornerRadius::ZERO,
                                    if active_any {
                                        Stroke::new(1.5, t.accent)
                                    } else {
                                        t.hairline()
                                    },
                                    StrokeKind::Inside,
                                );
                            }
                            if any_resp.clicked() && interactive {
                                state.pattern_brush_value = ANY_NON_ZERO_PATTERN;
                            }

                            // IntGrid Groups brushes
                            if !source_groups.is_empty() {
                                ui.add_space(4.0);
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                                    for g in source_groups {
                                        let gcode = GROUP_PATTERN_BASE + g.uid as i32;
                                        let act = state.pattern_brush_value == gcode;
                                        let btn = Button::new(
                                            RichText::new(format!("G:{}", g.name))
                                                .size(10.5)
                                                .color(t.ink_strong),
                                        )
                                        .fill(if act { t.accent } else { t.sunken })
                                        .stroke(t.hairline())
                                        .corner_radius(CornerRadius::ZERO);
                                        if ui.add(btn).clicked() && interactive {
                                            state.pattern_brush_value = gcode;
                                        }
                                    }
                                });
                            }
                        },
                    );

                    // Right Hatched Pattern Matrix Grid
                    ui.vertical(|ui| {
                        if render_pattern_matrix(
                            ui,
                            t,
                            &mut draft,
                            state.pattern_brush_value,
                            source_values,
                            source_groups,
                            icon_tileset,
                            icon_tex,
                            interactive,
                        ) {
                            changed = true;
                        }
                    });
                });

                ui.add_space(8.0);

                // 3. Footer Bar: Enable help + "Out-of-bounds" policy + #RuleId (Screenshots 11 & 15)
                let help_btn = Button::new(
                    RichText::new("Enable help")
                        .size(11.0)
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(56, 130, 74))
                .corner_radius(CornerRadius::ZERO);
                if ui
                    .add_sized([ui.available_width().max(180.0), 22.0], help_btn)
                    .clicked()
                    && interactive
                {
                    state.show_rule_help = !state.show_rule_help;
                }
                if state.show_rule_help {
                    ui.add_space(4.0);
                    info_banner::<fn(&mut Ui)>(
                        ui,
                        t,
                        "Left-click matrix cell: require selected IntGrid value/Any. Right-click cell: forbid value (Prohibit icon). Click again to clear.",
                        None,
                    );
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("\"Out-of-bounds\" policy")
                            .size(11.0)
                            .color(t.muted),
                    );
                    let oob_label = match draft.out_of_bounds_value {
                        None => "This rule should not apply outside bounds (default)".to_string(),
                        Some(0) => "Empty cells".to_string(),
                        Some(v) => {
                            let name = source_values
                                .iter()
                                .find(|ig| ig.value == v)
                                .map(|ig| ig.name.as_str())
                                .unwrap_or("Value");
                            format!("{v} - {name}")
                        }
                    };
                    ComboBox::from_id_salt(("rule_oob_policy", rule.id))
                        .selected_text(oob_label)
                        .width((ui.available_width() - 60.0).clamp(160.0, 290.0))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    draft.out_of_bounds_value.is_none(),
                                    "This rule should not apply when reading cells outside of layer bounds (default)",
                                )
                                .clicked()
                            {
                                draft.out_of_bounds_value = None;
                                changed = true;
                            }
                            if ui
                                .selectable_label(draft.out_of_bounds_value == Some(0), "Empty cells")
                                .clicked()
                            {
                                draft.out_of_bounds_value = Some(0);
                                changed = true;
                            }
                            for v in source_values {
                                if ui
                                    .selectable_label(
                                        draft.out_of_bounds_value == Some(v.value),
                                        format!("{} - {}", v.value, v.name),
                                    )
                                    .clicked()
                                {
                                    draft.out_of_bounds_value = Some(v.value);
                                    changed = true;
                                }
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("#{}", rule.id.0))
                                .size(11.0)
                                .monospace()
                                .color(t.faint),
                        );
                    });
                });

                if changed && draft != rule {
                    let stroke = rule.id.0.wrapping_add(0x4155_5403);
                    app.execute_command(Box::new(UpdateAutoRuleCommand::with_stroke(
                        def.id, group.id, draft, stroke,
                    )));
                }
            },
        );
}
