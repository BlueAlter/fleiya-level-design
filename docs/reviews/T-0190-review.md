# Review of T-0190 — Cross-crate consistency sweep

Reviewer: opencode (independent review pass in a separate session from the author)
Author: opencode (fresh-context subagent implementation session)
Branch: `agent/T-0190`

Verdict: APPROVE

## Summary
This review supersedes an earlier same-session sign-off on the branch. The sweep was re-checked independently against the diff, the acceptance criteria, and a fresh gate run.

The sweep covers stroke-id deduplication, command label normalization, removal of dead/redundant state and unused dependencies, and documentation drift. It also creates Phase C follow-up tasks.

## Gates
- `cargo xtask check` re-run on `agent/T-0190` after review: **ALL GATES PASSED** (fmt, clippy, test, architecture lint).
- No new dependencies; two unused `tracing` declarations were removed.
- No tests were deleted or weakened.

## Findings checked
1. **Stroke ids centralized** — `lc_canvas::ids::next_stroke_id` replaces per-crate atomic counters; `lc_feature_entities::stroke` re-exports it and tileset tools import it. Unit test covers uniqueness.
2. **Command labels normalized** — schema edits use "Edit", deletions use "Remove", and ids render as `#<raw>` instead of debug formatting. The enum test was updated to match the new label.
3. **Layer classification deduplicated** — `layer_kind_of` was removed in favour of `Layer::kind()`; no remaining references.
4. **Unused dependencies removed** — `tracing` dropped from `lc_feature_layers` and `lc_feature_tileset`, with matching `Cargo.lock` cleanup.
5. **Shell state deduplicated** — `ShellApp::active_tool_name` removed; status bar and tools already read `session.active_tool_name`.
6. **Docs aligned** — `02-architecture.md` now documents `Command::as_any` and the explicit level-tab assembly exception; `03-ui-layout.md` documents the current floating-window and per-tab state behaviour, with the contract gaps linked to `docs/QUESTIONS.md`.
7. **Follow-up tasks** — T-0205 (multi-tile selection/copy-paste), T-0206 (auto-layer rules), T-0207 (preferences/shortcuts), and T-0208 (export pipelines) exist with `todo` status, scope, deps, and acceptance criteria.

## Non-blocking notes
- The one new `#[allow(clippy::too_many_arguments)]` carries a one-line justification, as required by `CONSTRAINTS.md`.
- T-0205–T-0208 are definitions only; they still need their own implementation, gate, and review cycles.
