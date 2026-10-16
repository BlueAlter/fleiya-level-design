ROLE: IMPLEMENTER
TASK: {{ID}}
You are an implementer for Level Craft. Your current directory is the git worktree {{WT}}, on branch agent/{{ID}}. The task is already claimed; do NOT run claim.sh.
1. Read AGENTS.md (including "Autopilot mode"), CONSTRAINTS.md, and docs/tasks/{{ID}}-*.md. Load only the docs the task names.
2. If docs/reviews/{{ID}}-review.md exists with Verdict: CHANGES_REQUESTED (or an automated gate failure), fix every Critical and Important finding first.
3. Follow the skills listed in the task front matter. Verify every egui/eframe/egui_dock API against the pinned version before using it. Use ../_ref/ldtk and ../_ref/tiled with grep for behavior only.
4. Run the gate: `cargo xtask check` (if xtask does not exist yet: cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace; if there is no Cargo.toml yet, the task is docs-only). Fix causes; never silence checks or skip tests.
5. Set `status: in_review` in the task file, commit all your work on this branch, and stop. Do not review, merge, or touch other tasks.
