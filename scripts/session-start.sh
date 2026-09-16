#!/bin/sh
# session-start.sh — bootstrap per-session git worktree isolation for NeoTrix.
#
# Purpose:
#   Each concurrent session should own its own git worktree + feature branch so
#   that a session's `git reset` / `git checkout` only affects ITS OWN tree and
#   never wipes another session's uncommitted work in the main working dir
#   (/Users/neo/Downloads/neotrix). The main tree stays at HEAD; all work happens
#   in an isolated worktree, and is merged to main only when CI is green.
#
# Usage:
#   ./scripts/session-start.sh <name>
#
# This script only PRINTS the commands to run — it does not execute git itself.
# The isolation home /Users/neo/Downloads/neotrix-concurrent already exists on
# branch `concurrent-wip`; new sessions can be added alongside it.

set -eu

if [ $# -lt 1 ]; then
  echo "Usage: $0 <name>" >&2
  echo "  Creates/reuses a worktree at /Users/neo/Downloads/neotrix-<name>" >&2
  echo "  on branch <name>-wip and prints the 'cd' to use it." >&2
  exit 2
fi

NAME="$1"
WT_ROOT="/Users/neo/Downloads/neotrix-${NAME}"
BRANCH="${NAME}-wip"
REPO_ROOT="/Users/neo/Downloads/neotrix"

if [ -d "$WT_ROOT" ]; then
  echo "# Worktree already exists at $WT_ROOT (branch $BRANCH)."
  echo "cd \"$WT_ROOT\""
else
  echo "# Create isolated worktree + branch for session '$NAME':"
  echo "git -C \"$REPO_ROOT\" worktree add -b \"$BRANCH\" \"$WT_ROOT\""
  echo "cd \"$WT_ROOT\""
fi
