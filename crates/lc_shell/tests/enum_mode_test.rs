use egui::{Context, Pos2, Rect, Vec2};
use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::enums::file::convert_project_enums_to_file;
use lc_shell::features::enums::view::{drop_index, reorder_item, row_layout, EnumViewState};
use lc_shell::ShellApp;

#[test]
fn test_app_mode_default_and_switching() {
    let mut app = ShellApp::new(Project::new("Test Project"));
    assert_eq!(app.active_mode, AppMode::Editor);

    // Switch to Enums mode
    app.active_mode = AppMode::Enums;
    assert_eq!(app.active_mode, AppMode::Enums);

    // Switch back to Editor mode
    app.active_mode = AppMode::Editor;
    assert_eq!(app.active_mode, AppMode::Editor);
}

#[test]
fn test_editor_state_preserved_during_mode_switch() {
    let mut app = ShellApp::new(Project::new("Test Project"));
    app.zoom_level = 2.5;
    app.status_message = "Editing level 1".to_string();
    app.is_dirty = true;

    // Switch to Enums mode
    app.active_mode = AppMode::Enums;
    assert_eq!(app.zoom_level, 2.5);
    assert_eq!(app.status_message, "Editing level 1");
    assert!(app.is_dirty);

    // Add an enum in draft
    app.enum_draft.push(EnumDef {
        id: EnumDefId(101),
        name: "TestModeEnum".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });
    app.enum_draft_revision += 1;

    // Switch back to Editor mode: draft is preserved and switching is never blocked
    app.active_mode = AppMode::Editor;
    assert_eq!(app.zoom_level, 2.5);
    assert_eq!(app.status_message, "Editing level 1");
    assert!(app.is_dirty);
    assert!(app.enum_is_dirty());
    assert_eq!(app.enum_draft.len(), 1);
    // Other parts read saved, which remains empty
    assert_eq!(app.enum_saved.len(), 0);
}

#[test]
fn test_revert_restores_draft_equals_saved() {
    let mut app = ShellApp::new(Project::new("Test Project"));
    let initial_enum = EnumDef {
        id: EnumDefId(1),
        name: "InitialEnum".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "V1".to_string(),
            name: "V1".to_string(),
            color: [100, 100, 100, 255],
            tile: None,
        }],
    };
    app.project.defs.enums.push(initial_enum.clone());
    app.enum_saved = vec![initial_enum.clone()];
    app.enum_draft = vec![initial_enum.clone()];
    app.enum_saved_revision = 0;
    app.enum_draft_revision = 0;

    assert!(!app.enum_is_dirty());

    // Make edits to draft
    app.enum_draft.push(EnumDef {
        id: EnumDefId(2),
        name: "DraftEnum".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    });
    app.enum_draft_revision += 1;

    assert!(app.enum_is_dirty());
    assert_eq!(app.unsaved_changes_count(), 1);
    assert_ne!(app.enum_draft, app.enum_saved);

    // Revert restores draft = saved
    app.enum_draft = app.enum_saved.clone();
    app.enum_draft_revision = app.enum_saved_revision;

    assert!(!app.enum_is_dirty());
    assert_eq!(app.unsaved_changes_count(), 0);
    assert_eq!(app.enum_draft, app.enum_saved);
}

#[test]
fn test_save_clears_dirty_and_syncs_saved() {
    let mut app = ShellApp::new(Project::new("Test Project"));

    app.enum_draft.push(EnumDef {
        id: EnumDefId(1),
        name: "Rarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Legendary".to_string(),
            name: "Legendary".to_string(),
            color: [255, 215, 0, 255],
            tile: None,
        }],
    });
    app.enum_draft_revision += 1;

    assert!(app.enum_is_dirty());

    // Perform save conversion and state sync
    let resolver = |tid| app.resolve_tileset_image_size(tid);
    let file_data = convert_project_enums_to_file(
        &app.enum_draft,
        &app.project.defs.tilesets,
        Some(&app.enum_tileset_map),
        &resolver,
    );
    assert_eq!(file_data.enums.len(), 1);

    app.enum_saved = app.enum_draft.clone();
    app.project.defs.enums = app.enum_draft.clone();
    app.enum_saved_revision = app.enum_draft_revision;

    assert!(!app.enum_is_dirty());
    assert_eq!(app.unsaved_changes_count(), 0);
    assert_eq!(app.enum_saved, app.enum_draft);
    assert_eq!(app.project.defs.enums, app.enum_draft);
}

#[test]
fn test_render_enum_mode_frame() {
    let ctx = Context::default();
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    app.active_mode = AppMode::Enums;

    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.active_mode, AppMode::Enums);
}

#[test]
fn test_render_editor_mode_frame() {
    let ctx = Context::default();
    let mut app = ShellApp::new(Project::new("Fleiya Level Design Project"));
    app.active_mode = AppMode::Editor;

    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.active_mode, AppMode::Editor);
}

#[test]
fn test_row_layout_geometry_and_pinned_delete() {
    for width in [280.0, 320.0, 500.0, 800.0, 1200.0] {
        let row_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(width, 32.0));
        let r = row_layout(row_rect);

        assert_eq!(r.row, row_rect);
        // Handle is 20 wide and 32 high
        assert_eq!(r.handle.width(), 20.0);
        assert_eq!(r.handle.height(), 32.0);
        assert_eq!(r.handle.min, row_rect.min);

        // Bound tile is 32x32 square with 8px gap
        assert_eq!(r.bound_tile.width(), 32.0);
        assert_eq!(r.bound_tile.height(), 32.0);
        assert_eq!(r.bound_tile.min.x, r.handle.max.x + 8.0);

        // Color swatch is 32x32 square with 8px gap
        assert_eq!(r.color.width(), 32.0);
        assert_eq!(r.color.height(), 32.0);
        assert_eq!(r.color.min.x, r.bound_tile.max.x + 8.0);

        // Delete button is 32x32 square pinned to the right edge
        assert_eq!(r.delete_btn.width(), 32.0);
        assert_eq!(r.delete_btn.height(), 32.0);
        assert_eq!(r.delete_btn.max.x, row_rect.max.x);

        // Input rect starts 8px after color and ends 8px before delete button
        assert_eq!(r.input.min.x, r.color.max.x + 8.0);
        assert_eq!(r.input.max.x, r.delete_btn.min.x - 8.0);
        assert_eq!(r.input.height(), 32.0);
    }
}

#[test]
fn test_drop_index_calculations() {
    let row_rects = vec![
        Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 32.0)), // center y = 16
        Rect::from_min_size(Pos2::new(0.0, 40.0), Vec2::new(300.0, 32.0)), // center y = 56
        Rect::from_min_size(Pos2::new(0.0, 80.0), Vec2::new(300.0, 32.0)), // center y = 96
        Rect::from_min_size(Pos2::new(0.0, 120.0), Vec2::new(300.0, 32.0)), // center y = 136
    ];

    // Pointer above the first row
    assert_eq!(drop_index(-10.0, &row_rects), 0);
    assert_eq!(drop_index(10.0, &row_rects), 0);

    // Pointer between row 0 and row 1
    assert_eq!(drop_index(30.0, &row_rects), 1);

    // Pointer between row 1 and row 2
    assert_eq!(drop_index(70.0, &row_rects), 2);

    // Pointer between row 2 and row 3
    assert_eq!(drop_index(110.0, &row_rects), 3);

    // Pointer below the last row
    assert_eq!(drop_index(150.0, &row_rects), 4);
    assert_eq!(drop_index(500.0, &row_rects), 4);

    // Empty list returns 0
    assert_eq!(drop_index(100.0, &[]), 0);
}

#[test]
fn test_reorder_item_logic() {
    let mut items = vec!["A", "B", "C", "D"];

    // Move forward: index 0 to target position 2 (after B)
    assert!(reorder_item(&mut items, 0, 2));
    assert_eq!(items, vec!["B", "A", "C", "D"]);

    // Move to end: index 0 ("B") to target position 4 (after D)
    assert!(reorder_item(&mut items, 0, 4));
    assert_eq!(items, vec!["A", "C", "D", "B"]);

    // Move backward: index 3 ("B") to target position 0 (before A)
    assert!(reorder_item(&mut items, 3, 0));
    assert_eq!(items, vec!["B", "A", "C", "D"]);

    // No-ops: moving to same index or drop position right after it
    assert!(!reorder_item(&mut items, 1, 1));
    assert!(!reorder_item(&mut items, 1, 2));
    assert_eq!(items, vec!["B", "A", "C", "D"]);

    // Out of bounds
    assert!(!reorder_item(&mut items, 10, 0));
    assert!(!reorder_item(&mut items, 0, 10));
}

#[test]
fn test_value_uids_stability_and_reordering() {
    let mut state = EnumViewState::default();
    let enum_id = EnumDefId(42);

    // Ensure 3 UIDs allocated
    let uids = state.ensure_value_uids(enum_id, 3).to_vec();
    assert_eq!(uids.len(), 3);
    assert_eq!(uids, vec![1001, 1002, 1003]);

    // Subsequent call returns exact same UIDs
    let uids2 = state.ensure_value_uids(enum_id, 3).to_vec();
    assert_eq!(uids2, vec![1001, 1002, 1003]);

    // Reorder values and uids together
    let mut values = vec!["Val1".to_string(), "Val2".to_string(), "Val3".to_string()];
    let mut uids_copy = uids.clone();

    // Reorder from 0 to 2
    assert!(reorder_item(&mut values, 0, 2));
    assert!(reorder_item(&mut uids_copy, 0, 2));

    assert_eq!(values, vec!["Val2", "Val1", "Val3"]);
    assert_eq!(uids_copy, vec![1002, 1001, 1003]);
    // Each value maintains its original uid
    assert_eq!(uids_copy[1], 1001); // "Val1" is still tied to UID 1001
}

#[test]
fn test_enum_identifier_max_length_48_clamping() {
    let mut name =
        "ThisIsAnExtremelyLongEnumIdentifierNameToTestClampingExceedingFortyEightCharacters"
            .to_string();
    if name.chars().count() > 48 {
        name = name.chars().take(48).collect();
    }
    assert_eq!(name.chars().count(), 48);
}

#[test]
fn test_render_with_very_long_enum_name_does_not_panic() {
    let ctx = Context::default();
    let mut app = ShellApp::new(Project::new("Test Project"));
    app.active_mode = AppMode::Enums;

    let long_name =
        "SuperLongEnumNameThatCouldPotentiallyCauseHorizontalOverflowIfTruncationFails1234567890"
            .to_string();
    let enum_def = EnumDef {
        id: EnumDefId(999),
        name: long_name,
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Value1".to_string(),
            name: "Value1".to_string(),
            color: [255, 0, 0, 255],
            tile: None,
        }],
    };
    app.enum_draft.push(enum_def);
    app.selected_enum_def = Some(EnumDefId(999));

    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
}

#[test]
fn test_enum_card_sizes_and_uniformity() {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let mut app = ShellApp::new(Project::new("Test Project"));
    app.active_mode = AppMode::Enums;

    // Load sample enums matching Figure 2
    app.enum_draft = vec![
        EnumDef {
            id: EnumDefId(1),
            name: "BiomeType".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
        EnumDef {
            id: EnumDefId(2),
            name: "ItemRarity".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
        EnumDef {
            id: EnumDefId(3),
            name: "WeaponType".to_string(),
            tileset_id: None,
            tags: Vec::new(),
            values: vec![],
        },
    ];
    app.selected_enum_def = Some(EnumDefId(2));

    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    // Query card_rects from transient EnumViewState
    let state = ctx
        .data(|d| d.get_temp::<EnumViewState>(egui::Id::new("enum_mode_view_transient_state")))
        .expect("EnumViewState should be populated");

    assert_eq!(state.card_rects.len(), 3);

    // Height of all cards must match ENUM_CARD_HEIGHT (36.0)
    for rect in &state.card_rects {
        assert_eq!(
            rect.height(),
            lc_shell::features::enums::view::ENUM_CARD_HEIGHT,
            "Card height must exactly equal ENUM_CARD_HEIGHT"
        );
    }

    // Gap between Card 1 and Card 2 must match ENUM_CARD_GAP (5.0)
    assert_eq!(
        state.card_rects[1].top() - state.card_rects[0].bottom(),
        lc_shell::features::enums::view::ENUM_CARD_GAP,
        "Gap between Card 1 and Card 2 must equal ENUM_CARD_GAP"
    );

    // Gap between Card 2 and Card 3 must match ENUM_CARD_GAP (5.0)
    assert_eq!(
        state.card_rects[2].top() - state.card_rects[1].bottom(),
        lc_shell::features::enums::view::ENUM_CARD_GAP,
        "Gap between Card 2 and Card 3 must equal ENUM_CARD_GAP"
    );

    // Uniformity: all cards must have identical width and identical starting x-coordinate
    assert_eq!(state.card_rects[0].width(), state.card_rects[1].width());
    assert_eq!(state.card_rects[1].width(), state.card_rects[2].width());
    assert_eq!(state.card_rects[0].min.x, state.card_rects[1].min.x);
    assert_eq!(state.card_rects[1].min.x, state.card_rects[2].min.x);

    // Delete buttons: must be at the exact same horizontal position on all 3 cards
    assert_eq!(state.delete_btn_rects.len(), 3);
    assert_eq!(
        state.delete_btn_rects[0].min.x, state.delete_btn_rects[1].min.x,
        "Delete button must be in the same horizontal position on all cards"
    );
    assert_eq!(
        state.delete_btn_rects[1].min.x, state.delete_btn_rects[2].min.x,
        "Delete button must be in the same horizontal position on all cards"
    );
    assert_eq!(
        state.delete_btn_rects[0].max.x, state.delete_btn_rects[1].max.x,
        "Delete button right edge must be in the same position on all cards"
    );
    assert_eq!(
        state.delete_btn_rects[1].max.x, state.delete_btn_rects[2].max.x,
        "Delete button right edge must be in the same position on all cards"
    );

    // Right padding must match left padding (distance of drag handle from left edge):
    // Left distance = handle.min.x - card.min.x = 11.0 (10.0 padding + 1.0 border)
    // Right distance = card.max.x - delete_btn.max.x = 11.0 (10.0 padding + 1.0 border)
    for i in 0..3 {
        let card = state.card_rects[i];
        let del = state.delete_btn_rects[i];
        let right_distance = card.max.x - del.max.x;
        assert_eq!(
            right_distance,
            lc_shell::features::enums::view::ENUM_CARD_PADDING_X + 1.0,
            "Delete button right padding must match left padding for symmetry"
        );
    }
}

#[test]
fn test_enum_card_metrics_constants() {
    assert_eq!(lc_shell::features::enums::view::ENUM_CARD_HEIGHT, 36.0);
    assert_eq!(lc_shell::features::enums::view::ENUM_CARD_GAP, 5.0);
    assert_eq!(lc_shell::features::enums::view::ENUM_CARD_PADDING_X, 10.0);
    assert_eq!(lc_shell::features::enums::view::ENUM_CARD_PADDING_Y, 6.0);
    assert_eq!(lc_shell::features::enums::view::DRAG_HANDLE_WIDTH, 16.0);
    assert_eq!(lc_shell::features::enums::view::DRAG_HANDLE_GAP, 12.0);
    assert_eq!(lc_shell::features::enums::view::TRASH_ICON_WIDTH, 20.0);
    assert_eq!(lc_shell::features::enums::view::TRASH_ICON_GAP, 8.0);
}

#[test]
fn test_enum_card_layout_geometry() {
    let padding_x = lc_shell::features::enums::view::ENUM_CARD_PADDING_X;
    let padding_y = lc_shell::features::enums::view::ENUM_CARD_PADDING_Y;
    let card_height = lc_shell::features::enums::view::ENUM_CARD_HEIGHT;
    let stroke_width = 1.0;

    let content_h = card_height - 2.0 * padding_y - 2.0 * stroke_width;
    assert_eq!(content_h, 22.0);

    // Total outer height of frame = content_h + 2 * padding_y + 2 * stroke_width
    let outer_h = content_h + 2.0 * padding_y + 2.0 * stroke_width;
    assert_eq!(outer_h, 36.0);

    // Distance from card outer left to text start:
    // outer_left + stroke_width (1.0) + padding_x (10.0) + handle_w (16.0) + handle_gap (12.0) = 39.0
    let text_start_offset = stroke_width
        + padding_x
        + lc_shell::features::enums::view::DRAG_HANDLE_WIDTH
        + lc_shell::features::enums::view::DRAG_HANDLE_GAP;
    assert_eq!(text_start_offset, 39.0);
}

#[test]
fn test_uniform_control_height_constant() {
    assert_eq!(lc_shell::features::enums::view::CONTROL_HEIGHT, 32.0);
    assert_eq!(lc_shell::features::enums::view::KEBAB_ICON_WIDTH, 20.0);
    assert_eq!(lc_shell::features::enums::view::KEBAB_ICON_GAP, 8.0);
}

#[test]
fn test_tag_lifecycle_dedup_and_auto_cleanup() {
    let mut app = ShellApp::new(Project::new("Tag Test Project"));
    let enum1 = EnumDefId(1);
    let enum2 = EnumDefId(2);

    // 1. Create tags with deduplication (case-insensitive)
    let tag_a = app.get_or_create_tag("Gameplay");
    let tag_a_dup = app.get_or_create_tag("gameplay ");
    assert_eq!(
        tag_a, tag_a_dup,
        "Tags must be deduplicated case-insensitively"
    );
    assert_eq!(app.tag_registry.len(), 1);

    let tag_b = app.get_or_create_tag("Visual");
    assert_ne!(tag_a, tag_b);
    assert_eq!(app.tag_registry.len(), 2);

    // 2. Assign tags
    app.assign_tag_to_enum(enum1, tag_a);
    app.assign_tag_to_enum(enum1, tag_b);
    app.assign_tag_to_enum(enum2, tag_b);

    assert_eq!(app.enum_tags_map.get(&enum1).unwrap().len(), 2);
    assert_eq!(app.enum_tags_map.get(&enum2).unwrap().len(), 1);

    // Unused tags query
    let unused_enum2 = app.unused_tags_for_enum(enum2);
    assert_eq!(unused_enum2.len(), 1);
    assert_eq!(unused_enum2[0].id, tag_a);

    let unused_enum1 = app.unused_tags_for_enum(enum1);
    assert!(unused_enum1.is_empty());

    // 3. Remove tag_b from enum2 -> tag_b still used by enum1, so not pruned
    app.remove_tag_from_enum(enum2, tag_b);
    assert_eq!(app.tag_registry.len(), 2);

    // 4. Remove tag_b from enum1 -> tag_b is no longer used by any enum or tileset, so pruned!
    app.remove_tag_from_enum(enum1, tag_b);
    assert_eq!(app.tag_registry.len(), 1);
    assert_eq!(app.tag_registry[0].id, tag_a);
}

#[test]
fn test_tag_renaming_and_validation() {
    let mut app = ShellApp::new(Project::new("Tag Rename Project"));
    let tag1 = app.get_or_create_tag("Alpha");
    let _tag2 = app.get_or_create_tag("Beta");

    // Rejection of empty or whitespace name
    assert!(app.rename_tag(tag1, "").is_err());
    assert!(app.rename_tag(tag1, "   ").is_err());

    // Rejection of duplicate name (case-insensitive)
    assert!(app.rename_tag(tag1, "beta").is_err());
    assert!(app.rename_tag(tag1, "BETA").is_err());

    // Valid rename
    assert!(app.rename_tag(tag1, "Gamma").is_ok());
    assert_eq!(
        app.tag_registry.iter().find(|t| t.id == tag1).unwrap().name,
        "Gamma"
    );
}

#[test]
fn test_duplicate_enum_functionality() {
    let mut app = ShellApp::new(Project::new("Duplicate Test Project"));
    let e1 = EnumDef {
        id: EnumDefId(10),
        name: "ItemType".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Sword".to_string(),
            name: "Sword".to_string(),
            color: [200, 50, 50, 255],
            tile: None,
        }],
    };
    let e2 = EnumDef {
        id: EnumDefId(20),
        name: "ArmorType".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    };
    app.enum_draft = vec![e1.clone(), e2.clone()];
    let tag = app.get_or_create_tag("Equipment");
    app.assign_tag_to_enum(e1.id, tag);

    // Duplicate e1
    let dup1_id = app.duplicate_enum(e1.id).expect("Duplicate must succeed");
    assert_eq!(app.enum_draft.len(), 3);
    assert_eq!(
        app.enum_draft[1].id, dup1_id,
        "Duplicate must be placed directly below original"
    );
    assert_eq!(app.enum_draft[1].name, "ItemType_Copy");
    assert_eq!(app.enum_draft[1].values.len(), 1);
    assert_eq!(app.enum_draft[1].values[0].id, "Sword");
    // Tag copied to duplicated enum
    assert!(app.enum_tags_map.get(&dup1_id).unwrap().contains(&tag));

    // Duplicate e1 again -> ItemType_Copy2
    let dup2_id = app.duplicate_enum(e1.id).expect("Duplicate must succeed");
    assert_eq!(app.enum_draft.len(), 4);
    assert_eq!(app.enum_draft[1].id, dup2_id);
    assert_eq!(app.enum_draft[1].name, "ItemType_Copy2");
}

#[test]
fn test_tag_search_and_enum_view_state() {
    let mut app = ShellApp::new(Project::new("Tag Search Project"));
    let e1 = EnumDef {
        id: EnumDefId(1),
        name: "Biome".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![],
    };
    app.enum_draft = vec![e1];
    app.selected_enum_def = Some(EnumDefId(1));

    // Register tags
    let t_forest = app.get_or_create_tag("Forest");
    let _t_desert = app.get_or_create_tag("Desert");
    let _t_ocean = app.get_or_create_tag("Ocean");

    // Assign one tag
    app.assign_tag_to_enum(EnumDefId(1), t_forest);

    // Unused tags
    let unused = app.unused_tags_for_enum(EnumDefId(1));
    assert_eq!(unused.len(), 2);

    // Filter test (case-insensitive)
    let q = "des".to_lowercase();
    let filtered: Vec<_> = unused
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&q))
        .collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Desert");

    let q_empty = "".to_lowercase();
    let filtered_all: Vec<_> = unused
        .iter()
        .filter(|t| q_empty.is_empty() || t.name.to_lowercase().contains(&q_empty))
        .collect();
    assert_eq!(filtered_all.len(), 2);

    // Verify EnumViewState defaults
    let state = EnumViewState::default();
    assert_eq!(state.tag_recall_picker.search(), "");
    assert_eq!(state.tag_recall_picker.selected_idx(), 0);
    assert!(!state.tag_recall_picker.was_open());
    assert_eq!(state.previous_selected_enum, None);
}
