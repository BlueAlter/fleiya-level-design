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

    /// Returns the index of a panel in this slot, if present.
    pub fn position(&self, id: &PanelId) -> Option<usize> {
        self.panels.iter().position(|panel| panel == id)
    }

    /// Inserts a panel at `index` and focuses it.
    ///
    /// If the panel already belongs to this slot it is removed first and the
    /// index is adjusted, so dropping a tab onto its own position is a no-op.
    pub fn insert_panel(&mut self, id: PanelId, index: usize) {
        let mut index = index;
        if let Some(current) = self.position(&id) {
            self.panels.remove(current);
            if current < index {
                index -= 1;
            }
        }

        let index = index.min(self.panels.len());
        self.panels.insert(index, id.clone());
        self.active = Some(id);
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
    /// Smallest useful expanded height in pixels.
    pub const MIN_EXPANDED_HEIGHT: f32 = 96.0;
    /// Maximum expanded height as a fraction of the central area.
    pub const MAX_HEIGHT_FRACTION: f32 = 0.55;

    /// Maximum expanded height for the available central area.
    pub fn max_expanded_height(available_height: f32) -> f32 {
        (available_height * Self::MAX_HEIGHT_FRACTION).max(Self::COLLAPSED_HEIGHT + 1.0)
    }

    /// Height to use when expanding, clamped to the available central area.
    pub fn desired_expanded_height(&self, available_height: f32) -> f32 {
        let max = Self::max_expanded_height(available_height);
        self.height.clamp(Self::MIN_EXPANDED_HEIGHT.min(max), max)
    }

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

    /// Returns the index of a panel in this slot, if present.
    pub fn position(&self, id: &PanelId) -> Option<usize> {
        self.panels.iter().position(|panel| panel == id)
    }

    /// Inserts a panel at `index` and focuses it.
    pub fn insert_panel(&mut self, id: PanelId, index: usize) {
        let mut index = index;
        if let Some(current) = self.position(&id) {
            self.panels.remove(current);
            if current < index {
                index -= 1;
            }
        }

        let index = index.min(self.panels.len());
        self.panels.insert(index, id.clone());
        self.active = Some(id);
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
            left: DockSlotState::new(
                vec![PanelId::new("project"), PanelId::new("layers")],
                Some(PanelId::new("project")),
            ),
            right: DockSlotState::new(
                vec![PanelId::new("inspector"), PanelId::new("tileset")],
                Some(PanelId::new("inspector")),
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
        self.drop_dragged_at(target, usize::MAX)
    }

    /// Completes the drop at a specific tab index in `target`.
    pub fn drop_dragged_at(&mut self, target: DockSlotLocation, index: usize) -> bool {
        if let Some(id) = self.dragging_panel.take() {
            self.move_panel_to(&id, target, index);
            if target == DockSlotLocation::Bottom {
                self.bottom.collapsed = false;
            }
            true
        } else {
            false
        }
    }

    /// Cancels dragging without moving.
    pub fn cancel_drag(&mut self) {
        self.dragging_panel = None;
    }

    /// Returns the slot currently containing `id`.
    pub fn slot_of(&self, id: &PanelId) -> Option<DockSlotLocation> {
        if self.left.panels.contains(id) {
            Some(DockSlotLocation::Left)
        } else if self.right.panels.contains(id) {
            Some(DockSlotLocation::Right)
        } else if self.bottom.panels.contains(id) {
            Some(DockSlotLocation::Bottom)
        } else {
            None
        }
    }

    /// Moves a panel from its current slot to the specified target slot.
    pub fn move_panel(&mut self, id: &PanelId, target: DockSlotLocation) {
        self.move_panel_to(id, target, usize::MAX);
    }

    /// Moves a panel to a specific tab index in the target slot.
    pub fn move_panel_to(&mut self, id: &PanelId, target: DockSlotLocation, index: usize) {
        let source = self.slot_of(id);
        if source != Some(target) {
            // Remove from every slot first; the target slot's insert handles
            // same-slot reordering without losing the source index.
            self.left.remove_panel(id);
            self.right.remove_panel(id);
            self.bottom.remove_panel(id);
        }

        match target {
            DockSlotLocation::Left => self.left.insert_panel(id.clone(), index),
            DockSlotLocation::Right => self.right.insert_panel(id.clone(), index),
            DockSlotLocation::Bottom => self.bottom.insert_panel(id.clone(), index),
        }
    }

    /// Toggles bottom dock collapsed state.
    pub fn toggle_bottom_dock(&mut self) {
        self.bottom.toggle_collapsed();
    }

    /// Ensures a panel is registered in a dock slot if not already present.
    pub fn ensure_panel(&mut self, id: PanelId, default_loc: DockSlotLocation) {
        if id.as_str() == "entities" {
            self.left.remove_panel(&id);
            self.right.remove_panel(&id);
            self.bottom.remove_panel(&id);
            return;
        }

        let mut loc = default_loc;
        if id.as_str() == "tileset" || id.as_str() == "tile_palette" || id.as_str() == "inspector" {
            loc = DockSlotLocation::Right;
        } else if id.as_str() == "layers" || id.as_str() == "project" {
            loc = DockSlotLocation::Left;
        }

        let is_present = self.left.panels.contains(&id)
            || self.right.panels.contains(&id)
            || self.bottom.panels.contains(&id)
            || (id.as_str() == "tile_palette"
                && (self.left.panels.contains(&PanelId::new("tileset"))
                    || self.right.panels.contains(&PanelId::new("tileset"))
                    || self.bottom.panels.contains(&PanelId::new("tileset"))))
            || (id.as_str() == "tileset"
                && (self.left.panels.contains(&PanelId::new("tile_palette"))
                    || self.right.panels.contains(&PanelId::new("tile_palette"))
                    || self.bottom.panels.contains(&PanelId::new("tile_palette"))));

        if !is_present {
            match loc {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(id: &str) -> PanelId {
        PanelId::new(id)
    }

    #[test]
    fn insert_panel_reorders_within_a_slot() {
        let mut slot = DockSlotState::new(
            vec![panel("project"), panel("console"), panel("tools")],
            Some(panel("project")),
        );

        slot.insert_panel(panel("console"), 0);
        assert_eq!(
            slot.panels,
            vec![panel("console"), panel("project"), panel("tools")]
        );

        slot.insert_panel(panel("project"), 2);
        assert_eq!(
            slot.panels,
            vec![panel("console"), panel("project"), panel("tools")]
        );
    }

    #[test]
    fn move_panel_to_index_across_slots() {
        let mut layout = ShellDockLayout::default();

        layout.move_panel_to(&panel("console"), DockSlotLocation::Left, 0);
        assert_eq!(
            layout.left.panels,
            vec![panel("console"), panel("project"), panel("layers")]
        );
        assert!(layout.bottom.panels.is_empty());

        layout.move_panel_to(&panel("project"), DockSlotLocation::Right, 1);
        assert_eq!(
            layout.right.panels,
            vec![panel("inspector"), panel("project"), panel("tileset")]
        );
        assert!(layout.left.panels.contains(&panel("console")));
    }

    #[test]
    fn moving_to_the_same_position_is_a_no_op() {
        let mut layout = ShellDockLayout::default();
        layout.left.insert_panel(panel("console"), 1);

        layout.move_panel_to(&panel("console"), DockSlotLocation::Left, 1);
        assert_eq!(
            layout.left.panels,
            vec![panel("project"), panel("console"), panel("layers")]
        );

        layout.move_panel_to(&panel("console"), DockSlotLocation::Left, 3);
        assert_eq!(
            layout.left.panels,
            vec![panel("project"), panel("layers"), panel("console")]
        );
    }

    #[test]
    fn bottom_dock_height_is_clamped_to_available_space() {
        let mut bottom = BottomDockState::new(vec![panel("console")], Some(panel("console")));

        assert_eq!(bottom.desired_expanded_height(800.0), 180.0);

        bottom.height = 1_000.0;
        assert_eq!(
            bottom.desired_expanded_height(800.0),
            BottomDockState::max_expanded_height(800.0)
        );

        bottom.height = 10.0;
        assert_eq!(
            bottom.desired_expanded_height(800.0),
            BottomDockState::MIN_EXPANDED_HEIGHT
        );

        assert_eq!(bottom.desired_expanded_height(100.0), 55.0);
    }
}
