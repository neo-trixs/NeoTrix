#!/usr/bin/env python3
"""能力体系裁决台账门 —— 守住 `config/capability_systems.toml`。

## 这道门查什么（三条，全部零假阳性）

1. **锚点漂移**：每个声明的体系，其 `symbol` 必须仍存在于 `anchor` 文件。
   ⇒ 代码改了/挪了/删了，台账不会悄悄过期。
2. **处置闭集**：`disposition` 必须是台账头部定义的 5 个值之一。
   ⇒ 不会出现「待定」「TODO」这种无处落地的状态。
3. **证据强制**：每个体系必须有非空 `evidence`。
   ⇒ 「据说没人用」这种无据判断写不进来。

## ⛔ 这道门**故意不查**什么

**不查「生产消费者数量」**。要判定「某个调用点是否在 `#[cfg(test)]` 里」
必须真正读代码边界；文本级正则必然误报，而误报会诱导人去「修」正确的代码
（AGENTS.md R-SCAN-1 / R-SCAN-1b：grep 命中不构成证据）。该判断属人工裁决，
其结论写进台账的 `evidence` 字段，由本门强制「必须写」。

⇒ 门的作用是**防新增**（新体系必须先声明 + 写证据）与**防漂移**，
   **不是**替人做消费者计数。
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LEDGER = ROOT / "config" / "capability_systems.toml"

ALLOWED = {
    "spine",
    "active",
    "bridge-only",
    "dormant",
    "deprecated",
}

# 台账头部注释里定义的处置语义（与本清单一一对应）
SEMANTICS = {
    "spine": "脊柱本体或直接桥接",
    "active": "有真实生产消费者",
    "bridge-only": "仅作存量适配，禁止新增使用",
    "dormant": "定义完整但零生产消费者",
    "deprecated": "已判死，禁止新增调用，待删",
}


def main() -> int:
    if not LEDGER.is_file():
        print(f"capability-inventory: 找不到台账 {LEDGER}", file=sys.stderr)
        return 2
    try:
        data = tomllib.loads(LEDGER.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"capability-inventory: 台账解析失败: {exc}", file=sys.stderr)
        return 2

    systems = data.get("system") or []
    if not systems:
        print("capability-inventory: 台账为空 ⇒ 无从裁决", file=sys.stderr)
        return 1

    fail = 0
    seen: set[str] = set()
    counts: dict[str, int] = {}

    for i, s in enumerate(systems):
        sid = s.get("id")
        where = f"第 {i + 1} 条"

        if not sid:
            print(f"  ❌ {where}: 缺 id")
            fail = 1
            continue
        if sid in seen:
            print(f"  ❌ {sid}: 台账里 id 重复")
            fail = 1
            continue
        seen.add(sid)

        disp = s.get("disposition")
        if disp not in ALLOWED:
            print(
                f"  ❌ {sid}: disposition={disp!r} 不在闭集内 "
                f"（{'/'.join(sorted(ALLOWED))}）"
            )
            fail = 1
            continue
        counts[disp] = counts.get(disp, 0) + 1

        if not (s.get("evidence") or "").strip():
            print(f"  ❌ {sid}: 缺 evidence（禁止无据判断）")
            fail = 1

        anchor = s.get("anchor")
        symbol = s.get("symbol")
        if not anchor or not symbol:
            # 允许「模块级锚点」（如 market.rs / crystal_serve.rs 无单一符号）
            if not anchor:
                print(f"  ❌ {sid}: 缺 anchor")
                fail = 1
            continue

        p = ROOT / anchor
        if not p.is_file():
            print(f"  ❌ {sid}: anchor 文件不存在: {anchor}")
            fail = 1
            continue
        text = p.read_text(encoding="utf-8", errors="replace")
        if symbol not in text:
            print(
                f"  ❌ {sid}: 锚点漂移 —— {anchor} 里已找不到 {symbol!r}\n"
                f"       （代码改了/挪了/删了 ⇒ 台账必须同步更新）"
            )
            fail = 1

    summary = "  ".join(f"{k}={v}" for k, v in sorted(counts.items()))
    print(f"  体系数: {len(systems)}   {summary}")
    if fail:
        print("capability-inventory: FAIL")
        return 1
    print("capability-inventory: PASS（锚点全部在位 / 处置合法 / 证据齐备）")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())