#!/bin/bash
# Silent-failure gate — "does a discarded Result get observed anywhere?"
#
# Why this gate exists (2026-09-30, audit finding, not hypothetical):
#
#   A full audit asked: with --all-targets green, 12,209 tests green, 7 gates
#   rc=0 and forbid(unsafe_code) in force, what failure mode is still invisible?
#   Answer: silent failure. 4 P0 instances, each verified by reading the site:
#
#     agent.rs:823,833      stdio MCP tool channel: request write and stdout
#                           decode both discarded, yet `success` is only
#                           status.success() => a dead subprocess yields
#                           Ok(success:true, content:"") and the LLM reasons on it
#     safe_applier.rs:102   rollback write discarded, error string hardcodes
#                           "已回滚" whether or not the rollback happened
#     experience_tree:583   session evidence write discarded, hook returns ok()
#     write_guard.rs:171    doc says "失败仅告警", code is `let _ =` (no warning)
#                           => fewer recorded blocks => NT-SHIELD audit reports
#                              a GREEN light
#
#   None of these can be caught by a green build or a green test suite, because
#   THE FAILURE BRANCH HAS NO ASSERTION.
#
# The rule is NOT invented here. It is already written in this repo, at
# nt_channel_serve.rs:278:
#
#   "失败不阻断本轮出站（best-effort 语义），但不吞 —— `unwrap_or(0)` 会伪装成
#    『删了 0 行』"
#
# and the correct counter-pattern already exists 20 lines from a violation, at
# nt_dispatch_loop.rs:898:  Err(e) => (false, format!("... 失败: {e}"))
#
# So the repo already knows the answer; the knowledge is just not propagated.
# This gate is that propagation, mechanised.
#
#   RULE: a `let _ =` that discards a persistence / send / write call must have,
#         in the same enclosing function, at least one observation channel:
#         log::warn|error|info!  |  eprintln!/println!  |  a report/counter push.
#         No observation channel => the failure is silent => report it.
#
# Deliberate non-goals (a gate nobody trusts is worse than no gate):
#   - It does NOT judge whether best-effort is the RIGHT policy. Many discards
#     are correct (e.g. nt_crystal_task_fusion.rs:749 `tx.send` where the
#     receiver provably outlives the sender). Those are baselined with a reason.
#   - It is a RATCHET, same shape as check-unwrap.sh: a list, never a count, so
#     "delete one, add one" cannot hide.
#   - Precision over recall. A false positive costs reviewer trust, so the
#     persistence/IO predicate is a closed list, not a heuristic over any `()`.
#
# Usage:
#   bash scripts/check-silent-failure.sh                 # report new sites
#   bash scripts/check-silent-failure.sh --strict        # exit 1 on new sites
#   bash scripts/check-silent-failure.sh --list          # every hit + reason
#   bash scripts/check-silent-failure.sh --update-baseline
#
# Exit: 0 pass / 1 new sites (--strict) / 2 bad usage.
#
# Write operations: reads (rg/find/grep/python) plus "cargo" is NOT invoked at
# all — this gate is pure static analysis and is safe to dry-run. Audited per
# R-SCAN-4 before first run: no rm, no git state change, no backtick substitution.

set -uo pipefail

BASELINE="scripts/silent-failure-baseline.txt"
MODE=report
case "${1:-}" in
  "")                 MODE=report ;;
  --strict)           MODE=strict ;;
  --list)             MODE=list ;;
  --update-baseline)  MODE=update ;;
  *) echo "unknown arg: $1" >&2; exit 2 ;;
esac

if [ ! -d neotrix-core/src ]; then
  echo "run from repo root (no neotrix-core/src)" >&2; exit 2
fi

python3 - "$BASELINE" "$MODE" <<'PY'
import os, re, sys

baseline_path, mode = sys.argv[1], sys.argv[2]

sys.path.insert(0, "scripts/ops")
try:
    from nt_topology import _strip_noncode
except Exception:
    # Degraded but still read-only. If this path is ever taken the gate would
    # count comment/string occurrences, so it says so out loud instead of
    # silently reporting a different number than check-unwrap/nt_topology.
    sys.stderr.write("[silent-failure] WARNING: _strip_noncode unavailable, "
                     "comment/literal stripping degraded\n")
    def _strip_noncode(t):
        return "\n".join("" if l.strip().startswith("//") else l
                         for l in t.splitlines())

ROOTS = ["neotrix-core/src"] + [
    os.path.join("crates", d, "src")
    for d in sorted(os.listdir("crates")) if os.path.isdir(os.path.join("crates", d, "src"))
]

# --- GATED: high-signal. Failure changes what the caller BELIEVES. ----------
#
# create_dir_all is EXCLUDED. Measuring it produced 128 sites, most of them
# constructors where "ensure the dir exists" failing is immediately followed by
# a write that fails loudly. A signal that fires on correct code is not a signal.
GATED = re.compile(
    r"(\.(kv_set|kv_del|set_len)\s*\()"          # state/evidence not recorded
    r"|(\b(fs::|std::fs::|File::)"
    r"(write|read_to_string|read_to_end|rename|append|OpenOptions)\b)"  # data not persisted
    r"|(\bwriteln?!\s*\(\s*(?!std(out|err)))"     # manual write! to a handle
)
# --- REPORTED BUT NOT GATED: systematically ambiguous semantics -------------
#
# Measured inventory on 2026-09-30: 35x fs::remove_file, 20x .send*.
#   remove_file — in cleanup/prune paths, "best-effort delete" is frequently the
#     intended policy, and the surviving file is the safe outcome. Gating it
#     would be ~35 mostly-correct sites, which is how a gate loses its readers.
#   .send       — discarding a channel send is CORRECT whenever the receiver
#     outlives the sender (proved case: nt_crystal_task_fusion.rs:749, where
#     thread::scope + drop(tx) precede the receive loop). Deciding that needs
#     reachability analysis this gate does not do and will not fake.
#
# Excluding them is a SCOPE decision, not a silent gap: the count is printed on
# every run so the boundary of the gate is always visible.
AMBIGUOUS = re.compile(
    r"(\b(fs::|std::fs::)(remove_file|create_dir_all)\b)"
    r"|(\b(send|send_bytes|deliver|dispatch|emit|publish|enqueue|reply|post|upload)\w*\s*\()")

# Only unambiguous observation channels. An earlier, looser version also accepted
# `.push(` / `record_` / `report.` and that produced a real false negative:
# safe_applier.rs:102 `let _ = fs::write(...)` sat in the same block as
# `self.tracker.record_change(..).ok()`, and `record_` made the block look
# observed. For a ratchet, a missed bug costs far more than a baselined false
# positive, so bias to recall and baseline the noise.
OBSERVE = re.compile(
    r"log::(error|warn|info|debug|trace)!|eprintln!|println!|print!|"
    r"errors\.push\(|\.errors\b|warned|failures\.push\(")

# Test modules are not production silent failures. `mod tests` inside a file is
# the common shape, but this repo also has whole files that are test modules
# (nt_file_ability/tests.rs, and `#[cfg(test)]`-only files).
TEST_FILE = re.compile(r"(^|/)(tests?\.rs|test_[^/]*\.rs|[^/]*_tests?\.rs)$")


def enclosing_fn(lines, i):
    """Walk back to the nearest `fn ` at lower indent; forward to its closing brace."""
    indent = None
    start = i
    for j in range(i, -1, -1):
        s = lines[j]
        m = re.match(r"(\s*)(pub(\([^)]\))?\s+)?(async\s+)?fn\s", s)
        if m:
            indent = len(m.group(1))
            start = j
            break
    if indent is None:
        return start, i
    end = len(lines) - 1
    for j in range(i, len(lines)):
        s = lines[j]
        if s.strip() == "}" and (len(s) - len(s.lstrip())) == indent:
            end = j
            break
    return start, end


def enclosing_block(lines, i):
    """The INNERMOST block containing line i.

    Function granularity is too coarse and produced three misses against known
    P0s. The clearest one is guardian.rs: create_dir_all at :365 HAS a
    log::warn!, the fs::rename at :372 that delivers the SAME atomicity does
    NOT — both in one function. Function-level granularity let the first cover
    for the second. The question the gate asks is "is THIS discard observed",
    so the scope must be the block the discard sits in.
    """
    # forward boundary: the closing brace at the discard's own indent
    own = len(lines[i]) - len(lines[i].lstrip())
    close = len(lines) - 1
    for j in range(i, len(lines)):
        s = lines[j]
        if s.strip().startswith("}") and (len(s) - len(s.lstrip())) <= own:
            close = j
            break
    # backward boundary: nearest line at lower-or-equal indent that opens a block
    open_ = None
    for j in range(i, -1, -1):
        s = lines[j]
        ind = len(s) - len(s.lstrip())
        if s.strip() and ind < own and s.rstrip().endswith("{"):
            open_ = j
            break
        if s.strip() and ind < own and re.match(r"\s*fn\s", s):
            open_ = j
            break
    if open_ is None:
        open_ = 0
    return open_, close

def is_test(lines, i):
    for j in range(max(0, i - 400), i + 1):
        if re.match(r"\s*mod\s+tests\b", lines[j]):
            return True
    return False

hits = {}
ambiguous = 0
for root in ROOTS:
    for dirpath, _d, files in os.walk(root):
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            p = os.path.join(dirpath, fn)
            if TEST_FILE.search(p.replace(os.sep, "/")):
                continue
            try:
                text = open(p, encoding="utf-8", errors="ignore").read()
            except Exception:
                continue
            lines = _strip_noncode(text).splitlines()
            for i, line in enumerate(lines):
                # ⭐⭐⭐ 2026-10-02 修的**盲区 B1**：原正则
                #   `let\s+_\s*=\s*(.+?);\s*$` 要求分号在**同一行行尾**，
                #   而本仓大量调用是**多行**形态，例如：
                #       let _applied: Option<usize> =
                #           self.conn.execute(alter, []).ok();
                #   ⇒ 这些行**结构上**永远不匹配 ⇒ GATED 判据再严也看不到。
                #   实测规模：全仓 `let _ = ` 1493 行，单行形态仅 1235 行
                #   ⇒ **258 处多行开头全部隐形**（其中 22 处同时满足
                #   「GATED 调用 + 块内无观察通道 + 非测试」，即**语义上已是
                #   静默失败但门看不见」）⇒ 门少算 39%。
                #
                # ⭐ 单调性论证：`_strip_noncode` 自述「preserving line structure」，
                #   只删行内内容、**不会追加 `;`** ⇒ 原始行不匹配 ⇒ 剥离后也不可能匹配。
                #
                #   ⇒ 改为：先认 opener，再**沿后续行按括号配平**累积到收尾的 `;`。
                m = re.search(r"let\s+_\s*=\s*(.*)$", line)
                if not m:
                    continue
                rhs = m.group(1)
                if not rhs.rstrip().endswith(";"):
                    # 多行形态：向下累积直到括号配平且该行以 `;` 收尾。
                    depth = rhs.count("(") - rhs.count(")")
                    j = i
                    while j + 1 < len(lines) and (depth > 0 or not rhs.rstrip().endswith(";")):
                        j += 1
                        nxt = lines[j]
                        rhs += " " + nxt.strip()
                        depth += nxt.count("(") - nxt.count(")")
                        if depth <= 0 and nxt.rstrip().endswith(";"):
                            break
                    else:
                        # 未在文件内闭合 ⇒ 交给下一行重新起判，**不**误报。
                        continue
                    if depth > 0:
                        continue
                    rhs = rhs.rstrip()
                    if rhs.endswith(";"):
                        rhs = rhs[:-1]
                # strip a leading `mut ` and the receiver chain noise
                if AMBIGUOUS.search(rhs):
                    ambiguous += 1
                if not GATED.search(rhs):
                    continue
                if is_test(lines, i):
                    continue
                a, b = enclosing_block(lines, i)
                body = "\n".join(lines[a:b + 1])
                if OBSERVE.search(body):
                    continue          # observed => not silent, by definition
                hits["%s:%d" % (p, i + 1)] = rhs.strip()[:88]

have, judged = set(), {}
if os.path.exists(baseline_path):
    for line in open(baseline_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if line and not line.startswith("#"):
            parts = line.split("\t")
            have.add(parts[0])
            crit = parts[2] if len(parts) > 2 and parts[2] else "-"
            orc = parts[3] if len(parts) > 3 and parts[3] else "-"
            judged[parts[0]] = (crit, orc)

unjudged = sorted(k for k in have if judged.get(k, ("-", "-"))[0] == "-")

new = sorted(set(hits) - have)
stale = sorted(have - set(hits))

print("  [silent-failure] discarded IO/persistence results: %d" % len(hits))
print("  [silent-failure] baselined (recorded, not silent-by-omission): %d" % len(have))
print("  [silent-failure] out of scope by design (remove_file/.send*): %d" % ambiguous)
if mode != "list":
    print("  [silent-failure] OPEN CONTRACTS (baseline row with no criterion): %d/%d"
          % (len(unjudged), len(have)))

if mode == "list":
    for k in sorted(hits):
        print("    %-62s %s" % (k, hits[k]))
    print("  (list mode ignores baseline: this is the full current inventory)")
    sys.exit(0)

if mode == "update":
    with open(baseline_path, "w", encoding="utf-8") as fh:
        fh.write("# Silent-failure baseline — a LIST, never a count (same rationale as\n")
        fh.write("# unwrap/layer-deps): a count would let 'delete one, add one' hide.\n")
        fh.write("# Format: <path>:<line>\t<discarded call>\t<criterion|->\t<oracle|->\n")
        fh.write("#\n# Contract columns (2026-09-30, GLOBAL-MAP-DEFECTS 建议 A 第一刀):\n")
        fh.write("#   criterion = WHY this discard is acceptable OR what would make it a bug.\n")
        fh.write("#              '-' means UNJUDGED -> the gate reports it as an open contract.\n")
        fh.write("#   oracle    = WHAT proves it still holds (a test name, a command, or a\n")
        fh.write("#              manual verification note). '-' means UNVERIFIED.\n")
        fh.write("#   Rationale: a baseline row is a CLAIM. Without criterion+oracle it is\n")
        fh.write("#   indistinguishable from an unexamined leftover.\n")
        fh.write("# Regenerate: bash scripts/check-silent-failure.sh --update-baseline\n")
        fh.write("#\n# A baseline entry means: KNOWN, and deliberately best-effort. The audit\n")
        fh.write("# that produced it (docs: TODO.md §2026-09-30 全量审计) lists why per site.\n")
        for k in sorted(hits, key=lambda k: (hits[k], k)):
            # Preserve any judgement already recorded, so re-running
            # --update-baseline never silently discards a reviewed contract.
            fh.write("%s\t%s\t%s\t%s\n" % (k, hits[k],
                                               judged.get(k, ("-", "-"))[0],
                                               judged.get(k, ("-", "-"))[1]))
    print("  [silent-failure] baseline written: %d sites" % len(hits))
    sys.exit(0)

if stale:
    print("  ⛔ %d baseline site(s) no longer exist — baseline is stale:" % len(stale))
    print("     (good news: someone fixed them. Re-run --update-baseline to shrink.)")
    for k in stale[:5]:
        print("     %s" % k)
    if len(stale) > 5:
        print("     … %d more" % (len(stale) - 5))

if new:
    print("  ⛔ %d NEW silent failure(s) — a discarded Result with no observation channel:" % len(new))
    print("     same function must have log::!/eprintln!/report push, or propagate with `?`")
    for k in new[:20]:
        print("     + %-58s %s" % (k, hits[k]))
    if len(new) > 20:
        print("     … %d more" % (len(new) - 20))
    if mode == "strict":
        print("  [silent-failure] FAIL: %d new" % len(new))
        sys.exit(1)

print("  [silent-failure] PASS")
sys.exit(0)
PY
