//! Localization facade for the shell.
//!
//! The lookup, the fallback chain and the selected language live in
//! [`lc_ui_api::i18n`] because feature crates need the same functions; the
//! shell re-exports them here so shared and feature-facing UI code has one
//! import path. Domain keys are **not** re-exported: each feature crate owns
//! its keys and registers its catalog through
//! [`lc_ui_api::i18n::register_catalog`].

pub use lc_ui_api::i18n::{
    contains_key, doc, doc_for, keys as common_keys, language, register_catalog, set_language, tr,
    tr_for, CatalogFile, CatalogKind, DEFAULT_LANGUAGE,
};