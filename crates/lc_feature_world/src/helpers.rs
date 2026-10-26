//! Small allocation helpers shared by the world tab.

use lc_core::id::LevelId;
use lc_core::project::Project;

/// Returns the next free level identifier: one above the highest live level.
///
/// The project file has no persisted high-water mark yet, so deleting the
/// newest level frees its identifier for reuse. This matches the existing
/// project-panel behaviour and is tracked as a contract follow-up.
pub fn next_level_id(project: &Project) -> LevelId {
    let highest = project
        .worlds
        .iter()
        .flat_map(|world| &world.levels)
        .map(|level| level.id.0)
        .max()
        .unwrap_or(0);
    LevelId(highest.saturating_add(1))
}

/// Default display name for a newly created level.
pub fn level_name(id: LevelId) -> String {
    format!("Level_{}", id.0)
}

#[cfg(test)]
mod tests {
    use glam::UVec2;
    use lc_core::id::WorldId;
    use lc_core::level::Level;
    use lc_core::world::World;

    use super::*;

    #[test]
    fn next_id_follows_the_highest_live_level() {
        let mut project = Project::new("Ids");
        assert_eq!(next_level_id(&project), LevelId::new(1));

        let mut world = World::new(WorldId::new(1), "W");
        world
            .levels
            .push(Level::new(LevelId::new(4), "A", UVec2::new(16, 16)));
        world
            .levels
            .push(Level::new(LevelId::new(9), "B", UVec2::new(16, 16)));
        project.worlds.push(world);

        assert_eq!(next_level_id(&project), LevelId::new(10));
        assert_eq!(level_name(LevelId::new(10)), "Level_10");
    }
}
