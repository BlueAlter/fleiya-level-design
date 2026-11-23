use glam::UVec2;
use lc_core::defs::{Definitions, EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, IdGenerator, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::{Project, ProjectSettings};
use lc_io::format::ProjectFile;
use lc_io::{load_project, save_project};
use std::fs;

#[test]
fn test_project_file_without_marking_is_byte_identical() {
    // Project JSON representing an existing file before this change (no tags, no marking).
    let raw_json = r#"{
  "version": 1,
  "name": "LegacyProject",
  "settings": {
    "default_grid_size": [
      16,
      16
    ]
  },
  "defs": {
    "tilesets": [
      {
        "id": 1,
        "name": "Dungeon",
        "path": "dungeon.png",
        "tile_size": [
          16,
          16
        ],
        "spacing": 0,
        "padding": 0
      }
    ],
    "enums": [],
    "entities": [],
    "layers": []
  },
  "worlds": []
}"#;

    // Parse the legacy JSON
    let parsed: ProjectFile = serde_json::from_str(raw_json).expect("should parse legacy json");

    let tileset = parsed
        .defs
        .tilesets
        .first()
        .expect("tileset should be present");
    assert!(tileset.tags.is_empty());
    assert!(tileset.marking.is_empty());

    // Serialize back to pretty JSON
    let reserialized = serde_json::to_string_pretty(&parsed).expect("should serialize to json");

    // Must not contain "tags" or "marking" under the tileset, remaining byte-identical to raw_json
    assert!(!reserialized.contains("\"marking\""));
    assert_eq!(
        raw_json.replace("\r\n", "\n"),
        reserialized.replace("\r\n", "\n")
    );
}

#[test]
fn test_tileset_tags_and_marking_json_round_trip() {
    let tileset_id = TilesetDefId(10);
    let enum_id = EnumDefId(42);

    let mut marking = TileMarking::new();
    marking.source_enum = Some(enum_id);
    marking.set_tag("Solid", 0, true);
    marking.set_tag("Solid", 1, true);
    marking.set_tag("Solid", 15, true);
    marking.set_tag("Water", 5, true);

    let tileset = TilesetDef {
        id: tileset_id,
        name: "WorldTiles".to_string(),
        path: "tiles/world.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 1,
        padding: 2,
        tags: vec!["Dungeon".to_string(), "Nature".to_string()],
        marking,
        custom_data: Default::default(),
    };

    let mut defs = Definitions::new();
    defs.tilesets.push(tileset.clone());
    defs.enums.push(EnumDef {
        id: enum_id,
        name: "Collisions".to_string(),
        tileset_id: Some(tileset_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Solid".to_string(),
                name: "Solid".to_string(),
                color: [255, 0, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Water".to_string(),
                name: "Water".to_string(),
                color: [0, 0, 255, 255],
                tile: None,
            },
        ],
    });

    let project_file = ProjectFile {
        version: 1,
        name: "MarkingProject".to_string(),
        settings: ProjectSettings {
            default_grid_size: UVec2::new(16, 16),
        },
        defs,
        worlds: Vec::new(),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&project_file).expect("serialization succeeds");

    // Verify expected structure in JSON
    assert!(
        json_str.contains("\"tags\": [\n          \"Dungeon\",\n          \"Nature\"\n        ]")
    );
    assert!(json_str.contains("\"source_enum\": 42"));
    assert!(json_str.contains("\"Solid\": ["));
    assert!(json_str.contains("\"Water\": ["));

    // Deserialize back
    let round_tripped: ProjectFile =
        serde_json::from_str(&json_str).expect("deserialization succeeds");
    assert_eq!(project_file, round_tripped);

    // Byte-identical re-serialization
    let json_str_2 =
        serde_json::to_string_pretty(&round_tripped).expect("reserialization succeeds");
    assert_eq!(json_str, json_str_2);
}

#[test]
fn test_tileset_marking_multi_file_disk_round_trip() {
    let mut gen = IdGenerator::with_seed(42);
    let tileset_id = gen.next_tileset_def_id();
    let enum_id = gen.next_enum_def_id();

    let mut marking = TileMarking::new();
    marking.source_enum = Some(enum_id);
    marking.set_tag("Wall", 3, true);
    marking.set_tag("Wall", 7, true);
    marking.set_tag("Door", 9, true);

    let tileset = TilesetDef {
        id: tileset_id,
        name: "CastleTiles".to_string(),
        path: "assets/castle.png".to_string(),
        tile_size: UVec2::new(32, 32),
        spacing: 0,
        padding: 0,
        tags: vec!["Interior".to_string(), "Castle".to_string()],
        marking,
        custom_data: Default::default(),
    };

    let enum_def = EnumDef {
        id: enum_id,
        name: "TileTypes".to_string(),
        tileset_id: Some(tileset_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Wall".to_string(),
                name: "Wall".to_string(),
                color: [128, 128, 128, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Door".to_string(),
                name: "Door".to_string(),
                color: [200, 150, 50, 255],
                tile: None,
            },
        ],
    };

    let mut project = Project::new("DiskMarkingProject");
    project.defs.tilesets.push(tileset.clone());
    project.defs.enums.push(enum_def);

    let temp_dir =
        std::env::temp_dir().join(format!("lc_test_tileset_marking_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let proj_file_path = temp_dir.join("project.lcproj");

    save_project(&project, &proj_file_path).expect("save_project succeeds");
    let loaded_project = load_project(&proj_file_path).expect("load_project succeeds");

    assert_eq!(loaded_project.defs.tilesets.len(), 1);
    let loaded_ts = &loaded_project.defs.tilesets[0];
    assert_eq!(loaded_ts, &tileset);
    assert_eq!(loaded_ts.tags, vec!["Interior", "Castle"]);
    assert_eq!(loaded_ts.marking.source_enum, Some(enum_id));
    assert!(loaded_ts.marking.has_tag("Wall", 3));
    assert!(loaded_ts.marking.has_tag("Wall", 7));
    assert!(loaded_ts.marking.has_tag("Door", 9));

    let _ = fs::remove_dir_all(&temp_dir);
}
