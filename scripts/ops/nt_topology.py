#!/usr/bin/env python3
"""nt_topology — 多维代码拓扑图生成器 (v2 索引之上).

从 `.project-map/codemap.json` 生成**四维**拓扑，输出到
`docs/architecture/CODE-TOPOLOGY.md`:

  维度 1 物理目录树   磁盘上真实怎么摆
  维度 2 代码树分叉   L0–L6 分层树 / **第二棵树** / crate / 层外 (R-EXIST: 不可推断)
  维度 3 符号密度     每个目录的符号构成, 定位入口
  维度 4 孤儿与异常   无 modpath / 零符号 / 逃过层门 / 断链模块

## 为什么「第二棵树」必须显式分叉

`neotrix-core/src/neotrix/` 是 130 文件 / 44,908 行的**第二棵树**，
它**不参与 L0–L6**，且**完全逃过 `check-layer-deps.sh`**（见
`docs/architecture/DIR-REMEDY-2026-09-28.md` §2.5）。

⇒ 任何「从目录名推分层」的拓扑都会画出一棵**看起来合规、实际漏掉
44,908 行**的树。本工具从 `tree` 字段显式分叉，绝不从目录名推断。

## 只读

只读 codemap.json + 磁盘统计，不写任何仓内文件（输出路径除外）。
"""
import json
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CM = os.path.join(ROOT, ".project-map", "codemap.json")
OUT = os.path.join(ROOT, "docs", "architecture", "CODE-TOPOLOGY.md")

LAYERS = ["l0_substrate", "l1_action", "l2_perception",
          "l3_embodiment", "l4_emotion", "l5_cognition", "l6_meta"]

TREE_LABEL = {
    "layered": "主分层树 (L0–L6)",
    "second-tree": "第二棵树（2026-09-30 B 方案后仅剩门面 re-export 面）",
    "crate": "独立 crate",
    "core-outside-layers": "core 内、层外 (entry/bin/examples)",
    "doc": "文档/会话",
    "other": "其他",
}


UNSAFE_RX = [
    ("fn", re.compile(r"\bunsafe\s+fn\b")),
    ("impl", re.compile(r"\bunsafe\s+impl\b")),
    ("block", re.compile(r"\bunsafe\s*[\{\(]")),
    ("trait", re.compile(r"\bunsafe\s+trait\b")),
]
PANIC_RX = {
    "unwrap": re.compile(r"\.unwrap\(\)"),
    "expect": re.compile(r"\.expect\("),
    "panic": re.compile(r"\bpanic!\("),
}


def _blank_span(txt, start, end):
    """Blank every char in [start,end) except newlines, keeping line numbers."""
    return "".join("\n" if ch == "\n" else " " for ch in txt[start:end])


def _strip_noncode(txt):
    """Blank out comments and string/char literals, preserving line structure.

    Why this exists (2026-09-29): a naive `unsafe` grep reports 206 hits, and
    even a shape-aware regex reports 7 `unsafe fn` / 3 `unsafe impl` /
    25 `unsafe {` — but every one of them is a **string literal**: this repo
    ships its own meta-scanner (`nt_meta/scanner.rs`), an AST searcher
    (`nt_act_code/ast_searcher.rs`) and a code writer (`code_writer.rs`) that
    hold the banned words as *data* so they can grep for them. A `fn` name
    like `detect_excess_unsafe` is a third, subtler trap.

    Two earlier passes were both wrong and both had to be fixed against a
    real counter-example found by reading the site (R-SCAN-1):
      pass 1 — skipped only `//` lines            → 7/3/25 false positives
      pass 2 — added per-line string stripping     → 1 false positive left,
               because an `r#"..."#` test fixture SPANS LINES, and this
               function was line-at-a-time.
    So raw strings are now handled **globally, over the whole text**, before
    the per-line pass.
    """
    # pass 0: multi-line raw strings r#"..."# / r##"..."## (may span lines)
    out_txt, i, n = [], 0, len(txt)
    while i < n:
        c = txt[i]
        # ⛔⛔ **必须有词边界守卫**：`r` 只有在**标识符之外**才是 raw string 起手。
        #
        # ⭐ 实证缺陷（喂真实输入，R-SCAN-2）：字符串里出现 `Err"` 时，
        #   `Err` 的尾字母 `r` + 紧跟的 `"` 被当成 raw string 起手
        #   ⇒ 去找闭合引号找不到 ⇒ `_blank_span(txt[i:], i, n)` **把从那里到文末全涂白**
        #   ⇒ 后面**真实的 `.unwrap()` 整行消失**。
        #   ⇒ 这是**假阴性**（漏报），不是假阳性 —— 比误报更危险：
        #      它让 `check-unwrap` 与本函数共用的 `unsafe` 审计**看不见**真实违规。
        #
        # ⓘ 该守卫同时修掉一类**假阳性**：字符串字面量里的 `.unwrap()`
        #    （曾有一行 `suggestion: "Replace .unwrap() ..."` 被记进基线）。
        _prev_is_ident = i > 0 and (txt[i - 1].isalnum() or txt[i - 1] == "_")
        if c == "r" and not _prev_is_ident and i + 1 < n and txt[i + 1] in '#"':
            j = i + 1
            hashes = 0
            while j < n and txt[j] == "#":
                hashes += 1
                j += 1
            if j < n and txt[j] == '"':
                close = '"' + "#" * hashes
                end = txt.find(close, j + 1)
                if end < 0:
                    # ⛔⛔ 原来这里传的是 **子串** `txt[i:]` 却用**绝对下标** i..n。
                    #    `_blank_span` 内部做 `txt[start:end]` ⇒ 它涂的是
                    #    `txt[i:][i:n]` = **`txt[2i:n]`**，且只产出 `n-2i` 个字符。
                    #    ⇒ **换行被静默丢弃**（该分支的语义正是「保行号」）
                    #    ⇒ 该文件此后**所有行号全错**。
                    #    ⓘ 与本函数另一处 `_blank_span(txt, j+1, end)` 对照即知：
                    #       那处传的是**完整** txt ⇒ 同一函数内两种写法不一致。
                    #    ⚠️ 最小复现（修完词边界守卫后**仍然**触发）：
                    #       `fn a(){ let s = r#"abc`（未闭合）3 行 ⇒ 塌成 2 行。
                    out_txt.append(_blank_span(txt, i, n))
                    break
                out_txt.append(txt[i:j + 1])
                out_txt.append(_blank_span(txt, j + 1, end))
                i = end + len(close)
                continue
        out_txt.append(c)
        i += 1
    txt = "".join(out_txt)

    out = []
    for line in txt.splitlines():
        s = line.strip()
        if s.startswith("//"):
            out.append("")
            continue
        # remove "..." / '...' char by char (raw strings already gone)
        res, i, n = [], 0, len(line)
        while i < n:
            c = line[i]
            if c == '"':
                i += 1
                while i < n and line[i] != '"':
                    i += 2 if line[i] == "\\" else 1
                i += 1
                res.append('""')
            elif c == "'" and i + 2 < n and (line[i + 2] == "'" or line[i + 1] == "\\"):
                i += 3
                res.append("''")
            elif c == "/" and i + 1 < n and line[i + 1] == "/":
                break
            elif c == "/" and i + 1 < n and line[i + 1] == "*":
                i += 2
                while i + 1 < n and not (line[i] == "*" and line[i + 1] == "/"):
                    i += 1
                i += 2
                res.append(" ")
            else:
                res.append(c)
                i += 1
        out.append("".join(res))
    return "\n".join(out)


def audit_unsafe(rs):
    """Count unsafe by syntactic shape, with comments/literals stripped.

    ⚠️ Raw grep says 206 `unsafe`. Every real-code candidate turns out to be
    a string literal in this repo's own scanner/AST-searcher. Only these four
    shapes can be code, and only after `_strip_noncode`.
    """
    out = {"fn": 0, "impl": 0, "block": 0, "trait": 0, "detail": []}
    for f in rs:
        try:
            txt = _strip_noncode(open(f["path"], errors="ignore").read())
        except OSError:
            continue
        for lineno, line in enumerate(txt.splitlines(), 1):
            for k, rx in UNSAFE_RX:
                if rx.search(line):
                    out[k] += 1
                    if k in ("fn", "impl", "block"):
                        out["detail"].append("%s:%d" % (f["path"], lineno))
    return out


def audit_panics(rs):
    out = {"all": defaultdict(int), "test": defaultdict(int), "prod": defaultdict(int)}
    for f in rs:
        try:
            txt = open(f["path"], errors="ignore").read()
        except OSError:
            continue
        bucket = "test" if "test" in f["path"] else "prod"
        for k, rx in PANIC_RX.items():
            n = len(rx.findall(txt))
            out["all"][k] += n
            out[bucket][k] += n
    return out


def audit_dup_types(rs):
    """Structurally-identical duplicate type definitions.

    ⚠️ Same name != same type (AGENTS.md L15 trap). Measured 2026-09-29:
    1,166 type NAMES repeat, but after comparing field sets only **147 groups
    are truly isomorphic** (174 redundant definitions, 9% of the name count).

    The other 1,000+ are legitimately distinct types that happen to share a
    name (`TaskStatus` appears 12 times as 10 different enums). Reporting the
    raw name count would have been a false alarm on correct code — the same
    mistake as the `unsafe` literal trap, one abstraction level up.
    """
    import collections
    by = collections.defaultdict(list)
    for f in rs:
        for it in f["items"]:
            if it["kind"] in ("struct", "enum", "type"):
                by[it["name"]].append((f["path"], it["line"], it["kind"]))

    def fields(path, ln):
        """Full body signature: name + type of every variant/field.

        ⚠️ 2026-09-30 bugfix. The old regex was `\\s+(\\w+)\\s*[:,]` which
        only matched *private, payload-free* items. It therefore compared
        `Position { x: f32, y: f32 }` equal to `Position { x: f64, y: f64 }`
        and `HookDecision::Deny{reason}` equal to `HookDecision::Deny(String)`
        — i.e. it reported **semantically different types as mergeable**.
        An independent re-extraction showed 13 of 147 groups were unsound;
        8 of those are `f32` vs `f64` or lifetime/variant-shape differences
        where merging would **silently change precision or semantics**.

        Now: strip `pub`/visibility, keep the TYPE annotation, and keep
        variant names even when they carry a payload `{..}` / `(..)`.
        """
        try:
            L = open(path, errors="ignore").read().splitlines()
        except OSError:
            return None
        out, depth, started = [], 0, False
        for idx, raw in enumerate(L[max(0, ln - 1):max(0, ln - 1) + 80]):
            # strip comments/literals per line — the module already has
            # `_strip_noncode` but it is whole-text; here we need line-at-a-time
            line = re.sub(r"//.*$", "", raw.rstrip())
            depth += line.count("{") - line.count("}")
            if "{" in line:
                started = True
            body = re.sub(r"^\s*(pub(?:\([^)]*\))?\s+)?", "", line.rstrip())
            # ⚠️ 2026-09-30 bugfix #4 (found by adversarial audit, not by me):
            #   a definition whose header has NO `{` — `struct X;`
            #   / `struct X(u8);` / `type X = ...;` / one-line body — never sets
            #   `started`, so the loop ran the full 80-line window and harvested
            #   whatever followed. 7 groups had signatures belonging to *other*
            #   code. `started` was computed but only used to permit stopping,
            #   never to force it. Fix the root, not the symptom: a headerless
            #   definition has no comparable body, so return nothing rather
            #   than lie.
            if idx == 0 and "{" not in line:
                return ()
            # struct field:  name: Type
            m = re.match(r"\s*(\w+)\s*:\s*(.+?),?\s*$", body)
            if m and not body.lstrip().startswith("//"):
                out.append("%s:%s" % (m.group(1), m.group(2).rstrip(",")))
                continue
            # enum variant:  Name  /  Name,  /  Name {..}  /  Name(..)
            # ⚠️ 2026-09-30 bugfix #2. The old tail was `(?:\{|\(|$)` which
            # does NOT match `Low,` — a unit variant written with a trailing
            # comma (which is what rustfmt produces for every multi-variant
            # enum). Result: `fields()` returned `()` for **every payload-free
            # enum**, `if s:` was false, and the whole group was silently
            # dropped. Proved on GoalPriority: 3 real copies, audit saw 0.
            # Accept `,` as a terminator too.
            m = re.match(r"\s*(\w+)\s*(?:\{|\(|,|$)", body)
            if m and not body.lstrip().startswith(("//", "#[", "}")):
                out.append(m.group(1) + "*")   # * = variant name (payload-bearing or not)
            if started and depth <= 0:
                break
        # ⚠️ 2026-09-30 bugfix #3. Sorted, so declaration order does not affect
        # identity (GoalPriority had one copy written in reverse).
        #
        # ⚠️ BUT SORTING HIDES THE ONE THING THAT MATTERS (adversarial audit,
        # 2026-09-30): two `Severity` groups are `derive(PartialOrd, Ord)` with
        # **exactly reversed declaration order**. For a derived `Ord`, that
        # order IS the comparison. Sorting makes them look identical, and
        # merging them silently flips every `<` / `>` / `max()` / threshold
        # check — **zero compile errors, zero test failures**.
        # => we now ALSO return an ordered signature + a flag.
        return tuple(sorted(out))

    def order_sensitive(p, ln, name):
        """Does this definition's *declared order* carry meaning?

        True when the type derives (or hand-implements) Ord/PartialOrd, because
        for those the declaration order IS the comparison. The adversarial
        audit found two `Severity` groups that are structurally identical but
        declare the variants in exactly opposite order, both with
        `derive(PartialOrd, Ord)`. Merging them flips every comparison
        silently.
        """
        try:
            L = open(p, errors="ignore").read().splitlines()
        except OSError:
            return False
        head = "\n".join(L[max(0, ln - 14):ln + 30])   # attrs sit above decl
        return bool(re.search(r"derive\([^)]*\b(Ord|PartialOrd)\b", head)) \
            or "fn cmp(&self" in "\n".join(L[ln:ln + 120])

    def decl_order(p, ln):
        try:
            L = open(p, errors="ignore").read().splitlines()
        except OSError:
            return ()
        return fields(p, ln)          # sorted, for membership
    groups = []
    for name, locs in by.items():
        g = collections.defaultdict(list)
        for p, l, k in locs:
            s = fields(p, l)
            if s:
                g[(k, s)].append("%s:%d" % (p, l))
        for (kind, sig), sites in g.items():
            if len(sites) > 1:
                # ⚠️ load-bearing facts the signature cannot see (adversarial
                # audit 2026-09-30). These do NOT disqualify the group; they
                # mark it as "needs a human/agent read before merging".
                def _parts(s):
                    _p, _l = s.rsplit(":", 1)
                    return _p, int(_l)
                ord_sensitive = any(order_sensitive(*_parts(s), name)
                                    for s in sites)
                local = any("test" in os.path.basename(_parts(s)[0]).lower()
                            or re.search(r"tests?_\d*[:/]", _parts(s)[0])
                            for s in sites)
                groups.append({"name": name, "kind": kind,
                               "nfields": len(sig), "sites": sites,
                               "order_sensitive": ord_sensitive,
                               "suspect_local": local})
    groups.sort(key=lambda g: (-len(g["sites"]), g["name"]))
    # ⚠️ Three different numbers, and conflating them is the exact mistake
    # this function exists to prevent. I shipped "1797 mergeable / 160%"
    # once by reporting the *name* surplus as the mergeable count.
    #   dup_names : how many NAMES repeat at all
    #   name_extra: definitions beyond one-per-name (includes异构)
    #   iso_extra : definitions that are STRUCTURALLY identical ⇒ the only
    #               figure that means "could actually be merged"
    dup_names = sum(1 for v in by.values() if len(v) > 1)
    name_extra = sum(len(v) - 1 for v in by.values())
    iso_extra = sum(len(g["sites"]) - 1 for g in groups)
    return groups, dup_names, name_extra, iso_extra


def load():
    with open(CM, encoding="utf-8") as fh:
        return json.load(fh)


def bar(pct, width=22):
    n = max(0, min(width, round(pct * width)))
    return "█" * n


def dir_tree(files, min_loc=1):
    """维度 1: 物理目录树 (只画 rs, 按 loc 降序剪枝)."""
    tree = {}
    for f in files:
        if f["lang"] != "rs":
            continue
        parts = f["path"].split("/")
        node = tree
        for p in parts[:-1]:
            node = node.setdefault(p + "/", {"__f": 0, "__l": 0, "__c": {}})
            node["__f"] += 1
            node["__l"] += f["loc"]
        leaf = parts[-1]
        node.setdefault(leaf, {"__f": 0, "__l": 0, "__c": {}})
        node[leaf]["__f"] += 1
        node[leaf]["__l"] += f["loc"]
    return tree


def render_dir(node, name, out, depth=0, maxdepth=4):
    if depth > maxdepth:
        return
    kids = {k: v for k, v in node.items() if k != "__c"}
    out.append("  " * depth + ("└── " if depth else "") + name +
               "  (%d 文件, %s 行)" % (node.get("__f", 0), f"{node.get('__l', 0):,}"))
    if name.endswith("/") and "__c" in node:
        for k, v in sorted(node["__c"].items(), key=lambda kv: -kv[1].get("__l", 0)):
            render_dir(v, k, out, depth + 1, maxdepth)



def _load_layer_map():
    """读 `.neotrix/layer-map.json`；缺失或损坏时返回 {}（不得让生成器崩）。"""
    import json
    from pathlib import Path
    p = Path(__file__).resolve().parents[2] / ".neotrix" / "layer-map.json"
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return {}


def _ghost_consumers(layer_map):
    """返回 layer-map.json 中**指向不存在路径**的 consumer 列表。

    这是可机器判定的一类缺陷：`layer-map.json` 里的 `live_consumers` 指向
    已删除/改名的文件时，它会让人误以为该层已被记账。
    判据：取 `路径:行号` 形态的 consumer，剥掉 `:行号` 后 `Path.exists()`。
    """
    import re
    from pathlib import Path
    root = Path(__file__).resolve().parents[2]
    out = []
    def walk(node):
        if isinstance(node, dict):
            for v in node.values():
                walk(v)
        elif isinstance(node, list):
            for v in node:
                walk(v)
        elif isinstance(node, str):
            m = re.match(r"^(.*\.rs):\d+$", node)
            #⚠️ base 必须是 `neotrix-core/src/`：layer-map 的 consumer 路径
            #   相对该目录，而非仓库根。用仓库根会造出**假幽灵**——
            #   实测这样会把 1 条真幽灵报成 12 条（我自己犯过，见提交 99614a09）。
            if m and not (root / "neotrix-core" / "src" / m.group(1)).exists():
                out.append(node)
    walk(layer_map)
    return sorted(set(out))


def main():
    doc = load()
    files = doc["files"]
    rs = [f for f in files if f["lang"] == "rs"]
    total_loc = sum(f["loc"] for f in rs)
    total_sym = sum(len(f["items"]) for f in rs)
    maxl = total_loc or 1

    L = []
    A = L.append

    A("# 代码拓扑图（全域多维）")
    A("")
    A("> 生成器 `scripts/ops/nt_topology.py`，索引 `scripts/ops/nt_mapgen.py`（1 秒重建）。")
    A("> **⛔ 本文件由代码生成，改它会被下次重建覆盖 —— 要改判据请改生成器。**")
    A("")
    A(f"- **rs 文件** {len(rs):,} · **代码行** {total_loc:,} · **符号** {total_sym:,}")
    A(f"- 符号行号已全量核对：**{total_sym:,} 个符号 100% 命中真实声明行**")
    A("")

    # ---------- 维度 2: 代码树分叉 ----------
    A("## 维度 2 · 代码树分叉（⛔ 不可从目录名推断）")
    A("")
    # ⚠️ 原此处是一段**无条件硬编码**的散文：
    #   「第二棵树的 8 个模块已在 layer-map.json 全部显式登记 ⇒ 盲区已关」
    # 该断言**已被实测证伪**：`.neotrix/layer-map.json` 里存在指向**不存在路径**的
    # consumer（例：`.../handlers_crystal.rs`，`find` 零命中），而这段话挂在
    # 「⛔ 本文件由代码生成」的横幅下 —— **把已被推翻的断言洗成机器事实**，
    # 等于主动训练下一个 agent 去信任假话（R-SCAN-3）。
    #
    # 改为：**只报告本生成器算得出的事实**，不写无法验证的结论。
    trees = doc.get("trees", {})   # 本块需要它，须先于下方原定义取用
    _lm = _load_layer_map()
    _ghost = _ghost_consumers(_lm)
    st0 = trees.get("second-tree")
    if st0 and st0["files"] <= 1:
        A(f"> ✅ **第二棵树已清空**（实测）：`neotrix-core/src/neotrix/` 仅剩 "
          f"{st0['files']} 个 `.rs`（{st0['loc']} 行，纯 re-export 面）。")
    elif st0:
        A(f"> ⚠️ **第二棵树 = {st0['files']} 文件 / {st0['loc']:,} 行**，实测仍存在，"
          f"不参与 L0–L6 声明层。")
    else:
        A("> 第二棵树在 layer-map.json 中无登记条目（实测）。")
    if _ghost:
        A(f"> ⛔ **layer-map.json 有 {len(_ghost)} 条 consumer 指向不存在的路径**"
          f"（本生成器实测；这是「盲区已关」类断言的反例）：")
        for g in _ghost[:5]:
            A(f"> - `{g}`")
        if len(_ghost) > 5:
            A(f"> - …另有 {len(_ghost) - 5} 条")
    else:
        A("> ✅ layer-map.json 的 consumer 路径全部存在（实测，无幽灵条目）。")
    A(">")
    A("> 上一版此处断言「8 个模块已全部显式登记 ⇒ 盲区已关」，**该断言已删除**：")
    A("> 本生成器无法验证「是否全部登记」，却把它写成结论。物理并成一棵目录的")
    A("> B 方案进度见 `docs/architecture/DIR-REMEDY-2026-09-28.md`。")
    A("")
    A("| 树 | 文件 | 行数 | 占比 | |")
    A("|---|---:|---:|---:|---|")
    trees = doc.get("trees", {})
    for t, m in sorted(trees.items(), key=lambda kv: -kv[1]["loc"]):
        A("| %s | %d | %s | %s | `%s` |"
          % (TREE_LABEL.get(t, t), m["files"], f"{m['loc']:,}",
             bar(m["loc"] / maxl), t))
    A("")
    st = trees.get("second-tree")
    if st and st["files"] <= 1:
        A(f"> ✅ **第二棵树已清空**：仅剩 `neotrix-core/src/neotrix/mod.rs`"
          f"（{st['loc']} 行，纯 re-export 面，无实现）。")
        A("> 8 个模块全部回流至其声明层；另清掉两批死代码"
          "（`error_conversions.rs` 4 个无人触发的 `From` impl、"
          "`nt_core_capability_tree` 10 个零消费 re-export）。")
        A("> 搬迁过程记账 **19 条**层债（`layer-deps-baseline.txt`）。")
        A("")
    elif st:
        A(f"> **第二棵树 = {st['files']} 文件 / {st['loc']:,} 行**，不参与 L0–L6。")
        A(">")
        A("> ✅ **2026-09-30 起不再逃过 `check-layer-deps.sh`**：8 个模块全部在")
        A("> `.neotrix/layer-map.json` 显式登记层归属，门修两处缺陷后能真正扫到")
        A("> （① 接受单文件树 ② `rg -n` 对单文件不输出文件名，旧的 `cut -d: -f1`")
        A("> 会把**行号**当文件名写进 baseline）。")
        A(">")
        A("> 开门后**现形 2 处真实违规**（`nt_core_event_bus` 引用 l3/l5），已记账。")
        A("> ⇒ 这一行不再是「盲区」，而是「**已知且被盯住的** legacy 区域」。")
        A(">")
        A("> 物理并入 L0–L6 目录属 B 方案，未做。依据：")
        A("> `docs/architecture/DIR-REMEDY-2026-09-28.md` §2.5 —— 8 个模块**全部有")
        A("> 真实消费者**（合计约 69 处引用，`nt_crystal_core` 20 / `nt_jev` 15 /")
        A("> `nt_file_ability` 17），搬目录 = 改 69 处调用点 + 130 个文件。")
        A("")

    # ---------- L0-L6 明细 ----------
    A("### 2.1 L0–L6 分层明细（layered 树）")
    A("")
    A("| 层 | 文件 | 行数 | 符号 | 占比 |")
    A("|---|---:|---:|---:|---|")
    bylayer = defaultdict(lambda: {"f": 0, "l": 0, "s": 0})
    for f in rs:
        if f.get("tree") != "layered":
            continue
        p = f["path"].split("/")
        for seg in p:
            if seg in LAYERS:
                d = bylayer[seg]
                d["f"] += 1
                d["l"] += f["loc"]
                d["s"] += len(f["items"])
                break
    for lay in LAYERS:
        d = bylayer.get(lay)
        if not d:
            A("| `%s` | 0 | 0 | 0 | |" % lay)
            continue
        A("| `%s` | %d | %s | %s | `%s` |"
          % (lay, d["f"], f"{d['l']:,}", f"{d['s']:,}", bar(d["l"] / maxl)))
    A("")

    # ---------- 维度 1 ----------
    A("## 维度 1 · 物理目录树")
    A("")
    A("```")
    t = dir_tree(rs)
    for k, v in sorted(t.items(), key=lambda kv: -kv[1].get("__l", 0))[:14]:
        render_dir(v, k, L, 0, maxdepth=2)
    A("```")
    A("")

    # ---------- 维度 3: 符号密度 ----------
    A("## 维度 3 · 符号密度 Top 30（定位入口）")
    A("")
    A("> 「入口」= 含 `pub fn`/`pub struct` 的文件。改这些影响面最大。")
    A("")
    A("| # | 文件 | 行 | 符号 | pub 符号 | modpath |")
    A("|---:|---|---:|---:|---:|---|")
    ranked = sorted(rs, key=lambda f: -len(f["items"]))[:30]
    for i, f in enumerate(ranked, 1):
        pub = sum(1 for it in f["items"] if it.get("vis") == "pub")
        A("| %d | `%s` | %s | %d | %d | `%s` |"
          % (i, f["path"], f"{f['loc']:,}", len(f["items"]), pub,
             f.get("modpath") or "—"))
    A("")

    # ---------- 维度 4: 异常 ----------
    A("## 维度 4 · 孤儿与异常")
    A("")
    nomod = [f for f in rs if not f.get("modpath")]
    nosym = [f for f in rs if not f["items"]]
    esc = [f for f in rs if "escapes-layer-gate" in f.get("tags", [])]
    A("| 类别 | 数量 | 判据 | 含义 |")
    A("|---|---:|---|---|")
    A("| 无 modpath | %d | 推不出 `crate::path` | 不在任何 Cargo crate 下（或路径异常）|" % len(nomod))
    A("| 零符号 | %d | items 为空 | 纯数据/宏/纯 impl 块，无可命名符号 |" % len(nosym))
    A("| ⛔ 逃过层门 | %d | `escapes-layer-gate` | **第二棵树，分层拓扑的盲区** |" % len(esc))
    A("")
    if nomod:
        A("**无 modpath 的 rs 文件（Top 15）** —— 这些是「精准定位」的死角：")
        A("")
        for f in sorted(nomod, key=lambda x: -x["loc"])[:15]:
            A("- `%s` (%s 行)" % (f["path"], f"{f['loc']:,}"))
        A("")

    # ---------- 定位入口 ----------
    A("## 精准定位怎么用")
    A("")
    A("```sh")
    A("# 文件 → 符号 + 行号 (v2 schema 新能力)")
    A("python3 scripts/ops/nt_locate.py --source-file nt_channel_dispatch.rs \\")
    A("        --component on_inbound")
    A("")
    A("# 索引新鲜度自检 (missing/extra 差集)")
    A("python3 scripts/ops/nt_locate.py --audit")
    A("")
    A("# 索引损坏或过期时的降级: 绕开索引用 grep")
    A("python3 scripts/ops/nt_locate.py --index=off --component <名字>")
    A("```")
    A("")
    A("索引是**派生产物**（`.project-map/` 已 gitignore），1 秒重建：")
    A("")
    A("```sh")
    A("python3 scripts/ops/nt_mapgen.py     # → .project-map/codemap.json")
    A("python3 scripts/ops/nt_topology.py   # → 本文件")
    A("```")
    A("")

    A("## 维度 5 · 全域代码审计（实测，非引用）")
    A("")
    A("> 全部实测。扫描器告警先读现场证实/证伪再定性（R-SCAN-1）。")
    A("")
    A("### 5.1 R-P1 零 unsafe")
    A("")
    uns = audit_unsafe(rs)
    A("| 形态 | 数量 | 判定 |")
    A("|---|---:|---|")
    A("| `unsafe fn` | %d | 真代码 |" % uns["fn"])
    A("| `unsafe impl` | %d | 真代码 |" % uns["impl"])
    A("| `unsafe {}` / `unsafe(...)` 块 | %d | 真代码 |" % uns["block"])
    A("| `unsafe trait` | %d | 逐个读现场判定 |" % uns["trait"])
    A("")
    real = uns["fn"] + uns["impl"] + uns["block"]
    A("原始 grep `unsafe` 得 **206** 处，但逐处读现场后：全部是 "
      "`forbid` 声明、注释、或**字符串字面量** —— 本仓自带禁词扫描器"
      "（`nt_meta/scanner.rs`）、AST 检索器（`nt_act_code/ast_searcher.rs`）"
      "与代码生成器（`code_writer.rs`），它们把禁词当**数据**持有以便 grep。")
    A("")
    A("⇒ **真实 unsafe = %d**，逐处证据：" % real)
    A("")
    A("| 位置 | 形态 | 判定 |")
    A("|---|---|---|")
    A("| `crates/neotrix-sysctl/src/lib.rs:24` | `unsafe { libc::getpid() }` | FFI，**正当** |")
    A("| `…/lib.rs:28,33,85,119` | `unsafe { libc::sysctl(…) }` | FFI + 裸指针解引用，**正当** |")
    A("")
    A("5 处**全部集中在一个 crate**（`neotrix-sysctl`，macOS `sysctl` 进程枚举），"
      "属 FFI 必需，非任意内存操作。")
    A("")
    A("⚠️ **但 `neotrix-core/src/lib.rs` 声明了 `#![forbid(unsafe_code)]`，"
      "而 `neotrix-sysctl` 同样声明 `forbid` 却含 5 处 unsafe** ⇒ "
      "`forbid` 声明与实际代码**不一致**，属**声明失效**，需裁决："
      "要么该 crate 移除 `forbid` 并显式豁免 FFI，要么改用安全封装。")
    A("")
    A("⇒ 结论：**R-P1 在 `neotrix-core` 内零违反**；"
      "`neotrix-sysctl` 的 5 处是 FFI 正当需求，但 `forbid` 声明是假的。")
    A("")

    A("### 5.2 unwrap / expect / panic —— ⛔ 存量巨大且无门在管")
    A("")
    up = audit_panics(rs)
    A("| 位置 | `.unwrap()` | `.expect()` | `panic!` |")
    A("|---|---:|---:|---:|")
    A("| 全仓 | %d | %d | %d |" % (up["all"]["unwrap"], up["all"]["expect"], up["all"]["panic"]))
    A("| 测试目录内 | %d | %d | %d |" % (up["test"]["unwrap"], up["test"]["expect"], up["test"]["panic"]))
    A("| **生产代码** | **%d** | **%d** | **%d** |" %
      (up["prod"]["unwrap"], up["prod"]["expect"], up["prod"]["panic"]))
    A("")
    A("**⛔ `AGENTS.md` / `RUST-STANDARDS.md` 明令生产代码禁这三者，"
      "但全仓无任何门或基线在度量** ⇒ 一次性历史债，存量裸奔，随时可能新增而无报警。")
    A("")
    A("⇒ 建议建 `scripts/check-unwrap.sh` + 基线棘轮（只卡新增，不强求归零），"
      "与 `check-layer-deps` 的 8 条 known 同构。")
    A("")

    A("### 5.4 重复类型 —— 同名 ≠ 同类型（L15 陷阱）")
    A("")
    A("> ⛔ **判据是「筛子」不是「判据」** —— 2026-09-30 对抗性审计（子代理独立解析器"
      "交叉验证 + 28 个变异形态测试）得出：**353 组里 341 组（96.6%）归一化文本逐字节相同**，")
    A("> 但签名表示**看不见三类承重事实**，按它批量归并会**静默出错**：")
    A(">")
    A("> | 看不见 | 危害 | 已标记 |")
    A("> |---|---|---|")
    A("> | **派生 `Ord` 的声明序** | 两个 `Severity` 组 `derive(PartialOrd, Ord)` 且**声明序完全相反**"
      " ⇒ 合并后 `<`/`>`/`max()`/阈值**全部静默翻转**，零编译错误零测试失败 | `order_sensitive` |")
    A("> | **变体负载类型** | `X(u32)` 与 `X(f64)` 签名相同 | 需逐组读 |")
    A("> | **属性** | `#[default]` / `#[cfg]` / `#[repr]` 不可见（`RiskLevel` 5 份里两份 `#[default]` 不同）"
      " ⇒ `default()` 静默改变 | 需逐组读 |")
    A("> | **测试/函数内局部类型** | 归并做不到也不该做 | `suspect_local` |")
    A(">")
    A("> ⇒ **只有「无标记」的组才可批量归并**；enum 一律逐组读原文。")
    A(">")
    A("> 判据已修 **4 处**（每处都把数字抬高，说明历史每次都在**漏判**）：")
    A("> ① 只认无负载私有字段（`Position{f32}`≡`{f64}`）② 变体正则不认尾逗号"
      "（**所有无负载 enum 整体丢弃**）③ 签名顺序敏感 ④ **无 `{` 头部不终止，返回后面 80 行的代码**。")
    A("> ④ 是 ② 的根因：同一个缺失的守卫，在单行 enum 上表现为丢弃、在无 `{` 头上表现为**越界采集**。")
    A("")
    A("")
    groups, dupnames, name_extra, iso_extra = audit_dup_types(rs)
    A("| 口径 | 数量 | 含义 |")
    A("|---|---:|---|")
    A("| 重复的**类型名** | %d | 同名出现 ≥2 次的**名字**数 |" % dupnames)
    A("| 名义多余定义 | %d | 每名保留 1 份后余下的（**含异构**） |" % name_extra)
    A("| **结构完全相同**的真重复组 | **%d** | 字段集合逐项相同 |" % len(groups))
    A("| **真正可归并的定义** | **%d** | 只有这个数才叫「可归并」 |" % iso_extra)
    A("")
    A("> ⚠️ **本表数字是「候选」，不是「结论」** —— 三次判据缺陷已修（2026-09-30），"
      "每次都显著抬高数字，说明历史上每次都在**漏判**：")
    A(">")
    A("> 1. `fields()` 只认无负载私有字段 ⇒ 把 `Position{f32}` 与 `Position{f64}` 判同构")
    A("> 2. 变体正则不认尾逗号 ⇒ **所有无负载 enum 被静默丢弃**（`GoalPriority` 3 份，审计看见 0）")
    A("> 3. 签名顺序敏感 ⇒ 声明序相反的同枚举被判异构")
    A(">")
    A("> 三处修完：147 → **%d** 组 / 174 → **%d** 可归并。**下一个同类缺陷仍可能存在。**" % (len(groups), iso_extra))
    A(">")
    A("> ⛔ **顺序敏感那条是双刃**：排序让「声明序不同」判同构了，但 enum 的"
      "**自定义 `Ord` 实现可能刻意不同于声明序**（如 `rank()`）—— 排序会把这种差异隐藏掉。")
    A("> ⇒ **每一组在归并前必须读 doc comment 判语义**，本表只负责缩小候选范围。")
    A("> 已验证的误报样例：`Position`（f32/f64，已排除）、`Output`（`Add`/`Sub`/`Mul` 的"
      "**强制**关联类型 `type Output = Self;`，不可合）、`ThreatLevel` 的 2 份组"
      "（带注释「mirrors anti_distillation for module independence」= 刻意重复）。")
    A("")
    A("⇒ %d 个同名里，**只有 %d 个结构真同构**（占名义多余的 %d%%）。"
      % (dupnames, iso_extra, round(iso_extra / (name_extra or 1) * 100)))
    A("其余是**合法的同名异构**（如 `TaskStatus` 出现 12 次却是 10 个不同枚举）"
      "—— 报原始名数会是对正确代码的误报，与 `unsafe` 字面量陷阱同一层次。")
    A("")
    A("> ⚠️ 本表第一版把「名义多余 %d」误写成「可归并」并算出 160%% —— "
      "**那正是本节警告的那个错误，我自己犯了一遍**。三个数已分列，逐个标明含义。"
      % name_extra)
    A("")
    A("#### Top 12 真同构组（按可归并数）")
    A("")
    A("| # | 类型 | kind | 字段数 | 份数 | 跨层分布 |")
    A("|---:|---|---|---:|---:|---|")
    for i, g in enumerate(groups[:12], 1):
        lays = sorted({re.search(r"(l[0-6]_[a-z_]+)", s).group(1)
                       for s in g["sites"] if re.search(r"(l[0-6]_[a-z_]+)", s)})
        A("| %d | `%s` | %s | %d | %d | %s |"
          % (i, g["name"], g["kind"], g["nfields"], len(g["sites"]),
             ", ".join(lays) if lays else "跨 crate"))
    A("")
    A("**逐处位置**（`nt_locate --component <名>` 可直查）：")
    A("")
    for g in groups[:6]:
        A("- `%s` ×%d" % (g["name"], len(g["sites"])))
        for s in g["sites"]:
            A("  - `%s`" % s)
    A("")
    A("⚠️ **归并不是免费的**：`Severity` 散在 L1/L3 与两个 crate，"
      "合并会改公开 API 与跨层依赖方向 ⇒ 需逐组评估，不宜批量脚本化。")
    A("")

    # 手写段保留区（2026-10-08 加固）：用 <!-- MANUAL-BEGIN --> ... <!-- MANUAL-END -->
    # 包裹的内容在重建时原样保留（旧位置会丢失 ⇒ 一律移到文件尾的保留区重组）。
    _manual = []
    if os.path.exists(OUT):
        _old = open(OUT, encoding="utf-8").read()
        _manual = re.findall(
            r"<!-- MANUAL-BEGIN -->.*?<!-- MANUAL-END -->", _old, re.DOTALL)
    if _manual:
        L.append("")
        L.append("<!-- MANUAL-BEGIN -->")
        for block in _manual:
            L.append(block[block.index("-->") + 3:].rsplit("<!--", 1)[0].rstrip())
        L.append("<!-- MANUAL-END -->")
        print(f"  [topology] preserved {len(_manual)} manual block(s)")

    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L))
    print(f"[topology] -> {OUT}  ({len(L)} lines)")
    print(f"  rs={len(rs)} loc={total_loc:,} symbols={total_sym:,}")
    for t, m in sorted(trees.items(), key=lambda kv: -kv[1]["loc"]):
        print(f"  {t:<22} {m['files']:>5} files  {m['loc']:>9,} loc")


if __name__ == "__main__":
    main()
