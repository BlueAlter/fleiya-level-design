use glam::UVec2;
use lc_core::command::{Command, UndoStack};
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, IdGenerator, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;
use lc_feature_tileset::commands::*;

fn setup_test_project() -> (Project, TilesetDefId, EnumDefId) {
    let mut gen = IdGenerator::with_seed(100);
    let tileset_id = gen.next_tileset_def_id();
    let enum_id = gen.next_enum_def_id();

    let tileset = TilesetDef {
        id: tileset_id,
        name: "WorldTiles".to_string(),
        path: "tiles.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: vec!["Nature".to_string(), "Ground".to_string()],
        marking: TileMarking::new(),
        custom_data: Default::default(),
        animations: Default::default(),
    };

    let enum_def = EnumDef {
        id: enum_id,
        name: "TileType".to_string(),
        tileset_id: Some(tileset_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Grass".to_string(),
                name: "Grass Tile".to_string(),
                color: [0, 255, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Water".to_string(),
                name: "Water Tile".to_string(),
                color: [0, 0, 255, 255],
                tile: None,
            },
        ],
    };

    let mut project = Project::new("MarkingTest");
    project.defs.tilesets.push(tileset);
    project.defs.enums.push(enum_def);
    (project, tileset_id, enum_id)
}

#[test]
fn test_tileset_tag_commands_apply_revert() {
    let (mut project, tileset_id, _) = setup_test_project();
    let mut undo_stack = UndoStack::new();

    // 1. Add tag
    let add_cmd = Box::new(AddTilesetTagCommand::new(tileset_id, "  Dungeon  "));
    undo_stack.execute(add_cmd, &mut project).unwrap();

    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Ground", "Dungeon"]);

    // Duplicate add (case-insensitive) should fail
    let dup_cmd = Box::new(AddTilesetTagCommand::new(tileset_id, "dungeon"));
    let mut p_copy = project.clone();
    assert!(dup_cmd.clone().apply(&mut p_copy).is_err());

    // 2. Rename tag
    let rename_cmd = Box::new(RenameTilesetTagCommand::new(
        tileset_id, "Ground", "Terrain",
    ));
    undo_stack.execute(rename_cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Terrain", "Dungeon"]);

    // Rename with collision merge: "Terrain" into "Nature"
    let merge_cmd = Box::new(RenameTilesetTagCommand::new(
        tileset_id, "Terrain", "nature",
    ));
    undo_stack.execute(merge_cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Dungeon"]);

    // Undo collision merge
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Terrain", "Dungeon"]);

    // 3. Remove tag
    let remove_cmd = Box::new(RemoveTilesetTagCommand::new(tileset_id, "Terrain"));
    undo_stack.execute(remove_cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Dungeon"]);

    // Undo remove restores exact order
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Terrain", "Dungeon"]);

    // Full undo stack rewind restores initial project tags
    undo_stack.undo(&mut project); // undo rename
    undo_stack.undo(&mut project); // undo add
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tags, vec!["Nature", "Ground"]);
}

#[test]
fn test_set_marking_source_command() {
    let (mut project, tileset_id, enum_id) = setup_test_project();
    let mut undo_stack = UndoStack::new();

    // 1. Error if enum does not exist
    let bad_cmd = Box::new(SetMarkingSourceCommand::new(
        tileset_id,
        Some(EnumDefId(999)),
    ));
    let mut p_copy = project.clone();
    assert!(bad_cmd.clone().apply(&mut p_copy).is_err());

    // 2. Setting to existing enum
    let cmd = Box::new(SetMarkingSourceCommand::new(tileset_id, Some(enum_id)));
    undo_stack.execute(cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, Some(enum_id));

    // Paint some tags
    let paint_cmd = Box::new(PaintTileTagsCommand::new(
        tileset_id,
        "Grass",
        vec![1, 2, 3],
        true,
    ));
    undo_stack.execute(paint_cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Grass", 1));

    // 3. Changing source enum clears tags
    let clear_cmd = Box::new(SetMarkingSourceCommand::new(tileset_id, None));
    undo_stack.execute(clear_cmd, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, None);
    assert!(ts.marking.tags.is_empty());

    // 4. Undo restores source enum and markings exactly
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, Some(enum_id));
    assert!(ts.marking.has_tag("Grass", 1));
    assert!(ts.marking.has_tag("Grass", 2));
    assert!(ts.marking.has_tag("Grass", 3));
}

#[test]
fn test_paint_tile_tags_command() {
    let (mut project, tileset_id, enum_id) = setup_test_project();
    let mut undo_stack = UndoStack::new();

    // Configure source enum
    let set_src = Box::new(SetMarkingSourceCommand::new(tileset_id, Some(enum_id)));
    undo_stack.execute(set_src, &mut project).unwrap();

    // Error if value not in source enum
    let bad_val_cmd = Box::new(PaintTileTagsCommand::new(tileset_id, "Lava", vec![0], true));
    let mut p_copy = project.clone();
    assert!(bad_val_cmd.clone().apply(&mut p_copy).is_err());

    // One stroke painting
    let stroke1 = Box::new(PaintTileTagsCommand::new(
        tileset_id,
        "Grass",
        vec![0, 1, 2],
        true,
    ));
    undo_stack.execute(stroke1, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Grass", 0));
    assert!(ts.marking.has_tag("Grass", 1));
    assert!(ts.marking.has_tag("Grass", 2));

    // Painting already marked tiles only records changed
    let stroke2 = Box::new(PaintTileTagsCommand::new(
        tileset_id,
        "Grass",
        vec![1, 2, 3],
        true,
    ));
    undo_stack.execute(stroke2, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Grass", 3));

    // Untagging
    let untag = Box::new(PaintTileTagsCommand::new(
        tileset_id,
        "Grass",
        vec![0, 3],
        false,
    ));
    undo_stack.execute(untag, &mut project).unwrap();
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(!ts.marking.has_tag("Grass", 0));
    assert!(ts.marking.has_tag("Grass", 1));
    assert!(ts.marking.has_tag("Grass", 2));
    assert!(!ts.marking.has_tag("Grass", 3));

    // Undo untag
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Grass", 0));
    assert!(ts.marking.has_tag("Grass", 3));

    // Rewind all strokes restores empty markings
    undo_stack.undo(&mut project);
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.tags.is_empty());
}

#[test]
fn test_update_tileset_def_command_prunes_marking() {
    let (mut project, tileset_id, enum_id) = setup_test_project();
    let mut undo_stack = UndoStack::new();

    // Set source enum
    let set_src = Box::new(SetMarkingSourceCommand::new(tileset_id, Some(enum_id)));
    undo_stack.execute(set_src, &mut project).unwrap();

    // Tag tiles 0..15
    let tag_cmd = Box::new(PaintTileTagsCommand::new(
        tileset_id,
        "Grass",
        (0..16).collect(),
        true,
    ));
    undo_stack.execute(tag_cmd, &mut project).unwrap();

    let initial_copy = project.clone();

    // Resize tileset: tile_count shrinks from 16 to 4 (e.g. tile_size 32x32 on 64x64 image)
    let mut updated_def = project.defs.tileset(tileset_id).unwrap().clone();
    updated_def.tile_size = UVec2::new(32, 32);

    let update_cmd =
        Box::new(UpdateTilesetDefCommand::new(updated_def).with_image_size(UVec2::new(64, 64)));
    undo_stack.execute(update_cmd, &mut project).unwrap();

    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tile_size, UVec2::new(32, 32));
    // Marking should be pruned: tiles 0..3 kept, 4..15 pruned
    for t in 0..4 {
        assert!(ts.marking.has_tag("Grass", t));
    }
    for t in 4..16 {
        assert!(!ts.marking.has_tag("Grass", t));
    }

    // Undo restores old marking exactly
    undo_stack.undo(&mut project);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.tile_size, UVec2::new(16, 16));
    assert_eq!(&project, &initial_copy);
}
