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

def test_ranges(code):
    """生产代码行号集合（0基），排除所有 `#[cfg(test)]` mod 块。

    ⭐⭐ 2026-10-07 替换「只置位不复位」的朴素 `in_test_block`。
    病根：原实现一旦见到 `#[cfg(test)]`，就把**该文件此后全部行**当测试代码跳过
    ⇒ 测试块**之后**的生产代码对门完全不可见。
    实测受害：`nt_memory/coverage_ledger.rs`（cfg(test) 在 477 行 / 共 566 行）。

    ⛔ 为什么不数括号：剥离注释后的 Rust 源码里括号不可靠（字符串/字符字面量、
    宏体、`#[cfg(x)] mod y {` 同行形态…）。实测三版括号计数互相矛盾。

    ⇒ 用**缩进配对**：测试块结尾 = 首个与 `mod` 行**同缩进**的裸 `}`。
      A. `#[cfg(test)] mod t {`（**同行**）⇒ 延伸到 EOF
      B. 属性**独占行** ⇒ 下一非空行是 `mod ... {`，按同缩进的裸 `}` 闭合
    """
    prod = set()
    i, n = 0, len(code)
    while i < n:
        line = code[i]
        if re.match(r"\s*#\[cfg\(", line) and re.search(r"\btest\b", line):
            t = line.strip()
            if re.match(r"#\[cfg\([^)]*\)\s*mod\s+\w+", t):
                i = n
                continue
            j = i + 1
            while j < n and (not code[j].strip() or code[j].strip().startswith("//")):
                j += 1
            t2 = code[j].strip() if j < n else ""
            if re.match(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+.*\{\s*$", t2):
                ind = len(code[j]) - len(code[j].lstrip())
                k = j + 1
                while k < n:
                    raw = code[k]
                    if raw.strip() == "}" and len(raw) - len(raw.lstrip()) == ind:
                        k += 1
                        break
                    k += 1
                i = k
                continue
        prod.add(i)
        i += 1
    return prod


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
        # ⭐ 生产行判定改用 test_ranges（见其 docstring）
        prod_lines = test_ranges(code)
        for i, line in enumerate(code, 1):
            if (i - 1) not in prod_lines:
                continue
            # ⭐⭐ 2026-10-02 修的**假阳性**：原先只认**字面** `#[cfg(test)]`，
            #    而 `#[cfg(all(test, feature = "…"))]` / `#[cfg(any(test, …))]`
            #    同样是「仅测试期编译」，却不被识别 ⇒ 该模块里的 unwrap 被算成**生产代码**。
            #    实测受害者：`nt_shield_sandbox/mod.rs:782` 的
            #    `#[cfg(all(test, feature = "sandbox"))] mod sandbox_vault_tests`
            #    ⇒ 850 那条测试里的 `expect("tempdir")` 被误报为生产违规。
            # ⭐ 判据口径：`#[cfg(…)]` 里**出现** `test` 这个 ident 即算测试边界。
            # ⓰ 注意：`_strip_noncode` 会**抹掉字符串字面量内容**，故门看到的
            #   `#[cfg(all(test, feature = "sandbox"))]` 实际是 `feature = ""`。
            #   本判据不依赖字面量内容 ⇒ 不受影响。
            #   ⛔ 刻意不改成「任何 cfg 都算」——那会把 feature 门（`#[cfg(feature=…)]`）
            #   的生产分支也吞掉，等于放过整片生产代码。
            #   ⛔ 曾试过 `re.match(r"#\[cfg\(([^)]*)\)\]")` 取 cfg 体 ——
            #     **失配**：`all(test, …)` 的 cfg 体里**自带括号**，`[^)]*` 匹配不到
            #     闭合的 `)]` ⇒ 整个正则不命中（实测：修完仍 26，850 仍在名单）。
            #   ⇒ 改为「是 cfg 属性」+「该行含 `test` ident」两条独立判据。
            for name, rx in TOKENS:
                if rx.search(line):
                    # ⭐ **内容锚点**（2026-10-07）：`路径:行号` 会因**行号漂移**失配
                    #   ⇒ 上游任何编辑都可能让「同一位点」看起来是「新增」（假警报），
                    #   或让既有的一条看起来「已修」（棘轮失真）。
                    #   实测旧键已陈旧 **132** 条。
                    # ⇒ 键改为 `路径 \x1f 所属函数名 \x1f 归一化代码行`。
                    #   · 所属函数：向上找最近的 `fn ` 定义行（找不到用 `<toplevel>`）
                    #   · 归一化：去掉所有空白 ⇒ 纯格式调整不触发新增
                    fn_name = "<toplevel>"
                    for k2 in range(i - 1, max(-1, i - 400), -1):
                        cand = code[k2]
                        m2 = re.search(r"\bfn\s+([A-Za-z_]\w*)", cand)
                        if m2:
                            fn_name = m2.group(1)
                            break
                    norm = re.sub(r"\s+", "", line)
                    hits.append((rel, i, name, fn_name, norm))

# Baseline is a LIST, never a count: a count would let "delete one, add one"
# hide forever. Same rationale as layer-deps-baseline.txt.
SEP = "\x1f"


def anchor(path, fn_name, norm):
    """内容锚点键：**行号漂移免疫**。"""
    return "%s%s%s%s%s" % (path, SEP, fn_name, SEP, norm)


# `cur` 用内容锚点作键；同时保留 `路径:行号` 索引，供 v1 旧账本过渡期折算
cur = {}
cur_by_line = {}
for _p, _l, _n, _fn, _norm in hits:
    _a = anchor(_p, _fn, _norm)
    cur[_a] = _n
    cur_by_line["%s:%d" % (_p, _l)] = (_a, _n)
have = {}
if os.path.exists(baseline_path):
    for line in open(baseline_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if line and not line.startswith("#"):
            key = line.split("\t")[0]
            tok = line.split("\t")[-1]
            if SEP in key:
                # v2 内容锚点键 ⇒ 直接采用
                have[key] = tok
            elif key in cur_by_line:
                # v1 `路径:行号` ⇒ **仅当该位点此刻仍被检出**时才折算成锚点。
                # ⛔ 绝不无条件折算：旧账本那 132 条「已不存在」的条目若被折算，
                #    会凭空造出 never-seen 的锚点 ⇒ **等于洗白**。
                have[cur_by_line[key][0]] = tok

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
        # ⚠️⛔ 原为 `by_tok[t][:8]`：**每种 token 只打印前 8 条**。
        #   ⇒ 门探针 `check-unwrap` 断言「输出里指名注入文件」，
        #     而注入文件常排在后面 ⇒ **NEW 一旦超过 8，探针必假失败**。
        #   实测：NEW 16 时靠分组侥幸通过；NEW 升到 28 时注入项被挤出窗口 ⇒ 探针变红。
        # ⇒ 结论：**这个探针从未真正证明过本门**（它只在小规模下成立）。
        # ⇒ 现在打印全部（行数可控，且「边界可见」优先于「输出短」）。
        for k in by_tok[t]:
            print("      + %s" % k)

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
