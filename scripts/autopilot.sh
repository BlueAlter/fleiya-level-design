#!/usr/bin/env bash
# Level Craft autopilot: a DETERMINISTIC orchestrator (plain bash, no LLM).
# The human is replaced by this script; agents never call other agents (ADR-0003).
#
#   export AGENT_CMD='agy -p'          # implementer. The prompt is appended as the last argument.
#   export REVIEW_CMD='claude -p'      # optional: a DIFFERENT tool/model for review (default: AGENT_CMD)
#   export MAX_PARALLEL=3 MAX_ROUNDS=3 TASK_TIMEOUT=5400
#   export STOP_AFTER=T-0011           # optional: pause after the contract freeze
#   scripts/autopilot.sh
#
# Per task: claim (script) -> implementer process -> gate (script) -> independent reviewer process -> merge (script).
set -o pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1
ROOT=$(pwd)
: "${AGENT_CMD:?set AGENT_CMD, e.g. export AGENT_CMD='agy -p'}"
REVIEW_CMD="${REVIEW_CMD:-$AGENT_CMD}"
MAX_PARALLEL="${MAX_PARALLEL:-3}"; MAX_ROUNDS="${MAX_ROUNDS:-3}"; TASK_TIMEOUT="${TASK_TIMEOUT:-5400}"
STOP_AFTER="${STOP_AFTER:-}"; MAX_ITER="${MAX_ITER:-300}"
TIMEOUT_BIN=$(command -v timeout || command -v gtimeout || true)
mkdir -p .autopilot/logs .autopilot/rounds
[ -z "$(git status --porcelain)" ] || { echo "main has uncommitted changes; commit or stash first"; exit 1; }
[ "$(git rev-parse --abbrev-ref HEAD)" = main ] || { echo "run from branch main"; exit 1; }

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" | tee -a .autopilot/autopilot.log; }
task_file() { ls docs/tasks/$1-*.md 2>/dev/null | head -n1; }
status() { local f; f=$(task_file "$1")
  { if git show-ref --quiet "refs/heads/agent/$1"; then git show "agent/$1:$f"; else cat "$f"; fi; } | grep -m1 '^status:' | cut -d' ' -f2; }
deps_done() { local d df
  for d in $(grep -m1 '^depends_on:' "$(task_file "$1")" | sed 's/^depends_on: *\[//; s/\].*$//' | tr -d ' ' | tr ',' ' '); do
    df=$(task_file "$d"); { [ -n "$df" ] && grep -q '^status: done' "$df"; } || return 1; done; return 0; }
render() { sed "s#{{ID}}#$1#g; s#{{WT}}#$2#g" "prompts/$3.md"; }
run_agent() { # cmd prompt log dir
  ( cd "$4" && ${TIMEOUT_BIN:+$TIMEOUT_BIN $TASK_TIMEOUT} $1 "$2" ) >"$3" 2>&1 </dev/null; }
gate() { # wt id
  ( cd "$1" || exit 1; [ -f Cargo.toml ] || exit 0
    if cargo xtask --help >/dev/null 2>&1; then cargo xtask check
    else cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; fi
  ) >".autopilot/logs/$2.gate.log" 2>&1; }
reopen() { local id=$1 wt="$ROOT/../wt-$1" f; f=$(task_file "$1")
  sed -i.bak 's/^status: .*/status: in_progress/' "$wt/$f" && rm -f "$wt/$f.bak"
  git -C "$wt" add -A && git -C "$wt" commit -qm "chore($id): reopen after review/gate"; }

implement() { local id=$1 wt="$ROOT/../wt-$1" n
  git show-ref --quiet "refs/heads/agent/$id" || scripts/claim.sh "$id" autopilot >/dev/null 2>&1 || { log "claim failed $id"; return; }
  n=$(( $(cat ".autopilot/rounds/$id" 2>/dev/null || echo 0) + 1 )); echo $n > ".autopilot/rounds/$id"
  log "IMPLEMENT $id (round $n)"
  run_agent "$AGENT_CMD" "$(render "$id" "$wt" implementer)" ".autopilot/logs/$id.impl$n.log" "$wt"
  [ -n "$(git -C "$wt" status --porcelain)" ] && { git -C "$wt" add -A; git -C "$wt" commit -qm "wip($id): leftover changes"; }
}
sync_branch() { local id=$1 wt="$ROOT/../wt-$1"
  git -C "$wt" merge --no-edit main >/dev/null 2>&1 && return 0
  log "CONFLICT $id, asking agent to resolve"
  run_agent "$AGENT_CMD" "$(render "$id" "$wt" conflict)" ".autopilot/logs/$id.conflict.log" "$wt"
  [ ! -f "$(git -C "$wt" rev-parse --git-dir)/MERGE_HEAD" ] && [ -z "$(git -C "$wt" diff --name-only --diff-filter=U)" ]; }
review_one() { local id=$1 wt="$ROOT/../wt-$1" base bad rf="docs/reviews/$1-review.md"
  sync_branch "$id" || { echo conflict > ".autopilot/verdict.$id"; return; }
  gate "$wt" "$id" || { echo gatefail > ".autopilot/verdict.$id"; return; }
  base=$(git -C "$wt" rev-parse HEAD); log "REVIEW $id"
  run_agent "$REVIEW_CMD" "$(render "$id" "$wt" reviewer)" ".autopilot/logs/$id.review.log" "$wt"
  [ -n "$(git -C "$wt" status --porcelain docs/reviews)" ] && { git -C "$wt" add docs/reviews; git -C "$wt" commit -qm "review($id)"; }
  bad=$(git -C "$wt" diff --name-only "$base" HEAD | grep -v '^docs/reviews/' || true)
  [ -n "$bad" ] && { git -C "$wt" reset -q --hard "$base"; echo invalid > ".autopilot/verdict.$id"; return; }
  if grep -q '^Verdict: APPROVE *$' "$wt/$rf" 2>/dev/null; then echo approve; else echo changes; fi > ".autopilot/verdict.$id"; }
decide() { local id=$1 wt="$ROOT/../wt-$1" v; v=$(cat ".autopilot/verdict.$id" 2>/dev/null); rm -f ".autopilot/verdict.$id"
  case "$v" in
    approve) if scripts/merge.sh "$id" >".autopilot/logs/$id.merge.log" 2>&1; then
               log "MERGED $id"; echo "- $(date +%F) $id merged (rounds: $(cat .autopilot/rounds/$id))" >> docs/PROGRESS.md
               git add docs/PROGRESS.md; git commit -qm "chore: progress $id"
             else log "MERGE FAILED $id (see logs), reopening"; git reset -q --hard; git checkout -q main
               printf '# Automated merge failure\n\nVerdict: CHANGES_REQUESTED\n\n%s\n' "$(tail -n 40 ".autopilot/logs/$id.merge.log")" > "$wt/docs/reviews/$id-review.md"; reopen "$id"; fi;;
    gatefail|conflict) log "$v $id, reopening"
               printf '# Automated %s\n\nVerdict: CHANGES_REQUESTED\n\n```\n%s\n```\n' "$v" "$(tail -n 60 ".autopilot/logs/$id.gate.log" 2>/dev/null)" > "$wt/docs/reviews/$id-review.md"; reopen "$id";;
    *) log "CHANGES $id (verdict: ${v:-none}), reopening"; reopen "$id";;
  esac; }

report() { { echo "# Autopilot report $(date '+%F %T')"; echo; echo '## Board'; scripts/board.sh; echo
  echo '## Failed (need a human)'; ls .autopilot/failed.* 2>/dev/null | sed 's/.*failed\./- /' ; echo
  echo '## Open questions'; grep -n '^- \[ \]' docs/QUESTIONS.md 2>/dev/null; echo
  echo '## Manual checks (run the app and tick these)'; cat docs/MANUAL_CHECKS.md 2>/dev/null; echo
  echo '## Logs'; echo ".autopilot/logs/  (per task: implN, gate, review, merge)"; } | tee .autopilot/REPORT.md; }

list_status() { local s=$1 f id; for f in docs/tasks/T-*.md; do id=$(basename "$f" | cut -d- -f1-2)
  [ -e ".autopilot/failed.$id" ] && continue; [ "$(status "$id")" = "$s" ] && echo "$id"; done; }

iter=0
while [ $iter -lt "$MAX_ITER" ]; do iter=$((iter+1))
  if [ -n "$STOP_AFTER" ] && grep -q '^status: done' "$(task_file "$STOP_AFTER")"; then log "STOP_AFTER $STOP_AFTER reached"; break; fi
  ready=(); for id in $(list_status todo); do deps_done "$id" && ready+=("$id"); done
  for id in $(list_status in_progress); do ready+=("$id"); done
  batch=(); for id in "${ready[@]}"; do
    if [ "$(cat ".autopilot/rounds/$id" 2>/dev/null || echo 0)" -ge "$MAX_ROUNDS" ]; then touch ".autopilot/failed.$id"; log "FAILED $id (max rounds)"; continue; fi
    [ ${#batch[@]} -lt "$MAX_PARALLEL" ] && batch+=("$id"); done
  for id in "${batch[@]}"; do implement "$id" & done; wait
  inrev=($(list_status in_review))
  for id in "${inrev[@]}"; do review_one "$id" & done; wait
  for id in "${inrev[@]}"; do decide "$id"; done
  [ ${#batch[@]} -eq 0 ] && [ ${#inrev[@]} -eq 0 ] && { log "nothing left to do"; break; }
done
report
