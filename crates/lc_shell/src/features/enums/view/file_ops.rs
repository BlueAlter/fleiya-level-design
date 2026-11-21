//! File dialog coordination, background import/export handling, and conflict resolution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use lc_core::id::EnumDefId;

use super::types::{compute_enum_usage, ConflictModalState, EnumViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::{
    convert_file_enum_to_def, convert_project_enums_to_file, convert_single_enum_to_file,
    generate_unique_identifier, read_enum_file, validate_enum_file_data, write_enum_file_atomic,
    ConflictResolutionChoice, EnumConflict, EnumFileDialogMessage, FleiyaEnumFile,
    ENUM_FILE_EXTENSION,
};

/// Resolves default file path for saving/reverting enum definitions.
pub fn get_default_enum_file_path(app: &ShellApp) -> PathBuf {
    if let Some(ref path) = app.project_path {
        if let Some(parent) = path.parent() {
            return parent.join(format!("{}.{}", app.project.name, ENUM_FILE_EXTENSION));
        }
    }
    PathBuf::from(format!("{}.{}", app.project.name, ENUM_FILE_EXTENSION))
}

pub fn handle_file_dialog_message(
    app: &mut ShellApp,
    state: &mut EnumViewState,
    msg: EnumFileDialogMessage,
) {
    match msg {
        EnumFileDialogMessage::SaveEnums { path } => {
            let resolver = |tid| app.resolve_tileset_image_size(tid);
            let file_data = convert_project_enums_to_file(
                &app.enum_draft,
                &app.project.defs.tilesets,
                Some(&app.enum_tileset_map),
                &resolver,
            );
            match write_enum_file_atomic(&path, &file_data) {
                Ok(()) => {
                    app.enum_saved = app.enum_draft.clone();
                    app.project.defs.enums = app.enum_draft.clone();
                    app.enum_saved_revision = app.enum_draft_revision;
                    app.enum_saved_tile_rects = app.enum_value_tile_rects.clone();
                    app.enum_unsaved = false;
                    state.toast = Some((
                        format!("Saved all enums to '{}'", path.display()),
                        Instant::now(),
                    ));
                }
                Err(e) => {
                    state.error_modal = Some(vec![format!("Save error: {e}")]);
                }
            }
        }
        EnumFileDialogMessage::ExportAll { path } => {
            let resolver = |tid| app.resolve_tileset_image_size(tid);
            let file_data = convert_project_enums_to_file(
                &app.enum_draft,
                &app.project.defs.tilesets,
                Some(&app.enum_tileset_map),
                &resolver,
            );
            match validate_enum_file_data(&file_data) {
                Ok(_) => match write_enum_file_atomic(&path, &file_data) {
                    Ok(()) => {
                        state.toast = Some((
                            format!("Exported all enums to '{}'", path.display()),
                            Instant::now(),
                        ));
                    }
                    Err(e) => {
                        state.error_modal = Some(vec![format!("Export write error: {e}")]);
                    }
                },
                Err(errs) => {
                    state.error_modal = Some(errs);
                }
            }
        }
        EnumFileDialogMessage::ExportOne { path, enum_id } => {
            if let Some(enum_def) = app.enum_draft.iter().find(|e| e.id == enum_id) {
                let resolver = |tid| app.resolve_tileset_image_size(tid);
                let icon_ts = app.enum_tileset_map.get(&enum_id).copied();
                let file_data = convert_single_enum_to_file(
                    enum_def,
                    &app.project.defs.tilesets,
                    icon_ts,
                    &resolver,
                );
                match validate_enum_file_data(&file_data) {
                    Ok(_) => match write_enum_file_atomic(&path, &file_data) {
                        Ok(()) => {
                            state.toast = Some((
                                format!(
                                    "Exported enum '{}' to '{}'",
                                    enum_def.name,
                                    path.display()
                                ),
                                Instant::now(),
                            ));
                        }
                        Err(e) => {
                            state.error_modal = Some(vec![format!("Export write error: {e}")]);
                        }
                    },
                    Err(errs) => {
                        state.error_modal = Some(errs);
                    }
                }
            }
        }
        EnumFileDialogMessage::Import { path } => {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let is_valid_ext = Path::new(file_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
                .unwrap_or(false);

            if !is_valid_ext {
                state.error_modal = Some(vec![format!(
                    "Unsupported extension: '{file_name}'. Only .fleiyaenum is accepted."
                )]);
                return;
            }

            match read_enum_file(&path) {
                Ok(file_data) => {
                    let mut conflicts = Vec::new();
                    for fe in &file_data.enums {
                        if let Some(existing) =
                            app.enum_draft.iter().find(|e| e.name == fe.identifier)
                        {
                            let (entity_count, field_count) =
                                compute_enum_usage(&app.project, existing.id);
                            conflicts.push(EnumConflict {
                                file_enum: fe.clone(),
                                existing_id: existing.id,
                                existing_name: existing.name.clone(),
                                existing_values: existing.values.clone(),
                                entity_count,
                                field_count,
                                choice: ConflictResolutionChoice::KeepBoth,
                            });
                        }
                    }

                    if !conflicts.is_empty() {
                        state.conflict_modal = Some(ConflictModalState {
                            file_data,
                            conflicts,
                            understand_checked: false,
                        });
                    } else {
                        apply_direct_import(app, &file_data, state);
                    }
                }
                Err(errs) => {
                    state.error_modal = Some(errs);
                }
            }
        }
    }
}

pub fn apply_direct_import(
    app: &mut ShellApp,
    file_data: &FleiyaEnumFile,
    state: &mut EnumViewState,
) {
    let start_raw_id = app.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1;
    let mut first_new_id = None;
    let tileset_sizes: std::collections::HashMap<lc_core::id::TilesetDefId, (u32, u32)> = app
        .project
        .defs
        .tilesets
        .iter()
        .filter_map(|ts| app.resolve_tileset_image_size(ts.id).map(|sz| (ts.id, sz)))
        .collect();
    let resolver = |tid| tileset_sizes.get(&tid).copied();

    for (next_raw_id, fe) in (start_raw_id..).zip(file_data.enums.iter()) {
        let new_id = EnumDefId(next_raw_id);
        if first_new_id.is_none() {
            first_new_id = Some(new_id);
        }
        let def = convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
        if let Some(uid) = fe.icon_tileset_uid {
            app.enum_tileset_map
                .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
        }
        app.enum_draft.push(def);
    }

    if let Some(sel) = first_new_id {
        app.selected_enum_def = Some(sel);
    }
    app.enum_draft_revision += 1;
    state.toast = Some((
        format!("Imported {} enum(s) into draft", file_data.enums.len()),
        Instant::now(),
    ));
}

pub fn apply_import_with_conflicts(
    app: &mut ShellApp,
    conflict_state: &ConflictModalState,
    state: &mut EnumViewState,
) {
    let mut existing_names: HashSet<String> =
        app.enum_draft.iter().map(|e| e.name.clone()).collect();
    let mut next_raw_id = app.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1;

    let mut replaced_count = 0;
    let mut renamed_count = 0;
    let mut added_count = 0;
    let mut first_imported_id = None;
    let tileset_sizes: std::collections::HashMap<lc_core::id::TilesetDefId, (u32, u32)> = app
        .project
        .defs
        .tilesets
        .iter()
        .filter_map(|ts| app.resolve_tileset_image_size(ts.id).map(|sz| (ts.id, sz)))
        .collect();
    let resolver = |tid| tileset_sizes.get(&tid).copied();

    for fe in &conflict_state.file_data.enums {
        if let Some(c) = conflict_state
            .conflicts
            .iter()
            .find(|c| c.file_enum.identifier == fe.identifier)
        {
            match c.choice {
                ConflictResolutionChoice::Replace => {
                    let updated_def = convert_file_enum_to_def(
                        fe,
                        c.existing_id,
                        &app.project.defs.tilesets,
                        &resolver,
                    );
                    if let Some(pos) = app.enum_draft.iter().position(|e| e.id == c.existing_id) {
                        if let Some(slot) = app.enum_draft.get_mut(pos) {
                            *slot = updated_def;
                        }
                    }
                    if let Some(uid) = fe.icon_tileset_uid {
                        app.enum_tileset_map
                            .insert(c.existing_id, lc_core::id::TilesetDefId(uid as u64));
                    } else {
                        app.enum_tileset_map.remove(&c.existing_id);
                    }
                    if first_imported_id.is_none() {
                        first_imported_id = Some(c.existing_id);
                    }
                    replaced_count += 1;
                }
                ConflictResolutionChoice::KeepBoth => {
                    let unique_name = generate_unique_identifier(&fe.identifier, &existing_names);
                    existing_names.insert(unique_name.clone());
                    let new_id = EnumDefId(next_raw_id);
                    next_raw_id += 1;
                    let mut def =
                        convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
                    def.name = unique_name;
                    if let Some(uid) = fe.icon_tileset_uid {
                        app.enum_tileset_map
                            .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
                    }
                    app.enum_draft.push(def);
                    if first_imported_id.is_none() {
                        first_imported_id = Some(new_id);
                    }
                    renamed_count += 1;
                }
                ConflictResolutionChoice::Skip => {}
            }
        } else {
            let new_id = EnumDefId(next_raw_id);
            next_raw_id += 1;
            let def = convert_file_enum_to_def(fe, new_id, &app.project.defs.tilesets, &resolver);
            existing_names.insert(def.name.clone());
            if let Some(uid) = fe.icon_tileset_uid {
                app.enum_tileset_map
                    .insert(new_id, lc_core::id::TilesetDefId(uid as u64));
            }
            app.enum_draft.push(def);
            if first_imported_id.is_none() {
                first_imported_id = Some(new_id);
            }
            added_count += 1;
        }
    }

    if let Some(sel) = first_imported_id {
        app.selected_enum_def = Some(sel);
    }
    app.enum_draft_revision += 1;

    let total = replaced_count + renamed_count + added_count;
    state.toast = Some((
        format!("Imported {total} enums into draft ({replaced_count} replaced, {renamed_count} renamed)"),
        Instant::now(),
    ));
}
