//! Asynchronous native file dialogs for opening and saving projects.
//!
//! Dialogs run on a worker thread so the egui frame loop never blocks. Results are
//! delivered through a [`FileDialogHandle`] which the shell polls once per frame.
//!
//! Headless environments (no `DISPLAY`/`WAYLAND_DISPLAY` on Linux and BSDs) report
//! [`FileDialogOutcome::Unavailable`] immediately instead of trying to show a native
//! dialog. Set `LC_DISABLE_FILE_DIALOGS=1` to force that fallback on a desktop.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// Result produced by a file dialog request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDialogOutcome {
    /// The user picked a path.
    Selected(PathBuf),
    /// The user dismissed the dialog without picking a path.
    Cancelled,
    /// No native dialog could be shown in this environment.
    Unavailable,
}

/// Handle to an in-flight file dialog request.
pub struct FileDialogHandle {
    receiver: Receiver<FileDialogOutcome>,
}

impl FileDialogHandle {
    fn from_receiver(receiver: Receiver<FileDialogOutcome>) -> Self {
        Self { receiver }
    }

    /// Creates a handle that reports `outcome` on the next poll.
    pub fn ready(outcome: FileDialogOutcome) -> Self {
        let (sender, receiver) = mpsc::channel();
        let _ = sender.send(outcome);
        Self::from_receiver(receiver)
    }

    /// Creates a handle together with the sender used to complete it later.
    pub fn channel() -> (Self, Sender<FileDialogOutcome>) {
        let (sender, receiver) = mpsc::channel();
        (Self::from_receiver(receiver), sender)
    }

    /// Returns the outcome once the dialog finishes, or `None` while it is pending.
    pub fn try_take(&self) -> Option<FileDialogOutcome> {
        match self.receiver.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                tracing::warn!("File dialog worker exited without a result");
                Some(FileDialogOutcome::Unavailable)
            }
        }
    }
}

/// Backend that can request native open and "Save As" dialogs.
pub trait FileDialogService: Send + Sync {
    /// Requests a project file to open.
    fn open_project(&self) -> FileDialogHandle;

    /// Requests a destination file for "Save As".
    fn save_project_as(&self) -> FileDialogHandle;
}

/// Native `rfd`-backed file dialog service.
pub struct NativeFileDialogService {
    available: bool,
}

impl NativeFileDialogService {
    /// Creates a service that uses native dialogs when a desktop session exists.
    pub fn new() -> Self {
        Self {
            available: desktop_available(),
        }
    }

    /// Creates a service with an explicit availability flag (used by tests).
    pub fn with_availability(available: bool) -> Self {
        Self { available }
    }

    fn spawn(&self, request: DialogRequest) -> FileDialogHandle {
        if !self.available {
            return FileDialogHandle::ready(FileDialogOutcome::Unavailable);
        }

        let (sender, receiver) = mpsc::channel();
        let spawn = std::thread::Builder::new()
            .name("lc-file-dialog".to_string())
            .spawn(move || {
                let outcome = run_dialog(request);
                if sender.send(outcome).is_err() {
                    tracing::warn!("File dialog result receiver was dropped");
                }
            });

        match spawn {
            Ok(_) => FileDialogHandle::from_receiver(receiver),
            Err(error) => {
                tracing::error!(%error, "Failed to spawn file dialog thread");
                FileDialogHandle::ready(FileDialogOutcome::Unavailable)
            }
        }
    }
}

impl Default for NativeFileDialogService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDialogService for NativeFileDialogService {
    fn open_project(&self) -> FileDialogHandle {
        self.spawn(DialogRequest::OpenProject)
    }

    fn save_project_as(&self) -> FileDialogHandle {
        self.spawn(DialogRequest::SaveProjectAs)
    }
}

#[derive(Debug, Clone, Copy)]
enum DialogRequest {
    OpenProject,
    SaveProjectAs,
}

fn run_dialog(request: DialogRequest) -> FileDialogOutcome {
    let dialog = rfd::FileDialog::new().add_filter("Fleiya Level Design Project", &["lcproj"]);
    let picked = match request {
        DialogRequest::OpenProject => dialog
            .set_title("Open Fleiya Level Design Project")
            .pick_file(),
        DialogRequest::SaveProjectAs => dialog
            .set_title("Save Fleiya Level Design Project As")
            .set_file_name("project.lcproj")
            .save_file(),
    };

    match picked {
        Some(path) => FileDialogOutcome::Selected(path),
        None => FileDialogOutcome::Cancelled,
    }
}

/// Adds the `.lcproj` extension when the user omits it in the save dialog.
pub(crate) fn normalize_project_path(path: PathBuf) -> PathBuf {
    if path.extension().is_some() {
        path
    } else {
        path.with_extension("lcproj")
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn desktop_available() -> bool {
    if std::env::var_os("LC_DISABLE_FILE_DIALOGS").is_some() {
        return false;
    }
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn desktop_available() -> bool {
    std::env::var_os("LC_DISABLE_FILE_DIALOGS").is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_reports_pending_then_outcome() {
        let (handle, sender) = FileDialogHandle::channel();
        assert_eq!(handle.try_take(), None);

        let path = PathBuf::from("/tmp/example.lcproj");
        sender
            .send(FileDialogOutcome::Selected(path.clone()))
            .expect("result channel must accept one outcome");

        assert_eq!(handle.try_take(), Some(FileDialogOutcome::Selected(path)));
        assert_eq!(handle.try_take(), None);
    }

    #[test]
    fn dropped_worker_reports_unavailable() {
        let (handle, sender) = FileDialogHandle::channel();
        drop(sender);

        assert_eq!(handle.try_take(), Some(FileDialogOutcome::Unavailable));
    }

    #[test]
    fn unavailable_service_returns_immediately() {
        let service = NativeFileDialogService::with_availability(false);

        assert_eq!(
            service.open_project().try_take(),
            Some(FileDialogOutcome::Unavailable)
        );
        assert_eq!(
            service.save_project_as().try_take(),
            Some(FileDialogOutcome::Unavailable)
        );
    }

    #[test]
    fn normalize_project_path_adds_missing_extension() {
        assert_eq!(
            normalize_project_path(PathBuf::from("/tmp/my_game")),
            PathBuf::from("/tmp/my_game.lcproj")
        );
        assert_eq!(
            normalize_project_path(PathBuf::from("/tmp/my_game.lcproj")),
            PathBuf::from("/tmp/my_game.lcproj")
        );
        assert_eq!(
            normalize_project_path(PathBuf::from("/tmp/archive.backup")),
            PathBuf::from("/tmp/archive.backup")
        );
    }
}
