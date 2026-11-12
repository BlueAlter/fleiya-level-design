//! Thread-safe bounded ring buffer and filtering structures for the console panel.

use egui::Color32;
use lc_canvas::theme::Tokens;
use lc_ui_api::DocKind;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Severity levels supported by the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Error level (highest severity).
    Error,
    /// Warning level.
    Warn,
    /// Informational level.
    Info,
    /// Debugging level.
    Debug,
    /// Detailed trace level.
    Trace,
}

impl LogLevel {
    /// String representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    /// Color badge styling for egui rendering.
    pub fn color(&self, t: &Tokens) -> Color32 {
        match self {
            Self::Error => t.bad,
            Self::Warn => t.warn,
            Self::Info => t.accent_ink,
            Self::Debug => t.muted,
            Self::Trace => t.faint,
        }
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::ERROR => Self::Error,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::INFO => Self::Info,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::TRACE => Self::Trace,
        }
    }
}

/// A single structured log message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Formatted timestamp (e.g. HH:MM:SS).
    pub timestamp: String,
    /// Severity level.
    pub level: LogLevel,
    /// Target module or crate name.
    pub target: String,
    /// Formatted log message text.
    pub message: String,
    /// Associated document target if the log pertains to a specific document.
    pub doc_target: Option<DocKind>,
}

/// A collapsed log entry indicating consecutive duplicate occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapsedLogEntry {
    /// Underlying log entry.
    pub entry: LogEntry,
    /// Number of consecutive occurrences.
    pub count: usize,
}

/// Filter settings controlling console log display.
#[derive(Debug, Clone)]
pub struct ConsoleFilter {
    /// Show error messages.
    pub show_error: bool,
    /// Show warning messages.
    pub show_warn: bool,
    /// Show info messages.
    pub show_info: bool,
    /// Show debug messages.
    pub show_debug: bool,
    /// Show trace messages.
    pub show_trace: bool,
    /// Substring search query.
    pub search_query: String,
    /// Collapse consecutive duplicate messages.
    pub collapse_duplicates: bool,
}

impl Default for ConsoleFilter {
    fn default() -> Self {
        Self {
            show_error: true,
            show_warn: true,
            show_info: true,
            show_debug: false,
            show_trace: false,
            search_query: String::new(),
            collapse_duplicates: true,
        }
    }
}

impl ConsoleFilter {
    /// Checks whether the level matches active toggles.
    pub fn matches_level(&self, level: LogLevel) -> bool {
        match level {
            LogLevel::Error => self.show_error,
            LogLevel::Warn => self.show_warn,
            LogLevel::Info => self.show_info,
            LogLevel::Debug => self.show_debug,
            LogLevel::Trace => self.show_trace,
        }
    }

    /// Checks whether the text or target matches the search query.
    pub fn matches_text(&self, text: &str, target: &str) -> bool {
        let trimmed = self.search_query.trim();
        if trimmed.is_empty() {
            return true;
        }
        let q = trimmed.to_lowercase();
        self.matches_text_query(text, target, &q)
    }

    /// Checks whether text or target matches a pre-lowercased query string without reallocating query.
    pub fn matches_text_query(&self, text: &str, target: &str, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        text.to_lowercase().contains(query_lower) || target.to_lowercase().contains(query_lower)
    }
}

/// Bounded in-memory ring buffer for log entries.
#[derive(Debug)]
pub struct BoundedRingBuffer {
    capacity: usize,
    entries: VecDeque<LogEntry>,
}

impl BoundedRingBuffer {
    /// Default capacity of 10,000 entries.
    pub const DEFAULT_CAPACITY: usize = 10_000;

    /// Creates a new bounded ring buffer with the specified capacity limit.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            capacity: cap,
            entries: VecDeque::with_capacity(cap.min(10_000)),
        }
    }

    /// Pushes a new entry into the buffer, discarding the oldest when full.
    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no entries are stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Maximum capacity of the ring buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clears all entries from the buffer.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns a cloned snapshot of all entries in insertion order.
    pub fn all_entries(&self) -> Vec<LogEntry> {
        self.entries.iter().cloned().collect()
    }

    /// Produces a filtered and optionally collapsed list of entries according to `filter`.
    pub fn filtered_entries(&self, filter: &ConsoleFilter) -> Vec<CollapsedLogEntry> {
        let mut result: Vec<CollapsedLogEntry> = Vec::new();
        let query_lower = filter.search_query.trim().to_lowercase();
        let has_query = !query_lower.is_empty();

        for entry in &self.entries {
            if !filter.matches_level(entry.level) {
                continue;
            }
            if has_query && !filter.matches_text_query(&entry.message, &entry.target, &query_lower)
            {
                continue;
            }

            if filter.collapse_duplicates {
                if let Some(last) = result.last_mut() {
                    if last.entry.level == entry.level
                        && last.entry.target == entry.target
                        && last.entry.message == entry.message
                        && last.entry.doc_target == entry.doc_target
                    {
                        last.count = last.count.saturating_add(1);
                        continue;
                    }
                }
            }

            result.push(CollapsedLogEntry {
                entry: entry.clone(),
                count: 1,
            });
        }

        result
    }

    /// Returns counts of entries by severity level: (error, warn, info, debug, trace).
    pub fn counts_by_level(&self) -> (usize, usize, usize, usize, usize) {
        let mut errors = 0;
        let mut warns = 0;
        let mut infos = 0;
        let mut debugs = 0;
        let mut traces = 0;

        for e in &self.entries {
            match e.level {
                LogLevel::Error => errors += 1,
                LogLevel::Warn => warns += 1,
                LogLevel::Info => infos += 1,
                LogLevel::Debug => debugs += 1,
                LogLevel::Trace => traces += 1,
            }
        }

        (errors, warns, infos, debugs, traces)
    }
}

/// Thread-safe shared handle to a `BoundedRingBuffer`.
#[derive(Debug, Clone)]
pub struct SharedRingBuffer {
    inner: Arc<Mutex<BoundedRingBuffer>>,
}

impl SharedRingBuffer {
    /// Creates a new thread-safe shared ring buffer with the specified capacity limit.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BoundedRingBuffer::new(capacity))),
        }
    }

    /// Pushes an entry into the ring buffer safely without blocking worker threads.
    pub fn push(&self, entry: LogEntry) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.push(entry);
        }
    }

    /// Clears the ring buffer.
    pub fn clear(&self) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.clear();
        }
    }

    /// Retrieves filtered and collapsed entries.
    pub fn filtered_entries(&self, filter: &ConsoleFilter) -> Vec<CollapsedLogEntry> {
        if let Ok(lock) = self.inner.lock() {
            lock.filtered_entries(filter)
        } else {
            Vec::new()
        }
    }

    /// Returns the current number of items.
    pub fn len(&self) -> usize {
        if let Ok(lock) = self.inner.lock() {
            lock.len()
        } else {
            0
        }
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns counts by level: (error, warn, info, debug, trace).
    pub fn counts_by_level(&self) -> (usize, usize, usize, usize, usize) {
        if let Ok(lock) = self.inner.lock() {
            lock.counts_by_level()
        } else {
            (0, 0, 0, 0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::id::LevelId;

    fn sample_entry(level: LogLevel, msg: &str) -> LogEntry {
        LogEntry {
            timestamp: "12:00:00".to_string(),
            level,
            target: "lc_test".to_string(),
            message: msg.to_string(),
            doc_target: None,
        }
    }

    #[test]
    fn test_ring_buffer_bound_capacity() {
        let mut buf = BoundedRingBuffer::new(3);
        assert_eq!(buf.capacity(), 3);

        buf.push(sample_entry(LogLevel::Info, "one"));
        buf.push(sample_entry(LogLevel::Info, "two"));
        buf.push(sample_entry(LogLevel::Info, "three"));
        assert_eq!(buf.len(), 3);

        // Pushing a 4th entry drops "one"
        buf.push(sample_entry(LogLevel::Info, "four"));
        assert_eq!(buf.len(), 3);

        let entries = buf.all_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries.first().map(|e| e.message.as_str()), Some("two"));
        assert_eq!(entries.get(1).map(|e| e.message.as_str()), Some("three"));
        assert_eq!(entries.get(2).map(|e| e.message.as_str()), Some("four"));

        // Test with 15,000 items in a 10,000 capacity buffer
        let mut large_buf = BoundedRingBuffer::new(BoundedRingBuffer::DEFAULT_CAPACITY);
        for i in 0..15_000 {
            large_buf.push(sample_entry(LogLevel::Debug, &format!("msg {i}")));
        }
        assert_eq!(large_buf.len(), 10_000);
        let first = large_buf.all_entries().first().cloned();
        assert_eq!(first.map(|e| e.message), Some("msg 5000".to_string()));
    }

    #[test]
    fn test_filtering_by_level() {
        let mut buf = BoundedRingBuffer::new(10);
        buf.push(sample_entry(LogLevel::Error, "error 1"));
        buf.push(sample_entry(LogLevel::Warn, "warn 1"));
        buf.push(sample_entry(LogLevel::Info, "info 1"));
        buf.push(sample_entry(LogLevel::Debug, "debug 1"));
        buf.push(sample_entry(LogLevel::Trace, "trace 1"));

        // Default filter: Error, Warn, Info (Debug and Trace hidden)
        let filter = ConsoleFilter::default();
        let res = buf.filtered_entries(&filter);
        assert_eq!(res.len(), 3);
        assert_eq!(res.first().map(|r| r.entry.level), Some(LogLevel::Error));
        assert_eq!(res.get(1).map(|r| r.entry.level), Some(LogLevel::Warn));
        assert_eq!(res.get(2).map(|r| r.entry.level), Some(LogLevel::Info));

        // Only error
        let mut err_filter = ConsoleFilter {
            show_error: true,
            show_warn: false,
            show_info: false,
            show_debug: false,
            show_trace: false,
            search_query: String::new(),
            collapse_duplicates: false,
        };
        let res_err = buf.filtered_entries(&err_filter);
        assert_eq!(res_err.len(), 1);
        assert_eq!(
            res_err.first().map(|r| r.entry.message.as_str()),
            Some("error 1")
        );

        // Include debug
        err_filter.show_debug = true;
        let res_debug = buf.filtered_entries(&err_filter);
        assert_eq!(res_debug.len(), 2);
    }

    #[test]
    fn test_filtering_by_search_query() {
        let mut buf = BoundedRingBuffer::new(10);
        buf.push(sample_entry(LogLevel::Info, "Loading project levels.lc"));
        buf.push(sample_entry(LogLevel::Info, "Saving tileset dungeon.png"));
        buf.push(sample_entry(
            LogLevel::Warn,
            "Entity player missing texture",
        ));

        let filter = ConsoleFilter {
            search_query: "project".to_string(),
            ..Default::default()
        };
        let res = buf.filtered_entries(&filter);
        assert_eq!(res.len(), 1);
        assert_eq!(
            res.first().map(|r| r.entry.message.as_str()),
            Some("Loading project levels.lc")
        );

        // Case insensitive match
        let filter_case = ConsoleFilter {
            search_query: "PLAYER".to_string(),
            ..Default::default()
        };
        let res_case = buf.filtered_entries(&filter_case);
        assert_eq!(res_case.len(), 1);
        assert_eq!(
            res_case.first().map(|r| r.entry.message.as_str()),
            Some("Entity player missing texture")
        );
    }

    #[test]
    fn test_duplicate_collapsing() {
        let mut buf = BoundedRingBuffer::new(10);
        buf.push(sample_entry(LogLevel::Info, "identical message"));
        buf.push(sample_entry(LogLevel::Info, "identical message"));
        buf.push(sample_entry(LogLevel::Info, "identical message"));
        buf.push(sample_entry(LogLevel::Warn, "different message"));
        buf.push(sample_entry(LogLevel::Info, "identical message"));

        // With duplicate collapsing enabled
        let mut filter = ConsoleFilter {
            collapse_duplicates: true,
            ..Default::default()
        };
        let collapsed = buf.filtered_entries(&filter);

        assert_eq!(collapsed.len(), 3);
        assert_eq!(collapsed.first().map(|r| r.count), Some(3));
        assert_eq!(
            collapsed.first().map(|r| r.entry.message.as_str()),
            Some("identical message")
        );
        assert_eq!(collapsed.get(1).map(|r| r.count), Some(1));
        assert_eq!(
            collapsed.get(1).map(|r| r.entry.message.as_str()),
            Some("different message")
        );
        assert_eq!(collapsed.get(2).map(|r| r.count), Some(1));
        assert_eq!(
            collapsed.get(2).map(|r| r.entry.message.as_str()),
            Some("identical message")
        );

        // With duplicate collapsing disabled
        filter.collapse_duplicates = false;
        let uncollapsed = buf.filtered_entries(&filter);
        assert_eq!(uncollapsed.len(), 5);
        for item in &uncollapsed {
            assert_eq!(item.count, 1);
        }
    }

    #[test]
    fn test_doc_target_preservation() {
        let mut buf = BoundedRingBuffer::new(5);
        let mut entry = sample_entry(LogLevel::Info, "Opened level");
        entry.doc_target = Some(DocKind::Level(LevelId(42)));
        buf.push(entry);

        let filter = ConsoleFilter::default();
        let res = buf.filtered_entries(&filter);
        assert_eq!(res.len(), 1);
        assert_eq!(
            res.first().and_then(|r| r.entry.doc_target.as_ref()),
            Some(&DocKind::Level(LevelId(42)))
        );
    }
}
