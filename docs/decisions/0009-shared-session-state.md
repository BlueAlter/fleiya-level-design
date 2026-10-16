# ADR-0009 — Shared Session State Readable via EditorCtx

## Status
ACCEPTED

## Context
Phase B feature crates (T-0102 Tileset, T-0103 Tile Painting, T-0104 Layer Defs, T-0107 Entity Layer) require read access to active editor session state:
- Which document tab is active (`DocKind`)
- Which level is being edited (`LevelId`)
- Which layer is selected in the layers panel (`LayerDefId`)
- Current document selection (`Selection`)
- Active tool name (`String`)
- Active tile palette selection (`TileSelection`)

Under clean architecture, feature crates must not mutate shared session state directly, nor should transient UI focus/selection pollute the persistent `Project` data model or command undo stack.

## Decision
1. Introduce `SessionState` and `TileSelection` in `lc_ui_api::session`.
2. Provide read-only access to `SessionState` through `EditorCtx::session(&self)` and ergonomic helper getters (`active_document`, `active_level`, `active_layer`, `selection`, `active_tool_name`, `tile_selection`).
3. Shell owns `SessionState` in `ShellApp`.
4. State mutations occur strictly via `EditorEvent`s (`SetActiveLevel`, `SetActiveLayer`, `SetActiveTool`, `SetTileSelection`, `SelectionChanged`, `OpenDocument`, `FocusDocument`) or command post-execution `focus()` hooks, never direct mutation.

## Consequences
- Single source of truth for UI focus across panels, canvas tools, and inspector sections.
- Feature crates remain decoupled and testable in isolation using `SessionState::default()`.
- Zero project model pollution from ephemeral UI states.
