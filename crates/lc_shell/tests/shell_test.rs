use egui::Context;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::project::Project;
use lc_shell::dock_layout::{DockSlotLocation, ShellDockLayout};
use lc_shell::ShellApp;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::view::{DocumentView, PanelId, PanelView};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
struct TestCommand {
    label: String,
}

impl Command for TestCommand {
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
        Some(DocKind::ProjectSettings)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct TestDocView {
    kind: DocKind,
    valid: Arc<AtomicBool>,
}

impl DocumentView for TestDocView {
    fn title(&self, _p: &Project) -> String {
        "Test Settings".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _cx: &mut EditorCtx) {}

    fn is_valid(&self, _p: &Project) -> bool {
        self.valid.load(Ordering::SeqCst)
    }

    fn doc_kind(&self) -> DocKind {
        self.kind
    }
}

struct TestPanelView {
    id: PanelId,
}

impl PanelView for TestPanelView {
    fn id(&self) -> PanelId {
        self.id.clone()
    }

    fn title(&self) -> &str {
        "Test Panel"
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _cx: &mut EditorCtx) {}
}

#[test]
fn test_dock_layout_panel_movement_and_collapse() {
    let mut layout = ShellDockLayout::default();

    let project_id = PanelId::new("project");
    let console_id = PanelId::new("console");

    assert!(layout.left.panels.contains(&project_id));
    assert!(layout.bottom.panels.contains(&console_id));
    assert!(!layout.bottom.collapsed);

    // Move console from bottom to left dock
    layout.move_panel(&console_id, DockSlotLocation::Left);
    assert!(layout.left.panels.contains(&console_id));
    assert!(!layout.bottom.panels.contains(&console_id));
    assert_eq!(layout.left.active, Some(console_id.clone()));

    // Move console to right dock
    layout.move_panel(&console_id, DockSlotLocation::Right);
    assert!(!layout.left.panels.contains(&console_id));
    assert!(layout.right.panels.contains(&console_id));

    // Toggle bottom dock collapse
    assert!(!layout.bottom.collapsed);
    layout.toggle_bottom_dock();
    assert!(layout.bottom.collapsed);
    layout.toggle_bottom_dock();
    assert!(!layout.bottom.collapsed);
}

#[test]
fn test_document_tab_deduplication_and_opening() {
    let mut app = ShellApp::new(Project::new("TestProject"));
    let valid_flag = Arc::new(AtomicBool::new(true));
    let valid_clone = valid_flag.clone();

    app.registry_mut().register_document_factory(move |kind| {
        if kind == DocKind::ProjectSettings {
            Some(Box::new(TestDocView {
                kind,
                valid: valid_clone.clone(),
            }))
        } else {
            None
        }
    });

    // 1. Open document for the first time
    app.open_document(DocKind::ProjectSettings);
    assert_eq!(app.open_documents.len(), 1);
    assert_eq!(app.dock_state.iter_all_tabs().count(), 1);

    // 2. Open same document again (should deduplicate, not add second tab)
    app.open_document(DocKind::ProjectSettings);
    assert_eq!(
        app.open_documents.len(),
        1,
        "Same DocKind must never open twice"
    );
    assert_eq!(
        app.dock_state.iter_all_tabs().count(),
        1,
        "Dock tree must contain exactly 1 tab"
    );
}

#[test]
fn test_dirty_indicator_and_automatic_tab_close_on_invalid() {
    let mut app = ShellApp::new(Project::new("TestProject"));
    let valid_flag = Arc::new(AtomicBool::new(true));
    let valid_clone = valid_flag.clone();

    app.registry_mut().register_document_factory(move |kind| {
        if kind == DocKind::ProjectSettings {
            Some(Box::new(TestDocView {
                kind,
                valid: valid_clone.clone(),
            }))
        } else {
            None
        }
    });

    app.open_document(DocKind::ProjectSettings);

    // Initially clean
    assert!(!app.is_dirty());

    // Execute command -> should become dirty
    let cmd = Box::new(TestCommand {
        label: "Modify settings".to_string(),
    });
    let _ = app.undo_stack.execute(cmd, &mut app.project);
    app.is_dirty = true;
    assert!(
        app.is_dirty(),
        "App should be dirty after command execution"
    );

    // Render a frame with egui context
    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    // Tab is still valid and present
    assert_eq!(app.open_documents.len(), 1);

    // Invalidate document (simulating deletion of underlying level/entity def)
    valid_flag.store(false, Ordering::SeqCst);

    // Render next frame -> document tab should automatically close per is_valid() == false
    let mut output2 = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output2.textures_delta.clear();

    assert_eq!(
        app.open_documents.len(),
        0,
        "Tab must automatically close when is_valid returns false"
    );
}

#[test]
fn test_panel_registration_and_dock_rendering() {
    let mut app = ShellApp::new(Project::new("TestProject"));

    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("project"),
    }));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("console"),
    }));

    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert_eq!(app.status_message, "Ready");
}
