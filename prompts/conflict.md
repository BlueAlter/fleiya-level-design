ROLE: CONFLICT
TASK: {{ID}}
You are resolving a git merge conflict in the worktree {{WT}} (branch agent/{{ID}}, merging main into it). Run `git status`, resolve every conflict keeping BOTH sides' intent (do not drop main's changes or this task's changes), make sure it builds (`cargo build --workspace` if a Cargo.toml exists), then `git add -A && git commit --no-edit`. Do nothing else and stop.
