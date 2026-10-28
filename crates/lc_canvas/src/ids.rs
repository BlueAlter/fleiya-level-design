//! Process-wide ids shared by canvas interactions.

use std::sync::atomic::{AtomicU64, Ordering};

static STROKE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Allocates a process-unique stroke id.
///
/// Multi-frame interactions (paint strokes, drags) use the same id so their
/// commands merge into a single undo step.
pub fn next_stroke_id() -> u64 {
    STROKE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::next_stroke_id;

    #[test]
    fn stroke_ids_are_unique() {
        assert_ne!(next_stroke_id(), next_stroke_id());
    }
}
