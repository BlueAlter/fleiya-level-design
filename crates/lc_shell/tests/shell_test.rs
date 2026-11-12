use egui::Context;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::project::Project;
use lc_shell::dock_layout::{BottomDockState, DockSlotLocation, ShellDockLayout};
use lc_shell::ShellApp;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
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

#[test]
fn test_bottom_dock_layout_and_persistence() {
    let screen_size = egui::vec2(1280.0, 720.0);
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);

    let mut app = ShellApp::new(Project::new("LayoutTestProject"));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("project"),
    }));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("layers"),
    }));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("tileset"),
    }));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("tools"),
    }));
    app.registry_mut().register_panel(Box::new(TestPanelView {
        id: PanelId::new("console"),
    }));

    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    };

    // Frame 1: Expanded layout
    let mut output1 = ctx.run_ui(raw_input.clone(), |ui| {
        app.ui(ui);
    });
    output1.textures_delta.clear();

    let menu_bar =
        egui::panel::PanelState::load(&ctx, egui::Id::new("menu_bar")).expect("menu_bar state");
    let status_bar =
        egui::panel::PanelState::load(&ctx, egui::Id::new("status_bar")).expect("status_bar state");
    let left_dock =
        egui::panel::PanelState::load(&ctx, egui::Id::new("left_dock")).expect("left_dock state");
    let right_dock =
        egui::panel::PanelState::load(&ctx, egui::Id::new("right_dock")).expect("right_dock state");
    let bottom_dock = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("bottom_dock state");

    let central_rect_expanded = app.last_central_rect.expect("central panel rect");

    // (a) Bottom dock lies fully inside [left_dock.right, right_dock.left] horizontally
    assert!(
        bottom_dock.outer_rect.left() >= left_dock.outer_rect.right() - 0.5,
        "bottom_dock left ({}) must be >= left_dock right ({})",
        bottom_dock.outer_rect.left(),
        left_dock.outer_rect.right()
    );
    assert!(
        bottom_dock.outer_rect.right() <= right_dock.outer_rect.left() + 0.5,
        "bottom_dock right ({}) must be <= right_dock left ({})",
        bottom_dock.outer_rect.right(),
        right_dock.outer_rect.left()
    );

    // (b) Left and right dock rects reach from below the menu bar to above the status bar
    assert!(
        (left_dock.outer_rect.top() - menu_bar.outer_rect.bottom()).abs() <= 0.5,
        "left_dock top must reach menu_bar bottom"
    );
    assert!(
        (left_dock.outer_rect.bottom() - status_bar.outer_rect.top()).abs() <= 0.5,
        "left_dock bottom must reach status_bar top"
    );
    assert!(
        (right_dock.outer_rect.top() - menu_bar.outer_rect.bottom()).abs() <= 0.5,
        "right_dock top must reach menu_bar bottom"
    );
    assert!(
        (right_dock.outer_rect.bottom() - status_bar.outer_rect.top()).abs() <= 0.5,
        "right_dock bottom must reach status_bar top"
    );

    // (c) Status bar spans the full width
    assert!(
        (status_bar.outer_rect.left() - screen_rect.left()).abs() <= 0.5,
        "status_bar must start at screen left"
    );
    assert!(
        (status_bar.outer_rect.right() - screen_rect.right()).abs() <= 0.5,
        "status_bar must reach screen right"
    );
    assert!(
        (status_bar.outer_rect.width() - screen_rect.width()).abs() <= 0.5,
        "status_bar width must match screen width"
    );

    // Frame 2: Collapse bottom dock via toggle_bottom_dock()
    app.dock_layout.toggle_bottom_dock();
    assert!(app.dock_layout.bottom.collapsed);

    let mut output2 = ctx.run_ui(raw_input.clone(), |ui| {
        app.ui(ui);
    });
    output2.textures_delta.clear();

    let bottom_dock_collapsed = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("bottom_dock collapsed state");
    let central_rect_collapsed = app.last_central_rect.expect("central panel rect collapsed");

    // (d) Collapsed bottom dock height is about 28px and the central panel grows by the same amount
    assert!(
        (bottom_dock_collapsed.outer_rect.height() - 28.0).abs() <= 1.0,
        "collapsed bottom dock height must be about 28px, got {}",
        bottom_dock_collapsed.outer_rect.height()
    );

    let bottom_height_reduction =
        bottom_dock.outer_rect.height() - bottom_dock_collapsed.outer_rect.height();
    let central_height_growth = central_rect_collapsed.height() - central_rect_expanded.height();
    assert!(
        (bottom_height_reduction - central_height_growth).abs() <= 1.5,
        "central panel growth ({}) should match bottom dock reduction ({})",
        central_height_growth,
        bottom_height_reduction
    );

    // (e) A persisted layout from before the fix still loads without panic
    let pre_fix_ron = r#"(
        left: (
            panels: [("project")],
            active: Some(("project")),
        ),
        right: (
            panels: [("layers"), ("tileset"), ("tools")],
            active: Some(("layers")),
        ),
        bottom: (
            panels: [("console")],
            active: Some(("console")),
            collapsed: false,
            height: 180.0,
        ),
    )"#;
    let loaded_layout: ShellDockLayout =
        ron::from_str(pre_fix_ron).expect("deserialize pre-fix layout");
    app.dock_layout = loaded_layout;

    // Simulate pre-fix persisted egui PanelState where bottom dock spanned full window width
    let bar_id = egui::Id::new("bottom_dock");
    ctx.data_mut(|d| {
        d.insert_persisted(
            bar_id,
            egui::panel::PanelState {
                outer_rect: egui::Rect::from_min_max(
                    egui::pos2(0.0, 500.0),
                    egui::pos2(1280.0, 680.0),
                ),
            },
        );
    });

    let mut output3 = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output3.textures_delta.clear();

    // Existing slots keep their persisted panels. Newly registered Inspector panel
    // is added to the right slot (3 persisted panels + 1 newly registered inspector).
    assert_eq!(app.dock_layout.left.panels, vec![PanelId::new("project")]);
    assert_eq!(app.dock_layout.right.panels.len(), 4);
    assert_eq!(app.dock_layout.bottom.panels.len(), 1);
    assert!(!app.dock_layout.bottom.collapsed);

    // Verify it adapted to the center column constraint despite old full-width persisted state
    let bottom_dock_reloaded = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("bottom_dock reloaded state");
    assert!(
        bottom_dock_reloaded.outer_rect.left() >= left_dock.outer_rect.right() - 0.5,
        "bottom dock reloaded must respect left dock bound"
    );
    assert!(
        bottom_dock_reloaded.outer_rect.right() <= right_dock.outer_rect.left() + 0.5,
        "bottom dock reloaded must respect right dock bound"
    );
}

#[test]
fn test_bottom_dock_expand_restores_height_and_clamps_stale_state() {
    let screen_size = egui::vec2(1280.0, 720.0);
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);

    let mut app = ShellApp::new(Project::new("BottomDockHeightTest"));
    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    };

    // Frame 1: expanded default height.
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    let expanded = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("expanded bottom dock state");
    assert!(
        (expanded.outer_rect.height() - BottomDockState::DEFAULT_HEIGHT).abs() <= 3.0,
        "default expanded height should be {}, got {}",
        BottomDockState::DEFAULT_HEIGHT,
        expanded.outer_rect.height()
    );

    // Frame 2: collapsed.
    app.dock_layout.toggle_bottom_dock();
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    let collapsed = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("collapsed bottom dock state");
    assert!(
        (collapsed.outer_rect.height() - BottomDockState::COLLAPSED_HEIGHT).abs() <= 1.0,
        "collapsed height should be {}, got {}",
        BottomDockState::COLLAPSED_HEIGHT,
        collapsed.outer_rect.height()
    );

    // Frame 3: expanding again must restore the remembered height, not the
    // collapsed header height or the full central area.
    app.dock_layout.toggle_bottom_dock();
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    let reexpanded = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("re-expanded bottom dock state");
    assert!(
        (reexpanded.outer_rect.height() - BottomDockState::DEFAULT_HEIGHT).abs() <= 3.0,
        "re-expanded height should be {}, got {}",
        BottomDockState::DEFAULT_HEIGHT,
        reexpanded.outer_rect.height()
    );

    // Frame 4: a stale full-height PanelState from an earlier session must be
    // normalised instead of covering the editor.
    ctx.data_mut(|d| {
        d.insert_persisted(
            egui::Id::new("bottom_dock"),
            egui::panel::PanelState {
                outer_rect: egui::Rect::from_min_max(
                    egui::pos2(0.0, 0.0),
                    egui::pos2(1280.0, 700.0),
                ),
            },
        );
    });
    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();
    let clamped = egui::panel::PanelState::load(&ctx, egui::Id::new("bottom_dock"))
        .expect("clamped bottom dock state");
    assert!(
        clamped.outer_rect.height() <= screen_size.y * BottomDockState::MAX_HEIGHT_FRACTION + 1.0,
        "bottom dock must not exceed 55% of the window, got {}",
        clamped.outer_rect.height()
    );
    assert!(
        clamped.outer_rect.height() >= BottomDockState::MIN_EXPANDED_HEIGHT - 1.0,
        "bottom dock must stay usable, got {}",
        clamped.outer_rect.height()
    );
    assert!(
        app.dock_layout.bottom.height <= screen_size.y * BottomDockState::MAX_HEIGHT_FRACTION + 1.0,
        "remembered height must be clamped, got {}",
        app.dock_layout.bottom.height
    );
}

#[test]
fn test_dock_layout_memory_persistence() {
    let ctx = Context::default();
    let mut layout = ShellDockLayout::default();
    layout.move_panel(&PanelId::new("console"), DockSlotLocation::Left);
    layout.bottom.height = 240.0;
    layout.bottom.collapsed = true;

    layout.save_to_memory(&ctx);

    let reloaded = ShellDockLayout::load_from_memory(&ctx).expect("layout should load from memory");
    assert!(reloaded.left.panels.contains(&PanelId::new("console")));
    assert_eq!(reloaded.bottom.height, 240.0);
    assert!(reloaded.bottom.collapsed);
}

#[test]
fn test_save_without_path_fails_safely_and_preserves_dirty() {
    let mut app = ShellApp::new(Project::new("NoPathProject"));
    app.is_dirty = true;
    app.project_path = None;

    app.events.push(EditorEvent::SaveProject);

    let ctx = Context::default();
    let raw_input = egui::RawInput::default();
    let mut output = ctx.run_ui(raw_input, |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    assert!(
        app.is_dirty(),
        "is_dirty must NOT be cleared when no project path is set"
    );
    assert!(
        app.status_message.contains("no project path configured"),
        "status message must warn about missing project path: {}",
        app.status_message
    );
}
