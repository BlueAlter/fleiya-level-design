# ADR-0015 — egui_extras TableBuilder for structured tabular UI

## Status
ACCEPTED

## Context
Structured definition editors, such as Enum Values and Attributes, require responsive tabular presentation with pinned headers, flexible column widths, virtualized scrolling, and uniform left-aligned content. The user has specifically requested using `egui_extras::TableBuilder` to structure definition tables cleanly and consistently.

Constraints:
- The project runs egui `0.36.0`.
- Golden Rule 8: No new dependency without an ADR in `docs/decisions/`.
- Architecture boundaries: `egui_extras` is a UI presentation helper and must only be added where egui UI rendering occurs (`lc_shell`). It is forbidden in `lc_core` and `lc_io`.

## Decision
1. Add `egui_extras = "0.36.0"` to workspace dependencies in root `Cargo.toml`.
2. Add `egui_extras = { workspace = true, default-features = false }` to `crates/lc_shell/Cargo.toml`.
3. Use `egui_extras::TableBuilder` inside `lc_shell::def_windows` for rendering definition tables with left-aligned column content and centered header labels.

## Consequences
- Table layouts benefit from official egui column sizing, clipping, and scrolling infrastructure.
- Zero impact on core data models or headless backend crates.
