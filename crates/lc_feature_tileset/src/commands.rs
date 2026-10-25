//! Commands for managing Tileset definitions in the editor data model.

pub use crate::paint_commands::{
    FillCommand, FillValue, PaintIntGridCommand, PaintTilesCommand, RectCommand,
};

use lc_core::command::{Command, DocKind};
use lc_core::defs::TilesetDef;
use lc_core::error::CoreError;
use lc_core::id::{EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use lc_core::layer::{LayerData, TileRef};
use lc_core::project::Project;

/// Adds a new `TilesetDef` to the project definitions.
#[derive(Debug, Clone)]
pub struct AddTilesetDefCommand {
    def: TilesetDef,
    inserted_index: Option<usize>,
}

impl AddTilesetDefCommand {
    /// Creates a command to add a new tileset definition.
    pub fn new(def: TilesetDef) -> Self {
        Self {
            def,
            inserted_index: None,
        }
    }

    /// Accessor for the definition to be added.
    pub fn def(&self) -> &TilesetDef {
        &self.def
    }
}

impl Command for AddTilesetDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if p.defs.tileset(self.def.id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "TilesetDef with id {:?} already exists",
                self.def.id
            )));
        }
        let idx = p.defs.tilesets.len();
        self.inserted_index = Some(idx);
        p.defs.tilesets.push(self.def.clone());
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(idx) = self.inserted_index.take() {
            if idx < p.defs.tilesets.len() && p.defs.tilesets[idx].id == self.def.id {
                p.defs.tilesets.remove(idx);
            } else if let Some(pos) = p.defs.tilesets.iter().position(|t| t.id == self.def.id) {
                p.defs.tilesets.remove(pos);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Tileset \"{}\"", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Updates an existing `TilesetDef` properties.
#[derive(Debug, Clone)]
pub struct UpdateTilesetDefCommand {
    updated: TilesetDef,
    previous: Option<TilesetDef>,
    previous_index: Option<usize>,
}

impl UpdateTilesetDefCommand {
    /// Creates a command to update an existing tileset definition.
    pub fn new(updated: TilesetDef) -> Self {
        Self {
            updated,
            previous: None,
            previous_index: None,
        }
    }

    /// Accessor for the updated definition.
    pub fn updated(&self) -> &TilesetDef {
        &self.updated
    }
}

impl Command for UpdateTilesetDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let idx = p
            .defs
            .tilesets
            .iter()
            .position(|t| t.id == self.updated.id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.updated.id)))?;

        self.previous = Some(p.defs.tilesets[idx].clone());
        self.previous_index = Some(idx);
        p.defs.tilesets[idx] = self.updated.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(prev), Some(idx)) = (self.previous.take(), self.previous_index.take()) {
            if idx < p.defs.tilesets.len() && p.defs.tilesets[idx].id == self.updated.id {
                p.defs.tilesets[idx] = prev;
            } else if let Some(pos) = p.defs.tilesets.iter().position(|t| t.id == self.updated.id) {
                p.defs.tilesets[pos] = prev;
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.updated.id == other.updated.id {
                self.updated = other.updated.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Tileset \"{}\"", self.updated.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.updated.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A list of cell index and previous tile reference pairs for a tile layer.
pub type LayerCellReplacements = Vec<(usize, Option<TileRef>)>;

/// Records the location and cell changes of a tile layer affected by a tileset removal.
pub type AffectedTileLayer = (WorldId, LevelId, LayerDefId, LayerCellReplacements);

/// Removes a `TilesetDef` from the project.
///
/// Supports cascading removal: clearing `tileset_id` in referencing `LayerDef`s,
/// clearing tile icons in `EnumDef` values, and clearing tile cells in level tile layers.
/// Revert restores all references and tiles identically.
#[derive(Debug, Clone)]
pub struct RemoveTilesetDefCommand {
    tileset_id: TilesetDefId,
    cascade: bool,
    removed_def: Option<TilesetDef>,
    removed_index: Option<usize>,
    affected_layer_defs: Vec<(LayerDefId, Option<TilesetDefId>)>,
    affected_enum_values: Vec<(EnumDefId, String, Option<TileRef>)>,
    affected_tile_layers: Vec<AffectedTileLayer>,
}

impl RemoveTilesetDefCommand {
    /// Creates a command to remove a tileset with cascading deletion.
    pub fn new(tileset_id: TilesetDefId) -> Self {
        Self::with_cascade(tileset_id, true)
    }

    /// Creates a command to remove a tileset with or without cascade.
    pub fn with_cascade(tileset_id: TilesetDefId, cascade: bool) -> Self {
        Self {
            tileset_id,
            cascade,
            removed_def: None,
            removed_index: None,
            affected_layer_defs: Vec::new(),
            affected_enum_values: Vec::new(),
            affected_tile_layers: Vec::new(),
        }
    }

    /// Checks whether the tileset is currently referenced anywhere in the project.
    pub fn is_referenced(p: &Project, id: TilesetDefId) -> bool {
        // Check LayerDefs
        if p.defs.layers.iter().any(|l| l.tileset_id == Some(id)) {
            return true;
        }

        // Check EnumDefs
        if p.defs.enums.iter().any(|e| {
            e.values
                .iter()
                .any(|v| v.tile.as_ref().map(|t| t.tileset_id) == Some(id))
        }) {
            return true;
        }

        // Check level tile layers
        for world in &p.worlds {
            for level in &world.levels {
                for layer in &level.layers {
                    if let LayerData::Tiles(grid) = &layer.data {
                        if grid
                            .cells()
                            .iter()
                            .any(|cell| cell.as_ref().map(|t| t.tileset_id) == Some(id))
                        {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

impl Command for RemoveTilesetDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let idx = p
            .defs
            .tilesets
            .iter()
            .position(|t| t.id == self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        if !self.cascade && Self::is_referenced(p, self.tileset_id) {
            return Err(CoreError::DefNotFound(format!(
                "TilesetDef {:?} is referenced by layers or enums and cannot be removed without cascade",
                self.tileset_id
            )));
        }

        // Clear and record references
        self.affected_layer_defs.clear();
        for layer_def in &mut p.defs.layers {
            if layer_def.tileset_id == Some(self.tileset_id) {
                self.affected_layer_defs
                    .push((layer_def.id, layer_def.tileset_id));
                layer_def.tileset_id = None;
            }
        }

        self.affected_enum_values.clear();
        for enum_def in &mut p.defs.enums {
            for val in &mut enum_def.values {
                if val.tile.as_ref().map(|t| t.tileset_id) == Some(self.tileset_id) {
                    self.affected_enum_values
                        .push((enum_def.id, val.id.clone(), val.tile));
                    val.tile = None;
                }
            }
        }

        self.affected_tile_layers.clear();
        for world in &mut p.worlds {
            for level in &mut world.levels {
                for layer in &mut level.layers {
                    if let LayerData::Tiles(grid) = &mut layer.data {
                        let mut replaced = Vec::new();
                        let cells = grid.cells_mut();
                        for (cell_idx, cell) in cells.iter_mut().enumerate() {
                            if let Some(t_ref) = cell {
                                if t_ref.tileset_id == self.tileset_id {
                                    replaced.push((cell_idx, *cell));
                                    *cell = None;
                                }
                            }
                        }
                        if !replaced.is_empty() {
                            self.affected_tile_layers.push((
                                world.id,
                                level.id,
                                layer.def_id,
                                replaced,
                            ));
                        }
                    }
                }
            }
        }

        self.removed_index = Some(idx);
        self.removed_def = Some(p.defs.tilesets.remove(idx));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        // 1. Restore definition
        if let (Some(def), Some(idx)) = (self.removed_def.take(), self.removed_index.take()) {
            if idx <= p.defs.tilesets.len() {
                p.defs.tilesets.insert(idx, def);
            } else {
                p.defs.tilesets.push(def);
            }
        }

        // 2. Restore LayerDef references
        for (layer_id, prev_tileset) in self.affected_layer_defs.drain(..) {
            if let Some(l) = p.defs.layer_mut(layer_id) {
                l.tileset_id = prev_tileset;
            }
        }

        // 3. Restore EnumDef references
        for (enum_id, val_id, prev_tile) in self.affected_enum_values.drain(..) {
            if let Some(e) = p.defs.enum_def_mut(enum_id) {
                if let Some(v) = e.values.iter_mut().find(|v| v.id == val_id) {
                    v.tile = prev_tile;
                }
            }
        }

        // 4. Restore Tile layer cells
        for (world_id, level_id, layer_def_id, cells) in self.affected_tile_layers.drain(..) {
            if let Some(world) = p.world_mut(world_id) {
                if let Some(level) = world.level_mut(level_id) {
                    if let Some(layer) = level.layer_mut(layer_def_id) {
                        if let LayerData::Tiles(grid) = &mut layer.data {
                            let grid_cells = grid.cells_mut();
                            for (cell_idx, prev_tile) in cells {
                                if cell_idx < grid_cells.len() {
                                    grid_cells[cell_idx] = prev_tile;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        let name = self
            .removed_def
            .as_ref()
            .map(|t| t.name.as_str())
            .unwrap_or("Tileset");
        format!("Remove Tileset \"{name}\"")
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::defs::{EnumDef, EnumValueDef, LayerDef};
    use lc_core::grid::Grid;
    use lc_core::id::IdGenerator;
    use lc_core::layer::{LayerInstance, LayerKind};
    use lc_core::level::Level;
    use lc_core::world::World;

    fn setup_test_project() -> (Project, TilesetDefId, IdGenerator) {
        let mut gen = IdGenerator::new();
        let tileset_id = gen.next_tileset_def_id();
        let tileset = TilesetDef {
            id: tileset_id,
            name: "Dungeon".to_string(),
            path: "tiles/dungeon.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 1,
            padding: 2,
        };

        let mut project = Project::new("Test Project");
        project.defs.tilesets.push(tileset);
        (project, tileset_id, gen)
    }

    #[test]
    fn test_add_tileset_def_command_apply_and_revert() {
        let (mut project, existing_id, mut gen) = setup_test_project();
        let new_id = gen.next_tileset_def_id();
        let new_def = TilesetDef {
            id: new_id,
            name: "Overworld".to_string(),
            path: "tiles/overworld.png".to_string(),
            tile_size: UVec2::new(32, 32),
            spacing: 0,
            padding: 0,
        };

        let mut cmd = AddTilesetDefCommand::new(new_def.clone());
        assert_eq!(cmd.focus(), Some(DocKind::TilesetDef(new_id)));

        // Apply
        cmd.apply(&mut project).expect("apply should succeed");
        assert_eq!(project.defs.tilesets.len(), 2);
        assert_eq!(project.defs.tileset(new_id), Some(&new_def));

        // Duplicate apply should fail
        let mut dup_cmd = AddTilesetDefCommand::new(new_def.clone());
        assert!(dup_cmd.apply(&mut project).is_err());

        // Revert
        cmd.revert(&mut project);
        assert_eq!(project.defs.tilesets.len(), 1);
        assert!(project.defs.tileset(new_id).is_none());
        assert!(project.defs.tileset(existing_id).is_some());
    }

    #[test]
    fn test_update_tileset_def_command_apply_revert_merge() {
        let (mut project, tileset_id, _) = setup_test_project();
        let mut updated = project.defs.tileset(tileset_id).unwrap().clone();
        updated.name = "Dungeon Modified".to_string();
        updated.tile_size = UVec2::new(24, 24);

        let mut cmd1 = UpdateTilesetDefCommand::new(updated.clone());
        cmd1.apply(&mut project).expect("apply should succeed");
        assert_eq!(
            project.defs.tileset(tileset_id).unwrap().name,
            "Dungeon Modified"
        );
        assert_eq!(
            project.defs.tileset(tileset_id).unwrap().tile_size,
            UVec2::new(24, 24)
        );

        // Merge with consecutive edit
        let mut updated2 = updated.clone();
        updated2.name = "Dungeon Final".to_string();
        let cmd2 = UpdateTilesetDefCommand::new(updated2);

        assert!(cmd1.merge(&cmd2));
        assert_eq!(cmd1.updated.name, "Dungeon Final");

        // Revert restores initial state before cmd1
        cmd1.revert(&mut project);
        assert_eq!(project.defs.tileset(tileset_id).unwrap().name, "Dungeon");
        assert_eq!(
            project.defs.tileset(tileset_id).unwrap().tile_size,
            UVec2::new(16, 16)
        );
    }

    #[test]
    fn test_remove_tileset_def_command_unreferenced() {
        let (mut project, tileset_id, _) = setup_test_project();
        let mut cmd = RemoveTilesetDefCommand::new(tileset_id);

        cmd.apply(&mut project).expect("apply should succeed");
        assert!(project.defs.tileset(tileset_id).is_none());
        assert!(project.defs.tilesets.is_empty());

        cmd.revert(&mut project);
        assert_eq!(project.defs.tilesets.len(), 1);
        assert_eq!(project.defs.tileset(tileset_id).unwrap().name, "Dungeon");
    }

    #[test]
    fn test_remove_tileset_def_command_cascade_and_revert() {
        let (mut project, tileset_id, mut gen) = setup_test_project();

        // 1. LayerDef reference
        let layer_def_id = gen.next_layer_def_id();
        project.defs.layers.push(LayerDef {
            id: layer_def_id,
            name: "Foreground".to_string(),
            kind: LayerKind::Tiles,
            grid_size: UVec2::new(16, 16),
            tileset_id: Some(tileset_id),
            int_grid_values: Vec::new(),
        });

        // 2. EnumDef reference
        let enum_id = gen.next_enum_def_id();
        project.defs.enums.push(EnumDef {
            id: enum_id,
            name: "Biomes".to_string(),
            values: vec![EnumValueDef {
                id: "Swamp".to_string(),
                name: "Swamp".to_string(),
                color: [50, 150, 50, 255],
                tile: Some(TileRef::new(tileset_id, 42)),
            }],
        });

        // 3. Tile layer in a level
        let world_id = gen.next_world_id();
        let level_id = gen.next_level_id();
        let mut tile_grid = Grid::new(2, 2, None);
        tile_grid
            .set(1, 1, Some(TileRef::new(tileset_id, 99)))
            .unwrap();

        let mut level = Level::new(level_id, "Level_0".to_string(), UVec2::new(32, 32));
        level.layers.push(LayerInstance {
            def_id: layer_def_id,
            visible: true,
            data: LayerData::Tiles(tile_grid),
        });

        let mut world = World::new(world_id, "World_0".to_string());
        world.levels.push(level);
        project.worlds.push(world);

        // Verify references exist
        assert!(RemoveTilesetDefCommand::is_referenced(&project, tileset_id));

        // Blocking remove (without cascade) must fail
        let mut blocking_cmd = RemoveTilesetDefCommand::with_cascade(tileset_id, false);
        assert!(blocking_cmd.apply(&mut project).is_err());

        // Cascading remove must succeed and clear references
        let mut cascade_cmd = RemoveTilesetDefCommand::new(tileset_id);
        cascade_cmd
            .apply(&mut project)
            .expect("cascade apply should succeed");

        assert!(project.defs.tileset(tileset_id).is_none());
        assert_eq!(project.defs.layer(layer_def_id).unwrap().tileset_id, None);
        assert_eq!(project.defs.enums[0].values[0].tile, None);
        let level_layer = &project.worlds[0].levels[0].layers[0];
        if let LayerData::Tiles(ref g) = level_layer.data {
            assert_eq!(g.get(1, 1), Some(&None));
        } else {
            panic!("expected Tiles layer");
        }

        // Revert must restore EVERYTHING identically
        cascade_cmd.revert(&mut project);

        assert!(project.defs.tileset(tileset_id).is_some());
        assert_eq!(
            project.defs.layer(layer_def_id).unwrap().tileset_id,
            Some(tileset_id)
        );
        assert_eq!(
            project.defs.enums[0].values[0].tile,
            Some(TileRef::new(tileset_id, 42))
        );
        let level_layer = &project.worlds[0].levels[0].layers[0];
        if let LayerData::Tiles(ref g) = level_layer.data {
            assert_eq!(g.get(1, 1), Some(&Some(TileRef::new(tileset_id, 99))));
        } else {
            panic!("expected Tiles layer");
        }
    }
}
