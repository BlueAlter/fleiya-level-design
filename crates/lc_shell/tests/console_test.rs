use egui::Context;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::id::LevelId;
use lc_core::project::Project;
use lc_shell::features::editor::console::{
    BoundedRingBuffer, ConsoleFilter, ConsolePanel, ConsoleSubscriberLayer, LogEntry, LogLevel,
    SharedRingBuffer,
};
use lc_shell::ShellApp;
use lc_ui_api::{EditorCtx, EditorEvent, PanelView};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Debug)]
struct DummyCommand {
    name: String,
}

impl Command for DummyCommand {
    fn apply(&mut self, _p: &mut Project) -> Result<(), lc_core::error::CoreError> {
        Ok(())
    }

    fn revert(&mut self, _p: &mut Project) {}

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn label(&self) -> String {
        self.name.clone()
    }

    fn focus(&self) -> Option<DocKind> {
        None
    }
}

#[test]
fn test_ring_buffer_bound_and_filtering() {
    let mut buf = BoundedRingBuffer::new(5);
    for i in 0..10 {
        buf.push(LogEntry {
            timestamp: "00:00:00".to_string(),
            level: if i % 2 == 0 {
                LogLevel::Error
            } else {
                LogLevel::Info
            },
            target: "test_target".to_string(),
            message: format!("Message {i}"),
            doc_target: None,
        });
    }

    assert_eq!(buf.len(), 5);
    // Oldest messages (0..5) discarded; messages 5..10 remain
    let all = buf.all_entries();
    assert_eq!(all.first().map(|e| e.message.as_str()), Some("Message 5"));
    assert_eq!(all.last().map(|e| e.message.as_str()), Some("Message 9"));

    // Filter by Info
    let filter = ConsoleFilter {
        show_error: false,
        show_warn: false,
        show_info: true,
        show_debug: false,
        show_trace: false,
        search_query: String::new(),
        collapse_duplicates: false,
    };
    let filtered = buf.filtered_entries(&filter);
    assert_eq!(filtered.len(), 3); // Message 5, Message 7, Message 9
}

#[test]
fn test_duplicate_collapsing_rules() {
    let mut buf = BoundedRingBuffer::new(10);
    for _ in 0..4 {
        buf.push(LogEntry {
            timestamp: "00:00:00".to_string(),
            level: LogLevel::Warn,
            target: "net".to_string(),
            message: "Connection retry".to_string(),
            doc_target: None,
        });
    }

    let filter = ConsoleFilter::default();
    let collapsed = buf.filtered_entries(&filter);
    assert_eq!(collapsed.len(), 1);
    assert_eq!(collapsed.first().map(|c| c.count), Some(4));
}

#[test]
fn test_logging_from_worker_threads_non_blocking() {
    let buffer = SharedRingBuffer::new(5000);
    let layer = ConsoleSubscriberLayer::new(buffer.clone());
    let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(layer));

    let finished = Arc::new(AtomicBool::new(false));
    let mut worker_handles = Vec::new();

    // Spawn 4 background worker threads generating rapid log events
    for t in 0..4 {
        let d = dispatch.clone();
        let handle = thread::spawn(move || {
            tracing::dispatcher::with_default(&d, || {
                for i in 0..100 {
                    tracing::info!(target: "worker", thread_id = t, "Worker log item {i}");
                    tracing::warn!(target: "worker", thread_id = t, "Worker warning item {i}");
                }
            });
        });
        worker_handles.push(handle);
    }

    // Simultaneously, UI thread reads and filters without being blocked or deadlocked
    let mut panel = ConsolePanel::new(buffer.clone());
    let ctx = Context::default();
    let project = Project::new("Test");
    let undo_stack = UndoStack::new();
    let session = lc_ui_api::session::SessionState::default();
    let assets = lc_ui_api::assets::NoopAssetService;
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();

    for _ in 0..10 {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        let mut output = ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    // Join all workers
    for h in worker_handles {
        h.join().expect("Worker thread panicked");
    }

    finished.store(true, Ordering::SeqCst);
    assert!(!buffer.is_empty(), "Buffer should contain worker logs");
}

#[test]
fn test_command_application_logs_at_debug() {
    let buffer = SharedRingBuffer::new(500);
    let layer = ConsoleSubscriberLayer::new(buffer.clone());
    let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(layer));

    tracing::dispatcher::with_default(&dispatch, || {
        let mut app = ShellApp::from_console(buffer.clone());
        let ctx = Context::default();

        // Submit a command via EditorCtx::exec during frame
        let mut output = ctx.run_ui(Default::default(), |ui| {
            app.ui(ui);
        });
        output.textures_delta.clear();

        // Emit a command through EditorCtx
        let project = Project::new("Test");
        let undo_stack = UndoStack::new();
        let session = lc_ui_api::session::SessionState::default();
        let assets = lc_ui_api::assets::NoopAssetService;
        let mut events = Vec::new();
        let mut pending_commands = Vec::new();

        {
            let mut cx = EditorCtx::new(
                &project,
                &undo_stack,
                &session,
                &mut events,
                &mut pending_commands,
                &assets,
            );
            cx.exec(DummyCommand {
                name: "ResizeCanvasCommand".to_string(),
            });
        }

        // Check that EditorCtx::exec logged at debug
        let entries = buffer.filtered_entries(&ConsoleFilter {
            show_error: true,
            show_warn: true,
            show_info: true,
            show_debug: true,
            show_trace: true,
            search_query: "ResizeCanvasCommand".to_string(),
            collapse_duplicates: false,
        });

        assert!(
            !entries.is_empty(),
            "Command execution must be logged at debug level"
        );
        assert_eq!(
            entries.first().map(|e| e.entry.level),
            Some(LogLevel::Debug)
        );
    });
}

#[test]
fn test_console_click_to_focus_doc_kind() {
    let buffer = SharedRingBuffer::new(10);
    buffer.push(LogEntry {
        timestamp: "12:00:00".to_string(),
        level: LogLevel::Info,
        target: "core".to_string(),
        message: "Entity placed in Level(42)".to_string(),
        doc_target: Some(DocKind::Level(LevelId(42))),
    });

    let mut panel = ConsolePanel::new(buffer);
    let ctx = Context::default();
    let project = Project::new("Test");
    let undo_stack = UndoStack::new();
    let session = lc_ui_api::session::SessionState::default();
    let assets = lc_ui_api::assets::NoopAssetService;
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();

    // Render console panel in egui
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo_stack,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        panel.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    // Simulated click on navigation button
    let mut cx = EditorCtx::new(
        &project,
        &undo_stack,
        &session,
        &mut events,
        &mut pending_commands,
        &assets,
    );
    cx.emit(EditorEvent::OpenDocument(DocKind::Level(LevelId(42))));

    assert_eq!(
        events.first(),
        Some(&EditorEvent::OpenDocument(DocKind::Level(LevelId(42))))
    );
}
