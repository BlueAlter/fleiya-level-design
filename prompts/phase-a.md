You are the sole agent for Phase A of Level Craft. Work autonomously from T-0000 to T-0011, then STOP. Never ask me anything; I am away.
Read AGENTS.md first (including "Autopilot mode") and follow it.

LOOP until T-0011 is `done`:
1. Run scripts/board.sh. Take the lowest-numbered `todo` task in T-0000..T-0011 whose dependencies are all `done`.
2. scripts/claim.sh <ID> agy-1, then work ONLY in ../wt-<ID>.
3. Follow the skills listed in the task file. Verify every egui/eframe/egui_dock API against the pinned version (docs.rs or ~/.cargo/registry) before using it. Use ../_ref/ldtk and ../_ref/tiled with grep for behavior only.
4. Run the gate: `cargo xtask check` (before T-0002 exists: cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace; docs-only tasks have no gate). Fix causes; never silence checks, skip tests, or loosen lints.
5. Set `status: in_review`, commit on the task branch.
6. REVIEW with a fresh context. If you can spawn a subagent, spawn a reviewer that did not write the code and give it only AGENTS.md, docs/reviews/_TEMPLATE.md and the branch name (use doubt-driven-development for T-0004, T-0005, T-0007, T-0011). If you cannot, do a strict adversarial self-review with code-review-and-quality and write "Reviewer: self" in the file. The review goes to docs/reviews/<ID>-review.md on the branch. A self-review that approves everything is a failure.
7. CHANGES_REQUESTED: set status in_progress, fix, repeat 4-6 (max 3 rounds, then STOP and report). APPROVE: from main/ run scripts/merge.sh <ID>.
8. Append one line to docs/PROGRESS.md (date, task, result, reviewer type, SHA), commit on main, continue.

AUTOPILOT RULES
- T-0000: do not interview me; use docs/decisions/0001-defaults.md; open questions go to docs/QUESTIONS.md.
- T-0008: do the egui_dock spike for real and record it in ADR-0002. You cannot see the UI: write tests where feasible and list everything a human must eyeball in docs/MANUAL_CHECKS.md, one checkbox per behavior.
- T-0003: only create the workflow file; there is no remote.
- Low-risk ambiguity: choose the simplest option, write a short ADR, continue. Contract-level ambiguity: take the most conservative option and log it in docs/QUESTIONS.md.
- No new dependency without an ADR. Stay inside each task's scope. Commit small and often.
- If your context gets long, resume from scripts/board.sh, docs/PROGRESS.md and git log. Do not re-read the whole repo.

HARD STOPS (stop at once, then write the report below with the exact state):
- the gate is still red after 3 honest fix attempts on the same failure
- a task needs a change outside its scope that cannot be deferred via QUESTIONS.md
- the T-0008 spike shows neither egui_dock nor the fallback can meet docs/03-ui-layout.md
- 3 review rounds without APPROVE on one task

FINISH
T-0011 requires writing docs/PHASE_A_REPORT.md (spec is in the task file). If you hit a HARD STOP, still write docs/PHASE_A_REPORT.md covering what exists so far and the stop reason. Commit it on main. Do NOT start any T-01xx task. In your final message print only: the path of the report and a 5-line summary.
