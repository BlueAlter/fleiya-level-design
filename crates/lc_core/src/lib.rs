#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! `lc_core`: Core data model, identifiers, and grid storage for Level Craft.
//! No egui dependency. No filesystem access.

pub mod defs;
pub mod error;
pub mod fields;
pub mod grid;
pub mod id;
pub mod layer;
pub mod level;
pub mod project;
pub mod world;

pub use defs::{
    Definitions, EntityDef, EnumDef, EnumValueDef, IntGridValueDef, LayerDef, TilesetDef,
};
pub use error::CoreError;
pub use fields::{FieldDef, FieldType, FieldValue, FieldValues};
pub use grid::Grid;
pub use id::{
    EntityDefId, EntityId, EnumDefId, IdGenerator, LayerDefId, LevelId, ProjectId, TilesetDefId,
    WorldId,
};
pub use layer::{EntityInstance, LayerData, LayerInstance, LayerKind, TileRef};
pub use level::Level;
pub use project::{Project, ProjectSettings};
pub use world::{World, WorldLayout};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{IVec2, UVec2, Vec2};

    #[test]
    fn test_id_generator_determinism() {
        let mut gen1 = IdGenerator::with_seed(42);
        let mut gen2 = IdGenerator::with_seed(42);

        for _ in 0..10 {
            assert_eq!(gen1.next_raw(), gen2.next_raw());
        }

        let mut gen = IdGenerator::with_seed(100);
        assert_eq!(gen.next_project_id(), ProjectId(100));
        assert_eq!(gen.next_world_id(), WorldId(101));
        assert_eq!(gen.next_level_id(), LevelId(102));
        assert_eq!(gen.next_layer_def_id(), LayerDefId(103));
        assert_eq!(gen.next_entity_def_id(), EntityDefId(104));
        assert_eq!(gen.next_enum_def_id(), EnumDefId(105));
        assert_eq!(gen.next_tileset_def_id(), TilesetDefId(106));
        assert_eq!(gen.next_entity_id(), EntityId(107));
    }

    #[test]
    fn test_grid_operations_and_bounds() {
        let mut grid: Grid<i32> = Grid::new(4, 3, 0);
        assert_eq!(grid.width(), 4);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.cells().len(), 12);

        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(3, 2));
        assert!(!grid.in_bounds(4, 2));
        assert!(!grid.in_bounds(2, 3));

        assert!(grid.set(2, 1, 99).is_ok());
        assert_eq!(grid.get(2, 1), Some(&99));
        assert_eq!(grid.get(0, 0), Some(&0));
        assert_eq!(grid.get(4, 0), None);

        assert!(grid.set(4, 0, 1).is_err());
    }

    #[test]
    fn test_id_lookups_no_indices() {
        let mut project = Project::new("TestProject");
        let world_id = WorldId(10);
        let level_id = LevelId(20);
        let layer_def_id = LayerDefId(30);
        let entity_def_id = EntityDefId(40);
        let entity_id = EntityId(50);

        let mut world = World::new(world_id, "MainWorld");
        let mut level = Level::new(level_id, "Level_1", UVec2::new(100, 100));

        let entity = EntityInstance {
            id: entity_id,
            def_id: entity_def_id,
            pos_px: IVec2::new(10, 20),
            size_px: UVec2::new(16, 16),
            fields: FieldValues::new(),
        };

        let layer = LayerInstance::new(layer_def_id, LayerData::Entities(vec![entity]));
        level.layers.push(layer);
        world.levels.push(level);
        project.worlds.push(world);

        // ID Lookups on Project
        let found_world = project.world(world_id);
        assert!(found_world.is_some());
        let found_level = project.level(level_id);
        assert!(found_level.is_some());

        // ID Lookups on Level
        if let Some(lvl) = found_level {
            assert!(lvl.layer(layer_def_id).is_some());
            assert!(lvl.entity(entity_id).is_some());
            assert_eq!(lvl.entity(EntityId(999)), None);
        }

        // Test defs lookup
        let entity_def = EntityDef {
            id: entity_def_id,
            name: "Player".to_string(),
            size_px: UVec2::new(16, 16),
            color: [255, 0, 0, 255],
            pivot: Vec2::new(0.5, 0.5),
            tile: None,
            fields: Vec::new(),
        };
        project.defs.entities.push(entity_def);
        assert!(project.entity_def(entity_def_id).is_some());
        assert_eq!(project.entity_def(EntityDefId(999)), None);
    }
}
