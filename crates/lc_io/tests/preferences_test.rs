use lc_io::{
    load_preferences, load_preferences_or_default, save_preferences, AppPreferences,
    ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutMap, ThemePreference,
    WorkspaceStartupMode,
};
use std::path::PathBuf;

fn temp_prefs_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("lc_io_preferences_tests")
        .join(name)
        .join("preferences.json")
}

#[test]
fn preferences_round_trip_disk_persistence() {
    let path = temp_prefs_path("round_trip");
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }

    let mut prefs = AppPreferences::default();
    prefs.general.auto_open_last_project = false;
    prefs.general.default_workspace = WorkspaceStartupMode::Entities;
    prefs.general.last_project_path = Some("C:/Projects/Demo/project.lcproj".to_string());
    prefs.appearance.theme = ThemePreference::SolarizedDark;
    prefs.appearance.ui_scale = 1.25;
    prefs.canvas.zoom_sensitivity = 1.5;
    prefs.canvas.checkerboard_tile_px = 24;
    prefs.system.auto_save_enabled = true;
    prefs.system.auto_save_interval_secs = 90;
    prefs.shortcuts.set(
        ShortcutAction::WorkspaceSettings,
        ShortcutBinding::new(ShortcutKey::F9, false, false, false),
    );

    save_preferences(&prefs, &path).expect("save preferences");
    let loaded = load_preferences(&path).expect("load preferences");
    assert_eq!(loaded, prefs);

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn load_preferences_or_default_handles_missing_and_corrupt_files() {
    let missing = temp_prefs_path("missing");
    if let Some(parent) = missing.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
    assert_eq!(load_preferences_or_default(&missing), AppPreferences::default());

    let corrupt = temp_prefs_path("corrupt");
    if let Some(parent) = corrupt.parent() {
        std::fs::create_dir_all(parent).expect("create temp parent");
    }
    std::fs::write(&corrupt, "{ not valid json").expect("write corrupt file");
    assert_eq!(load_preferences_or_default(&corrupt), AppPreferences::default());

    if let Some(parent) = corrupt.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn partial_json_uses_serde_defaults() {
    let path = temp_prefs_path("partial");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create temp parent");
    }
    std::fs::write(&path, "{\n  \"appearance\": {\n    \"theme\": \"ProDark\"\n  }\n}\n")
        .expect("write partial json");

    let loaded = load_preferences(&path).expect("load partial json");
    assert_eq!(loaded.appearance.theme, ThemePreference::ProDark);
    assert_eq!(loaded.appearance.ui_scale, 1.0);
    assert_eq!(
        loaded.shortcuts.get(ShortcutAction::SaveProject),
        ShortcutBinding::ctrl(ShortcutKey::S)
    );

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn shortcut_map_detects_collisions_and_resets() {
    let mut map = ShortcutMap::default();
    assert!(map.all_conflicts().is_empty());

    let ctrl_s = ShortcutBinding::ctrl(ShortcutKey::S);
    map.set(ShortcutAction::WorkspaceSettings, ctrl_s);

    let conflicts = map.conflicts_for(ShortcutAction::WorkspaceSettings, ctrl_s);
    assert_eq!(conflicts, vec![ShortcutAction::SaveProject]);
    assert!(!map.all_conflicts().is_empty());

    map.reset_action(ShortcutAction::WorkspaceSettings);
    assert!(map.all_conflicts().is_empty());
    assert_eq!(
        map.get(ShortcutAction::WorkspaceSettings),
        ShortcutBinding::ctrl(ShortcutKey::Num6)
    );
}
