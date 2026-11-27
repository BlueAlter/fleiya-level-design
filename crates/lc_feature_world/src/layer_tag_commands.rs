//! Undoable commands for per-World Layer Tag filtering and explicit level layer pruning.

use std::any::Any;

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{LevelId, WorldId};
use lc_core::layer::LayerInstance;
use lc_core::project::Project;

/// Updates `World::included_layer_tags` and `World::excluded_layer_tags`.
#[derive(Debug, Clone)]
pub struct SetWorldLayerTagsCommand {
    world_id: WorldId,
    included_layer_tags: Vec<String>,
    excluded_layer_tags: Vec<String>,
    previous: Option<(Vec<String>, Vec<String>)>,
    stroke_id: Option<u64>,
}

impl SetWorldLayerTagsCommand {
    /// Creates a command setting include/exclude layer tags on `world_id`.
    pub fn new(
        world_id: WorldId,
        included_layer_tags: Vec<String>,
        excluded_layer_tags: Vec<String>,
    ) -> Self {
        Self {
            world_id,
            included_layer_tags,
            excluded_layer_tags,
            previous: None,
            stroke_id: None,
        }
    }

    /// Creates a mergeable command tagged with a UI stroke ID.
    pub fn with_stroke(
        world_id: WorldId,
        included_layer_tags: Vec<String>,
        excluded_layer_tags: Vec<String>,
        stroke_id: u64,
    ) -> Self {
        Self {
            world_id,
            included_layer_tags,
            excluded_layer_tags,
            previous: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for SetWorldLayerTagsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        if self.previous.is_none() {
            self.previous = Some((
                world.included_layer_tags.clone(),
                world.excluded_layer_tags.clone(),
            ));
        }
        world.included_layer_tags = self.included_layer_tags.clone();
        world.excluded_layer_tags = self.excluded_layer_tags.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some((prev_inc, prev_exc)) = self.previous.take() {
            if let Some(world) = p.world_mut(self.world_id) {
                world.included_layer_tags = prev_inc;
                world.excluded_layer_tags = prev_exc;
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.world_id == other.world_id
                && self.stroke_id.is_some()
                && self.stroke_id == other.stroke_id
            {
                self.included_layer_tags = other.included_layer_tags.clone();
                self.excluded_layer_tags = other.excluded_layer_tags.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Update World Layer Tags".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Explicitly removes `LayerInstance`s from levels in `world_id` whose `LayerDef`
/// does not match `world.layer_matches()`, preserving full undo/revert parity.
#[derive(Debug, Clone)]
pub struct PruneUnusedWorldLayersCommand {
    world_id: WorldId,
    removed_instances: Vec<(LevelId, usize, LayerInstance)>,
}

impl PruneUnusedWorldLayersCommand {
    /// Creates a command to prune unused layer instances inside `world_id`.
    pub const fn new(world_id: WorldId) -> Self {
        Self {
            world_id,
            removed_instances: Vec::new(),
        }
    }

    /// Counts how many `LayerInstance`s in `world_id` do not match the world's layer tag filter.
    pub fn count_unused(p: &Project, world_id: WorldId) -> usize {
        let Some(world) = p.world(world_id) else {
            return 0;
        };
        let mut count = 0usize;
        for level in &world.levels {
            for inst in &level.layers {
                let keep = p
                    .defs
                    .layer(inst.def_id)
                    .is_some_and(|def| world.layer_matches(def));
                if !keep {
                    count += 1;
                }
            }
        }
        count
    }
}

impl Command for PruneUnusedWorldLayersCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let defs = p.defs.layers.clone();
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;

        let mut keep_flags = Vec::with_capacity(defs.len());
        for def in &defs {
            keep_flags.push((def.id, world.layer_matches(def)));
        }

        self.removed_instances.clear();
        for level in &mut world.levels {
            let mut idx = 0usize;
            while idx < level.layers.len() {
                let def_id = level.layers[idx].def_id;
                let keep = keep_flags
                    .iter()
                    .find(|(id, _)| *id == def_id)
                    .is_some_and(|(_, k)| *k);
                if keep {
                    idx += 1;
                } else {
                    let removed = level.layers.remove(idx);
                    self.removed_instances.push((level.id, idx, removed));
                }
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(world) = p.world_mut(self.world_id) {
            for (level_id, idx, inst) in self.removed_instances.drain(..).rev() {
                if let Some(level) = world.level_mut(level_id) {
                    let at = idx.min(level.layers.len());
                    level.layers.insert(at, inst);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Prune Unused World Layers".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::defs::LayerDef;
    use lc_core::grid::Grid;
    use lc_core::id::{LayerDefId, LevelId, WorldId};
    use lc_core::layer::{LayerData, LayerKind};
    use lc_core::level::Level;
    use lc_core::world::World;

    #[test]
    fn set_world_layer_tags_and_prune_unused_layers_revert_cleanly() {
        let mut p = Project::new("World Tag Filter");
        let mut tundra_def = LayerDef::new(LayerDefId(1), "Tundra", LayerKind::Tiles);
        tundra_def.tags = vec!["tundra".to_string()];
        let mut desert_def = LayerDef::new(LayerDefId(2), "Desert", LayerKind::Tiles);
        desert_def.tags = vec!["desert".to_string()];
        p.defs.layers.push(tundra_def);
        p.defs.layers.push(desert_def);

        let wid = WorldId(1);
        let mut world = World::new(wid, "World_1");
        let mut lvl = Level::new(LevelId(10), "Lvl", UVec2::splat(64));
        lvl.layers.push(LayerInstance::new(
            LayerDefId(1),
            LayerData::Tiles(Grid::new(4, 4, None)),
        ));
        lvl.layers.push(LayerInstance::new(
            LayerDefId(2),
            LayerData::Tiles(Grid::new(4, 4, None)),
        ));
        world.levels.push(lvl);
        p.worlds.push(world);

        let before_tags = p.clone();
        let mut cmd_tags = SetWorldLayerTagsCommand::new(wid, vec!["tundra".to_string()], vec![]);
        cmd_tags.apply(&mut p).expect("apply tags");
        assert_eq!(PruneUnusedWorldLayersCommand::count_unused(&p, wid), 1);

        let before_prune = p.clone();
        let mut cmd_prune = PruneUnusedWorldLayersCommand::new(wid);
        cmd_prune.apply(&mut p).expect("prune");
        assert_eq!(p.level(LevelId(10)).unwrap().layers.len(), 1);
        assert_eq!(
            p.level(LevelId(10)).unwrap().layers[0].def_id,
            LayerDefId(1)
        );

        cmd_prune.revert(&mut p);
        assert_eq!(p, before_prune);

        cmd_tags.revert(&mut p);
        assert_eq!(p, before_tags);
    }
}
