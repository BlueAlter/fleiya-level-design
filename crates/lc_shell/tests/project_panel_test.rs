use egui::Context;
use glam::UVec2;
use lc_core::command::DocKind;
use lc_core::commands::MoveLevelCommand;
use lc_core::id::{LevelId, WorldId};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_shell::project_panel::ProjectPanel;
use lc_shell::ShellApp;
use lc_ui_api::{EditorCtx, EditorEvent, PanelView};

#[test]
fn test_project_panel_create_add_save_reload() {
    let mut shell = ShellApp::new(Project::new("AdventureGame"));

    // Add a world
    let world_id = WorldId(1);
    let mut world = World::new(world_id, "Overworld");

    // Add a level
    let level_id = LevelId(10);
    let mut level = Level::new(level_id, "Forest_1", UVec2::new(64, 48));
    level.world_depth = 1;
    world.levels.push(level);
    shell.project.worlds.push(world);

    // Open level tab (placeholder DocumentView)
    shell.open_document(DocKind::Level(level_id));
    assert!(shell.open_documents.contains_key(&DocKind::Level(level_id)));
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 1);

    // Save project to disk
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let save_path = std::env::temp_dir()
        .join(format!("lc_test_proj_{timestamp}"))
        .join("project.lcproj");

    shell
        .save_project_to_path(&save_path)
        .expect("Project save must succeed");

    // Reload project in a new ShellApp
    let mut reloaded_shell = ShellApp::new(Project::new("Blank"));
    reloaded_shell
        .open_project_from_path(&save_path)
        .expect("Project load must succeed");

    // Verify tree structure is identical
    assert_eq!(reloaded_shell.project.name, shell.project.name);
    assert_eq!(
        reloaded_shell.project.worlds.len(),
        shell.project.worlds.len()
    );

    let reloaded_world = reloaded_shell
        .project
        .world(world_id)
        .expect("World must exist");
    assert_eq!(reloaded_world.name, "Overworld");
    assert_eq!(reloaded_world.levels.len(), 1);

    let reloaded_level = reloaded_world
        .levels
        .iter()
        .find(|l| l.id == level_id)
        .expect("Level must exist");
    assert_eq!(reloaded_level.name, "Forest_1");
    assert_eq!(reloaded_level.world_depth, 1);
    assert_eq!(reloaded_level.size_px, UVec2::new(64, 48));
}

#[test]
fn test_project_panel_double_click_focus_no_duplicate() {
    let mut shell = ShellApp::new(Project::new("TestProject"));
    let world_id = WorldId(1);
    let mut world = World::new(world_id, "Dungeon");

    let level_id = LevelId(5);
    let level = Level::new(level_id, "Room_A", UVec2::new(32, 32));
    world.levels.push(level);
    shell.project.worlds.push(world);

    // First open
    shell.open_document(DocKind::Level(level_id));
    assert_eq!(shell.open_documents.len(), 1);
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 1);

    // Second open (e.g. double click in tree)
    shell.open_document(DocKind::Level(level_id));
    assert_eq!(
        shell.open_documents.len(),
        1,
        "Double click must not duplicate open document"
    );
    assert_eq!(
        shell.dock_state.iter_all_tabs().count(),
        1,
        "DockState must contain only one tab for the document"
    );

    // Open via event queue
    shell
        .events
        .push(EditorEvent::OpenDocument(DocKind::Level(level_id)));
    // Process commands and events
    let egui_ctx = Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        shell.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(
        shell.open_documents.len(),
        1,
        "Document count must remain 1 after repeated OpenDocument events"
    );
    assert_eq!(shell.dock_state.iter_all_tabs().count(), 1);
}

#[test]
fn test_project_panel_drag_level_move_depth_and_undo() {
    let mut shell = ShellApp::new(Project::new("TestProject"));
    let world_id = WorldId(2);
    let mut world = World::new(world_id, "Cave");

    let level_id = LevelId(8);
    let mut level = Level::new(level_id, "Entrance", UVec2::new(32, 32));
    level.world_depth = 0;
    world.levels.push(level);
    shell.project.worlds.push(world);

    // Initial depth is 0
    let level = shell.project.level(level_id).expect("Level exists");
    assert_eq!(level.world_depth, 0);

    // Issue MoveLevelCommand (simulating drag to depth 3)
    let move_cmd = MoveLevelCommand::new(level_id, level.world_pos, 3);
    shell.pending_commands.push(Box::new(move_cmd));

    let egui_ctx = Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        shell.ui(ui);
    });
    output.textures_delta.clear();

    // Verify depth changed to 3
    let level = shell.project.level(level_id).expect("Level exists");
    assert_eq!(level.world_depth, 3);
    assert!(shell.is_dirty());

    // Undo move
    shell.undo();
    let level = shell.project.level(level_id).expect("Level exists");
    assert_eq!(level.world_depth, 0, "Undo must restore original depth 0");

    // Redo move
    shell.redo();
    let level = shell.project.level(level_id).expect("Level exists");
    assert_eq!(level.world_depth, 3, "Redo must re-apply depth 3");
}

#[test]
fn test_project_panel_renders_without_panic() {
    let mut panel = ProjectPanel::new();
    let mut project = Project::new("TestProject");
    let world_id = WorldId(1);
    let mut world = World::new(world_id, "MainWorld");
    let level = Level::new(LevelId(1), "Start", UVec2::new(16, 16));
    world.levels.push(level);
    project.worlds.push(world);

    let undo_stack = lc_core::command::UndoStack::new();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();

    let egui_ctx = Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(&project, &undo_stack, &mut events, &mut pending_commands);
        panel.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert_eq!(panel.id().as_str(), "project");
    assert_eq!(panel.title(), "Project");
}
