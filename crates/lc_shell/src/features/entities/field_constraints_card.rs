//! Type-specific `VALUE SPECIFICATIONS` constraints (Limits, Regex, Language, FileTypes, EntityRef).

use egui::{ComboBox, DragValue, RichText, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::EntityDef;
use lc_core::fields::{AllowedRefs, FieldDef, FieldType, TextLanguageMode};

use super::attributes_card::attr_row;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::text_field::{text_field, FieldOutline, FieldStyle};

/// Renders type-specific constraint rows inside the `VALUE SPECIFICATIONS` section.
pub fn render_type_constraints(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    def: &EntityDef,
    field_idx: usize,
    draft: &mut FieldDef,
    changed: &mut bool,
) {
    match draft.base_type().clone() {
        FieldType::Int | FieldType::Float => {
            ui.add_space(6.0);
            attr_row(ui, t, "Limits", |ui| {
                let mut has_min = draft.min.is_some();
                if styled_checkbox(ui, t, &mut has_min, "Min").changed() {
                    draft.min = has_min.then_some(0.0);
                    *changed = true;
                }
                if let Some(ref mut min_val) = draft.min {
                    if ui.add(DragValue::new(min_val).speed(0.5)).changed() {
                        *changed = true;
                    }
                }
                ui.add_space(8.0);
                ui.label(RichText::new("to").color(t.muted));
                ui.add_space(8.0);
                let mut has_max = draft.max.is_some();
                if styled_checkbox(ui, t, &mut has_max, "Max").changed() {
                    draft.max = has_max.then_some(100.0);
                    *changed = true;
                }
                if let Some(ref mut max_val) = draft.max {
                    if ui.add(DragValue::new(max_val).speed(0.5)).changed() {
                        *changed = true;
                    }
                }
            });
        }
        FieldType::String => {
            ui.add_space(6.0);
            attr_row(ui, t, "Regex check", |ui| {
                let mut regex_buf = draft.regex.clone().unwrap_or_default();
                let res = text_field(
                    ui,
                    &mut regex_buf,
                    FieldStyle::default()
                        .with_hint("(none)")
                        .with_outline_preset(FieldOutline::Hairline),
                );
                if res.changed {
                    let trimmed = regex_buf.trim().to_string();
                    draft.regex = (!trimmed.is_empty()).then_some(trimmed);
                    *changed = true;
                }
            });
        }
        FieldType::Multilines => {
            ui.add_space(6.0);
            attr_row(ui, t, "Language", |ui| {
                let mut lang = draft.text_language_mode.unwrap_or_default();
                ComboBox::from_id_salt(("field_lang_mode", def.id, field_idx))
                    .selected_text(format!("{lang:?}"))
                    .show_ui(ui, |ui| {
                        for m in [
                            TextLanguageMode::Plain,
                            TextLanguageMode::Markdown,
                            TextLanguageMode::Json,
                            TextLanguageMode::Lua,
                            TextLanguageMode::Python,
                            TextLanguageMode::Js,
                            TextLanguageMode::C,
                            TextLanguageMode::Haxe,
                            TextLanguageMode::Ruby,
                            TextLanguageMode::Xml,
                            TextLanguageMode::Log,
                        ] {
                            if ui
                                .selectable_value(&mut lang, m, format!("{m:?}"))
                                .clicked()
                            {
                                draft.text_language_mode = Some(lang);
                                *changed = true;
                            }
                        }
                    });
            });
        }
        FieldType::FilePath => {
            ui.add_space(6.0);
            attr_row(ui, t, "File extensions", |ui| {
                let mut exts_buf = draft.accept_file_types.join(", ");
                let res = text_field(
                    ui,
                    &mut exts_buf,
                    FieldStyle::default()
                        .with_hint("Any file (e.g. png, json)")
                        .with_outline_preset(FieldOutline::Hairline),
                );
                if res.changed {
                    draft.accept_file_types = exts_buf
                        .split(',')
                        .map(|s| s.trim().trim_start_matches('.').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    *changed = true;
                }
            });
        }
        FieldType::Point => {
            ui.add_space(6.0);
            attr_row(ui, t, "Point bounds", |ui| {
                if styled_checkbox(ui, t, &mut draft.allow_out_of_bounds, "Allow out of bounds")
                    .changed()
                {
                    *changed = true;
                }
            });
        }
        FieldType::EntityRef => {
            ui.add_space(6.0);
            attr_row(ui, t, "Reference options", |ui| {
                if draft.allowed_refs == AllowedRefs::OnlySame {
                    if styled_checkbox(ui, t, &mut draft.symmetrical_ref, "Symmetrical").changed() {
                        *changed = true;
                    }
                } else {
                    draft.symmetrical_ref = false;
                    ui.label(
                        RichText::new("Symmetrical (OnlySame)")
                            .size(11.0)
                            .color(t.faint),
                    );
                }
                ui.add_space(10.0);
                if styled_checkbox(ui, t, &mut draft.auto_chain_ref, "Auto-chain").changed() {
                    *changed = true;
                }
            });
            ui.add_space(6.0);
            attr_row(ui, t, "Allowed references", |ui| {
                ComboBox::from_id_salt(("field_allowed_refs", def.id, field_idx))
                    .selected_text(format!("{:?}", draft.allowed_refs))
                    .show_ui(ui, |ui| {
                        for ar in [
                            AllowedRefs::OnlySame,
                            AllowedRefs::Any,
                            AllowedRefs::OnlySpecificEntity,
                            AllowedRefs::OnlyTags,
                        ] {
                            if ui
                                .selectable_value(&mut draft.allowed_refs, ar, format!("{ar:?}"))
                                .clicked()
                            {
                                if ar != AllowedRefs::OnlySame {
                                    draft.symmetrical_ref = false;
                                }
                                *changed = true;
                            }
                        }
                    });
            });
            if draft.allowed_refs == AllowedRefs::OnlySpecificEntity {
                ui.add_space(4.0);
                attr_row(ui, t, "Target Entity", |ui| {
                    let target_name = draft
                        .allowed_refs_entity_id
                        .and_then(|id| app.project.defs.entity(id))
                        .map(|e| e.name.as_str())
                        .unwrap_or("Select Entity...");
                    ComboBox::from_id_salt(("field_ref_target", def.id, field_idx))
                        .selected_text(target_name)
                        .show_ui(ui, |ui| {
                            for ent in &app.project.defs.entities {
                                if ui
                                    .selectable_value(
                                        &mut draft.allowed_refs_entity_id,
                                        Some(ent.id),
                                        &ent.name,
                                    )
                                    .clicked()
                                {
                                    *changed = true;
                                }
                            }
                        });
                });
            } else if draft.allowed_refs == AllowedRefs::OnlyTags {
                ui.add_space(4.0);
                attr_row(ui, t, "Target Tags", |ui| {
                    let mut tags_buf = draft.allowed_ref_tags.join(", ");
                    let res = text_field(
                        ui,
                        &mut tags_buf,
                        FieldStyle::default()
                            .with_hint("comma-separated tags")
                            .with_outline_preset(FieldOutline::Hairline),
                    );
                    if res.changed {
                        draft.allowed_ref_tags = tags_buf
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        *changed = true;
                    }
                });
            }
        }
        _ => {}
    }
}
