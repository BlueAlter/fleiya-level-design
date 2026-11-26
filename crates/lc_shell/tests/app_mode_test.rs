use egui::{Context, Event, Key, Modifiers, RawInput};
use lc_canvas::theme::Tokens;
use lc_core::command::DocKind;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::{
    render_workspace_rail, shows_unsaved, AppMode, WORKSPACE_RAIL_ITEMS,
};
use lc_shell::features::editor::menu::handle_global_shortcuts;
use lc_shell::ShellApp;
use std::collections::HashSet;

#[test]
fn test_workspace_rail_order_and_unique_icons() {
    assert_eq!(
        WORKSPACE_RAIL_ITEMS.len(),
        5,
        "WORKSPACE_RAIL_ITEMS must have 5 entries (Editor, Layers, Entities, Tilesets, Enums)"
    );

    // Order must be: Editor, Layers, Entities, Tilesets, Enums
    assert_eq!(WORKSPACE_RAIL_ITEMS[0].mode, Some(AppMode::Editor));
    assert!(!WORKSPACE_RAIL_ITEMS[0].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[1].mode, None);
    assert!(!WORKSPACE_RAIL_ITEMS[1].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[2].mode, Some(AppMode::Entities));
    assert!(WORKSPACE_RAIL_ITEMS[2].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[3].mode, Some(AppMode::Tilesets));
    assert!(WORKSPACE_RAIL_ITEMS[3].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[4].mode, Some(AppMode::Enums));
    assert!(WORKSPACE_RAIL_ITEMS[4].enabled);

    let mut seen_icons = HashSet::new();
    for item in &WORKSPACE_RAIL_ITEMS {
        assert!(
            seen_icons.insert(item.icon),
            "Duplicate icon found in WORKSPACE_RAIL_ITEMS: {:?}",
            item.icon
        );
    }
}

#[test]
fn test_mode_shortcuts_ctrl_4_ctrl_5_and_disabled_ctrl_1() {
    let mut app = ShellApp::new(Project::new("Test Mode Shortcuts"));
    assert_eq!(app.active_mode, AppMode::Enums);
    let ctx = Context::default();

    // 1. Ctrl+4 switches to Tilesets
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num4,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Tilesets);

    // 2. Ctrl+1 does NOT switch to Editor while Editor is disabled
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
    assert_eq!(app.active_mode, AppMode::Tilesets);

    // 3. Ctrl+5 switches to Enums
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num5,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut out = ctx.run_ui(raw_input, |_| {});
    out.textures_delta.clear();
    handle_global_shortcuts(&mut app, &ctx);
    assert_eq!(app.active_mode, AppMode::Enums);

    // 4. Ctrl+4 through full app.ui frame switches to Tilesets
    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num4,
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
    assert!(!shows_unsaved(AppMode::Entities, false));
    assert!(!shows_unsaved(AppMode::Entities, true));
    assert!(!shows_unsaved(AppMode::Tilesets, false));
    assert!(!shows_unsaved(AppMode::Tilesets, true));
    assert!(!shows_unsaved(AppMode::Enums, false));
    assert!(shows_unsaved(AppMode::Enums, true));
}

#[test]
fn test_ctrl_3_switches_to_entities_workspace_and_dockind_focuses() {
    let mut app = ShellApp::new(Project::new("Test Entities Mode"));
    assert_eq!(app.active_mode, AppMode::Enums);
    let ctx = Context::default();

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
    assert_eq!(app.active_mode, AppMode::Entities);
    assert_eq!(app.active_mode.workspace_title(), "ENTITY WORKSPACE");

    app.active_mode = AppMode::Enums;
    app.open_document(DocKind::EntityDef(lc_core::id::EntityDefId(7)));
    assert_eq!(app.active_mode, AppMode::Entities);
    assert_eq!(app.selected_entity_def, Some(lc_core::id::EntityDefId(7)));
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

#[test]
fn test_dockind_tileset_def_switches_mode_and_selects_tileset() {
    let mut app = ShellApp::new(Project::new("Test DocKind"));
    assert_eq!(app.active_mode, AppMode::Enums);
    assert_eq!(app.selected_tileset_def, None);

    app.open_document(DocKind::TilesetDef(TilesetDefId(42)));

    assert_eq!(app.active_mode, AppMode::Tilesets);
    assert_eq!(app.selected_tileset_def, Some(TilesetDefId(42)));
}

#[test]
fn test_navigation_switching_and_active_highlight() {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    // 1. Rail rendering and active highlight check for enabled modes (Enums & Tilesets)
    for expected_mode in [AppMode::Enums, AppMode::Tilesets] {
        let mut active_mode = expected_mode;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            render_workspace_rail(ui, &mut active_mode, false, &t);
        });
        output.textures_delta.clear();

        let mut accent_fill_count = 0;
        for clipped in &output.shapes {
            if let egui::Shape::Rect(r) = &clipped.shape {
                if r.fill == t.accent {
                    accent_fill_count += 1;
                }
            }
        }
        assert_eq!(
            accent_fill_count, 1,
            "Exactly one rail button must have active accent highlight for mode {expected_mode:?}"
        );
    }

    // Disabled mode (Editor) receives no active highlight on the rail
    {
        let mut active_mode = AppMode::Editor;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            render_workspace_rail(ui, &mut active_mode, false, &t);
        });
        output.textures_delta.clear();

        let accent_fill_count = output
            .shapes
            .iter()
            .filter(|clipped| matches!(&clipped.shape, egui::Shape::Rect(r) if r.fill == t.accent))
            .count();
        assert_eq!(
            accent_fill_count, 0,
            "Disabled Editor mode must not show an active highlight on the rail"
        );
    }

    // 2. Switching modes via Ctrl+4 and Ctrl+5
    let mut app = ShellApp::new(Project::new("Navigation Test"));
    assert_eq!(app.active_mode, AppMode::Enums);

    for (key, mode) in [
        (Key::Num4, AppMode::Tilesets),
        (Key::Num5, AppMode::Enums),
        (Key::Num4, AppMode::Tilesets),
    ] {
        let mut raw_input = RawInput::default();
        raw_input.events.push(Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        });
        let mut out = ctx.run_ui(raw_input, |_| {});
        out.textures_delta.clear();
        handle_global_shortcuts(&mut app, &ctx);
        assert_eq!(
            app.active_mode, mode,
            "Pressing Ctrl+{:?} must switch to {:?}",
            key, mode
        );
    }
}

#[test]
fn test_top_header_displays_active_workspace_title_and_rail_renders() {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let mut app = ShellApp::new(Project::new("Header Title Test"));
    assert_eq!(app.active_mode, AppMode::Enums);
    assert_eq!(app.active_mode.workspace_title(), "ENUM WORKSPACE");

    let mut out_enum = ctx.run_ui(RawInput::default(), |ui| {
        app.ui(ui);
    });
    out_enum.textures_delta.clear();

    let mut found_enum_title = false;
    for clipped in &out_enum.shapes {
        if let egui::Shape::Text(t) = &clipped.shape {
            if t.galley.text() == "ENUM WORKSPACE" {
                found_enum_title = true;
            }
        }
    }
    assert!(
        found_enum_title,
        "Top header must display active workspace title 'ENUM WORKSPACE'"
    );

    app.active_mode = AppMode::Tilesets;
    assert_eq!(app.active_mode.workspace_title(), "TILESET WORKSPACE");
    let mut out_ts = ctx.run_ui(RawInput::default(), |ui| {
        app.ui(ui);
    });
    out_ts.textures_delta.clear();

    let mut found_ts_title = false;
    for clipped in &out_ts.shapes {
        if let egui::Shape::Text(t) = &clipped.shape {
            if t.galley.text() == "TILESET WORKSPACE" {
                found_ts_title = true;
            }
        }
    }
    assert!(
        found_ts_title,
        "Top header must display active workspace title 'TILESET WORKSPACE'"
    );
}
