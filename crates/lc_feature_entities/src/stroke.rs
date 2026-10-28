//! Per-interaction stroke ids used to merge multi-frame edits into one undo step.

pub use lc_canvas::ids::next_stroke_id;

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

/// Keeps one stroke id alive while a field editor keeps focus.
///
/// Typing produces changed frames separated by quiet frames, so a plain
/// [`StrokeTracker`] would split a single typing session into several undo
/// steps. This tracker keeps the id while the editor reports focus and drops it
/// once focus is lost, giving "one undo step per focus session".
#[derive(Debug)]
pub struct FocusStrokeTracker<T: Clone + PartialEq> {
    current: Option<(T, u64)>,
    kept: bool,
}

impl<T: Clone + PartialEq> Default for FocusStrokeTracker<T> {
    fn default() -> Self {
        Self {
            current: None,
            kept: false,
        }
    }
}

impl<T: Clone + PartialEq> FocusStrokeTracker<T> {
    /// Starts a new frame.
    pub fn begin_frame(&mut self) {
        self.kept = false;
    }

    /// Returns the stroke id for `key` when the editor is focused or changed.
    ///
    /// `held` is the editor's current focus state, `changed` whether the value
    /// changed this frame. Returns `None` for an idle editor.
    pub fn id(&mut self, key: T, held: bool, changed: bool) -> Option<u64> {
        if !held && !changed {
            return None;
        }
        self.kept = true;
        match &self.current {
            Some((current, id)) if *current == key => Some(*id),
            _ => {
                let id = next_stroke_id();
                self.current = Some((key, id));
                Some(id)
            }
        }
    }

    /// Drops the current stroke when nothing used it this frame.
    pub fn end_frame(&mut self) {
        if !self.kept {
            self.current = None;
        }
    }

    /// Forgets any active stroke, e.g. when the selection changes.
    pub fn reset(&mut self) {
        self.current = None;
        self.kept = false;
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

    #[test]
    fn focus_tracker_keeps_a_stroke_across_quiet_frames_while_focused() {
        let mut tracker: FocusStrokeTracker<String> = FocusStrokeTracker::default();
        let key = "Player.health".to_string();

        tracker.begin_frame();
        let first = tracker.id(key.clone(), true, true).expect("changed");
        tracker.end_frame();

        // User pauses typing: no change, but the editor still has focus.
        tracker.begin_frame();
        assert_eq!(tracker.id(key.clone(), true, false), Some(first));
        tracker.end_frame();

        tracker.begin_frame();
        assert_eq!(tracker.id(key.clone(), true, true), Some(first));
        tracker.end_frame();
    }

    #[test]
    fn focus_tracker_starts_a_new_stroke_after_focus_is_lost() {
        let mut tracker: FocusStrokeTracker<String> = FocusStrokeTracker::default();
        let key = "Player.health".to_string();

        tracker.begin_frame();
        let first = tracker.id(key.clone(), true, true).expect("changed");
        tracker.end_frame();

        // Focus lost: idle frame drops the stroke.
        tracker.begin_frame();
        assert_eq!(tracker.id(key.clone(), false, false), None);
        tracker.end_frame();

        tracker.begin_frame();
        let second = tracker.id(key, true, true).expect("changed again");
        tracker.end_frame();
        assert_ne!(second, first);
    }

    #[test]
    fn focus_tracker_reset_starts_a_new_stroke() {
        let mut tracker: FocusStrokeTracker<&'static str> = FocusStrokeTracker::default();
        tracker.begin_frame();
        let first = tracker.id("pos", true, true).expect("changed");
        tracker.end_frame();

        tracker.reset();
        tracker.begin_frame();
        let second = tracker.id("pos", true, true).expect("changed");
        tracker.end_frame();
        assert_ne!(first, second);
    }
}
