# Review of T-0004
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0004   Commit reviewed: a1f0248

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (passed)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (passed)
  - Gate 3: cargo test --workspace (passed: 3 unit tests in lc_core, 1 integration test in lc_core/tests/round_trip.rs, 9 unit tests in xtask, doc-tests ok)
  - Gate 4: Architecture & Crate Boundaries Lint (passed)
  All gates passed with 0 warnings and 0 errors.

## Acceptance criteria evidence
- [x] All types compile with serde; round-trip tests via an in-memory serializer dev-dependency.
  - Evidence: `crates/lc_core/Cargo.toml` adds `ron = "0.12"` under `[dev-dependencies]`. `crates/lc_core/tests/round_trip.rs` implements `test_core_model_serde_round_trip` constructing a complete `Project` with all definitions (tileset, enum, entity, layers), worlds, levels, layers (IntGrid, Tiles, Entities, Auto), field schemas, and values. It serializes to RON string, deserializes back, and asserts structural equality (`assert_eq!(project, deserialized)`).
- [x] ID generator is deterministic in tests.
  - Evidence: `crates/lc_core/src/id.rs` implements `IdGenerator` with `with_seed(seed)`. Unit test `test_id_generator_determinism` in `crates/lc_core/src/lib.rs:42-59` verifies that two generators initialized with the same seed generate identical raw sequences, and validates consecutive generation across all typed IDs (`ProjectId`, `WorldId`, `LevelId`, `LayerDefId`, `EntityDefId`, `EnumDefId`, `TilesetDefId`, `EntityId`).
- [x] Lookup helpers by ID (no index-based public API).
  - Evidence: ID-based lookup methods implemented on `Project` (`world`, `world_mut`, `level`, `level_mut`, `level_and_world`, `tileset_def`, `enum_def`, `entity_def`, `layer_def`), `Definitions` (`tileset`, `tileset_mut`, `enum_def`, `enum_def_mut`, `entity`, `entity_mut`, `layer`, `layer_mut`), `World` (`level`, `level_mut`), and `Level` (`layer`, `layer_mut`, `entity`, `entity_mut`, `iter_entities`). Unit test `test_id_lookups_no_indices` in `crates/lc_core/src/lib.rs:81-132` verifies ID lookups across project, world, level, and entity layers without index-based access.
- [x] No egui, no fs in the crate.
  - Evidence: Gate 4 architecture lint validates that `lc_core` does not depend on `egui`, `eframe`, `egui_dock`, `serde_json`, or other workspace crates, and performs no filesystem access via `std::fs`. `crates/lc_core/src/lib.rs` also enforces `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]`.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  - Satisfied / Not applicable for T-0004: T-0004 establishes the pure data model. Commands and undo/redo history are scoped to T-0005.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  - Satisfied: All domain relationships use strongly-typed ID newtypes (`ProjectId`, `WorldId`, `LevelId`, `LayerDefId`, `EntityDefId`, `EnumDefId`, `TilesetDefId`, `EntityId`). No indices are used as public keys. Cascading deletion and undo restoration will be implemented in T-0005 commands.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  - Satisfied: Crate has zero egui and zero fs usage. The only added dependency is `ron` as a dev-dependency, documented and accepted in ADR-0005 (`docs/decisions/0005-in-memory-serializer.md`).
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  - N/A for T-0004 (no UI in core model).
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  - Satisfied: Type names follow PascalCase, IDs end with `Id` (`WorldId`, `LevelId`, etc.), definitions end with `Def` (`EntityDef`, etc.). `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` is enforced in `crates/lc_core/src/lib.rs`. Zero occurrences of `unwrap`, `expect`, or `panic` in `crates/lc_core/src/`.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  - N/A for T-0004 (no egui in core).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  - Satisfied: No `#[allow(...)]` or `#[ignore]` attributes. Zero skipped tests. Gates green. Scope strictly confined to `crates/lc_core/`, ADR-0005, Cargo.lock, and task tracking.
- [x] Acceptance criteria each have evidence.
  - Satisfied: All 4 acceptance criteria documented with specific test and code evidence above.

## Findings
1. [Important] `crates/lc_core/src/id.rs:43-111`: `Project` does not store the next ID counter or expose a `max_id(&self) -> u64` helper. When an existing project is loaded from disk (T-0006) or mutated via commands (T-0005), an `IdGenerator` must be seeded above the highest existing ID to prevent ID collisions. Recommend introducing a helper on `Project` (e.g. `pub fn max_id(&self) -> u64`) in T-0005 or T-0006 to seed `IdGenerator::with_seed(project.max_id().saturating_add(1))`.
2. [Nit] `crates/lc_core/src/project.rs:96-114`: `Project` provides mutable lookup helpers for worlds and levels (`world_mut`, `level_mut`), but only provides immutable lookup helpers for definitions (`tileset_def`, `enum_def`, `entity_def`, `layer_def`). While mutable access is available via `project.defs.tileset_mut(id)`, etc., adding corresponding `_mut` convenience methods on `Project` would improve ergonomic consistency.
3. [Nit] `crates/lc_core/tests/round_trip.rs:5-240`: The round-trip test covers nearly the entire data model, but omits a few variants: `FieldValue::Int`, `FieldType::Int`, `FieldType::Float`, `FieldType::Color`, and the remaining `WorldLayout` variants (`Free`, `LinearHorizontal`, `LinearVertical`). Adding these variants to the round-trip fixture in a follow-up commit will provide 100% variant coverage.
4. [Nit] `crates/lc_core/src/defs.rs:109-114`: `EntityDef` provides `field_def(&self, name: &str) -> Option<&FieldDef>`, but does not provide `field_def_mut(&mut self, name: &str) -> Option<&mut FieldDef>`.
5. [Nit] `crates/lc_core/src/grid.rs:31`: In `Grid::from_cells`, `(width as usize).saturating_mul(height as usize)` is used. Using `(width as usize).checked_mul(height as usize).ok_or(...)` would provide a more descriptive error if cell count overflows on 32-bit platforms.
6. [FYI] `crates/lc_core/src/world.rs:16-18`: `WorldLayout` uses `LinearHorizontal` and `LinearVertical` instead of the abbreviations `LinearH` and `LinearV` in the `docs/01-data-model.md` comment. This is an idiomatic improvement for Rust enum naming.
7. [FYI] `crates/lc_core/src/fields.rs:68`: `FieldValues` stores dynamic values in a `BTreeMap<String, FieldValue>`. This guarantees deterministic key order during serialization and iteration, preventing non-deterministic JSON diffs when saving projects.

## Doubt pass (required if a trait/contract/format changed)
- **Claim 1**: The core data model structs in `lc_core` faithfully reflect the domain model defined in `docs/01-data-model.md` and satisfy Phase A contract freeze requirements.
  - **Evidence**: All specified types (`Project`, `Definitions`, `World`, `Level`, `LayerDef`, `EntityDef`, `EnumDef`, `TilesetDef`, `LayerInstance`, `LayerData`, `EntityInstance`, `FieldType`, `FieldValue`, `FieldValues`, `Grid<T>`) are implemented with field types and derives matching `01-data-model.md`. All types compile with serde and round-trip cleanly via RON.
  - **Counter-argument**: Vector fields on structs (`project.worlds`, `level.layers`, `defs.entities`) are public, which could tempt external callers to bypass ID lookup helpers or break layer order conventions.
  - **Resolution**: `docs/01-data-model.md` explicitly defines these collections as public vectors to allow commands to perform reordering (e.g. `Level.layers` order = render order). All provided lookup utilities strictly operate on stable IDs, satisfying the architectural requirement that references across entities and layers avoid indices.

- **Claim 2**: Adding `ron` as a `[dev-dependencies]` dependency satisfies acceptance criterion 1 without violating architectural crate boundaries.
  - **Evidence**: ADR-0005 documents the decision to use `ron` in `[dev-dependencies]` of `lc_core`. The architecture linter in Gate 4 verifies that `lc_core` contains zero dependencies on `serde_json` and zero filesystem access. `ron` is compiled strictly for integration testing and does not appear in release binaries.
  - **Counter-argument**: Introducing any external serializer dependency to `lc_core` risks creeping dependencies into the core crate.
  - **Resolution**: Because `ron` is strictly constrained to `[dev-dependencies]`, it does not affect `lc_core`'s runtime footprint or downstream dependents (`lc_io`, `lc_ui_api`, features). It allows rigorous round-trip testing of all types without violating the architectural rule that `serde_json` is reserved for `lc_io`.

- **Claim 3**: Dense `Grid<T>` storage using row-major flat `Vec<T>` is safe against arithmetic overflow and out-of-bounds access.
  - **Evidence**: `Grid::in_bounds` validates bounds before access. `Grid::index_of` uses `checked_mul` and `checked_add`. Accesses to cells use `.get()` and `.get_mut()` with bounds checking, and clippy denies `indexing_slicing` across `lc_core`.
  - **Counter-argument**: Large dimensions could cause memory allocation failures during `Grid::new`.
  - **Resolution**: `docs/01-data-model.md` explicitly caps v1 grid dimensions at up to 1024x1024 cells per layer (~1MB), well within system limits. Safe indexing prevents panics or buffer overruns.
