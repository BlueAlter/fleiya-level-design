//! Console subsystem for Level Craft.
//!
//! Provides in-memory ring-buffered logging, Godot-style console UI panel,
//! and subscriber layer connecting `tracing` to the shell.

pub mod layer;
pub mod panel;
pub mod ring_buffer;

pub use layer::{parse_doc_kind, ConsoleSubscriberLayer};
pub use panel::ConsolePanel;
pub use ring_buffer::{
    BoundedRingBuffer, CollapsedLogEntry, ConsoleFilter, LogEntry, LogLevel, SharedRingBuffer,
};
