//! Standardized text input field with commitment tracking and identifier validation.
//!
//! Provides [`text_field`] for single-line text editing with explicit focus/Enter
//! commitment and Escape cancellation events, along with [`validate_identifier`]
//! for enforcing project naming rules.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::text_field::{text_field, FieldStyle};
//!
//! let res = text_field(ui, &mut my_string, FieldStyle::default());
//! if res.committed {
//!     // handle Enter or focus loss
//! }
//! ```

use egui::{vec2, Color32, CornerRadius, Key, Response, Stroke, StrokeKind, TextEdit, Ui};
use lc_canvas::theme::Tokens;

use super::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Outline style presets for [`text_field`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FieldOutline {
    /// No outline border drawn in normal state.
    #[default]
    None,
    /// 1px hairline border in normal state.
    Hairline,
    /// 1.5px colored border in normal state (e.g. for color-coded enum badges/rows).
    Color(Color32),
}

/// Visual and formatting configuration for a [`text_field`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldStyle<'a> {
    /// Outline border preset in normal state.
    pub outline: FieldOutline,
    /// Placeholder hint displayed when the field is empty.
    pub hint: Option<&'a str>,
    /// Optional error message; draws a red border and attaches a hover tooltip.
    pub error: Option<&'a str>,
    /// Whether to render text in monospace font.
    pub monospace: bool,
}

impl<'a> FieldStyle<'a> {
    /// Sets placeholder hint text.
    pub fn with_hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Sets an error message to highlight the field as invalid.
    pub fn with_error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Sets whether the field should render with a monospace font.
    pub fn with_monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }

    /// Sets whether to draw an outline border (backward compatibility: true = Hairline, false = None).
    pub fn with_outline(mut self, outline: bool) -> Self {
        self.outline = if outline {
            FieldOutline::Hairline
        } else {
            FieldOutline::None
        };
        self
    }

    /// Sets a colored outline border (1.5 px).
    pub fn with_outline_color(mut self, color: Color32) -> Self {
        self.outline = FieldOutline::Color(color);
        self
    }

    /// Sets the outline preset.
    pub fn with_outline_preset(mut self, outline: FieldOutline) -> Self {
        self.outline = outline;
        self
    }
}

/// Interaction result returned by [`text_field`].
#[derive(Clone, Debug)]
pub struct FieldResponse {
    /// Underlying egui widget response.
    pub response: Response,
    /// Whether the text content was modified this frame.
    pub changed: bool,
    /// Whether editing was committed (Enter pressed or focus lost without Escape).
    pub committed: bool,
    /// Whether editing was cancelled via Escape key.
    pub cancelled: bool,
}

/// Errors detected when validating an identifier name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentifierError {
    /// The provided identifier is empty or whitespace-only.
    Empty,
    /// The identifier contains characters outside `[a-zA-Z0-9_]` or begins with a digit.
    InvalidCharacters,
    /// Another definition already uses this identifier (case-insensitive).
    Duplicate,
}

impl std::fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Identifier cannot be empty"),
            Self::InvalidCharacters => write!(
                f,
                "Identifier must start with a letter or underscore and contain only alphanumeric characters"
            ),
            Self::Duplicate => write!(f, "An item with this identifier already exists"),
        }
    }
}

impl std::error::Error for IdentifierError {}

impl IdentifierError {
    /// Localization message key for this validation error.
    pub fn message_key(&self) -> &'static str {
        match self {
            Self::Empty => "errors.identifier.empty",
            Self::InvalidCharacters => "errors.identifier.invalid_chars",
            Self::Duplicate => "errors.identifier.duplicate",
        }
    }
}

/// Validates an identifier string against standard naming constraints and uniqueness.
///
/// Ensures the identifier is non-empty, contains only valid identifier characters
/// (`[a-zA-Z_][a-zA-Z0-9_]*`), and does not duplicate existing identifiers
/// (case-insensitive comparison, allowing matching `current`).
pub fn validate_identifier(
    name: &str,
    existing: &[impl AsRef<str>],
    current: Option<&str>,
) -> Result<(), IdentifierError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(IdentifierError::Empty);
    }

    // Must start with an ASCII letter or underscore
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap_or('_');
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(IdentifierError::InvalidCharacters);
    }

    // Remaining characters must be ASCII alphanumeric or underscore
    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(IdentifierError::InvalidCharacters);
        }
    }

    // Check duplicate case-insensitively, allowing self-match
    for exist in existing {
        let exist_str = exist.as_ref();
        if let Some(cur) = current {
            if exist_str.eq_ignore_ascii_case(cur) {
                continue;
            }
        }
        if exist_str.eq_ignore_ascii_case(trimmed) {
            return Err(IdentifierError::Duplicate);
        }
    }

    Ok(())
}

/// Renders a single-line text input field styled with project tokens and explicit event tracking.
pub fn text_field(ui: &mut Ui, text: &mut String, style: FieldStyle<'_>) -> FieldResponse {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width();
    let mut edit = TextEdit::singleline(text)
        .desired_width(f32::INFINITY)
        .vertical_align(egui::Align::Center)
        .frame(
            egui::Frame::new()
                .fill(t.sunken)
                .corner_radius(CornerRadius::same(3))
                .inner_margin(egui::Margin::symmetric(8, 0)),
        )
        .min_size(vec2(width, CONTROL_HEIGHT_FORM));

    if style.monospace {
        edit = edit.font(egui::TextStyle::Monospace);
    }
    if let Some(hint) = style.hint {
        edit = edit.hint_text(hint);
    }

    let response = ui.add_sized([width, CONTROL_HEIGHT_FORM], edit);

    let changed = response.changed();
    let esc_pressed = response.has_focus() && ui.input(|i| i.key_pressed(Key::Escape));
    let enter_pressed = response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let lost_focus = response.lost_focus();

    let cancelled = esc_pressed;
    let committed = (lost_focus && !cancelled) || enter_pressed;

    if cancelled {
        response.surrender_focus();
    }

    let stroke = if style.error.is_some() {
        Some(Stroke::new(1.0, t.bad))
    } else if response.has_focus() {
        Some(Stroke::new(1.0, t.accent))
    } else {
        match style.outline {
            FieldOutline::None => None,
            FieldOutline::Hairline => Some(t.hairline()),
            FieldOutline::Color(color) => Some(Stroke::new(1.5, color)),
        }
    };

    if let Some(stroke) = stroke {
        ui.painter().rect_stroke(
            response.rect,
            CornerRadius::same(3),
            stroke,
            StrokeKind::Inside,
        );
    }

    if let Some(err_msg) = style.error {
        response.clone().on_hover_text(err_msg);
    }

    FieldResponse {
        response,
        changed,
        committed,
        cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_validate_identifier_empty() {
        let existing = ["Alpha", "Beta"];
        assert_eq!(
            validate_identifier("", &existing, None),
            Err(IdentifierError::Empty)
        );
        assert_eq!(
            validate_identifier("   ", &existing, None),
            Err(IdentifierError::Empty)
        );
    }

    #[test]
    fn test_validate_identifier_invalid_chars() {
        let existing = ["Alpha", "Beta"];
        assert_eq!(
            validate_identifier("123abc", &existing, None),
            Err(IdentifierError::InvalidCharacters)
        );
        assert_eq!(
            validate_identifier("hello-world", &existing, None),
            Err(IdentifierError::InvalidCharacters)
        );
        assert_eq!(
            validate_identifier("foo bar", &existing, None),
            Err(IdentifierError::InvalidCharacters)
        );
        assert_eq!(
            validate_identifier("test!", &existing, None),
            Err(IdentifierError::InvalidCharacters)
        );
    }

    #[test]
    fn test_validate_identifier_duplicate() {
        let existing = ["Alpha", "Beta"];
        assert_eq!(
            validate_identifier("alpha", &existing, None),
            Err(IdentifierError::Duplicate)
        );
        assert_eq!(
            validate_identifier("BETA", &existing, None),
            Err(IdentifierError::Duplicate)
        );
        // Self-rename allowed when current matches
        assert_eq!(
            validate_identifier("Alpha", &existing, Some("Alpha")),
            Ok(())
        );
        assert_eq!(
            validate_identifier("alpha", &existing, Some("Alpha")),
            Ok(())
        );
    }

    #[test]
    fn test_validate_identifier_valid() {
        let existing = ["Alpha", "Beta"];
        assert_eq!(validate_identifier("Gamma", &existing, None), Ok(()));
        assert_eq!(validate_identifier("_Internal", &existing, None), Ok(()));
        assert_eq!(validate_identifier("Player_123", &existing, None), Ok(()));
    }

    #[test]
    fn test_identifier_error_message_keys() {
        assert_eq!(
            IdentifierError::Empty.message_key(),
            "errors.identifier.empty"
        );
        assert_eq!(
            IdentifierError::InvalidCharacters.message_key(),
            "errors.identifier.invalid_chars"
        );
        assert_eq!(
            IdentifierError::Duplicate.message_key(),
            "errors.identifier.duplicate"
        );
    }

    #[test]
    fn test_text_field_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let mut val = "ItemName".to_string();
        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let res = text_field(
                ui,
                &mut val,
                FieldStyle::default()
                    .with_hint("Enter name...")
                    .with_outline(true),
            );
            assert!(!res.changed);
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "text_field should produce render shapes"
        );
    }
}
