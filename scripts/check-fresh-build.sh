#!/bin/bash
# Fresh-checkout buildability gate — "a clean clone must be able to build".
#
# Why this is a SEPARATE gate from check-truth-surface.sh (2026-09-28):
#
#   check-truth-surface.sh answers exactly one question:
#     "does the toolchain see every .rs module that `mod X;` declares?"
#   It scans `mod X;` targets in tracked .rs files. That is the right scope
#   for its three classes, but it is structurally blind to the dominant
#   failure mode found this session — every one of these shipped green:
#
#     1. Cargo manifest referenced a file git does not have
#        (apps/neobot-desktop/Cargo.toml — a workspace *member* with no manifest;
#         neotrix-core [[bench]] memory_bench/security_bench)
#     2. A .rs file called a function whose definition was never committed
#        (cli.rs:327 -> migrate_legacy)
#     3. include_str!() pointed at an untracked data file
#        (models/training/jev_platts.json)
#     4. HEAD content stale while a dirty worktree held the fix
#        (13 files) — local `cargo check` passed, clean clone failed with 56 errors
#
#   Items 1-3 are "committed code references a file git lacks" (the class the
#   truth-surface docstring *calls* UNCOMMITTED_DEP but only detects for
#   `mod X;`). Item 4 is a different axis entirely: no file is missing, the
#   tracked content is simply older than the working tree.
#
# The only honest oracle for "can a fresh clone build?" is to build one.
# This gate does that, and degrades to a cheap manifest-only tier by default.
#
# Usage:
#   bash scripts/check-fresh-build.sh              # cheap tier: cargo metadata
#   bash scripts/check-fresh-build.sh --full       # + cargo check --lib -p neotrix
#
# Tiers:
#   metadata  cargo metadata --no-deps
#             catches tier-1 manifest breakage (workspace members, [[bench]],
#             [[bin]], [[example]] targets whose files are absent). Seconds.
#   full      also runs cargo check on the main lib. Catches content staleness
#             (tier 2). Minutes, and a cold target dir unless CARGO_TARGET_DIR
#             is pointed at a warm one.
#
# Where it runs:
#   CI             -> in place. A CI checkout IS a fresh clone by construction
#                     (actions/checkout + nothing has touched sources yet), so a
#                     worktree of it would be redundant and cold-cache.
#   Clean checkout -> in place (the checkout itself is the oracle).
#   Dirty worktree -> a throwaway `git worktree add --detach HEAD`, removed on
#                     exit. A dirty tree is NOT a valid oracle: it compiles
#                     against uncommitted fixes and hides exactly the breakage
#                     this gate exists to find.
#
# Exit: 0 buildable, 1 not buildable, 2 bad usage / preconditions unmet.

set -uo pipefail

TIER="metadata"
PKG="neotrix"
case "${1:-}" in
  "")           TIER="metadata" ;;
  --full)       TIER="full" ;;
  --metadata)   TIER="metadata" ;;
  -h|--help)    sed -n '2,45p' "$0"; exit 0 ;;
  *) echo "unknown arg: $1 (use --metadata | --full)" >&2; exit 2 ;;
esac

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO=$(cd "$SCRIPT_DIR/.." && pwd)
RED=$'\033[31m'; GRN=$'\033[32m'; YEL=$'\033[33m'; NC=$'\033[0m'

command -v cargo >/dev/null 2>&1 || { echo "cargo not found in PATH" >&2; exit 2; }
git -C "$REPO" rev-parse --git-dir >/dev/null 2>&1 || {
  echo "not a git repo: $REPO" >&2; exit 2; }

WORKTREE=""
cleanup() {
  if [ -n "$WORKTREE" ] && [ -d "$WORKTREE" ]; then
    git -C "$REPO" worktree remove --force "$WORKTREE" >/dev/null 2>&1
    rm -rf "$WORKTREE"
  fi
  git -C "$REPO" worktree prune >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

# A dirty tree cannot answer the question; build a pristine one instead.
DIRTY=$(git -C "$REPO" status --porcelain 2>/dev/null)
if [ -n "${CI:-}" ]; then
  ROOT="$REPO"; WHERE="in place (CI: the checkout is the fresh clone)"
elif [ -z "$DIRTY" ]; then
  ROOT="$REPO"; WHERE="in place (clean checkout)"
else
  WORKTREE=$(mktemp -d "${TMPDIR:-/tmp}/nt-fresh-XXXXXX")
  rmdir "$WORKTREE" 2>/dev/null
  if ! git -C "$REPO" worktree add --detach "$WORKTREE" HEAD >/dev/null 2>&1; then
    echo "could not create a pristine worktree at HEAD" >&2; exit 2
  fi
  ROOT="$WORKTREE"; WHERE="fresh worktree @ $(git -C "$REPO" rev-parse --short HEAD)"
fi

echo "=== fresh-build gate ($TIER tier) ==="
echo "  oracle: $WHERE"
echo "  root:   $ROOT"

# ---------- tier 1: manifest integrity ----------
# cargo metadata fails BEFORE compiling when a workspace member has no
# manifest or a declared target's file is absent. Cheap, and it is the tier
# that catches the "a fresh clone cannot even load the workspace" class.
META_LOG=$(mktemp)
if ! (cd "$ROOT" && cargo metadata --no-deps --format-version 1) >"$META_LOG" 2>&1; then
  echo
  echo "${RED}FAIL: a fresh checkout cannot load the cargo workspace.${NC}"
  echo "  Committed manifests reference files git does not have."
  echo "  A fresh clone CANNOT build; nothing downstream can run."
  echo "  --- cargo metadata ---"
  sed -n '1,25p' "$META_LOG" | sed 's/^/  /'
  rm -f "$META_LOG"
  exit 1
fi
rm -f "$META_LOG"
echo "  ${GRN}ok${NC}  cargo metadata (workspace loads)"

if [ "$TIER" = "metadata" ]; then
  echo "${GRN}PASS${NC} (metadata tier: manifests are self-consistent)"
  echo "note: content-level staleness is NOT covered by this tier — use --full"
  exit 0
fi

# ---------- tier 2: the lib actually compiles ----------
CHK_LOG=$(mktemp)
if (cd "$ROOT" && cargo check --lib -p "$PKG") >"$CHK_LOG" 2>&1; then
  rm -f "$CHK_LOG"
  echo "  ${GRN}ok${NC}  cargo check --lib -p $PKG"
  echo "${GRN}PASS${NC} (full tier: a fresh clone builds)"
  exit 0
fi

ERRS=$(grep -cE '^error' "$CHK_LOG" 2>/dev/null; true)
ERRS=${ERRS:-0}
FILES=$(grep -oE '^ *--> [^:]+' "$CHK_LOG" 2>/dev/null | sed 's/.*--> //' | sort -u | wc -l | tr -d ' ')
FILES=${FILES:-0}
echo
echo "${RED}FAIL: a fresh checkout does not build (${ERRS} errors across ${FILES} files).${NC}"
echo "  The working tree may be fine — that is exactly the trap: uncommitted"
echo "  fixes make a local build pass while HEAD stays broken."
echo "  Group the errors before acting; most cascades from a few root causes:"
echo "  --- first errors ---"
grep -E '^error' -A2 "$CHK_LOG" | sed -n '1,40p' | sed 's/^/  /'
echo "  --- files with errors (${FILES}) ---"
grep -oE '^ *--> [^:]+' "$CHK_LOG" | sed 's/.*--> //' | sort -u | sed 's/^/  /'
cp "$CHK_LOG" /tmp/check-fresh-build.log 2>/dev/null
echo "  full log: /tmp/check-fresh-build.log"
rm -f "$CHK_LOG"
exit 1
