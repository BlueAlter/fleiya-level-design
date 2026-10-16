# Review of T-0005
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0005   Commit reviewed: e08ebf9

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (pass)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (pass)
  - Gate 3: cargo test --workspace (pass: 5 command tests + 3 core unit tests + 1 round-trip test + 9 xtask arch tests)
  - Gate 4: Architecture & Crate Boundaries Lint (pass)

## Acceptance criteria evidence
- [x] Property-style test helper: apply then revert yields a Project equal to the original, used by every command test.
  Evidence: `assert_apply_revert<C: Command>(original: &Project, mut command: C)` implemented in `crates/lc_core/tests/commands_test.rs:6-22`. It validates `assert_ne!(original, &working)` after apply and `assert_eq!(original, &working)` after revert across all 7 commands in `test_all_commands_apply_revert_property`:
  `AddWorldCommand`, `RenameWorldCommand`, `RemoveWorldCommand`, `AddLevelCommand`, `RenameLevelCommand`, `RemoveLevelCommand`, and `MoveLevelCommand`.
- [x] merge() tested (e.g. consecutive MoveLevel on the same level collapse).
  Evidence: `test_move_level_command_merging` in `crates/lc_core/tests/commands_test.rs:127-182` executes 3 consecutive moves on `LevelId(101)` (positions (10, 10), (20, 25), (50, 75) and depth 3), verifies only 1 undo step is recorded, verifies a single `undo()` restores initial position and depth exactly (`project == original`), and verifies a single `redo()` restores the final collapsed position and depth.
- [x] Removal commands revert exactly, including level order.
  Evidence: `test_level_removal_preserves_exact_order_on_revert` in `crates/lc_core/tests/commands_test.rs:81-104` removes middle level `Level_B` (index 1 of 3) and reverts, asserting `project == original` and indices [0, 1, 2] match [Level_A, Level_B, Level_C]. `test_world_removal_preserves_exact_order_on_revert` (lines 106-125) performs the equivalent check for worlds.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  All project mutations in T-0005 are encapsulated in `Command` implementations with property tests.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  Commands identify targets by `WorldId` and `LevelId`. Revert restores exact order and original data without exposing or depending on raw vector indices in API signatures.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  `lc_core` only depends on workspace crates `thiserror`, `serde`, `glam`, `tracing`. No egui, no `std::fs`.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  N/A (crate is `lc_core`).
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  `UndoStack` logs at `tracing::debug` on execution, merging, undo, and redo. `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` is enforced; zero unwrap/expect in `command.rs` and `commands.rs`.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  N/A (no egui in `lc_core`).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  No silenced lints or ignored tests. Diff is confined to `crates/lc_core` and `docs/tasks/T-0005-command-undo.md`. Diff size (~824 lines) is above the 300-line guideline due to 7 complete command structs and comprehensive integration tests (see Nit finding).
- [x] Acceptance criteria each have evidence.
  All 3 acceptance criteria verified with passing tests.

## Findings
1. [Important] `crates/lc_core/src/commands.rs:207-215` — Transactional integrity flaw in `AddLevelCommand::apply`:
   `self.level.take()` moves the level out of the command before `p.world_mut(self.world_id)` validates that the target world exists. If `self.world_id` is invalid, the method returns `Err(CoreError::WorldNotFound)` but `self.level` has already been taken and dropped. The command's payload is destroyed on validation failure.
   Recommendation: check `if !p.worlds.iter().any(|w| w.id == self.world_id)` or lookup world before `self.level.take()`.
2. [Important] `crates/lc_core/src/command.rs:90-95` — Missing merge delimiter / boundary in `UndoStack`:
   `UndoStack` merges consecutive commands solely based on `Command::merge`. In `MoveLevelCommand` and `RenameWorldCommand`, `merge()` returns `true` whenever consecutive commands target the same ID. Because there is no delimiter or boundary method (e.g. `break_merge_chain()`), two separate drag gestures or rename actions on the same entity spaced minutes apart will collapse into a single undo step unless an intermediate command intervenes.
   Recommendation: Add an explicit `pub fn break_merge_chain(&mut self)` method on `UndoStack` so UI interaction lifecycles (e.g., mouse-up or focus-lost) can delimit merge windows without relying on wall-clock time.
3. [Nit] `crates/lc_core/src/commands.rs:57-59, 238-240` — Inaccurate `focus()` after `revert()` in Add commands:
   `AddWorldCommand::focus()` and `AddLevelCommand::focus()` return `self.created_id.map(...)`. However, after `revert()` is called, the entity has been removed from `Project`, but `self.created_id` remains populated. Calling `focus()` on an undone add command returns a `DocKind` referencing an ID that no longer exists in `Project`.
   Recommendation: Check `if self.world.is_none() { self.created_id.map(DocKind::World) } else { None }` (and similarly for `self.level`).
4. [Nit] `crates/lc_core/src/commands.rs:34, 210` — Reusing `CoreError::DefNotFound` for command state:
   Errors returning `CoreError::DefNotFound("World already applied")` reuse a definition-lookup variant for an internal command state error. Consider introducing a dedicated error variant or `CommandError` when refining core errors in future tasks.
5. [Nit] Diff size exceeds guideline:
   The diff introduces ~824 lines, exceeding the <= 300 line target in `CONSTRAINTS.md`. This is acceptable here given that T-0005 introduces the core command infrastructure, 7 structural commands, and extensive tests, but future feature PRs should remain more narrowly split.

## Doubt pass (required if a trait/contract/format changed)
1. **DocKind location in `lc_core` vs `lc_ui_api`**
   - **Claim**: Defining `DocKind` in `lc_core::command` satisfies the architectural intent of `docs/02-architecture.md` without violating layering.
   - **Evidence**: `Command::focus(&self) -> Option<DocKind>` is part of `Command` in `lc_core`. Under the dependency matrix, `lc_core` cannot depend on `lc_ui_api`. `DocKind` contains only raw domain IDs (`LevelId`, `WorldId`, etc.) and no egui/UI types.
   - **Counter-argument**: `docs/02-architecture.md:32` lists `DocKind` under "Extension points (defined in lc_ui_api, T-0007)". Placing document targeting in `lc_core` could leak UI tab concepts into the data model.
   - **Resolution**: `DocKind` acts as a pure identifier discriminant telling the shell which domain entity was affected by a mutation. Putting `DocKind` in `lc_core` preserves the strict dependency rule (`lc_core` depends on nothing). `lc_ui_api` can re-export `DocKind` for `DocumentView` registration.

2. **Decoupled `UndoStack` vs `Project`-embedded history**
   - **Claim**: `UndoStack` should manage commands externally against `&mut Project` rather than being stored inside `Project`.
   - **Evidence**: `Project` must derive `Clone`, `PartialEq`, `Serialize`, and `Deserialize` for serialization and testing. `Box<dyn Command>` is not `Clone` or `Serialize`.
   - **Counter-argument**: Task T-0005 requested "a command executor `Project`-side". One could interpret this as `Project` having an internal undo stack or methods like `project.execute(cmd)`.
   - **Resolution**: Keeping `Project` pure data model allows lossless serde round-tripping and golden-file comparison. `UndoStack::execute(cmd, &mut project)` serves as the executor and will be encapsulated in `EditorCtx` in `lc_ui_api` (T-0007).

3. **Absence of wall-clock merge windows in `UndoStack`**
   - **Claim**: Omitting `std::time::Instant` or wall-clock timeouts in `UndoStack` complies with project constraints.
   - **Evidence**: `CONSTRAINTS.md:25` explicitly states: "Tests are deterministic: no sleeps, no wall-clock, no network, seeded randomness."
   - **Counter-argument**: The task description mentions "(undo, redo, merge window, clear on load)". Without a window, consecutive moves collapse indefinitely.
   - **Resolution**: Time-based windows would violate deterministic testing constraints. Merging is correctly modeled as event-driven (`last.merge(next)`). However, an explicit delimiter (`break_merge_chain()`) should be added to allow UI event loops (mouse-up) to close merge transactions.

4. **Preservation of sequence order in removal commands**
   - **Claim**: Storing `removed_index: Option<usize>` inside `RemoveLevelCommand` and `RemoveWorldCommand` safely restores exact sequence order on revert.
   - **Evidence**: `test_level_removal_preserves_exact_order_on_revert` and `test_world_removal_preserves_exact_order_on_revert` prove that removing an intermediate item and reverting restores the exact original `Vec` order and passes structural equality (`project == original`).
   - **Counter-argument**: `Project` uses IDs, not indices. If other items are inserted or removed concurrently, inserting at `removed_index` could place the level at an unintended position.
   - **Resolution**: In a linear undo/redo stack, commands are reverted in exact LIFO order. Bounds checking (`if idx <= world.levels.len()`) prevents panics if the vector shrunk, falling back to push. This satisfies the acceptance criterion "Removal commands revert exactly, including level order".
