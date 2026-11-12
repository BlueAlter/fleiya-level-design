//! Tracing subscriber layer routing events to the in-memory ring buffer and rotating file log.

use super::ring_buffer::{LogEntry, LogLevel, SharedRingBuffer};
use lc_core::id::{EntityDefId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use lc_io::RotatingLogWriter;
use lc_ui_api::DocKind;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// Custom tracing subscriber layer capturing log events for the Level Craft console.
#[derive(Clone)]
pub struct ConsoleSubscriberLayer {
    buffer: SharedRingBuffer,
    file_writer: Option<Arc<RotatingLogWriter>>,
}

impl ConsoleSubscriberLayer {
    /// Creates a new subscriber layer pointing to the given ring buffer.
    pub fn new(buffer: SharedRingBuffer) -> Self {
        Self {
            buffer,
            file_writer: None,
        }
    }

    /// Attaches a rotating file writer to mirror all logs to disk.
    pub fn with_file_writer(mut self, writer: Arc<RotatingLogWriter>) -> Self {
        self.file_writer = Some(writer);
        self
    }
}

impl<S: Subscriber> Layer<S> for ConsoleSubscriberLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = LogLevel::from(*metadata.level());
        let target = metadata.target().to_string();

        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);

        let doc_target = visitor
            .doc_target
            .or_else(|| parse_doc_kind(&visitor.message));

        let timestamp = current_timestamp();

        let entry = LogEntry {
            timestamp: timestamp.clone(),
            level,
            target: target.clone(),
            message: visitor.message.clone(),
            doc_target,
        };

        // Push to in-memory ring buffer (non-blocking, fast mutex lock)
        self.buffer.push(entry);

        // Optionally write to rotating file
        if let Some(writer) = &self.file_writer {
            let log_line = format!(
                "[{}] [{}] [{}] {}\n",
                timestamp,
                level.as_str(),
                target,
                visitor.message
            );
            let _ = writer.write_bytes(log_line.as_bytes());
        }
    }
}

/// Visitor that formats event fields into a human-readable message string.
#[derive(Default)]
struct EventVisitor {
    message: String,
    doc_target: Option<DocKind>,
}

impl Visit for EventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{value:?}");
        } else if field.name() == "doc_target" || field.name() == "doc_kind" {
            let s = format!("{value:?}");
            self.doc_target = parse_doc_kind(&s);
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={:?}", field.name(), value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(value);
        } else if field.name() == "doc_target" || field.name() == "doc_kind" {
            self.doc_target = parse_doc_kind(value);
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={}", field.name(), value);
        }
    }
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = now.as_secs();
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02}")
}

/// Parses an optional `DocKind` identifier from a string or message.
pub fn parse_doc_kind(text: &str) -> Option<DocKind> {
    if text.contains("ProjectSettings") {
        return Some(DocKind::ProjectSettings);
    }

    if let Some(id) = extract_id(text, &["Level(LevelId(", "LevelId(", "Level("]) {
        return Some(DocKind::Level(LevelId(id)));
    }
    if let Some(id) = extract_id(text, &["World(WorldId(", "WorldId(", "World("]) {
        return Some(DocKind::World(WorldId(id)));
    }
    if let Some(id) = extract_id(
        text,
        &["EntityDef(EntityDefId(", "EntityDefId(", "EntityDef("],
    ) {
        return Some(DocKind::EntityDef(EntityDefId(id)));
    }
    if let Some(id) = extract_id(text, &["EnumDef(EnumDefId(", "EnumDefId(", "EnumDef("]) {
        return Some(DocKind::EnumDef(EnumDefId(id)));
    }
    if let Some(id) = extract_id(
        text,
        &["TilesetDef(TilesetDefId(", "TilesetDefId(", "TilesetDef("],
    ) {
        return Some(DocKind::TilesetDef(TilesetDefId(id)));
    }
    if let Some(id) = extract_id(text, &["LayerDef(LayerDefId(", "LayerDefId(", "LayerDef("]) {
        return Some(DocKind::LayerDef(LayerDefId(id)));
    }

    None
}

fn extract_id(text: &str, prefixes: &[&str]) -> Option<u64> {
    for &prefix in prefixes {
        if let Some(pos) = text.find(prefix) {
            let rest = text.get(pos + prefix.len()..)?;
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(id) = digits.parse::<u64>() {
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_kind() {
        assert_eq!(
            parse_doc_kind("Loaded Level(10)"),
            Some(DocKind::Level(LevelId(10)))
        );
        assert_eq!(
            parse_doc_kind("doc_target: LevelId(10)"),
            Some(DocKind::Level(LevelId(10)))
        );
        assert_eq!(
            parse_doc_kind("doc_target: Level(LevelId(10))"),
            Some(DocKind::Level(LevelId(10)))
        );
        assert_eq!(
            parse_doc_kind("Active World(3)"),
            Some(DocKind::World(WorldId(3)))
        );
        assert_eq!(
            parse_doc_kind("WorldId(3) selected"),
            Some(DocKind::World(WorldId(3)))
        );
        assert_eq!(
            parse_doc_kind("Edited EntityDef(5)"),
            Some(DocKind::EntityDef(EntityDefId(5)))
        );
        assert_eq!(
            parse_doc_kind("EntityDefId(5) updated"),
            Some(DocKind::EntityDef(EntityDefId(5)))
        );
        assert_eq!(
            parse_doc_kind("Opened EnumDef(1)"),
            Some(DocKind::EnumDef(EnumDefId(1)))
        );
        assert_eq!(
            parse_doc_kind("TilesetDef(9) loaded"),
            Some(DocKind::TilesetDef(TilesetDefId(9)))
        );
        assert_eq!(
            parse_doc_kind("LayerDef(2) changed"),
            Some(DocKind::LayerDef(LayerDefId(2)))
        );
        assert_eq!(
            parse_doc_kind("Show ProjectSettings dialog"),
            Some(DocKind::ProjectSettings)
        );
        assert_eq!(parse_doc_kind("Random log without doc"), None);
    }
}
