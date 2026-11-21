//! Confirmation modal dialogs (Revert, Change Tileset, Delete, Conflict, Error) and TilePicker.

use std::time::Instant;

use egui::{Color32, RichText, ScrollArea, Ui};

use lc_canvas::theme::Tokens;

use super::file_ops::apply_import_with_conflicts;
use super::tile_picker::render_tile_picker_modal;
use super::types::{DeleteTarget, EnumViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::{compute_lost_keys, ConflictResolutionChoice};
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandardModalButton {
    Confirm,
    Cancel,
}

pub fn render_modals(app: &mut ShellApp, ui: &mut Ui, state: &mut EnumViewState) {
    let t = Tokens::get(ui.ctx());

    // 1. Tile picker pop-up modal
    render_tile_picker_modal(app, ui.ctx(), state);

    // =========================================================================
    // 2. REVERT CONFIRMATION MODAL
    // =========================================================================
    if state.revert_confirm_open {
        let buttons = [
            ModalButton::new(
                StandardModalButton::Confirm,
                "Discard",
                ModalButtonKind::Danger,
            ),
            ModalButton::new(
                StandardModalButton::Cancel,
                "Cancel",
                ModalButtonKind::Normal,
            ),
        ];
        let spec = ModalSpec {
            title: "Discard Changes?".to_string(),
            body: |ui: &mut Ui| {
                ui.label("Discard all unsaved enum changes? This cannot be undone.");
            },
            buttons: &buttons,
            width: 320.0,
            default_button: Some(StandardModalButton::Confirm),
        };
        if let Some(res) = modal(ui.ctx(), "revert_enum_confirm_modal", spec) {
            match res {
                StandardModalButton::Confirm => {
                    if let Some(open_id) = app.selected_enum_def {
                        if let Some(saved) =
                            app.enum_saved_defs.get(&open_id).cloned().or_else(|| {
                                app.project
                                    .defs
                                    .enums
                                    .iter()
                                    .find(|e| e.id == open_id)
                                    .cloned()
                            })
                        {
                            if let Some(slot) = app.enum_draft.iter_mut().find(|e| e.id == open_id)
                            {
                                *slot = saved.clone();
                            }
                            if let Some(draft) = app.enum_drafts.get_mut(&open_id) {
                                draft.def = saved;
                                draft.revision = draft.saved_revision;
                            }
                        }
                        app.enum_value_tile_rects
                            .retain(|(eid, _), _| *eid != open_id);
                        for ((eid, vid), rect) in &app.enum_saved_tile_rects {
                            if *eid == open_id {
                                app.enum_value_tile_rects.insert((*eid, vid.clone()), *rect);
                            }
                        }
                        state.toast = Some((
                            "Reverted enum to last saved state".to_string(),
                            Instant::now(),
                        ));
                    }
                    state.revert_confirm_open = false;
                }
                StandardModalButton::Cancel => {
                    state.revert_confirm_open = false;
                }
            }
        }
    }

    // =========================================================================
    // 3. CHANGE TILESET CONFIRMATION MODAL
    // =========================================================================
    if let Some(new_ts) = state.pending_tileset_change {
        let buttons = [
            ModalButton::new(
                StandardModalButton::Confirm,
                "Change Tileset",
                ModalButtonKind::Primary,
            ),
            ModalButton::new(
                StandardModalButton::Cancel,
                "Cancel",
                ModalButtonKind::Normal,
            ),
        ];
        let spec = ModalSpec {
            title: "Change Tileset?".to_string(),
            body: |ui: &mut Ui| {
                ui.label("Changing the tileset will clear all bound tiles in this enum. Continue?");
            },
            buttons: &buttons,
            width: 340.0,
            default_button: Some(StandardModalButton::Confirm),
        };
        if let Some(res) = modal(ui.ctx(), "change_tileset_confirm_modal", spec) {
            match res {
                StandardModalButton::Confirm => {
                    if let Some(sel_id) = app.selected_enum_def {
                        if let Some(ed) = app.enum_draft.iter_mut().find(|e| e.id == sel_id) {
                            ed.tileset_id = new_ts;
                            for v in &mut ed.values {
                                v.tile = None;
                            }
                        }
                        if let Some(draft) = app.enum_drafts.get_mut(&sel_id) {
                            draft.def.tileset_id = new_ts;
                            for v in &mut draft.def.values {
                                v.tile = None;
                            }
                            draft.tile_rects.clear();
                            draft.revision += 1;
                        }
                        if let Some(tid) = new_ts {
                            app.enum_tileset_map.insert(sel_id, tid);
                        } else {
                            app.enum_tileset_map.remove(&sel_id);
                        }
                        app.enum_value_tile_rects
                            .retain(|(eid, _), _| *eid != sel_id);
                        app.enum_draft_revision += 1;
                    }
                    state.pending_tileset_change = None;
                }
                StandardModalButton::Cancel => {
                    state.pending_tileset_change = None;
                }
            }
        }
    }

    // =========================================================================
    // 4. DELETE CONFIRMATION MODAL
    // =========================================================================
    let mut pending_delete_execute = None;
    if let Some(ref target) = state.pending_delete {
        let (title, desc) = match target {
            DeleteTarget::Enum {
                name,
                entity_count,
                field_count,
                ..
            } => {
                let usage_info = if *entity_count > 0 || *field_count > 0 {
                    format!(
                        "\n\nWARNING: Used by {} Entity Def(s) & {} Field(s).",
                        entity_count, field_count
                    )
                } else {
                    String::new()
                };
                (
                    "Delete Enum Definition?".to_string(),
                    format!("Are you sure you want to delete enum '{name}'?{usage_info}"),
                )
            }
            DeleteTarget::EnumValue { val_id, .. } => (
                "Delete Enum Value?".to_string(),
                format!("Are you sure you want to delete value '{val_id}'?"),
            ),
        };

        let buttons = [
            ModalButton::new(
                StandardModalButton::Confirm,
                "Delete",
                ModalButtonKind::Danger,
            ),
            ModalButton::new(
                StandardModalButton::Cancel,
                "Cancel",
                ModalButtonKind::Normal,
            ),
        ];
        let spec = ModalSpec {
            title,
            body: |ui: &mut Ui| {
                ui.label(desc);
            },
            buttons: &buttons,
            width: 320.0,
            default_button: Some(StandardModalButton::Confirm),
        };
        if let Some(res) = modal(ui.ctx(), "enum_delete_confirm_modal", spec) {
            match res {
                StandardModalButton::Confirm => {
                    pending_delete_execute = state.pending_delete.take();
                }
                StandardModalButton::Cancel => {
                    state.pending_delete = None;
                }
            }
        }
    }

    if let Some(target) = pending_delete_execute {
        match target {
            DeleteTarget::Enum { id, .. } => {
                if let Some(pos) = app.enum_draft.iter().position(|e| e.id == id) {
                    app.enum_draft.remove(pos);
                    state.value_uids.remove(&id);
                    app.enum_tags_map.remove(&id);
                    app.cleanup_unused_tags();
                    app.selected_enum_def = app.enum_draft.first().map(|e| e.id);
                    app.enum_draft_revision += 1;
                }
            }
            DeleteTarget::EnumValue {
                enum_id, val_id, ..
            } => {
                if let Some(ed) = app.enum_draft.iter_mut().find(|e| e.id == enum_id) {
                    if let Some(pos) = ed.values.iter().position(|v| v.id == val_id) {
                        ed.values.remove(pos);
                        if let Some(uids) = state.value_uids.get_mut(&enum_id) {
                            if pos < uids.len() {
                                uids.remove(pos);
                            }
                        }
                        app.enum_draft_revision += 1;
                    }
                }
            }
        }
    }

    // =========================================================================
    // 5. CONFLICT RESOLUTION MODAL
    // =========================================================================
    if let Some(ref mut cstate) = state.conflict_modal {
        let mut apply_clicked = false;
        let mut cancel_clicked = false;

        let can_apply = cstate.understand_checked
            || !cstate
                .conflicts
                .iter()
                .any(|c| c.choice == ConflictResolutionChoice::Replace);

        let buttons = [
            ModalButton::new(
                StandardModalButton::Confirm,
                "Apply Import",
                ModalButtonKind::Primary,
            ),
            ModalButton::new(
                StandardModalButton::Cancel,
                "Cancel",
                ModalButtonKind::Normal,
            ),
        ];

        let spec = ModalSpec {
            title: "Enum Identifier Conflicts".to_string(),
            body: |ui: &mut Ui| {
                ui.label(
                    "One or more imported enums match existing enum identifiers in your draft.",
                );
                ui.add_space(8.0);

                for c in &mut cstate.conflicts {
                    ui.group(|ui| {
                        ui.label(
                            RichText::new(&c.file_enum.identifier)
                                .strong()
                                .color(t.accent_ink),
                        );
                        if c.entity_count > 0 || c.field_count > 0 {
                            ui.label(
                                RichText::new(format!(
                                    "Used by {} Entity Def(s) & {} Field(s)",
                                    c.entity_count, c.field_count
                                ))
                                .size(11.0)
                                .color(t.muted),
                            );
                        }
                        let lost = compute_lost_keys(&c.existing_values, &c.file_enum.values);
                        if !lost.is_empty() {
                            ui.label(
                                RichText::new(format!(
                                    "Warning: Values to be lost: {}",
                                    lost.join(", ")
                                ))
                                .size(11.0)
                                .color(Color32::from_rgb(255, 193, 77)),
                            );
                        }

                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut c.choice,
                                ConflictResolutionChoice::Replace,
                                "Replace",
                            );
                            ui.radio_value(
                                &mut c.choice,
                                ConflictResolutionChoice::KeepBoth,
                                "Keep Both (Rename)",
                            );
                            ui.radio_value(&mut c.choice, ConflictResolutionChoice::Skip, "Skip");
                        });
                    });
                    ui.add_space(4.0);
                }

                ui.add_space(8.0);
                ui.checkbox(
                    &mut cstate.understand_checked,
                    "I understand that replacing an enum may break existing entity fields.",
                );
            },
            buttons: &buttons,
            width: 440.0,
            default_button: Some(StandardModalButton::Confirm),
        };

        if let Some(res) = modal(ui.ctx(), "enum_conflict_resolution_modal", spec) {
            match res {
                StandardModalButton::Confirm => {
                    if can_apply {
                        apply_clicked = true;
                    }
                }
                StandardModalButton::Cancel => {
                    cancel_clicked = true;
                }
            }
        }

        if apply_clicked {
            let conflict_state_clone = cstate.clone();
            state.conflict_modal = None;
            apply_import_with_conflicts(app, &conflict_state_clone, state);
        } else if cancel_clicked {
            state.conflict_modal = None;
        }
    }

    // =========================================================================
    // 6. ERROR MODAL
    // =========================================================================
    if let Some(ref errs) = state.error_modal {
        let buttons = [ModalButton::new(
            StandardModalButton::Cancel,
            "Close",
            ModalButtonKind::Normal,
        )];
        let errs_clone = errs.clone();
        let spec = ModalSpec {
            title: "Validation Errors".to_string(),
            body: move |ui: &mut Ui| {
                ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for err in &errs_clone {
                        ui.label(RichText::new(format!("• {err}")).size(12.0).color(t.ink));
                    }
                });
            },
            buttons: &buttons,
            width: 420.0,
            default_button: Some(StandardModalButton::Cancel),
        };
        if modal(ui.ctx(), "enum_error_modal", spec).is_some() {
            state.error_modal = None;
        }
    }
}
