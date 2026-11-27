//! Headless integration tests for the Settings Workspace (`AppMode::Settings`),
//! custom keybinding dispatch, and project settings commands.

use egui::{vec2, Context, Event, Key, Modifiers, Pos2, RawInput, Rect};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_core::project::Project;
use lc_core::{ProjectSettings, RenameProjectCommand, UpdateProjectSettingsCommand};
use lc_io::{ShortcutAction, ShortcutBinding, ShortcutKey, ThemePreference};
use lc_shell::features::editor::app_mode::{render_workspace_rail, AppMode};
use lc_shell::features::editor::menu::handle_global_shortcuts;
use lc_shell::features::settings::SettingsCategory;
use lc_shell::ShellApp;

fn run_key_frame(ctx: &Context, key: Key, modifiers: Modifiers) {
    let mut raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    raw_input.events.push(Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
}

#[test]
fn ctrl_6_and_ctrl_comma_switch_to_settings_workspace() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let mut app = ShellApp::new(Project::new("Settings Nav"));
    assert_eq!(app.active_mode, AppMode::Enums);
    assert_eq!(AppMode::Settings.workspace_title(), "SETTINGS WORKSPACE");

    run_key_frame(&ctx, Key::Num6, Modifiers::CTRL);
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Settings);

    app.active_mode = AppMode::Enums;
    run_key_frame(&ctx, Key::Comma, Modifiers::CTRL);
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Settings);
}

#[test]
fn rail_highlights_settings_button_when_settings_mode_active() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let mut active_mode = AppMode::Settings;
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(44.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        render_workspace_rail(ui, &mut active_mode, false, &t);
    });
    out.textures_delta.clear();

    let accent_count = out
        .shapes
        .iter()
        .filter(|clipped| matches!(&clipped.shape, egui::Shape::Rect(r) if r.fill == t.accent))
        .count();
    assert_eq!(
        accent_count, 1,
        "Exactly one rail button (Settings) must be highlighted when AppMode::Settings is active"
    );
}

#[test]
fn settings_workspace_renders_every_category_and_search_without_panicking() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);

    let mut app = ShellApp::new(Project::new("Settings Render"));
    app.active_mode = AppMode::Settings;

    for category in SettingsCategory::ALL {
        app.settings_view_state.selected_category = category;
        app.settings_view_state.search_query.clear();
        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
            ..Default::default()
        };
        let mut out = ctx.run_ui(raw_input, |ui| {
            app.ui(ui);
        });
        out.textures_delta.clear();
        assert!(!out.shapes.is_empty());
    }

    // Search filtering path
    app.settings_view_state.search_query = "grid".to_string();
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    out.textures_delta.clear();
    assert!(!out.shapes.is_empty());
}

#[test]
fn custom_keybinding_dispatch_uses_rebound_shortcut() {
    let ctx = Context::default();
    let mut app = ShellApp::new(Project::new("Custom Shortcuts"));

    // Rebind WorkspaceSettings from Ctrl+6 to F9
    app.preferences.shortcuts.set(
        ShortcutAction::WorkspaceSettings,
        ShortcutBinding::new(ShortcutKey::F9, false, false, false),
    );

    // Old Ctrl+6 must no longer trigger Settings
    run_key_frame(&ctx, Key::Num6, Modifiers::CTRL);
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Enums);

    // New F9 must switch to Settings
    run_key_frame(&ctx, Key::F9, Modifiers::NONE);
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Settings);

    // While capturing a shortcut inside Settings, global shortcuts are suppressed
    app.active_mode = AppMode::Settings;
    app.settings_view_state.capturing_shortcut = Some(ShortcutAction::WorkspaceEnums);
    run_key_frame(&ctx, Key::Num5, Modifiers::CTRL);
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Settings);
}

#[test]
fn set_theme_syncs_preferences_and_project_commands_are_undoable() {
    let ctx = Context::default();
    let mut app = ShellApp::new(Project::new("InitialName"));

    app.set_theme(&ctx, ThemeChoice::SolarizedDark);
    assert_eq!(
        app.preferences.appearance.theme,
        ThemePreference::SolarizedDark
    );

    app.execute_command(Box::new(RenameProjectCommand::new("UpdatedName")));
    app.execute_command(Box::new(UpdateProjectSettingsCommand::new(
        ProjectSettings {
            default_grid_size: glam::UVec2::new(32, 32),
        },
    )));
    app.process_commands_and_events();

    assert_eq!(app.project.name, "UpdatedName");
    assert_eq!(
        app.project.settings.default_grid_size,
        glam::UVec2::new(32, 32)
    );
    assert!(app.is_dirty);

    app.undo();
    assert_eq!(
        app.project.settings.default_grid_size,
        glam::UVec2::new(16, 16)
    );
    app.undo();
    assert_eq!(app.project.name, "InitialName");
}

#[test]
fn settings_rail_button_renders_inside_full_shell_app() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let mut app = ShellApp::new(Project::new("Settings Rail Visibility"));
    app.active_mode = AppMode::Settings;

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    out.textures_delta.clear();

    let settings_btn_rect = out.shapes.iter().find_map(|clipped| {
        if let egui::Shape::Rect(r) = &clipped.shape {
            if r.fill == t.accent
                && r.rect.max.x <= 44.0
                && (r.rect.width() - 30.0).abs() < 1.0
                && (r.rect.height() - 30.0).abs() < 1.0
            {
                return Some(r.rect);
            }
        }
        None
    });

    let rect = settings_btn_rect.expect("Settings 30x30 button must render inside 44px left rail");
    assert!(
        rect.min.y >= 600.0 && rect.max.y <= 768.0,
        "Settings rail button should be pinned near bottom of 768px rail, got {:?}",
        rect
    );
}
