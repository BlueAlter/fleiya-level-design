use egui::{vec2, Context, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_shell::ui::tile_bounds::{TileGrid, TileRect};
use lc_shell::ui::tile_picker::{TilePicker, TilePickerEvent, TilesetData};

#[test]
fn test_tile_picker_default_and_lifecycle() {
    let mut picker = TilePicker::new();
    assert!(!picker.is_open());
    assert!(picker.show_grid);
    assert_eq!(picker.view.zoom, 1.0);
    assert!(picker.selected_tile_rect.is_none());
    assert!(picker.hovered_tile.is_none());

    picker.open(
        Some("Sword".to_string()),
        Some(TilesetDefId(2)),
        Some(TileRect::single(2, 5)),
    );
    assert!(picker.is_open());
    assert_eq!(picker.target_val_id.as_deref(), Some("Sword"));
    assert_eq!(picker.tileset_id, Some(TilesetDefId(2)));
    assert_eq!(picker.selected_tile_rect, Some(TileRect::single(2, 5)));
    assert!(picker.needs_initial_fit);

    picker.close();
    assert!(!picker.is_open());
    assert!(picker.target_val_id.is_none());
    assert!(picker.hovered_tile.is_none());
}

#[test]
fn test_tileset_data_empty_and_conversion() {
    let empty = TilesetData::empty();
    assert!(empty.is_empty());
    assert_eq!(empty.id, TilesetDefId(0));

    let def = TilesetDef {
        id: TilesetDefId(1),
        name: "DungeonTiles".to_string(),
        path: "assets/tilesets/dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let data = TilesetData::from(&def);
    assert!(!data.is_empty());
    assert_eq!(data.id, TilesetDefId(1));
    assert_eq!(data.name, "DungeonTiles");
    assert_eq!(data.tile_size, glam::UVec2::new(16, 16));
}

#[test]
fn test_tile_picker_empty_tileset_handling() {
    let ctx = Context::default();
    let mut picker = TilePicker::new();
    picker.open(Some("Forest".to_string()), None, None);

    let empty_data = TilesetData::empty();

    let mut output = ctx.run_ui(Default::default(), |ctx| {
        let ev = picker.show(ctx, &empty_data, None);
        assert_eq!(ev, TilePickerEvent::None);
    });
    output.textures_delta.clear();

    assert!(picker.is_open());

    // Press Escape to close empty popup
    let mut output2 = ctx.run_ui(
        RawInput {
            events: vec![egui::Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            let ev = picker.show(ctx, &empty_data, None);
            assert_eq!(ev, TilePickerEvent::Closed);
        },
    );
    output2.textures_delta.clear();
    assert!(!picker.is_open());
}

#[test]
fn test_tile_picker_keyboard_shortcuts() {
    let ctx = Context::default();
    let mut picker = TilePicker::new();
    picker.open(
        Some("Val1".to_string()),
        Some(TilesetDefId(1)),
        Some(TileRect::single(2, 1)),
    );
    assert!(picker.show_grid);

    let def = TilesetDef {
        id: TilesetDefId(1),
        name: "DungeonTiles".to_string(),
        path: "examples/sample_project/assets/tilesets/dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let data = TilesetData::from(&def);

    // Ctrl+H toggles grid
    let mut out1 = ctx.run_ui(
        RawInput {
            events: vec![egui::Event::Key {
                key: Key::H,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::CTRL,
            }],
            ..Default::default()
        },
        |ctx| {
            let ev = picker.show(ctx, &data, None);
            assert_eq!(ev, TilePickerEvent::None);
        },
    );
    out1.textures_delta.clear();
    assert!(
        !picker.show_grid,
        "Ctrl+H should have toggled grid to false"
    );

    // Enter applies selected tile and closes
    let mut out2 = ctx.run_ui(
        RawInput {
            events: vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            let ev = picker.show(ctx, &data, None);
            assert_eq!(ev, TilePickerEvent::Selected(TileRect::single(2, 1)));
        },
    );
    out2.textures_delta.clear();
    assert!(!picker.is_open());
}

#[test]
fn test_tile_picker_texture_caching_and_registration() {
    let ctx = Context::default();
    let mut picker = TilePicker::new();

    // Verify DungeonTiles (64x64) and Icons (512x1024)
    let dungeon_def = TilesetDef {
        id: TilesetDefId(1),
        name: "DungeonTiles".to_string(),
        path: "examples/sample_project/assets/tilesets/dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let dungeon_data = TilesetData::from(&dungeon_def);

    let icons_def = TilesetDef {
        id: TilesetDefId(2),
        name: "Icons".to_string(),
        path: "examples/sample_project/assets/tilesets/finalbossblues-icons_full_16.png"
            .to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let icons_data = TilesetData::from(&icons_def);

    picker.open(Some("Item".to_string()), Some(TilesetDefId(1)), None);
    let mut out1 = ctx.run_ui(Default::default(), |ctx| {
        let _ = picker.show(ctx, &dungeon_data, None);
    });
    out1.textures_delta.clear();
    assert!(picker.textures.contains_key(&TilesetDefId(1)));

    // Switch to Icons
    picker.open(Some("Weapon".to_string()), Some(TilesetDefId(2)), None);
    let mut out2 = ctx.run_ui(Default::default(), |ctx| {
        let _ = picker.show(ctx, &icons_data, None);
    });
    out2.textures_delta.clear();
    assert!(picker.textures.contains_key(&TilesetDefId(2)));

    // Both textures remain cached
    assert!(picker.textures.contains_key(&TilesetDefId(1)));
    assert!(picker.textures.contains_key(&TilesetDefId(2)));
}

#[test]
fn test_tile_picker_page_centering_and_view() {
    let ctx = Context::default();
    let mut picker = TilePicker::new();

    let def = TilesetDef {
        id: TilesetDefId(1),
        name: "DungeonTiles".to_string(),
        path: "examples/sample_project/assets/tilesets/dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let data = TilesetData::from(&def);

    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0));
    picker.open(Some("Test".to_string()), Some(TilesetDefId(1)), None);

    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            let _ = picker.show(ctx, &data, None);
        },
    );
    out.textures_delta.clear();

    // Viewport width = 1280 * 0.8 = 1024, height = 800 * 0.8 = 640
    // DungeonTiles is 64x64. Fit zoom fits available size with ~24px padding:
    // avail = 640 - 48 = 592 -> zoom = 592 / 64 = 9.25
    assert!(picker.view.zoom > 1.0);
    // Page is centered horizontally and vertically in viewport
    let zoomed_w = 64.0 * picker.view.zoom;
    let expected_center_x = 128.0 + (1024.0 - zoomed_w) / 2.0;
    assert!((picker.view.offset.x - expected_center_x).abs() < 2.0);
}

#[test]
fn test_tile_picker_multi_select_right_drag_and_cancel() {
    let ctx = Context::default();
    let mut picker = TilePicker::new();

    let def = TilesetDef {
        id: TilesetDefId(1),
        name: "DungeonTiles".to_string(),
        path: "examples/sample_project/assets/tilesets/dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let data = TilesetData::from(&def);

    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0));
    picker.open(Some("TestVal".to_string()), Some(TilesetDefId(1)), None);

    // Frame 1: center and setup
    let mut out1 = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            let _ = picker.show(ctx, &data, None);
        },
    );
    out1.textures_delta.clear();

    // Right press at cell (1, 1)
    let grid = TileGrid::new(16, 16, 0, 0, 4, 4);
    let start_cell_screen = picker.view.page_to_screen(grid.cell_rect(1, 1).center());
    let mut out2 = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::PointerButton {
                pos: start_cell_screen,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            let _ = picker.show(ctx, &data, None);
        },
    );
    out2.textures_delta.clear();
    assert!(picker.is_right_dragging);
    assert_eq!(picker.drag_start_cell, Some((1, 1)));

    // Drag to cell (2, 3)
    let end_cell_screen = picker.view.page_to_screen(grid.cell_rect(2, 3).center());
    let mut out3 = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::PointerMoved(end_cell_screen)],
            ..Default::default()
        },
        |ctx| {
            let _ = picker.show(ctx, &data, None);
        },
    );
    out3.textures_delta.clear();
    assert_eq!(
        picker.live_drag_rect,
        Some(TileRect {
            col: 1,
            row: 1,
            w: 2,
            h: 3,
        })
    );

    // Press Escape during drag cancels drag without closing modal
    let mut out4 = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            let ev = picker.show(ctx, &data, None);
            assert_eq!(ev, TilePickerEvent::None);
        },
    );
    out4.textures_delta.clear();
    assert!(!picker.is_right_dragging);
    assert!(picker.is_open());
}
