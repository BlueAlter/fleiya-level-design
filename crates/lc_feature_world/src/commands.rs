//! Undoable world-tab commands: level duplication, batch positioning, and layout changes.
//!
//! Structural level creation/removal and single-level moves already live in
//! `lc_core::commands`; this module only adds the operations specific to the
//! world overview.

use std::any::Any;

use glam::IVec2;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{LevelId, WorldId};
use lc_core::project::Project;
use lc_core::world::WorldLayout;

/// Duplicates a level inside a world, inserting the copy right after its source.
#[derive(Debug)]
pub struct DuplicateLevelCommand {
    world_id: WorldId,
    source_id: LevelId,
    new_id: LevelId,
    new_name: String,
    offset: IVec2,
    created: Option<lc_core::level::Level>,
    inserted_at: Option<usize>,
}

impl DuplicateLevelCommand {
    /// Creates a duplicate command placing the copy at `offset` from the source.
    pub fn new(
        world_id: WorldId,
        source_id: LevelId,
        new_id: LevelId,
        new_name: impl Into<String>,
        offset: IVec2,
    ) -> Self {
        Self {
            world_id,
            source_id,
            new_id,
            new_name: new_name.into(),
            offset,
            created: None,
            inserted_at: None,
        }
    }

    /// Identifier assigned to the duplicated level.
    pub fn new_id(&self) -> LevelId {
        self.new_id
    }
}

impl Command for DuplicateLevelCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;

        if let Some(copy) = self.created.clone() {
            let at = self
                .inserted_at
                .map(|index| index.min(world.levels.len()))
                .unwrap_or(world.levels.len());
            world.levels.insert(at, copy);
            return Ok(());
        }

        let source = world
            .level(self.source_id)
            .cloned()
            .ok_or(CoreError::LevelNotFound(self.source_id))?;
        let source_index = world
            .levels
            .iter()
            .position(|level| level.id == self.source_id)
            .ok_or(CoreError::LevelNotFound(self.source_id))?;

        let mut copy = source;
        copy.id = self.new_id;
        copy.name = self.new_name.clone();
        copy.world_pos += self.offset;

        let at = source_index.saturating_add(1).min(world.levels.len());
        world.levels.insert(at, copy.clone());
        self.inserted_at = Some(at);
        self.created = Some(copy);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(world) = p.world_mut(self.world_id) {
            if let Some(index) = world
                .levels
                .iter()
                .position(|level| level.id == self.new_id)
            {
                self.created = Some(world.levels.remove(index));
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Duplicate Level".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Sets world positions for several levels in one undo step.
///
/// Used by linear rearrangement and "snap all levels" so a single drop is a
/// single undo entry regardless of how many levels moved.
#[derive(Debug)]
pub struct SetLevelPositionsCommand {
    world_id: WorldId,
    positions: Vec<(LevelId, IVec2)>,
    previous: Option<Vec<(LevelId, IVec2)>>,
}

impl SetLevelPositionsCommand {
    /// Creates a command from the target positions of the levels to move.
    pub fn new(world_id: WorldId, positions: Vec<(LevelId, IVec2)>) -> Self {
        Self {
            world_id,
            positions,
            previous: None,
        }
    }

    /// Whether the command would move at least one level.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

impl Command for SetLevelPositionsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;

        if self.previous.is_none() {
            let mut previous = Vec::with_capacity(self.positions.len());
            for (id, _) in &self.positions {
                if let Some(level) = world.level(*id) {
                    previous.push((*id, level.world_pos));
                }
            }
            self.previous = Some(previous);
        }

        for (id, pos) in &self.positions {
            if let Some(level) = world.level_mut(*id) {
                level.world_pos = *pos;
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        let Some(previous) = self.previous.take() else {
            return;
        };
        if let Some(world) = p.world_mut(self.world_id) {
            for (id, pos) in previous {
                if let Some(level) = world.level_mut(id) {
                    level.world_pos = pos;
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Move Levels".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Changes the arrangement mode of a world.
#[derive(Debug)]
pub struct SetWorldLayoutCommand {
    world_id: WorldId,
    new_layout: WorldLayout,
    previous: Option<WorldLayout>,
}

impl SetWorldLayoutCommand {
    /// Creates a layout change command.
    pub fn new(world_id: WorldId, new_layout: WorldLayout) -> Self {
        Self {
            world_id,
            new_layout,
            previous: None,
        }
    }
}

impl Command for SetWorldLayoutCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let world = p
            .world_mut(self.world_id)
            .ok_or(CoreError::WorldNotFound(self.world_id))?;
        if self.previous.is_none() {
            self.previous = Some(world.layout);
        }
        world.layout = self.new_layout;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(previous), Some(world)) = (self.previous, p.world_mut(self.world_id)) {
            world.layout = previous;
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_layout) = next.as_any().downcast_ref::<Self>() {
            if next_layout.world_id == self.world_id {
                self.new_layout = next_layout.new_layout;
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Change World Layout".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::World(self.world_id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Adapter delegating to another command while keeping the world tab focused.
///
/// Core level commands focus the level tab on apply/undo; the world overview
/// wants to stay open after moving or creating a level.
#[derive(Debug)]
pub struct WorldScopedCommand<C> {
    inner: C,
    world_id: WorldId,
}

impl<C: Command> WorldScopedCommand<C> {
    /// Wraps `inner`, redirecting focus to `world_id`.
    pub fn new(inner: C, world_id: WorldId) -> Self {
        Self { inner, world_id }
    }
}

impl<C: Command> Command for WorldScopedCommand<C> {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        self.inner.apply(p)
    }

    fn revert(&mut self, p: &mut Project) {
        self.inner.revert(p);
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next) = next.as_any().downcast_ref::<Self>() {
            if next.world_id == self.world_id {
                return self.inner.merge(&next.inner);
            }
        }
        false
    }

    fn label(&self) -> String {
        self.inner.label()
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
    use lc_core::level::Level;
    use lc_core::world::World;

    fn project_with_levels() -> (Project, WorldId) {
        let mut project = Project::new("World Commands");
        let world_id = WorldId::new(1);
        let mut world = World::new(world_id, "Overworld");

        let mut first = Level::new(LevelId::new(1), "First", UVec2::new(100, 80));
        first.world_pos = IVec2::new(0, 0);
        first.world_depth = 0;
        let mut second = Level::new(LevelId::new(2), "Second", UVec2::new(100, 80));
        second.world_pos = IVec2::new(120, 0);
        world.levels.push(first);
        world.levels.push(second);

        project.worlds.push(world);
        (project, world_id)
    }

    fn assert_round_trip(mut command: impl Command, mut project: Project) {
        let before = project.clone();
        command.apply(&mut project).expect("apply succeeds");
        assert_ne!(project, before, "apply must change the project");
        command.revert(&mut project);
        assert_eq!(project, before, "revert restores the project exactly");
    }

    #[test]
    fn duplicate_level_round_trip_and_contents() {
        let (mut project, world_id) = project_with_levels();
        let before = project.clone();
        let mut command = DuplicateLevelCommand::new(
            world_id,
            LevelId::new(1),
            LevelId::new(3),
            "First copy",
            IVec2::new(16, 0),
        );

        command.apply(&mut project).expect("apply succeeds");
        let world = project.world(world_id).expect("world exists");
        assert_eq!(world.levels.len(), 3, "copy is inserted");
        assert_eq!(world.levels[1].id, LevelId::new(3), "inserted after source");
        assert_eq!(world.levels[1].name, "First copy");
        assert_eq!(world.levels[1].world_pos, IVec2::new(16, 0));
        assert_eq!(world.levels[1].size_px, UVec2::new(100, 80));

        command.revert(&mut project);
        assert_eq!(project, before);

        // Redo re-inserts the same copy at the same index.
        command.apply(&mut project).expect("re-apply succeeds");
        let world = project.world(world_id).expect("world exists");
        assert_eq!(world.levels[1].id, LevelId::new(3));
        assert_eq!(world.levels.len(), 3);
    }

    #[test]
    fn duplicate_missing_source_fails() {
        let (mut project, world_id) = project_with_levels();
        let before = project.clone();
        let mut command = DuplicateLevelCommand::new(
            world_id,
            LevelId::new(99),
            LevelId::new(3),
            "Missing",
            IVec2::ZERO,
        );
        assert_eq!(
            command.apply(&mut project),
            Err(CoreError::LevelNotFound(LevelId::new(99)))
        );
        assert_eq!(project, before);
    }

    #[test]
    fn set_positions_round_trip_and_skips_unknown_ids() {
        let (mut project, world_id) = project_with_levels();
        let before = project.clone();
        let mut command = SetLevelPositionsCommand::new(
            world_id,
            vec![
                (LevelId::new(1), IVec2::new(-50, 30)),
                (LevelId::new(2), IVec2::new(200, 30)),
                (LevelId::new(42), IVec2::new(0, 0)),
            ],
        );

        command.apply(&mut project).expect("apply succeeds");
        let world = project.world(world_id).expect("world exists");
        assert_eq!(world.levels[0].world_pos, IVec2::new(-50, 30));
        assert_eq!(world.levels[1].world_pos, IVec2::new(200, 30));

        command.revert(&mut project);
        assert_eq!(project, before);
    }

    #[test]
    fn set_positions_missing_world_fails() {
        let (mut project, _) = project_with_levels();
        let before = project.clone();
        let mut command =
            SetLevelPositionsCommand::new(WorldId::new(9), vec![(LevelId::new(1), IVec2::ZERO)]);
        assert_eq!(
            command.apply(&mut project),
            Err(CoreError::WorldNotFound(WorldId::new(9)))
        );
        assert_eq!(project, before);
    }

    #[test]
    fn layout_command_round_trip_and_merge() {
        let (mut project, world_id) = project_with_levels();
        assert_round_trip(
            SetWorldLayoutCommand::new(world_id, WorldLayout::LinearHorizontal),
            project.clone(),
        );

        let mut first = SetWorldLayoutCommand::new(world_id, WorldLayout::GridVania);
        first.apply(&mut project).expect("apply succeeds");
        let second = SetWorldLayoutCommand::new(world_id, WorldLayout::LinearVertical);
        assert!(first.merge(&second), "same world merges");
        assert_eq!(first.new_layout, WorldLayout::LinearVertical);

        let other = SetWorldLayoutCommand::new(WorldId::new(2), WorldLayout::Free);
        assert!(!first.merge(&other), "different worlds do not merge");
    }

    #[test]
    fn world_scoped_command_redirects_focus_and_delegates() {
        let (mut project, world_id) = project_with_levels();
        let mut command = WorldScopedCommand::new(
            lc_core::commands::MoveLevelCommand::new(LevelId::new(1), IVec2::new(40, 10), 2),
            world_id,
        );

        assert_eq!(command.focus(), Some(DocKind::World(world_id)));
        assert_round_trip(
            WorldScopedCommand::new(
                lc_core::commands::MoveLevelCommand::new(LevelId::new(1), IVec2::new(40, 10), 2),
                world_id,
            ),
            project.clone(),
        );

        command.apply(&mut project).expect("apply succeeds");
        let level = project.level(LevelId::new(1)).expect("level exists");
        assert_eq!(level.world_pos, IVec2::new(40, 10));
        assert_eq!(level.world_depth, 2);
    }
}
