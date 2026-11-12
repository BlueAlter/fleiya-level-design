//! Headless harness for the asynchronous file dialog integration.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use lc_core::project::Project;
use lc_shell::shared::file_dialog::{FileDialogHandle, FileDialogOutcome, FileDialogService};
use lc_shell::ShellApp;

/// Dialog backend that returns pre-queued outcomes without touching a desktop.
#[derive(Default)]
struct MockFileDialogService {
    open_outcomes: Mutex<VecDeque<FileDialogOutcome>>,
    save_outcomes: Mutex<VecDeque<FileDialogOutcome>>,
    open_requests: AtomicUsize,
    save_requests: AtomicUsize,
}

impl MockFileDialogService {
    fn queue_open(&self, outcome: FileDialogOutcome) {
        self.open_outcomes
            .lock()
            .expect("open outcome queue mutex must not be poisoned")
            .push_back(outcome);
    }

    fn queue_save(&self, outcome: FileDialogOutcome) {
        self.save_outcomes
            .lock()
            .expect("save outcome queue mutex must not be poisoned")
            .push_back(outcome);
    }

    fn open_requests(&self) -> usize {
        self.open_requests.load(Ordering::SeqCst)
    }

    fn save_requests(&self) -> usize {
        self.save_requests.load(Ordering::SeqCst)
    }
}

impl FileDialogService for MockFileDialogService {
    fn open_project(&self) -> FileDialogHandle {
        self.open_requests.fetch_add(1, Ordering::SeqCst);
        let outcome = self
            .open_outcomes
            .lock()
            .expect("open outcome queue mutex must not be poisoned")
            .pop_front()
            .unwrap_or(FileDialogOutcome::Unavailable);
        FileDialogHandle::ready(outcome)
    }

    fn save_project_as(&self) -> FileDialogHandle {
        self.save_requests.fetch_add(1, Ordering::SeqCst);
        let outcome = self
            .save_outcomes
            .lock()
            .expect("save outcome queue mutex must not be poisoned")
            .pop_front()
            .unwrap_or(FileDialogOutcome::Unavailable);
        FileDialogHandle::ready(outcome)
    }
}

/// Dialog backend that stays pending until the test sends a result.
#[derive(Default)]
struct DelayedFileDialogService {
    open_sender: Mutex<Option<Sender<FileDialogOutcome>>>,
}

impl FileDialogService for DelayedFileDialogService {
    fn open_project(&self) -> FileDialogHandle {
        let (handle, sender) = FileDialogHandle::channel();
        *self
            .open_sender
            .lock()
            .expect("open sender mutex must not be poisoned") = Some(sender);
        handle
    }

    fn save_project_as(&self) -> FileDialogHandle {
        FileDialogHandle::ready(FileDialogOutcome::Unavailable)
    }
}

fn temp_project_path(label: &str) -> PathBuf {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join(format!(
            "lc_shell_dialog_{label}_{}_{unique}",
            std::process::id()
        ))
        .join("project.lcproj")
}

#[test]
fn open_dialog_selection_loads_project() {
    let saved_path = temp_project_path("open");
    lc_io::save_project(&Project::new("DialogLoaded"), &saved_path)
        .expect("fixture project must save");

    let mut app = ShellApp::new(Project::new("Blank"));
    let dialogs = Arc::new(MockFileDialogService::default());
    dialogs.queue_open(FileDialogOutcome::Selected(saved_path.clone()));
    app.set_file_dialog_service(dialogs.clone());

    app.request_open_project_dialog();
    assert_eq!(dialogs.open_requests(), 1);
    assert_eq!(app.status_message, "Select a project file...");
    assert_eq!(app.project.name, "Blank");

    assert!(app.poll_file_dialogs());
    assert_eq!(app.project.name, "DialogLoaded");
    assert_eq!(app.project_path.as_deref(), Some(saved_path.as_path()));
    assert!(app.status_message.contains("Opened project"));
}

#[test]
fn save_as_dialog_writes_project_and_sets_path() {
    let chosen_path = temp_project_path("save");
    let mut app = ShellApp::new(Project::new("SaveMe"));
    app.is_dirty = true;

    let dialogs = Arc::new(MockFileDialogService::default());
    dialogs.queue_save(FileDialogOutcome::Selected(chosen_path.clone()));
    app.set_file_dialog_service(dialogs.clone());

    app.request_save_project_as_dialog();
    assert_eq!(dialogs.save_requests(), 1);
    assert!(app.poll_file_dialogs());

    assert!(chosen_path.exists());
    assert_eq!(app.project_path.as_deref(), Some(chosen_path.as_path()));
    assert!(!app.is_dirty);
    assert!(app.status_message.contains("Project saved"));
    assert_eq!(
        lc_io::load_project(&chosen_path)
            .expect("saved project must load")
            .name,
        "SaveMe"
    );
}

#[test]
fn save_as_adds_missing_lcproj_extension() {
    let chosen_path = temp_project_path("save_extension");
    let directory = chosen_path
        .parent()
        .expect("temp project path must have a parent")
        .to_path_buf();
    let extensionless_path = directory.join("my_game");

    let mut app = ShellApp::new(Project::new("ExtensionProject"));
    let dialogs = Arc::new(MockFileDialogService::default());
    dialogs.queue_save(FileDialogOutcome::Selected(extensionless_path));
    app.set_file_dialog_service(dialogs);

    app.request_save_project_as_dialog();
    assert!(app.poll_file_dialogs());

    let expected = directory.join("my_game.lcproj");
    assert!(expected.exists());
    assert_eq!(app.project_path.as_deref(), Some(expected.as_path()));
}

#[test]
fn cancelled_dialog_keeps_current_project() {
    let mut app = ShellApp::new(Project::new("KeepMe"));
    let dialogs = Arc::new(MockFileDialogService::default());
    dialogs.queue_open(FileDialogOutcome::Cancelled);
    app.set_file_dialog_service(dialogs);

    app.request_open_project_dialog();
    assert!(app.poll_file_dialogs());

    assert_eq!(app.project.name, "KeepMe");
    assert_eq!(app.project_path, None);
    assert_eq!(app.status_message, "File dialog cancelled");
}

#[test]
fn unavailable_dialog_reports_headless_fallback() {
    let mut app = ShellApp::new(Project::new("Headless"));
    let dialogs = Arc::new(MockFileDialogService::default());
    dialogs.queue_save(FileDialogOutcome::Unavailable);
    app.set_file_dialog_service(dialogs);

    app.request_save_project_as_dialog();
    assert!(app.poll_file_dialogs());

    assert_eq!(app.project.name, "Headless");
    assert_eq!(app.project_path, None);
    assert!(app.status_message.contains("headless"));
}

#[test]
fn polling_does_not_block_while_dialog_is_pending() {
    let mut app = ShellApp::new(Project::new("Pending"));
    let dialogs = Arc::new(DelayedFileDialogService::default());
    app.set_file_dialog_service(dialogs.clone());

    app.request_open_project_dialog();
    assert!(!app.poll_file_dialogs());
    assert_eq!(app.status_message, "Select a project file...");

    let sender = dialogs
        .open_sender
        .lock()
        .expect("open sender mutex must not be poisoned")
        .take()
        .expect("dialog request must store a sender");
    sender
        .send(FileDialogOutcome::Cancelled)
        .expect("pending dialog must accept a result");

    assert!(app.poll_file_dialogs());
    assert_eq!(app.status_message, "File dialog cancelled");
}

#[test]
fn duplicate_open_request_does_not_spawn_a_second_dialog() {
    let mut app = ShellApp::new(Project::new("SingleDialog"));
    let dialogs = Arc::new(MockFileDialogService::default());
    app.set_file_dialog_service(dialogs.clone());

    app.request_open_project_dialog();
    app.request_open_project_dialog();

    assert_eq!(dialogs.open_requests(), 1);
    assert_eq!(app.status_message, "Open dialog is already active");
}
