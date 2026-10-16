//! Load and save logic for multi-file project workspaces.

use crate::error::IoError;
use crate::format::{LevelFile, LevelManifestEntry, ProjectFile, WorldFile, WorldManifestEntry};
use lc_core::{Level, Project, World};
use std::fs;
use std::path::Path;

/// Version currently supported by this binary.
pub const SUPPORTED_VERSION: u32 = 1;

/// Loads a multi-file project workspace from disk.
pub fn load_project(project_file: &Path) -> Result<Project, IoError> {
    let project_dir = project_file.parent().unwrap_or_else(|| Path::new("."));

    // 1. Read project.lcproj
    let proj_content = fs::read_to_string(project_file).map_err(|e| {
        tracing::error!(path = %project_file.display(), error = %e, "Failed to read project file");
        IoError::Fs {
            path: project_file.to_path_buf(),
            source: e,
        }
    })?;

    let project_file_data: ProjectFile = serde_json::from_str(&proj_content).map_err(|e| {
        tracing::error!(path = %project_file.display(), error = %e, "Failed to parse project JSON");
        IoError::Json {
            path: project_file.to_path_buf(),
            source: e,
        }
    })?;

    if project_file_data.version > SUPPORTED_VERSION {
        let err = IoError::UnsupportedVersion {
            version: project_file_data.version,
            max_supported: SUPPORTED_VERSION,
            path: project_file.to_path_buf(),
        };
        tracing::error!(%err, "Unsupported project version");
        return Err(err);
    }

    let mut worlds = Vec::with_capacity(project_file_data.worlds.len());

    // 2. Load each world
    for world_entry in &project_file_data.worlds {
        let world_path = project_dir.join(&world_entry.file);
        let world_dir = world_path.parent().unwrap_or_else(|| Path::new("."));

        let world_content = fs::read_to_string(&world_path).map_err(|e| {
            tracing::error!(path = %world_path.display(), error = %e, "Failed to read world file");
            IoError::Fs {
                path: world_path.clone(),
                source: e,
            }
        })?;

        let world_file_data: WorldFile = serde_json::from_str(&world_content).map_err(|e| {
            tracing::error!(path = %world_path.display(), error = %e, "Failed to parse world JSON");
            IoError::Json {
                path: world_path.clone(),
                source: e,
            }
        })?;

        if world_file_data.version > SUPPORTED_VERSION {
            let err = IoError::UnsupportedVersion {
                version: world_file_data.version,
                max_supported: SUPPORTED_VERSION,
                path: world_path,
            };
            tracing::error!(%err, "Unsupported world version");
            return Err(err);
        }

        let mut levels = Vec::with_capacity(world_file_data.levels.len());

        // 3. Load each level in world
        for level_entry in &world_file_data.levels {
            let level_path = world_dir.join(&level_entry.file);

            let level_content = fs::read_to_string(&level_path).map_err(|e| {
                tracing::error!(path = %level_path.display(), error = %e, "Failed to read level file");
                IoError::Fs {
                    path: level_path.clone(),
                    source: e,
                }
            })?;

            let level_file_data: LevelFile = serde_json::from_str(&level_content).map_err(|e| {
                tracing::error!(path = %level_path.display(), error = %e, "Failed to parse level JSON");
                IoError::Json {
                    path: level_path.clone(),
                    source: e,
                }
            })?;

            if level_file_data.version > SUPPORTED_VERSION {
                let err = IoError::UnsupportedVersion {
                    version: level_file_data.version,
                    max_supported: SUPPORTED_VERSION,
                    path: level_path,
                };
                tracing::error!(%err, "Unsupported level version");
                return Err(err);
            }

            levels.push(Level {
                id: level_file_data.id,
                name: level_file_data.name,
                world_pos: level_file_data.world_pos,
                world_depth: level_file_data.world_depth,
                size_px: level_file_data.size_px,
                layers: level_file_data.layers,
                fields: level_file_data.fields,
            });
        }

        worlds.push(World {
            id: world_file_data.id,
            name: world_file_data.name,
            layout: world_file_data.layout,
            default_grid: world_file_data.default_grid,
            levels,
        });
    }

    Ok(Project {
        version: project_file_data.version,
        name: project_file_data.name,
        defs: project_file_data.defs,
        worlds,
        settings: project_file_data.settings,
    })
}

/// Saves a project workspace to multi-file JSON on disk.
pub fn save_project(project: &Project, project_file: &Path) -> Result<(), IoError> {
    let project_dir = project_file.parent().unwrap_or_else(|| Path::new("."));

    fs::create_dir_all(project_dir).map_err(|e| {
        tracing::error!(path = %project_dir.display(), error = %e, "Failed to create project directory");
        IoError::Fs {
            path: project_dir.to_path_buf(),
            source: e,
        }
    })?;

    let mut world_manifest_entries = Vec::with_capacity(project.worlds.len());

    // 1. Save worlds and their levels
    for world in &project.worlds {
        let world_folder_name = format!("world_{}", world.id.raw());
        let rel_world_dir = Path::new("worlds").join(&world_folder_name);
        let abs_world_dir = project_dir.join(&rel_world_dir);
        let abs_levels_dir = abs_world_dir.join("levels");

        fs::create_dir_all(&abs_levels_dir).map_err(|e| {
            tracing::error!(path = %abs_levels_dir.display(), error = %e, "Failed to create levels directory");
            IoError::Fs {
                path: abs_levels_dir.clone(),
                source: e,
            }
        })?;

        let rel_world_file = rel_world_dir.join("world.json");
        let abs_world_file = project_dir.join(&rel_world_file);

        let mut level_manifest_entries = Vec::with_capacity(world.levels.len());

        for level in &world.levels {
            let level_filename = format!("level_{}.json", level.id.raw());
            let rel_level_file = Path::new("levels").join(&level_filename);
            let abs_level_file = abs_world_dir.join(&rel_level_file);

            let level_file_data = LevelFile {
                version: SUPPORTED_VERSION,
                id: level.id,
                name: level.name.clone(),
                world_pos: level.world_pos,
                world_depth: level.world_depth,
                size_px: level.size_px,
                layers: level.layers.clone(),
                fields: level.fields.clone(),
            };

            let level_json = format_json(&level_file_data, &abs_level_file)?;
            write_file_atomic(&abs_level_file, level_json)?;

            level_manifest_entries.push(LevelManifestEntry {
                id: level.id,
                name: level.name.clone(),
                world_pos: level.world_pos,
                world_depth: level.world_depth,
                file: format!("levels/{level_filename}"),
            });
        }

        let world_file_data = WorldFile {
            version: SUPPORTED_VERSION,
            id: world.id,
            name: world.name.clone(),
            layout: world.layout,
            default_grid: world.default_grid,
            levels: level_manifest_entries,
        };

        let world_json = format_json(&world_file_data, &abs_world_file)?;
        write_file_atomic(&abs_world_file, world_json)?;

        world_manifest_entries.push(WorldManifestEntry {
            id: world.id,
            name: world.name.clone(),
            file: format!("worlds/{world_folder_name}/world.json"),
        });
    }

    // 2. Save project.lcproj
    let project_file_data = ProjectFile {
        version: SUPPORTED_VERSION,
        name: project.name.clone(),
        settings: project.settings.clone(),
        defs: project.defs.clone(),
        worlds: world_manifest_entries,
    };

    let project_json = format_json(&project_file_data, project_file)?;
    write_file_atomic(project_file, project_json)?;

    Ok(())
}

fn format_json<T: serde::Serialize>(data: &T, path: &Path) -> Result<String, IoError> {
    serde_json::to_string_pretty(data)
        .map(|mut s| {
            s.push('\n');
            s
        })
        .map_err(|e| {
            tracing::error!(path = %path.display(), error = %e, "JSON formatting failed");
            IoError::Json {
                path: path.to_path_buf(),
                source: e,
            }
        })
}

fn write_file_atomic(path: &Path, content: String) -> Result<(), IoError> {
    let tmp_path = path.with_extension("tmp_atomic");
    fs::write(&tmp_path, content).map_err(|e| {
        tracing::error!(path = %tmp_path.display(), error = %e, "File write to temporary file failed");
        IoError::Fs {
            path: tmp_path.clone(),
            source: e,
        }
    })?;

    fs::rename(&tmp_path, path).map_err(|e| {
        tracing::error!(from = %tmp_path.display(), to = %path.display(), error = %e, "Atomic file rename failed");
        IoError::Fs {
            path: path.to_path_buf(),
            source: e,
        }
    })
}
