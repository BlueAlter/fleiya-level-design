//! Conversions and interactive key-capture helpers between `lc_io` shortcuts and `egui`.

use egui::{Event, Key, KeyboardShortcut, Modifiers};
use lc_io::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutMap};

/// Outcome of polling `egui` input while recording a shortcut rebind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCapture {
    /// User pressed a valid key combination.
    Captured(ShortcutBinding),
    /// User pressed bare `Escape` to cancel recording.
    Cancelled,
}

/// Converts a serializable [`ShortcutKey`] into an [`egui::Key`].
pub const fn to_egui_key(key: ShortcutKey) -> Key {
    match key {
        ShortcutKey::A => Key::A,
        ShortcutKey::B => Key::B,
        ShortcutKey::C => Key::C,
        ShortcutKey::D => Key::D,
        ShortcutKey::E => Key::E,
        ShortcutKey::F => Key::F,
        ShortcutKey::G => Key::G,
        ShortcutKey::H => Key::H,
        ShortcutKey::I => Key::I,
        ShortcutKey::J => Key::J,
        ShortcutKey::K => Key::K,
        ShortcutKey::L => Key::L,
        ShortcutKey::M => Key::M,
        ShortcutKey::N => Key::N,
        ShortcutKey::O => Key::O,
        ShortcutKey::P => Key::P,
        ShortcutKey::Q => Key::Q,
        ShortcutKey::R => Key::R,
        ShortcutKey::S => Key::S,
        ShortcutKey::T => Key::T,
        ShortcutKey::U => Key::U,
        ShortcutKey::V => Key::V,
        ShortcutKey::W => Key::W,
        ShortcutKey::X => Key::X,
        ShortcutKey::Y => Key::Y,
        ShortcutKey::Z => Key::Z,
        ShortcutKey::Num0 => Key::Num0,
        ShortcutKey::Num1 => Key::Num1,
        ShortcutKey::Num2 => Key::Num2,
        ShortcutKey::Num3 => Key::Num3,
        ShortcutKey::Num4 => Key::Num4,
        ShortcutKey::Num5 => Key::Num5,
        ShortcutKey::Num6 => Key::Num6,
        ShortcutKey::Num7 => Key::Num7,
        ShortcutKey::Num8 => Key::Num8,
        ShortcutKey::Num9 => Key::Num9,
        ShortcutKey::F1 => Key::F1,
        ShortcutKey::F2 => Key::F2,
        ShortcutKey::F3 => Key::F3,
        ShortcutKey::F4 => Key::F4,
        ShortcutKey::F5 => Key::F5,
        ShortcutKey::F6 => Key::F6,
        ShortcutKey::F7 => Key::F7,
        ShortcutKey::F8 => Key::F8,
        ShortcutKey::F9 => Key::F9,
        ShortcutKey::F10 => Key::F10,
        ShortcutKey::F11 => Key::F11,
        ShortcutKey::F12 => Key::F12,
        ShortcutKey::Backtick => Key::Backtick,
        ShortcutKey::Comma => Key::Comma,
        ShortcutKey::Period => Key::Period,
        ShortcutKey::Minus => Key::Minus,
        ShortcutKey::Plus => Key::Plus,
        ShortcutKey::Equals => Key::Equals,
        ShortcutKey::Slash => Key::Slash,
        ShortcutKey::Backslash => Key::Backslash,
        ShortcutKey::Semicolon => Key::Semicolon,
        ShortcutKey::Quote => Key::Quote,
        ShortcutKey::OpenBracket => Key::OpenBracket,
        ShortcutKey::CloseBracket => Key::CloseBracket,
        ShortcutKey::Space => Key::Space,
        ShortcutKey::Enter => Key::Enter,
        ShortcutKey::Tab => Key::Tab,
        ShortcutKey::Backspace => Key::Backspace,
        ShortcutKey::Delete => Key::Delete,
        ShortcutKey::Insert => Key::Insert,
        ShortcutKey::Home => Key::Home,
        ShortcutKey::End => Key::End,
        ShortcutKey::PageUp => Key::PageUp,
        ShortcutKey::PageDown => Key::PageDown,
        ShortcutKey::ArrowUp => Key::ArrowUp,
        ShortcutKey::ArrowDown => Key::ArrowDown,
        ShortcutKey::ArrowLeft => Key::ArrowLeft,
        ShortcutKey::ArrowRight => Key::ArrowRight,
    }
}

/// Converts an [`egui::Key`] into a [`ShortcutKey`] if supported by the binding schema.
pub const fn from_egui_key(key: Key) -> Option<ShortcutKey> {
    match key {
        Key::A => Some(ShortcutKey::A),
        Key::B => Some(ShortcutKey::B),
        Key::C => Some(ShortcutKey::C),
        Key::D => Some(ShortcutKey::D),
        Key::E => Some(ShortcutKey::E),
        Key::F => Some(ShortcutKey::F),
        Key::G => Some(ShortcutKey::G),
        Key::H => Some(ShortcutKey::H),
        Key::I => Some(ShortcutKey::I),
        Key::J => Some(ShortcutKey::J),
        Key::K => Some(ShortcutKey::K),
        Key::L => Some(ShortcutKey::L),
        Key::M => Some(ShortcutKey::M),
        Key::N => Some(ShortcutKey::N),
        Key::O => Some(ShortcutKey::O),
        Key::P => Some(ShortcutKey::P),
        Key::Q => Some(ShortcutKey::Q),
        Key::R => Some(ShortcutKey::R),
        Key::S => Some(ShortcutKey::S),
        Key::T => Some(ShortcutKey::T),
        Key::U => Some(ShortcutKey::U),
        Key::V => Some(ShortcutKey::V),
        Key::W => Some(ShortcutKey::W),
        Key::X => Some(ShortcutKey::X),
        Key::Y => Some(ShortcutKey::Y),
        Key::Z => Some(ShortcutKey::Z),
        Key::Num0 => Some(ShortcutKey::Num0),
        Key::Num1 => Some(ShortcutKey::Num1),
        Key::Num2 => Some(ShortcutKey::Num2),
        Key::Num3 => Some(ShortcutKey::Num3),
        Key::Num4 => Some(ShortcutKey::Num4),
        Key::Num5 => Some(ShortcutKey::Num5),
        Key::Num6 => Some(ShortcutKey::Num6),
        Key::Num7 => Some(ShortcutKey::Num7),
        Key::Num8 => Some(ShortcutKey::Num8),
        Key::Num9 => Some(ShortcutKey::Num9),
        Key::F1 => Some(ShortcutKey::F1),
        Key::F2 => Some(ShortcutKey::F2),
        Key::F3 => Some(ShortcutKey::F3),
        Key::F4 => Some(ShortcutKey::F4),
        Key::F5 => Some(ShortcutKey::F5),
        Key::F6 => Some(ShortcutKey::F6),
        Key::F7 => Some(ShortcutKey::F7),
        Key::F8 => Some(ShortcutKey::F8),
        Key::F9 => Some(ShortcutKey::F9),
        Key::F10 => Some(ShortcutKey::F10),
        Key::F11 => Some(ShortcutKey::F11),
        Key::F12 => Some(ShortcutKey::F12),
        Key::Backtick => Some(ShortcutKey::Backtick),
        Key::Comma => Some(ShortcutKey::Comma),
        Key::Period => Some(ShortcutKey::Period),
        Key::Minus => Some(ShortcutKey::Minus),
        Key::Plus => Some(ShortcutKey::Plus),
        Key::Equals => Some(ShortcutKey::Equals),
        Key::Slash => Some(ShortcutKey::Slash),
        Key::Backslash => Some(ShortcutKey::Backslash),
        Key::Semicolon => Some(ShortcutKey::Semicolon),
        Key::Quote => Some(ShortcutKey::Quote),
        Key::OpenBracket => Some(ShortcutKey::OpenBracket),
        Key::CloseBracket => Some(ShortcutKey::CloseBracket),
        Key::Space => Some(ShortcutKey::Space),
        Key::Enter => Some(ShortcutKey::Enter),
        Key::Tab => Some(ShortcutKey::Tab),
        Key::Backspace => Some(ShortcutKey::Backspace),
        Key::Delete => Some(ShortcutKey::Delete),
        Key::Insert => Some(ShortcutKey::Insert),
        Key::Home => Some(ShortcutKey::Home),
        Key::End => Some(ShortcutKey::End),
        Key::PageUp => Some(ShortcutKey::PageUp),
        Key::PageDown => Some(ShortcutKey::PageDown),
        Key::ArrowUp => Some(ShortcutKey::ArrowUp),
        Key::ArrowDown => Some(ShortcutKey::ArrowDown),
        Key::ArrowLeft => Some(ShortcutKey::ArrowLeft),
        Key::ArrowRight => Some(ShortcutKey::ArrowRight),
        _ => None,
    }
}

/// Converts a [`ShortcutBinding`] into an [`egui::KeyboardShortcut`].
pub fn to_egui_shortcut(binding: ShortcutBinding) -> KeyboardShortcut {
    let mut modifiers = Modifiers::NONE;
    if binding.ctrl {
        modifiers = modifiers | Modifiers::CTRL;
    }
    if binding.shift {
        modifiers = modifiers | Modifiers::SHIFT;
    }
    if binding.alt {
        modifiers = modifiers | Modifiers::ALT;
    }
    KeyboardShortcut::new(modifiers, to_egui_key(binding.key))
}

/// Returns shortcut actions sorted from most specific (most modifiers) to least specific.
///
/// `egui::InputState::consume_shortcut` uses `Modifiers::matches_logically`, which ignores extra
/// Shift/Alt modifiers; checking more specific shortcuts first avoids `Ctrl+Z` consuming `Ctrl+Shift+Z`.
pub fn actions_in_dispatch_order(map: &ShortcutMap) -> Vec<(ShortcutAction, ShortcutBinding)> {
    let mut list: Vec<(ShortcutAction, ShortcutBinding)> = ShortcutAction::ALL
        .iter()
        .copied()
        .map(|action| (action, map.get(action)))
        .collect();
    list.sort_by(|a, b| {
        b.1.modifier_count()
            .cmp(&a.1.modifier_count())
            .then_with(|| a.0.cmp(&b.0))
    });
    list
}

/// Polls `ctx` for a non-modifier key press to capture a new shortcut binding.
pub fn capture_shortcut_from_ctx(ctx: &egui::Context) -> Option<ShortcutCapture> {
    ctx.input(|i| {
        for ev in &i.events {
            if let Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = ev
            {
                if *key == Key::Escape && !modifiers.ctrl && !modifiers.shift && !modifiers.alt {
                    return Some(ShortcutCapture::Cancelled);
                }
                if let Some(sk) = from_egui_key(*key) {
                    return Some(ShortcutCapture::Captured(ShortcutBinding::new(
                        sk,
                        modifiers.ctrl || modifiers.command || modifiers.mac_cmd,
                        modifiers.shift,
                        modifiers.alt,
                    )));
                }
            }
        }
        None
    })
}
