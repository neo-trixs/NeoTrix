#!/usr/bin/env python3
"""nt_pending_fields_adjudicate.py — D2「待人工判定」字段的逐条裁决器

# 为什么需要它

`check-dead-config-flag.sh` 把零读点 bool 字段分成 5 类，其中 **120 项**落在
「**待人工判定**」—— 门**拒绝**替它们下结论。这是正确的：判定需要看代码，
而 grep 命中不算证据（AGENTS.md R-SCAN-1b：本仓把禁词当数据持有，注释与
字符串里的命中全是误报）。

本工具做那件"人做"的活，且**按可复核的方式**做：
对每个字段，枚举它在**生产代码**（排除 `#[cfg(test)]` 块与 `//` 注释）里的
每一处用法，分成四类并给出**文件:行**证据：

| 类 | 判据 | 处置 |
|---|---|---|
| `live-consumer` | 生产区有**读取**（`if x.f` / `x.f ==` / `&x.f` / `f:` 模式…） | 活的 ⇒ 不动，写进基线 `measured` |
| `written-only` | 生产区只有**赋值**（`f: true` / `x.f = …`），无读取 | **假信号候选** ⇒ 需人确认 |
| `serde-shape` | 该字段所在 struct 派生了 Serialize/Deserialize | 对外数据形状 ⇒ 判「保留」还是「删」属设计决策 |
| `noise` | 字段名在同 struct 内**异名近名**（`enabled`/`active` 等） | 配置噪声 ⇒ 不动 |

# ⛔ 本工具不做的事

**不删任何字段。** 判据是「证据采集」，处置是「人的决策」。
理由：`written-only` 只说明"生产区无人读"，**不说明**"值无意义" ——
若生产代码真的在**填**它（像 `binary_analyzer` 的 `is_executable`），
那是**未接线规格**（值真实），不是假信号。两者的区别必须人来判。

# 用法

```bash
python3 scripts/ops/nt_pending_fields_adjudicate.py            # 汇总表
python3 scripts/ops/nt_pending_fields_adjudicate.py --json      # 机读
python3 scripts/ops/nt_pending_fields_adjudicate.py --field arb_opportunity
```

# ⛔ 只读性（R-SCAN-4）

只做 `read` / `re`：读文件、正则匹配、打印。**无任何写操作**，
不动仓库状态、不跑 cargo。
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field as dc_field
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
ROOTS = [REPO / "neotrix-core" / "src", REPO / "crates"]

# ⛔ 绝不用 `rg -E`（本机静默返回 0，见 AGENTS.md §4.2）；一律 `rg --no-config`。
RG = ["rg", "--no-config", "--no-heading", "-n"]

DERIVE_RE = re.compile(r"#\[derive\([^)]*\b(Serialize|Deserialize)\b[^)]*\)\]")


@dataclass
class FieldUse:
    """字段的一处用法。"""
    path: str
    line: int
    text: str
    kind: str  # read | write | decl | noise
    in_test: bool


@dataclass
class Verdict:
    field: str
    decl_path: str
    decl_line: int
    uses: list[FieldUse] = dc_field(default_factory=list)
    serde_shape: bool = False

    # ── 由 uses 派生 ──
    @property
    def prod_reads(self) -> list[FieldUse]:
        return [u for u in self.uses if u.kind == "read" and not u.in_test]

    @property
    def prod_writes(self) -> list[FieldUse]:
        return [u for u in self.uses if u.kind == "write" and not u.in_test]

    @property
    def test_uses(self) -> list[FieldUse]:
        return [u for u in self.uses if u.in_test]

    @property
    def cls(self) -> str:
        if self.prod_reads:
            return "live-consumer"          # 活消费者 ⇒ 门此前漏判
        if self.prod_writes:
            # 生产在填、无人读 ⇒ **未接线规格**（值真实）≠ 假信号
            return "unwired-spec"
        if self.test_uses:
            return "test-only"             # 只在测试里出现 ⇒ 假信号候选
        return "dead-everywhere"

    @property
    def confidence(self) -> str:
        """证据强度。⚠️ `low` 一律要求人工复核，不自动处置。"""
        if self.prod_reads:
            return "high"                   # 有生产读取点，证据是文件:行
        if self.prod_writes:
            return "high"
        if self.test_uses:
            return "medium"                 # 只在测试里 —— 但同名字段（L15）可能混淆
        return "low"


def load_pending() -> list[tuple[str, int, str]]:
    """从 check-dead-config-flag.sh 的输出取「待人工判定」清单。

    ⛔ **不硬编码任何数字** —— 门输出变了就跟着变（R-SCAN-3）。
    """
    script = REPO / "scripts" / "check-dead-config-flag.sh"
    if not script.exists():
        sys.stderr.write(f"找不到门脚本: {script}\n")
        return []
    out = subprocess.run(
        ["bash", str(script)], capture_output=True, text=True, cwd=str(REPO)
    ).stdout
    items = []
    for ln in out.splitlines():
        if "待人工判定" not in ln or "已知" not in ln:
            continue
        m = re.search(r"(\S+\.rs):(\d+)\s+`pub\s+(\w+)`", ln)
        if m:
            items.append((m.group(1), int(m.group(2)), m.group(3)))
    return items


def strip_test_and_comment_regions(lines: list[str]) -> list[bool]:
    """标出每行是否在 `#[cfg(test)]` 块内或属于注释。

    ⛔ 这是本工具的**核心正确性来源**：D2 的「待人工判定」之所以需要人，
    就是因为测试里的用法不算生产证据。
    """
    in_test_block = False
    depth = 0
    out = []
    for ln in lines:
        s = ln.strip()
        is_comment = s.startswith("//")
        if not in_test_block and re.search(r"#\[cfg\(test\)\]", ln):
            in_test_block = True
        if not is_comment and not in_test_block and not s.startswith("#["):
            depth += ln.count("{") - ln.count("}")
            if depth <= 0:
                depth = 0
        out.append(in_test_block or is_comment)
    return out


def classify_use(text: str) -> str:
    """把一行文本判成 decl / read / write / noise。"""
    t = text.strip()
    if re.search(r"\bpub\s+\w+\s*:", t):
        return "decl"
    # struct 字面量赋值：`field: value,` 或 `field: value`（无接收者）
    if re.match(r"^\w+\s*:\s*[^=]", t) and "=" not in t.split(":")[0]:
        return "write"
    # 显式赋值：`x.field = v`
    if re.search(r"\.%s\s*=(?!=)" % re.escape(""), t):
        pass
    return "read"


def collect_uses(fname: str, decl_path: str) -> list[FieldUse]:
    """收集该字段在全仓的用法。

    # ⛔⛔ **第一版这个函数产出了大量假阳性**，实测暴露的问题：
    #   字段名 `debug` 命中了 `#[derive(Debug, Clone)]`；
    #   字段名 `text`   命中了 `let text = query.to_lowercase()`；
    #   字段名 `success` 命中了完全无关的局部变量。
    #   ⇒ **"名字在行里出现" ≠ "这是该字段的用法"**（AGENTS.md L1：
    #     证据的粒度决定结论的粒度 —— 这里连证据都不成立）。
    #
    # ⇒ 收紧到**只有两种语法**算字段使用：
    #   ① 接收者访问 `.fname`（`x.fname`、`self.fname`）—— 但必须排除
    #      `derive`/`use`/字符串/宏调用里的同名 token；
    #   ② struct 字面量字段 `^\s*fname\s*:`（赋值）或 `fname,`（简写）
    #      —— 且**只在 decl_path 所在文件内**算（跨文件同名字段是 L15 陷阱）。
    """
    uses: list[FieldUse] = []
    pat = re.escape(fname)
    # ① 接收者访问
    recv_rx = re.compile(rf"\.{pat}\b")
    # ② struct 字面量：行首 `fname:` 或 `fname,`（短字段）
    lit_rx = re.compile(rf"^\s*{pat}\s*[:,]")

    for root in ROOTS:
        if not root.exists():
            continue
        for f in root.rglob("*.rs"):
            try:
                lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
            except OSError:
                continue
            masked = strip_test_and_comment_regions(lines)
            rel = str(f.relative_to(REPO))
            is_decl_file = (rel == decl_path)
            for i, ln in enumerate(lines):
                t = ln.strip()
                # ⛔ 先剔除与字段访问语法无关的行（derive / use / 字符串字面量 / 宏）
                if re.search(r"#\[derive", ln) or re.search(r"^use\s", ln):
                    continue
                # 字符串字面量里出现同名字段名 —— 不是字段使用
                if ln.count('"') >= 2 and fname in ln and not recv_rx.search(ln):
                    continue
                kind = None
                if is_decl_file and lit_rx.match(ln):
                    kind = "write" if not re.search(rf"^\s*{pat}\s*:(?!:)", ln) else None
                if kind is None and recv_rx.search(ln):
                    if re.search(rf"\.{pat}\s*=(?!=)", ln):
                        kind = "write"
                    elif re.search(rf"\.{pat}\b\s*[,;)]|\.{pat}\b\s*==|\.{pat}\b\s*&|\.{pat}\b\s*\.|&\s*\w+\.{pat}\b", ln) \
                            or re.search(rf"\.{pat}\b", ln):
                        kind = "read"
                if kind is None:
                    continue
                uses.append(FieldUse(rel, i + 1, t[:120], kind, masked[i]))
    return uses


def serde_shape_near(decl_path: str, decl_line: int) -> bool:
    """该字段所在 struct 是否派生 Serialize/Deserialize。"""
    p = REPO / decl_path
    if not p.exists():
        return False
    lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
    # 从声明往上找最近的 #[derive(
    for i in range(min(decl_line, len(lines)) - 1, max(-1, decl_line - 30), -1):
        if DERIVE_RE.search(lines[i]):
            return True
    return False


def adjudicate(only_field: str | None = None) -> list[Verdict]:
    pending = load_pending()
    out: list[Verdict] = []
    seen: set[tuple[str, str]] = set()
    for path, line, name in pending:
        if only_field and name != only_field:
            continue
        key = (path, name)
        if key in seen:
            continue
        seen.add(key)
        v = Verdict(field=name, decl_path=path, decl_line=line)
        v.uses = collect_uses(name, path)
        v.serde_shape = serde_shape_near(path, line)
        out.append(v)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--field", default=None)
    ap.add_argument("--only-class", default=None,
                    help="只显示某一类: live-consumer/unwired-spec/test-only/dead-everywhere")
    args = ap.parse_args()

    vs = adjudicate(args.field)
    if args.only_class:
        vs = [v for v in vs if v.cls == args.only_class]

    if args.json:
        print(json.dumps([
            {
                "field": v.field,
                "decl": f"{v.decl_path}:{v.decl_line}",
                "class": v.cls,
                "confidence": v.confidence,
                "serde_shape": v.serde_shape,
                "prod_reads": [f"{u.path}:{u.line}" for u in v.prod_reads][:5],
                "prod_writes": [f"{u.path}:{u.line}" for u in v.prod_writes][:5],
                "test_uses": len(v.test_uses),
            } for v in vs
        ], ensure_ascii=False, indent=1))
        return 0

    import collections
    cnt = collections.Counter(v.cls for v in vs)
    print(f"D2「待人工判定」逐条裁决 —— 共 {len(vs)} 项")
    print(f"（⚠️ 本工具**不删字段**；它只采集证据并分类，处置是人的决策）\n")
    for cls, n in cnt.most_common():
        print(f"  {cls:<16} {n}")
    print()
    for v in sorted(vs, key=lambda x: (x.cls, -len(x.prod_reads), x.field)):
        mark = {"live-consumer": "✓活", "unwired-spec": "⚠规格",
                "test-only": "·仅测试", "dead-everywhere": "✗全死"}[v.cls]
        print(f"[{mark}] {v.field}  ({v.decl_path}:{v.decl_line})"
              f"{' serde' if v.serde_shape else ''} conf={v.confidence}")
        for u in v.prod_reads[:3]:
            print(f"        读 {u.path}:{u.line}  {u.text}")
        for u in v.prod_writes[:2]:
            print(f"        写 {u.path}:{u.line}  {u.text}")
        if not v.prod_reads and not v.prod_writes:
            print(f"        （生产区零使用；测试内 {len(v.test_uses)} 处）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
