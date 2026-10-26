# Review of T-0107 (re-review)

Reviewer: independent-reviewer   Author: opencode   Branch: `agent/T-0107`
Re-reviewed commit: `3b176666f00906250109682e1a2c64e17eb374cd` (fixes for the prior review at `57628fe`; original implementation `78359eb`)

Verdict: APPROVE

All four blocking findings from the first review are fixed and covered by tests. The gate is green at the re-reviewed commit. The remaining items below are non-blocking follow-ups (one contract decision already logged in `docs/QUESTIONS.md`, two minor UX/undo-granularity notes).

## Gate

- `cargo xtask check` at `3b17666` on a clean worktree: **pass**, exit 0; final line
  `=== ALL GATES PASSED: fmt, clippy, test, architecture lint ===`.
- `cargo test -p lc_feature_entities`: lib **13**, `entities_test` **18**, `entity_layer_test` **21**; all pass, 0 ignored/filtered.
- No manifest/lock change and no new dependency. No `lc_core` / `lc_ui_api` contract change.
- Worktree clean before and after the gate; only test/build artifacts were produced.

## Prior findings

| # | Prior finding | Status | Evidence |
|---|---|---|---|
| 1 | Critical: Delete/Backspace/Ctrl+D fired while a field had keyboard focus | **FIXED** | `tools.rs:65-84` returns early on `cv.response.ctx.egui_wants_keyboard_input()`. `entity_layer_test.rs:980-1009` uses a real focused `TextEdit` and asserts no command/event; `:1012-1050` proves the same Backspace deletes when nothing is focused. `TextEdit` reads events through `filtered_events` without consuming them from `InputState` (`egui-0.36.2/src/widgets/text_edit/builder.rs:1098`), so the first test would fail without the guard. |
| 2 | Important: a paused numeric drag split one undo stroke | **FIXED** | `FieldEditorResponse.dragging` + `keeps_stroke()` (`field_editors.rs:55-78`), propagated through array editors (`:233`); the inspector uses it for fields (`inspector.rs:137-154`) and `dragged()` for POSITION/SIZE (`:233-238`, `:257-262`). `entity_layer_test.rs:1053-1119` asserts the drag frame, a no-change pause frame that is still dragging, and the same stroke id through `FocusStrokeTracker`. |
| 3 | Important: stale/cross-level selection could display or edit another level | **FIXED** | The inspector resolves `cx.active_level()` and filters ids through `entities_in_level` (`inspector.rs:73-89`, `:295-308`); shift-click seeds from `selected_in_level` (`tools.rs:195-237`). Tests: `inspector.rs:391-437`, `entity_layer_test.rs:1122-1160`. The root per-document-selection decision is recorded in `docs/QUESTIONS.md:11`. |
| 4 | Important: duplicate always moved copies to the active Entities layer | **FIXED** | `duplicate_selected` groups copies by each source entity's `entity_layer_of` (`tools.rs:99-151`). `entity_layer_test.rs:1163-1250` selects an entity on a non-active layer and asserts the copy stays there. |
| 5 | Important/contract: freed highest instance id can be reused | **ACCEPTED FOLLOW-UP** | Still `max(live id) + 1` (`helpers.rs:98-113`); now explicitly logged in `docs/QUESTIONS.md:10` and the task notes, not claimed fixed. Correct disposition until a persisted allocator / UUID-like IIDs are designed. |
| 6 | Mixed-value fallback used the type zero value instead of the definition default | **FIXED** | One shared `effective_value` (`inspector.rs:281-293`) is used for both the seed and the mixed comparison (`:123-127`). |
| 7 | Inspector never resets its stroke on an empty selection | **STILL OPEN (non-blocking, was optional)** | The host still calls `ui` only when `applies` is true, so the `Selection::None` reset at `inspector.rs:57-60` remains unreachable from the real host. The destructive delete-while-focused path is gone, so this is now only relevant to event-driven deselect/reselect of identical ids. |
| 8 | Modules over 500 lines | **FIXED** | `wc -l`: `instance_commands.rs` 407, `tools.rs` 467, `inspector.rs` 438, `field_editors.rs` 405, new `geometry.rs` 94, new `field_instance_commands.rs` 128. No source module > 500. |
| 9 | Nits: unused `inserted`, dead `Frame::clicked()`, duplicate label arm | **FIXED** | `inserted` removed from `instance_commands.rs`; the ineffective Frame response check is gone from `panel.rs`; `RemoveEntityInstancesCommand::label` now has plural-aware single arms (`instance_commands.rs:203-208`). |
| 10 | Missing interaction/merge/array tests | **FIXED** | Merge-false for Add/Remove: `entity_layer_test.rs:1253-1278`. Nested arrays in delete: instance array and level array at `:260-309`, with full-project undo snapshot. Shortcut, drag, cross-level shift-click and duplicate tests added (above). |
| 11 | FYI: shell test assertion update | **UNCHANGED / valid** | The exact `[project, entities]` list assertion remains stricter than the old length check. |
| 12 | FYI: scope deviation and selection feedback wording | **FIXED (wording)** | `docs/QUESTIONS.md:7` now says the floating inspector shows fields only and there is no canvas outline, matching reality. The shell-wiring deviation is still disclosed and accepted for this task. |

## New findings (non-blocking)

1. **[Minor] A multi-layer duplicate is split into one command per layer, so it is more than one undo step.**
   `duplicate_selected` executes `AddEntityInstancesCommand` per `(layer, copies)` group (`tools.rs:140-146`), and the shell pushes each pending command as its own history entry (`crates/lc_shell/src/shell_app.rs:441-458`). Selecting entities on two Entities layers and pressing Ctrl+D therefore needs two undos. Single-layer duplicate (the tested and manual path) is still one command. A compound/multi-layer add command, or a note in the task/manual docs, would close this; it is not destructive.
2. **[Minor/UX] A stale cross-level selection leaves the floating PROPERTIES window empty instead of showing B's level summary.**
   The inspector correctly refuses to render ids outside the active level and returns without drawing (`inspector.rs:85-89`), but `render_floating_inspector` sets `any_rendered = true` whenever `section.applies(&sel)` is true (`crates/lc_feature_level/src/canvas.rs:322-331`), and `applies` only sees the selection (`inspector.rs:52-53`). After switching from A to B with A's ids, the fallback level summary is skipped and the window has no content. The new manual check says "B's properties show level B" (`docs/MANUAL_CHECKS.md:196`); literally it shows nothing. Safe (A is not shown or edited), but either update that check to "B must not show or edit A's entity" or draw a short "selection is in another level" hint. The latter is a small in-scope inspector change; a host-side `any_rendered` fix needs an inspector-contract change.
3. **[FYI] Split dock surfaces can still display the focused level's inspector in a non-focused visible level tab.**
   `DocTabViewer` renders every visible tab, while `session.active_level` is the focused leaf (`crates/lc_shell/src/shell_app.rs:402-408`); the inspector reads `cx.active_level()` rather than the level passed to the canvas. A non-focused visible level tab can therefore show the focused level's entity fields. A click in that tab moves focus to it at the end of the frame (egui_dock `new_focused`), so a subsequent edit targets that tab's level; I did not find a destructive silent-cross-level-edit path. This is the same root per-document-selection issue logged in `docs/QUESTIONS.md:11`.
4. **[FYI] The shortcut guard is deliberately conservative.** `egui_wants_keyboard_input()` is true for any focused widget, not only text edits (`egui-0.36.2/src/context.rs:2986`), so Delete/Ctrl+D are ignored while, for example, a `DragValue` still holds keyboard focus after a click. Clicking the canvas surrenders focus (egui's `surrender_focus_on` handling), so normal select-then-delete still works. No action needed.

## Regression checks

- **Focus guard vs. menus/buttons:** egui 0.36.2 only calls `request_focus` from `DragValue` and `TextEdit`, so menus do not hold keyboard focus and do not block shortcuts. No regression found.
- **Headless inspector:** the `active_level().or_else(...)` fallback (`inspector.rs:73-79`) keeps the existing headless render test (`entity_layer_test.rs:762-792`, `SessionState::default()`) working. Gate confirms.
- **Duplicate order/IDs:** ids are allocated sequentially from `next_entity_id` before execution; copy order follows the filtered selection; one `SelectionChanged` is emitted with the new ids. Source-layer placement is verified.
- **Borrow/API/semantics:** the command and helper splits are re-exported from `lib.rs:35,45-48`; no public type disappeared, clippy is clean, and no unsafe/allow/ignore was added.
- **Architecture:** no new cross-feature dependency, no egui in `lc_core`, no fs outside `lc_io`.

## Manual checks still required

- The four T-0107 additions at `docs/MANUAL_CHECKS.md:194-197`, with the empty-window note above for the cross-level check.
- The full T-0107 checklist at `docs/MANUAL_CHECKS.md:178-197`.
- Optional: split two level tabs side by side and confirm the non-focused tab's PROPERTIES window does not mislead (see FYI 3).

## Verdict

**APPROVE.** The critical focus bug, the paused-drag undo split, the cross-level edit path and the duplicate target-layer bug are genuinely fixed and tested; the gate passes at `3b17666`. ID reuse is correctly logged as a contract follow-up. The new items are non-blocking and should be tracked for a shell/contract or polish task.
