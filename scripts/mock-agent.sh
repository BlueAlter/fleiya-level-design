#!/usr/bin/env bash
# Dry-run "agent": simulates implementer/reviewer/conflict-resolver WITHOUT an LLM.
# Use it to test the orchestration for free:  AGENT_CMD=scripts/mock-agent.sh scripts/autopilot.sh
prompt="${@: -1}"
id=$(printf '%s' "$prompt" | grep -o 'T-[0-9]\{4\}' | head -1)
case "$prompt" in
  *"ROLE: REVIEWER"*)  mkdir -p docs/reviews; printf '# Review of %s\n\nVerdict: APPROVE\n' "$id" > "docs/reviews/$id-review.md"; git add -A; git commit -qm "review($id)";;
  *"ROLE: CONFLICT"*)  git add -A; git commit -qm "resolve ($id)";;
  *) f=$(ls docs/tasks/$id-*.md | head -n1); sed -i.bak 's/^status: .*/status: in_review/' "$f"; rm -f "$f.bak"; git add -A; git commit -qm "impl($id)";;
esac
