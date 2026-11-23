//! Pinned detail footer with enum usage counts and Save / Revert action buttons.

use std::time::Instant;

use egui::{vec2, Align, Color32, CornerRadius, Frame, Layout, Panel, RichText, Stroke, Ui};

use lc_canvas::theme::Tokens;
use lc_core::id::EnumDefId;

use super::file_ops::get_default_enum_file_path;
use super::types::EnumViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::{
    convert_project_enums_to_file, validate_enum_file_data, write_enum_file_atomic,
};
use crate::features::enums::metrics::{CONTROL_HEIGHT_COMPACT, WORKSPACE_PADDING};

pub fn render_detail_footer(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut EnumViewState,
    selected_enum_info: Option<&(EnumDefId, String, usize)>,
    entity_count: usize,
    field_count: usize,
) {
    let t = Tokens::get(ui.ctx());

    Panel::bottom("enum_mode_detail_footer")
        .exact_size(48.0)
        .frame(Frame::new().fill(t.panel))
        .show_separator_line(true)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(WORKSPACE_PADDING);
                if let Some(e) = selected_enum_info {
                    let mut info = format!(
                        "ID: EnumDefId({}) • Used by {} Entity Def(s) & {} Field(s)",
                        e.0 .0, entity_count, field_count
                    );
                    if app.enum_is_dirty() {
                        info.push_str(" • Unsaved changes");
                    }
                    ui.label(RichText::new(info).size(11.5).color(t.muted));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(WORKSPACE_PADDING);

                    let is_dirty = app.enum_is_dirty();

                    // "Save Enum Def" button
                    let save_btn = ui.add_enabled(
                        is_dirty,
                        egui::Button::new(
                            RichText::new("Save Enum Def")
                                .size(12.0)
                                .strong()
                                .color(if is_dirty { t.invert_fg } else { t.muted }),
                        )
                        .fill(if is_dirty { t.accent } else { t.sunken })
                        .corner_radius(CornerRadius::same(3))
                        .min_size(vec2(110.0, CONTROL_HEIGHT_COMPACT)),
                    );
                    if save_btn.clicked() {
                        if let Some(open_id) = app.selected_enum_def {
                            if let Some(open_def) =
                                app.enum_draft.iter().find(|e| e.id == open_id).cloned()
                            {
                                if let Some(pos) =
                                    app.project.defs.enums.iter().position(|e| e.id == open_id)
                                {
                                    app.project.defs.enums[pos] = open_def.clone();
                                } else {
                                    app.project.defs.enums.push(open_def.clone());
                                }
                                app.enum_saved_defs.insert(open_id, open_def.clone());
                                if let Some(pos) =
                                    app.enum_saved.iter().position(|e| e.id == open_id)
                                {
                                    app.enum_saved[pos] = open_def.clone();
                                } else {
                                    app.enum_saved.push(open_def.clone());
                                }
                                if let Some(draft) = app.enum_drafts.get_mut(&open_id) {
                                    draft.saved_revision = draft.revision;
                                }
                                for ((eid, vid), rect) in &app.enum_value_tile_rects {
                                    if *eid == open_id {
                                        app.enum_saved_tile_rects
                                            .insert((*eid, vid.clone()), *rect);
                                    }
                                }
                            }
                        }

                        let save_path = get_default_enum_file_path(app);
                        let resolver = |tid| app.resolve_tileset_image_size(tid);
                        let file_data = convert_project_enums_to_file(
                            &app.project.defs.enums,
                            &app.project.defs.tilesets,
                            Some(&app.enum_tileset_map),
                            &resolver,
                        );

                        match validate_enum_file_data(&file_data) {
                            Ok(()) => match write_enum_file_atomic(&save_path, &file_data) {
                                Ok(()) => {
                                    app.enum_unsaved = false;
                                    app.status_message = format!(
                                        "Saved enum definition to '{}'",
                                        save_path.display()
                                    );
                                    state.toast = Some((
                                        "Enum definition saved successfully".to_string(),
                                        Instant::now(),
                                    ));
                                }
                                Err(e) => {
                                    state.error_modal =
                                        Some(vec![format!("Failed to save enum definition: {e}")]);
                                }
                            },
                            Err(errs) => {
                                state.error_modal = Some(errs);
                            }
                        }
                    }

                    ui.add_space(6.0);

                    // "Revert" button
                    let revert_btn = ui.add_enabled(
                        is_dirty,
                        egui::Button::new(RichText::new("Revert").size(12.0).color(t.ink))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(
                                1.0,
                                if is_dirty { t.line_strong } else { t.line },
                            ))
                            .corner_radius(CornerRadius::same(3))
                            .min_size(vec2(70.0, CONTROL_HEIGHT_COMPACT)),
                    );
                    if revert_btn.clicked() {
                        state.revert_confirm_open = true;
                    }
                });
            });
        });
}
