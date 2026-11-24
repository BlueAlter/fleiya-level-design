//! Standardized row kebab context menu widget with toggle and right-click support.
//!
//! Provides the "⋮" kebab button with toggle behavior (opens right-aligned to the button,
//! closes on a second click or outside click) plus optional right-click on the row.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::row_menu::{row_menu, RowMenuItem};
//!
//! let items = vec![
//!     RowMenuItem::new(1, "Duplicate"),
//!     RowMenuItem::danger(2, "Delete"),
//! ];
//! if let Some(action_id) = row_menu(ui, "item_kebab", Some(&row_resp), &items) {
//!     // handle action
//! }
//! ```

use egui::{Id, Response, Ui};
use lc_canvas::icons::Icon;

use super::context_menu::{is_menu_open, show_context_menu, MenuAnchor, MenuItem};
use super::icon_button::{icon_button, IconButtonStyle};
use super::workspace_metrics::ICON_BUTTON_SIZE_COMPACT;

/// An item in a [`row_menu`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowMenuItem<K: Clone + PartialEq> {
    /// Unique identifier for this menu option.
    pub id: K,
    /// Human-readable label displayed in the menu.
    pub label: String,
    /// Whether this action is destructive (highlighted in red).
    pub danger: bool,
    /// Whether this action is enabled and clickable.
    pub enabled: bool,
    /// Optional custom icon glyph.
    pub icon: Option<Icon>,
}

impl<K: Clone + PartialEq> RowMenuItem<K> {
    /// Creates a normal action menu item.
    pub fn new(id: K, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            danger: false,
            enabled: true,
            icon: None,
        }
    }

    /// Creates a destructive action menu item (styled red on hover).
    pub fn danger(id: K, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            danger: true,
            enabled: true,
            icon: None,
        }
    }

    /// Assigns a custom icon to this menu item.
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets whether this menu item is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Converts a slice of [`RowMenuItem`] into filtered [`MenuItem`] for display.
pub fn filter_menu_items<K: Clone + PartialEq>(items: &[RowMenuItem<K>]) -> Vec<MenuItem<K>> {
    items
        .iter()
        .filter(|item| item.enabled)
        .map(|item| {
            let icon = item.icon.unwrap_or(if item.danger {
                Icon::Delete
            } else {
                Icon::DotsThree
            });
            if item.danger {
                MenuItem::destructive(item.id.clone(), &item.label, icon)
            } else {
                MenuItem::action(item.id.clone(), &item.label, icon)
            }
        })
        .collect()
}

/// Renders a kebab button and attaches toggle menu and right-click handler.
///
/// Returns `(kebab_button_response, selected_item_id)`.
pub fn row_menu_button<K: Clone + PartialEq + 'static>(
    ui: &mut Ui,
    id_source: impl egui::AsId,
    row_response: Option<&Response>,
    items: &[RowMenuItem<K>],
) -> (Response, Option<K>) {
    let popup_id = Id::new(id_source).with("kebab_popup");
    let menu_open = is_menu_open(ui.ctx(), popup_id);

    let kebab_resp = icon_button(
        ui,
        Icon::DotsThreeVertical,
        ICON_BUTTON_SIZE_COMPACT,
        if menu_open {
            IconButtonStyle::bordered().selected(true)
        } else {
            IconButtonStyle::ghost()
        },
    )
    .on_hover_text("More actions");

    let active_items = filter_menu_items(items);
    let mut selected_id = None;

    // 1. Kebab button toggle menu (right-aligned to the button)
    if let Some(act) = show_context_menu(
        ui,
        MenuAnchor::button_with_id(&kebab_resp, popup_id),
        &active_items,
    ) {
        selected_id = Some(act);
    }

    // 2. Right click on row at pointer
    if let Some(row_resp) = row_response {
        let row_context_id = popup_id.with("row_context");
        if let Some(act) = show_context_menu(
            ui,
            MenuAnchor::context_menu_with_id(row_resp, row_context_id),
            &active_items,
        ) {
            selected_id = Some(act);
        }
    }

    (kebab_resp, selected_id)
}

/// Renders the row menu and returns the selected action ID, if triggered this frame.
pub fn row_menu<K: Clone + PartialEq + 'static>(
    ui: &mut Ui,
    id_source: impl egui::AsId,
    row_response: Option<&Response>,
    items: &[RowMenuItem<K>],
) -> Option<K> {
    let (_btn_resp, choice) = row_menu_button(ui, id_source, row_response, items);
    choice
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_filter_menu_items_filtering_and_danger() {
        let items = vec![
            RowMenuItem::new(1, "Edit"),
            RowMenuItem::new(2, "Hidden").with_enabled(false),
            RowMenuItem::danger(3, "Delete"),
        ];

        let filtered = filter_menu_items(&items);
        assert_eq!(filtered.len(), 2);

        match &filtered[0] {
            MenuItem::Action {
                id,
                label,
                destructive,
                ..
            } => {
                assert_eq!(*id, 1);
                assert_eq!(label, "Edit");
                assert!(!destructive);
            }
            _ => panic!("Expected action"),
        }

        match &filtered[1] {
            MenuItem::Action {
                id,
                label,
                destructive,
                ..
            } => {
                assert_eq!(*id, 3);
                assert_eq!(label, "Delete");
                assert!(destructive);
            }
            _ => panic!("Expected action"),
        }
    }

    #[test]
    fn test_row_menu_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let items = vec![
            RowMenuItem::new("clone", "Clone"),
            RowMenuItem::danger("del", "Delete"),
        ];

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let choice = row_menu(ui, "test_row_menu", None, &items);
            assert_eq!(choice, None);
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "row_menu should produce render shapes for kebab button"
        );
    }
}
