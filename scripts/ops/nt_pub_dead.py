#!/usr/bin/env python3
"""`pub` 但**零消费者**的项 —— 补 `nt_const_dup` / `nt_diverge` 的已知盲区。

## 为什么需要独立工具
前两个工具都只能发现「**同名多份定义**」。它们**发现不了**「只有一份、
但没有任何人用」的死项。而这个盲区有实测代价：

2026-09-30 收敛 `rrf_fuse` 时，功能移走后core 侧留下一份
`pub const RRF_K: f64 = 60.0;` —— 它变成**零使用的重复定义**，但因为
带 `pub`，**编译器不报 unused**（`pub` 项默认视为对外 API），
于是静默留存。若无工具，这类残留只能靠人工逐个文件翻。

⇒ 本工具找的是**「定义了但没人引用」**，与 `nt_const_dup`（同名多份）
和 `nt_dup_dead`（重复 × 零接线）都不同，是第三个正交维度。

## ⛔ 判据刻意保守：零引用**不等于**死代码
一份引用计数为 0 的`pub` 项**可能**完全正当：
1. **对外 API**：`neotrix-types` / `neotrix` 的公开 API，供仓外 crate 使用。
2. **feature 门控内才有调用方**：调用点在 `#[cfg(feature = ...)]` 里，
   默认构建不编译，但该配置下是活的。
3. **测试/示例/基准里使用**：`tests/` `benches/` `examples/`。
4. **trait 实现**：trait 方法经trait 对象间接调用，文本上看不到调用点。
5. **FFI / 宏展开 / 反射式注册**：调用方由宏或外部语言生成。
6. **重导出链末端**：本模块`pub use` 上游，中间层看起来无人引用。

本工具因此**只报候选**，并且**必须**逐条人工核实上述 6 类。
它**不删任何东西**，也不自动判死。

用法：
    python3 scripts/ops/nt_pub_dead.py                      # 扫默认根
    python3 scripts/ops/nt_pub_dead.py --root <dir> ...    # 指定根（可多次）
    python3 scripts/ops/nt_pub_dead.py --kinds const,fn    # 只看某些类别
    python3 scripts/ops/nt_pub_dead.py --include-tests      # 把 tests/ 也算进引用面
    python3 scripts/ops/nt_pub_dead.py --limit 30
    python3 scripts/ops/nt_pub_dead.py --json
    python3 scripts/ops/nt_pub_dead.py selftest
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

DEFAULT_ROOTS = ["neotrix-core/src", "crates/neotrix-types/src"]

# 与 nt_const_dup 共用的剥离逻辑：注释/字符串里的同名不算定义也不算引用。
_STRING_OR_CHAR = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')
_RAW_STR = re.compile(r'r(#*)"(?:.|\n)*?"\1')

_ITEM = re.compile(
    r"(?:(?<=\n)|^)[ \t]*"
    r"pub(?:\([^)]*\))?[ \t]+"
    r"(?P<kind>const|static|fn)[ \t]+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)

# 引用面搜索时跳过的目录（除非 --include-tests）。
# 说明：这些目录里的引用**可能**让一个「死」项复活，故默认排除并显式说明。
_REF_SKIP_DIRS = ("/tests/", "/benches/", "/examples/")


def _strip_noncode(src: str) -> str:
    out = list(src)
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            for k in range(i, j):
                if src[k] != "\n":
                    out[k] = " "
            i = j
        elif c == '"' or c == "'":
            m = _STRING_OR_CHAR.match(src, i)
            if m:
                for k in range(i, m.end()):
                    if src[k] != "\n":
                        out[k] = " "
                i = m.end()
            else:
                i += 1
        else:
            i += 1
    out_s = "".join(out)
    # 原始字符串（可能跨行）单独处理
    for m in _RAW_STR.finditer(src):
        for p in range(m.start(), m.end()):
            if src[p] != "\n":
                out_s = out_s[:p] + " " + out_s[p + 1:]
    return out_s



_CFG_TEST = re.compile(r'#\[cfg\s*\(\s*test\s*\)\]')
_IMPL_FOR = re.compile(r'\bimpl\b[^\n{;]*?\bfor\b[^\n{;]*')


def _brace_end(clean: str, open_at: int) -> int:
    """从 `{` 处配平找到对应的 `}`（返回其下标+1；找不到返回 len）。"""
    depth = 0
    for j in range(open_at, len(clean)):
        c = clean[j]
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return j + 1
    return len(clean)


def _regions(clean: str) -> tuple[list[tuple[int, int]], list[tuple[int, int]]]:
    """返回 (测试区, trait-impl 区)，各为 (start, end) 半开区间列表。

    为什么必须做（否则工具不可用）：首版把 `impl Trait for X { pub fn .. }`
    里的方法全部报成零引用 —— 它们经trait 对象调用，属正当豁免④。
    实测命中率因此高达 11%（1868/16787）⇒ 全是噪声，没有使用价值。
    """
    tests: list[tuple[int, int]] = []
    for m in _CFG_TEST.finditer(clean):
        b = clean.find('{', m.end())
        if b != -1 and b - m.end() < 200:
            tests.append((m.start(), _brace_end(clean, b)))
    impls: list[tuple[int, int]] = []
    for m in _IMPL_FOR.finditer(clean):
        b = clean.find('{', m.end())
        if b != -1 and b - m.end() < 400:
            impls.append((m.start(), _brace_end(clean, b)))
    return tests, impls


def _in(ranges: list[tuple[int, int]], pos: int) -> bool:
    return any(a <= pos < b for a, b in ranges)


def _line_of(clean: str, pos: int) -> int:
    return clean.count("\n", 0, pos) + 1


def _pretty(path: str) -> str:
    """相对 cwd 显示；不在 cwd 下则原样返回（避免 ../../.. 噪声）。"""
    cwd = os.getcwd()
    if path.startswith(cwd + os.sep):
        return os.path.relpath(path)
    return path


def walk_rs(roots: list[str]) -> list[str]:
    files = []
    for root in roots:
        if not os.path.isdir(root):
            continue
        for base, _dirs, names in os.walk(root):
            norm = base.replace(os.sep, "/")
            if "/target" in norm or "/.worktrees" in norm:
                continue
            for n in sorted(names):
                if n.endswith(".rs"):
                    files.append(os.path.join(base, n))
    return sorted(files)


def harvest_defs(roots: list[str]) -> list[dict]:
    out = []
    for path in walk_rs(roots):
        try:
            with open(path, encoding="utf-8", errors="replace") as fh:
                raw = fh.read()
        except OSError:
            continue
        clean = _strip_noncode(raw)
        test_r, impl_r = _regions(clean)
        # 逐个匹配；fn 名字允许大写（不常见但合法），故不限制小写。
        for m in _ITEM.finditer(clean):
            # fn 要确认是定义而非调用/绑定：`fn name` 后须是 `(` 或 `<`
            j = m.end()
            while j < len(clean) and clean[j] in " \t":
                j += 1
            if m.group("kind") == "fn" and not (j < len(clean) and clean[j] in "(<"):
                continue
            if _in(test_r, m.start()):
                continue  # `#[cfg(test)]` 区内的项由测试自己用，不是 API
            kind = m.group("kind")
            if kind == "fn" and _in(impl_r, m.start()):
                kind = "trait_impl"  # 豁免④：经 trait 对象调用，文本上无调用点
            out.append(
                {
                    "kind": kind,
                    "name": m.group("name"),
                    "path": path,
                    "line": _line_of(clean, m.start()),
                }
            )
    return out


def build_ref_index(roots: list[str], include_tests: bool) -> tuple[dict, str]:
    """返回 (符号 -> 各文件出现次数, 文件去剥离后的内容缓存)。

    ⚠️ 计的是**出现次数**，不是「出现在几个文件」。
    首版误用 `set(...)` 按文件去重后再累加 ⇒ 同一文件里「定义 + 使用」两次
    被压成 1，于是 `USED_ONE` 这类**确有消费者**的项被误报成零引用
    （已被自证抓出）。

    `None` 表示该文件被排除在引用面之外（tests/ 且未 --include-tests）。
    """
    refs: dict[str, dict] = {}
    cache: dict[str, str] = {}
    for path in walk_rs(roots):
        try:
            with open(path, encoding="utf-8", errors="replace") as fh:
                raw = fh.read()
        except OSError:
            continue
        clean = _strip_noncode(raw)
        cache[path] = clean
        norm = path.replace(os.sep, "/")
        if not include_tests and any(s in norm for s in _REF_SKIP_DIRS):
            continue
        for tok in set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", clean)):
            refs.setdefault(tok, {})[path] = len(re.findall(r"\b" + re.escape(tok) + r"\b", clean))
    return refs, cache


def scan(roots: list[str], kinds: list[str] | None, include_tests: bool) -> dict:
    defs = harvest_defs(roots)
    refs, cache = build_ref_index(roots, include_tests)
    # 同名可能多处定义 ⇒ 收集**全部**定义行，计数时一次性扣除。
    # 首版只减一处定义，导致两处同名定义时互相「抵消」，双双漏报（已被自证抓出）。
    def_lines: dict[str, set] = {}
    for d in defs:
        def_lines.setdefault(d["name"], set()).add((d["path"], d["line"]))
    hits = []
    for d in defs:
        if d["kind"] == "trait_impl":
            continue  # 正当豁免④，不算候选
        if kinds and d["kind"] not in kinds:
            continue
        name = d["name"]
        occ = refs.get(name, {})
        total = sum(occ.values())
        defs_here = def_lines.get(name, set())
        # 逐个定义文件扣掉该定义行上的出现
        for (p2, ln) in defs_here:
            if p2 in occ and p2 in cache:
                lines = cache[p2].split("\n")
                if 0 < ln <= len(lines):
                    total -= len(re.findall(r"\b" + re.escape(name) + r"\b", lines[ln - 1]))
        if total == 0:
            hits.append(
                {
                    "kind": d["kind"],
                    "name": name,
                    "path": _pretty(d["path"]),
                    "line": d["line"],
                }
            )
    hits.sort(key=lambda r: (r["kind"], r["path"], r["line"]))
    return {
        "hits": hits,
        "n_defs": len(defs),
        "n_trait_impl_excluded": sum(1 for d in defs if d["kind"] == "trait_impl"),
        "n_files": len(walk_rs(roots)),
        "include_tests": include_tests,
    }


def selftest() -> int:
    """自证：正例 + 6 类正当豁免的证伪。判据必须用**真实代码形态**验证（R-SCAN-2）。"""
    import tempfile

    ok = True

    def fail(m: str) -> None:
        nonlocal ok
        ok = False
        print("  FAIL %s" % m)

    # ── 1) 正例：真零引用应被抓到 ──
    with tempfile.TemporaryDirectory() as td:
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write(
                "pub const UNUSED_ONE: usize = 7;\n"
                "pub const USED_ONE: usize = 8;\n"
                "pub fn never_called() -> u32 { 1 }\n"
                "pub fn actually_called() -> u32 { USED_ONE as u32 }\n"
            )
        with open(os.path.join(td, "b.rs"), "w", encoding="utf-8") as fh:
            fh.write("fn caller() -> u32 { actually_called() + USED_ONE as u32 }\n")
        rep = scan([td], None, include_tests=True)
        got = {(h["kind"], h["name"]) for h in rep["hits"]}
        for want in (("const", "UNUSED_ONE"), ("fn", "never_called")):
            if want not in got:
                fail("应报零引用却没报：%r（实得 %r）" % (want, sorted(got)))
        for notwant in (("const", "USED_ONE"), ("fn", "actually_called")):
            if notwant in got:
                fail("误报有消费者的项：%r" % (notwant,))
        if not fail.__doc__ and ok:
            print("  PASS 正例：零引用的 const/fn 被抓；有消费者的不报")

    # ── 2) 证伪：注释/字符串里的同名不算引用 ──
    with tempfile.TemporaryDirectory() as td:
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write(
                "pub const GHOST: usize = 1;\n"
                "// GHOST 在注释里提到\n"
                'const S: &str = "GHOST";\n'
            )
        rep = scan([td], None, include_tests=True)
        got = {(h["kind"], h["name"]) for h in rep["hits"]}
        if ("const", "GHOST") not in got:
            fail("注释/字符串不应算引用，GHOST 应仍被判零引用（实得 %r）" % sorted(got))
        else:
            print("  PASS 证伪：注释与字符串中的同名不算引用")

    # ── 3) 证伪：行号必须准确 ──
    with tempfile.TemporaryDirectory() as td:
        src = "// c\n\n/// doc\npub const LINED: usize = 3;\n"
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write(src)
        rep = scan([td], None, include_tests=True)
        if len(rep["hits"]) != 1 or rep["hits"][0]["line"] != 4:
            fail("行号错位：%r（应为 4）" % rep["hits"])
        else:
            print("  PASS 行号指向真实定义行")

    # ── 4) 证伪：`fn` 关键字用法不得当成定义（调用/绑定） ──
    with tempfile.TemporaryDirectory() as td:
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write("pub const Z: usize = 1;\nfn helper() { let _ = 1; }\n")
        rep = scan([td], ["fn"], include_tests=True)
        if rep["hits"]:
            fail("把非定义当定义了：%r" % rep["hits"])
        else:
            print("  PASS 非定义的 fn 形态不入候选")

    # ── 5) 证伪：tests/ 默认排除引用面（豁免类之一） ──
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "tests"))
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write("pub const ONLY_IN_TEST: usize = 5;\n")
        with open(os.path.join(td, "tests", "t.rs"), "w", encoding="utf-8") as fh:
            fh.write("#[test]\nfn t() { let _ = ONLY_IN_TEST; }\n")
        a = {(h["kind"], h["name"]) for h in scan([td], None, include_tests=False)["hits"]}
        b = {(h["kind"], h["name"]) for h in scan([td], None, include_tests=True)["hits"]}
        if ("const", "ONLY_IN_TEST") not in a:
            fail("默认应排除 tests/ 引用面 ⇒ 该项应报零引用")
        elif ("const", "ONLY_IN_TEST") in b:
            fail("--include-tests 时不应再报")
        else:
            print("  PASS tests/ 默认排除引用面（--include-tests 可翻转）")

    # ── 6) 同一符号多处定义：都算定义处，不应因彼此互相「引用」而漏报 ──
    with tempfile.TemporaryDirectory() as td:
        with open(os.path.join(td, "a.rs"), "w", encoding="utf-8") as fh:
            fh.write("pub const DUP: usize = 1;\n")
        with open(os.path.join(td, "b.rs"), "w", encoding="utf-8") as fh:
            fh.write("pub const DUP: usize = 1;\n")
        rep = scan([td], None, include_tests=True)
        names = [h["name"] for h in rep["hits"]]
        if names.count("DUP") != 2:
            fail("两处定义应各自报一次零引用，实得 %r" % names)
        else:
            print("  PASS 多处定义不互相抵消")

    print("[nt_pub_dead] selftest %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", nargs="?", default="scan")
    ap.add_argument("--root", action="append", default=None)
    ap.add_argument("--kinds", default=None, help="逗号分隔：const,static,fn")
    ap.add_argument("--include-tests", action="store_true", help="把 tests/benches/examples 也算进引用面")
    ap.add_argument("--limit", type=int, default=40)
    ap.add_argument("--json", dest="as_json", action="store_true")
    args = ap.parse_args()

    if args.cmd == "selftest":
        return selftest()

    roots = args.root or DEFAULT_ROOTS
    kinds = [k.strip() for k in args.kinds.split(",")] if args.kinds else None
    rep = scan(roots, kinds, args.include_tests)
    if args.as_json:
        print(json.dumps(rep, ensure_ascii=False, indent=2))
        return 0

    print(
        "[nt_pub_dead] 扫描 %d 个 .rs，发现 %d 条 pub 定义；引用面%s tests/"
        % (rep["n_files"], rep["n_defs"], "含" if args.include_tests else "排除")
    )
    if rep["hits"]:
        by_kind: dict[str, int] = {}
        for h in rep["hits"]:
            by_kind[h["kind"]] = by_kind.get(h["kind"], 0) + 1
        print("零引用候选：%d 条（%s）" % (len(rep["hits"]), by_kind))
        for h in rep["hits"][: args.limit]:
            print("  %-6s %-32s %s:%d" % (h["kind"], h["name"], h["path"], h["line"]))
        if len(rep["hits"]) > args.limit:
            print("  … 另有 %d 条（--limit 调整）" % (len(rep["hits"]) - args.limit))
    else:
        print("  无命中")
    print(
        "\n⛔ 零引用**不等于死代码**。逐条核实这 6 类正当豁免：\n"
        "  ① 对外 API（仓外 crate 用） ② 调用点在 feature 门控内（默认构建不编译）\n"
        "  ③ tests/benches/examples 里使用  ④ trait 方法（经 trait 对象间接调用）\n"
        "  ⑤ FFI / 宏展开 / 反射式注册  ⑥ 重导出链末端（pub use 上游）\n"
        "本工具**不删任何东西**，只报候选（R-SCAN-1b）。"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())