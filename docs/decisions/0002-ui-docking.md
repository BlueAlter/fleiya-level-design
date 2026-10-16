# ADR-0002 — UI Docking Architecture (Finalized by Spike in T-0008)

## Status
ACCEPTED (Spike completed in T-0008)

## Context
`docs/03-ui-layout.md` requires:
- Central editor area holding only `DocumentView` tabs (Level, World, EntityDef, EnumDef, TilesetDef, LayerDef, ProjectSettings), dockable and splittable side-by-side.
- Side and bottom docks holding `PanelView`s (Project, Layers, Tile palette, Tools, Console), movable/reorderable between Left, Right, and Bottom docks.
- Hard constraint: Panels cannot be dropped into the editor area, and documents cannot be dropped into panel docks.
- Bottom dock must collapse to a thin header bar (tab titles visible) and expand on click or `Ctrl+``, preserving expanded height and state.
- Layout must persist and support `View > Reset Layout`.

## Spike Results (`egui_dock` 0.21.1)
1. **Can one `DockState` host both docks and a restricted central document area?**
   **No.** In `egui_dock`, all tabs in a single `DockState<T>` share the same tree and type. `DockArea` drag-and-drop allows any tab to be dropped onto any leaf node or split anywhere.
2. **Can tab drops be restricted by tab kind or target node?**
   **No.** `DockArea` and `TabViewer` lack a predicate or callback such as `can_drop(source_tab, target_node) -> bool`.
3. **Is a collapsible bottom dock region supported natively?**
   **No.** `egui_dock` has no support for collapsing a dock node into a thin header bar with visible tab buttons while remembering height.

## Decision
Adopt the dual-container architecture specified as the preferred fallback in ADR-0002:
1. **Central Document Area (`egui_dock::DockState<DocKind>`)**:
   - Uses `egui_dock` exclusively for `DocumentView` tabs.
   - Supports tab splits, side-by-side editing, reordering, and docking within the central document region.
   - Enforces the document barrier: documents can never be dragged into panel docks.
2. **Dockable Panel Containers (Left, Right, Bottom)**:
   - Left and Right docks use `egui::SidePanel` (running full height from below the menu bar down to above the status bar).
   - Bottom dock uses `egui::TopBottomPanel` (spanning ONLY the center column directly under the editor area, between the left and right docks).
   - The status bar is full width at the very bottom.
   - Each dock slot manages a tabbed strip of `PanelView` instances.
   - Panels can be moved between Left, Right, and Bottom slots via tab bar context menu ("Move to Left", "Move to Right", "Move to Bottom") or drag-and-drop.
3. **Collapsible Bottom Dock**:
   - Collapses to a thin header tab bar showing tab titles.
   - Expands upon clicking any tab header or pressing `Ctrl+``.
   - Remembers previous expanded height; collapse state and panel allocations persist across frames/restarts.
4. **Contract Invariant**:
   - `PanelView` and `DocumentView` trait signatures in `lc_ui_api` remain strictly decoupled and unaffected.

## Consequences
- Clean separation of concerns between dynamic document tabs and fixed/dockable tool panels.
- Guaranteed enforcement of the layout rules in `docs/03-ui-layout.md`.
- Layout state serialization and `View > Reset Layout` are trivial and robust.
