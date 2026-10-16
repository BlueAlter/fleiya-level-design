# 03 — UI layout

```
┌────────────────────────────────────────────────────────────────────────┐
│ Menu bar                                                               │
├────────────┬──────────────────────────────────────────────┬────────────┤
│ LEFT DOCK  │  EDITOR AREA (documents only, tab view)      │ RIGHT DOCK │
│ Project    │  [Level_1][World A][Enum: Biome][Entity]     │ Layers     │
│ panel      │ ┌──────────────────────────────────────────┐ │ Tile       │
│ ...        │ │ canvas                    ┌───────────┐  │ │ palette    │
│            │ │                           │ floating  │  │ │ Tools      │
│ (full      │ │                           │ entity    │  │ │            │
│  height    │ │                           │ properties│  │ │ (full      │
│  from menu │ │                           └───────────┘  │ │  height    │
│  to status │ └──────────────────────────────────────────┘ │  from menu │
│  bar)      ├──────────────────────────────────────────────┤  to status │
│            │ BOTTOM DOCK: [Console] [...]                 │  bar)      │
│            │ (center column only, between left/right dock;│            │
│            │  click active tab header or Ctrl+` to toggle)│            │
├────────────┴──────────────────────────────────────────────┴────────────┤
│ Status bar: grid coords | zoom | tool | dirty indicator (full width)   │
└────────────────────────────────────────────────────────────────────────┘
```

## Panels (docks)
- Left and right docks run full height from below the menu bar down to above the status bar.
- The bottom dock (Console and any panel placed in the bottom slot) sits directly under the editor area and spans ONLY the center column, between the left dock and the right dock.
- The status bar is full width at the very bottom of the window.
- Panels (Project, Layers, Tile palette, Tools, Console, ...) are `PanelView`s. They can be dragged/moved between left, right and bottom docks and reordered/tabbed together. Layout persists to the user config dir. `View > Reset Layout` restores defaults.
- The editor area holds only `DocumentView` tabs. Panels cannot be dropped into it and documents cannot be dropped into docks.
- Collapsed state: bottom dock collapses to a ~28px header bar (tab titles visible) in the center column only, expanding the editor area to take the freed height. Expands on click or `Ctrl+``. Height is user-resizable and remembered/persisted.
- Panels show context of the ACTIVE document tab (e.g. Layers panel follows the active level tab). No active level: panel shows an empty state.

## Documents (tabs)
- Double-click in Project panel opens or focuses the tab. Same `DocKind` never opens twice.
- Entity defs, enums, tilesets, layer defs, worlds, levels are all tabs and can be split side by side (docking inside the editor area).
- Tab title shows `*` when dirty. Tab closes automatically when its target is deleted (`DocumentView::is_valid`).
- Per-tab `ViewState`: zoom, pan, active tool, selection, floating window position. Never stored in `Project`.

## Floating properties (inside the level canvas)
- An egui `Area`/`Window` constrained to the canvas rect (verify the exact API, e.g. `Window::constrain_to`, in the egui version used).
- Appears when something is selected: entity (fields editor), multiple entities (common fields), or level when nothing is selected.
- Draggable, collapsible, pin button. Unpinned: closes on Esc or when selection clears. Position remembered per tab, clamped when the canvas resizes.
- Content comes from registered `InspectorSection`s, so the entities crate owns entity fields and level crate owns level fields.
- Edits go through Commands; typing in a text field merges into one undo step per focus session.

## Console (Godot-style output)
- A `tracing` subscriber layer pushes events into a bounded ring buffer (e.g. 10k lines) read by the Console panel. Also write the same logs to
  `logs/editor.log` (rotating) so agents and humans can read them without the UI.
- Features: level filter toggles (error/warn/info/debug), text search, clear, copy, collapse identical consecutive lines (x3), auto-scroll with pause,
  click a line with a `DocKind` target to focus that tab. Default: info and above. Console never blocks the UI thread.
- Later: command input line.

## Spike required (T-0008) — verify, do not assume
egui_dock: can one DockState host both docks and a restricted central document area? Can drops be restricted by tab type? Is a collapsible bottom
region supported or must it be a custom `TopBottomPanel`? If a single DockState cannot satisfy the rules above, use the fallback in ADR-0002
(documents in egui_dock; side/bottom docks as a small custom tabbed-panel container). Record the outcome as an ADR.
