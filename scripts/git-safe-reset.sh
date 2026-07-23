#!/usr/bin/env bash
# NeoTrix Safe Reset Hook — intercepts git reset --hard to protect uncommitted work
# v1.0 — Installed as global git hook via core.hooksPath
#
# This runs as a pre-commit + pre-merge + post-checkout hook
# to prevent accidental loss of uncommitted changes.

set -euo pipefail

NEOTRIX_HOOK_LOG="/tmp/neotrix-git-hooks.log"

log_hook() {
    local hook_name="$1"
    local message="$2"
    echo "[$(date '+%Y-%m-%dT%H:%M:%S%z')] [hook:${hook_name}] ${message}" >> "$NEOTRIX_HOOK_LOG"
}

# ============================================================
# Block git reset --hard if there are uncommitted changes
# ============================================================
check_reset_hard() {
    # Check if the actual command being run is git reset --hard
    # GIT_* env vars are set by git when running hooks
    if [ "${GIT_REFLOG_ACTION:-}" = "reset" ]; then
        # Check for uncommitted changes
        local changes
        changes=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
        if [ "$changes" -gt 0 ]; then
            echo ""
            echo "🛑 [GIT-SAFE-RESET] BLOCKED: git reset --hard with ${changes} uncommitted change(s)"
            echo "   This would destroy your work!"
            echo "   Alternatives:"
            echo "     • git stash push -m \"save work\"       (save for later)"
            echo "     • git checkout -- <file>              (discard single file)"
            echo "     • git restore --staged <file>         (unstage only)"
            echo ""
            log_hook "reset-hard" "BLOCKED: ${changes} uncommitted changes"
            exit 1
        fi
        log_hook "reset-hard" "ALLOWED: no uncommitted changes"
    fi
}

check_reset_hard
log_hook "$(basename "$0")" "passed"
exit 0
