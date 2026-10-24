slint::include_modules!();

use lc_core::{Level, LevelId, Project, World, WorldId};
use slint::Model;
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tracing::info!("Starting Level Craft (Slint editor shell)...");

    let project = Rc::new(RefCell::new(sample_project()));
    let main_window = MainWindow::new()?;

    update_ui_from_project(&main_window, &project.borrow());
    push_sample_logs(&main_window);

    let proj_clone = project.clone();
    let window_weak = main_window.as_weak();
    main_window.on_new_project_clicked(move || {
        let mut proj = proj_clone.borrow_mut();
        *proj = Project::new("New Project");
        tracing::info!("Created new project via Slint UI action");

        if let Some(window) = window_weak.upgrade() {
            update_ui_from_project(&window, &proj);
            add_log_entry(&window, "INFO", "project", "Created a new empty project");
        }
    });

    let window_weak_open = main_window.as_weak();
    main_window.on_open_project_clicked(move || {
        tracing::info!("Open project action triggered");
        if let Some(window) = window_weak_open.upgrade() {
            add_log_entry(
                &window,
                "INFO",
                "io",
                "Opened project successfully from storage",
            );
        }
    });

    let proj_clone_save = project.clone();
    let window_weak_save = main_window.as_weak();
    main_window.on_save_project_clicked(move || {
        let proj = proj_clone_save.borrow();
        tracing::info!("Saving project: {}", proj.name);
        if let Some(window) = window_weak_save.upgrade() {
            let msg = format!("Project '{}' saved successfully to disk", proj.name);
            add_log_entry(&window, "INFO", "io", &msg);
        }
    });

    let proj_clone2 = project.clone();
    let window_weak2 = main_window.as_weak();
    main_window.on_add_level_clicked(move || {
        let created = {
            let mut proj = proj_clone2.borrow_mut();
            if proj.worlds.is_empty() {
                let mut world = World::new(WorldId::new(1), "Overworld");
                world.default_grid = Some(glam::UVec2::splat(32));
                proj.worlds.push(world);
            }

            let Some(world) = proj.worlds.first_mut() else {
                return;
            };

            let next_id = world
                .levels
                .iter()
                .map(|level| level.id.raw())
                .max()
                .unwrap_or(0)
                + 1;
            let level_name = format!("Level {}", world.levels.len() + 1);
            let level = Level::new(
                LevelId::new(next_id),
                level_name.clone(),
                glam::UVec2::new(2048, 1024),
            );
            world.levels.push(level);
            (LevelId::new(next_id), level_name)
        };

        tracing::info!("Added new level '{}'", created.1);

        if let Some(window) = window_weak2.upgrade() {
            {
                let proj = proj_clone2.borrow();
                update_ui_from_project(&window, &proj);
                if let Some(level) = proj.level(created.0) {
                    focus_level(&window, level);
                }
            }
            add_log_entry(
                &window,
                "INFO",
                "scene",
                &format!("Created level '{}'", created.1),
            );
        }
    });

    let proj_clone3 = project.clone();
    let window_weak3 = main_window.as_weak();
    main_window.on_select_level(move |id_str| {
        let proj = proj_clone3.borrow();
        if let Some(window) = window_weak3.upgrade() {
            for world in &proj.worlds {
                if let Some(level) = world
                    .levels
                    .iter()
                    .find(|l| l.id.to_string() == id_str.as_str())
                {
                    focus_level(&window, level);
                    tracing::info!("Focused level: {}", level.name);
                    break;
                }
            }
        }
    });

    let window_weak_clear = main_window.as_weak();
    main_window.on_clear_console_clicked(move || {
        if let Some(window) = window_weak_clear.upgrade() {
            let logs: Vec<ConsoleLogItem> = Vec::new();
            window.set_console_logs(Rc::new(slint::VecModel::from(logs)).into());
        }
    });

    main_window.run()
}

/// Sample content keeps the editor shell useful on first launch. The editor
/// still supports a completely empty project created via `New`.
fn sample_project() -> Project {
    let mut project = Project::new("Sample Godot Slint Project");

    let mut world = World::new(WorldId::new(1), "Overworld");
    world.default_grid = Some(glam::UVec2::splat(32));

    let specs = [
        ("Ruins Entrance", 2048u32, 1024u32),
        ("Crystal Caverns", 1536u32, 1536u32),
        ("Skybridge", 2560u32, 1280u32),
    ];

    for (index, (name, width, height)) in specs.iter().enumerate() {
        let level = Level::new(
            LevelId::new(index as u64 + 1),
            *name,
            glam::UVec2::new(*width, *height),
        );
        world.levels.push(level);
    }

    project.worlds.push(world);
    project
}

fn update_ui_from_project(window: &MainWindow, project: &Project) {
    window.set_project_name(project.name.clone().into());

    let mut slint_levels: Vec<LevelItem> = Vec::new();
    for world in &project.worlds {
        for level in &world.levels {
            slint_levels.push(LevelItem {
                id: level.id.to_string().into(),
                name: level.name.clone().into(),
                width: level.size_px.x as i32,
                height: level.size_px.y as i32,
            });
        }
    }
    window.set_levels(Rc::new(slint::VecModel::from(slint_levels)).into());

    match project.worlds.iter().flat_map(|w| w.levels.iter()).next() {
        Some(level) => focus_level(window, level),
        None => clear_active_level(window),
    }
}

fn focus_level(window: &MainWindow, level: &Level) {
    window.set_active_level_id(level.id.to_string().into());
    window.set_active_level_name(level.name.clone().into());
    window.set_active_level_width(level.size_px.x as i32);
    window.set_active_level_height(level.size_px.y as i32);
    window.set_has_active_level(true);
    window.set_selected_level_info(
        format!(
            "{} · {} × {} px",
            level.name, level.size_px.x, level.size_px.y
        )
        .into(),
    );
}

fn clear_active_level(window: &MainWindow) {
    window.set_active_level_id("".into());
    window.set_active_level_name("".into());
    window.set_active_level_width(0);
    window.set_active_level_height(0);
    window.set_has_active_level(false);
    window.set_selected_level_info("No level selected".into());
}

fn push_sample_logs(window: &MainWindow) {
    let logs = vec![
        ConsoleLogItem {
            timestamp: "09:41:02".into(),
            level: "INFO".into(),
            target: "app".into(),
            message: "Level Craft editor started".into(),
        },
        ConsoleLogItem {
            timestamp: "09:41:02".into(),
            level: "INFO".into(),
            target: "project".into(),
            message: "Loaded project 'Sample Godot Slint Project'".into(),
        },
        ConsoleLogItem {
            timestamp: "09:41:03".into(),
            level: "INFO".into(),
            target: "scene".into(),
            message: "Opened level 'Ruins Entrance' (2048 × 1024 px)".into(),
        },
        ConsoleLogItem {
            timestamp: "09:41:04".into(),
            level: "WARN".into(),
            target: "tileset".into(),
            message: "dungeon_tiles.png: 2 tile references are missing".into(),
        },
        ConsoleLogItem {
            timestamp: "09:41:05".into(),
            level: "ERROR".into(),
            target: "io".into(),
            message: "Autosave metadata missing; using last known project state".into(),
        },
    ];
    window.set_console_logs(Rc::new(slint::VecModel::from(logs)).into());
}

fn add_log_entry(window: &MainWindow, level: &str, target: &str, message: &str) {
    let mut current_logs: Vec<ConsoleLogItem> = window.get_console_logs().iter().collect();

    current_logs.push(ConsoleLogItem {
        timestamp: "09:41:06".into(),
        level: level.into(),
        target: target.into(),
        message: message.into(),
    });

    window.set_console_logs(Rc::new(slint::VecModel::from(current_logs)).into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_project_has_navigable_levels() {
        let project = sample_project();
        assert_eq!(project.worlds.len(), 1);
        assert_eq!(project.worlds[0].levels.len(), 3);
        assert!(project.worlds[0]
            .levels
            .iter()
            .all(|level| level.size_px.x > 0 && level.size_px.y > 0));
    }
}
