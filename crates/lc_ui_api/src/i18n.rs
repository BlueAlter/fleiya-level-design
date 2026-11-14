//! Localization lookup for editor UI strings and documentation pages.
//!
//! Every language is a folder under `assets/localization/<lang>/` holding short
//! UI strings (`editor/<feature>.ron`) and long-form Markdown documents
//! (`docs/<topic>/<page>.md`). The files are embedded at compile time, so the
//! editor never reads them from disk at runtime.
//!
//! Lookup order is the selected language, then `en-US`, then the key itself.
//! A missing key is logged once as a warning so translation gaps are visible
//! without flooding the console every frame.
//!
//! The selected language is an application preference and deliberately not part
//! of `Project`: it changes how the editor talks to the user, not their data.
//! v1 ships `en-US` only.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock, RwLock};

/// Language used when the selected one has no entry.
pub const DEFAULT_LANGUAGE: &str = "en-US";

/// Well-known localization keys used by Rust code.
///
/// Every constant is checked against `en-US` by the module tests, so a key can
/// never be used in code without being translated.
pub mod keys {
    /// Label of the `ENUM DEF` sub-tab.
    pub const ENUMS_TAB_DEFINITION: &str = "enums.tab.definition";
    /// Label of the `DOCUMENT` sub-tab.
    pub const ENUMS_TAB_DOCUMENT: &str = "enums.tab.document";
    /// One-line description under the enum title.
    pub const ENUMS_DESCRIPTION: &str = "enums.description";

    /// Every key defined above, for completeness tests.
    pub const ALL: &[&str] = &[ENUMS_TAB_DEFINITION, ENUMS_TAB_DOCUMENT, ENUMS_DESCRIPTION];
}

/// One language bundle: the short UI string tables of that language.
struct Bundle {
    language: &'static str,
    sources: &'static [&'static str],
}

/// Language bundles compiled into the binary.
const BUNDLES: &[Bundle] = &[Bundle {
    language: DEFAULT_LANGUAGE,
    sources: &[include_str!(
        "../../../assets/localization/en-US/editor/enums.ron"
    )],
}];

/// One long-form documentation page compiled into the binary.
struct DocPage {
    language: &'static str,
    topic: &'static str,
    page: &'static str,
    text: &'static str,
}

/// Documentation pages compiled into the binary.
const DOC_PAGES: &[DocPage] = &[DocPage {
    language: DEFAULT_LANGUAGE,
    topic: "enum",
    page: "overview",
    text: include_str!("../../../assets/localization/en-US/docs/enum/overview.md"),
}];

/// Parsed string tables, keyed by the language code of the bundle.
static TABLES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

/// Keys already reported as missing, so each one warns only once.
static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Selected language, or `None` for [`DEFAULT_LANGUAGE`].
static LANGUAGE: RwLock<Option<String>> = RwLock::new(None);

/// Returns the language currently selected by the application.
pub fn language() -> String {
    match LANGUAGE.read() {
        Ok(guard) => guard.clone().unwrap_or_else(default_language),
        Err(poisoned) => poisoned
            .into_inner()
            .clone()
            .unwrap_or_else(default_language),
    }
}

/// Selects the language used by [`tr`] and [`doc`].
///
/// Unknown languages are accepted: every lookup then falls back to
/// [`DEFAULT_LANGUAGE`], which keeps an unsupported preference harmless.
pub fn set_language(language: impl Into<String>) {
    let value = language.into();
    match LANGUAGE.write() {
        Ok(mut guard) => *guard = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_owned()
}

fn tables() -> &'static HashMap<&'static str, HashMap<String, String>> {
    TABLES.get_or_init(|| {
        BUNDLES
            .iter()
            .map(|bundle| {
                let mut table = HashMap::new();
                for source in bundle.sources {
                    match ron::from_str::<HashMap<String, String>>(source) {
                        Ok(parsed) => table.extend(parsed),
                        Err(error) => tracing::error!(
                            language = bundle.language,
                            %error,
                            "failed to parse localization table"
                        ),
                    }
                }
                (bundle.language, table)
            })
            .collect()
    })
}

fn bundle_code(language: &str) -> Option<&'static str> {
    BUNDLES
        .iter()
        .find(|bundle| bundle.language.eq_ignore_ascii_case(language))
        .map(|bundle| bundle.language)
}

fn lookup(language: &str, key: &str) -> Option<&'static str> {
    let code = bundle_code(language)?;
    tables().get(code)?.get(key).map(String::as_str)
}

fn warn_missing_once(key: &str, language: &str) {
    let warned = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    let mut warned = match warned.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if warned.insert(key.to_owned()) {
        tracing::warn!(
            key,
            language,
            "localization key missing; falling back to en-US or the key itself"
        );
    }
}

/// Looks up `key` for an explicit language, with the `en-US` → key fallback.
pub fn tr_for(language: &str, key: &'static str) -> &'static str {
    if let Some(text) = lookup(language, key) {
        return text;
    }
    warn_missing_once(key, language);
    lookup(DEFAULT_LANGUAGE, key).unwrap_or(key)
}

/// Looks up a short UI string for the selected language.
pub fn tr(key: &'static str) -> &'static str {
    tr_for(&language(), key)
}

fn find_doc(language: &str, topic: &str, page: &str) -> Option<&'static str> {
    DOC_PAGES
        .iter()
        .find(|entry| {
            entry.language.eq_ignore_ascii_case(language)
                && entry.topic == topic
                && entry.page == page
        })
        .map(|entry| entry.text)
}

/// Returns the Markdown of `topic/page` for an explicit language.
///
/// Falls back to `en-US`, then to an empty page (with a one-time warning).
pub fn doc_for(language: &str, topic: &str, page: &str) -> &'static str {
    if let Some(text) = find_doc(language, topic, page) {
        return text;
    }
    warn_missing_once(&format!("doc:{topic}/{page}"), language);
    find_doc(DEFAULT_LANGUAGE, topic, page).unwrap_or("")
}

/// Returns the Markdown of `topic/page` for the selected language.
pub fn doc(topic: &str, page: &str) -> &'static str {
    doc_for(&language(), topic, page)
}

/// Returns `true` when `language` defines `key`.
pub fn contains_key(language: &str, key: &str) -> bool {
    lookup(language, key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_resolve_to_their_english_text() {
        assert_eq!(tr_for("en-US", keys::ENUMS_TAB_DEFINITION), "ENUM DEF");
        assert_eq!(tr_for("en-US", keys::ENUMS_TAB_DOCUMENT), "DOCUMENT");
        assert!(tr_for("en-US", keys::ENUMS_DESCRIPTION).starts_with("Define enumeration keys"));
    }

    #[test]
    fn every_key_used_in_code_exists_in_english() {
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(
            tr_for("fr-FR", keys::ENUMS_TAB_DEFINITION),
            tr_for(DEFAULT_LANGUAGE, keys::ENUMS_TAB_DEFINITION)
        );
    }

    #[test]
    fn unknown_key_falls_back_to_itself() {
        assert_eq!(tr_for("en-US", "enums.unknown.key"), "enums.unknown.key");
    }

    #[test]
    fn language_codes_are_case_insensitive() {
        assert_eq!(
            tr_for("EN-us", keys::ENUMS_TAB_DOCUMENT),
            tr_for("en-US", keys::ENUMS_TAB_DOCUMENT)
        );
    }

    #[test]
    fn documents_fall_back_to_english() {
        let english = doc_for(DEFAULT_LANGUAGE, "enum", "overview");
        assert!(
            english.contains("# Enums"),
            "overview page must be embedded"
        );
        assert_eq!(doc_for("fr-FR", "enum", "overview"), english);
    }

    #[test]
    fn missing_document_is_empty() {
        assert_eq!(doc_for(DEFAULT_LANGUAGE, "enum", "missing-page"), "");
    }

    #[test]
    fn language_preference_defaults_to_english() {
        // Keep the global preference untouched for other tests.
        let previous = language();
        set_language(DEFAULT_LANGUAGE);
        assert_eq!(language(), DEFAULT_LANGUAGE);
        set_language(previous);
    }
}
