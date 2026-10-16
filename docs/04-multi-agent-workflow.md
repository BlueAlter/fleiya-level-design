# 04 — Multi-agent workflow (no human relaying)

## Setup once
```
agy plugin install https://github.com/addyosmani/agent-skills.git      # Antigravity CLI
```
(Other hosts: see the agent-skills README. In some Antigravity releases the slash-command wrappers are not discoverable; invoke skills by name.)

## Phases
- Phase A, CONTRACT (T-0001..T-0011): ONE implementer at a time, mostly sequential. Goal: data model for all domains, Command/undo, io, ui_api traits,
  shell, console. Ends with the integrator freezing the contract and tagging `contract-v0.1`.
- Phase B, FEATURES (T-01xx): MULTI-AGENT. Several agents in parallel, each in its own git worktree, each owning its own crate.
- Phase C, CROSS-REVIEW SWEEP after each batch: reviewers check consistency across crates (naming, Command usage, ID usage, logging, UI patterns).

## Task lifecycle
`todo` -> `in_progress` (claim) -> `in_review` (author done, gates green) -> `done` (integrator merged after APPROVE).
Claim = creating branch `agent/T-XXXX` (git refuses a second creation, so two agents cannot take the same task).

## How you start sessions (human does only this)
Open N terminals. Paste one prompt per terminal. Nothing is relayed through a chat.

Implementer prompt (replace ROLE and NAME):
> Read AGENTS.md. Run scripts/board.sh. Pick one `todo` task with role ROLE whose dependencies are done, run `scripts/claim.sh <ID> NAME`, and work only in
> the created worktree. Follow the skills listed in the task. Finish when `cargo xtask check` is green, set `status: in_review`, commit, and stop.

Reviewer prompt (fresh session, must not be the author):
> Read AGENTS.md and docs/reviews/_TEMPLATE.md. Run scripts/board.sh and pick a branch whose task is `in_review` and which you did not write.
> Review it with the code-review-and-quality skill and the consistency checklist. Write docs/reviews/<ID>-review.md on that branch, commit only that file, and stop.

Integrator prompt:
> Read AGENTS.md. Process docs/QUESTIONS.md, then for each reviewed branch with `Verdict: APPROVE` run `scripts/merge.sh <ID>`.
> For CHANGES_REQUESTED, set the task back to `in_progress` on that branch. After a batch, run a consistency sweep and open follow-up tasks.

## Cross-checking so output stays consistent (three layers)
1. Mechanical (cannot be argued with): `cargo xtask check` — fmt, clippy -D warnings, tests, architecture lint on the dependency matrix, golden file test.
2. Contract in code: traits in `lc_core` and `lc_ui_api` are the only agreed interface. Features implement them; they do not invent parallel ones.
3. Review by a different agent: consistency checklist below, using `code-review-and-quality`. For contract-touching changes, also `doubt-driven-development`
   (adversarial pass with fresh context).

## Consistency checklist (reviewer fills it in `docs/reviews/<ID>-review.md`)
- Mutations only via Command; Commands have apply/revert/merge tests.
- IDs not indices; deletion/cascade handled; undo restores identical state.
- No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
- New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
- Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
- Egui API calls verified against the pinned version (no hallucinated methods).
- No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
- Acceptance criteria each have evidence.

## Contract change protocol
Feature agent needs a new field/trait method -> appends to `docs/QUESTIONS.md` with a concrete proposal -> integrator creates a `contract` task,
merges it first, features rebase. Features never edit core/ui_api inside their own task.

## Conflict handling
Each agent works in its own worktree and crate, so conflicts are rare. Shared files that do conflict: `Cargo.lock`, shell `register` list, docs.
On conflict the later-merging branch rebases on main and re-runs `cargo xtask check`.
