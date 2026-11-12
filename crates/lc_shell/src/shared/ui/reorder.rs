//! Pure reordering mathematics shared by every drag-to-reorder list.
//!
//! No UI code lives here: only index and coordinate math, so the behaviour can be
//! unit tested without an egui context. The egui side lives in [`super::drag_list`].
//!
//! # Index conventions
//! * `from` is the index of the dragged item in the *original* order.
//! * `to` is an *insertion index* in `0..=len` in terms of the original order,
//!   i.e. the number of rows whose centre lies above the pointer. It is exactly the
//!   value [`drop_index`] returns and the value [`reorder_item`] expects.

use egui::{Pos2, Rect};

/// One entry of the temporary display order used while an item is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// A regular row, identified by its index in the original order.
    Item(usize),
    /// The ghost slot marking where the dragged item will land when dropped.
    Placeholder,
}

/// Computes the drop target index from a pointer Y coordinate against row bounding rects.
///
/// The result is the number of rows whose centre is above the pointer.
pub fn drop_index(pointer_y: f32, row_rects: &[Rect]) -> usize {
    if row_rects.is_empty() {
        return 0;
    }
    let mut count = 0;
    for r in row_rects {
        if pointer_y > r.center().y {
            count += 1;
        }
    }
    count
}

/// Pure reordering function: moves the item at `from` to insertion index `to`.
///
/// Returns `false` (and leaves `items` untouched) when the move would be a no-op.
pub fn reorder_item<T>(items: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from >= items.len() || to > items.len() || from == to || from + 1 == to {
        return false;
    }
    let item = items.remove(from);
    let target = if to > from { to - 1 } else { to };
    items.insert(target.min(items.len()), item);
    true
}

/// Position the dragged item ends up at once dropped on insertion index `to`.
pub fn final_index(from: usize, to: usize) -> usize {
    if to > from {
        to - 1
    } else {
        to
    }
}

/// Returns `true` when dropping at `to` would not change the order.
pub fn is_noop_move(from: usize, to: usize) -> bool {
    from == to || from + 1 == to
}

/// Temporary display order while dragging: the dragged row is removed from its
/// original place and a [`Slot::Placeholder`] is inserted where it would land.
///
/// Every other row keeps its relative order, so rows visually make room for the
/// dragged item. With `from >= len` the normal order is returned.
pub fn preview_layout(len: usize, from: usize, to: usize) -> Vec<Slot> {
    if from >= len {
        return (0..len).map(Slot::Item).collect();
    }
    let at = final_index(from, to.min(len)).min(len - 1);
    let mut slots: Vec<Slot> = (0..len).filter(|&i| i != from).map(Slot::Item).collect();
    slots.insert(at, Slot::Placeholder);
    slots
}

/// Computes the drop target from a pointer position **relative to the list top**
/// and a snapshot of the original row rects, also relative to the list top.
///
/// Because the snapshot is taken once when the drag starts, the answer does not
/// depend on how rows are currently shifted by the live preview. That keeps the
/// placeholder from flickering back and forth around row boundaries.
pub fn snapshot_target(pointer_y_from_top: f32, snapshot: &[Rect]) -> usize {
    drop_index(pointer_y_from_top, snapshot)
}

/// Converts absolute row rects to rects relative to the top of the first row.
pub fn relative_to_top(rects: &[Rect]) -> Vec<Rect> {
    let Some(first) = rects.first() else {
        return Vec::new();
    };
    let top = first.min.y;
    rects
        .iter()
        .map(|r| r.translate(egui::vec2(0.0, -top)))
        .collect()
}

/// Vertical scroll step to apply while dragging near the edges of a scroll area.
///
/// Uses the `Ui::scroll_with_delta` convention: positive scrolls towards the top
/// (revealing content above), negative towards the bottom, `0.0` when the pointer
/// is not within `edge` pixels of either border.
pub fn auto_scroll_delta(pointer: Pos2, clip: Rect, edge: f32, speed: f32) -> f32 {
    if pointer.y < clip.top() + edge {
        speed
    } else if pointer.y > clip.bottom() - edge {
        -speed
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn rows(n: usize, h: f32, gap: f32) -> Vec<Rect> {
        (0..n)
            .map(|i| Rect::from_min_size(pos2(0.0, i as f32 * (h + gap)), vec2(100.0, h)))
            .collect()
    }

    #[test]
    fn drop_index_counts_rows_above_pointer() {
        let r = rows(3, 40.0, 4.0);
        assert_eq!(drop_index(-10.0, &r), 0);
        assert_eq!(drop_index(25.0, &r), 1);
        assert_eq!(drop_index(70.0, &r), 2);
        assert_eq!(drop_index(500.0, &r), 3);
        assert_eq!(drop_index(10.0, &[]), 0);
    }

    #[test]
    fn reorder_item_moves_and_ignores_noops() {
        let mut v = vec!['a', 'b', 'c', 'd'];
        assert!(reorder_item(&mut v, 0, 3));
        assert_eq!(v, vec!['b', 'c', 'a', 'd']);
        assert!(reorder_item(&mut v, 3, 0));
        assert_eq!(v, vec!['d', 'b', 'c', 'a']);
        assert!(!reorder_item(&mut v, 1, 1));
        assert!(!reorder_item(&mut v, 1, 2));
        assert!(!reorder_item(&mut v, 9, 0));
        assert!(!reorder_item(&mut v, 0, 99));
        assert_eq!(v, vec!['d', 'b', 'c', 'a']);
    }

    #[test]
    fn final_index_matches_reorder_item() {
        for len in 1..6usize {
            for from in 0..len {
                for to in 0..=len {
                    let mut v: Vec<usize> = (0..len).collect();
                    let moved = reorder_item(&mut v, from, to);
                    let pos = v.iter().position(|&x| x == from).unwrap();
                    assert_eq!(pos, final_index(from, to), "len={len} from={from} to={to}");
                    assert_eq!(!moved, is_noop_move(from, to));
                }
            }
        }
    }

    #[test]
    fn preview_layout_drag_down_and_up() {
        use Slot::*;
        // len 5, drag item 2 down to insertion index 4 (lands at final index 3)
        assert_eq!(
            preview_layout(5, 2, 4),
            vec![Item(0), Item(1), Item(3), Placeholder, Item(4)]
        );
        // len 5, drag item 2 up to insertion index 0
        assert_eq!(
            preview_layout(5, 2, 0),
            vec![Placeholder, Item(0), Item(1), Item(3), Item(4)]
        );
        // dragging to the very end
        assert_eq!(preview_layout(3, 0, 3), vec![Item(1), Item(2), Placeholder]);
    }

    #[test]
    fn preview_layout_noop_keeps_original_slot() {
        use Slot::*;
        assert_eq!(
            preview_layout(4, 1, 1),
            vec![Item(0), Placeholder, Item(2), Item(3)]
        );
        assert_eq!(
            preview_layout(4, 1, 2),
            vec![Item(0), Placeholder, Item(2), Item(3)]
        );
    }

    #[test]
    fn preview_layout_edge_cases() {
        assert_eq!(preview_layout(0, 0, 0), Vec::<Slot>::new());
        assert_eq!(preview_layout(1, 0, 0), vec![Slot::Placeholder]);
        // stale `from` falls back to the normal order
        assert_eq!(preview_layout(2, 5, 0), vec![Slot::Item(0), Slot::Item(1)]);
    }

    #[test]
    fn preview_order_equals_committed_order() {
        // The order the user sees (placeholder standing in for the dragged item)
        // must be identical to the order after reorder_item.
        for len in 1..6usize {
            for from in 0..len {
                for to in 0..=len {
                    let slots = preview_layout(len, from, to);
                    let shown: Vec<usize> = slots
                        .iter()
                        .map(|s| match s {
                            Slot::Item(i) => *i,
                            Slot::Placeholder => from,
                        })
                        .collect();
                    let mut v: Vec<usize> = (0..len).collect();
                    reorder_item(&mut v, from, to);
                    assert_eq!(shown, v, "len={len} from={from} to={to}");
                }
            }
        }
    }

    #[test]
    fn snapshot_target_is_stable_against_shifted_rows() {
        // Snapshot is relative to the list top and never changes during the drag.
        let abs = rows(4, 40.0, 4.0);
        let snap = relative_to_top(&abs);
        assert_eq!(snap[0].min.y, 0.0);
        let a = snapshot_target(60.0, &snap);
        let b = snapshot_target(60.0, &snap);
        assert_eq!(a, b);
        // Same relative pointer position at a different scroll offset gives the same answer.
        let scrolled: Vec<Rect> = abs.iter().map(|r| r.translate(vec2(0.0, -300.0))).collect();
        let snap2 = relative_to_top(&scrolled);
        assert_eq!(snap, snap2);
    }

    #[test]
    fn auto_scroll_only_near_edges() {
        let clip = Rect::from_min_max(pos2(0.0, 100.0), pos2(200.0, 400.0));
        assert_eq!(auto_scroll_delta(pos2(10.0, 110.0), clip, 24.0, 8.0), 8.0);
        assert_eq!(auto_scroll_delta(pos2(10.0, 390.0), clip, 24.0, 8.0), -8.0);
        assert_eq!(auto_scroll_delta(pos2(10.0, 250.0), clip, 24.0, 8.0), 0.0);
    }
}
