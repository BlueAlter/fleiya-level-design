//! Structural commands for Worlds and Levels.

use crate::command::{Command, DocKind};
use crate::error::CoreError;
use crate::id::{LevelId, WorldId};
use crate::level::Level;
use crate::project::Project;
use crate::world::World;
use glam::IVec2;
use std::any::Any;

/// Adds a new world to the project.
#[derive(Debug)]
pub struct AddWorldCommand {
    world: Option<World>,
    created_id: Option<WorldId>,
}

impl AddWorldCommand {
    /// Creates a new `AddWorldCommand` taking ownership of the world to add.
    pub fn new(world: World) -> Self {
        Self {
            world: Some(world),
            created_id: None,
        }
    }
}

impl Command for AddWorldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = self
            .world
            .take()
            .ok_or_else(|| CoreError::DefNotFound("World already applied".to_string()))?;
        self.created_id = Some(world.id);
        p.worlds.push(world);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(id) = self.created_id {
            if let Some(idx) = p.worlds.iter().position(|w| w.id == id) {
                let removed = p.worlds.remove(idx);
                self.world = Some(removed);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Add World".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        self.created_id.map(DocKind::World)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Renames an existing world in the project.
#[derive(Debug)]
pub struct RenameWorldCommand {
    pub world_id: WorldId,
    pub new_name: String,
    old_name: Option<String>,
}

impl RenameWorldCommand {
    /// Creates a new `RenameWorldCommand`.
    pub fn new(world_id: WorldId, new_name: impl Into<String>) -> Self {
        Self {
            world_id,
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl Command for RenameWorldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        if self.old_name.is_none() {
            self.old_name = Some(world.name.clone());
        }
        world.name = self.new_name.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old) = &self.old_name {
            if let Some(world) = p.world_mut(self.world_id) {
                world.name = old.clone();
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_rename) = next.as_any().downcast_ref::<Self>() {
            if next_rename.world_id == self.world_id {
                self.new_name = next_rename.new_name.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Rename World".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Removes a world from the project, preserving its exact position for clean reversion.
#[derive(Debug)]
pub struct RemoveWorldCommand {
    pub world_id: WorldId,
    removed_world: Option<World>,
    removed_index: Option<usize>,
}

impl RemoveWorldCommand {
    /// Creates a new `RemoveWorldCommand`.
    pub const fn new(world_id: WorldId) -> Self {
        Self {
            world_id,
            removed_world: None,
            removed_index: None,
        }
    }
}

impl Command for RemoveWorldCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let idx = p
            .worlds
            .iter()
            .position(|w| w.id == self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        self.removed_index = Some(idx);
        self.removed_world = Some(p.worlds.remove(idx));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(world), Some(idx)) = (self.removed_world.take(), self.removed_index) {
            if idx <= p.worlds.len() {
                p.worlds.insert(idx, world);
            } else {
                p.worlds.push(world);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Remove World".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Adds a new level to a world.
#[derive(Debug)]
pub struct AddLevelCommand {
    pub world_id: WorldId,
    level: Option<Level>,
    created_id: Option<LevelId>,
}

impl AddLevelCommand {
    /// Creates a new `AddLevelCommand`.
    pub fn new(world_id: WorldId, level: Level) -> Self {
        Self {
            world_id,
            level: Some(level),
            created_id: None,
        }
    }
}

impl Command for AddLevelCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = self
            .level
            .take()
            .ok_or_else(|| CoreError::DefNotFound("Level already added".to_string()))?;
        self.created_id = Some(level.id);
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        world.levels.push(level);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(level_id) = self.created_id {
            if let Some(world) = p.world_mut(self.world_id) {
                if let Some(idx) = world.levels.iter().position(|l| l.id == level_id) {
                    let removed = world.levels.remove(idx);
                    self.level = Some(removed);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Add Level".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        self.created_id.map(DocKind::Level)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Renames a level in the project.
#[derive(Debug)]
pub struct RenameLevelCommand {
    pub level_id: LevelId,
    pub new_name: String,
    old_name: Option<String>,
}

impl RenameLevelCommand {
    /// Creates a new `RenameLevelCommand`.
    pub fn new(level_id: LevelId, new_name: impl Into<String>) -> Self {
        Self {
            level_id,
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl Command for RenameLevelCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        if self.old_name.is_none() {
            self.old_name = Some(level.name.clone());
        }
        level.name = self.new_name.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old) = &self.old_name {
            if let Some(level) = p.level_mut(self.level_id) {
                level.name = old.clone();
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_rename) = next.as_any().downcast_ref::<Self>() {
            if next_rename.level_id == self.level_id {
                self.new_name = next_rename.new_name.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Rename Level".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Removes a level from a world, preserving its exact position for clean reversion.
#[derive(Debug)]
pub struct RemoveLevelCommand {
    pub world_id: WorldId,
    pub level_id: LevelId,
    removed_level: Option<Level>,
    removed_index: Option<usize>,
}

impl RemoveLevelCommand {
    /// Creates a new `RemoveLevelCommand`.
    pub const fn new(world_id: WorldId, level_id: LevelId) -> Self {
        Self {
            world_id,
            level_id,
            removed_level: None,
            removed_index: None,
        }
    }
}

impl Command for RemoveLevelCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        let idx = world
            .levels
            .iter()
            .position(|l| l.id == self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        self.removed_index = Some(idx);
        self.removed_level = Some(world.levels.remove(idx));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(level), Some(idx)) = (self.removed_level.take(), self.removed_index) {
            if let Some(world) = p.world_mut(self.world_id) {
                if idx <= world.levels.len() {
                    world.levels.insert(idx, level);
                } else {
                    world.levels.push(level);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Remove Level".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Moves a level in world position and/or z-depth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveLevelCommand {
    pub level_id: LevelId,
    pub new_pos: IVec2,
    pub new_depth: i32,
    old_pos: Option<IVec2>,
    old_depth: Option<i32>,
}

impl MoveLevelCommand {
    /// Creates a new `MoveLevelCommand`.
    pub const fn new(level_id: LevelId, new_pos: IVec2, new_depth: i32) -> Self {
        Self {
            level_id,
            new_pos,
            new_depth,
            old_pos: None,
            old_depth: None,
        }
    }
}

impl Command for MoveLevelCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        if self.old_pos.is_none() {
            self.old_pos = Some(level.world_pos);
        }
        if self.old_depth.is_none() {
            self.old_depth = Some(level.world_depth);
        }
        level.world_pos = self.new_pos;
        level.world_depth = self.new_depth;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(pos), Some(depth)) = (self.old_pos, self.old_depth) {
            if let Some(level) = p.level_mut(self.level_id) {
                level.world_pos = pos;
                level.world_depth = depth;
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_move) = next.as_any().downcast_ref::<Self>() {
            if next_move.level_id == self.level_id {
                self.new_pos = next_move.new_pos;
                self.new_depth = next_move.new_depth;
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Move Level".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
