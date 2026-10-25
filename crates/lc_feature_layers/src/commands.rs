//! Commands for creating, updating, removing, and reordering Layer definitions and instances.

use lc_core::command::{Command, DocKind};
use lc_core::defs::LayerDef;
use lc_core::error::CoreError;
use lc_core::grid::Grid;
use lc_core::id::{LayerDefId, LevelId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind};
use lc_core::project::Project;

/// Records the location where a matching layer instance was inserted into a level.
pub type InsertedLevelLayer = (WorldId, LevelId, usize);

/// Records a layer instance removed from a level during cascading deletion.
pub type RemovedLayerInstance = (WorldId, LevelId, usize, LayerInstance);

/// Adds a new `LayerDef` to the project and instantiates corresponding layer instances across levels.
#[derive(Debug, Clone)]
pub struct AddLayerDefCommand {
    def: LayerDef,
    inserted_index: Option<usize>,
    inserted_level_layers: Vec<InsertedLevelLayer>,
}

impl AddLayerDefCommand {
    /// Creates a command to add a new layer definition.
    pub fn new(def: LayerDef) -> Self {
        Self {
            def,
            inserted_index: None,
            inserted_level_layers: Vec::new(),
        }
    }

    /// Accessor for the definition to be added.
    pub fn def(&self) -> &LayerDef {
        &self.def
    }
}

impl Command for AddLayerDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if p.defs.layer(self.def.id).is_some() {
            return Err(CoreError::DefNotFound(format!(
                "LayerDef with id {:?} already exists",
                self.def.id
            )));
        }

        let idx = p.defs.layers.len();
        self.inserted_index = Some(idx);
        p.defs.layers.push(self.def.clone());

        // Instantiate layer instance in each level matching LayerDef
        self.inserted_level_layers.clear();
        for world in &mut p.worlds {
            for level in &mut world.levels {
                let cell_w = (level.size_px.x / self.def.grid_size.x.max(1)).max(1);
                let cell_h = (level.size_px.y / self.def.grid_size.y.max(1)).max(1);

                let data = match self.def.kind {
                    LayerKind::IntGrid => LayerData::IntGrid(Grid::new(cell_w, cell_h, 0)),
                    LayerKind::Tiles => LayerData::Tiles(Grid::new(cell_w, cell_h, None)),
                    LayerKind::Entities => LayerData::Entities(Vec::new()),
                    LayerKind::Auto => LayerData::Auto,
                };

                let instance = LayerInstance::new(self.def.id, data);
                let inst_idx = level.layers.len();
                level.layers.push(instance);
                self.inserted_level_layers
                    .push((world.id, level.id, inst_idx));
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(idx) = self.inserted_index.take() {
            if idx < p.defs.layers.len() && p.defs.layers[idx].id == self.def.id {
                p.defs.layers.remove(idx);
            } else if let Some(pos) = p.defs.layers.iter().position(|l| l.id == self.def.id) {
                p.defs.layers.remove(pos);
            }
        }

        for (world_id, level_id, idx) in self.inserted_level_layers.drain(..).rev() {
            if let Some(world) = p.world_mut(world_id) {
                if let Some(level) = world.level_mut(level_id) {
                    if idx < level.layers.len() && level.layers[idx].def_id == self.def.id {
                        level.layers.remove(idx);
                    } else if let Some(pos) =
                        level.layers.iter().position(|l| l.def_id == self.def.id)
                    {
                        level.layers.remove(pos);
                    }
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Layer \"{}\"", self.def.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.def.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Updates an existing `LayerDef` properties.
#[derive(Debug, Clone)]
pub struct UpdateLayerDefCommand {
    updated: LayerDef,
    previous: Option<LayerDef>,
    previous_index: Option<usize>,
}

impl UpdateLayerDefCommand {
    /// Creates a command to update an existing layer definition.
    pub fn new(updated: LayerDef) -> Self {
        Self {
            updated,
            previous: None,
            previous_index: None,
        }
    }

    /// Accessor for the updated definition.
    pub fn updated(&self) -> &LayerDef {
        &self.updated
    }
}

impl Command for UpdateLayerDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let idx = p
            .defs
            .layers
            .iter()
            .position(|l| l.id == self.updated.id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.updated.id)))?;

        self.previous = Some(p.defs.layers[idx].clone());
        self.previous_index = Some(idx);
        p.defs.layers[idx] = self.updated.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(prev), Some(idx)) = (self.previous.take(), self.previous_index.take()) {
            if idx < p.defs.layers.len() && p.defs.layers[idx].id == self.updated.id {
                p.defs.layers[idx] = prev;
            } else if let Some(pos) = p.defs.layers.iter().position(|l| l.id == self.updated.id) {
                p.defs.layers[pos] = prev;
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
        format!("Edit Layer \"{}\"", self.updated.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.updated.id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes a `LayerDef` and cascades deletion to matching layer instances in levels.
#[derive(Debug, Clone)]
pub struct RemoveLayerDefCommand {
    layer_def_id: LayerDefId,
    cascade: bool,
    removed_def: Option<LayerDef>,
    removed_index: Option<usize>,
    removed_instances: Vec<RemovedLayerInstance>,
}

impl RemoveLayerDefCommand {
    /// Creates a command to remove a layer definition with cascade.
    pub fn new(layer_def_id: LayerDefId) -> Self {
        Self::with_cascade(layer_def_id, true)
    }

    /// Creates a command with explicit cascade setting.
    pub fn with_cascade(layer_def_id: LayerDefId, cascade: bool) -> Self {
        Self {
            layer_def_id,
            cascade,
            removed_def: None,
            removed_index: None,
            removed_instances: Vec::new(),
        }
    }

    /// Checks if any instances of this layer have non-empty data.
    pub fn has_data(p: &Project, id: LayerDefId) -> bool {
        for world in &p.worlds {
            for level in &world.levels {
                if let Some(instance) = level.layer(id) {
                    match &instance.data {
                        LayerData::IntGrid(grid) => {
                            if grid.cells().iter().any(|&v| v != 0) {
                                return true;
                            }
                        }
                        LayerData::Tiles(grid) => {
                            if grid.cells().iter().any(|t| t.is_some()) {
                                return true;
                            }
                        }
                        LayerData::Entities(entities) => {
                            if !entities.is_empty() {
                                return true;
                            }
                        }
                        LayerData::Auto => {}
                    }
                }
            }
        }
        false
    }
}

impl Command for RemoveLayerDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let idx = p
            .defs
            .layers
            .iter()
            .position(|l| l.id == self.layer_def_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_def_id)))?;

        if !self.cascade && Self::has_data(p, self.layer_def_id) {
            return Err(CoreError::DefNotFound(format!(
                "LayerDef {:?} contains placed tiles or entities and cannot be removed without cascade",
                self.layer_def_id
            )));
        }

        // Remove instances from all levels
        self.removed_instances.clear();
        for world in &mut p.worlds {
            for level in &mut world.levels {
                if let Some(inst_idx) = level
                    .layers
                    .iter()
                    .position(|l| l.def_id == self.layer_def_id)
                {
                    let removed = level.layers.remove(inst_idx);
                    self.removed_instances
                        .push((world.id, level.id, inst_idx, removed));
                }
            }
        }

        self.removed_index = Some(idx);
        self.removed_def = Some(p.defs.layers.remove(idx));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(def), Some(idx)) = (self.removed_def.take(), self.removed_index.take()) {
            if idx <= p.defs.layers.len() {
                p.defs.layers.insert(idx, def);
            } else {
                p.defs.layers.push(def);
            }
        }

        // Restore instances to levels at original indices
        for (world_id, level_id, idx, instance) in self.removed_instances.drain(..) {
            if let Some(world) = p.world_mut(world_id) {
                if let Some(level) = world.level_mut(level_id) {
                    if idx <= level.layers.len() {
                        level.layers.insert(idx, instance);
                    } else {
                        level.layers.push(instance);
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
            .map(|l| l.name.as_str())
            .unwrap_or("Layer");
        format!("Remove Layer \"{name}\"")
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Reorders `LayerDef`s in the project and syncs the render order of level layer instances.
#[derive(Debug, Clone)]
pub struct ReorderLayerDefsCommand {
    new_order: Vec<LayerDefId>,
    previous_order: Option<Vec<LayerDefId>>,
}

impl ReorderLayerDefsCommand {
    /// Creates a command to reorder layer definitions according to `new_order`.
    pub fn new(new_order: Vec<LayerDefId>) -> Self {
        Self {
            new_order,
            previous_order: None,
        }
    }
}

impl Command for ReorderLayerDefsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        self.previous_order = Some(p.defs.layers.iter().map(|l| l.id).collect());

        // Reorder project defs
        let mut reordered_defs = Vec::with_capacity(p.defs.layers.len());
        for &id in &self.new_order {
            if let Some(pos) = p.defs.layers.iter().position(|l| l.id == id) {
                reordered_defs.push(p.defs.layers.remove(pos));
            }
        }
        reordered_defs.append(&mut p.defs.layers);
        p.defs.layers = reordered_defs;

        // Reorder layer instances in each level
        for world in &mut p.worlds {
            for level in &mut world.levels {
                let mut reordered_inst = Vec::with_capacity(level.layers.len());
                for &id in &self.new_order {
                    if let Some(pos) = level.layers.iter().position(|l| l.def_id == id) {
                        reordered_inst.push(level.layers.remove(pos));
                    }
                }
                reordered_inst.append(&mut level.layers);
                level.layers = reordered_inst;
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous_order.take() {
            let mut reordered_defs = Vec::with_capacity(p.defs.layers.len());
            for &id in &prev {
                if let Some(pos) = p.defs.layers.iter().position(|l| l.id == id) {
                    reordered_defs.push(p.defs.layers.remove(pos));
                }
            }
            reordered_defs.append(&mut p.defs.layers);
            p.defs.layers = reordered_defs;

            for world in &mut p.worlds {
                for level in &mut world.levels {
                    let mut reordered_inst = Vec::with_capacity(level.layers.len());
                    for &id in &prev {
                        if let Some(pos) = level.layers.iter().position(|l| l.def_id == id) {
                            reordered_inst.push(level.layers.remove(pos));
                        }
                    }
                    reordered_inst.append(&mut level.layers);
                    level.layers = reordered_inst;
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Reorder Layers".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Toggles the visibility flag of a layer instance in a specific level.
#[derive(Debug, Clone)]
pub struct ToggleLayerVisibilityCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    previous_visible: Option<bool>,
}

impl ToggleLayerVisibilityCommand {
    /// Creates a command to toggle layer visibility in a level.
    pub fn new(level_id: LevelId, layer_def_id: LayerDefId) -> Self {
        Self {
            level_id,
            layer_def_id,
            previous_visible: None,
        }
    }
}

impl Command for ToggleLayerVisibilityCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;

        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;

        self.previous_visible = Some(layer.visible);
        layer.visible = !layer.visible;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous_visible.take() {
            if let Some(level) = p.level_mut(self.level_id) {
                if let Some(layer) = level.layer_mut(self.layer_def_id) {
                    layer.visible = prev;
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Toggle Layer Visibility".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::defs::IntGridValueDef;
    use lc_core::id::IdGenerator;
    use lc_core::level::Level;
    use lc_core::world::World;

    fn setup_test_project_with_layers() -> (Project, LayerDefId, LevelId, IdGenerator) {
        let mut gen = IdGenerator::new();
        let layer_def_id = gen.next_layer_def_id();
        let layer_def = LayerDef {
            id: layer_def_id,
            name: "Collisions".to_string(),
            kind: LayerKind::IntGrid,
            grid_size: UVec2::new(16, 16),
            tileset_id: None,
            int_grid_values: vec![IntGridValueDef {
                value: 1,
                name: "Solid".to_string(),
                color: [255, 0, 0, 255],
            }],
        };

        let mut project = Project::new("Layer Test");
        project.defs.layers.push(layer_def);

        let world_id = gen.next_world_id();
        let level_id = gen.next_level_id();
        let mut level = Level::new(level_id, "Level_0".to_string(), UVec2::new(64, 64));

        let instance = LayerInstance::new(layer_def_id, LayerData::IntGrid(Grid::new(4, 4, 0)));
        level.layers.push(instance);

        let mut world = World::new(world_id, "World_0".to_string());
        world.levels.push(level);
        project.worlds.push(world);

        (project, layer_def_id, level_id, gen)
    }

    #[test]
    fn test_add_layer_def_command_apply_and_revert() {
        let (mut project, existing_id, level_id, mut gen) = setup_test_project_with_layers();
        let new_id = gen.next_layer_def_id();
        let new_def = LayerDef {
            id: new_id,
            name: "Tiles Layer".to_string(),
            kind: LayerKind::Tiles,
            grid_size: UVec2::new(16, 16),
            tileset_id: None,
            int_grid_values: Vec::new(),
        };

        let mut cmd = AddLayerDefCommand::new(new_def.clone());
        assert_eq!(cmd.focus(), Some(DocKind::LayerDef(new_id)));

        // Apply
        cmd.apply(&mut project).expect("apply should succeed");
        assert_eq!(project.defs.layers.len(), 2);
        assert_eq!(project.defs.layer(new_id), Some(&new_def));

        // Instance in level must also be created
        let level = project.level(level_id).unwrap();
        assert_eq!(level.layers.len(), 2);
        assert_eq!(level.layers[1].def_id, new_id);

        // Revert
        cmd.revert(&mut project);
        assert_eq!(project.defs.layers.len(), 1);
        assert!(project.defs.layer(new_id).is_none());
        assert!(project.defs.layer(existing_id).is_some());

        let level = project.level(level_id).unwrap();
        assert_eq!(level.layers.len(), 1);
        assert_eq!(level.layers[0].def_id, existing_id);
    }

    #[test]
    fn test_update_layer_def_command_apply_revert_merge() {
        let (mut project, layer_id, _, _) = setup_test_project_with_layers();
        let mut updated = project.defs.layer(layer_id).unwrap().clone();
        updated.name = "Collisions Renamed".to_string();

        let mut cmd1 = UpdateLayerDefCommand::new(updated.clone());
        cmd1.apply(&mut project).expect("apply should succeed");
        assert_eq!(
            project.defs.layer(layer_id).unwrap().name,
            "Collisions Renamed"
        );

        // Merge with consecutive edit
        let mut updated2 = updated.clone();
        updated2.name = "Collisions Final".to_string();
        let cmd2 = UpdateLayerDefCommand::new(updated2);

        assert!(cmd1.merge(&cmd2));
        assert_eq!(cmd1.updated.name, "Collisions Final");

        cmd1.revert(&mut project);
        assert_eq!(project.defs.layer(layer_id).unwrap().name, "Collisions");
    }

    #[test]
    fn test_remove_layer_def_command_cascade_and_revert() {
        let (mut project, layer_id, level_id, _) = setup_test_project_with_layers();

        // Put some data in grid
        {
            let level = project.level_mut(level_id).unwrap();
            let layer = level.layer_mut(layer_id).unwrap();
            if let LayerData::IntGrid(ref mut g) = layer.data {
                g.set(0, 0, 1).unwrap();
            }
        }

        assert!(RemoveLayerDefCommand::has_data(&project, layer_id));

        // Non-cascade should fail
        let mut blocking_cmd = RemoveLayerDefCommand::with_cascade(layer_id, false);
        assert!(blocking_cmd.apply(&mut project).is_err());

        // Cascading remove
        let mut cascade_cmd = RemoveLayerDefCommand::new(layer_id);
        cascade_cmd
            .apply(&mut project)
            .expect("cascade apply should succeed");

        assert!(project.defs.layer(layer_id).is_none());
        assert!(project.defs.layers.is_empty());
        assert!(project.level(level_id).unwrap().layers.is_empty());

        // Revert
        cascade_cmd.revert(&mut project);
        assert!(project.defs.layer(layer_id).is_some());
        assert_eq!(project.defs.layers.len(), 1);

        let level = project.level(level_id).unwrap();
        assert_eq!(level.layers.len(), 1);
        if let LayerData::IntGrid(ref g) = level.layers[0].data {
            assert_eq!(g.get(0, 0), Some(&1));
        } else {
            panic!("expected IntGrid layer data");
        }
    }

    #[test]
    fn test_reorder_layer_defs_command() {
        let (mut project, l1, level_id, mut gen) = setup_test_project_with_layers();
        let l2 = gen.next_layer_def_id();
        project.defs.layers.push(LayerDef {
            id: l2,
            name: "L2".to_string(),
            kind: LayerKind::Entities,
            grid_size: UVec2::new(16, 16),
            tileset_id: None,
            int_grid_values: Vec::new(),
        });

        let level = project.level_mut(level_id).unwrap();
        level
            .layers
            .push(LayerInstance::new(l2, LayerData::Entities(Vec::new())));

        assert_eq!(project.defs.layers[0].id, l1);
        assert_eq!(project.defs.layers[1].id, l2);

        // Reorder l2 before l1
        let mut cmd = ReorderLayerDefsCommand::new(vec![l2, l1]);
        cmd.apply(&mut project).expect("apply should succeed");

        assert_eq!(project.defs.layers[0].id, l2);
        assert_eq!(project.defs.layers[1].id, l1);

        let level = project.level(level_id).unwrap();
        assert_eq!(level.layers[0].def_id, l2);
        assert_eq!(level.layers[1].def_id, l1);

        // Revert
        cmd.revert(&mut project);
        assert_eq!(project.defs.layers[0].id, l1);
        assert_eq!(project.defs.layers[1].id, l2);

        let level = project.level(level_id).unwrap();
        assert_eq!(level.layers[0].def_id, l1);
        assert_eq!(level.layers[1].def_id, l2);
    }

    #[test]
    fn test_toggle_layer_visibility_command() {
        let (mut project, layer_id, level_id, _) = setup_test_project_with_layers();
        assert!(
            project
                .level(level_id)
                .unwrap()
                .layer(layer_id)
                .unwrap()
                .visible
        );

        let mut cmd = ToggleLayerVisibilityCommand::new(level_id, layer_id);
        cmd.apply(&mut project).expect("apply should succeed");
        assert!(
            !project
                .level(level_id)
                .unwrap()
                .layer(layer_id)
                .unwrap()
                .visible
        );

        cmd.revert(&mut project);
        assert!(
            project
                .level(level_id)
                .unwrap()
                .layer(layer_id)
                .unwrap()
                .visible
        );
    }
}
