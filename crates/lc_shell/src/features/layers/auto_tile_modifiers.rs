//! Right-column `LIVE SANDBOX PREVIEW`, fine numeric Rule Modifiers, and TilePicker modal for `AUTO TILE`.

use egui::{ComboBox, DragValue, RichText, TextEdit, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::{
    AutoLayerRuleDef, AutoLayerRuleGroup, AutoRuleCheckerMode, AutoRuleTileMode, AutoTileRect,
};
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_feature_layers::UpdateAutoRuleCommand;
use lc_ui_api::AssetService;

use super::rule_pattern_widget::render_sandbox_preview;
use super::state::LayerViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::tile_picker::{TilePickerEvent, TilesetData};

/// Renders the Live 7x7 Sandbox Preview first, followed by fine numeric properties for the selected rule.
#[allow(clippy::too_many_arguments)]
pub fn render_rule_modifiers_and_preview(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    selected_group: Option<&AutoLayerRuleGroup>,
    selected_rule: Option<&AutoLayerRuleDef>,
    source_values: &[IntGridValueDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    // 1. Live Mini Sandbox Preview pinned first
    ui.label(
        RichText::new("LIVE SANDBOX PREVIEW")
            .size(11.0)
            .strong()
            .color(t.accent_ink),
    );
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Paint:").size(11.0).color(t.muted));
        ComboBox::from_id_salt(("sandbox_paint_combo", def.id))
            .selected_text(format!("#{}", state.sandbox_paint_value))
            .width(86.0)
            .show_ui(ui, |ui| {
                if source_values.is_empty() {
                    ui.selectable_value(&mut state.sandbox_paint_value, 1, "#1 Solid");
                } else {
                    for v in source_values {
                        ui.selectable_value(
                            &mut state.sandbox_paint_value,
                            v.value,
                            format!("#{} {}", v.value, v.name),
                        );
                    }
                }
            });
        if ui.small_button("Clear").clicked() && interactive {
            state.sandbox_grid = lc_core::grid::Grid::new(7, 7, 0);
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Biomes:").size(11.0).color(t.muted));
        ui.add(
            TextEdit::singleline(&mut state.sandbox_biomes_text)
                .hint_text("e.g. Tundra")
                .desired_width(136.0),
        );
    });
    ui.add_space(6.0);

    render_sandbox_preview(
        ui,
        t,
        state,
        def,
        source_values,
        tileset,
        tileset_tex,
        interactive,
    );

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    // 2. Numeric rule modifiers (Modulo, Offset, Perlin, Pivot)
    let (Some(group), Some(rule)) = (selected_group, selected_rule) else {
        ui.label(
            RichText::new("Select a rule on the left to inspect Modulo / Perlin / Pivot.")
                .size(11.0)
                .color(t.muted),
        );
        return;
    };

    let mut draft = rule.clone();
    let mut changed = false;

    ui.label(
        RichText::new(format!("RULE #{} DETAILS", rule.id.0))
            .size(11.0)
            .strong()
            .color(t.accent_ink),
    );
    ui.add_space(4.0);

    egui::Grid::new(("rule_detail_mod_grid", rule.id))
        .num_columns(2)
        .spacing([8.0, 5.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Modulo X/Y").size(11.0).color(t.muted));
            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut draft.x_modulo)
                            .range(1..=32)
                            .prefix("X:"),
                    )
                    .changed()
                {
                    draft.x_modulo = draft.x_modulo.max(1);
                    changed = true;
                }
                if ui
                    .add(
                        DragValue::new(&mut draft.y_modulo)
                            .range(1..=32)
                            .prefix("Y:"),
                    )
                    .changed()
                {
                    draft.y_modulo = draft.y_modulo.max(1);
                    changed = true;
                }
            });
            ui.end_row();

            ui.label(RichText::new("Offset X/Y").size(11.0).color(t.muted));
            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut draft.x_offset)
                            .range(0..=31)
                            .prefix("X:"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        DragValue::new(&mut draft.y_offset)
                            .range(0..=31)
                            .prefix("Y:"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.end_row();

            ui.label(RichText::new("Checker").size(11.0).color(t.muted));
            let chk_lbl = match draft.checker {
                AutoRuleCheckerMode::None => "None",
                AutoRuleCheckerMode::Horizontal => "Horizontal",
                AutoRuleCheckerMode::Vertical => "Vertical",
            };
            ComboBox::from_id_salt(("rule_detail_checker", rule.id))
                .selected_text(chk_lbl)
                .width(106.0)
                .show_ui(ui, |ui| {
                    for mode in [
                        AutoRuleCheckerMode::None,
                        AutoRuleCheckerMode::Horizontal,
                        AutoRuleCheckerMode::Vertical,
                    ] {
                        let lbl = match mode {
                            AutoRuleCheckerMode::None => "None",
                            AutoRuleCheckerMode::Horizontal => "Horizontal",
                            AutoRuleCheckerMode::Vertical => "Vertical",
                        };
                        if ui.selectable_label(draft.checker == mode, lbl).clicked() {
                            draft.checker = mode;
                            changed = true;
                        }
                    }
                });
            ui.end_row();

            if draft.tile_mode == AutoRuleTileMode::Stamp {
                ui.label(RichText::new("Stamp Pivot").size(11.0).color(t.muted));
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            DragValue::new(&mut draft.pivot_x)
                                .range(0.0..=1.0)
                                .speed(0.1)
                                .prefix("X:"),
                        )
                        .changed()
                    {
                        draft.pivot_x = draft.pivot_x.clamp(0.0, 1.0);
                        changed = true;
                    }
                    if ui
                        .add(
                            DragValue::new(&mut draft.pivot_y)
                                .range(0.0..=1.0)
                                .speed(0.1)
                                .prefix("Y:"),
                        )
                        .changed()
                    {
                        draft.pivot_y = draft.pivot_y.clamp(0.0, 1.0);
                        changed = true;
                    }
                });
                ui.end_row();
            }
        });

    ui.add_space(4.0);
    if styled_checkbox(ui, t, &mut draft.perlin_active, "Perlin noise filter").changed() {
        changed = true;
    }
    if draft.perlin_active {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(DragValue::new(&mut draft.perlin_seed).prefix("Seed:"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    DragValue::new(&mut draft.perlin_scale)
                        .range(0.01..=2.0)
                        .speed(0.01)
                        .prefix("Scale:"),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    DragValue::new(&mut draft.perlin_octaves)
                        .range(1..=6)
                        .prefix("Oct:"),
                )
                .changed()
            {
                changed = true;
            }
        });
    }

    if changed && draft != *rule {
        let stroke = rule.id.0.wrapping_add(0x4155_5401);
        app.execute_command(Box::new(UpdateAutoRuleCommand::with_stroke(
            def.id, group.id, draft, stroke,
        )));
    }
}

/// Polls and renders the visual `TilePicker` modal (Screenshot 14) for Auto-Rule output tiles/stamps.
pub fn render_auto_rule_tile_picker(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    if !def.uses_auto_rules() {
        state.auto_tile_picker = None;
        return;
    }

    let Some((layer_id, group_id, rule_id, mut picker)) = state.auto_tile_picker.take() else {
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
    let tex_size = app
        .assets
        .load_texture(ui.ctx(), &ts.path)
        .ok()
        .map(|tex| tex.size_vec2());

    let mut close = false;
    let current_sel = picker.selected_tile_rect;
    match picker.show(ui.ctx(), &ts_data, current_sel) {
        TilePickerEvent::Selected(tile_rect) => {
            if let Some(rule) = def
                .rule_group(group_id)
                .and_then(|g| g.rule(rule_id))
                .cloned()
            {
                let tw = ts.tile_size.x.max(1);
                let eff_w = tex_size
                    .map(|sz| (sz.x as u32).saturating_sub(ts.padding * 2))
                    .unwrap_or(tw * (tile_rect.col + tile_rect.w).max(1));
                let cols = (eff_w + ts.spacing)
                    .checked_div(tw + ts.spacing)
                    .unwrap_or(1)
                    .max(1);

                let mut updated = rule;
                if tile_rect.w <= 1 && tile_rect.h <= 1 {
                    let tid = tile_rect.row * cols + tile_rect.col;
                    updated.tile_rects.push(AutoTileRect::single(tid));
                } else {
                    let mut stamp_tiles = Vec::new();
                    for dy in 0..tile_rect.h {
                        for dx in 0..tile_rect.w {
                            let tid = (tile_rect.row + dy) * cols + (tile_rect.col + dx);
                            stamp_tiles.push(tid);
                        }
                    }
                    updated.tile_mode = AutoRuleTileMode::Stamp;
                    updated.tile_rects.push(AutoTileRect::stamp(
                        stamp_tiles,
                        tile_rect.w,
                        tile_rect.h,
                    ));
                }
                app.execute_command(Box::new(UpdateAutoRuleCommand::new(
                    def.id, group_id, updated,
                )));
            }
            close = true;
        }
        TilePickerEvent::Closed => close = true,
        TilePickerEvent::None => {}
    }

    if !close && picker.is_open {
        state.auto_tile_picker = Some((layer_id, group_id, rule_id, picker));
    }
}
