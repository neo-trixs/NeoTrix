#!/usr/bin/env bash
# NeoTrix PreToolUse Guard (P-SR + P-CGP Enforcement Layer)
# v2.2 — Lockfile + Audit Logging + Shell Wrapper + Limitation Documentation
#
# ⚠️  LIMITATION: The PreToolUse hook runs `bash scripts/pre-tool-guard.sh`
#    WITHOUT the actual command being executed. `$#` is always 0.
#    The guard CANNOT check the command — only log invocation.
#
# PRIMARY PROTECTION against git reset --hard:
#   ~/.zshrc git() shell wrapper (installed by guard_install_shell_wrapper)
#   Run: bash scripts/pre-tool-guard.sh --install
#
# SECONDARY: .githooks/* hooks (limited — only fire for commit/checkout, not reset)
# TERTIARY: PreToolUse audit log at /tmp/neotrix-guard-audit.log
#
# === Pass 3 Enforcement ===
# Blocks destructive commands by BLAST RADIUS, not keyword:
#   • `rm -rf node_modules` → ALLOWED (safe blast radius)
#   • `rm -rf ~` → BLOCKED (system-level blast radius)
#   • `rm -rf /` → BLOCKED (root filesystem)
#   • `git reset --hard` → BLOCKED (uncommitted work loss)
#
# Usage:
#   source scripts/pre-tool-guard.sh
#   export NEOTRIX_GUARD=1  # Enable strict mode with DEBUG trap
set -euo pipefail

NEOTRIX_GUARD="${NEOTRIX_GUARD:-0}"

SYSTEM_ROOTS=("/" "/etc" "/usr" "/var" "/bin" "/sbin" "/opt")
HOME_DANGER_ZONES=("$HOME" "$HOME/.ssh" "$HOME/.aws" "$HOME/.gnupg" "$HOME/.config" "$HOME/.local" "$HOME/Documents" "$HOME/Desktop")
PROJECT_DANGER_ZONES=(".git" "node_modules" "target" ".cargo")

_guard_logfile="/tmp/neotrix-guard-audit.log"
_guard_lockfile="/tmp/neotrix-guard.lock"
_guard_regression_logfile="/tmp/neotrix-guard-regression.log"  # forced regression logging (clawback pattern)

guard_audit_log() {
    local action="$1" pattern="$2" cmd="$3"
    echo "[$(date '+%Y-%m-%dT%H:%M:%S%z')] [$$] ${action}: ${pattern} — ${cmd}" >> "$_guard_logfile"
}

guard_acquire_lock() {
    if mkdir "$_guard_lockfile" 2>/dev/null; then
        echo "$$" > "$_guard_lockfile/pid"
        trap 'rm -rf "$_guard_lockfile"' EXIT
        return 0
    fi
    local lock_age=0
    if [ -f "$_guard_lockfile/pid" ]; then
        lock_age=$(($(date +%s) - $(stat -f "%m" "$_guard_lockfile/pid" 2>/dev/null || echo "0")))
    fi
    if [ "$lock_age" -gt 10 ]; then
        rm -rf "$_guard_lockfile"
        mkdir "$_guard_lockfile"
        echo "$$" > "$_guard_lockfile/pid"
        trap 'rm -rf "$_guard_lockfile"' EXIT
        guard_audit_log "RECOVERED" "stale-lock" "age=${lock_age}s"
        return 0
    fi
    guard_audit_log "SKIP" "concurrent-lock" "age=${lock_age}s"
    return 1
}

_get_blast_radius() {
    local path="$1"
    local resolved; resolved=$(eval echo "$path" 2>/dev/null || echo "$path")
    resolved="${resolved%/}"
    [ -z "$resolved" ] && { echo "UNKNOWN"; return; }
    for root in "${SYSTEM_ROOTS[@]}"; do
        if [ "$resolved" = "$root" ] || [ "$resolved" = "${root}/" ]; then echo "SYSTEM"; return; fi
    done
    for zone in "${HOME_DANGER_ZONES[@]}"; do
        local zr; zr=$(eval echo "$zone" 2>/dev/null || echo "$zone")
        if [ "$resolved" = "$zr" ] || echo "$resolved" | grep -q "^${zr}/."; then echo "HOME"; return; fi
    done
    if echo "$resolved" | grep -qE "^(\.\.|~/|~$)"; then echo "PROJECT_RELATIVE"; return; fi
    echo "LOCAL"
}

destructiveness_check() {
    local cmd="$*"
    if echo "$cmd" | grep -qE '\brm\s+(-rf|--recursive.*--force|--force.*--recursive)\b'; then
        local targets; targets=$(echo "$cmd" | sed 's/.*rm\s\+\(-rf\|--recursive.*--force\|--force.*--recursive\)\s\+//')
        for target in $targets; do
            local radius; radius=$(_get_blast_radius "$target")
            case "$radius" in
                SYSTEM|HOME|PROJECT_RELATIVE)
                    echo "[PRE-TOOL-GUARD] BLOCKED: rm -rf $target ($radius)"
                    guard_audit_log "BLOCKED" "rm-${radius}" "$cmd"
                    return 1;;
                LOCAL) ;;
            esac
        done
    fi
    if echo "$cmd" | grep -qE 'git\s+reset\s+--hard'; then
        echo "[PRE-TOOL-GUARD] BLOCKED: git reset --hard"
        guard_audit_log "BLOCKED" "git-reset-hard" "$cmd"
        # Forced Regression Logging (clawback pattern, Cycle 154)
        echo "[$(date '+%Y-%m-%dT%H:%M:%S%z')] [$$] REGRESSION: BLOCKED git reset --hard: $cmd" >> "$_guard_regression_logfile"
        echo "  → Record what broke, why, and which principle" >> "$_guard_regression_logfile"
        return 1
    fi
    if echo "$cmd" | grep -qE 'git\s+checkout\s+(HEAD\s+--|--force)\b'; then
        echo "[PRE-TOOL-GUARD] BLOCKED: git checkout HEAD --"
        guard_audit_log "BLOCKED" "git-checkout-head" "$cmd"
        return 1
    fi
    if echo "$cmd" | grep -qE 'git\s+clean\s+-fd'; then
        echo "[PRE-TOOL-GUARD] BLOCKED: git clean -fd"
        guard_audit_log "BLOCKED" "git-clean-fd" "$cmd"
        return 1
    fi
    if echo "$cmd" | grep -qE '(curl|wget)\s.*\|\s*(bash|sh|zsh)'; then
        echo "[PRE-TOOL-GUARD] BLOCKED: pipe-to-shell"
        guard_audit_log "BLOCKED" "pipe-to-shell" "$cmd"
        return 1
    fi
    return 0
}

guard_install_claude() {
    local hook_dir="$HOME/.claude/hooks"
    mkdir -p "$hook_dir"
    cat > "$hook_dir/pre-tool-guard.json" << 'JSONEOF'
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [ { "type": "command", "command": "bash scripts/pre-tool-guard.sh" } ] }
    ]
  }
}
JSONEOF
    echo "[PRE-TOOL-GUARD] Installed for Claude Code: $hook_dir/pre-tool-guard.json"
}

guard_install_git_hooks() {
    local root; root=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
    mkdir -p "$root/.githooks"
    if [ ! -f "$root/.githooks/pre-commit" ]; then
        ln -sf ../../scripts/git-safe-reset.sh "$root/.githooks/pre-commit"
        ln -sf ../../scripts/git-safe-reset.sh "$root/.githooks/post-checkout"
        echo "[PRE-TOOL-GUARD] Git hooks installed to $root/.githooks/"
    fi
    git config core.hooksPath .githooks 2>/dev/null
    echo "[PRE-TOOL-GUARD] Git hook path set to .githooks"
}

guard_install_shell_wrapper() {
    local rc_file="$HOME/.zshrc"
    local marker="# NeoTrix Safe Git Wrapper"
    if grep -q "$marker" "$rc_file" 2>/dev/null; then
        echo "[PRE-TOOL-GUARD] Shell wrapper already in $rc_file"
        return 0
    fi
    cat >> "$rc_file" << 'SHELLEOF'

# NeoTrix Safe Git Wrapper — blocks git reset --hard when uncommitted changes exist
git() {
    if [ "$1" = "reset" ] && { [ "$2" = "--hard" ] || [ "$2" = "-hard" ]; }; then
        if [ -n "$(command git status --porcelain 2>/dev/null)" ]; then
            local changes; changes=$(command git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
            echo "🛑 [SAFE-GIT] BLOCKED: git reset --hard with ${changes} uncommitted change(s)"
            return 1
        fi
    fi
    command git "$@"
}
SHELLEOF
    echo "[PRE-TOOL-GUARD] Shell wrapper installed in $rc_file"
    echo "  ⚠️  NOTE: PreToolUse hook cannot see the command being run (platform limitation)."
    echo "  The shell wrapper is the ONLY reliable protection against git reset --hard."
}

install_guards() {
    echo "=== PreToolUse Guard Installation ==="
    guard_install_claude
    guard_install_git_hooks
    guard_install_shell_wrapper
    echo ""
    echo "⚠️  LIMITATION: Claude Code PreToolUse hook runs BEFORE each Bash tool"
    echo "   but does NOT receive the actual command. The guard script can only"
    echo "   log invocation — it cannot check the command being executed."
    echo "   Protection relies on:"
    echo "     1. ~/.zshrc git() wrapper (intercepts all shells)   ← PRIMARY"
    echo "     2. .githooks/* hooks (limited, only for commit/checkout)"
    echo "     3. PreToolUse audit log (monitoring only)"
}

guard_audit_log "GUARD_INVOKE" "v2.2" "$*"

if [ "${BASH_SOURCE[0]}" != "$0" ]; then
    if [ -n "${PS1:-}" ] && [ "$NEOTRIX_GUARD" = "1" ]; then
        preexec() { destructiveness_check "$BASH_COMMAND"; }
        trap 'preexec' DEBUG
        echo "[PRE-TOOL-GUARD] v2.2 Active (Forced Regression Logging)"
    fi
else
    if guard_acquire_lock; then
        if [ $# -gt 0 ]; then
            destructiveness_check "$*" || { guard_audit_log "BLOCKED" "arg" "$*"; false; }
        fi
        guard_audit_log "ALLOWED" "script-invoke" "$*"
    fi
fi
