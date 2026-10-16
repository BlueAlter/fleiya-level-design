#!/usr/bin/env bash
# Lists tasks. If a branch agent/<ID> exists, its (newer) task file is read from that branch.
cd "$(git rev-parse --show-toplevel)" || exit 1
for f in docs/tasks/T-*.md; do
  id=$(basename "$f" | cut -d- -f1-2)
  if git show-ref --quiet "refs/heads/agent/${id}"; then
    txt=$(git show "agent/${id}:${f}" 2>/dev/null || cat "$f")
  else
    txt=$(cat "$f")
  fi
  st=$(printf '%s\n' "$txt" | grep -m1 '^status:' | cut -d' ' -f2)
  ow=$(printf '%s\n' "$txt" | grep -m1 '^owner:' | cut -d' ' -f2-)
  ro=$(printf '%s\n' "$txt" | grep -m1 '^role:' | cut -d' ' -f2-)
  ti=$(printf '%s\n' "$txt" | grep -m1 '^title:' | cut -d' ' -f2-)
  dp=$(printf '%s\n' "$txt" | grep -m1 '^depends_on:' | cut -d' ' -f2-)
  printf '%-8s %-12s %-11s %-12s %-22s %s\n' "$id" "$st" "$ro" "${ow:--}" "deps:$dp" "$ti"
done
echo; echo "Active branches:"; git branch --list 'agent/*'
