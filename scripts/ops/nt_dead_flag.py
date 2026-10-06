#!/usr/bin/env python3
"""nt_dead_flag.py — 「声明了默认值，却从未被读取」的配置开关门。

## 要防的缺陷（A55 实测实例）

`EntityMappingConfig::enable_semantic_mapping`：
- 声明为 `pub enable_semantic_mapping: bool`（有默认值 `true`）
- 模块文档明写该能力是本模块核心
- **`cargo check` 通过、现有测试全绿、`deny(warnings)` 也不报**
  （因为字段出现在 `Default` 初始化里，被视为「已使用」）
- 但它在 `calculate_name_similarity` 中**从未被读取**
  ⇒ 该能力实际不存在，而 `test_name_similarity` 里那条
  `assert!(calculate_name_similarity("HashMap", "Dict") > 0.0)`
  **从写下来就不可能通过**

这是比编译错误更隐蔽的一类：**开关看起来是活的（默认值存在、warning 干净），
实际是死的。** 故需要独立门。

## 判据

1. 收集全部 `pub <name>: bool` 字段（配置开关候选）。
2. 剥离注释与字符串字面量（落实 R-SCAN-1b：裸 grep 命中不构成证据）。
3. 读点 = `<任意标识符>.<name>` 的出现。
   字段声明（`pub x: bool`）与字段初始化（`x: true`）**不带前导点**，
   故天然不会被计为读点 —— 这是本判据能成立的关键。
4. 零读点 ⇒ 上报为「疑似死开关」。

## 已知误报来源（进基线，不当缺陷）

- 只为序列化落盘、由外部工具读取的字段
- 被测试专用代码读取但生产路径未用的字段
- trait 关联常量（不在本判据范围）

## 用法

    python3 scripts/ops/nt_dead_flag.py            # advisory，列清单，exit 0
    python3 scripts/ops/nt_dead_flag.py --strict   # 对基线外的新增非零
    python3 scripts/ops/nt_dead_flag.py --audit    # 同时打印读点计数

退出码遵循门族约定：0 通过 / 1 有新增（--strict）/ 2 门自身无法判定。
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

SKIP_DIRS = {"target", ".git", "node_modules", ".worktrees", "models", "vendor"}
FIELD_RE = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*bool\b")
CONFIG_HINT_RE = re.compile(r"\b(Config|Settings|Options|Params|Policy|Params)\b")


_RE_RAW = re.compile(r'r(#*)"(?:\\.|[^"\\])*"\1', re.S)
_RE_STR = re.compile(r'"(?:\\.|[^"\\])*"', re.S)
_RE_BLOCK = re.compile(r'/\*.*?\*/', re.S)
_RE_LINE = re.compile(r'//[^\n]*')
_RE_CHAR = re.compile(r"'(?:\\.|[^'\\])'")


def strip_comments_and_strings(src: str) -> str:
    """把注释与字符串字面量清空，保留行结构（供按行定位）。

    ⚠️ 这是**正则近似**而非完整词法分析：形如 `"//"` 或 `/* " */` 的
    极端嵌套可能误判。对门而言可接受——误判只会让某字段的读点数偏多
    （漏报），不会产生误报（误报才是棘轮门的成本）。
    落实 R-SCAN-1b：裸 grep 命中不构成证据，故必须先剥离非代码。
    """
    src = _RE_RAW.sub(_blank, src)
    src = _RE_STR.sub(_blank, src)
    src = _RE_CHAR.sub(_blank, src)
    src = _RE_BLOCK.sub(_blank_keep_nl, src)
    src = _RE_LINE.sub(lambda m: " " * len(m.group(0)), src)
    return src


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _blank_keep_nl(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def iter_rs_files(root: Path):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if fn.endswith(".rs"):
                yield Path(dirpath) / fn


_RE_FIELD = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*bool\b")
_RE_STRUCT = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)")
_RE_DOT_TOKEN = re.compile(r"\.\s*([a-z_][a-z0-9_]*)")


def scan_once(root: Path):
    """单次扫描全仓：返回 (字段候选列表, 全仓 `.field` 读点计数)。

    原实现是 O(文件 × 字段) —— 每字段全仓重扫，实测超时（RC=124）。
    改为一次遍历同时统计所有 `.field` 出现，复杂度降到 O(总行数)。
    """
    fields = []
    read_counts: dict[str, int] = {}
    for path in iter_rs_files(root):
        try:
            raw = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        code = strip_comments_and_strings(raw)
        lines = code.split("\n")
        struct_name = None
        derives_serde = False
        pending_serde = False
        for idx, line in enumerate(lines, start=1):
            # 属性行先暂存：derive 紧邻 struct 之前，必须由 struct 行消费。
            # ⚠️ 早期版本在 struct 行直接把 derives_serde 重置为 False，
            # 正好清掉紧邻其前的 #[derive(..., Serialize)] ⇒ 分类全部失真。
            if re.match(r"^\s*#\[", line):
                if "Serialize" in line or "Deserialize" in line:
                    pending_serde = True
                continue
            sm = _RE_STRUCT.match(line)
            if sm:
                struct_name = sm.group(1)
                derives_serde = pending_serde
                pending_serde = False
                continue
            fm = _RE_FIELD.match(line)
            if fm:
                fields.append(
                    (fm.group(1), path, idx, struct_name, derives_serde)
                )
            for tok in _RE_DOT_TOKEN.findall(line):
                read_counts[tok] = read_counts.get(tok, 0) + 1
    return fields, read_counts


def locate_read_sites(root: Path, field: str, limit: int = 3):
    """零读点字段的定位复查（本应为空；有则说明索引口径需修正）。"""
    pat = re.compile(r"\.\s*" + re.escape(field) + r"\b")
    hits = []
    for path in iter_rs_files(root):
        try:
            code = strip_comments_and_strings(path.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            continue
        for idx, line in enumerate(code.split("\n"), start=1):
            if pat.search(line):
                hits.append(f"{path}:{idx}")
                if len(hits) >= limit:
                    return hits
    return hits


def load_baseline(path: Path):
    if not path.exists():
        return set()
    names = set()
    for line in path.read_text(encoding="utf-8").split("\n"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        names.add(line.split("|")[0].strip())
    return names


def main() -> int:
    ap = argparse.ArgumentParser(description="检出「声明了默认值却从未被读取」的配置开关")
    ap.add_argument("--root", default=".")
    ap.add_argument("--strict", action="store_true", help="对基线外新增非零")
    ap.add_argument("--audit", action="store_true", help="打印读点计数与样例位置")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    if not (root / "neotrix-core").exists() and not (root / "crates").exists():
        print(f"dead-flag: 无法在 {root} 下识别 NeoTrix 布局 ⇒ 门自身无法判定", file=sys.stderr)
        return 2

    baseline_path = root / "scripts" / "dead-flag-baseline.txt"
    baseline = load_baseline(baseline_path)

    fields, read_counts = scan_once(root)
    dead = []
    for name, decl_path, line_no, struct_name, serde in fields:
        if read_counts.get(name, 0) == 0:
            # 二次复查：确认真的零读点（防止索引口径错误导致误报）
            if not locate_read_sites(root, name, limit=1):
                dead.append((name, decl_path, line_no, struct_name, serde))
            else:
                continue
        elif args.audit:
            print(f"  活{name}: {read_counts[name]} 读点")

    rel = lambda p: str(Path(p).resolve().relative_to(root))  # noqa: E731

    new = [d for d in dead if d[0] not in baseline]
    serde_mirrors = [d for d in dead if d[4]]
    behavioral = [d for d in dead if not d[4]]
    print(
        f"dead-flag: 布尔字段 {len(fields)} 个；零读点 {len(dead)} 个"
        f"（其中 serde 外部格式镜像 {len(serde_mirrors)}、本仓行为开关 {len(behavioral)}）；"
        f"基线已裁决 {len(baseline)}；新增 {len(new)}"
    )

    for name, decl_path, line_no, struct_name, serde in sorted(dead, key=lambda d: d[0]):
        tag = "已知" if name in baseline else "新增"
        hint = f" [{struct_name}]" if struct_name and CONFIG_HINT_RE.search(struct_name) else ""
        kind = "serde-镜像" if serde else "**本仓行为开关**"
        print(f"      · {tag} {rel(decl_path)}:{line_no} `pub {name}: bool`{hint}  ({kind})")

    if not dead:
        print("      · 无零读点布尔配置字段")

    if args.strict and new:
        print(f"⛔ 新增 {len(new)} 个疑似死开关（声明了默认值但从未被读取）", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())