//! Shared, feature-agnostic services and UI widgets.
//!
//! Everything here is owned by the `shared/ui-kit` branch: it must not depend
//! on any specific editor feature (enum, layer, tileset, ...).

pub mod assets;
pub mod file_dialog;
pub mod i18n;
pub mod ui;
