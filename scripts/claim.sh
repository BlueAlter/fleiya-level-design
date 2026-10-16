#!/usr/bin/env bash
# Usage: scripts/claim.sh T-0004 agent-name   (run from the main checkout)
# The branch agent/<ID> is the lock: git refuses to create it twice.
set -euo pipefail
id="${1:?usage: scripts/claim.sh T-0001 agent-name}"
agent="${2:?agent name required}"
cd "$(git rev-parse --show-toplevel)"
f=$(ls docs/tasks/${id}-*.md 2>/dev/null | head -n1 || true)
[ -n "$f" ] || { echo "task ${id} not found"; exit 1; }
grep -q '^status: todo' "$f" || { echo "${id} is not 'todo'"; exit 1; }
deps=$(grep -m1 '^depends_on:' "$f" | sed 's/^depends_on: *\[//; s/\].*$//' | tr -d ' ' | tr ',' ' ')
for d in $deps; do
  df=$(ls docs/tasks/${d}-*.md 2>/dev/null | head -n1 || true)
  [ -n "$df" ] && grep -q '^status: done' "$df" || { echo "dependency ${d} is not done"; exit 1; }
done
wt="../wt-${id}"
git worktree add -b "agent/${id}" "$wt" main
sed -i.bak "s/^status: todo/status: in_progress/; s/^owner: .*/owner: ${agent}/" "$wt/$f" && rm -f "$wt/$f.bak"
git -C "$wt" add "$f"
git -C "$wt" commit -qm "chore(${id}): claim by ${agent}"
echo "Claimed ${id}. Work ONLY in: ${wt}"
