//! Enum Definitions workspace view, detail editor, and modal dialogs.

pub mod attributes;
pub mod file_ops;
pub mod footer;
pub mod left_panel;
pub mod left_panel_preview;
pub mod modals;
pub mod tile_picker;
pub mod types;
pub mod values;

pub use attributes::*;
pub use file_ops::*;
pub use footer::*;
pub use left_panel::*;
pub use modals::*;
pub use tile_picker::*;
pub use types::*;
pub use values::*;

use egui::{CursorIcon, Frame, Id, Margin, ScrollArea, Ui};
use lc_canvas::theme::Tokens;

use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::{
    convert_project_enums_to_file, validate_enum_file_data, write_enum_file_atomic,
    EnumFileDialogManager,
};
use crate::features::enums::i18n;
use crate::features::enums::metrics::WORKSPACE_PADDING;
use crate::shared::ui::tag_input::input_blocker;

/// Primary entrypoint rendering the Enum Definitions workspace.
pub fn render_enum_view(app: &mut ShellApp, ui: &mut Ui) {
    i18n::register();
    let t = Tokens::get(ui.ctx());
    let state_id = Id::new("enum_mode_view_transient_state");
    let mut state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<EnumViewState>(state_id).unwrap_or_default());

    let dialog_mgr = Id::new("enum_file_dialog_mgr");
    let dialog_manager = ui.ctx().data_mut(|d| {
        d.get_temp::<std::sync::Arc<EnumFileDialogManager>>(dialog_mgr)
            .unwrap_or_else(|| {
                let m = std::sync::Arc::new(EnumFileDialogManager::new());
                d.insert_temp(dialog_mgr, m.clone());
                m
            })
    });

    // 0. Poll background file dialog results
    if let Some(msg) = dialog_manager.poll() {
        handle_file_dialog_message(app, &mut state, msg);
    }

    // Ensure draft is initialized from project if needed
    if app.enum_draft.is_empty() && !app.project.defs.enums.is_empty() {
        app.enum_draft = app.project.defs.enums.clone();
        app.enum_saved = app.project.defs.enums.clone();
        app.enum_saved_revision = app.enum_draft_revision;
    }

    // Ensure a valid enum is selected from draft
    let enums = &app.enum_draft;
    if app.selected_enum_def.is_none() || !enums.iter().any(|e| Some(e.id) == app.selected_enum_def)
    {
        app.selected_enum_def = enums
            .iter()
            .find(|e| e.name == "ItemRarity")
            .map(|e| e.id)
            .or_else(|| enums.first().map(|e| e.id));
    }

    // Reinforce text selection disabling on all styles
    ui.ctx().all_styles_mut(|s| {
        s.interaction.selectable_labels = false;
        s.interaction.multi_widget_text_select = false;
    });

    // 1. LEFT PANEL: Enum cards list & quick actions
    render_left_panel(app, ui, &mut state, &dialog_manager);

    // 2. DETAIL PANEL: CentralPanel with definition form or document tab
    let sections = sections_for(state.active_sub_tab);

    let sel_id = app.selected_enum_def;
    let selected_idx = sel_id.and_then(|id| app.enum_draft.iter().position(|e| e.id == id));

    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(t.bg)
                .inner_margin(Margin::same(WORKSPACE_PADDING as i8)),
        )
        .show(ui, |ui| {
            ui.set_min_width(0.0);
            ui.set_max_width(ui.available_width());

            if let Some(enum_idx) = selected_idx {
                if sections.title {
                    ScrollArea::vertical()
                        .id_salt("enum_right_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_attributes(app, ui, &mut state, &dialog_manager, enum_idx);
                            ui.add_space(10.0);
                            render_values(app, ui, &mut state, enum_idx);
                        });
                } else {
                    enum_sub_tabs(
                        ui,
                        &t,
                        &mut state.active_sub_tab,
                        &[
                            crate::shared::i18n::tr(
                                crate::features::enums::i18n::keys::ENUMS_TAB_DEFINITION,
                            ),
                            crate::shared::i18n::tr(
                                crate::features::enums::i18n::keys::ENUMS_TAB_DOCUMENT,
                            ),
                        ],
                    );
                    // CommonMarkViewer prepends a 15 px newline before the first heading.
                    // Compensate so the first heading lands at y = 52.0, matching the ENUM DEF title.
                    ui.add_space(8.0 - 15.0);
                    render_enum_document(ui, &t, &state.doc_cache, &mut state.doc_scroll);
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        egui::RichText::new(
                            "Select an enum definition from the left panel to edit.",
                        )
                        .color(t.muted),
                    );
                });
            }
        });

    // Reset color picker drag tracking on mouse release
    if ui.input(|i| i.pointer.any_released()) {
        state.color_picker_dragging = false;
    }

    // 3. Confirmation modals and TilePicker
    render_modals(app, ui, &mut state);

    // If any drag is active, enforce Grabbing cursor at the end
    if state.value_drag.is_active() || state.enum_drag.is_active() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }

    sync_enum_draft_shadows(app);

    // Auto-save: keep project.defs.enums and app.enum_draft in sync and mark project dirty on changes
    if app.enum_draft_revision > app.enum_saved_revision
        || app.enum_draft != app.project.defs.enums
    {
        if app.enum_draft_revision > app.enum_saved_revision || !app.enum_draft.is_empty() {
            app.project.defs.enums = app.enum_draft.clone();
            app.is_dirty = true;
            app.enum_saved = app.enum_draft.clone();
            app.enum_saved_revision = app.enum_draft_revision;
            app.enum_saved_tile_rects = app.enum_value_tile_rects.clone();
            for e in &app.enum_draft {
                app.enum_saved_defs.insert(e.id, e.clone());
            }
            app.enum_unsaved = false;

            let save_path = get_default_enum_file_path(app);
            if save_path.exists() {
                let resolver = |tid| app.resolve_tileset_image_size(tid);
                let file_data = convert_project_enums_to_file(
                    &app.project.defs.enums,
                    &app.project.defs.tilesets,
                    Some(&app.enum_tileset_map),
                    &resolver,
                );
                if let Ok(()) = validate_enum_file_data(&file_data) {
                    let _ = write_enum_file_atomic(&save_path, &file_data);
                }
            }
        } else {
            app.enum_draft = app.project.defs.enums.clone();
            app.enum_saved = app.project.defs.enums.clone();
            app.enum_saved_revision = app.enum_draft_revision;
        }
    }

    if state.tag_editing_id.is_some() || state.tag_inline_new_active {
        input_blocker(ui.ctx());
    }

    state.previous_selected_enum = app.selected_enum_def;
    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Context;
    use lc_core::defs::{EnumDef, EnumValueDef};
    use lc_core::id::EnumDefId;
    use lc_core::Project;

    #[test]
    fn test_render_enum_view_auto_saves_to_project() {
        let mut project = Project::new("TestProject");
        let enum_id = EnumDefId(1);
        let initial_enum = EnumDef {
            id: enum_id,
            name: "ItemRarity".to_string(),
            tileset_id: None,
            tags: vec![],
            values: vec![EnumValueDef {
                id: "Common".to_string(),
                name: "Common".to_string(),
                color: [255, 255, 255, 255],
                tile: None,
            }],
        };
        project.defs.enums.push(initial_enum.clone());

        let mut app = ShellApp::new(project);
        app.selected_enum_def = Some(enum_id);
        app.is_dirty = false;

        // Simulate an in-place draft edit (e.g. changing identifier name)
        app.enum_draft[0].name = "UpdatedRarity".to_string();
        app.enum_draft_revision += 1;

        assert_eq!(app.project.defs.enums[0].name, "ItemRarity");
        assert!(!app.is_dirty);

        let ctx = Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            render_enum_view(&mut app, ui);
        });
        output.textures_delta.clear();

        // Verify that after render, the edit auto-saved to app.project.defs.enums and marked is_dirty
        assert_eq!(app.project.defs.enums[0].name, "UpdatedRarity");
        assert!(app.is_dirty, "Project must be marked dirty after auto-save");
        assert_eq!(app.enum_saved_revision, app.enum_draft_revision);
    }
}
