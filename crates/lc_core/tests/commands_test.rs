use glam::{IVec2, UVec2};
use lc_core::*;

/// Property-style test helper: applying and then reverting any command must yield
/// a Project that is byte-for-byte / structurally equal to the original.
fn assert_apply_revert<C: Command>(original: &Project, mut command: C) {
    let mut working = original.clone();
    command.apply(&mut working).expect("Command apply failed");
    assert_ne!(
        original,
        &working,
        "Command must actually mutate the project on apply: {}",
        command.label()
    );
    command.revert(&mut working);
    assert_eq!(
        original,
        &working,
        "Command revert must restore identical project state for: {}",
        command.label()
    );
}

fn create_sample_project() -> Project {
    let mut project = Project::new("CommandTestProject");
    let mut world1 = World::new(WorldId(1), "Overworld");
    world1
        .levels
        .push(Level::new(LevelId(101), "Level_A", UVec2::new(100, 100)));
    world1
        .levels
        .push(Level::new(LevelId(102), "Level_B", UVec2::new(100, 100)));
    world1
        .levels
        .push(Level::new(LevelId(103), "Level_C", UVec2::new(100, 100)));

    let world2 = World::new(WorldId(2), "Underworld");
    project.worlds.push(world1);
    project.worlds.push(world2);
    project
}

#[test]
fn test_all_commands_apply_revert_property() {
    let project = create_sample_project();

    // 1. AddWorldCommand
    let new_world = World::new(WorldId(3), "SkyWorld");
    assert_apply_revert(&project, AddWorldCommand::new(new_world));

    // 2. RenameWorldCommand
    assert_apply_revert(
        &project,
        RenameWorldCommand::new(WorldId(1), "RenamedOverworld"),
    );

    // 3. RemoveWorldCommand
    assert_apply_revert(&project, RemoveWorldCommand::new(WorldId(2)));

    // 4. AddLevelCommand
    let new_level = Level::new(LevelId(104), "Level_D", UVec2::new(200, 200));
    assert_apply_revert(&project, AddLevelCommand::new(WorldId(1), new_level));

    // 5. RenameLevelCommand
    assert_apply_revert(
        &project,
        RenameLevelCommand::new(LevelId(101), "Renamed_Level_A"),
    );

    // 6. RemoveLevelCommand
    assert_apply_revert(&project, RemoveLevelCommand::new(WorldId(1), LevelId(102)));

    // 7. MoveLevelCommand
    assert_apply_revert(
        &project,
        MoveLevelCommand::new(LevelId(101), IVec2::new(50, 60), 2),
    );
}

#[test]
fn test_level_removal_preserves_exact_order_on_revert() {
    let mut project = create_sample_project();
    let original = project.clone();

    // Remove middle level (Level_B, index 1)
    let mut cmd = RemoveLevelCommand::new(WorldId(1), LevelId(102));
    cmd.apply(&mut project).expect("RemoveLevel apply failed");

    let world = project.world(WorldId(1)).expect("World 1 exists");
    assert_eq!(world.levels.len(), 2);
    assert_eq!(world.levels[0].name, "Level_A");
    assert_eq!(world.levels[1].name, "Level_C");

    // Revert and ensure Level_B is back at index 1 exactly
    cmd.revert(&mut project);
    assert_eq!(project, original);

    let restored_world = project.world(WorldId(1)).expect("World 1 exists");
    assert_eq!(restored_world.levels.len(), 3);
    assert_eq!(restored_world.levels[0].id, LevelId(101));
    assert_eq!(restored_world.levels[1].id, LevelId(102));
    assert_eq!(restored_world.levels[2].id, LevelId(103));
}

#[test]
fn test_world_removal_preserves_exact_order_on_revert() {
    let mut project = create_sample_project();
    project.worlds.push(World::new(WorldId(3), "ThirdWorld"));
    let original = project.clone();

    // Remove middle world (Underworld, index 1)
    let mut cmd = RemoveWorldCommand::new(WorldId(2));
    cmd.apply(&mut project).expect("RemoveWorld apply failed");

    assert_eq!(project.worlds.len(), 2);
    assert_eq!(project.worlds[0].id, WorldId(1));
    assert_eq!(project.worlds[1].id, WorldId(3));

    cmd.revert(&mut project);
    assert_eq!(project, original);
    assert_eq!(project.worlds[0].id, WorldId(1));
    assert_eq!(project.worlds[1].id, WorldId(2));
    assert_eq!(project.worlds[2].id, WorldId(3));
}

#[test]
fn test_move_level_command_merging() {
    let mut project = create_sample_project();
    let original = project.clone();
    let mut undo_stack = UndoStack::new();

    let level_id = LevelId(101);

    // Initial state: pos (0, 0), depth 0
    let level = project.level(level_id).expect("Level exists");
    assert_eq!(level.world_pos, IVec2::ZERO);
    assert_eq!(level.world_depth, 0);

    // Simulate drag gesture: series of MoveLevelCommand events
    undo_stack
        .execute(
            Box::new(MoveLevelCommand::new(level_id, IVec2::new(10, 10), 0)),
            &mut project,
        )
        .expect("Command 1 failed");

    undo_stack
        .execute(
            Box::new(MoveLevelCommand::new(level_id, IVec2::new(20, 25), 0)),
            &mut project,
        )
        .expect("Command 2 failed");

    undo_stack
        .execute(
            Box::new(MoveLevelCommand::new(level_id, IVec2::new(50, 75), 3)),
            &mut project,
        )
        .expect("Command 3 failed");

    // All consecutive moves for the same level must have collapsed into 1 undo entry
    let updated_level = project.level(level_id).expect("Level exists");
    assert_eq!(updated_level.world_pos, IVec2::new(50, 75));
    assert_eq!(updated_level.world_depth, 3);
    assert!(undo_stack.can_undo());
    assert!(!undo_stack.can_redo());

    // Single undo must revert all collapsed moves back to original state!
    let undo_result = undo_stack.undo(&mut project);
    assert!(undo_result.is_some());
    assert_eq!(undo_result.unwrap().0, "Move Level");
    assert_eq!(project, original);
    assert!(!undo_stack.can_undo());
    assert!(undo_stack.can_redo());

    // Single redo must re-apply the final collapsed state
    let redo_result = undo_stack.redo(&mut project).expect("redo succeeded");
    assert!(redo_result.is_some());
    let redone_level = project.level(level_id).expect("Level exists");
    assert_eq!(redone_level.world_pos, IVec2::new(50, 75));
    assert_eq!(redone_level.world_depth, 3);
}

#[test]
fn test_undo_stack_clear_and_labels() {
    let mut project = create_sample_project();
    let mut undo_stack = UndoStack::new();

    undo_stack
        .execute(
            Box::new(RenameWorldCommand::new(WorldId(1), "NewName")),
            &mut project,
        )
        .expect("Rename succeeded");

    assert_eq!(undo_stack.undo_label().as_deref(), Some("Rename World"));
    assert_eq!(undo_stack.redo_label(), None);

    undo_stack.undo(&mut project);
    assert_eq!(undo_stack.undo_label(), None);
    assert_eq!(undo_stack.redo_label().as_deref(), Some("Rename World"));

    undo_stack.clear();
    assert!(!undo_stack.can_undo());
    assert!(!undo_stack.can_redo());
}
