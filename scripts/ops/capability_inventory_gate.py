#!/usr/bin/env python3
"""能力体系裁决台账门 —— 守住 `config/capability_systems.toml`。

## 这道门查什么（三条，全部零假阳性）

1. **锚点漂移**：每个声明的体系，其 `symbol` 必须仍存在于 `anchor` 文件。
   ⇒ 代码改了/挪了/删了，台账不会悄悄过期。
2. **处置闭集**：`disposition` 必须是台账头部定义的 5 个值之一。
   ⇒ 不会出现「待定」「TODO」这种无处落地的状态。
3. **证据强制**：每个体系必须有非空 `evidence`。
   ⇒ 「据说没人用」这种无据判断写不进来。
4. **注册表枚举合法性**（2026-10-08 新增）：`.neotrix/capability_registry.json`
   里每个节点的 `domain` / `layer` / `constellation` 必须落在 Rust 枚举的
   序列化取值闭集内，且每个 `evolution_log[].op` 必须是合法枚举名。

## ⛔ 这道门**故意不查**什么

**不查「生产消费者数量」**。要判定「某个调用点是否在 `#[cfg(test)]` 里」
必须真正读代码边界；文本级正则必然误报，而误报会诱导人去「修」正确的代码
（AGENTS.md R-SCAN-1 / R-SCAN-1b：grep 命中不构成证据）。该判断属人工裁决，
其结论写进台账的 `evidence` 字段，由本门强制「必须写」。

⇒ 门的作用是**防新增**（新体系必须先声明 + 写证据）与**防漂移**，
   **不是**替人做消费者计数。
"""

from __future__ import annotations

import json
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


# 与 crates/nt-core-capability-tree/src/node.rs 的 serde 取值**逐字一致**。
# ⛔ 这是**手抄**的闭集 —— 若枚举改名，本门会红，那是刻意的：
#    「门与代码不同步」本身就是要暴露的漂移。
LEGAL_DOMAIN = {
    "core", "mind", "memory", "world", "act", "shield", "io", "meta",
    "nexus", "governance", "repair", "neobot",
}
LEGAL_LAYER = {
    "l0primitive", "l1composite", "l2orchestrator", "l2world", "l3domainservice",
    "l3memory", "l4application", "l4cognition", "l5conscious", "l6self",
    "l7capability", "l8autonomic",
}
LEGAL_CONSTELLATION = {
    "c0compile", "c1unittest", "c2integrationtest", "c3benchmark",
    "c4mainpipeline", "c5selfhealing", "c6evolutionloop", "c2integration",
}
LEGAL_EVOLUTION_OP = {
    "budding", "grafting", "pruning", "cross_pollination", "maturation", "strengthen",
}
REGISTRY_JSON = ROOT / ".neotrix" / "capability_registry.json"


def audit_registry_enums() -> int:
    """判据 ④：注册表里每个枚举字段都必须合法。

    为什么必须有这条（实测事故）：`.neotrix/capability_registry.json` 曾出现
    `domain:"neobot"` / `layer:"l1primitive"` / `layer:"l6meta"` /
    `constellation:"c2system"` / `constellation:"c3experience"` / `op:"bud"`。
    serde 是 **fail-fast**：只报第一个错 ⇒ 整份 326 节点注册表解析失败 ⇒
    **326 个节点在生产里全部不可见**，而 `skill_tree.rs` 用 `.ok()?` **完全静默**。
    ⇒ 这类漂移必须有机器判据，否则只能靠「恰好有人注意到测试红」。
    """
    if not REGISTRY_JSON.is_file():
        print(f"  ❌ 判据④：找不到 {REGISTRY_JSON}")
        return 1
    try:
        data = json.loads(REGISTRY_JSON.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"  ❌ 判据④：注册表 JSON 解析失败: {exc}")
        return 1

    bad = 0
    checks = (
        ("domain", LEGAL_DOMAIN),
        ("layer", LEGAL_LAYER),
        ("constellation", LEGAL_CONSTELLATION),
    )
    for field, legal in checks:
        offenders = sorted(
            {
                str(n.get(field))
                for n in data.get("nodes", [])
                if n.get(field) not in legal
            }
        )
        if offenders:
            bad = 1
            print(
                f"  ❌ 判据④：{field} 非法取值 {offenders}"
                f"（合法集 {sorted(legal)}）\n"
                f"       ⇒ serde 会 fail-fast ⇒ 整份注册表在生产里不可解析"
            )

    bad_ops = sorted(
        {
            str(e.get("op"))
            for n in data.get("nodes", [])
            for e in (n.get("evolution_log") or [])
            if e.get("op") not in LEGAL_EVOLUTION_OP
        }
    )
    if bad_ops:
        bad = 1
        print(f"  ❌ 判据④：evolution_log[].op 非法取值 {bad_ops}")

    if not bad:
        n = len(data.get("nodes", []))
        print(f"  判据④：注册表 {n} 节点的 domain/layer/constellation/op 全部合法 ✅")
    return bad


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

    fail |= audit_registry_enums()

    summary = "  ".join(f"{k}={v}" for k, v in sorted(counts.items()))
    print(f"  体系数: {len(systems)}   {summary}")
    if fail:
        print("capability-inventory: FAIL")
        return 1
    print("capability-inventory: PASS（锚点全部在位 / 处置合法 / 证据齐备）")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())