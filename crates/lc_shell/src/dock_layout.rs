//! Dock layout configuration and persistence for side and bottom panels.

use lc_ui_api::view::PanelId;
use serde::{Deserialize, Serialize};

/// Target slot location for dockable tool panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockSlotLocation {
    Left,
    Right,
    Bottom,
}

/// State of a single dock slot containing a collection of panel tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockSlotState {
    /// Ordered list of panels docked in this slot.
    pub panels: Vec<PanelId>,
    /// Identifier of the currently active panel tab.
    pub active: Option<PanelId>,
}

impl DockSlotState {
    /// Creates a new slot with the given panels and active selection.
    pub fn new(panels: Vec<PanelId>, active: Option<PanelId>) -> Self {
        Self { panels, active }
    }

    /// Selects a panel tab if present.
    pub fn set_active(&mut self, id: PanelId) {
        if self.panels.contains(&id) {
            self.active = Some(id);
        }
    }

    /// Adds a panel to this slot and focuses it.
    pub fn add_panel(&mut self, id: PanelId) {
        if !self.panels.contains(&id) {
            self.panels.push(id.clone());
        }
        self.active = Some(id);
    }

    /// Removes a panel from this slot.
    pub fn remove_panel(&mut self, id: &PanelId) {
        self.panels.retain(|p| p != id);
        if self.active.as_ref() == Some(id) {
            self.active = self.panels.first().cloned();
        }
    }
}

/// State of the bottom dock slot with collapse toggle and height memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BottomDockState {
    /// Ordered list of panels docked in the bottom slot.
    pub panels: Vec<PanelId>,
    /// Identifier of the currently active panel tab.
    pub active: Option<PanelId>,
    /// Whether the dock is collapsed to a thin header bar.
    pub collapsed: bool,
    /// Height in pixels when expanded.
    pub height: f32,
}

impl BottomDockState {
    /// Default height of the bottom dock in pixels.
    pub const DEFAULT_HEIGHT: f32 = 180.0;
    /// Collapsed header bar height in pixels.
    pub const COLLAPSED_HEIGHT: f32 = 28.0;

    /// Creates a new bottom dock state.
    pub fn new(panels: Vec<PanelId>, active: Option<PanelId>) -> Self {
        Self {
            panels,
            active,
            collapsed: false,
            height: Self::DEFAULT_HEIGHT,
        }
    }

    /// Selects a panel tab if present.
    pub fn set_active(&mut self, id: PanelId) {
        if self.panels.contains(&id) {
            self.active = Some(id);
        }
    }

    /// Adds a panel to the bottom slot and focuses it.
    pub fn add_panel(&mut self, id: PanelId) {
        if !self.panels.contains(&id) {
            self.panels.push(id.clone());
        }
        self.active = Some(id);
    }

    /// Removes a panel from the bottom slot.
    pub fn remove_panel(&mut self, id: &PanelId) {
        self.panels.retain(|p| p != id);
        if self.active.as_ref() == Some(id) {
            self.active = self.panels.first().cloned();
        }
    }

    /// Toggles the collapsed state.
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

/// Complete shell panel docking layout configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellDockLayout {
    /// Left dock slot (typically Project panel).
    pub left: DockSlotState,
    /// Right dock slot (typically Layers, Palette, Tools).
    pub right: DockSlotState,
    /// Bottom dock slot (typically Console).
    pub bottom: BottomDockState,
    /// Transient ID of the panel tab currently being dragged by the user, if any.
    #[serde(skip)]
    pub dragging_panel: Option<PanelId>,
}

impl Default for ShellDockLayout {
    fn default() -> Self {
        Self {
            left: DockSlotState::new(vec![PanelId::new("project")], Some(PanelId::new("project"))),
            right: DockSlotState::new(
                vec![
                    PanelId::new("layers"),
                    PanelId::new("tileset"),
                    PanelId::new("tools"),
                ],
                Some(PanelId::new("layers")),
            ),
            bottom: BottomDockState::new(
                vec![PanelId::new("console")],
                Some(PanelId::new("console")),
            ),
            dragging_panel: None,
        }
    }
}

impl ShellDockLayout {
    /// Begins dragging a panel tab.
    pub fn start_drag(&mut self, id: PanelId) {
        self.dragging_panel = Some(id);
    }

    /// Completes the drop of the currently dragged panel into `target`, returning true if moved.
    pub fn drop_dragged(&mut self, target: DockSlotLocation) -> bool {
        if let Some(id) = self.dragging_panel.take() {
            self.move_panel(&id, target);
            true
        } else {
            false
        }
    }

    /// Cancels dragging without moving.
    pub fn cancel_drag(&mut self) {
        self.dragging_panel = None;
    }

    /// Moves a panel from its current slot to the specified target slot.
    pub fn move_panel(&mut self, id: &PanelId, target: DockSlotLocation) {
        // Remove from existing locations
        self.left.remove_panel(id);
        self.right.remove_panel(id);
        self.bottom.remove_panel(id);

        // Add to destination slot
        match target {
            DockSlotLocation::Left => self.left.add_panel(id.clone()),
            DockSlotLocation::Right => self.right.add_panel(id.clone()),
            DockSlotLocation::Bottom => self.bottom.add_panel(id.clone()),
        }
    }

    /// Toggles bottom dock collapsed state.
    pub fn toggle_bottom_dock(&mut self) {
        self.bottom.toggle_collapsed();
    }

    /// Ensures a panel is registered in a dock slot if not already present.
    pub fn ensure_panel(&mut self, id: PanelId, default_loc: DockSlotLocation) {
        if !self.left.panels.contains(&id)
            && !self.right.panels.contains(&id)
            && !self.bottom.panels.contains(&id)
        {
            match default_loc {
                DockSlotLocation::Left => self.left.add_panel(id),
                DockSlotLocation::Right => self.right.add_panel(id),
                DockSlotLocation::Bottom => self.bottom.add_panel(id),
            }
        }
    }

    /// Persists layout state into egui context storage.
    pub fn save_to_memory(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.insert_persisted(egui::Id::new("shell_dock_layout"), self.clone());
        });
    }

    /// Loads persisted layout state from egui context storage, if present.
    pub fn load_from_memory(ctx: &egui::Context) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(egui::Id::new("shell_dock_layout")))
    }
}
