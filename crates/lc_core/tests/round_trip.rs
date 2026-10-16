use glam::{IVec2, UVec2, Vec2};
use lc_core::*;

#[test]
fn test_core_model_serde_round_trip() {
    let mut gen = IdGenerator::with_seed(1);

    let tileset_id = gen.next_tileset_def_id();
    let tileset = TilesetDef {
        id: tileset_id,
        name: "OverworldTiles".to_string(),
        path: "assets/tilesets/overworld.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 1,
        padding: 2,
    };

    let enum_id = gen.next_enum_def_id();
    let enum_def = EnumDef {
        id: enum_id,
        name: "ItemType".to_string(),
        values: vec![
            EnumValueDef {
                id: "Potion".to_string(),
                name: "Health Potion".to_string(),
                color: [255, 0, 0, 255],
                tile: Some(TileRef::new(tileset_id, 5)),
            },
            EnumValueDef {
                id: "Key".to_string(),
                name: "Golden Key".to_string(),
                color: [255, 215, 0, 255],
                tile: None,
            },
        ],
    };

    let entity_def_id = gen.next_entity_def_id();
    let entity_def = EntityDef {
        id: entity_def_id,
        name: "Door".to_string(),
        size_px: UVec2::new(16, 32),
        color: [139, 69, 19, 255],
        pivot: Vec2::new(0.5, 1.0),
        tile: None,
        fields: vec![
            FieldDef {
                name: "locked".to_string(),
                field_type: FieldType::Bool,
                default_value: Some(FieldValue::Bool(true)),
            },
            FieldDef {
                name: "key_type".to_string(),
                field_type: FieldType::Enum(enum_id),
                default_value: Some(FieldValue::Enum("Key".to_string())),
            },
            FieldDef {
                name: "target_pos".to_string(),
                field_type: FieldType::Point,
                default_value: Some(FieldValue::Point(IVec2::new(100, 200))),
            },
            FieldDef {
                name: "connected_door".to_string(),
                field_type: FieldType::EntityRef,
                default_value: None,
            },
            FieldDef {
                name: "tags".to_string(),
                field_type: FieldType::Array(Box::new(FieldType::String)),
                default_value: Some(FieldValue::Array(vec![FieldValue::String(
                    "dungeon".to_string(),
                )])),
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
        int_grid_values: vec![
            IntGridValueDef {
                value: 1,
                name: "Solid".to_string(),
                color: [255, 255, 255, 255],
            },
            IntGridValueDef {
                value: 2,
                name: "Water".to_string(),
                color: [0, 0, 255, 255],
            },
        ],
    };

    let layer_tiles_id = gen.next_layer_def_id();
    let layer_tiles_def = LayerDef {
        id: layer_tiles_id,
        name: "GroundTiles".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(tileset_id),
        int_grid_values: Vec::new(),
    };

    let layer_entities_id = gen.next_layer_def_id();
    let layer_entities_def = LayerDef {
        id: layer_entities_id,
        name: "Actors".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    };

    let layer_auto_id = gen.next_layer_def_id();
    let layer_auto_def = LayerDef {
        id: layer_auto_id,
        name: "AutoFoliage".to_string(),
        kind: LayerKind::Auto,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(tileset_id),
        int_grid_values: Vec::new(),
    };

    let mut project = Project::new("LevelCraft_Demo");
    project.defs.tilesets.push(tileset);
    project.defs.enums.push(enum_def);
    project.defs.entities.push(entity_def);
    project.defs.layers.push(layer_intgrid_def);
    project.defs.layers.push(layer_tiles_def);
    project.defs.layers.push(layer_entities_def);
    project.defs.layers.push(layer_auto_def);

    let world_id = gen.next_world_id();
    let mut world = World::new(world_id, "DungeonWorld");
    world.layout = WorldLayout::GridVania;
    world.default_grid = Some(UVec2::new(256, 256));

    let level_id = gen.next_level_id();
    let mut level = Level::new(level_id, "Room_01", UVec2::new(256, 256));
    level.world_pos = IVec2::new(0, 0);
    level.world_depth = 1;
    level
        .fields
        .set("ambient_color", FieldValue::Color([20, 20, 30, 255]));
    level.fields.set("difficulty", FieldValue::Float(2.5));

    // IntGrid layer data
    let mut intgrid = Grid::new(16, 16, 0);
    intgrid.set(0, 0, 1).expect("valid coordinate");
    intgrid.set(1, 0, 1).expect("valid coordinate");
    intgrid.set(5, 5, 2).expect("valid coordinate");
    level.layers.push(LayerInstance::new(
        layer_intgrid_id,
        LayerData::IntGrid(intgrid),
    ));

    // Tiles layer data
    let mut tiles_grid = Grid::new(16, 16, None);
    tiles_grid
        .set(
            0,
            0,
            Some(TileRef {
                tileset_id,
                tile_id: 12,
                flip_x: true,
                flip_y: false,
            }),
        )
        .expect("valid coordinate");
    level.layers.push(LayerInstance::new(
        layer_tiles_id,
        LayerData::Tiles(tiles_grid),
    ));

    // Entities layer data
    let entity_inst_id = gen.next_entity_id();
    let mut entity_fields = FieldValues::new();
    entity_fields.set("locked", FieldValue::Bool(false));
    entity_fields.set("key_type", FieldValue::Enum("Potion".to_string()));
    entity_fields.set("target_pos", FieldValue::Point(IVec2::new(32, 64)));
    entity_fields.set("connected_door", FieldValue::EntityRef(None));
    entity_fields.set(
        "tags",
        FieldValue::Array(vec![
            FieldValue::String("exit".to_string()),
            FieldValue::String("boss".to_string()),
        ]),
    );

    let entity_inst = EntityInstance {
        id: entity_inst_id,
        def_id: entity_def_id,
        pos_px: IVec2::new(32, 48),
        size_px: UVec2::new(16, 32),
        fields: entity_fields,
    };
    level.layers.push(LayerInstance::new(
        layer_entities_id,
        LayerData::Entities(vec![entity_inst]),
    ));

    // Auto layer data
    level
        .layers
        .push(LayerInstance::new(layer_auto_id, LayerData::Auto));

    world.levels.push(level);
    project.worlds.push(world);

    // Round-trip test via in-memory serializer (RON)
    let serialized = ron::to_string(&project).expect("Serialization to RON failed");
    assert!(!serialized.is_empty());

    let deserialized: Project =
        ron::from_str(&serialized).expect("Deserialization from RON failed");

    assert_eq!(project, deserialized);
    assert_eq!(deserialized.version, Project::CURRENT_VERSION);
    assert_eq!(deserialized.name, "LevelCraft_Demo");

    // Verify lookup helpers still work on deserialized object
    assert!(deserialized.world(world_id).is_some());
    assert!(deserialized.level(level_id).is_some());
    assert!(deserialized.tileset_def(tileset_id).is_some());
    assert!(deserialized.enum_def(enum_id).is_some());
    assert!(deserialized.entity_def(entity_def_id).is_some());
    assert!(deserialized.layer_def(layer_intgrid_id).is_some());

    let lvl = deserialized.level(level_id).expect("Level exists");
    assert!(lvl.layer(layer_intgrid_id).is_some());
    assert!(lvl.layer(layer_tiles_id).is_some());
    assert!(lvl.layer(layer_entities_id).is_some());
    assert!(lvl.layer(layer_auto_id).is_some());
    assert!(lvl.entity(entity_inst_id).is_some());
}
