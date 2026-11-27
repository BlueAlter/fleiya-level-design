//! Serializable keyboard shortcut actions, bindings, and conflict detection.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Global actions that can be bound to keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    /// Save the active project.
    SaveProject,
    /// Undo the last command.
    Undo,
    /// Redo the last undone command (primary).
    Redo,
    /// Redo the last undone command (secondary).
    RedoAlt,
    /// Toggle the bottom dock panel.
    ToggleBottomDock,
    /// Switch to Editor workspace.
    WorkspaceEditor,
    /// Switch to Entities workspace.
    WorkspaceEntities,
    /// Switch to Tilesets workspace.
    WorkspaceTilesets,
    /// Switch to Enums workspace.
    WorkspaceEnums,
    /// Switch to Settings workspace.
    WorkspaceSettings,
    /// Open Settings workspace (alternate shortcut).
    OpenSettingsAlt,
}

impl ShortcutAction {
    /// All customizable shortcut actions in display order.
    pub const ALL: [Self; 11] = [
        Self::SaveProject,
        Self::Undo,
        Self::Redo,
        Self::RedoAlt,
        Self::ToggleBottomDock,
        Self::WorkspaceEditor,
        Self::WorkspaceEntities,
        Self::WorkspaceTilesets,
        Self::WorkspaceEnums,
        Self::WorkspaceSettings,
        Self::OpenSettingsAlt,
    ];

    /// Human-readable label shown in the shortcut table and menus.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SaveProject => "Save Project",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::RedoAlt => "Redo (Alternate)",
            Self::ToggleBottomDock => "Toggle Bottom Dock",
            Self::WorkspaceEditor => "Editor Workspace",
            Self::WorkspaceEntities => "Entities Workspace",
            Self::WorkspaceTilesets => "Tilesets Workspace",
            Self::WorkspaceEnums => "Enums Workspace",
            Self::WorkspaceSettings => "Settings Workspace",
            Self::OpenSettingsAlt => "Open Settings (Alternate)",
        }
    }

    /// Short description of what the shortcut triggers.
    pub const fn description(self) -> &'static str {
        match self {
            Self::SaveProject => "Save the active project to disk.",
            Self::Undo => "Revert the most recent command.",
            Self::Redo => "Re-apply the most recently undone command.",
            Self::RedoAlt => "Alternative shortcut to re-apply an undone command.",
            Self::ToggleBottomDock => "Collapse or expand the bottom dock in Editor mode.",
            Self::WorkspaceEditor => "Switch to the level and world Editor workspace.",
            Self::WorkspaceEntities => "Switch to the Entity Definitions workspace.",
            Self::WorkspaceTilesets => "Switch to the Tilesets workspace.",
            Self::WorkspaceEnums => "Switch to the Enum Definitions workspace.",
            Self::WorkspaceSettings => "Switch to the Settings workspace.",
            Self::OpenSettingsAlt => "Alternative shortcut to open Settings.",
        }
    }
}

/// Serializable keyboard key identifier independent of UI toolkit types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum ShortcutKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Backtick,
    Comma,
    Period,
    Minus,
    Plus,
    Equals,
    Slash,
    Backslash,
    Semicolon,
    Quote,
    OpenBracket,
    CloseBracket,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

impl ShortcutKey {
    /// Display name of the key for shortcut badges.
    pub const fn label(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Num0 => "0",
            Self::Num1 => "1",
            Self::Num2 => "2",
            Self::Num3 => "3",
            Self::Num4 => "4",
            Self::Num5 => "5",
            Self::Num6 => "6",
            Self::Num7 => "7",
            Self::Num8 => "8",
            Self::Num9 => "9",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::Backtick => "`",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Minus => "-",
            Self::Plus => "+",
            Self::Equals => "=",
            Self::Slash => "/",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Tab => "Tab",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::Insert => "Insert",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::ArrowUp => "Up",
            Self::ArrowDown => "Down",
            Self::ArrowLeft => "Left",
            Self::ArrowRight => "Right",
        }
    }
}

/// A modifier + key combination bound to an editor action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShortcutBinding {
    /// Primary key.
    pub key: ShortcutKey,
    /// Control modifier required.
    pub ctrl: bool,
    /// Shift modifier required.
    pub shift: bool,
    /// Alt modifier required.
    pub alt: bool,
}

impl ShortcutBinding {
    /// Creates a new shortcut binding.
    pub const fn new(key: ShortcutKey, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
        }
    }

    /// Creates a `Ctrl+<key>` shortcut binding.
    pub const fn ctrl(key: ShortcutKey) -> Self {
        Self::new(key, true, false, false)
    }

    /// Number of active modifiers (used to match more specific shortcuts first).
    pub const fn modifier_count(self) -> u8 {
        (self.ctrl as u8) + (self.shift as u8) + (self.alt as u8)
    }

    /// Formats the binding for display (e.g. `"Ctrl+Shift+Z"`).
    pub fn display_label(self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        parts.push(self.key.label());
        parts.join("+")
    }
}

/// Table of customizable keyboard shortcuts with conflict detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutMap {
    /// Active keybindings keyed by action.
    pub bindings: BTreeMap<ShortcutAction, ShortcutBinding>,
}

impl Default for ShortcutMap {
    fn default() -> Self {
        let mut bindings = BTreeMap::new();
        for action in ShortcutAction::ALL {
            bindings.insert(action, Self::default_for(action));
        }
        Self { bindings }
    }
}

impl ShortcutMap {
    /// Returns the built-in default binding for an action.
    pub const fn default_for(action: ShortcutAction) -> ShortcutBinding {
        match action {
            ShortcutAction::SaveProject => ShortcutBinding::ctrl(ShortcutKey::S),
            ShortcutAction::Undo => ShortcutBinding::ctrl(ShortcutKey::Z),
            ShortcutAction::Redo => ShortcutBinding::ctrl(ShortcutKey::Y),
            ShortcutAction::RedoAlt => ShortcutBinding::new(ShortcutKey::Z, true, true, false),
            ShortcutAction::ToggleBottomDock => ShortcutBinding::ctrl(ShortcutKey::Backtick),
            ShortcutAction::WorkspaceEditor => ShortcutBinding::ctrl(ShortcutKey::Num1),
            ShortcutAction::WorkspaceEntities => ShortcutBinding::ctrl(ShortcutKey::Num3),
            ShortcutAction::WorkspaceTilesets => ShortcutBinding::ctrl(ShortcutKey::Num4),
            ShortcutAction::WorkspaceEnums => ShortcutBinding::ctrl(ShortcutKey::Num5),
            ShortcutAction::WorkspaceSettings => ShortcutBinding::ctrl(ShortcutKey::Num6),
            ShortcutAction::OpenSettingsAlt => ShortcutBinding::ctrl(ShortcutKey::Comma),
        }
    }

    /// Returns the effective binding for an action (falling back to default).
    pub fn get(&self, action: ShortcutAction) -> ShortcutBinding {
        self.bindings
            .get(&action)
            .copied()
            .unwrap_or_else(|| Self::default_for(action))
    }

    /// Updates the binding for an action.
    pub fn set(&mut self, action: ShortcutAction, binding: ShortcutBinding) {
        self.bindings.insert(action, binding);
    }

    /// Returns any other actions currently bound to the same shortcut.
    pub fn conflicts_for(
        &self,
        action: ShortcutAction,
        binding: ShortcutBinding,
    ) -> Vec<ShortcutAction> {
        ShortcutAction::ALL
            .iter()
            .copied()
            .filter(|&other| other != action && self.get(other) == binding)
            .collect()
    }

    /// Returns all currently conflicting action pairs in the map.
    pub fn all_conflicts(&self) -> Vec<(ShortcutAction, Vec<ShortcutAction>)> {
        ShortcutAction::ALL
            .iter()
            .copied()
            .filter_map(|action| {
                let conflicts = self.conflicts_for(action, self.get(action));
                if conflicts.is_empty() {
                    None
                } else {
                    Some((action, conflicts))
                }
            })
            .collect()
    }

    /// Resets one action back to its built-in default.
    pub fn reset_action(&mut self, action: ShortcutAction) {
        self.bindings.insert(action, Self::default_for(action));
    }

    /// Resets every action back to its built-in default.
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }
}
