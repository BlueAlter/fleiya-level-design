use glam::{IVec2, UVec2, Vec2};
use lc_core::*;
use lc_io::*;
use std::fs;
use std::path::{Path, PathBuf};

fn sample_project_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("examples/sample_project")
}

fn create_sample_project() -> Project {
    let mut gen = IdGenerator::with_seed(1);
    let tileset_id = gen.next_tileset_def_id();
    let tileset = TilesetDef {
        id: tileset_id,
        name: "DungeonTiles".to_string(),
        path: "assets/tilesets/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    let enum_id = gen.next_enum_def_id();
    let enum_def = EnumDef {
        id: enum_id,
        name: "Difficulty".to_string(),
        tileset_id: None,
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
                tile: Some(TileRef::new(tileset_id, 1)),
            },
        ],
    };

    let entity_def_id = gen.next_entity_def_id();
    let entity_def = EntityDef {
        id: entity_def_id,
        name: "SpawnPoint".to_string(),
        size_px: UVec2::new(16, 16),
        color: [0, 100, 255, 255],
        pivot: Vec2::new(0.5, 0.5),
        tile: None,
        fields: vec![
            FieldDef {
                name: "checkpoint_id".to_string(),
                field_type: FieldType::Int,
                default_value: Some(FieldValue::Int(1)),
            },
            FieldDef {
                name: "active".to_string(),
                field_type: FieldType::Bool,
                default_value: Some(FieldValue::Bool(true)),
            },
        ],
    };

    let layer_intgrid_id = gen.next_layer_def_id();
    let layer_intgrid_def = LayerDef {
        id: layer_intgrid_id,
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![IntGridValueDef {
            value: 1,
            name: "SolidWall".to_string(),
            color: [128, 128, 128, 255],
        }],
    };

    let layer_entities_id = gen.next_layer_def_id();
    let layer_entities_def = LayerDef {
        id: layer_entities_id,
        name: "Entities".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    };

    let mut project = Project::new("GoldenSample");
    project.settings.default_grid_size = UVec2::new(16, 16);
    project.defs.tilesets.push(tileset);
    project.defs.enums.push(enum_def);
    project.defs.entities.push(entity_def);
    project.defs.layers.push(layer_intgrid_def);
    project.defs.layers.push(layer_entities_def);

    let world_id = gen.next_world_id();
    let mut world = World::new(world_id, "SampleWorld");
    world.layout = WorldLayout::Free;

    let level_id = gen.next_level_id();
    let mut level = Level::new(level_id, "StartRoom", UVec2::new(256, 256));
    level.world_pos = IVec2::new(0, 0);
    level.world_depth = 0;
    level.fields.set(
        "notes",
        FieldValue::String("First room of the dungeon".to_string()),
    );

    let mut intgrid = Grid::new(16, 16, 0);
    intgrid.set(0, 0, 1).expect("valid grid cell");
    intgrid.set(1, 0, 1).expect("valid grid cell");
    level.layers.push(LayerInstance::new(
        layer_intgrid_id,
        LayerData::IntGrid(intgrid),
    ));

    let mut entity_fields = FieldValues::new();
    entity_fields.set("checkpoint_id", FieldValue::Int(1));
    entity_fields.set("active", FieldValue::Bool(true));
    let entity = EntityInstance {
        id: gen.next_entity_id(),
        def_id: entity_def_id,
        pos_px: IVec2::new(32, 32),
        size_px: UVec2::new(16, 16),
        fields: entity_fields,
    };
    level.layers.push(LayerInstance::new(
        layer_entities_id,
        LayerData::Entities(vec![entity]),
    ));

    world.levels.push(level);
    project.worlds.push(world);
    project
}

fn compare_dirs_byte_identical(dir_a: &Path, dir_b: &Path) {
    let mut entries_a = Vec::new();
    collect_files(dir_a, &mut entries_a);

    let mut entries_b = Vec::new();
    collect_files(dir_b, &mut entries_b);

    assert_eq!(
        entries_a.len(),
        entries_b.len(),
        "Directory file counts differ: {} in {:?} vs {} in {:?}",
        entries_a.len(),
        dir_a,
        entries_b.len(),
        dir_b
    );

    for file_a in entries_a {
        let rel_path = file_a.strip_prefix(dir_a).expect("strip prefix");
        let file_b = dir_b.join(rel_path);

        assert!(
            file_b.exists(),
            "File {} missing in saved directory",
            file_b.display()
        );

        let bytes_a = fs::read(&file_a).expect("read file a");
        let bytes_b = fs::read(&file_b).expect("read file b");

        assert_eq!(
            bytes_a,
            bytes_b,
            "Byte mismatch between {} and {}",
            file_a.display(),
            file_b.display()
        );
    }
}

fn collect_files(dir: &Path, results: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, results);
        } else if path.extension().and_then(|s| s.to_str()) == Some("json")
            || path.extension().and_then(|s| s.to_str()) == Some("lcproj")
        {
            results.push(path);
        }
    }
}

#[test]
fn test_golden_sample_project_round_trip() {
    let sample_dir = sample_project_dir();
    let sample_proj_file = sample_dir.join("project.lcproj");

    // Ensure golden sample project is written and valid
    if !sample_proj_file.exists() {
        let sample = create_sample_project();
        save_project(&sample, &sample_proj_file).expect("Failed to write golden project");
    }

    // 1. Load golden sample project
    let loaded = load_project(&sample_proj_file).expect("Failed to load golden project");
    assert_eq!(loaded.name, "GoldenSample");
    assert_eq!(loaded.worlds.len(), 1);

    // 2. Save loaded project to a temporary directory
    let temp_dir = std::env::temp_dir().join(format!("golden_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    let temp_proj_file = temp_dir.join("project.lcproj");

    save_project(&loaded, &temp_proj_file).expect("Failed to save loaded project to temp dir");

    // 3. Compare byte-for-byte identity of all files
    compare_dirs_byte_identical(&sample_dir, &temp_dir);

    // Clean up temp dir
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_missing_file_typed_error() {
    let missing_path = PathBuf::from("/nonexistent/dir/project.lcproj");
    let err = load_project(&missing_path);
    assert!(err.is_err());
    match err {
        Err(IoError::Fs { path, .. }) => {
            assert_eq!(path, missing_path);
        }
        other => panic!("Expected IoError::Fs, got: {:?}", other),
    }
}

#[test]
fn test_corrupt_json_typed_error() {
    let temp_dir = std::env::temp_dir().join(format!("corrupt_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create dir");

    let corrupt_file = temp_dir.join("project.lcproj");
    fs::write(&corrupt_file, "{ this is not valid json }").expect("write corrupt");

    let err = load_project(&corrupt_file);
    assert!(err.is_err());
    match err {
        Err(IoError::Json { path, .. }) => {
            assert_eq!(path, corrupt_file);
        }
        other => panic!("Expected IoError::Json, got: {:?}", other),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_unknown_newer_version_rejected() {
    let temp_dir = std::env::temp_dir().join(format!("version_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create dir");

    let future_file = temp_dir.join("project.lcproj");
    let content = r#"{
  "version": 999,
  "name": "FutureProject",
  "settings": { "default_grid_size": [16, 16] },
  "defs": { "tilesets": [], "enums": [], "entities": [], "layers": [] },
  "worlds": []
}"#;
    fs::write(&future_file, content).expect("write future version");

    let err = load_project(&future_file);
    assert!(err.is_err());
    match err {
        Err(IoError::UnsupportedVersion {
            version,
            max_supported,
            path,
        }) => {
            assert_eq!(version, 999);
            assert_eq!(max_supported, SUPPORTED_VERSION);
            assert_eq!(path, future_file);
        }
        other => panic!("Expected IoError::UnsupportedVersion, got: {:?}", other),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}
