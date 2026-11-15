//! Layout metrics for the Enum workspace.
//!
//! Note: These metrics are planned to move to `shared` in a future cleanup
//! across all workspace features.

/// Padding on all four sides of both panels (left panel and detail panel).
pub const WORKSPACE_PADDING: f32 = 12.0;

/// Fixed width of the left enum list panel.
pub const LEFT_PANEL_WIDTH: f32 = 260.0;

/// Compact control height (left toolbar, search, sub_tabs, footer buttons).
pub const CONTROL_HEIGHT_COMPACT: f32 = 28.0;

/// Form control height (card fields, combos, tag chips, values rows, "+ Add Value").
pub const CONTROL_HEIGHT_FORM: f32 = 32.0;

/// Width of the card label column in the enum definition card.
pub const FORM_LABEL_COL_W: f32 = 110.0;

/// Exact square size of compact icon buttons (e.g. kebab 28x28).
pub const ICON_BUTTON_SIZE_COMPACT: f32 = CONTROL_HEIGHT_COMPACT;

/// Exact square size of form icon buttons (export 32x32, tags "+" and "⌄" 32x32, trash 32x32).
pub const ICON_BUTTON_SIZE_FORM: f32 = CONTROL_HEIGHT_FORM;
