//! Central shell application managing docking layout, menu, status bar, and document tabs.

use egui::{CentralPanel, Id, Panel, Ui};
use egui_dock::{DockArea, DockState};
use glam::IVec2;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::defs::EnumDef;
use lc_core::project::Project;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::features::editor::console::{BoundedRingBuffer, ConsolePanel, SharedRingBuffer};
use crate::features::editor::dock_layout::{DockSlotLocation, ShellDockLayout};
use crate::features::world::doc_tab_viewer::DocTabViewer;
use crate::shared::assets::ShellAssetService;
use crate::shared::file_dialog::{
    normalize_project_path, FileDialogHandle, FileDialogOutcome, FileDialogService,
    NativeFileDialogService,
};
use lc_canvas::theme::{self, ThemeChoice};
use lc_ui_api::session::SessionState;
use std::sync::Arc;

/// Which action a pending file dialog will perform when it completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileDialogAction {
    OpenProject,
    SaveProjectAs,
}

/// A file dialog whose worker thread has not reported back yet.
struct PendingFileDialog {
    action: FileDialogAction,
    handle: FileDialogHandle,
}

/// Active draft copy of enum definition being edited, keyed per enum id.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumDraft {
    pub def: EnumDef,
    pub tile_rects: HashMap<String, crate::shared::ui::tile_bounds::TileRect>,
    pub revision: u64,
    pub saved_revision: u64,
}

impl EnumDraft {
    pub fn new(def: EnumDef) -> Self {
        Self {
            def,
            tile_rects: HashMap::new(),
            revision: 0,
            saved_revision: 0,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }
}

/// Main application shell orchestrating panels, document tabs, commands, and layout.
pub struct ShellApp {
    /// Active project data model.
    pub project: Project,
    /// Path on disk where project was loaded/saved, if any.
    pub project_path: Option<PathBuf>,
    /// Global command undo/redo stack.
    pub undo_stack: UndoStack,
    /// Registry of feature providers, panels, and document factories.
    pub registry: Registry,
    /// Docking state for central document tabs.
    pub dock_state: DockState<DocKind>,
    /// Active open document views keyed by document kind.
    pub open_documents: HashMap<DocKind, Box<dyn DocumentView>>,
    /// Layout configuration for left, right, and bottom panel docks.
    pub dock_layout: ShellDockLayout,
    /// Queue of editor events emitted during frame rendering.
    pub events: Vec<EditorEvent>,
    /// Queue of commands submitted for execution during frame rendering.
    pub pending_commands: Vec<Box<dyn Command>>,
    /// Shared session state readable by panels, tools, and inspector sections.
    pub session: SessionState,
    /// Asset and texture service.
    pub assets: Arc<ShellAssetService>,
    /// Current status bar notification message.
    pub status_message: String,
    /// Active cursor grid position shown in status bar.
    pub cursor_grid_pos: Option<IVec2>,
    /// Current canvas zoom level (1.0 = 100%).
    pub zoom_level: f32,
    /// Tracks if there are unsaved modifications.
    pub is_dirty: bool,
    /// Selected appearance preference.
    pub theme_choice: ThemeChoice,
    /// Shared console log ring buffer.
    pub console_buffer: SharedRingBuffer,
    /// Last recorded bounding rect of the central document area.
    pub last_central_rect: Option<egui::Rect>,
    /// Backend used to request native file dialogs.
    file_dialogs: Arc<dyn FileDialogService>,
    /// File dialogs waiting for a user decision.
    pending_file_dialogs: Vec<PendingFileDialog>,
    /// Top-level active editor mode (Editor or Enums).
    pub active_mode: crate::features::editor::app_mode::AppMode,
    /// Active draft copies of enum definitions keyed per EnumDefId.
    pub enum_drafts: HashMap<lc_core::id::EnumDefId, EnumDraft>,
    /// Last saved copies of enum definitions keyed per EnumDefId.
    pub enum_saved_defs: HashMap<lc_core::id::EnumDefId, EnumDef>,
    /// Last saved copy of enum definitions.
    pub enum_saved: Vec<EnumDef>,
    /// Active draft copy of enum definitions being edited.
    pub enum_draft: Vec<EnumDef>,
    /// Revision counter for saved enums.
    pub enum_saved_revision: u64,
    /// Revision counter for draft enums.
    pub enum_draft_revision: u64,
    /// Mapping of enum id to chosen icon tileset id.
    pub enum_tileset_map: HashMap<lc_core::id::EnumDefId, lc_core::id::TilesetDefId>,
    /// Whether there are unsaved enum modifications (legacy flag).
    pub enum_unsaved: bool,
    /// Whether to show the prompt on closing with unsaved enum changes.
    pub show_unsaved_close_dialog: bool,
    /// Whether the floating layer definitions window is open.
    pub show_layer_window: bool,
    /// Whether the floating entity definitions window is open.
    pub show_entities_window: bool,
    /// Whether the floating tileset definitions window is open.
    pub show_tileset_window: bool,
    /// Whether the floating enum definitions window is open.
    pub show_enum_window: bool,
    /// Currently focused layer definition in the Layer window.
    pub selected_layer_def: Option<lc_core::id::LayerDefId>,
    /// Currently focused entity definition in the Entities window.
    pub selected_entity_def: Option<lc_core::id::EntityDefId>,
    /// Currently focused tileset definition in the Tileset window.
    pub selected_tileset_def: Option<lc_core::id::TilesetDefId>,
    /// Currently focused enum definition in the Enum window.
    pub selected_enum_def: Option<lc_core::id::EnumDefId>,
    /// Pending delete action for Enum definitions window.
    pub pending_enum_delete: Option<crate::features::enums::window::PendingDeleteModal>,
    /// Project-level tag registry shared across enums and tilesets.
    pub tag_registry: Vec<TagDef>,
    /// Counter for allocating unique TagIds.
    pub next_tag_id: u32,
    /// Tags assigned to each enum definition (draft).
    pub enum_tags_map: HashMap<lc_core::id::EnumDefId, Vec<TagId>>,
    /// Tags assigned to each tileset definition.
    pub tileset_tags_map: HashMap<lc_core::id::TilesetDefId, Vec<TagId>>,
    /// Saved tag registry for revert/save parity.
    pub tag_registry_saved: Vec<TagDef>,
    /// Saved next tag ID.
    pub next_tag_id_saved: u32,
    /// Saved enum tags mapping for revert/save parity.
    pub enum_tags_saved: HashMap<lc_core::id::EnumDefId, Vec<TagId>>,
    /// Mapping of (EnumDefId, EnumValueId) to TileRect multi-tile selections (draft).
    pub enum_value_tile_rects:
        HashMap<(lc_core::id::EnumDefId, String), crate::shared::ui::tile_bounds::TileRect>,
    /// Saved copy of enum value tile rects for revert/save parity.
    pub enum_saved_tile_rects:
        HashMap<(lc_core::id::EnumDefId, String), crate::shared::ui::tile_bounds::TileRect>,
}

/// Strongly typed identifier for a project tag.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TagId(pub u32);

/// Project-level definition of a tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TagDef {
    pub id: TagId,
    pub name: String,
}

impl Default for ShellApp {
    fn default() -> Self {
        Self::new(Project::new("Fleiya Level Design Project"))
    }
}

impl ShellApp {
    /// Creates a new shell instance for a given project with a default console buffer.
    pub fn new(project: Project) -> Self {
        let buffer = SharedRingBuffer::new(BoundedRingBuffer::DEFAULT_CAPACITY);
        Self::new_with_console(project, buffer)
    }

    /// Creates a shell instance with default project and specified console buffer.
    pub fn from_console(console_buffer: SharedRingBuffer) -> Self {
        Self::new_with_console(Project::new("Fleiya Level Design Project"), console_buffer)
    }

    /// Creates a new shell instance using an existing console ring buffer.
    pub fn new_with_console(project: Project, console_buffer: SharedRingBuffer) -> Self {
        let mut registry = Registry::new();
        registry.register_panel(Box::new(ConsolePanel::new(console_buffer.clone())));
        registry.register_panel(Box::new(crate::features::project::ProjectPanel::new()));

        lc_feature_level::register(&mut registry);
        lc_feature_tileset::register(&mut registry);
        lc_feature_layers::register(&mut registry);
        lc_feature_enums::register(&mut registry);
        lc_feature_entities::register(&mut registry);
        lc_feature_world::register(&mut registry);

        registry.register_panel(Box::new(
            crate::features::tileset::panel::CombinedTilesetPanel::new(),
        ));
        registry.register_panel(Box::new(
            crate::features::editor::inspector_panel::InspectorPanel::new(),
        ));

        let assets = Arc::new(ShellAssetService::new());

        let enum_saved = project.defs.enums.clone();
        let enum_draft = project.defs.enums.clone();

        let mut enum_drafts = HashMap::new();
        let mut enum_saved_defs = HashMap::new();
        for def in &project.defs.enums {
            enum_drafts.insert(def.id, EnumDraft::new(def.clone()));
            enum_saved_defs.insert(def.id, def.clone());
        }

        Self {
            project,
            project_path: None,
            undo_stack: UndoStack::new(),
            registry,
            dock_state: DockState::new(Vec::new()),
            open_documents: HashMap::new(),
            dock_layout: ShellDockLayout::default(),
            events: Vec::new(),
            pending_commands: Vec::new(),
            session: SessionState::default(),
            assets,
            status_message: "Ready".to_string(),
            cursor_grid_pos: None,
            zoom_level: 1.0,
            is_dirty: false,
            theme_choice: ThemeChoice::Dark,
            console_buffer,
            last_central_rect: None,
            file_dialogs: Arc::new(NativeFileDialogService::new()),
            pending_file_dialogs: Vec::new(),
            active_mode: crate::features::editor::app_mode::AppMode::Editor,
            enum_drafts,
            enum_saved_defs,
            enum_saved,
            enum_draft,
            enum_saved_revision: 0,
            enum_draft_revision: 0,
            enum_tileset_map: HashMap::new(),
            enum_unsaved: false,
            show_unsaved_close_dialog: false,
            show_layer_window: false,
            show_entities_window: false,
            show_tileset_window: false,
            show_enum_window: false,
            selected_layer_def: None,
            selected_entity_def: None,
            selected_tileset_def: None,
            selected_enum_def: None,
            pending_enum_delete: None,
            tag_registry: Vec::new(),
            next_tag_id: 1,
            enum_tags_map: HashMap::new(),
            tileset_tags_map: HashMap::new(),
            tag_registry_saved: Vec::new(),
            next_tag_id_saved: 1,
            enum_tags_saved: HashMap::new(),
            enum_value_tile_rects: HashMap::new(),
            enum_saved_tile_rects: HashMap::new(),
        }
    }

    /// Retrieves the `TileRect` for an enum value, with backward compatibility for old data (single tile read as 1x1).
    pub fn get_enum_value_tile_rect(
        &self,
        enum_id: lc_core::id::EnumDefId,
        val_id: &str,
        tile: Option<&lc_core::layer::TileRef>,
        cols: u32,
    ) -> Option<crate::shared::ui::tile_bounds::TileRect> {
        if let Some(rect) = self
            .enum_value_tile_rects
            .get(&(enum_id, val_id.to_string()))
            .copied()
        {
            return Some(rect);
        }
        tile.map(|tr| {
            let col = tr.tile_id.checked_rem(cols).unwrap_or(0);
            let row = tr.tile_id.checked_div(cols).unwrap_or(0);
            crate::shared::ui::tile_bounds::TileRect::single(col, row)
        })
    }

    /// Access to the console ring buffer.
    pub fn console_buffer(&self) -> &SharedRingBuffer {
        &self.console_buffer
    }

    /// Ensures an EnumDraft exists for the given ID and returns a mutable reference.
    pub fn ensure_enum_draft(&mut self, enum_id: lc_core::id::EnumDefId) -> &mut EnumDraft {
        if !self.enum_drafts.contains_key(&enum_id) {
            let def = self
                .enum_draft
                .iter()
                .find(|e| e.id == enum_id)
                .cloned()
                .or_else(|| {
                    self.project
                        .defs
                        .enums
                        .iter()
                        .find(|e| e.id == enum_id)
                        .cloned()
                })
                .unwrap_or_else(|| EnumDef::new(enum_id, format!("Enum_{}", enum_id.0)));
            let mut draft = EnumDraft::new(def);
            for ((eid, vid), rect) in &self.enum_value_tile_rects {
                if *eid == enum_id {
                    draft.tile_rects.insert(vid.clone(), *rect);
                }
            }
            self.enum_drafts.insert(enum_id, draft);
        }
        self.enum_drafts.get_mut(&enum_id).unwrap()
    }

    /// Gets an immutable reference to the EnumDraft for the given EnumDefId.
    pub fn get_enum_draft(&self, enum_id: lc_core::id::EnumDefId) -> Option<&EnumDraft> {
        self.enum_drafts.get(&enum_id)
    }

    /// Checks if the draft for a specific enum is dirty (modified since last save).
    pub fn is_enum_draft_dirty(&self, enum_id: lc_core::id::EnumDefId) -> bool {
        if let Some(draft) = self.enum_drafts.get(&enum_id) {
            if draft.is_dirty() {
                return true;
            }
            if let Some(saved) = self.enum_saved_defs.get(&enum_id) {
                if draft.def != *saved {
                    return true;
                }
            }
        }
        if let Some(draft_def) = self.enum_draft.iter().find(|e| e.id == enum_id) {
            if let Some(saved_def) = self.enum_saved.iter().find(|e| e.id == enum_id) {
                if draft_def != saved_def {
                    return true;
                }
            }
        }
        false
    }

    /// Whether there are unsaved enum modifications between draft and saved.
    pub fn enum_is_dirty(&self) -> bool {
        if let Some(sel_id) = self.selected_enum_def {
            self.is_enum_draft_dirty(sel_id)
        } else {
            self.enum_drafts.values().any(|d| d.is_dirty())
                || self.enum_draft_revision != self.enum_saved_revision
        }
    }

    /// Number of unsaved enum modifications.
    pub fn unsaved_changes_count(&self) -> u64 {
        self.enum_draft_revision
            .saturating_sub(self.enum_saved_revision)
    }

    /// Resolves the width and height of a tileset's image asset.
    pub fn resolve_tileset_image_size(&self, id: lc_core::id::TilesetDefId) -> Option<(u32, u32)> {
        let ts = self.project.defs.tileset(id)?;
        let full_path = self.assets.resolve_path(&ts.path);
        if full_path.exists() {
            if let Some(dims) = crate::features::enums::file::read_png_dimensions(&full_path) {
                return Some(dims);
            }
        }
        let candidate_paths = [
            std::path::PathBuf::from(&ts.path),
            std::path::PathBuf::from("examples/sample_project").join(&ts.path),
            std::path::PathBuf::from("examples").join(&ts.path),
        ];
        for p in &candidate_paths {
            if p.exists() {
                if let Some(dims) = crate::features::enums::file::read_png_dimensions(p) {
                    return Some(dims);
                }
            }
        }
        Some((256, 256))
    }

    /// Gets existing tag ID with case-insensitive match, or creates a new tag in the registry.
    pub fn get_or_create_tag(&mut self, name: &str) -> TagId {
        let trimmed = name.trim();
        if let Some(existing) = self
            .tag_registry
            .iter()
            .find(|t| t.name.trim().eq_ignore_ascii_case(trimmed))
        {
            return existing.id;
        }
        let id = TagId(self.next_tag_id);
        self.next_tag_id += 1;
        self.tag_registry.push(TagDef {
            id,
            name: trimmed.to_string(),
        });
        id
    }

    /// Assigns a tag to an enum.
    pub fn assign_tag_to_enum(&mut self, enum_id: lc_core::id::EnumDefId, tag_id: TagId) {
        let tags = self.enum_tags_map.entry(enum_id).or_default();
        if !tags.contains(&tag_id) {
            tags.push(tag_id);
            self.enum_draft_revision += 1;
        }
    }

    /// Removes a tag from an enum and cleans up unused tags.
    pub fn remove_tag_from_enum(&mut self, enum_id: lc_core::id::EnumDefId, tag_id: TagId) {
        if let Some(tags) = self.enum_tags_map.get_mut(&enum_id) {
            tags.retain(|&id| id != tag_id);
            self.cleanup_unused_tags();
            self.enum_draft_revision += 1;
        }
    }

    /// Removes any tags from the registry that are not used by any enum or tileset.
    pub fn cleanup_unused_tags(&mut self) {
        let mut used = std::collections::HashSet::new();
        for tags in self.enum_tags_map.values() {
            for &t in tags {
                used.insert(t);
            }
        }
        for tags in self.tileset_tags_map.values() {
            for &t in tags {
                used.insert(t);
            }
        }
        self.tag_registry.retain(|t| used.contains(&t.id));
    }

    /// Renames a tag across all enums and tilesets.
    /// Rejects if new_name is empty or duplicates an existing tag name (case-insensitive).
    pub fn rename_tag(&mut self, tag_id: TagId, new_name: &str) -> Result<(), String> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err("Tag name cannot be empty".to_string());
        }
        if self
            .tag_registry
            .iter()
            .any(|t| t.id != tag_id && t.name.trim().eq_ignore_ascii_case(trimmed))
        {
            return Err("A tag with this name already exists".to_string());
        }
        if let Some(tag) = self.tag_registry.iter_mut().find(|t| t.id == tag_id) {
            tag.name = trimmed.to_string();
            self.enum_draft_revision += 1;
            Ok(())
        } else {
            Err("Tag not found".to_string())
        }
    }

    /// Returns tags from the registry that are not currently assigned to the given enum.
    pub fn unused_tags_for_enum(&self, enum_id: lc_core::id::EnumDefId) -> Vec<TagDef> {
        let assigned = self
            .enum_tags_map
            .get(&enum_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        self.tag_registry
            .iter()
            .filter(|t| !assigned.contains(&t.id))
            .cloned()
            .collect()
    }

    /// Duplicates an enum definition and places it directly below the original.
    pub fn duplicate_enum(
        &mut self,
        original_id: lc_core::id::EnumDefId,
    ) -> Option<lc_core::id::EnumDefId> {
        let orig_idx = self.enum_draft.iter().position(|e| e.id == original_id)?;
        let orig = self.enum_draft[orig_idx].clone();

        let base_name = &orig.name;
        let mut candidate_name = format!("{base_name}_Copy");
        let mut counter = 2;
        while self.enum_draft.iter().any(|e| e.name == candidate_name) {
            candidate_name = format!("{base_name}_Copy{counter}");
            counter += 1;
        }

        let next_id =
            lc_core::id::EnumDefId(self.enum_draft.iter().map(|e| e.id.0).max().unwrap_or(0) + 1);

        let mut dup_def = orig;
        dup_def.id = next_id;
        dup_def.name = candidate_name;

        self.enum_draft.insert(orig_idx + 1, dup_def);

        // Copy tags
        if let Some(tags) = self.enum_tags_map.get(&original_id).cloned() {
            self.enum_tags_map.insert(next_id, tags);
        }

        // Copy tileset mapping
        if let Some(ts_id) = self.enum_tileset_map.get(&original_id).copied() {
            self.enum_tileset_map.insert(next_id, ts_id);
        }

        self.selected_enum_def = Some(next_id);
        self.enum_draft_revision += 1;
        Some(next_id)
    }

    /// Read-only access to registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Mutable access to registry for feature registration.
    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }

    /// Read-only access to the loaded project model.
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Mutable access to the project model.
    pub fn project_mut(&mut self) -> &mut Project {
        &mut self.project
    }

    /// Read-only access to shared session state.
    pub fn session(&self) -> &SessionState {
        &self.session
    }

    /// Mutable access to shared session state.
    pub fn session_mut(&mut self) -> &mut SessionState {
        &mut self.session
    }

    /// Returns true if there are uncommitted or unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Applies a new appearance preference immediately.
    pub fn set_theme(&mut self, ctx: &egui::Context, choice: ThemeChoice) {
        self.theme_choice = choice;
        theme::apply(ctx, choice);
    }

    /// Saves the active project to the given directory path using `lc_io`.
    pub fn save_project_to_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        lc_io::save_project(&self.project, path)?;
        self.project_path = Some(path.to_path_buf());
        self.assets
            .set_project_dir(path.parent().map(|p| p.to_path_buf()));
        self.is_dirty = false;
        self.status_message = format!("Project saved to {}", path.display());
        tracing::info!(path = %path.display(), "Saved project");
        Ok(())
    }

    /// Loads a project from the given directory path using `lc_io`.
    pub fn open_project_from_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        let loaded = lc_io::load_project(path)?;
        self.project = loaded;
        self.enum_saved = self.project.defs.enums.clone();
        self.enum_draft = self.project.defs.enums.clone();
        self.enum_saved_revision = 0;
        self.enum_draft_revision = 0;
        self.enum_tileset_map.clear();
        self.enum_drafts.clear();
        self.enum_saved_defs.clear();
        let has_ts1 = self
            .project
            .defs
            .tileset(lc_core::id::TilesetDefId(1))
            .is_some();
        let has_ts2 = self
            .project
            .defs
            .tileset(lc_core::id::TilesetDefId(2))
            .is_some();
        for enum_def in &mut self.project.defs.enums {
            if enum_def.tileset_id.is_none() {
                enum_def.tileset_id = enum_def
                    .values
                    .iter()
                    .find_map(|v| v.tile.map(|t| t.tileset_id));
            }
            if enum_def.name == "BiomeType" && enum_def.tileset_id.is_none() && has_ts1 {
                enum_def.tileset_id = Some(lc_core::id::TilesetDefId(1));
            }
            if enum_def.name == "WeaponType" && enum_def.tileset_id.is_none() && has_ts2 {
                enum_def.tileset_id = Some(lc_core::id::TilesetDefId(2));
            }
            if let Some(ts_id) = enum_def.tileset_id {
                self.enum_tileset_map.insert(enum_def.id, ts_id);
            }
            self.enum_drafts
                .insert(enum_def.id, EnumDraft::new(enum_def.clone()));
            self.enum_saved_defs.insert(enum_def.id, enum_def.clone());
        }
        self.enum_saved = self.project.defs.enums.clone();
        self.enum_draft = self.project.defs.enums.clone();
        self.tag_registry.clear();
        self.next_tag_id = 1;
        self.enum_tags_map.clear();
        self.tileset_tags_map.clear();
        self.tag_registry_saved.clear();
        self.next_tag_id_saved = 1;
        self.enum_tags_saved.clear();
        self.project_path = Some(path.to_path_buf());
        self.assets
            .set_project_dir(path.parent().map(|p| p.to_path_buf()));
        self.undo_stack.clear();
        self.open_documents.clear();
        self.dock_state = DockState::new(Vec::new());
        self.is_dirty = false;
        self.status_message = format!("Opened project from {}", path.display());
        tracing::info!(path = %path.display(), "Opened project");
        Ok(())
    }

    /// Replaces the backend used for native file dialogs.
    ///
    /// Tests and headless harnesses use this to inject a mock service.
    pub fn set_file_dialog_service(&mut self, service: Arc<dyn FileDialogService>) {
        self.file_dialogs = service;
    }

    /// Starts an asynchronous "Open project" dialog.
    pub fn request_open_project_dialog(&mut self) {
        if self.has_pending_dialog(FileDialogAction::OpenProject) {
            self.status_message = "Open dialog is already active".to_string();
            return;
        }

        tracing::info!("Requested open project dialog");
        self.pending_file_dialogs.push(PendingFileDialog {
            action: FileDialogAction::OpenProject,
            handle: self.file_dialogs.open_project(),
        });
        self.status_message = "Select a project file...".to_string();
    }

    /// Starts an asynchronous "Save project as" dialog.
    pub fn request_save_project_as_dialog(&mut self) {
        if self.has_pending_dialog(FileDialogAction::SaveProjectAs) {
            self.status_message = "Save As dialog is already active".to_string();
            return;
        }

        tracing::info!("Requested save project dialog");
        self.pending_file_dialogs.push(PendingFileDialog {
            action: FileDialogAction::SaveProjectAs,
            handle: self.file_dialogs.save_project_as(),
        });
        self.status_message = "Choose a destination for the project...".to_string();
    }

    /// Polls pending file dialogs and applies any completed results.
    ///
    /// Returns `true` when at least one dialog finished during this call.
    pub fn poll_file_dialogs(&mut self) -> bool {
        if self.pending_file_dialogs.is_empty() {
            return false;
        }

        let pending = std::mem::take(&mut self.pending_file_dialogs);
        let mut completed = Vec::new();
        let mut still_pending = Vec::with_capacity(pending.len());
        for dialog in pending {
            match dialog.handle.try_take() {
                Some(outcome) => completed.push((dialog.action, outcome)),
                None => still_pending.push(dialog),
            }
        }
        self.pending_file_dialogs = still_pending;

        let any_completed = !completed.is_empty();
        for (action, outcome) in completed {
            self.apply_file_dialog_outcome(action, outcome);
        }
        any_completed
    }

    fn has_pending_dialog(&self, action: FileDialogAction) -> bool {
        self.pending_file_dialogs
            .iter()
            .any(|dialog| dialog.action == action)
    }

    fn apply_file_dialog_outcome(&mut self, action: FileDialogAction, outcome: FileDialogOutcome) {
        match outcome {
            FileDialogOutcome::Selected(path) => match action {
                FileDialogAction::OpenProject => {
                    if let Err(error) = self.open_project_from_path(&path) {
                        tracing::error!(
                            %error,
                            path = %path.display(),
                            "Failed to open project from dialog"
                        );
                        self.status_message = format!("Failed to open project: {error}");
                    }
                }
                FileDialogAction::SaveProjectAs => {
                    let path = normalize_project_path(path);
                    if let Err(error) = self.save_project_to_path(&path) {
                        tracing::error!(
                            %error,
                            path = %path.display(),
                            "Failed to save project from dialog"
                        );
                        self.status_message = format!("Failed to save project: {error}");
                    }
                }
            },
            FileDialogOutcome::Cancelled => {
                tracing::info!("File dialog cancelled");
                self.status_message = "File dialog cancelled".to_string();
            }
            FileDialogOutcome::Unavailable => {
                tracing::warn!("File dialog unavailable in this environment");
                self.status_message =
                    "File dialog unavailable in this environment (headless mode)".to_string();
            }
        }
    }

    /// Enqueues a command for deferred execution.
    pub fn execute_command(&mut self, cmd: Box<dyn Command>) {
        self.pending_commands.push(cmd);
    }

    /// Opens or focuses a document tab, or opens the corresponding floating window for definition kinds.
    pub fn open_document(&mut self, kind: DocKind) {
        match kind {
            DocKind::Level(level_id) => {
                if self.open_documents.contains_key(&kind) {
                    self.focus_document(kind);
                    return;
                }
                let mut view = lc_feature_level::LevelDocumentView::new(level_id);
                view.add_handler(Box::new(lc_feature_tileset::TilesLayerHandler::new()));
                view.add_handler(Box::new(lc_feature_tileset::IntGridLayerHandler::new()));
                view.add_handler(Box::new(lc_feature_entities::EntitiesLayerHandler::new()));
                view.add_inspector_section(Box::new(
                    lc_feature_entities::EntityInspectorSection::new(),
                ));
                view.set_tool(Box::new(
                    crate::features::editor::tool_router::LevelCanvasToolRouter::new(),
                ));
                self.open_documents.insert(kind, Box::new(view));
                self.dock_state.push_to_first_leaf(kind);
                self.status_message = format!("Opened Level {:?}", level_id.0);
            }
            DocKind::World(world_id) => {
                if self.open_documents.contains_key(&kind) {
                    self.focus_document(kind);
                    return;
                }
                let doc = self.registry.create_document(kind).unwrap_or_else(|| {
                    Box::new(
                        crate::features::world::doc_tab_viewer::PlaceholderDocumentView { kind },
                    )
                });
                self.open_documents.insert(kind, doc);
                self.dock_state.push_to_first_leaf(kind);
                self.status_message = format!("Opened World {:?}", world_id.0);
            }
            DocKind::LayerDef(layer_id) => {
                self.show_layer_window = true;
                self.selected_layer_def = Some(layer_id);
                self.status_message = "Opened Layer Window".to_string();
            }
            DocKind::EntityDef(entity_id) => {
                self.show_entities_window = true;
                self.selected_entity_def = Some(entity_id);
                self.status_message = "Opened Entities Window".to_string();
            }
            DocKind::TilesetDef(tileset_id) => {
                self.show_tileset_window = true;
                self.selected_tileset_def = Some(tileset_id);
                self.status_message = "Opened Tileset Window".to_string();
            }
            DocKind::EnumDef(enum_id) => {
                if self.open_documents.contains_key(&kind) {
                    self.focus_document(kind);
                    return;
                }
                let doc = self.registry.create_document(kind).unwrap_or_else(|| {
                    Box::new(
                        crate::features::world::doc_tab_viewer::PlaceholderDocumentView { kind },
                    )
                });
                self.open_documents.insert(kind, doc);
                self.dock_state.push_to_first_leaf(kind);
                self.status_message = format!("Opened Enum {:?}", enum_id.0);
            }
            DocKind::ProjectSettings => {
                if self.open_documents.contains_key(&kind) {
                    self.focus_document(kind);
                    return;
                }
                let doc = self.registry.create_document(kind).unwrap_or_else(|| {
                    Box::new(
                        crate::features::world::doc_tab_viewer::PlaceholderDocumentView { kind },
                    )
                });
                self.open_documents.insert(kind, doc);
                self.dock_state.push_to_first_leaf(kind);
                self.status_message = "Opened Project Settings".to_string();
            }
        }
    }

    /// Automatically opens the first level of the project into a document tab and focuses it.
    pub fn open_first_level(&mut self) {
        if let Some(level_id) = self
            .project
            .worlds
            .first()
            .and_then(|w| w.levels.first())
            .map(|l| l.id)
        {
            self.open_document(DocKind::Level(level_id));
            self.session.active_level = Some(level_id);
            self.session.active_document = Some(DocKind::Level(level_id));

            if let Some(first_layer) = self.project.level(level_id).and_then(|l| l.layers.first()) {
                self.session.active_layer = Some(first_layer.def_id);
            }
        }
    }

    /// Focuses an existing open document tab in the dock state.
    pub fn focus_document(&mut self, kind: DocKind) {
        if let Some(pos) = self.dock_state.find_tab(&kind) {
            let _ = self.dock_state.set_active_tab(pos);
        }
    }

    /// Closes an open document tab.
    pub fn close_document(&mut self, kind: &DocKind) {
        self.open_documents.remove(kind);
        self.dock_state.retain_tabs(|t| t != kind);
        if self.session.active_document == Some(*kind) {
            self.session.active_document = None;
            if let DocKind::Level(_) = kind {
                self.session.active_level = None;
            }
        }
    }

    /// Triggers an undo step.
    pub fn undo(&mut self) {
        if let Some((label, focus)) = self.undo_stack.undo(&mut self.project) {
            self.status_message = format!("Undo: {label}");
            if let Some(doc) = focus {
                self.open_document(doc);
                self.session.active_document = Some(doc);
                if let DocKind::Level(lid) = doc {
                    self.session.active_level = Some(lid);
                }
            }
        }
    }

    /// Triggers a redo step.
    pub fn redo(&mut self) {
        match self.undo_stack.redo(&mut self.project) {
            Ok(Some((label, focus))) => {
                self.status_message = format!("Redo: {label}");
                if let Some(doc) = focus {
                    self.open_document(doc);
                    self.session.active_document = Some(doc);
                    if let DocKind::Level(lid) = doc {
                        self.session.active_level = Some(lid);
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                self.status_message = format!("Redo failed: {e}");
            }
        }
    }

    /// Restores default docking layout.
    pub fn reset_layout(&mut self) {
        self.dock_layout = ShellDockLayout::default();
        self.dock_state = DockState::new(self.open_documents.keys().copied().collect());
        self.status_message = "Layout reset to defaults".to_string();
    }

    /// Renders the complete shell UI inside a top-level egui `Ui`.
    pub fn ui(&mut self, ui: &mut Ui) {
        // Apply results from asynchronous file dialogs without blocking this frame.
        self.poll_file_dialogs();
        if !self.pending_file_dialogs.is_empty() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Ensure registered panels are tracked in dock layout
        for panel in self.registry.panels() {
            self.dock_layout
                .ensure_panel(panel.id(), DockSlotLocation::Left);
        }

        crate::features::editor::menu::handle_global_shortcuts(self, ui.ctx());

        let tokens = theme::Tokens::get(ui.ctx());

        // 1. Top menu bar
        Panel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(tokens.bg)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| {
                crate::features::editor::menu::render_menu_bar(self, ui);
            });
        match self.active_mode {
            crate::features::editor::app_mode::AppMode::Editor => {
                // 2. Bottom status bar
                Panel::bottom("status_bar")
                    .exact_size(theme::STATUS_H)
                    .frame(
                        egui::Frame::new()
                            .fill(tokens.bg)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ui, |ui| {
                        crate::features::editor::status_bar::render_status_bar(self, ui);
                    });
                let mut dock_geometry =
                    crate::features::editor::dock_panels::DockGeometry::default();
                const DOCK_UNIFORM_WIDTH: f32 = 260.0;
                const DOCK_MIN_WIDTH: f32 = 240.0;

                // 3. Left dock panel
                if !self.dock_layout.left.panels.is_empty() {
                    Panel::left("left_dock")
                        .resizable(true)
                        .default_size(DOCK_UNIFORM_WIDTH)
                        .min_size(DOCK_MIN_WIDTH)
                        .frame(
                            egui::Frame::new()
                                .fill(tokens.panel)
                                .inner_margin(egui::Margin::ZERO),
                        )
                        .show(ui, |ui| {
                            ui.set_min_width(DOCK_MIN_WIDTH);
                            dock_geometry.left =
                                Some(crate::features::editor::dock_panels::render_dock_slot(
                                    self,
                                    ui,
                                    DockSlotLocation::Left,
                                ));
                        });
                }
                // 4. Right dock panel
                if !self.dock_layout.right.panels.is_empty() {
                    Panel::right("right_dock")
                        .resizable(true)
                        .default_size(DOCK_UNIFORM_WIDTH)
                        .min_size(DOCK_MIN_WIDTH)
                        .frame(
                            egui::Frame::new()
                                .fill(tokens.panel)
                                .inner_margin(egui::Margin::ZERO),
                        )
                        .show(ui, |ui| {
                            ui.set_min_width(DOCK_MIN_WIDTH);
                            dock_geometry.right =
                                Some(crate::features::editor::dock_panels::render_dock_slot(
                                    self,
                                    ui,
                                    DockSlotLocation::Right,
                                ));
                        });
                }
                // 5. Bottom dock (collapsible)
                dock_geometry.bottom =
                    crate::features::editor::dock_panels::render_bottom_dock(self, ui);
                // 6. Central document area (egui_dock)
                CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(tokens.bg)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ui, |ui| {
                        self.render_central_documents(ui);
                    });

                // 7. Drag preview, drop indicator and drop completion.
                crate::features::editor::dock_panels::handle_drag_and_drop(
                    self,
                    ui,
                    &dock_geometry,
                );

                // 7b. Floating definition windows
                crate::features::layers::window::render_layer_window(self, ui.ctx());
                crate::features::entities::window::render_entities_window(self, ui.ctx());
                crate::features::tileset::window::render_tileset_window(self, ui.ctx());
            }
            crate::features::editor::app_mode::AppMode::Enums => {
                crate::features::enums::view::render_enum_view(self, ui);
            }
            crate::features::editor::app_mode::AppMode::Tilesets => {}
        }

        // 8. Process deferred commands and events
        self.process_commands_and_events();
    }

    fn render_central_documents(&mut self, ui: &mut Ui) {
        self.last_central_rect = Some(ui.max_rect());

        if self.open_documents.is_empty() && self.dock_state.iter_all_tabs().next().is_none() {
            self.session.active_document = None;
            self.session.active_level = None;
            crate::features::editor::workspace::render_workspace_empty(self, ui);
            return;
        }

        let is_dirty = self.is_dirty();

        if let Some((_, active_tab)) = self.dock_state.find_active_focused() {
            self.session.active_document = Some(*active_tab);
            self.session.active_level = if let DocKind::Level(lid) = *active_tab {
                Some(lid)
            } else {
                None
            };
        } else {
            self.session.active_document = None;
            self.session.active_level = None;
        }

        let mut tab_viewer = DocTabViewer {
            project: &self.project,
            open_documents: &mut self.open_documents,
            undo_stack: &self.undo_stack,
            session: &self.session,
            events: &mut self.events,
            pending_commands: &mut self.pending_commands,
            assets: &*self.assets,
            is_dirty,
        };

        let dock_style = crate::features::editor::dock_style::style(&theme::Tokens::get(ui.ctx()));
        DockArea::new(&mut self.dock_state)
            .id(Id::new("central_document_dock_area"))
            .style(dock_style)
            .show_inside(ui, &mut tab_viewer);

        // Prune any documents that were closed or became invalid, or that should not be tabs
        let active_tabs: Vec<DocKind> = self
            .dock_state
            .iter_all_tabs()
            .map(|(_, t)| *t)
            .filter(|t| {
                matches!(
                    t,
                    DocKind::Level(_)
                        | DocKind::World(_)
                        | DocKind::ProjectSettings
                        | DocKind::TilesetDef(_)
                        | DocKind::EnumDef(_)
                )
            })
            .collect();
        self.open_documents
            .retain(|k, v| active_tabs.contains(k) && v.is_valid(&self.project));
        let valid_docs = self.open_documents.keys().copied().collect::<Vec<_>>();
        self.dock_state.retain_tabs(|t| valid_docs.contains(t));
    }

    /// Processes queued commands and events.
    pub fn process_commands_and_events(&mut self) {
        // 1. Apply queued commands
        let commands = std::mem::take(&mut self.pending_commands);
        for cmd in commands {
            let label = cmd.label();
            let focus = cmd.focus();
            if let Err(e) = self.undo_stack.execute(cmd, &mut self.project) {
                tracing::error!("Error executing {label}: {e}");
                self.status_message = format!("Error executing {label}: {e}");
            } else {
                tracing::debug!("Applied command: {label}");
                self.is_dirty = true;
                self.status_message = label;
                if let Some(doc) = focus {
                    self.open_document(doc);
                    self.session.active_document = Some(doc);
                    if let DocKind::Level(lid) = doc {
                        self.session.active_level = Some(lid);
                    }
                }
            }
        }

        // 2. Handle emitted events
        let events = std::mem::take(&mut self.events);
        for event in events {
            match event {
                EditorEvent::OpenDocument(kind) => {
                    self.open_document(kind);
                    self.session.active_document = Some(kind);
                    if let DocKind::Level(lid) = kind {
                        self.session.active_level = Some(lid);
                    }
                }
                EditorEvent::FocusDocument(kind) => {
                    self.focus_document(kind);
                    self.session.active_document = Some(kind);
                    if let DocKind::Level(lid) = kind {
                        self.session.active_level = Some(lid);
                    }
                }
                EditorEvent::CloseDocument(kind) => {
                    self.close_document(&kind);
                }
                EditorEvent::Status(msg) => self.status_message = msg,
                EditorEvent::SelectionChanged(sel) => {
                    tracing::debug!("Selection changed: {sel:?}");
                    self.session.selection = sel;
                }
                EditorEvent::SetActiveLevel(id) => {
                    self.session.active_level = id;
                }
                EditorEvent::SetActiveLayer(id) => {
                    self.session.active_layer = id;
                }
                EditorEvent::SetActiveTool(name) => {
                    self.session.active_tool_name = name;
                }
                EditorEvent::SetTileSelection(ts) => {
                    self.session.tile_selection = ts;
                }
                EditorEvent::SaveProject => {
                    if let Some(ref path) = self.project_path {
                        if let Err(e) = lc_io::save_project(&self.project, path) {
                            tracing::error!("Failed to save project: {e}");
                            self.status_message = format!("Failed to save project: {e}");
                        } else {
                            self.is_dirty = false;
                            self.status_message = format!("Saved project to {}", path.display());
                        }
                    } else {
                        tracing::warn!("Cannot save project: no project path configured");
                        self.status_message =
                            "Save failed: no project path configured (specify path or open project first)".to_string();
                    }
                }
            }
        }
    }
}
