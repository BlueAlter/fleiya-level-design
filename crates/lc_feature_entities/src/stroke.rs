//! Per-interaction stroke ids used to merge multi-frame edits into one undo step.

use std::sync::atomic::{AtomicU64, Ordering};

static STROKE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Allocates a process-unique stroke id.
pub fn next_stroke_id() -> u64 {
    STROKE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Tracks which edit currently owns a stroke id.
///
/// Call [`StrokeTracker::begin_frame`] before drawing, [`StrokeTracker::id`]
/// for every edit that changed, and [`StrokeTracker::end_frame`] afterwards.
/// The id survives as long as the same field keeps changing; it is dropped when
/// the interaction ends so the next edit becomes its own undo step.
#[derive(Debug)]
pub struct StrokeTracker<T: Copy + PartialEq> {
    current: Option<(T, u64)>,
    kept: bool,
}

impl<T: Copy + PartialEq> Default for StrokeTracker<T> {
    fn default() -> Self {
        Self {
            current: None,
            kept: false,
        }
    }
}

impl<T: Copy + PartialEq> StrokeTracker<T> {
    /// Starts a new frame.
    pub fn begin_frame(&mut self) {
        self.kept = false;
    }

    /// Returns the stroke id for `field`, allocating one when the field changed.
    pub fn id(&mut self, field: T) -> u64 {
        self.kept = true;
        match self.current {
            Some((current, id)) if current == field => id,
            _ => {
                let id = next_stroke_id();
                self.current = Some((field, id));
                id
            }
        }
    }

    /// Keeps the current stroke if it was used, otherwise forgets it.
    pub fn end_frame(&mut self) {
        if !self.kept {
            self.current = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_ids_are_unique() {
        assert_ne!(next_stroke_id(), next_stroke_id());
    }

    #[test]
    fn tracker_reuses_while_the_same_field_keeps_changing() {
        let mut tracker = StrokeTracker::default();
        tracker.begin_frame();
        let first = tracker.id(1);
        tracker.end_frame();

        tracker.begin_frame();
        assert_eq!(tracker.id(1), first);
        tracker.end_frame();
    }

    #[test]
    fn tracker_starts_a_new_stroke_after_a_quiet_frame() {
        let mut tracker = StrokeTracker::default();
        tracker.begin_frame();
        let first = tracker.id(1);
        tracker.end_frame();

        tracker.begin_frame();
        tracker.end_frame();

        tracker.begin_frame();
        assert_ne!(tracker.id(1), first);
        tracker.end_frame();
    }
}
