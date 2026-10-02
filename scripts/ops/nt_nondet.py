#!/usr/bin/env python3
"""确定性缺陷扫描：在**无序容器**上做极值选择 / 排序，且缺tie-break。

## 为什么需要独立工具
2026-09-30 一个会话内**三次**踩同一类缺陷：

| # | 位置 | 症状 |
|---|---|---|
| 1 | `hybrid_retrieval/bm25_search.rs` | `sort_by` 只比 score，doc1/doc3 分数**完全相同**（实测 0.4700036292）⇒ `results[0]` 纯看哈希种子 ⇒ 测试随机失败 |
| 2 | `session_replay/context_router/analyzer.rs` | `HashMap::iter().max_by_key()` 取「主导 agent」，agent_a/agent_b 计数并列 ⇒ 输出随机 |
| 3 | 同上文件 `dominant_task` | 同根因 |

⇒ 三次都是**「结果依赖哈希迭代序」**。这是**真缺陷**而非风格问题：
它让程序在**相同输入下产生不同输出**，且**只在特定运行里暴露**。

## 判据（刻意分层，避免淹没真缺陷）
只报**两类高置信**形态：

1. **`max_by_key` / `min_by_key`**：这两个方法**签名里就没有tie-break 的位置**
   ⇒ 并列时**必然**未定义。无论是否并列概率高，都值得看。
2. **`sort_by` / `sort_unstable_by`**：只有当**上游迭代对象是无序容器**
   （`HashMap`/`HashSet`）时才报 —— 因为 `sort_by` 虽然是稳定排序，
   但**输入顺序本身**已由哈希决定，稳定排序救不了。
   `sort_unstable_by` 额外不稳定（并列时重排），单独标注。

**不报**的形态：
· `sort_by_key` / `max_by`（比较器）—— `max_by` 可以自己写 tie-break，
  需逐条读比较器才能判，本工具不猜；
· 链式表达式里**看不出**上游容器类型的情形（宁可漏报也不误报）；
· 注释、字符串里的同名文本（已剥离）。

## ⛔ 本工具只报候选，不判改
「该加tie-break」还是「该接受并列任意」是**产品/语义决策**：
有时两个候选**确实等价**，任意一个都对。
⇒ 每个命中都要读那一行本身（R-SCAN-1b），确认并列是否**真的有害**。

用法：
    python3 scripts/ops/nt_nondet.py                     # 扫默认根
    python3 scripts/ops/nt_nondet.py --root <dir> ...    # 指定根（可多次）
    python3 scripts/ops/nt_nondet.py --kind by_key       # 只看 max_by_key/min_by_key
    python3 scripts/ops/nt_nondet.py --kind sort         # 只看无序容器上的排序
    python3 scripts/ops/nt_nondet.py --limit 40
    python3 scripts/ops/nt_nondet.py --json
    python3 scripts/ops/nt_nondet.py selftest
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

DEFAULT_ROOTS = ["neotrix-core/src", "crates/neotrix-types/src"]

_STRING_OR_CHAR = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')
_RAW_STR = re.compile(r'r(#*)"(?:.|\n)*?"\1')

# 无序容器：HashMap / HashSet（含 std::collections:: 前缀与 use 别名 HashMap）
_UNORDERED = re.compile(r'\b(?:std::collections::)?(?:HashMap|HashSet)\b')

# 极值：无 tie-break 位置
_EXTREMUM = re.compile(r'\.(?P<op>max_by_key|min_by_key)\s*\(')
# 排序：需上游是无序容器才报
_SORT = re.compile(r'\.(?P<op>sort_by|sort_unstable_by|sort_by_key|sort_unstable_by_key)\s*\(')
# 显式 tie-break（则不报 sort 类）
_TIEBREAK = re.compile(r'\.(?:then|then_with|then_with_key)\s*\(')


def _strip_noncode(src: str) -> str:
    """剥离注释与字符串（等长替换，保持行号可算）。"""
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
    s = "".join(out)
    for m in _RAW_STR.finditer(src):
        for p in range(m.start(), m.end()):
            if src[p] != "\n":
                s = s[:p] + " " + s[p + 1:]
    return s


def _line_of(clean: str, pos: int) -> int:
    return clean.count("\n", 0, pos) + 1


def _upstream_window(clean: str, pos: int, back: int = 420) -> str:
    """取该调用点**上游**的一段文本，用于判断迭代对象是否无序容器。"""
    return clean[max(0, pos - back):pos]


def scan_file(path: str) -> list[dict]:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            raw = fh.read()
    except OSError:
        return []
    clean = _strip_noncode(raw)
    hits: list[dict] = []

    # ① 极值类：max_by_key / min_by_key —— 无 tie-break 位置，一律报
    for m in _EXTREMUM.finditer(clean):
        win = _upstream_window(clean, m.start())
        line = clean[m.start():clean.find("\n", m.start())].strip()
        hits.append({
            "kind": "extremum",
            "op": m.group("op"),
            "path": path,
            "line": _line_of(clean, m.start()),
            "unordered_upstream": bool(_UNORDERED.search(win)),
            "snippet": line[:96],
        })

    # ② 排序类：需上游含无序容器
    for m in _SORT.finditer(clean):
        win = _upstream_window(clean, m.start())
        if not _UNORDERED.search(win):
            continue  # 上游看不出无序容器 ⇒ 可能是 Vec/切片 ⇒ 稳定排序有意义
        # 若比较器里有显式 tie-break ⇒ 不报
        # 取本调用的参数括号范围
        i = clean.index("(", m.end() - 1)
        depth, j = 0, i
        while j < len(clean) and j - i < 2000:
            if clean[j] == "(":
                depth += 1
            elif clean[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        args = clean[i:j + 1]
        if _TIEBREAK.search(args):
            continue
        line = clean[m.start():clean.find("\n", m.start())].strip()
        hits.append({
            "kind": "sort",
            "op": m.group("op"),
            "path": path,
            "line": _line_of(clean, m.start()),
            "unordered_upstream": True,
            "snippet": line[:96],
        })
    return hits


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


def scan(roots: list[str], kinds: list[str] | None) -> dict:
    hits: list[dict] = []
    nfiles = 0
    for p in walk_rs(roots):
        nfiles += 1
        for h in scan_file(p):
            if kinds and h["kind"] not in kinds:
                continue
            hits.append(h)
    hits.sort(key=lambda h: (h["kind"], h["path"], h["line"]))
    return {"hits": hits, "n_files": nfiles}


def selftest() -> int:
    """自证：正例 + 证伪。判据必须用**真实代码形态**验证（R-SCAN-2）。"""
    import tempfile

    ok = True

    def fail(m: str) -> None:
        nonlocal ok
        ok = False
        print("  FAIL %s" % m)

    with tempfile.TemporaryDirectory() as td:
        cases = [
            (
                "正例：HashMap 上 max_by_key ⇒ 必须报（by_key 无 tie-break 位置）",
                "use std::collections::HashMap;\n"
                "fn f(m: &HashMap<String, usize>) -> String {\n"
                "    m.iter().max_by_key(|(_, v)| *v).map(|(k, _)| k.clone()).unwrap()\n"
                "}\n",
                {"extremum": 1},
            ),
            (
                "证伪：注释里的 max_by_key 不得计入",
                "// m.iter().max_by_key(|(_, v)| *v)\nfn g() {}\n",
                {"extremum": 0},
            ),
            (
                "证伪：字符串里的 max_by_key 不得计入",
                'const S: &str = "max_by_key(|(_, v)| *v)";\nfn h() {}\n',
                {"extremum": 0},
            ),
            (
                "正例：HashMap 上 sort_by 且无 tie-break ⇒ 报",
                "use std::collections::HashMap;\n"
                "fn k(m: &HashMap<String, u32>) -> Vec<(String, u32)> {\n"
                "    let mut v: Vec<_> = m.iter().collect();\n"
                "    v.sort_by(|a, b| a.1.cmp(&b.1));\n    v\n}\n",
                {"sort": 1},
            ),
            (
                "证伪：有显式 tie-break（then_with）⇒ 不报",
                "use std::collections::HashMap;\n"
                "fn l(m: &HashMap<String, u32>) -> Vec<(String, u32)> {\n"
                "    let mut v: Vec<_> = m.iter().collect();\n"
                "    v.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)));\n    v\n}\n",
                {"sort": 0},
            ),
            (
                "证伪：Vec 上的 sort_by（上游无无序容器）⇒ 不报",
                "fn o(mut v: Vec<(String, u32)>) -> Vec<(String, u32)> {\n"
                "    v.sort_by(|a, b| a.1.cmp(&b.1));\n    v\n}\n",
                {"sort": 0},
            ),
            (
                "证伪：max_by（比较器形式，可自带 tie-break）⇒ 不报",
                "use std::collections::HashMap;\n"
                "fn p(m: &HashMap<String, u32>) -> String {\n"
                "    m.iter().max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))\n"
                "        .map(|(k, _)| k.clone()).unwrap()\n}\n",
                {"extremum": 0, "sort": 0},
            ),
        ]
        for title, body, want in cases:
            p = os.path.join(td, "a.rs")
            with open(p, "w", encoding="utf-8") as fh:
                fh.write(body)
            got = {}
            for h in scan_file(p):
                got[h["kind"]] = got.get(h["kind"], 0) + 1
            problems = []
            for k, n in want.items():
                if got.get(k, 0) != n:
                    problems.append("%s 期望 %d 实得 %d" % (k, n, got.get(k, 0)))
            # 行号必须指向真实调用
            for h in scan_file(p):
                lines = body.split("\n")
                if h["line"] - 1 >= len(lines) or (
                    "sort_by" not in lines[h["line"] - 1]
                    and "by_key" not in lines[h["line"] - 1]
                ):
                    problems.append("行号错位 %s@%d" % (h["op"], h["line"]))
                    break
            if problems:
                fail("%s -> %s" % (title, "; ".join(problems)))
            else:
                print("  PASS %s" % title)

    print("[nt_nondet] selftest %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", nargs="?", default="scan")
    ap.add_argument("--root", action="append", default=None)
    ap.add_argument("--kind", default=None, help="逗号分隔：extremum,sort")
    ap.add_argument("--limit", type=int, default=40)
    ap.add_argument("--json", dest="as_json", action="store_true")
    args = ap.parse_args()

    if args.cmd == "selftest":
        return selftest()

    roots = args.root or DEFAULT_ROOTS
    kinds = [k.strip() for k in args.kind.split(",")] if args.kind else None
    rep = scan(roots, kinds)
    if args.as_json:
        print(json.dumps(rep, ensure_ascii=False, indent=2))
        return 0

    by_kind: dict[str, int] = {}
    for h in rep["hits"]:
        by_kind[h["kind"]] = by_kind.get(h["kind"], 0) + 1
    print("[nt_nondet] 扫描 %d 个 .rs" % rep["n_files"])
    print("[nt_nondet] 候选 %d（%s）" % (len(rep["hits"]), by_kind))
    for h in rep["hits"][: args.limit]:
        tag = "无序上游" if h["unordered_upstream"] else "上游未知"
        print("  %-9s %-16s %s:%d" % (h["kind"], h["op"], os.path.relpath(h["path"]), h["line"]))
        print("            %s   [%s]" % (h["snippet"], tag))
    if len(rep["hits"]) > args.limit:
        print("  … 另有 %d 条（--limit 调整）" % (len(rep["hits"]) - args.limit))
    print(
        "\n⛔ 本工具只报候选：「并列是否真的有害」是**语义决策**——"
        "有时两个候选确实等价、任意一个都对。\n"
        "  · `extremum`（max_by_key/min_by_key）**签名里就没有 tie-break 位置** ⇒ 并列必然未定；\n"
        "  · `sort` 类仅在**上游是无序容器**时报（Vec 上的稳定排序是有意义的）；\n"
        "  · 命中后必须读那一行本身，确认并列是否**真的**会导致错误输出（R-SCAN-1b）。\n"
        "本工具**不改任何代码**。"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())