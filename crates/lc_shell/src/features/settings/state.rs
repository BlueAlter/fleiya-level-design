//! State and category navigation model for the Settings workspace.

use lc_canvas::icons::Icon;
use lc_io::ShortcutAction;

/// Top-level category pages inside the Settings workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsCategory {
    /// Startup, language, and workflow defaults.
    General,
    /// Theme palette, UI scale, and top/status bar chrome.
    #[default]
    Appearance,
    /// Viewport grid, snapping, zoom, and pan interaction.
    Canvas,
    /// Customizable global keyboard shortcuts with collision checks.
    Keybindings,
    /// Auto-save, console defaults, undo limits, and config storage.
    System,
    /// Per-project settings (`Project.name` and `ProjectSettings`).
    Project,
}

impl SettingsCategory {
    /// All categories in sidebar display order.
    pub const ALL: [Self; 6] = [
        Self::General,
        Self::Appearance,
        Self::Canvas,
        Self::Keybindings,
        Self::System,
        Self::Project,
    ];

    /// Application-scoped categories.
    pub const APPLICATION: [Self; 5] = [
        Self::General,
        Self::Appearance,
        Self::Canvas,
        Self::Keybindings,
        Self::System,
    ];

    /// Project-scoped categories.
    pub const PROJECT: [Self; 1] = [Self::Project];

    /// Display label in the sidebar and page header.
    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Canvas => "Canvas & Controls",
            Self::Keybindings => "Keybindings",
            Self::System => "System & Storage",
            Self::Project => "Project Defaults",
        }
    }

    /// Short description shown beneath the category heading.
    pub const fn description(self) -> &'static str {
        match self {
            Self::General => "Configure startup workspace, language, and project loading behavior.",
            Self::Appearance => "Choose editor color palette, UI scale, and workspace chrome visibility.",
            Self::Canvas => "Adjust default grid overlay, snapping, zoom sensitivity, and pan speed.",
            Self::Keybindings => "Inspect and rebind global keyboard shortcuts with collision detection.",
            Self::System => "Manage auto-save intervals, console defaults, undo history, and reset actions.",
            Self::Project => "Edit active project title and default grid dimensions stored in project.lcproj.",
        }
    }

    /// Semantic icon representing this category in the sidebar.
    pub const fn icon(self) -> Icon {
        match self {
            Self::General => Icon::Settings,
            Self::Appearance => Icon::Palette,
            Self::Canvas => Icon::Grid,
            Self::Keybindings => Icon::Play,
            Self::System => Icon::Debug,
            Self::Project => Icon::Project,
        }
    }

    /// Search keywords matched when filtering settings in the workspace.
    pub const fn keywords(self) -> &'static str {
        match self {
            Self::General => "general startup language workspace auto open last project confirm exit",
            Self::Appearance => "appearance theme dark light color palette ui scale zoom title status bar",
            Self::Canvas => "canvas grid snap zoom sensitivity pan speed checkerboard",
            Self::Keybindings => "keybindings shortcuts hotkeys ctrl undo redo save workspace rebind",
            Self::System => "system storage auto save interval console debug collapse undo limit reset layout",
            Self::Project => "project name default grid size version worlds entities tilesets enums",
        }
    }

    /// Returns `true` when this category matches a lowercase search query.
    pub fn matches_query(self, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        self.label().to_lowercase().contains(query_lower)
            || self.description().to_lowercase().contains(query_lower)
            || self.keywords().contains(query_lower)
    }
}

/// Ephemeral UI state for the Settings workspace.
#[derive(Debug, Clone, Default)]
pub struct SettingsViewState {
    /// Currently selected category in the left sidebar.
    pub selected_category: SettingsCategory,
    /// Search query filtering categories and settings rows.
    pub search_query: String,
    /// Action currently waiting for a key press to rebind, if any.
    pub capturing_shortcut: Option<ShortcutAction>,
    /// Transient inline notice shown in the settings header when non-empty.
    pub notice: Option<String>,
    /// Editable draft buffer for `Project.name` before committing a command.
    pub project_name_draft: String,
    /// Snapshot of `Project.name` that `project_name_draft` was synced from.
    pub synced_project_name: Option<String>,
}

impl SettingsViewState {
    /// Syncs `project_name_draft` when the active project name changes externally.
    pub fn sync_project_name(&mut self, current_name: &str) {
        if self.synced_project_name.as_deref() != Some(current_name) {
            self.project_name_draft = current_name.to_string();
            self.synced_project_name = Some(current_name.to_string());
        }
    }
}
