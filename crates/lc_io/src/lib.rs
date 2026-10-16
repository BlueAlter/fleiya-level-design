#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! `lc_io`: Multi-file JSON serialization, migrations, and project I/O.
//! The ONLY crate using `std::fs`.
