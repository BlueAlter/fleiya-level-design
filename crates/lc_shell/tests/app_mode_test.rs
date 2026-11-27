use egui::{Context, Event, Key, Modifiers, RawInput};
use lc_canvas::theme::Tokens;
use lc_core::command::DocKind;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::{
    render_workspace_rail, shows_unsaved, AppMode, RailSvgIcon, SETTINGS_RAIL_ITEM,
    WORKSPACE_RAIL_ITEMS,
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
    assert_eq!(WORKSPACE_RAIL_ITEMS[1].mode, Some(AppMode::Layers));
    assert!(WORKSPACE_RAIL_ITEMS[1].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[2].mode, Some(AppMode::Entities));
    assert!(WORKSPACE_RAIL_ITEMS[2].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[3].mode, Some(AppMode::Tilesets));
    assert!(WORKSPACE_RAIL_ITEMS[3].enabled);
    assert_eq!(WORKSPACE_RAIL_ITEMS[4].mode, Some(AppMode::Enums));
    assert!(WORKSPACE_RAIL_ITEMS[4].enabled);

    let mut seen_icons = HashSet::new();
    let mut seen_svg_icons = HashSet::new();
    let mut seen_uris = HashSet::new();
    for item in WORKSPACE_RAIL_ITEMS
        .iter()
        .chain(std::iter::once(&SETTINGS_RAIL_ITEM))
    {
        assert!(
            seen_icons.insert(item.icon),
            "Duplicate icon found in workspace rail: {:?}",
            item.icon
        );
        assert!(
            seen_svg_icons.insert(item.svg_icon),
            "Duplicate SVG icon found in workspace rail: {:?}",
            item.svg_icon
        );
        assert!(
            seen_uris.insert(item.svg_icon.uri()),
            "Duplicate SVG URI found in workspace rail: {}",
            item.svg_icon.uri()
        );
        assert!(
            item.svg_icon.raw_svg().contains("<svg"),
            "Embedded SVG asset must contain <svg tag: {:?}",
            item.svg_icon
        );
    }
    assert_eq!(WORKSPACE_RAIL_ITEMS[0].svg_icon, RailSvgIcon::Editor);
    assert_eq!(WORKSPACE_RAIL_ITEMS[1].svg_icon, RailSvgIcon::Layers);
    assert_eq!(WORKSPACE_RAIL_ITEMS[2].svg_icon, RailSvgIcon::Entity);
    assert_eq!(WORKSPACE_RAIL_ITEMS[3].svg_icon, RailSvgIcon::Tileset);
    assert_eq!(WORKSPACE_RAIL_ITEMS[4].svg_icon, RailSvgIcon::Enum);
    assert_eq!(SETTINGS_RAIL_ITEM.svg_icon, RailSvgIcon::Setting);
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
    assert!(!shows_unsaved(AppMode::Layers, false));
    assert!(!shows_unsaved(AppMode::Layers, true));
    assert!(!shows_unsaved(AppMode::Entities, false));
    assert!(!shows_unsaved(AppMode::Entities, true));
    assert!(!shows_unsaved(AppMode::Tilesets, false));
    assert!(!shows_unsaved(AppMode::Tilesets, true));
    assert!(!shows_unsaved(AppMode::Enums, false));
    assert!(shows_unsaved(AppMode::Enums, true));
}

#[test]
fn test_ctrl_2_switches_to_layers_workspace_and_dockind_focuses() {
    let mut app = ShellApp::new(Project::new("Test Layers Mode"));
    assert_eq!(app.active_mode, AppMode::Enums);
    let ctx = Context::default();

    let mut raw_input = RawInput::default();
    raw_input.events.push(Event::Key {
        key: Key::Num2,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::CTRL,
    });
    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    assert_eq!(app.active_mode, AppMode::Layers);
    assert_eq!(app.active_mode.workspace_title(), "LAYER WORKSPACE");

    app.active_mode = AppMode::Enums;
    app.open_document(DocKind::LayerDef(lc_core::id::LayerDefId(5)));
    assert_eq!(app.active_mode, AppMode::Layers);
    assert_eq!(app.selected_layer_def, Some(lc_core::id::LayerDefId(5)));
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

#[test]
fn test_workspace_rail_renders_and_rasterizes_all_svg_icons() {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let mut active_mode = AppMode::Layers;
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        render_workspace_rail(ui, &mut active_mode, false, &t);
    });
    let svg_texture_count = output
        .textures_delta
        .set
        .iter()
        .filter(|(id, _)| **id != egui::TextureId::Managed(0))
        .count();
    let textured_rect_count = output
        .shapes
        .iter()
        .filter(|clipped| {
            matches!(
                &clipped.shape,
                egui::Shape::Rect(r) if r.fill_texture_id() != egui::TextureId::default()
            )
        })
        .count();
    output.textures_delta.clear();

    // All 6 rail SVGs (5 WORKSPACE_RAIL_ITEMS + SETTINGS_RAIL_ITEM) should be rasterized as textures
    // and their ImageSource normalized to white fills for theme tinting.
    for item in WORKSPACE_RAIL_ITEMS
        .iter()
        .chain(std::iter::once(&SETTINGS_RAIL_ITEM))
    {
        match item.svg_icon.image_source() {
            egui::ImageSource::Bytes { uri, bytes } => {
                assert!(uri.ends_with(".svg"));
                let text = std::str::from_utf8(bytes.as_ref()).expect("valid utf8 svg");
                assert!(
                    text.contains("#ffffff"),
                    "Normalized SVG {:?} must use #ffffff for tinting",
                    item.svg_icon
                );
                assert!(
                    !text.contains("#000000"),
                    "Normalized SVG {:?} must not keep #000000 fills",
                    item.svg_icon
                );
            }
            other => panic!("Expected ImageSource::Bytes, got {:?}", other),
        }
    }

    assert_eq!(
        svg_texture_count, 6,
        "Expected 6 rasterized SVG textures in textures_delta, got {}",
        svg_texture_count
    );
    assert_eq!(
        textured_rect_count, 6,
        "Expected 6 textured icon rects on the workspace rail, got {}",
        textured_rect_count
    );
}
