//! `DISPLAY OPTIONS` and type-specific (`ENTITIES` / `AUTOLAYER` / `TILES`) settings cards.

use egui::{vec2, Button, Color32, ComboBox, CornerRadius, DragValue, Id, RichText, Stroke, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::LayerDef;
use lc_core::fields::FieldType;
use lc_core::layer::LayerKind;
use lc_core::marking::known_tags;
use lc_feature_layers::UpdateLayerDefCommand;

use super::attributes_card::attr_row;
use super::left_panel::layer_kind_color;
use super::state::LayerViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::info_banner::info_banner;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::swatch::color_swatch;
use crate::shared::ui::tag_row::{tag_row, TagEvent};
use crate::shared::ui::tileset_select::{tileset_select_full_width, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, CONTROL_HEIGHT_FORM};

/// Renders Section 2 (`DISPLAY OPTIONS`) matching LDtk Screenshots 1–8.
pub fn render_display_options_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let mut draft = def.clone();
    let mut changed = false;
    let mut collapse = state.collapse_visual;

    SectionContainer::new("DISPLAY OPTIONS")
        .use_toggle_collapse(true)
        .show(
            ui,
            t,
            &mut collapse,
            |_| {},
            |ui| {
                // 1. Color in layers list + reset icon
                attr_row(ui, t, "Color in layers list", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let mut ui_col = draft
                            .ui_color
                            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], 255))
                            .unwrap_or_else(|| layer_kind_color(def.kind));
                        if color_swatch(ui, Id::new(("layer_ui_color", def.id)), &mut ui_col)
                            .changed()
                        {
                            draft.ui_color = Some([ui_col.r(), ui_col.g(), ui_col.b(), 255]);
                            changed = true;
                        }
                        if draft.ui_color.is_some() {
                            let rst = Button::new(Icon::Undo.rich(ui.ctx(), 13.0, t.muted))
                                .fill(t.sunken)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::ZERO);
                            if ui
                                .add_sized([28.0, CONTROL_HEIGHT_COMPACT], rst)
                                .on_hover_text("Reset layer list color to default")
                                .clicked()
                            {
                                draft.ui_color = None;
                                changed = true;
                            }
                        }
                    });
                });
                ui.add_space(5.0);

                // 2. Visual options (2-column compact checkbox grid)
                attr_row(ui, t, "Visual options", |ui| {
                    egui::Grid::new(("visual_opts_grid", def.id))
                        .num_columns(2)
                        .spacing([14.0, 4.0])
                        .show(ui, |ui| {
                            if styled_checkbox(ui, t, &mut draft.hide_in_list, "Hide in list")
                                .changed()
                            {
                                changed = true;
                            }
                            if draft.kind != LayerKind::Entities {
                                if styled_checkbox(
                                    ui,
                                    t,
                                    &mut draft.render_in_world_view,
                                    "Show in world view",
                                )
                                .changed()
                                {
                                    changed = true;
                                }
                            } else if styled_checkbox(
                                ui,
                                t,
                                &mut draft.can_select_when_inactive,
                                "Allow selections when inactive",
                            )
                            .changed()
                            {
                                changed = true;
                            }
                            ui.end_row();

                            if matches!(draft.kind, LayerKind::IntGrid | LayerKind::Tiles)
                                && styled_checkbox(
                                    ui,
                                    t,
                                    &mut draft.can_select_when_inactive,
                                    "Allow selections when inactive",
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            if draft.kind != LayerKind::Entities
                                && styled_checkbox(
                                    ui,
                                    t,
                                    &mut draft.async_render,
                                    "Asynchronous render",
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            ui.end_row();
                        });
                });
                ui.add_space(5.0);

                // 3. Optional "guide" grid
                attr_row(ui, t, "Optional \"guide\" grid", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if ui
                            .add(
                                DragValue::new(&mut draft.guide_grid_size.x)
                                    .range(0..=1024)
                                    .prefix("W: ")
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(RichText::new("x").size(11.0).color(t.muted));
                        if ui
                            .add(
                                DragValue::new(&mut draft.guide_grid_size.y)
                                    .range(0..=1024)
                                    .prefix("H: ")
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                });
                ui.add_space(5.0);

                // 4. UI tags (used for left sidebar grouping & World Layer Tag filtering)
                attr_row(ui, t, "UI tags", |ui| {
                    ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                    let mut extra_tags: Vec<String> = app
                        .project
                        .defs
                        .layers
                        .iter()
                        .flat_map(|l| l.tags.clone())
                        .collect();
                    for w in &app.project.worlds {
                        extra_tags.extend(w.included_layer_tags.clone());
                        extra_tags.extend(w.excluded_layer_tags.clone());
                    }
                    for tag in &app.tag_registry {
                        extra_tags.push(tag.name.clone());
                    }
                    let all_tags = known_tags(&app.project.defs.tilesets, &extra_tags);
                    for ev in tag_row(ui, &draft.tags, &all_tags) {
                        apply_tag_event(&mut draft.tags, ev, &mut changed);
                    }
                });
            },
        );
    state.collapse_visual = collapse;

    if changed && draft != *def {
        let stroke = def.id.0.wrapping_add(0x4C59_5206);
        app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(draft, stroke)));
    }
}

/// Shared Auto-Layer bindings rendered in both `AUTOLAYER SETTINGS` and `INTGRID SETTINGS`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_autolayer_bindings(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    draft: &mut LayerDef,
    changed: &mut bool,
    show_source: bool,
) {
    if show_source {
        attr_row(ui, t, "Auto-Layer source", |ui| {
            let src_label = draft
                .auto_source_layer_def_id
                .and_then(|id| app.project.defs.layer(id))
                .map(|l| l.name.as_str())
                .unwrap_or("-- Select IntGrid layer --");
            ComboBox::from_id_salt(("auto_source_layer", draft.id))
                .selected_text(src_label)
                .width(240.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(draft.auto_source_layer_def_id.is_none(), "-- None --")
                        .clicked()
                    {
                        draft.auto_source_layer_def_id = None;
                        *changed = true;
                    }
                    for other in &app.project.defs.layers {
                        if other.id != draft.id
                            && other.kind == LayerKind::IntGrid
                            && ui
                                .selectable_label(
                                    draft.auto_source_layer_def_id == Some(other.id),
                                    &other.name,
                                )
                                .clicked()
                        {
                            draft.auto_source_layer_def_id = Some(other.id);
                            draft.grid_size = other.grid_size;
                            *changed = true;
                        }
                    }
                });
        });
        ui.add_space(4.0);
        info_banner::<fn(&mut Ui)>(
            ui,
            t,
            "This auto-layer reads the selected IntGrid layer above to evaluate its rules.",
            None,
        );
        ui.add_space(6.0);
    }

    // Auto-layer tileset + EDIT RULES jump button
    attr_row(ui, t, "Auto-layer tileset", |ui| {
        let ts_items: Vec<TilesetSelectItem<_>> = app
            .project
            .defs
            .tilesets
            .iter()
            .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
            .collect();
        let edit_btn_w = 104.0;
        let combo_w = (ui.available_width() - edit_btn_w - 8.0).max(160.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.allocate_ui(vec2(combo_w, CONTROL_HEIGHT_FORM), |ui| {
                let mut sel_ts = draft.tileset_id;
                if tileset_select_full_width(ui, &ts_items, &mut sel_ts, true) {
                    draft.tileset_id = sel_ts;
                    *changed = true;
                }
            });
            let can_edit = draft.uses_auto_rules();
            let btn = Button::new(
                RichText::new("EDIT RULES")
                    .size(11.5)
                    .strong()
                    .color(if can_edit { t.invert_fg } else { t.muted }),
            )
            .fill(if can_edit { t.accent } else { t.sunken })
            .stroke(if can_edit { Stroke::NONE } else { t.hairline() })
            .corner_radius(CornerRadius::ZERO);
            if ui
                .add_enabled(
                    can_edit,
                    btn.min_size(vec2(edit_btn_w, CONTROL_HEIGHT_FORM)),
                )
                .clicked()
            {
                state.active_sub_tab = 1; // AUTO_TILE_SUB_TAB
            }
        });
    });
    ui.add_space(5.0);

    // Kill tiles
    attr_row(ui, t, "Kill tiles", |ui| {
        let kill_label = draft
            .auto_tiles_killed_by_other_layer_id
            .and_then(|id| app.project.defs.layer(id))
            .map(|l| l.name.as_str())
            .unwrap_or("-- Select a layer --");
        ComboBox::from_id_salt(("auto_killed_by", draft.id))
            .selected_text(kill_label)
            .width(240.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(draft.auto_tiles_killed_by_other_layer_id.is_none(), "None")
                    .clicked()
                {
                    draft.auto_tiles_killed_by_other_layer_id = None;
                    *changed = true;
                }
                for other in &app.project.defs.layers {
                    if other.id != draft.id
                        && other.kind == LayerKind::IntGrid
                        && ui
                            .selectable_label(
                                draft.auto_tiles_killed_by_other_layer_id == Some(other.id),
                                &other.name,
                            )
                            .clicked()
                    {
                        draft.auto_tiles_killed_by_other_layer_id = Some(other.id);
                        *changed = true;
                    }
                }
            });
    });
    ui.add_space(5.0);

    // Biome enum
    let enum_fields: Vec<(lc_core::id::FieldDefId, String)> = app
        .project
        .defs
        .entities
        .iter()
        .flat_map(|e| {
            e.fields
                .iter()
                .filter(|f| matches!(f.field_type, FieldType::Enum(_)))
                .map(move |f| (f.id, format!("{}.{}", e.name, f.name)))
        })
        .collect();
    attr_row(ui, t, "Biome enum", |ui| {
        let bio_label = draft
            .biome_field_id
            .and_then(|id| enum_fields.iter().find(|(fid, _)| *fid == id))
            .map(|(_, n)| n.as_str())
            .unwrap_or("No biome enum");
        ui.add_enabled_ui(
            !enum_fields.is_empty() || draft.biome_field_id.is_some(),
            |ui| {
                ComboBox::from_id_salt(("auto_biome_field", draft.id))
                    .selected_text(bio_label)
                    .width(240.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(draft.biome_field_id.is_none(), "No biome enum")
                            .clicked()
                        {
                            draft.biome_field_id = None;
                            *changed = true;
                        }
                        for (fid, fname) in &enum_fields {
                            if ui
                                .selectable_label(draft.biome_field_id == Some(*fid), fname)
                                .clicked()
                            {
                                draft.biome_field_id = Some(*fid);
                                *changed = true;
                            }
                        }
                    });
            },
        );
    });

    if draft.uses_auto_rules() {
        ui.add_space(6.0);
        attr_row(ui, t, "Advanced operations", |ui| {
            let bake_btn = Button::new(
                RichText::new("BAKE AUTO-LAYER")
                    .size(11.5)
                    .strong()
                    .color(t.invert_fg),
            )
            .fill(t.accent)
            .corner_radius(CornerRadius::ZERO);
            if ui
                .add_sized(
                    [ui.available_width().max(180.0), CONTROL_HEIGHT_COMPACT],
                    bake_btn,
                )
                .on_hover_text("Convert generated auto-tiles into a regular Tiles layer")
                .clicked()
            {
                state.pending_bake_auto = Some(draft.id);
            }
        });
    }
}

pub(crate) fn apply_tag_event(tags: &mut Vec<String>, ev: TagEvent, changed: &mut bool) {
    match ev {
        TagEvent::Add(tag) => {
            let clean = tag.trim().to_string();
            if !clean.is_empty() && !tags.iter().any(|t| t.eq_ignore_ascii_case(&clean)) {
                tags.push(clean);
                *changed = true;
            }
        }
        TagEvent::Remove(tag) => {
            let prev = tags.len();
            tags.retain(|t| t != &tag);
            if tags.len() != prev {
                *changed = true;
            }
        }
        TagEvent::Rename { from, to } => {
            for t in tags.iter_mut() {
                if *t == from {
                    *t = to.clone();
                    *changed = true;
                }
            }
        }
    }
}
