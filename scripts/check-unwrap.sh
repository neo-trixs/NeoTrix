#!/bin/bash
# Production unwrap/expect/panic ratchet.
#
# WHY THIS GATE EXISTS (2026-09-29)
#   `AGENTS.md` and `RUST-STANDARDS.md` both state: "生产代码禁
#   unwrap()/expect()/panic!()，错误用 ? 传播". Measured 2026-09-29:
#   **4,110 production `.unwrap()`** existed and **no gate or baseline in
#   the entire repo measured it**. The rule was written but never enforced,
#   so the stock is unguarded: new violations would land with no alarm.
#
#   This differs in kind from `check-layer-deps`'s 8 known violations: those
#   are recorded in `layer-deps-baseline.txt` and protected. These 4,110 were
#   naked.
#
# SCOPE — ratchet, NOT a purge.
#   This gate does NOT demand the stock go to zero. It records the current
#   state as a baseline list and fails only on **new** violations. Deleting
#   4,110 call sites is a separate, far larger project; pretending otherwise
#   would make this gate either useless (always red) or dishonest (ignore).
#
# R-SCAN-1 — this repo writes its own banned words as DATA.
#   Raw `grep '\.unwrap()'` returns 5,121; **230 of those are inside string
#   literals** (test fixtures like `let x = val.unwrap();` embedded in
#   `r#"..."#` for the shield/laws scanners). The same trap applies to
#   `unsafe` (see `nt_topology.py::_strip_noncode`). So counting is done in
#   Python with comments and literals stripped, never by grep.
#
# READ-ONLY by construction (R-EXIST-2): reads files, writes only the
# baseline when --update-baseline is passed explicitly. No cargo, no git
# mutation, no network.
#
# Usage:
#   bash scripts/check-unwrap.sh                    # advisory: report, never fail
#   bash scripts/check-unwrap.sh --strict          # exit 1 on new violations
#   bash scripts/check-unwrap.sh --update-baseline # rewrite the baseline
# Exit: 0 pass / 1 new violations (--strict) / 2 bad usage.

set -uo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "FAIL: not inside a git work tree" >&2; exit 2; }

STRICT=0
UPDATE=0
case "${1:-}" in
  "") ;;
  --strict)          STRICT=1 ;;
  --update-baseline) UPDATE=1 ;;
  *) echo "unknown arg: $1 (use --strict | --update-baseline)" >&2; exit 2 ;;
esac

BASELINE="scripts/unwrap-baseline.txt"

# Emit "<path>:<line>\t<token>" for every production unwrap/expect/panic.
# Shares the literal/comment stripper with nt_topology so both tools agree.
python3 - "$BASELINE" "$UPDATE" "$STRICT" <<'PY'
import os, re, sys

baseline_path, update = sys.argv[1], sys.argv[2] == "1"
# ⚠️ STRICT must be passed in, not read from the shell: the logic below runs
# inside the heredoc (Python scope). Reading a bare `STRICT` here raised
# NameError and crashed exactly when a violation existed — i.e. the gate
# failed open on the one input it exists to catch. Caught by feeding it a
# synthetic violation instead of reasoning about it (R-SCAN-2).
strict = sys.argv[3] == "1"
sys.path.insert(0, "scripts/ops")
try:
    from nt_topology import _strip_noncode
except Exception:
    def _strip_noncode(t):          # degraded but still read-only
        return "\n".join("" if l.strip().startswith("//") else l
                         for l in t.splitlines())

SKIP = ("target", ".git", "models", "node_modules", ".worktrees")
TOKENS = [("unwrap", re.compile(r"\.unwrap\(\)")),
          ("expect", re.compile(r"\.expect\(")),
          ("panic",  re.compile(r"\bpanic!\("))]

def is_production(path):
    """Production = not a test file, not a test module, not a test-only crate.

    `tests/`, `benches/`, `examples/` and `#[cfg(test)]` blocks are where
    unwrap is idiomatic and `RUST-STANDARDS` does not forbid it.

    ⚠️ The basename test matters as much as the directory one. Measured
    2026-09-29: `neotrix-core/src/bin/experience/tests.rs` is a test file
    that lives in `src/bin/`, NOT under a `tests/` directory. Matching only
    on directories counted it as production, so the first run reported 1,397
    where an independent recount said 1,398. The file is named `tests.rs`
    and holds 1 `unwrap`; it is excluded now.
    """
    base = os.path.basename(path)
    if any(s in path for s in ("/tests/", "/benches/", "/examples/")):
        return False
    stem = base[:-3] if base.endswith(".rs") else base
    if stem in ("test", "tests", "mod_tests") or stem.endswith(("_test", "_tests")):
        return False
    if stem.startswith("test_"):
        return False
    return True

hits = []
for dirpath, dirnames, filenames in os.walk("."):
    dirnames[:] = [d for d in dirnames
                   if d not in SKIP and not (d.startswith(".") and d != ".github")]
    for fn in sorted(filenames):
        if not fn.endswith(".rs"):
            continue
        full = os.path.join(dirpath, fn)
        rel = os.path.relpath(full, ".")
        if rel.startswith("..") or not is_production(rel):
            continue
        try:
            text = open(full, encoding="utf-8", errors="ignore").read()
        except OSError:
            continue
        code = _strip_noncode(text).splitlines()
        in_test_block = False
        depth = 0
        for i, line in enumerate(code, 1):
            if re.search(r"#\[cfg\(test\)\]", line):
                in_test_block = True
            if in_test_block:
                continue
            for name, rx in TOKENS:
                if rx.search(line):
                    hits.append((rel, i, name))

# Baseline is a LIST, never a count: a count would let "delete one, add one"
# hide forever. Same rationale as layer-deps-baseline.txt.
cur = {"%s:%d" % (p, l): n for p, l, n in hits}
have = {}
if os.path.exists(baseline_path):
    for line in open(baseline_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if line and not line.startswith("#"):
            have[line.split("\t")[0]] = line.split("\t")[-1]

if update:
    with open(baseline_path, "w", encoding="utf-8") as fh:
        fh.write("# Production unwrap/expect/panic baseline — generated 2026-09-29.\n")
        fh.write("# Ratchet: only NEW sites fail. Format = <path>:<line>\t<token>\n")
        fh.write("# Regenerate: bash scripts/check-unwrap.sh --update-baseline\n")
        fh.write("# A count would let 'delete one, add one' hide; this is a list.\n")
        for k in sorted(cur, key=lambda k: (cur[k], k)):
            fh.write("%s\t%s\n" % (k, cur[k]))
    print("[unwrap] baseline written: %d sites -> %s" % (len(cur), baseline_path))
    sys.exit(0)

new = sorted(set(cur) - set(have))
fixed = sorted(set(have) - set(cur))
by_tok = {}
for k in new:
    by_tok.setdefault(cur[k], []).append(k)

print("  production unwrap/expect/panic sites: %d" % len(cur))
print("  baseline entries:                   %d" % len(have))
if fixed:
    print("  ⛔ %d baseline site(s) no longer exist — baseline is stale:" % len(fixed))
    print("     (good news: someone fixed them. Re-run --update-baseline to shrink.)")
    for k in fixed[:5]:
        print("       - %s" % k)
if new:
    print("  NEW violations: %d" % len(new))
    for t in sorted(by_tok):
        print("    %s: %d" % (t, len(by_tok[t])))
        for k in by_tok[t][:8]:
            print("      + %s" % k)
        if len(by_tok[t]) > 8:
            print("      … %d more" % (len(by_tok[t]) - 8))

if not new:
    print("PASS: 0 new violation(s); %d known/recorded." % len(have))
    if have:
        print("      The standing debt is real and unresolved — see TODO.md.")
        print("      This gate protects against regression; it does NOT certify")
        print("      the codebase is panic-free. Clearing the stock is a")
        print("      separate project.")
    sys.exit(0)

if strict:
    print("FAIL: %d new violation(s). Fix them, or add to the baseline only if" % len(new))
    print("      each one is deliberate AND commented with a // SAFETY-style")
    print("      rationale. Do not baseline a sweep: that erases the signal.")
    sys.exit(1)
sys.exit(0)
PY
