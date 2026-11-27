//! Persistent user preferences stored outside project files.
//!
//! Preferences live in the OS user configuration directory (outside `.lcproj`)
//! so editor theme, shortcuts, canvas feel, and system options follow the user
//! across projects.

use crate::error::IoError;
use crate::io::{format_json, write_file_atomic};
use crate::shortcuts::ShortcutMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Current preferences schema version.
pub const PREFERENCES_VERSION: u32 = 1;

/// Subdirectory inside the user config root for Level Craft settings.
const CONFIG_DIR_NAME: &str = "FleiyaLevelDesign";
/// File name of the persisted preferences JSON.
const PREFERENCES_FILE_NAME: &str = "preferences.json";

/// Top-level application preferences persisted across sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppPreferences {
    /// Schema version for forward migration.
    pub version: u32,
    /// General startup, language, and workflow behavior.
    pub general: GeneralPreferences,
    /// Visual theme, scaling, and chrome visibility preferences.
    pub appearance: AppearancePreferences,
    /// Default viewport, grid, and interaction preferences.
    pub canvas: CanvasPreferences,
    /// Auto-save, console defaults, and undo history limits.
    pub system: SystemPreferences,
    /// User-customizable global keyboard shortcuts.
    pub shortcuts: ShortcutMap,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            version: PREFERENCES_VERSION,
            general: GeneralPreferences::default(),
            appearance: AppearancePreferences::default(),
            canvas: CanvasPreferences::default(),
            system: SystemPreferences::default(),
            shortcuts: ShortcutMap::default(),
        }
    }
}

/// Startup workspace choice applied when launching the desktop app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceStartupMode {
    /// Open directly into the Enum Definitions workspace.
    #[default]
    Enums,
    /// Open directly into the Entity Definitions workspace.
    Entities,
    /// Open directly into the Tilesets workspace.
    Tilesets,
    /// Open directly into the Settings workspace.
    Settings,
}

impl WorkspaceStartupMode {
    /// All selectable startup workspace modes.
    pub const ALL: [Self; 4] = [Self::Enums, Self::Entities, Self::Tilesets, Self::Settings];

    /// Display label for UI dropdowns.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Enums => "Enums Workspace",
            Self::Entities => "Entities Workspace",
            Self::Tilesets => "Tilesets Workspace",
            Self::Settings => "Settings Workspace",
        }
    }
}

/// General application behavior preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralPreferences {
    /// Active UI language code (e.g. `"en-US"`).
    pub language: String,
    /// Re-open the last opened project on startup when available.
    pub auto_open_last_project: bool,
    /// Absolute path of the most recently opened or saved project.
    pub last_project_path: Option<String>,
    /// Prompt before closing or replacing unsaved changes.
    pub confirm_before_exit_unsaved: bool,
    /// Workspace mode selected when launching the editor.
    pub default_workspace: WorkspaceStartupMode,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            language: "en-US".to_string(),
            auto_open_last_project: true,
            last_project_path: None,
            confirm_before_exit_unsaved: true,
            default_workspace: WorkspaceStartupMode::Enums,
        }
    }
}

/// Serializable theme preference mirroring `lc_canvas::theme::ThemeChoice`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreference {
    /// Follow the operating system dark/light preference.
    System,
    /// Studio Dark (default).
    #[default]
    Dark,
    /// Studio Light.
    Light,
    /// Pro Dark.
    ProDark,
    /// Pro Medium Gray.
    ProMedium,
    /// Pro Light.
    ProLight,
    /// Soft Dark.
    SoftDark,
    /// Soft Light.
    SoftLight,
    /// Solarized Dark.
    SolarizedDark,
}

/// Appearance and chrome visibility preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearancePreferences {
    /// Active editor color theme.
    pub theme: ThemePreference,
    /// UI zoom multiplier (`0.75` to `2.0`).
    pub ui_scale: f32,
    /// Show the centered workspace title in the top menu bar.
    pub show_workspace_titles: bool,
    /// Show cursor grid coordinates in the status bar.
    pub show_status_bar_coords: bool,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::Dark,
            ui_scale: 1.0,
            show_workspace_titles: true,
            show_status_bar_coords: true,
        }
    }
}

/// Canvas and editing interaction defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CanvasPreferences {
    /// Show grid overlay by default in new level views.
    pub show_grid_by_default: bool,
    /// Snap entity and tile operations to grid by default.
    pub snap_to_grid_by_default: bool,
    /// Scroll-wheel zoom multiplier (`0.25` to `3.0`).
    pub zoom_sensitivity: f32,
    /// Pan speed multiplier (`0.25` to `3.0`).
    pub pan_speed: f32,
    /// Pixel size of background transparency checkerboard tiles (`4` to `64`).
    pub checkerboard_tile_px: u32,
}

impl Default for CanvasPreferences {
    fn default() -> Self {
        Self {
            show_grid_by_default: true,
            snap_to_grid_by_default: true,
            zoom_sensitivity: 1.0,
            pan_speed: 1.0,
            checkerboard_tile_px: 16,
        }
    }
}

/// System-level runtime, logging, and safety preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemPreferences {
    /// Periodically save the active project when dirty and already pathed.
    pub auto_save_enabled: bool,
    /// Interval in seconds between automatic saves (`30` to `1800`).
    pub auto_save_interval_secs: u64,
    /// Enable debug-level messages in the console panel by default.
    pub console_default_debug: bool,
    /// Collapse consecutive identical log entries in the console panel.
    pub console_collapse_duplicates: bool,
    /// Maximum number of undo steps retained in memory (`20` to `1000`).
    pub max_undo_history: usize,
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            auto_save_enabled: false,
            auto_save_interval_secs: 120,
            console_default_debug: false,
            console_collapse_duplicates: true,
            max_undo_history: 100,
        }
    }
}

/// Resolves the default OS-specific user preferences file path.
pub fn default_preferences_path() -> Option<PathBuf> {
    config_root_dir().map(|dir| dir.join(CONFIG_DIR_NAME).join(PREFERENCES_FILE_NAME))
}

fn config_root_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }
}

/// Loads user preferences from disk.
pub fn load_preferences(path: &Path) -> Result<AppPreferences, IoError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        tracing::error!(path = %path.display(), error = %e, "Failed to read preferences file");
        IoError::Fs {
            path: path.to_path_buf(),
            source: e,
        }
    })?;

    serde_json::from_str(&raw).map_err(|e| {
        tracing::error!(path = %path.display(), error = %e, "Failed to parse preferences JSON");
        IoError::Json {
            path: path.to_path_buf(),
            source: e,
        }
    })
}

/// Loads user preferences from disk, returning defaults if the file is missing or unreadable.
pub fn load_preferences_or_default(path: &Path) -> AppPreferences {
    if !path.exists() {
        return AppPreferences::default();
    }
    match load_preferences(path) {
        Ok(prefs) => prefs,
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "Falling back to default preferences"
            );
            AppPreferences::default()
        }
    }
}

/// Atomically persists user preferences to disk, creating parent directories if needed.
pub fn save_preferences(prefs: &AppPreferences, path: &Path) -> Result<(), IoError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                tracing::error!(
                    path = %parent.display(),
                    error = %e,
                    "Failed to create preferences directory"
                );
                IoError::Fs {
                    path: parent.to_path_buf(),
                    source: e,
                }
            })?;
        }
    }

    let json = format_json(prefs, path)?;
    write_file_atomic(path, json)?;
    tracing::info!(path = %path.display(), "Saved user preferences");
    Ok(())
}
