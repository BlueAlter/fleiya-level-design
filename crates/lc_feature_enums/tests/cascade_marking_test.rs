use glam::UVec2;
use lc_core::command::UndoStack;
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;
use lc_feature_enums::commands::{
    RemoveEnumDefCommand, RemoveEnumValueCommand, RenameEnumValueIdCommand,
};

fn setup_test_project() -> (Project, EnumDefId, TilesetDefId) {
    let mut project = Project::new("CascadeTest");
    let enum_id = EnumDefId(10);
    let tileset_id = TilesetDefId(1);

    let mut marking = TileMarking::new();
    marking.source_enum = Some(enum_id);
    marking.set_tag("Easy", 1, true);
    marking.set_tag("Easy", 2, true);
    marking.set_tag("Hard", 5, true);

    let tileset = TilesetDef {
        id: tileset_id,
        name: "WorldTiles".to_string(),
        path: "tiles.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking,
    };

    let enum_def = EnumDef {
        id: enum_id,
        name: "Difficulty".to_string(),
        tileset_id: Some(tileset_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Easy".to_string(),
                name: "Easy Mode".to_string(),
                color: [0, 255, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Hard".to_string(),
                name: "Hardcore".to_string(),
                color: [255, 0, 0, 255],
                tile: None,
            },
        ],
    };

    project.defs.tilesets.push(tileset);
    project.defs.enums.push(enum_def);
    (project, enum_id, tileset_id)
}

#[test]
fn test_remove_enum_def_clears_tileset_source_and_reverts() {
    let (mut project, enum_id, tileset_id) = setup_test_project();
    let initial_copy = project.clone();
    let mut undo_stack = UndoStack::new();

    // Remove enum with cascade
    let cmd = Box::new(RemoveEnumDefCommand::cascade(enum_id));
    undo_stack.execute(cmd, &mut project).unwrap();

    // Verify enum deleted and tileset source cleared
    assert!(project.defs.enum_def(enum_id).is_none());
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, None);
    assert!(ts.marking.tags.is_empty());

    // Revert restores everything identically
    undo_stack.undo(&mut project);
    assert_eq!(&project, &initial_copy);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, Some(enum_id));
    assert!(ts.marking.has_tag("Easy", 1));
    assert!(ts.marking.has_tag("Easy", 2));
    assert!(ts.marking.has_tag("Hard", 5));
}

#[test]
fn test_remove_enum_value_clears_marking_value_and_reverts() {
    let (mut project, enum_id, tileset_id) = setup_test_project();
    let initial_copy = project.clone();
    let mut undo_stack = UndoStack::new();

    // Remove value "Easy"
    let cmd = Box::new(RemoveEnumValueCommand::cascade(enum_id, "Easy"));
    undo_stack.execute(cmd, &mut project).unwrap();

    let ts = project.defs.tileset(tileset_id).unwrap();
    assert_eq!(ts.marking.source_enum, Some(enum_id));
    assert!(!ts.marking.has_tag("Easy", 1));
    assert!(!ts.marking.has_tag("Easy", 2));
    assert!(ts.marking.has_tag("Hard", 5));

    // Revert restores "Easy" tags identically
    undo_stack.undo(&mut project);
    assert_eq!(&project, &initial_copy);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Easy", 1));
    assert!(ts.marking.has_tag("Easy", 2));
    assert!(ts.marking.has_tag("Hard", 5));
}

#[test]
fn test_rename_enum_value_id_renames_marking_and_reverts() {
    let (mut project, enum_id, tileset_id) = setup_test_project();
    let initial_copy = project.clone();
    let mut undo_stack = UndoStack::new();

    // Rename "Easy" -> "Normal"
    let cmd = Box::new(RenameEnumValueIdCommand::new(enum_id, "Easy", "Normal"));
    undo_stack.execute(cmd, &mut project).unwrap();

    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(!ts.marking.has_tag("Easy", 1));
    assert!(ts.marking.has_tag("Normal", 1));
    assert!(ts.marking.has_tag("Normal", 2));
    assert!(ts.marking.has_tag("Hard", 5));

    // Revert restores "Easy" markings identically
    undo_stack.undo(&mut project);
    assert_eq!(&project, &initial_copy);
    let ts = project.defs.tileset(tileset_id).unwrap();
    assert!(ts.marking.has_tag("Easy", 1));
    assert!(ts.marking.has_tag("Easy", 2));
    assert!(!ts.marking.has_tag("Normal", 1));
    assert!(ts.marking.has_tag("Hard", 5));
}
