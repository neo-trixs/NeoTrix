#!/bin/sh
# no-main-direct.sh — pre-commit / pre-push guard hook for NeoTrix.
#
# Purpose:
#   Enforce per-session git isolation by refusing any direct commit or push
#   to the protected integration branches `main` and `master`. Concurrent
#   sessions must work on feature branches (e.g. <name>-wip) inside their own
#   git worktree, and only merge into main via PR / merge when CI is green.
#   This prevents one session from resetting / clobbering another's work.
#
# Install:
#   - copy to .git/hooks/pre-commit and .git/hooks/pre-push, or
#   - `git config core.hooksPath scripts` (this file lives in scripts/).
#
# Exit 1 => block the operation. Exit 0 => allow.

set -eu

# Determine the branch being committed or pushed.
# For pre-commit the current branch is the target.
# For pre-push, GIT_PREFIX is set and $1/$2 are the local ref/sha; fall back
# to `git symbolic-ref` resolution for a robust branch name.
branch=""
if [ "${1:-}" = "" ]; then
  branch=$(git symbolic-ref --short -q HEAD 2>/dev/null || true)
else
  # pre-push passes: <remote> <url> via stdin lines "local_ref local_sha remote_ref remote_sha"
  # Read from stdin if available.
  if [ ! -t 0 ]; then
    while read -r local_ref _remote_ref _remote_sha; do
      case "$local_ref" in
        refs/heads/*) branch="${local_ref#refs/heads/}"; break ;;
      esac
    done
  fi
  [ -z "$branch" ] && branch=$(git symbolic-ref --short -q HEAD 2>/dev/null || true)
fi

case "$branch" in
  main|master)
    echo "=============================================================="
    echo "  BLOCKED: direct commit/push to '$branch' is not allowed."
    echo "=============================================================="
    echo "Per-session isolation policy:"
    echo "  - Commit on your feature branch (e.g. <name>-wip)."
    echo "  - Use a git worktree so 'git reset' only affects your tree."
    echo "  - Merge into '$branch' only via PR / merge when CI is GREEN."
    echo ""
    echo "If you are certain this is intentional, bypass with:"
    echo "  git commit --no-verify   (not recommended)"
    exit 1
    ;;
esac

exit 0
