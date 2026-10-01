#!/usr/bin/env python3
"""同域常量重复定义扫描（constant shadowing / cross-domain redefinition）。

## 为什么需要独立工具
`nt_fn_drift.py` 只覆盖**函数**。2026-09-30 实测中手工发现的真实缺陷是
**常量**层面的：`neotrix-core/src/l2_perception/nt_world/nt_world_e8.rs`
把 `nt_core_e8` 域的 4 个 E8 数学常量（`E8_DIM` `DAYAN_NUMBER`
`OBSERVABLE_DOF` `OBSERVER_DOF`）**影子式复写**了一遍 —— 值全同、无理由注释、
其中 3 个在文件内零使用。那是**跨域错位**，函数级工具看不见。

## 判据（刻意保守，避免把正常现象报成缺陷）
只报**同名 + 同类型**的定义，并且区分两种严重度：

* `IDENTICAL`  值也相同 ⇒ 真重复（影子副本），**收敛候选**
* `DIVERGENT`  值不同     ⇒ 更危险：同名不同义，读者必然误用

刻意**不**做的事：
* 不跨文件比较「值相等但名字不同」的常量（如 `HEXAGRAM_DIM` vs `HEXAGRAM_COUNT`
  都等于 64）—— 那是命名选择问题，不是重复定义，报出来只会淹没真缺陷。
* 不解析跨文件依赖图（`nt_callgraph` 的职责），本工具只做**文本层**取证。

⚠️ 本工具**只报候选，不下结论**。每个命中都必须读那一行本身（R-SCAN-1b）：
仓库大量把 `unsafe` / `forbid` / 错误码等**当数据持有**在字符串与注释里，
`# 解释 E8_DIM 的含义` 这类注释会造成假阳性。工具已剥离注释与字符串（见 `_strip`）。

用法：
    python3 scripts/ops/nt_const_dup.py                     # 扫描默认根
    python3 scripts/ops/nt_const_dup.py --root <dir> ...    # 指定根（可多次）
    python3 scripts/ops/nt_const_dup.py --only-identical    # 只看值同的真重复
    python3 scripts/ops/nt_const_dup.py --only-divergent    # 只看同名不同义
    python3 scripts/ops/nt_const_dup.py --min-sites 2
    python3 scripts/ops/nt_const_dup.py selftest
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

DEFAULT_ROOTS = ["neotrix-core/src", "crates/neotrix-types/src"]

# `const NAME: TYPE = VALUE;` / `static NAME: TYPE = VALUE;`
# 说明：
#  · 类型必带（无类型的 const 在 Rust 里不存在），用于「同名 + 同类型」判据。
#  * `unsafe`/可见性修饰一律不吃，因为它在名字之前，用 `.*?` 宽松吃。
#  * VALUE 不跨分号，允许 `64 * 6` 这类表达式。
_DECL = re.compile(
    r"(?:(?<=\n)|^)[ \t]*"
    r"(?:pub(?:\([^)]*\))?[ \t]+)?"
    r"(?:const|static)[ \t]+"
    r"(?P<name>[A-Z][A-Z0-9_]*)[ \t]*:[ \t]*"
    r"(?P<ty>[^=;\n]+?)[ \t]*=[ \t]*"
    r"(?P<val>[^;]+);"
)

_STRING_OR_CHAR = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')


def _strip_noncode(src: str) -> str:
    """把注释与字符串替换成等长空格（保留行结构，便于按行定位）。

    必须保留长度：调用方用 `offset -> line` 定位，若长度变了行号就错。
    """
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
        elif c == "r" and i + 1 < n and src[i + 1] in '#"':
            # 原始字符串 r"..." / r#"..."# / br#"..."#
            j = i + 1
            hashes = 0
            while j < n and src[j] == "#":
                hashes += 1
                j += 1
            if j < n and src[j] == '"':
                close = '"' + "#" * hashes
                k = src.find(close, j + 1)
                k = n if k < 0 else k + len(close)
                for p in range(i, k):
                    if src[p] != "\n":
                        out[p] = " "
                i = k
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def _norm_ty(ty: str) -> str:
    return re.sub(r"\s+", "", ty)


def _norm_val(val: str) -> str:
    """归一化常量值，使「排版差异」不被误判为语义差异。

    只做**排版层**归一（空白、尾逗号），**不做语义层**归一 ——
    把 `1e-10` 与 `0.0000000001` 判同属于过度归一，会掩盖真分歧
    （`nt_diverge.py` 的 selftest 正是为此设了「过度归一化」证伪用例）。
    """
    v = val.strip().rstrip(",").strip()
    v = re.sub(r"\s+", " ", v)
    return v


def harvest_file(path: str) -> list[dict]:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            raw = fh.read()
    except OSError:
        return []
    code = _strip_noncode(raw)
    base = os.path.basename(path)
    out = []
    for m in _DECL.finditer(code):
        line = code.count("\n", 0, m.start()) + 1
        out.append(
            {
                "name": m.group("name"),
                "ty": _norm_ty(m.group("ty")),
                "val": _norm_val(m.group("val")),
                "path": path,
                "line": line,
                "file": base,
            }
        )
    return out


def walk_rs(roots: list[str]) -> list[str]:
    files = []
    for root in roots:
        if not os.path.isdir(root):
            continue
        for base, _dirs, names in os.walk(root):
            if "/target" in base.replace(os.sep, "/") or "/.worktrees" in base.replace(os.sep, "/"):
                continue
            for n in names:
                if n.endswith(".rs"):
                    files.append(os.path.join(base, n))
    return sorted(files)


def scan(roots: list[str], min_sites: int = 2) -> dict:
    decls = []
    for f in walk_rs(roots):
        decls.extend(harvest_file(f))
    by: dict[tuple[str, str], list[dict]] = {}
    for d in decls:
        by.setdefault((d["name"], d["ty"]), []).append(d)
    identical, divergent = [], []
    for (name, ty), sites in by.items():
        if len(sites) < min_sites:
            continue
        files = {s["path"] for s in sites}
        if len(files) < 2 and min_sites > 1:
            # 同一文件内多次定义也算（影子/重复声明），保留
            pass
        vals = {}
        for s in sites:
            vals.setdefault(s["val"], []).append(s)
        rec = {
            "name": name,
            "ty": ty,
            "n_sites": len(sites),
            "n_files": len(files),
            "n_values": len(vals),
            "files": sorted({os.path.relpath(s["path"]) for s in sites}),
            "sites": [
                {"file": os.path.relpath(s["path"]), "line": s["line"], "val": s["val"]}
                for s in sorted(sites, key=lambda x: (x["path"], x["line"]))
            ],
        }
        if len(vals) == 1:
            identical.append(rec)
        else:
            divergent.append(rec)
    identical.sort(key=lambda r: (-r["n_files"], -r["n_sites"], r["name"]))
    divergent.sort(key=lambda r: (-r["n_files"], r["name"]))
    return {
        "identical": identical,
        "divergent": divergent,
        "n_files_scanned": len(walk_rs(roots)),
        "n_decls": len(decls),
    }


def _fmt(rec: dict) -> str:
    tag = "IDENTICAL" if rec["n_values"] == 1 else "DIVERGENT"
    head = "  [%s] %s: %s  (%d 处 / %d 文件)" % (
        tag,
        rec["name"],
        rec["ty"],
        rec["n_sites"],
        rec["n_files"],
    )
    lines = [head]
    for s in rec["sites"]:
        lines.append("      %s:%d  = %s" % (s["file"], s["line"], s["val"]))
    return "\n".join(lines)


def selftest() -> int:
    """自证：正例 + 证伪。判据的正确性必须用**真实代码形态**验证（R-SCAN-2）。

    用临时目录写真实 .rs 文件，而不是把源码当字符串直接喂 —— 后者会绕过
    `_strip_noncode` 的行号计算，测不到「剥离后定位是否正确」这个真会出错的地方。
    （本工具第一版就栽在这里：正则吃掉了前导换行 ⇒ 行号全部差 1。）
    """
    import tempfile

    ok = True

    def fail(msg: str) -> None:
        nonlocal ok
        ok = False
        print("  FAIL %s" % msg)

    # ---------- 单文件层：能否取到声明 / 有无假阳性 / 行号是否准确 ----------
    single: list[tuple[str, str, list[str], list[str]]] = [
        (
            "正例：pub const / const / pub static 都能取到",
            """
use serde::Serialize;

/// E₈ 维数
pub const E8_DIM: usize = 248;
pub static E8_RANK: usize = 8;
const HEXAGRAM_COUNT: usize = 64;
pub(crate) const LO_SHU: usize = 15;
""",
            ["E8_DIM", "E8_RANK", "HEXAGRAM_COUNT", "LO_SHU"],
            [],
        ),
        (
            "证伪：行注释/文档注释里的声明不得计入",
            '// E8_DIM: usize = 999;\n/// OBSERVABLE_DOF = 123\nconst REAL: usize = 1;\n',
            ["REAL"],
            ["E8_DIM", "OBSERVABLE_DOF"],
        ),
        (
            "证伪：字符串/原始字符串/字节串里的声明不得计入",
            'const NAME: &str = "E8_DIM: usize = 999";\n'
            'const T: &str = r#"LO_SHU_CONSTANT = 7"#;\n'
            'const B: &[u8] = b"DAYAN_NUMBER = 3";\n',
            ["NAME", "T", "B"],
            ["E8_DIM", "LO_SHU_CONSTANT", "DAYAN_NUMBER"],
        ),
        (
            "证伪：小写名/无类型/非常量结构不得计入",
            "fn not_a_const() {}\nstruct S;\nconst lower_case: usize = 1;\n"
            "pub fn f() -> usize { 1 }\n",
            [],
            ["lower_case", "not_a_const"],
        ),
    ]
    for title, body, want, forbid in single:
        with tempfile.TemporaryDirectory() as td:
            p = os.path.join(td, "a.rs")
            with open(p, "w", encoding="utf-8") as fh:
                fh.write(body)
            got = harvest_file(p)
            names = {d["name"] for d in got}
            lines = body.split("\n")
            problems = []
            for n in want:
                if n not in names:
                    problems.append("缺 %s" % n)
            for n in forbid:
                if n in names:
                    problems.append("误报 %s" % n)
            for d in got:  # 行号必须指向**含该名字**的真实行
                if d["line"] - 1 >= len(lines) or d["name"] not in lines[d["line"] - 1]:
                    problems.append("行号错位 %s@%d (该行=%r)" % (d["name"], d["line"], lines[d["line"] - 1] if d["line"] - 1 < len(lines) else None))
                    break
            if problems:
                fail("%s -> %s" % (title, "; ".join(problems)))
            else:
                print("  PASS %s" % title)

    # ---------- 值归一：排版差异不得判成分歧 ----------
    body = "pub const L: usize = 64 ;\npub const R: usize = 64;\n"
    with tempfile.TemporaryDirectory() as td:
        p = os.path.join(td, "a.rs")
        with open(p, "w", encoding="utf-8") as fh:
            fh.write(body)
        got = harvest_file(p)
        if {d["val"] for d in got} != {"64"}:
            fail("值排版差异应归一：got %s" % [d["val"] for d in got])
        else:
            print("  PASS 值排版差异归一（尾随空白）")

    # ---------- 禁止过度归一：数值等价写法必须判 DIVERGENT ----------
    body = "pub const S: f64 = 1e-10;\npub const T: f64 = 0.0000000001;\n"
    with tempfile.TemporaryDirectory() as td:
        p = os.path.join(td, "a.rs")
        with open(p, "w", encoding="utf-8") as fh:
            fh.write(body)
            got = harvest_file(p)
        if len({d["val"] for d in got}) == 1:
            fail("过度归一：1e-10 与 0.0000000001 被判同")
        else:
            print("  PASS 禁止过度归一（1e-10 与 0.0000000001 判异）")

    # ---------- 结构层：跨文件 IDENTICAL / DIVERGENT ----------
    with tempfile.TemporaryDirectory() as td:
        def w(fn: str, text: str) -> None:
            with open(os.path.join(td, fn), "w", encoding="utf-8") as fh:
                fh.write(text)

        for fn in ("m1.rs", "m2.rs", "m3.rs"):
            w(fn, "pub const E8_DIM: usize = 248;\n")  # 三处同值
        w("d1.rs", "pub const FOO: usize = 7;\n")
        w("d2.rs", "pub const FOO: usize = 9;\n")  # 两处异值
        w("t1.rs", "pub const E8_DIM: f64 = 248.0;\n")  # 同名**不同类型** => 另一组
        r = scan([td])
        idn = {x["name"] for x in r["identical"]}
        div = {x["name"] for x in r["divergent"]}
        if "E8_DIM" not in idn:
            fail("跨文件 IDENTICAL 未识别（got %s）" % sorted(idn))
        else:
            print("  PASS 跨文件 IDENTICAL（同名同类型同值，跨 3 文件）")
        if "FOO" not in div:
            fail("跨文件 DIVERGENT 未识别（got %s）" % sorted(div))
        else:
            print("  PASS 跨文件 DIVERGENT（同名同类型不同值）")
        # 同名不同类型是**另一个身份**，不算分歧
        e8 = [x for x in r["identical"] if x["name"] == "E8_DIM"]
        if len(e8) != 1 or e8[0]["n_files"] != 3:
            fail("同名不同类型被混入同一组：%s" % e8)
        else:
            print("  PASS 同名不同类型不混入（类型是身份的一部分）")

    print("[nt_const_dup] selftest %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", nargs="?", default="scan")
    ap.add_argument("--root", action="append", default=None)
    ap.add_argument("--min-sites", type=int, default=2)
    ap.add_argument("--only-identical", action="store_true")
    ap.add_argument("--only-divergent", action="store_true")
    ap.add_argument("--limit", type=int, default=40)
    ap.add_argument("--json", dest="as_json", action="store_true")
    args = ap.parse_args()

    if args.cmd == "selftest":
        return selftest()

    roots = args.root or DEFAULT_ROOTS
    rep = scan(roots, args.min_sites)
    if args.as_json:
        print(json.dumps(rep, ensure_ascii=False, indent=2))
        return 0

    print(
        "[nt_const_dup] 扫描 %d 个 .rs，解析 %d 条 const/static 声明"
        % (rep["n_files_scanned"], rep["n_decls"])
    )
    show_i = not args.only_divergent
    show_d = not args.only_identical
    if show_i and rep["identical"]:
        print("\n== IDENTICAL：值相同的同名重复定义（影子副本，收敛候选）==")
        for rec in rep["identical"][: args.limit]:
            print(_fmt(rec))
    if show_d and rep["divergent"]:
        print("\n== DIVERGENT：同名不同值（更危险：读者必然误用）==")
        for rec in rep["divergent"][: args.limit]:
            print(_fmt(rec))
    if not rep["identical"] and not rep["divergent"]:
        print("  无命中")
    print(
        "\n共 IDENTICAL %d 组 / DIVERGENT %d 组。"
        "⚠️ 本工具只报候选：每个命中都要读那一行本身再决定，勿据报告直接删（R-SCAN-1b）。"
        % (len(rep["identical"]), len(rep["divergent"]))
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())