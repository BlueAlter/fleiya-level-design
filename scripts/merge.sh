#!/usr/bin/env bash
# Integrator only. Usage: scripts/merge.sh T-0004
# If anything fails after the squash, run: git reset --hard && git checkout main
set -euo pipefail
id="${1:?usage: scripts/merge.sh T-0001}"
branch="agent/${id}"
cd "$(git rev-parse --show-toplevel)"
git show "${branch}:docs/reviews/${id}-review.md" | grep -q '^Verdict: APPROVE *$' || { echo "no APPROVE review on ${branch}"; exit 1; }
git checkout main
git merge --squash "$branch"
f=$(ls docs/tasks/${id}-*.md | head -n1)
sed -i.bak 's/^status: .*/status: done/' "$f" && rm -f "$f.bak"
if cargo xtask --help >/dev/null 2>&1; then cargo xtask check; fi
title=$(grep -m1 '^title:' "$f" | cut -d' ' -f2-)
git add -A && git commit -m "feat(${id}): ${title}"
git worktree remove "../wt-${id}" --force || true
git branch -D "$branch"
echo "Merged ${id}."
