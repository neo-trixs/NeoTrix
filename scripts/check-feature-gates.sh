#!/bin/bash
# Feature-gate buildability gate — "a non-default feature must also compile".
#
# Why this gate exists (2026-09-30, real regression, not hypothetical):
#
#   Today I deleted 10 re-exports from neotrix-core/src/neotrix/mod.rs on the
#   evidence that "rg found zero consumers". The real consumer was
#   l0_substrate/ffi/skill_tree.rs, written as a BRACED import:
#
#       use crate::neotrix::{CapabilityTreeRegistry, NodeLayer};
#
#   My search pattern was 'neotrix::(CapabilityTreeRegistry|NodeLayer|...)',
#   which requires the name to follow '::' directly and therefore cannot match
#   a braced import. Zero hits -> I concluded "zero consumers" -> deleted live code.
#
#   EVERY ordinary gate stayed green while I did it:
#     - cargo check -p neotrix --lib            0 error
#     - cargo check -p neotrix --all-targets    0 error
#     - cargo check -p neotrix --tests          0 error
#     - cargo test  -p neotrix --lib            12,209 passed / 0 failed
#     - check-layer-deps / check-layout / check-doc-drift   all rc=0
#
#   Reason: ffi/ sits behind #[cfg(feature = "ios-bridge")], which is NOT in the
#   default feature set, so the default build never compiles it. The whole
#   default-build verification surface is structurally blind to it.
#
#   This is the same shape as the class check-fresh-build.sh documents: a real
#   failure mode that the existing gates are architecturally unable to see. And
#   it is the general lesson, stated in its strongest form:
#
#       ZERO HITS IS NOT EVIDENCE OF ZERO CONSUMERS.
#       It is only evidence that one pattern found nothing.
#
# CI already covers this — .github/workflows/ci.yml has a feature matrix that
# includes ios-bridge, plus a drift self-check. So the infrastructure was right
# and the local verification habit was wrong. This gate makes the local surface
# match the CI surface, so the regression is caught before commit instead of by
# a later CI run.
#
# Usage:
#   bash scripts/check-feature-gates.sh --list    # cheap: which features, no cargo
#   bash scripts/check-feature-gates.sh           # cargo check each gated feature
#   bash scripts/check-feature-gates.sh --quick   # lib + tests, skip examples/benches
#
# Mode coverage (both proved by injected-break test on 2026-09-30):
#   --quick  -> --lib --tests    catches test-target breaks; skips examples/benches
#   (default)-> --all-targets     also catches examples/benches (this is what caught
#                                  the ort Outlet private-field break)
#   NOTE: --lib alone must NEVER be used here. The 3 prefer_free errors were all in
#   #[cfg(test)] blocks; --lib reported them clean.
#
# Exit: 0 pass / 1 build error / 2 bad usage / 3 parse failure (see PARSE below).
#
# Write operations: reads (rg/find/grep) plus "cargo check", which writes only to
# target/. No rm, no git state change, no file creation. Audited per R-SCAN-4.

set -uo pipefail

CRATE=neotrix-core
MODE=full
case "${1:-}" in
  --list)  MODE=list ;;
  --quick) MODE=quick ;;
  "")      MODE=full ;;
  *) echo "unknown arg: $1 (use --list | --quick | no arg)" >&2; exit 2 ;;
esac

ROOT="neotrix-core/src"
[ -d "$ROOT" ] || { echo "no $ROOT — run from repo root" >&2; exit 2; }

# ---------------------------------------------------------------------------
# 1. Which non-default features gate a `mod` declaration?
#
# PARSE FAILURE IS FATAL BY DESIGN. A silent zero here is exactly the bug class
# this gate exists to stop: if the pairing misses a construct, the gate would
# report "nothing to check" and go green while blind. So unparsed cfg lines are
# surfaced and exit 3, rather than dropped.
# ---------------------------------------------------------------------------

OUT=$(python3 - "$ROOT" <<'PY'
import os, re, sys

root = sys.argv[1]
default = set()
cargo = open(os.path.join(os.path.dirname(root.rstrip('/')), 'Cargo.toml'),
             encoding='utf-8', errors='ignore').read()
m = re.search(r'^default\s*=\s*\[(.*?)\]', cargo, re.S | re.M)
if m:
    default = set(re.findall(r'"([^"]+)"', m.group(1)))

gated, unparsed = {}, []
for dirpath, _dirs, files in os.walk(root):
    for fn in files:
        if not fn.endswith('.rs'):
            continue
        p = os.path.join(dirpath, fn)
        lines = open(p, encoding='utf-8', errors='ignore').read().splitlines()
        for i, line in enumerate(lines):
            if 'cfg(feature' not in line:
                continue
            feats = re.findall(r'feature\s*=\s*"([^"]+)"', line)
            if not feats:
                unparsed.append((p, i + 1, line.strip()))
                continue
            for j in range(i + 1, min(i + 4, len(lines))):
                if re.match(r'^\s*(pub\s+)?mod\s+\w+\s*;', lines[j]):
                    for f in feats:
                        gated.setdefault(f, []).append("%s:%d" % (p, j + 1))
                    break

non_default = sorted(f for f in gated if f not in default)
# One GATED line per feature, NOT a tab-joined single line: the shell side reads
# field $2 of each line, so a joined line would silently yield only the first
# feature. That bug shipped for one run and was caught only because --list and
# the run mode disagreed on the count — keep the two modes fed by the same shape.
for f in non_default:
    print("GATED\t%s" % f)
for f in non_default:
    print("WHERE\t%s\t%s" % (f, ",".join(sorted(set(gated[f])))))
print("UNPARSED\t%d" % len(unparsed))
for p, ln, txt in unparsed[:10]:
    print("UNPARSED_WHERE\t%s:%d\t%s" % (p, ln, txt[:100]))
PY
)

FEATURES=$(printf '%s\n' "$OUT" | awk -F'\t' '$1=="GATED"{print $2}')
UNPARSED=$(printf '%s\n' "$OUT" | awk -F'\t' '$1=="UNPARSED"{print $2}')

if [ -z "$FEATURES" ]; then
  echo "  [feature-gates] no non-default feature gates a mod declaration"
  echo "  (if you expected some, the parse is broken — see UNPARSED below)"
fi

printf '  [feature-gates] gated by non-default feature: %s\n' "$(printf '%s' "$FEATURES" | tr '\n' ' ' | sed 's/ $//')"

if [ "$MODE" = list ]; then
  printf '%s\n' "$OUT" | awk -F'\t' '$1=="WHERE"{printf "    %-14s %s\n", $2, $3}'
  if [ "${UNPARSED:-0}" != "0" ]; then
    echo "  [feature-gates] ⛔ $UNPARSED cfg(feature) line(s) the parse could not read:"
    printf '%s\n' "$OUT" | awk -F'\t' '$1=="UNPARSED_WHERE"{printf "    %s  %s\n", $2, $3}'
    exit 3
  fi
  exit 0
fi

# ---------------------------------------------------------------------------
# 2. Drift: every gated feature must be in the CI matrix too, else the feature
#    is gated locally AND invisible upstream.
# ---------------------------------------------------------------------------
CIW=.github/workflows/ci.yml
DRIFT=0
if [ -f "$CIW" ]; then
  for f in $FEATURES; do
    if ! grep -qE "^\s+- ${f}\s*$" "$CIW"; then
      echo "  [feature-gates] ⚠️  '$f' gates a module but is absent from the CI matrix"
      DRIFT=1
    fi
  done
else
  echo "  [feature-gates] ⚠️  $CIW not found — cannot check matrix drift"
  DRIFT=1
fi

# ---------------------------------------------------------------------------
# 3. Build each gated feature.
# ---------------------------------------------------------------------------
FAILED=""
for f in $FEATURES; do
  printf '  [feature-gates] cargo check --features %-14s ' "$f"
  LOG=$(mktemp)
  if [ "$MODE" = quick ]; then
    # --lib ALONE IS A FOOTGUN for this gate, proven by falsification test on
    # 2026-09-30: the very break this gate was born for (NeoTrixConfig.prefer_free
    # missing from 3 initializers) lives entirely in #[cfg(test)] blocks, so
    # `cargo check --lib` reported it clean and --quick exited 0. --tests is
    # therefore mandatory here, not optional. This skips examples/benches only.
    cargo check -p neotrix --lib --tests --features "$f" >"$LOG" 2>&1
  else
    cargo check -p neotrix --all-targets --features "$f" >"$LOG" 2>&1
  fi
  rc=$?
  n=$(grep -cE '^error' "$LOG")
  if [ "$rc" -eq 0 ]; then
    echo "ok"
  else
    echo "⛔ $n error (rc=$rc)"
    grep -E '^error' -A3 "$LOG" | head -20 | sed 's/^/      /'
    FAILED="$FAILED $f"
  fi
  rm -f "$LOG"
done

if [ "${UNPARSED:-0}" != "0" ]; then
  echo "  [feature-gates] ⛔ $UNPARSED cfg(feature) line(s) unreadable — parse gap, see --list"
  exit 3
fi
if [ -n "$FAILED" ]; then
  echo "  [feature-gates] FAIL: feature(s) broken:$FAILED"
  exit 1
fi
if [ "$DRIFT" -eq 1 ]; then
  echo "  [feature-gates] FAIL: matrix drift (a gated feature is not in CI)"
  exit 1
fi
echo "  [feature-gates] PASS"
exit 0

