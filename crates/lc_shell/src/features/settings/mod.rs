//! Editor and project settings workspace (`AppMode::Settings`).

pub mod i18n;
pub mod keybindings;
pub mod sections;
pub mod shortcuts;
pub mod state;
pub mod theme;
pub mod view;

pub use state::{SettingsCategory, SettingsViewState};
pub use view::render_settings_view;
