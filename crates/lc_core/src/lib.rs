#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! `lc_core`: Core data model, commands, and undo stack.
//! No egui dependency. No filesystem access.
