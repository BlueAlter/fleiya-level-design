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
use crate::features::enums::file::EnumFileDialogManager;
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

    // 2. DETAIL PANEL: Pinned footer via TopBottomPanel::bottom + CentralPanel
    let selected_enum_info = app
        .selected_enum_def
        .and_then(|id| app.enum_draft.iter().find(|e| e.id == id))
        .map(|e| (e.id, e.name.clone(), e.values.len()));

    let (entity_count, field_count) = if let Some(ref info) = selected_enum_info {
        match state.cached_usage {
            Some((cid, vcnt, ec, fc)) if cid == info.0 && vcnt == info.2 => (ec, fc),
            _ => {
                let (ec, fc) = compute_enum_usage(&app.project, info.0);
                state.cached_usage = Some((info.0, info.2, ec, fc));
                (ec, fc)
            }
        }
    } else {
        (0, 0)
    };

    let sections = sections_for(state.active_sub_tab);

    // A. Footer docked at bottom of detail area
    if sections.footer {
        render_detail_footer(
            app,
            ui,
            &mut state,
            selected_enum_info.as_ref(),
            entity_count,
            field_count,
        );
    }

    // B. Central content area filling the space above the footer
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
                    ui.add_space(8.0);
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

    if state.tag_editing_id.is_some() || state.tag_inline_new_active {
        input_blocker(ui.ctx());
    }

    state.previous_selected_enum = app.selected_enum_def;
    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
}
