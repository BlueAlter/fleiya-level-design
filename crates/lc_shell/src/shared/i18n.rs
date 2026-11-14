//! Localization facade for the shell.
//!
//! The tables, the fallback chain and the selected language live in
//! [`lc_ui_api::i18n`] because feature crates need the same lookups; the shell
//! re-exports them here so shared and feature-facing UI code has one import path.

pub use lc_ui_api::i18n::{
    contains_key, doc, doc_for, keys, language, set_language, tr, tr_for, DEFAULT_LANGUAGE,
};
