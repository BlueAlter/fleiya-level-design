//! `AUTO TILE` tab coordinator with strict uninteractable guard + 3-zone LDtk workbench
//! (`Left RULES` + `Center [Rule Editor | Auto-Layer Rules Assistant]` + `Right LIVE SANDBOX PREVIEW`).

use egui::{vec2, Button, CornerRadius, Id, RichText, ScrollArea, Sense, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::IntGridValueGroupDef;
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::layer::LayerKind;
use lc_ui_api::AssetService;

use super::auto_tile_groups_col::render_groups_and_rules_col;
use super::auto_tile_modifiers::{render_auto_rule_tile_picker, render_rule_modifiers_and_preview};
use super::auto_tile_pattern_col::render_pattern_and_output_col;
use super::rule_wizard::render_center_rules_assistant;
use super::state::{CenterAutoMode, LayerViewState, RulesAssistantState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::info_banner::info_banner;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

/// Renders the `AUTO TILE` tab for `def` (locked & uninteractable when `!def.uses_auto_rules()`).
pub fn render_auto_tile_tab(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let interactive = def.uses_auto_rules();

    if !interactive {
        let lock_reason = match def.kind {
            LayerKind::Tiles => {
                "Auto-Tile Rules are disabled for Tiles layers. Switch tabs or bind/select an Auto-Layer or IntGrid layer with a tileset."
            }
            LayerKind::Entities => {
                "Auto-Tile Rules are disabled for Entities layers. This tab is locked (uninteractable)."
            }
            LayerKind::IntGrid => {
                "This IntGrid layer has no bound Tileset. Bind an Auto-layer tileset in LAYERS DEF -> INTGRID SETTINGS to unlock Auto-Tile Rules."
            }
            LayerKind::Auto => "Auto-Layer locked.",
        };
        info_banner::<fn(&mut Ui)>(ui, t, lock_reason, None);
        ui.add_space(6.0);
    }

    // Resolve source IntGrid values + groups (self if IntGrid, or source layer if Auto)
    let source_layer: Option<LayerDef> = match def.kind {
        LayerKind::IntGrid => Some(def.clone()),
        LayerKind::Auto => def
            .auto_source_layer_def_id
            .and_then(|sid| app.project.defs.layer(sid))
            .cloned(),
        _ => None,
    };
    let source_values: Vec<IntGridValueDef> = source_layer
        .as_ref()
        .map(|l| l.int_grid_values.clone())
        .unwrap_or_default();
    let source_groups: Vec<IntGridValueGroupDef> = source_layer
        .as_ref()
        .map(|l| l.int_grid_value_groups.clone())
        .unwrap_or_default();

    let tileset: Option<TilesetDef> = def
        .tileset_id
        .and_then(|tid| app.project.defs.tileset(tid))
        .cloned();
    let tileset_tex: Option<TextureHandle> = tileset
        .as_ref()
        .and_then(|ts| app.assets.load_texture(ui.ctx(), &ts.path).ok());

    let icon_tileset: Option<TilesetDef> = source_layer
        .as_ref()
        .and_then(|src| src.effective_int_grid_icons_tileset())
        .or(def.tileset_id)
        .and_then(|tid| app.project.defs.tileset(tid))
        .cloned();
    let icon_tex: Option<TextureHandle> = icon_tileset
        .as_ref()
        .and_then(|ts| app.assets.load_texture(ui.ctx(), &ts.path).ok());

    // Keep selected_group_id / selected_rule_id valid
    if let Some(first_grp) = def.auto_rule_groups.first() {
        let grp_valid = state
            .selected_group_id
            .is_some_and(|gid| def.rule_group(gid).is_some());
        if !grp_valid {
            state.selected_group_id = Some(first_grp.id);
        }
        if let Some(gid) = state.selected_group_id {
            if let Some(grp) = def.rule_group(gid) {
                let rule_valid = state
                    .selected_rule_id
                    .is_some_and(|rid| grp.rule(rid).is_some());
                if !rule_valid {
                    state.selected_rule_id = grp.rules.first().map(|r| r.id);
                }
            }
        }
    } else {
        state.selected_group_id = None;
        state.selected_rule_id = None;
    }

    let body_rect = ui.available_rect_before_wrap();
    ui.add_enabled_ui(interactive, |ui| {
        render_auto_tile_body(
            app,
            ui,
            t,
            state,
            def,
            source_layer.as_ref(),
            &source_values,
            &source_groups,
            tileset.as_ref(),
            tileset_tex.as_ref(),
            icon_tileset.as_ref(),
            icon_tex.as_ref(),
            interactive,
        );
    });

    if !interactive {
        let _lock_guard = ui.interact(
            body_rect,
            Id::new(("auto_tile_uninteractable_guard", def.id)),
            Sense::click_and_drag(),
        );
    } else {
        render_auto_rule_tile_picker(app, ui, state, def);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_auto_tile_body(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_layer: Option<&LayerDef>,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    icon_tileset: Option<&TilesetDef>,
    icon_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    // 1. Top Toolbar: Source info + Center Mode Switcher [Rule Editor | Rules Assistant]
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let src_text = match def.kind {
            LayerKind::Auto => source_layer
                .map(|l| format!("Source: {}", l.name))
                .unwrap_or_else(|| "Source: (Unbound IntGrid)".to_string()),
            LayerKind::IntGrid => format!("Source: {} (Self)", def.name),
            _ => "Source: N/A".to_string(),
        };
        ui.label(RichText::new(src_text).size(11.5).color(t.muted));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let asst_active = state.center_auto_mode == CenterAutoMode::RulesAssistant;
            let asst_btn = Button::new(RichText::new("RULES ASSISTANT").size(11.0).strong().color(
                if asst_active {
                    t.invert_fg
                } else {
                    t.ink_strong
                },
            ))
            .fill(if asst_active { t.accent } else { t.sunken })
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO);
            if ui
                .add_sized([128.0, CONTROL_HEIGHT_COMPACT], asst_btn)
                .clicked()
                && interactive
            {
                if state.assistant.layer_id != def.id {
                    let dv = source_values.first().map(|v| v.value).unwrap_or(1);
                    let dn = source_values
                        .first()
                        .map(|v| v.name.clone())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "AutoTerrain".to_string());
                    state.assistant = RulesAssistantState::for_layer(def.id, dv, &dn);
                }
                state.center_auto_mode = CenterAutoMode::RulesAssistant;
            }

            let ed_active = state.center_auto_mode == CenterAutoMode::RuleEditor;
            let ed_btn = Button::new(
                RichText::new("RULE EDITOR")
                    .size(11.0)
                    .strong()
                    .color(if ed_active { t.invert_fg } else { t.ink_strong }),
            )
            .fill(if ed_active { t.accent } else { t.sunken })
            .stroke(t.hairline())
            .corner_radius(CornerRadius::ZERO);
            if ui
                .add_sized([106.0, CONTROL_HEIGHT_COMPACT], ed_btn)
                .clicked()
                && interactive
            {
                state.center_auto_mode = CenterAutoMode::RuleEditor;
            }
        });
    });

    if interactive && def.kind == LayerKind::Auto && (source_layer.is_none() || tileset.is_none()) {
        ui.add_space(6.0);
        info_banner::<fn(&mut Ui)>(
            ui,
            t,
            "Assign both an Auto-Layer source and Auto-layer tileset in LAYERS DEF so Auto-Layer rules can match IntGrid values.",
            None,
        );
    }

    ui.add_space(8.0);

    // 2. Three-Zone Rule Workbench
    let total_w = ui.available_width();
    let avail_h = ui.available_height().max(260.0);
    let left_col_w = 278.0;
    let right_col_w = 240.0;
    let gap = WORKSPACE_PADDING;

    if total_w >= 820.0 {
        let center_w = (total_w - left_col_w - right_col_w - gap * 2.0).max(260.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            ui.allocate_ui_with_layout(
                vec2(left_col_w, avail_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left_col_w);
                    ui.set_max_width(left_col_w);
                    render_groups_and_rules_col(
                        app,
                        ui,
                        t,
                        state,
                        def,
                        source_values,
                        source_groups,
                        tileset,
                        tileset_tex,
                        interactive,
                    );
                },
            );

            ui.add_space(gap);

            ui.allocate_ui_with_layout(
                vec2(center_w, avail_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(center_w);
                    ui.set_max_width(center_w);
                    render_center_workbench(
                        app,
                        ui,
                        t,
                        state,
                        def,
                        source_values,
                        source_groups,
                        tileset,
                        tileset_tex,
                        icon_tileset,
                        icon_tex,
                        interactive,
                    );
                },
            );

            ui.add_space(gap);

            ui.allocate_ui_with_layout(
                vec2(right_col_w, avail_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right_col_w);
                    ui.set_max_width(right_col_w);
                    render_right_modifiers_col(
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
        });
    } else {
        ScrollArea::vertical()
            .id_salt(("auto_tile_narrow_scroll", def.id))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_groups_and_rules_col(
                    app,
                    ui,
                    t,
                    state,
                    def,
                    source_values,
                    source_groups,
                    tileset,
                    tileset_tex,
                    interactive,
                );
                ui.add_space(10.0);
                render_center_workbench(
                    app,
                    ui,
                    t,
                    state,
                    def,
                    source_values,
                    source_groups,
                    tileset,
                    tileset_tex,
                    icon_tileset,
                    icon_tex,
                    interactive,
                );
                ui.add_space(10.0);
                render_right_modifiers_col(
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
            });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_center_workbench(
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
    match state.center_auto_mode {
        CenterAutoMode::RuleEditor => render_pattern_and_output_col(
            app,
            ui,
            t,
            state,
            def,
            source_values,
            source_groups,
            tileset,
            tileset_tex,
            icon_tileset,
            icon_tex,
            interactive,
        ),
        CenterAutoMode::RulesAssistant => render_center_rules_assistant(
            app,
            ui,
            t,
            state,
            def,
            source_values,
            tileset,
            tileset_tex,
            interactive,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_right_modifiers_col(
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
    let mut dummy = false;
    let selected_group = state
        .selected_group_id
        .and_then(|gid| def.rule_group(gid))
        .cloned();
    let selected_rule = selected_group
        .as_ref()
        .and_then(|g| state.selected_rule_id.and_then(|rid| g.rule(rid)))
        .cloned();

    SectionContainer::new("SANDBOX & RULE DETAILS")
        .use_toggle_collapse(false)
        .show(
            ui,
            t,
            &mut dummy,
            |_| {},
            |ui| {
                ScrollArea::vertical()
                    .id_salt(("auto_mod_scroll", def.id))
                    .max_height((ui.available_height() - 12.0).max(460.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        render_rule_modifiers_and_preview(
                            app,
                            ui,
                            t,
                            state,
                            def,
                            selected_group.as_ref(),
                            selected_rule.as_ref(),
                            source_values,
                            tileset,
                            tileset_tex,
                            interactive,
                        );
                    });
            },
        );
}
