# AGENTS.md (Level Craft) — read this first, every session

Project: LEVEL CRAFT, a level editor in Rust + egui (binary: `levelcraft`, project file: `project.lcproj`). Data model from LDtk (Project > World > Level > Layer / Entity / Enum / Tileset, world depth),
UI from Tiled (project panel + tabbed content area). Everything (levels, worlds, entity defs, enums, tilesets) opens as a TAB in the
central editor area. Side panels are movable docks. A collapsible console sits at the bottom. Entity properties float INSIDE the level canvas.

Docs are in English. Code, comments, commit messages in English.

## Read order (do not read everything; load only what the task needs)
1. This file  2. `CONSTRAINTS.md`  3. your task file in `docs/tasks/`  4. only the docs named in the task
   (`docs/01-data-model.md`, `docs/02-architecture.md`, `docs/03-ui-layout.md`, `docs/04-multi-agent-workflow.md`, `docs/decisions/*`)

## Golden rules
1. `lc_core` never depends on egui and never touches the filesystem.
2. All project mutations go through a `Command` (apply/revert/merge). UI code never mutates `Project` directly.
3. References between objects use stable IDs, never Vec indices.
4. The `Project` is the single source of truth. Tabs/panels hold only view state (zoom, pan, selection, tool).
5. Feature crates never depend on each other. Cross-feature needs go through `lc_core` types or `lc_ui_api` events/traits.
6. Edit only the files inside your task `scope`. Need a change outside it? Write it in `docs/QUESTIONS.md` (contract change) and continue with what you can.
7. Do not guess egui / egui_dock / eframe APIs. They change between versions. Check the version in `Cargo.lock` and read its docs.rs page
   or source in `~/.cargo/registry` before writing the call. If unverified, say so in the PR description.
8. No new dependency without an ADR in `docs/decisions/`.
9. Never silence a check, skip/delete a test, or loosen a lint to get green. Fix the cause. (See `CONSTRAINTS.md`.)
10. One task per session, one branch per task, one PR per branch. Stop after the task is `in_review`.

## Commands
- `scripts/board.sh` — list tasks, status, owner, active branches
- `scripts/claim.sh T-XXXX <agent-name>` — claim a task (creates branch `agent/T-XXXX` + worktree `../wt-T-XXXX`; fails if already claimed)
- `cargo xtask check` — fmt + clippy(-D warnings) + test + architecture lint. Must be green before `in_review`.
- `scripts/merge.sh T-XXXX` — integrator only; needs an APPROVE review on the branch

## Workflow summary (details: `docs/04-multi-agent-workflow.md`)
claim -> implement with tests (skills below) -> `cargo xtask check` -> set `status: in_review` -> a DIFFERENT agent reviews
(`docs/reviews/T-XXXX-review.md`) -> author fixes -> integrator merges.
Never review your own work. Never merge without an APPROVE.

## Skills (installed from addyosmani/agent-skills; invoke by name)
| Moment | Skill |
|---|---|
| Session start | `using-agent-skills`, `context-engineering` |
| Implementing | `incremental-implementation`, `test-driven-development`, `source-driven-development` |
| Touching a trait/contract/public API | `api-and-interface-design`, `doubt-driven-development`, `documentation-and-adrs` |
| Reviewing | `code-review-and-quality` (+ `code-reviewer`, `test-engineer` personas if available) |
| After it works | `code-simplification` |
| Commits | `git-workflow-and-versioning` |
| Debugging | `debugging-and-error-recovery` |
Not applicable (web-specific): `browser-testing-with-devtools`, `performance-optimization` (Core Web Vitals), accessibility/React parts of
`frontend-ui-engineering`. Use egui concepts instead. `references/testing-patterns.md` of that pack is JS/TS; use the Rust rules in `CONSTRAINTS.md`.

## Blocked or unsure
Do not stop and wait. Low-risk ambiguity: pick the simplest option, record it as a short ADR, continue.
Contract-level ambiguity (traits in `lc_core` / `lc_ui_api`, file format): append to `docs/QUESTIONS.md` and work on the unblocked part.

## Folder layout on disk (created by bootstrap)
```
level-craft/
  main/      <- this git repo (branch main)
  _ref/      <- READ-ONLY references: ldtk/, tiled/ (shallow clones, not part of the repo)
  wt-T-XXXX/ <- one git worktree per claimed task (created by scripts/claim.sh)
```
From both `main/` and any `wt-*/` the references are at `../_ref/`. Use them to understand BEHAVIOR and file formats only:
search with grep (e.g. `grep -ril "worldDepth" ../_ref/ldtk`), never read whole trees, never copy code (Tiled is GPL; LDtk is a different language anyway).
Never modify `_ref/`.

## Autopilot mode (when run by scripts/autopilot.sh; no human is available)
- You are a headless process. Never ask the human anything and never wait for input. Do exactly one role (implementer OR reviewer OR conflict-resolver) for one task, then stop.
- T-0000: do not interview; use docs/decisions/0001-defaults.md; put open questions in docs/QUESTIONS.md.
- T-0008: do the egui_dock spike for real, record it in ADR-0002. You cannot see the UI: write tests where feasible and list everything a human must eyeball in docs/MANUAL_CHECKS.md (one checkbox per behavior).
- Low-risk ambiguity: choose the simplest option, write a short ADR, continue. Contract-level ambiguity: take the most conservative option and log it in docs/QUESTIONS.md.
- The script (not you) runs the gate, merges, and decides. Do not run scripts/claim.sh or scripts/merge.sh. Do not edit files outside the task scope.
