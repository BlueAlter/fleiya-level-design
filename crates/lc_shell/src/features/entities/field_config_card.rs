//! Selected Entity Field Configuration right-column panels (100% LDtk FieldDefsForm rules).

use egui::{Color32, DragValue, Pos2, RichText, Stroke, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, TilesetRect};
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_feature_entities::{
    entity_choices, field_type_editor, field_type_label, field_value_editor, FieldEditorContext,
    UpdateEntityFieldCommand,
};

use super::attributes_card::{attr_row, open_tile_picker, resolve_tile_icon};
use super::field_constraints_card::render_type_constraints;
use super::field_display_card::render_editor_display_settings;
use super::state::{EntityTileSlot, EntityViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::swatch::{color_swatch, tile_icon_button};
use crate::shared::ui::text_field::{text_field, FieldOutline, FieldStyle};
use crate::shared::ui::tileset_select::{tileset_select, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::ICON_BUTTON_SIZE_FORM;

/// Renders an LDtk-style uppercase gold/accent sub-heading with a full-width hairline underline.
pub(crate) fn render_sub_heading(ui: &mut Ui, t: &Tokens, title: &str) {
    ui.label(RichText::new(title).size(12.0).strong().color(t.accent_ink));
    let avail = ui.available_width().max(120.0);
    let (line_rect, _) = ui.allocate_exact_size(egui::vec2(avail, 4.0), egui::Sense::hover());
    ui.painter().line_segment(
        [
            Pos2::new(line_rect.left(), line_rect.top() + 1.0),
            Pos2::new(line_rect.right(), line_rect.top() + 1.0),
        ],
        Stroke::new(1.0, t.line),
    );
    ui.add_space(4.0);
}

/// Renders the detailed configuration for the selected field inside the scrollable right card.
pub fn render_selected_field_config(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
    field_idx: usize,
    orig: &FieldDef,
) {
    let mut draft = orig.clone();
    let mut changed = false;
    let enums = app.project.defs.enums.clone();
    let choices = entity_choices(&app.project);
    let editor_ctx = FieldEditorContext::new(&enums, &choices);
    let base_ty = draft.base_type().clone();

    // 1. GENERAL SETTINGS
    render_sub_heading(ui, t, "GENERAL SETTINGS");

    attr_row(ui, t, "Type", |ui| {
        let mut is_array = draft.is_array();
        let type_text = match &base_ty {
            FieldType::Enum(eid) => {
                let ename = enums
                    .iter()
                    .find(|e| e.id == *eid)
                    .map(|e| e.name.as_str())
                    .unwrap_or("Unknown");
                if is_array {
                    format!("Array<Enum.{ename}>")
                } else {
                    format!("Enum.{ename}")
                }
            }
            _ => {
                if is_array {
                    format!("Array<{}>", field_type_label(&base_ty))
                } else {
                    field_type_label(&base_ty).to_string()
                }
            }
        };
        ui.label(RichText::new(type_text).color(t.ink_strong).strong());
        ui.add_space(8.0);
        ui.label(RichText::new("CONVERT:").size(10.5).color(t.muted));
        if let Some(new_base) = field_type_editor(ui, &base_ty, &enums) {
            let clean_base = match new_base {
                FieldType::Array(inner) => *inner,
                other => other,
            };
            draft.field_type = if is_array {
                FieldType::Array(Box::new(clean_base))
            } else {
                clean_base
            };
            changed = true;
        }
        ui.add_space(6.0);
        if styled_checkbox(ui, t, &mut is_array, "Array").changed() {
            let base = draft.base_type().clone();
            draft.field_type = if is_array {
                FieldType::Array(Box::new(base))
            } else {
                base
            };
            changed = true;
        }
    });
    ui.add_space(6.0);

    attr_row(ui, t, "Field identifier", |ui| {
        let mut name_buf = draft.name.clone();
        let mut style = FieldStyle::default()
            .with_hint("field_name")
            .with_outline_preset(FieldOutline::Hairline);
        if name_buf.trim().is_empty() {
            style = style.with_error("Field name cannot be empty");
        } else if def
            .fields
            .iter()
            .enumerate()
            .any(|(i, f)| i != field_idx && f.name == name_buf.trim())
        {
            style = style.with_error("Duplicate field name");
        }
        let res = text_field(ui, &mut name_buf, style);
        if res.committed && !name_buf.trim().is_empty() {
            let trimmed = name_buf.trim().to_string();
            if !def
                .fields
                .iter()
                .enumerate()
                .any(|(i, f)| i != field_idx && f.name == trimmed)
            {
                draft.name = trimmed;
                changed = true;
            }
        }
    });
    ui.add_space(6.0);

    attr_row(ui, t, "Documentation", |ui| {
        let mut doc_buf = draft.doc.clone().unwrap_or_default();
        let res = text_field(
            ui,
            &mut doc_buf,
            FieldStyle::default()
                .with_hint("Field description")
                .with_outline_preset(FieldOutline::Hairline),
        );
        if res.changed {
            let trimmed = doc_buf.trim().to_string();
            draft.doc = (!trimmed.is_empty()).then_some(trimmed);
            changed = true;
        }
    });
    ui.add_space(6.0);

    attr_row(ui, t, "Export to ToC", |ui| {
        if def.export_to_toc {
            if styled_checkbox(ui, t, &mut draft.export_to_toc, "Include in JSON ToC").changed() {
                changed = true;
            }
        } else {
            ui.add_enabled(
                false,
                egui::Label::new(
                    RichText::new("Enable Entity 'Add to JSON Table-of-content' to export")
                        .size(11.0)
                        .color(t.faint),
                ),
            );
        }
    });

    if matches!(
        base_ty,
        FieldType::String | FieldType::Multilines | FieldType::Enum(_)
    ) {
        ui.add_space(6.0);
        attr_row(ui, t, "Searchable", |ui| {
            if styled_checkbox(
                ui,
                t,
                &mut draft.searchable,
                "Searchable in command palette",
            )
            .changed()
            {
                changed = true;
            }
        });
    }
    ui.add_space(10.0);

    // 2. VALUE SPECIFICATIONS (LDtk per-type filtered)
    render_sub_heading(ui, t, "VALUE SPECIFICATIONS");

    render_default_value_row(
        app,
        ui,
        t,
        state,
        def,
        field_idx,
        &base_ty,
        &enums,
        &editor_ctx,
        &mut draft,
        &mut changed,
    );

    // `Can be null` / `Is optional` / `Can contain nulls` — LDtk hides for Bool & Color
    if !matches!(base_ty, FieldType::Bool | FieldType::Color) {
        ui.add_space(6.0);
        let null_label = if draft.is_array() {
            "Can contain nulls"
        } else if matches!(
            base_ty,
            FieldType::Point | FieldType::FilePath | FieldType::EntityRef | FieldType::Tile
        ) {
            "Is optional"
        } else {
            "Can be null"
        };
        attr_row(ui, t, null_label, |ui| {
            if styled_checkbox(ui, t, &mut draft.can_be_null, "").changed() {
                changed = true;
            }
        });
    }

    render_type_constraints(app, ui, t, def, field_idx, &mut draft, &mut changed);

    if draft.is_array() {
        ui.add_space(6.0);
        attr_row(ui, t, "Array length", |ui| {
            let mut has_min = draft.array_min_length.is_some();
            if styled_checkbox(ui, t, &mut has_min, "Min").changed() {
                draft.array_min_length = has_min.then_some(0);
                changed = true;
            }
            if let Some(ref mut v) = draft.array_min_length {
                if ui.add(DragValue::new(v).range(0..=999)).changed() {
                    changed = true;
                }
            }
            ui.add_space(8.0);
            let mut has_max = draft.array_max_length.is_some();
            if styled_checkbox(ui, t, &mut has_max, "Max").changed() {
                draft.array_max_length = has_max.then_some(10);
                changed = true;
            }
            if let Some(ref mut v) = draft.array_max_length {
                if ui.add(DragValue::new(v).range(0..=999)).changed() {
                    changed = true;
                }
            }
        });
    }

    ui.add_space(10.0);
    // 3. DISPLAY
    render_editor_display_settings(ui, t, def, field_idx, &mut draft, &mut changed);

    if changed && draft != *orig {
        app.execute_command(Box::new(UpdateEntityFieldCommand::with_stroke(
            def.id,
            orig.name.clone(),
            draft,
            def.id
                .0
                .wrapping_add(0x4649_4500)
                .wrapping_add(field_idx as u64),
        )));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_default_value_row(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
    field_idx: usize,
    base_ty: &FieldType,
    enums: &[lc_core::defs::EnumDef],
    editor_ctx: &FieldEditorContext<'_>,
    draft: &mut FieldDef,
    changed: &mut bool,
) {
    match base_ty {
        FieldType::Bool => {
            attr_row(ui, t, "Default state", |ui| {
                let mut state_val = matches!(draft.default_value, Some(FieldValue::Bool(true)));
                if styled_checkbox(ui, t, &mut state_val, "").changed() {
                    draft.default_value = Some(FieldValue::Bool(state_val));
                    *changed = true;
                }
            });
        }
        FieldType::Color => {
            attr_row(ui, t, "Default color", |ui| {
                let cur = match draft.default_value {
                    Some(FieldValue::Color(c)) => c,
                    _ => [255, 255, 255, 255],
                };
                let mut col = Color32::from_rgba_unmultiplied(cur[0], cur[1], cur[2], cur[3]);
                if color_swatch(ui, ("field_def_color", def.id, field_idx), &mut col).changed() {
                    draft.default_value =
                        Some(FieldValue::Color([col.r(), col.g(), col.b(), col.a()]));
                    *changed = true;
                }
            });
        }
        FieldType::Tile => {
            attr_row(ui, t, "Tileset", |ui| {
                let ts_items: Vec<TilesetSelectItem<lc_core::id::TilesetDefId>> = app
                    .project
                    .defs
                    .tilesets
                    .iter()
                    .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
                    .collect();
                if tileset_select(ui, &ts_items, &mut draft.tileset_id, true) {
                    *changed = true;
                }
                if draft.tileset_id.is_some() {
                    ui.add_space(6.0);
                    let current_rect = match draft.default_value {
                        Some(FieldValue::Tile(r)) => r,
                        _ => None,
                    };
                    let icon = resolve_tile_icon(app, ui, current_rect);
                    if tile_icon_button(ui, icon)
                        .on_hover_text("Pick default tile")
                        .clicked()
                    {
                        let seed = current_rect.or_else(|| {
                            draft
                                .tileset_id
                                .map(|id| TilesetRect::new(id, 0, 0, 16, 16))
                        });
                        open_tile_picker(
                            app,
                            state,
                            def.id,
                            EntityTileSlot::FieldDefaultTile(field_idx),
                            seed,
                        );
                    }
                    if current_rect.is_some()
                        && icon_button(
                            ui,
                            Icon::Close,
                            ICON_BUTTON_SIZE_FORM,
                            IconButtonStyle::bordered(),
                        )
                        .on_hover_text("Clear default tile")
                        .clicked()
                    {
                        draft.default_value = None;
                        *changed = true;
                    }
                }
            });
        }
        FieldType::Int
        | FieldType::Float
        | FieldType::String
        | FieldType::Multilines
        | FieldType::Enum(_) => {
            attr_row(ui, t, "Default value", |ui| {
                let mut has_default = draft.default_value.is_some();
                if styled_checkbox(ui, t, &mut has_default, "Custom").changed() {
                    draft.default_value = has_default
                        .then(|| lc_feature_entities::default_field_value(base_ty, enums));
                    *changed = true;
                }
                if let Some(ref mut val) = draft.default_value {
                    ui.add_space(6.0);
                    let resp = field_value_editor(ui, val, base_ty, editor_ctx);
                    if resp.changed {
                        *changed = true;
                    }
                }
            });
        }
        FieldType::Point | FieldType::EntityRef | FieldType::FilePath | FieldType::Array(_) => {}
    }
}
