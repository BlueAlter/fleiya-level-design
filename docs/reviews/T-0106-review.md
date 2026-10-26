# Review of T-0106
Reviewer: independent-reviewer   Author: opencode   Branch: agent/T-0106   Commit reviewed: 8bee4727d127a2ab6aa4886d7b0c570bf1e3a7ec

Verdict: APPROVE

## Gates
- `cargo xtask check` (post-fix): **pass**
  - Gate 1 `cargo fmt --all -- --check`: clean
  - Gate 2 `cargo clippy --workspace --all-targets -- -D warnings`: clean
  - Gate 3 `cargo test --workspace`: all pass; `lc_feature_entities` now has 7 unit tests + 18 integration tests (was 16), no ignored/filtered tests
  - Gate 4 architecture & crate-boundary lint: clean (no `egui`/`fs` in `lc_core`, no feature-to-feature dependency, no forbidden dependency)
- `git diff --check 8e98b3b..HEAD`: clean
- No clippy or coverage regression from the fix; the only changed tests are additions and full-snapshot strengthening.
- Final confirmation at `8bee472`: only the crate-root `entity_choices` re-export and the T-0106 entries in `docs/MANUAL_CHECKS.md` were added (plus this review document); the gate was re-run and remains green.

## Re-review scope
Re-review covered only `git diff 8e98b3b..HEAD` (`04eb57a`, the fix commit), plus a final confirmation pass over `git diff 04eb57a..8bee472`.

| File | Change |
|---|---|
| `crates/lc_feature_entities/src/commands.rs` | `RefOwner` enum; cascade now sweeps level fields and entity field defaults; restore dispatches by owner |
| `crates/lc_feature_entities/src/field_commands.rs` | Add/update now normalize `FieldDef::default_value`; field names are trimmed; dead `inserted_index` removed |
| `crates/lc_feature_entities/src/field_defs.rs` | Color added to the default-value test |
| `crates/lc_feature_entities/src/field_editors.rs` | `entity_choices` made public for T-0107 reuse |
| `crates/lc_feature_entities/src/view.rs` | Uses the shared `entity_choices` |
| `crates/lc_feature_entities/tests/entities_test.rs` | Full-project snapshots; two new regression tests |
| `docs/tasks/T-0106-entity-defs.md` | Notes updated to describe the fixed behavior |
| `docs/reviews/T-0106-review.md` | My previous review was included in the author's commit (docs only) |

All product changes are inside `crates/lc_feature_entities`. No new out-of-scope product edits were introduced by the fix commit; the sanctioned one-line `lc_shell` registration from the original commit is unchanged.

I re-ran the external probes from the first review (`/tmp/opencode/t0106probe`, outside the repo; no product files touched) against the fixed code: schema-default migration, level-field refs, nested `Array(Array(EntityRef))` refs in both level fields and entity field defaults, multi-layer cascade byte-identical restore, and add-field default normalization all pass.

## Acceptance criteria evidence
- [x] Field editors are plain functions usable by the floating inspector (T-0107) and tab.
  - `crates/lc_feature_entities/src/field_editors.rs:67` (`field_value_editor`) and `:245` (`field_type_editor`) are free functions taking `&mut Ui`, a scratch value/type and `FieldEditorContext`; no view/state ownership. Exported from `crates/lc_feature_entities/src/lib.rs:21-24`.
  - All nine `FieldType` variants are handled, including recursive `Array` editing and `EntityRef` selection from a caller-supplied `EntityChoice` list. `entity_choices()` is now public (`field_editors.rs:360`) so T-0107 can build the same context.
  - Rendered headlessly for every type by `field_editors_render_every_field_type_headlessly` (`crates/lc_feature_entities/tests/entities_test.rs:574-612`) and `field_type_editor_reports_no_change_without_interaction` (`:614`).
  - Remaining non-blocking gap: the tests verify rendering/no-op behavior but do not drive a widget to assert the new value is written (see Findings).
- [x] Add/edit/remove field defs via Commands with revert tests; value migration when a field type changes is defined and tested.
  - `AddEntityFieldCommand` / `UpdateEntityFieldCommand` / `RemoveEntityFieldCommand` in `crates/lc_feature_entities/src/field_commands.rs`; entity-definition lifecycle in `commands.rs`.
  - **Fixed:** `AddEntityFieldCommand` normalizes a supplied default (`field_commands.rs:82-86`) and `UpdateEntityFieldCommand` coerces the stored default to the new type (`field_commands.rs:174-178`). Regression test: `changing_field_type_migrates_the_stored_default_value` (`tests/entities_test.rs:673`), which feeds `Int(10)` into a `String` field and asserts `String("10")`, then undo restores the snapshot.
  - Merge is restricted to matching strokes and the same target: `UpdateEntityDefCommand::merge` (`commands.rs:156-167`) and `UpdateEntityFieldCommand::merge` (`field_commands.rs:276-292`, additionally same `def_id`, same field type, chained names); add/remove return `false`.
  - Type change and rename migrate every instance through `for_each_instance_mut` + `coerce_field_value`; undo restores values through `FieldMigration` and the original schema slot (`field_commands.rs:171-274`).
  - Field names are now trimmed before storage (`field_commands.rs:43`), removing the previous `" speed "` inconsistency.
- [x] Removing an EntityDef blocks while instances exist, cascade removes instances and clears dangling `EntityRef`s in instances, level fields and entity field defaults (including nested arrays), and undo restores everything.
  - `RemoveEntityDefCommand::safe/cascade` (`commands.rs:184-320`) blocks before mutating; cascade removes instances and records `(world, level, layer, index)`; revert re-inserts in the inverse order, restoring the original per-layer order.
  - **Fixed:** `RefOwner` (`commands.rs:23-41`) now distinguishes `Level`, `EntityDefault` and `EntityInstance` owners. `scrub_dangling_refs` (`commands.rs:328-431`) walks level fields, then entity field defaults, then surviving entity instances; `restore_cleared_refs` (`commands.rs:434-473`) restores each owner by path. Arrays are handled recursively with the same owner.
  - Regression test: `cascade_clears_refs_in_level_fields_and_field_defaults` (`tests/entities_test.rs:708`) covers a level field and an entity field default, asserting both become `EntityRef(None)` and that undo restores the full snapshot.
  - My probe additionally verified nested `Array(Array(EntityRef))` refs in level fields and entity defaults are scrubbed and restored byte-identically, and that multiple entity layers/instances restore at their original indices.
- [x] Undo tests assert byte-identical project state.
  - `assert_eq!(project, snapshot)` after undo now covers: add entity def (`tests/entities_test.rs:168`), merged update (`:196`), cascade (`:269`), cascade refs (`:306`), add field (`:362`), type change (`:394`), rename (`:432`), remove field (`:466`), field merge (`:496`), default migration (`:673`), cascade level/default refs (`:708`).
  - Remaining nit: `update_entity_def_without_matching_stroke_keeps_two_steps` (`:235`) still compares only the def, not a full project snapshot; it is a narrow test and not blocking.
- [x] Architecture rules.
  - `lc_core` is untouched; the feature crate only uses `lc_core`, `lc_ui_api`, `lc_canvas`, `egui` and the existing workspace `glam`; architecture lint is green.
  - The one-line `crates/lc_shell/src/shell_app.rs:92` registration is explicitly sanctioned by `docs/02-architecture.md:60`; `Cargo.lock` is the automatic result of adding the existing `glam` workspace dependency. Both remain justified.
- [x] No silenced lint, skipped test or weakened gate: no new `#[allow]`, `#[ignore]`, `cfg(not(test))`, `todo!`/`unimplemented!`/`dbg!` in the crate.
- [x] Acceptance criteria each have evidence.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — UI clones a draft and only emits commands; the undo tests now use full project snapshots.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — `RefOwner` + path-based restore covers all three owner kinds; probe confirmed order and byte identity.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — Gate 4 green; `glam` is an existing workspace dependency.
- [x] New view uses `DocumentView` / `PanelView` as defined; registered through `register()` — `EntityDefDocumentView` implements `DocumentView`; factory registered in `lib.rs:31-40` and in the shell.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Consistent naming; `UndoStack` logs command application at debug.
- [x] egui API calls verified against the pinned version — Compiles and headless `egui::Context::run_ui` tests pass on egui 0.36.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope — only feature-crate files changed by the fix; docs changes are task notes, manual checks, and the captured review.
- [x] Acceptance criteria each have evidence.

## Findings

### Resolved in 04eb57a
1. **[Resolved][Important] Schema `default_value` was not migrated with the field type.**
   - Fixed in `field_commands.rs:82-86` (add) and `:174-178` (update); covered by `tests/entities_test.rs:673` and re-verified with an external probe.
2. **[Resolved][Important] Cascade left dangling `EntityRef`s in level fields and entity field defaults.**
   - Fixed by `RefOwner` (`commands.rs:23-41`), the extended sweep (`:328-431`) and owner-aware restore (`:434-473`); covered by `tests/entities_test.rs:708` and re-verified for nested arrays with an external probe.
3. **[Resolved][Important] Revert tests did not compare a full `Project` snapshot.**
   - All command tests now assert `project == snapshot` after undo; the cascade test was also renamed from "unreferenced" to `..._blocks_while_used_...`.
4. **[Resolved][Nit] `AddEntityFieldCommand::inserted_index` was dead state.** Removed.
5. **[Resolved][Nit] `default_value` test omitted `Color`.** Added in `field_defs.rs:203`.
6. **[Resolved][Nit] Field names were stored untrimmed.** `validate_field_name` now trims (`field_commands.rs:43`).

### Resolved in 8bee472
7. **[Resolved][Optional] Field editors had no interaction-level test or manual checklist entry.**
   - T-0106 manual checks were added in `docs/MANUAL_CHECKS.md:166-176` covering metadata, field add/rename/type-change/delete, all nine editor types, and cascade deletion. No `egui_kittest` interaction test was added, which is acceptable under `CONSTRAINTS.md:24` (the manual checklist is the documented alternative).
8. **[Resolved][Nit] `entity_choices` was missing from the crate-root re-exports.**
   - Now exported from `crates/lc_feature_entities/src/lib.rs:21-24`; T-0107 can use `lc_feature_entities::entity_choices` directly.

### Remaining (non-blocking)
9. **[Nit] One undo test still lacks a full project snapshot.**
   - `update_entity_def_without_matching_stroke_keeps_two_steps` (`tests/entities_test.rs:235-265`) asserts the def equals the original after both undos but not `project == snapshot`. The command only mutates that def, so the risk is low; adding the snapshot would complete the pattern.
10. **[Optional] The new cascade regression test does not itself use a nested array.**
    - `tests/entities_test.rs:708` covers a scalar level field and a scalar default; the shared recursive scrubber and the existing instance-array test cover arrays, and my external probe confirmed nested arrays in both new owner kinds. A nested-array case in this test would make the new behavior self-documenting in-repo.

### FYI
- Both `04eb57a` and `8bee472` included revisions of `docs/reviews/T-0106-review.md` (the file was untracked/modified in the worktree and got picked up). It is documentation only; this final update is left uncommitted, as requested. No product code was modified by this review.
- The diff size remains above the 300-line guideline (`CONSTRAINTS.md:36`), but it is one cohesive feature and matches the assigned task scope.
- `commands.rs` is now 489 lines, just under the 500-line split threshold; a future cascade extension should split the reference-sweep helpers into their own module.

## Doubt pass (cascade reference sweep)
- **Claim:** cascade deletion removes every reference to the removed instances and undo restores them exactly.
- **Evidence:** `RefOwner`-aware sweep covers level fields, entity field defaults and surviving entity instances; nested arrays are handled by the shared recursive `scrub_value`; `restore_cleared_refs` dispatches by owner and path. `cascade_clears_refs_in_level_fields_and_field_defaults` plus my external probes confirm byte-identical restore, including nested arrays and multi-layer instance order.
- **Counter-argument:** "Do default values count as live references?" The task notes now explicitly claim they do, `FieldDef.default_value` stores a `FieldValue`, and the default editor can select a live entity, so a dangling default would seed an invalid instance later. Treating defaults as references is the safer and now-tested behavior.
- **Resolution:** resolved. The earlier "`EntityRef`s only live on entity instances" counter-argument was wrong given the serialized level fields and defaults, and the fix addresses it.

## Suggestions for the author
- Add the missing full-project snapshot in the non-matching-stroke test (Finding 9) and, optionally, a nested-array case in the new cascade test (Finding 10); neither blocks merge.
