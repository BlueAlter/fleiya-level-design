use egui::{Context, Event, Key, Modifiers, RawInput};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::{control_width, shows_unsaved, AppMode, MODE_BUTTONS};
use lc_shell::features::editor::menu::handle_global_shortcuts;
use lc_shell::ShellApp;
use std::collections::HashSet;

#[test]
fn test_control_width_for_one_two_and_three_buttons() {
    let size = 26.0;
    let gap = 4.0;

    assert_eq!(control_width(1, size, gap), 26.0);
    assert_eq!(control_width(2, size, gap), 56.0);
    assert_eq!(control_width(3, size, gap), 86.0);
    assert_eq!(control_width(0, size, gap), 0.0);
}

#[test]
fn test_mode_buttons_guard_exhaustive_and_unique_icons() {
    let mut editor_count = 0;
    let mut enums_count = 0;
    let mut tilesets_count = 0;

    for btn in &MODE_BUTTONS {
        // Exhaustive match ensures adding an AppMode variant forces updating this test
        match btn.mode {
            AppMode::Editor => editor_count += 1,
            AppMode::Enums => enums_count += 1,
            AppMode::Tilesets => tilesets_count += 1,
        }
    }

    assert_eq!(
        editor_count, 1,
        "AppMode::Editor must appear in MODE_BUTTONS exactly once"
    );
    assert_eq!(
        enums_count, 1,
        "AppMode::Enums must appear in MODE_BUTTONS exactly once"
    );
    assert_eq!(
        tilesets_count, 1,
        "AppMode::Tilesets must appear in MODE_BUTTONS exactly once"
    );
    assert_eq!(
        MODE_BUTTONS.len(),
        3,
        "MODE_BUTTONS must have exactly 3 entries"
    );

    let mut seen_icons = HashSet::new();
    for btn in &MODE_BUTTONS {
        assert!(
            seen_icons.insert(btn.icon),
            "Duplicate icon found for mode {:?}",
            btn.mode
        );
    }
}

#[test]
fn test_mode_shortcuts_ctrl_3_ctrl_1_ctrl_2() {
    let mut app = ShellApp::new(Project::new("Test Mode Shortcuts"));
    assert_eq!(app.active_mode, AppMode::Editor);
    let ctx = Context::default();

    // 1. Ctrl+3 switches to Tilesets
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num3,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Tilesets);

    // 2. Ctrl+1 returns to Editor
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num1,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Editor);

    // 3. Ctrl+2 opens Enums
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num2,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Enums);

    // 4. Ctrl+3 through full app.ui frame switches to Tilesets
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num3,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    assert_eq!(app.active_mode, AppMode::Tilesets);
}

#[test]
fn test_shows_unsaved_only_for_enums() {
    assert!(!shows_unsaved(AppMode::Editor, false));
    assert!(!shows_unsaved(AppMode::Editor, true));
    assert!(!shows_unsaved(AppMode::Tilesets, false));
    assert!(!shows_unsaved(AppMode::Tilesets, true));
    assert!(!shows_unsaved(AppMode::Enums, false));
    assert!(shows_unsaved(AppMode::Enums, true));
}

#[test]
fn test_headless_frame_tilesets_mode_renders_cleanly() {
    let mut app = ShellApp::new(Project::new("Test Tileset Mode Clean"));
    app.active_mode = AppMode::Tilesets;
    let initial_project = app.project.clone();

    let ctx = Context::default();
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.active_mode, AppMode::Tilesets);
    assert_eq!(app.project, initial_project);
    assert!(app.pending_commands.is_empty());
}
