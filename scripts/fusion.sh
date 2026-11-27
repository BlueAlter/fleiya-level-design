#!/usr/bin/env bash
# Rebuild the disposable `fusion-cargo` integration branch in an isolated worktree
# by merging multiple feature branches on top of main, without touching active worktrees.
#
# Usage:
#   scripts/fusion.sh [--push] [--check] [--skip-fetch] [branch...]
# Defaults to: feature/settings feature/layer feature/entity
set -euo pipefail

push_branch=false
run_check=false
skip_fetch=false
base_ref="origin/main"
target_branch="fusion-cargo"
branches=()

while [ $# -gt 0 ]; do
  case "$1" in
    --push) push_branch=true; shift ;;
    --check) run_check=true; shift ;;
    --skip-fetch) skip_fetch=true; shift ;;
    --base) base_ref="${2:?--base requires a ref}"; shift 2 ;;
    --target) target_branch="${2:?--target requires a branch name}"; shift 2 ;;
    -h|--help)
      echo "Usage: scripts/fusion.sh [--push] [--check] [--skip-fetch] [--base <ref>] [--target <branch>] [branch...]"
      exit 0
      ;;
    *) branches+=("$1"); shift ;;
  esac
done

if [ ${#branches[@]} -eq 0 ]; then
  branches=("feature/settings" "feature/layer" "feature/entity")
fi

repo_root=$(git rev-parse --show-toplevel)
common_dir=$(git rev-parse --git-common-dir)
case "$common_dir" in
  /*) ;;
  *) common_dir="${repo_root}/${common_dir}" ;;
esac
wt_dir="$(cd "$(dirname "$common_dir")/.." && pwd)/wt-${target_branch}"

cd "$repo_root"

if [ "$skip_fetch" = false ]; then
  echo "==> Fetching origin..."
  git fetch origin --prune
fi

if ! git rev-parse --verify --quiet "$base_ref" >/dev/null; then
  if [ "$base_ref" = "origin/main" ] && git rev-parse --verify --quiet "main" >/dev/null; then
    base_ref="main"
  else
    echo "Base ref '$base_ref' does not exist." >&2
    exit 1
  fi
fi

if [ ! -d "$wt_dir" ]; then
  echo "==> Creating integration worktree at $wt_dir..."
  if git rev-parse --verify --quiet "refs/heads/$target_branch" >/dev/null; then
    git worktree add "$wt_dir" "$target_branch"
  else
    git worktree add -b "$target_branch" "$wt_dir" "$base_ref"
  fi
else
  if [ -n "$(git -C "$wt_dir" status --porcelain)" ]; then
    echo "==> Cleaning dirty state in $wt_dir..."
    git -C "$wt_dir" reset --hard HEAD
    git -C "$wt_dir" clean -fd
  fi
  current_wt_branch=$(git -C "$wt_dir" rev-parse --abbrev-ref HEAD)
  if [ "$current_wt_branch" != "$target_branch" ]; then
    if git rev-parse --verify --quiet "refs/heads/$target_branch" >/dev/null; then
      git -C "$wt_dir" switch "$target_branch"
    else
      git -C "$wt_dir" switch -c "$target_branch" "$base_ref"
    fi
  fi
fi

resolve_ref() {
  local b="$1"
  local has_local=false
  local has_origin=false
  git rev-parse --verify --quiet "refs/heads/$b" >/dev/null && has_local=true
  git rev-parse --verify --quiet "refs/remotes/origin/$b" >/dev/null && has_origin=true

  if [ "$has_local" = true ] && [ "$has_origin" = true ]; then
    if git merge-base --is-ancestor "refs/remotes/origin/$b" "refs/heads/$b"; then
      echo "$b"
    elif git merge-base --is-ancestor "refs/heads/$b" "refs/remotes/origin/$b"; then
      echo "  [info] local '$b' is behind 'origin/$b'; using 'origin/$b'" >&2
      echo "origin/$b"
    else
      echo "  [warn] '$b' and 'origin/$b' diverged; using local '$b'" >&2
      echo "$b"
    fi
  elif [ "$has_local" = true ]; then
    echo "$b"
  elif [ "$has_origin" = true ]; then
    echo "origin/$b"
  else
    echo "Branch '$b' not found locally or on origin." >&2
    return 1
  fi
}

echo "==> Resetting '$target_branch' to $base_ref..."
git -C "$wt_dir" reset --hard "$base_ref"

for b in "${branches[@]}"; do
  ref=$(resolve_ref "$b")
  sha=$(git rev-parse --short "$ref")
  echo "==> Merging $ref ($sha) into $target_branch..."
  if ! git -C "$wt_dir" merge --no-ff --no-edit "$ref"; then
    echo "Merge conflict while merging '$ref' into '$target_branch' inside $wt_dir." >&2
    exit 1
  fi
done

if [ "$run_check" = true ]; then
  echo "==> Running cargo xtask check in $wt_dir..."
  (cd "$wt_dir" && cargo xtask check)
fi

if [ "$push_branch" = true ]; then
  echo "==> Pushing $target_branch to origin..."
  git -C "$wt_dir" push --force-with-lease origin "$target_branch"
  echo "==> Pushed 'origin/$target_branch'. GitHub Actions CI will run cargo xtask check."
fi

if [ "$run_check" = false ] && [ "$push_branch" = false ]; then
  echo ""
  echo "Integration branch '$target_branch' built at: $wt_dir"
  echo "Next steps:"
  echo "  - Run CI in GitHub Actions: scripts/fusion.sh --push"
  echo "  - Run check locally:        scripts/fusion.sh --check"
fi
