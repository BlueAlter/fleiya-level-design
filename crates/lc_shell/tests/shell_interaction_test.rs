use egui::Context;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::LevelId;
use lc_core::project::Project;
use lc_shell::dock_layout::DockSlotLocation;
use lc_shell::ShellApp;
use lc_ui_api::assets::{AssetError, AssetService};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::view::{DocumentView, PanelId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct TestDocView {
    kind: DocKind,
    valid: Arc<AtomicBool>,
}

impl DocumentView for TestDocView {
    fn title(&self, _p: &Project) -> String {
        format!("{:?}", self.kind)
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _cx: &mut EditorCtx) {}

    fn is_valid(&self, _p: &Project) -> bool {
        self.valid.load(Ordering::SeqCst)
    }

    fn doc_kind(&self) -> DocKind {
        self.kind
    }
}

#[derive(Debug)]
struct FocusCommand {
    label: String,
    target: DocKind,
}

impl Command for FocusCommand {
    fn apply(&mut self, _p: &mut Project) -> Result<(), CoreError> {
        Ok(())
    }

    fn revert(&mut self, _p: &mut Project) {}

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        self.label.clone()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(self.target)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct FailingCommand;

impl Command for FailingCommand {
    fn apply(&mut self, _p: &mut Project) -> Result<(), CoreError> {
        Err(CoreError::DefNotFound(
            "Target entity was deleted".to_string(),
        ))
    }

    fn revert(&mut self, _p: &mut Project) {}

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "FailingCommand".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_command_error_surfacing_in_status_bar() {
    let mut app = ShellApp::new(Project::new("ErrorSurfacingProject"));

    // Push failing command into pending commands queue
    app.pending_commands.push(Box::new(FailingCommand));

    let ctx = Context::default();
    let raw_input = egui::RawInput::default();
    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert!(
        app.status_message
            .starts_with("Error executing FailingCommand"),
        "Status bar must surface command apply error: {}",
        app.status_message
    );
    assert!(
        app.status_message.contains("Target entity was deleted"),
        "Status bar message must contain typed error details: {}",
        app.status_message
    );
}

#[test]
fn test_panel_tab_drag_and_drop_between_docks() {
    let mut app = ShellApp::new(Project::new("DnDTestProject"));
    let console_id = PanelId::new("console");

    // Initially console is in bottom dock
    assert!(app.dock_layout.bottom.panels.contains(&console_id));
    assert!(!app.dock_layout.left.panels.contains(&console_id));

    // Simulate drag start
    app.dock_layout.start_drag(console_id.clone());
    assert_eq!(app.dock_layout.dragging_panel, Some(console_id.clone()));

    // Drop into left dock
    let moved = app.dock_layout.drop_dragged(DockSlotLocation::Left);
    assert!(moved);
    assert_eq!(app.dock_layout.dragging_panel, None);
    assert!(app.dock_layout.left.panels.contains(&console_id));
    assert!(!app.dock_layout.bottom.panels.contains(&console_id));

    // Drag from left to right dock
    app.dock_layout.start_drag(console_id.clone());
    app.dock_layout.drop_dragged(DockSlotLocation::Right);
    assert!(app.dock_layout.right.panels.contains(&console_id));
    assert!(!app.dock_layout.left.panels.contains(&console_id));
}

#[test]
fn test_panel_tab_reorder_and_insert_at_index() {
    let mut app = ShellApp::new(Project::new("DnDIndexTestProject"));
    let project_id = PanelId::new("project");
    let console_id = PanelId::new("console");

    // Insert Project at the front of the bottom dock.
    app.dock_layout.start_drag(project_id.clone());
    assert!(app.dock_layout.drop_dragged_at(DockSlotLocation::Bottom, 0));
    assert_eq!(
        app.dock_layout.bottom.panels,
        vec![project_id.clone(), console_id.clone()]
    );
    assert!(!app.dock_layout.bottom.collapsed);

    // Reorder within the same dock by dropping past the last tab.
    app.dock_layout.start_drag(project_id.clone());
    assert!(app
        .dock_layout
        .drop_dragged_at(DockSlotLocation::Bottom, usize::MAX));
    assert_eq!(app.dock_layout.bottom.panels, vec![console_id, project_id]);
}

#[test]
fn test_escape_cancels_panel_drag() {
    let mut app = ShellApp::new(Project::new("DragEscapeProject"));
    app.dock_layout.start_drag(PanelId::new("console"));
    assert!(app.dock_layout.dragging_panel.is_some());

    let ctx = Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(
        app.dock_layout.dragging_panel, None,
        "Escape must cancel a panel drag"
    );
}

#[test]
fn test_asset_service_typed_errors_and_invalidation() {
    let service = lc_shell::ShellAssetService::new();
    let ctx = Context::default();

    // 1. Empty path -> InvalidPath
    let err_empty = service.load_texture(&ctx, "   ");
    assert!(matches!(err_empty, Err(AssetError::InvalidPath(_))));

    // 2. Non-existent file -> NotFound
    let err_not_found = service.load_texture(&ctx, "textures/missing_tileset.png");
    assert!(matches!(err_not_found, Err(AssetError::NotFound(_))));

    // 3. Invalidation works without panic
    service.invalidate("textures/missing_tileset.png");
    service.invalidate("*");
}

#[test]
fn test_undo_redo_jumps_to_focus_document() {
    let mut app = ShellApp::new(Project::new("UndoFocusProject"));
    let valid = Arc::new(AtomicBool::new(true));
    let valid_c1 = valid.clone();

    app.registry_mut().register_document_factory(move |kind| {
        Some(Box::new(TestDocView {
            kind,
            valid: valid_c1.clone(),
        }))
    });

    // Execute command focusing Level(1)
    let cmd = Box::new(FocusCommand {
        label: "Edit Level 1".to_string(),
        target: DocKind::Level(LevelId(1)),
    });
    let _ = app.undo_stack.execute(cmd, &mut app.project);

    // Switch focus to ProjectSettings
    app.open_document(DocKind::ProjectSettings);
    app.session.active_document = Some(DocKind::ProjectSettings);
    assert_eq!(app.session.active_document, Some(DocKind::ProjectSettings));

    // Trigger undo -> must jump back to Level(1)
    app.undo();
    assert_eq!(
        app.session.active_document,
        Some(DocKind::Level(LevelId(1)))
    );
    assert_eq!(app.session.active_level, Some(LevelId(1)));

    // Switch focus back to ProjectSettings
    app.open_document(DocKind::ProjectSettings);
    app.session.active_document = Some(DocKind::ProjectSettings);

    // Trigger redo -> must jump back to Level(1)
    app.redo();
    assert_eq!(
        app.session.active_document,
        Some(DocKind::Level(LevelId(1)))
    );
    assert_eq!(app.session.active_level, Some(LevelId(1)));
}

#[test]
fn test_active_level_cleared_when_closing_tab_or_switching() {
    let mut app = ShellApp::new(Project::new("ActiveLevelProject"));
    let valid = Arc::new(AtomicBool::new(true));
    let valid_c = valid.clone();

    app.registry_mut().register_document_factory(move |kind| {
        Some(Box::new(TestDocView {
            kind,
            valid: valid_c.clone(),
        }))
    });

    let level_doc = DocKind::Level(LevelId(42));
    app.open_document(level_doc);
    app.session.active_document = Some(level_doc);
    app.session.active_level = Some(LevelId(42));

    // Close the level tab
    app.close_document(&level_doc);
    assert_eq!(app.session.active_document, None);
    assert_eq!(app.session.active_level, None);
}

#[test]
fn test_drag_cancelled_when_released_outside_dock() {
    let mut app = ShellApp::new(Project::new("DragCancelProject"));
    let console_id = PanelId::new("console");

    app.dock_layout.start_drag(console_id);
    assert!(app.dock_layout.dragging_panel.is_some());

    let ctx = Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.events.push(egui::Event::PointerButton {
        pos: egui::pos2(500.0, 500.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(
        app.dock_layout.dragging_panel, None,
        "Releasing pointer outside docks must cancel dragging"
    );
}
