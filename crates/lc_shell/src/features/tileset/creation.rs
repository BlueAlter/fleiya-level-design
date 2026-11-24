//! Validated creation of tileset definitions from picked PNG images.

use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_feature_tileset::generate_unique_tileset_name;
use std::path::{Path, PathBuf};
use std::{fs::copy, fs::create_dir_all};

/// Error variants encountered when creating a tileset definition from an image file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateTilesetError {
    /// The file does not have a `.png` extension.
    NotPng,
    /// The file is located outside the active project root directory.
    OutsideProject,
    /// The file cannot be read from disk or decoded as an image.
    Unreadable,
    /// The image width or height is less than 1 pixel.
    TooSmall,
}

impl std::fmt::Display for CreateTilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPng => write!(f, "File must be a PNG image (.png)"),
            Self::OutsideProject => write!(f, "Selected image is outside the project root"),
            Self::Unreadable => write!(f, "Could not read or decode image file"),
            Self::TooSmall => write!(f, "Image dimensions must be at least 1x1"),
        }
    }
}

impl std::error::Error for CreateTilesetError {}

/// Pure function validating and creating a new `TilesetDef` from a picked image path.
///
/// Returns `Ok(TilesetDef)` with a unique name derived from the file stem, relative path,
/// default tile size 16x16, spacing 0, padding 0.
pub fn create_tileset_from_png(
    project_root: &Path,
    picked_path: &Path,
    existing_names: &[String],
) -> Result<TilesetDef, CreateTilesetError> {
    // 1. Extension check
    let is_png = picked_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false);
    if !is_png {
        return Err(CreateTilesetError::NotPng);
    }

    // 2. Read bytes
    let bytes = lc_io::read_bytes(picked_path).map_err(|_| CreateTilesetError::Unreadable)?;

    // 3. Decode dimensions
    let img = image::load_from_memory(&bytes).map_err(|_| CreateTilesetError::Unreadable)?;
    let (w, h) = (img.width(), img.height());
    if w < 1 || h < 1 {
        return Err(CreateTilesetError::TooSmall);
    }

    // 4. Verify inside project root
    let canonical_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let canonical_file = picked_path
        .canonicalize()
        .unwrap_or_else(|_| picked_path.to_path_buf());

    let Ok(rel) = canonical_file.strip_prefix(&canonical_root) else {
        return Err(CreateTilesetError::OutsideProject);
    };

    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let stem = picked_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("tileset");
    let unique_name = generate_unique_tileset_name(existing_names, stem);

    Ok(TilesetDef {
        id: TilesetDefId(0),
        name: unique_name,
        path: rel_str,
        tile_size: glam::UVec2::splat(16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
    })
}

/// Copies an external image into `<project_root>/assets/<file_name>` and returns the destination path.
pub fn copy_image_to_assets(
    project_root: &Path,
    picked_path: &Path,
) -> Result<PathBuf, std::io::Error> {
    let assets_dir = project_root.join("assets");
    if !assets_dir.exists() {
        create_dir_all(&assets_dir)?;
    }
    let file_name = picked_path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing file name")
    })?;
    let dest_path = assets_dir.join(file_name);
    copy(picked_path, &dest_path)?;
    Ok(dest_path)
}

/// Renders the confirmation modal when a picked image is outside the project root.
pub fn render_outside_copy_modal(
    app: &mut crate::ShellApp,
    ui: &mut egui::Ui,
    _t: &lc_canvas::theme::Tokens,
) {
    let Some(picked_path) = app.tileset_view_state.pending_outside_copy.clone() else {
        return;
    };

    let buttons = [
        crate::shared::ui::modal::ModalButton::new(
            0,
            "Cancel",
            crate::shared::ui::modal::ModalButtonKind::Normal,
        ),
        crate::shared::ui::modal::ModalButton::new(
            1,
            "Copy",
            crate::shared::ui::modal::ModalButtonKind::Primary,
        ),
    ];
    let spec = crate::shared::ui::modal::ModalSpec {
        title: "Copy Image to Project?".to_string(),
        body: |ui: &mut egui::Ui| {
            ui.label("Copy the image into the project's assets folder?");
        },
        buttons: &buttons,
        width: 380.0,
        default_button: Some(1),
    };
    let res = crate::shared::ui::modal::modal(ui.ctx(), "tileset_copy_outside_confirm_modal", spec);

    if let Some(btn_idx) = res {
        app.tileset_view_state.pending_outside_copy = None;
        if btn_idx == 1 {
            let project_root = app
                .project_path
                .as_deref()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new("."));
            match copy_image_to_assets(project_root, &picked_path) {
                Ok(copied_path) => {
                    let existing: Vec<String> = app
                        .project
                        .defs
                        .tilesets
                        .iter()
                        .map(|t| t.name.clone())
                        .collect();
                    match create_tileset_from_png(project_root, &copied_path, &existing) {
                        Ok(mut def) => {
                            let next_id = TilesetDefId(
                                app.project
                                    .defs
                                    .tilesets
                                    .iter()
                                    .map(|ts| ts.id.0)
                                    .max()
                                    .unwrap_or(0)
                                    + 1,
                            );
                            def.id = next_id;
                            app.execute_command(Box::new(
                                lc_feature_tileset::AddTilesetDefCommand::new(def),
                            ));
                            app.selected_tileset_def = Some(next_id);
                            app.tileset_view_state.select_tileset(next_id);
                            app.tileset_view_state.reset_atlas_fit(next_id);
                            app.tileset_view_state.error_message = None;
                        }
                        Err(e) => {
                            app.tileset_view_state.error_message = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    app.tileset_view_state.error_message =
                        Some(format!("Failed to copy image to assets folder: {e}"));
                }
            }
        }
    }
}
