//! Editor `DISPLAY` settings sub-section (100% LDtk per-type filtered visibility).

use egui::{ComboBox, DragValue, RichText, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::EntityDef;
use lc_core::fields::{EditorDisplayMode, EditorDisplayPos, EditorLinkStyle, FieldDef, FieldType};

use super::attributes_card::attr_row;
use super::field_config_card::render_sub_heading;
use crate::shared::ui::checkbox::styled_checkbox;

fn valid_display_modes(base: &FieldType, is_array: bool) -> Vec<EditorDisplayMode> {
    let mut modes = vec![
        EditorDisplayMode::Hidden,
        EditorDisplayMode::ValueOnly,
        EditorDisplayMode::NameAndValue,
    ];
    if is_array {
        modes.push(EditorDisplayMode::ArrayCountWithLabel);
        modes.push(EditorDisplayMode::ArrayCountNoLabel);
    }
    if matches!(base, FieldType::Enum(_) | FieldType::Tile) {
        modes.push(EditorDisplayMode::EntityTile);
    }
    if !is_array && matches!(base, FieldType::Int | FieldType::Float) {
        modes.push(EditorDisplayMode::RadiusPx);
        modes.push(EditorDisplayMode::RadiusGrid);
    }
    if matches!(base, FieldType::Point) {
        modes.push(EditorDisplayMode::Points);
        modes.push(EditorDisplayMode::PointStar);
        if is_array {
            modes.push(EditorDisplayMode::PointPath);
            modes.push(EditorDisplayMode::PointPathLoop);
        }
    }
    if matches!(base, FieldType::EntityRef) {
        modes.push(EditorDisplayMode::RefLinkBetweenPivots);
        modes.push(EditorDisplayMode::RefLinkBetweenCenters);
    }
    modes
}

/// Renders the canvas editor `DISPLAY` configuration rows matching LDtk `FieldDefsForm` rules.
pub fn render_editor_display_settings(
    ui: &mut Ui,
    t: &Tokens,
    def: &EntityDef,
    field_idx: usize,
    draft: &mut FieldDef,
    changed: &mut bool,
) {
    render_sub_heading(ui, t, "DISPLAY");
    let base = draft.base_type().clone();
    let modes = valid_display_modes(&base, draft.is_array());
    if !modes.contains(&draft.editor_display_mode) {
        draft.editor_display_mode = EditorDisplayMode::Hidden;
        *changed = true;
    }

    let is_text_display = matches!(
        draft.editor_display_mode,
        EditorDisplayMode::ValueOnly
            | EditorDisplayMode::NameAndValue
            | EditorDisplayMode::ArrayCountWithLabel
            | EditorDisplayMode::ArrayCountNoLabel
    );
    let is_link_display = matches!(
        draft.editor_display_mode,
        EditorDisplayMode::PointStar
            | EditorDisplayMode::PointPath
            | EditorDisplayMode::PointPathLoop
            | EditorDisplayMode::RefLinkBetweenPivots
            | EditorDisplayMode::RefLinkBetweenCenters
    );

    attr_row(ui, t, "Display in editor", |ui| {
        ComboBox::from_id_salt(("field_display_mode", def.id, field_idx))
            .selected_text(format!("{:?}", draft.editor_display_mode))
            .show_ui(ui, |ui| {
                for m in &modes {
                    if ui
                        .selectable_value(&mut draft.editor_display_mode, *m, format!("{m:?}"))
                        .clicked()
                    {
                        *changed = true;
                    }
                }
            });

        if is_text_display {
            ui.add_space(6.0);
            ComboBox::from_id_salt(("field_display_pos", def.id, field_idx))
                .selected_text(format!("{:?}", draft.editor_display_pos))
                .show_ui(ui, |ui| {
                    for p in [
                        EditorDisplayPos::Above,
                        EditorDisplayPos::Center,
                        EditorDisplayPos::Beneath,
                    ] {
                        if ui
                            .selectable_value(&mut draft.editor_display_pos, p, format!("{p:?}"))
                            .clicked()
                        {
                            *changed = true;
                        }
                    }
                });
            ui.add_space(6.0);
            ui.label(RichText::new("Scale").color(t.muted));
            if ui
                .add(
                    DragValue::new(&mut draft.editor_display_scale)
                        .speed(0.05)
                        .range(0.1..=4.0),
                )
                .changed()
            {
                *changed = true;
            }
        }
    });

    if is_link_display {
        ui.add_space(6.0);
        attr_row(ui, t, "Link style", |ui| {
            ComboBox::from_id_salt(("field_link_style", def.id, field_idx))
                .selected_text(format!("{:?}", draft.editor_link_style))
                .show_ui(ui, |ui| {
                    for ls in [
                        EditorLinkStyle::StraightArrow,
                        EditorLinkStyle::CurvedArrow,
                        EditorLinkStyle::ArrowsLine,
                        EditorLinkStyle::DashedLine,
                        EditorLinkStyle::ZigZag,
                    ] {
                        if ui
                            .selectable_value(&mut draft.editor_link_style, ls, format!("{ls:?}"))
                            .clicked()
                        {
                            *changed = true;
                        }
                    }
                });
        });
    }

    if matches!(base, FieldType::Int | FieldType::Float) {
        ui.add_space(6.0);
        attr_row(ui, t, "Prefix / suffix", |ui| {
            let mut prefix_buf = draft.editor_text_prefix.clone().unwrap_or_default();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut prefix_buf)
                        .desired_width(64.0)
                        .hint_text("$"),
                )
                .changed()
            {
                let trimmed = prefix_buf.trim().to_string();
                draft.editor_text_prefix = (!trimmed.is_empty()).then_some(trimmed);
                *changed = true;
            }
            ui.label(RichText::new("1234").color(t.faint));
            let mut suffix_buf = draft.editor_text_suffix.clone().unwrap_or_default();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut suffix_buf)
                        .desired_width(64.0)
                        .hint_text("px"),
                )
                .changed()
            {
                let trimmed = suffix_buf.trim().to_string();
                draft.editor_text_suffix = (!trimmed.is_empty()).then_some(trimmed);
                *changed = true;
            }
        });
    }

    ui.add_space(6.0);
    attr_row(ui, t, "Always show", |ui| {
        if draft.editor_display_mode != EditorDisplayMode::Hidden {
            if styled_checkbox(ui, t, &mut draft.editor_always_show, "").changed() {
                *changed = true;
            }
        } else {
            ui.label(
                RichText::new("Select a display mode above to enable")
                    .size(11.0)
                    .color(t.faint),
            );
        }
    });

    if matches!(base, FieldType::String | FieldType::Multilines) {
        ui.add_space(6.0);
        attr_row(ui, t, "Only show excerpt", |ui| {
            if styled_checkbox(ui, t, &mut draft.editor_cut_long_values, "Cut long values")
                .changed()
            {
                *changed = true;
            }
        });
    }

    if matches!(base, FieldType::Color | FieldType::Enum(_)) {
        ui.add_space(6.0);
        attr_row(ui, t, "Override Entity color", |ui| {
            if styled_checkbox(ui, t, &mut draft.use_for_smart_color, "").changed() {
                *changed = true;
            }
        });
    }
}
