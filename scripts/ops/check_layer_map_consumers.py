#!/usr/bin/env python3
"""check_layer_map_consumers.py — layer-map.json 的 consumer 路径可达性检查。

## 为什么单独成脚本

原先这个检查被写成 `nt_map_reconcile.py` 的 ```assert 围栏里的一条
`cmd:` 一行命令 —— 结果踩了三个坑：

1. **base 路径错**：consumer 路径相对 `neotrix-core/src/`，不是仓库根。
   用仓库根会把 1 条真幽灵报成 12 条（6 条纯假阳性）。
2. **退出码取反**：写成 `sys.exit(0 if g else 1)` ⇒ **无幽灵（健康）时反而失败**。
3. **围栏解析失败**：复杂的一行命令内嵌在 markdown 围栏里，编辑时极易破坏围栏结构
   ⇒ `nt_map_reconcile` 报 `assertions=0`，而这个"失败"被误当成修复失败。

⇒ 复杂逻辑落脚本，围栏只写 `cmd:python3 scripts/ops/check_layer_map_consumers.py`。
这与本仓既有纪律一致：门逻辑不写在配置/文档里。

退出码遵循门族约定：0 通过 / 1 有幽灵 / 2 门自身无法判定。
"""

from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

# layer-map.json 的 consumer 路径是相对**这个目录**，不是仓库根。
# ★ 这个 base 曾经写错，是「12 条幽灵」里 6 条假阳性的来源。
BASE = Path("neotrix-core/src")
CONSUMER_RE = re.compile(r"^(?P<path>.+\.rs):(?P<line>\d+)$")


def collect_consumers(node, out: list[str]) -> None:
    if isinstance(node, dict):
        for v in node.values():
            collect_consumers(v, out)
    elif isinstance(node, list):
        for v in node:
            collect_consumers(v, out)
    elif isinstance(node, str):
        if CONSUMER_RE.match(node):
            out.append(node)


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    lm = root / ".neotrix" / "layer-map.json"
    if not lm.exists():
        print(f"layer-map:缺失 {lm} ⇒ 门自身无法判定", file=sys.stderr)
        return 2
    try:
        data = json.loads(lm.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        print(f"layer-map: 无法解析（{e}）⇒ 门自身无法判定", file=sys.stderr)
        return 2

    consumers: list[str] = []
    collect_consumers(data, consumers)

    ghosts: list[str] = []
    for c in consumers:
        m = CONSUMER_RE.match(c)
        if m is None:
            continue
        if not (root / BASE / m.group("path")).exists():
            ghosts.append(c)

    total = len(consumers)
    if ghosts:
        print(f"layer-map: consumer {total} 条，其中 {len(ghosts)} 条指向不存在的路径")
        for g in ghosts[:20]:
            print(f"  GHOST  {g}")
        if len(ghosts) > 20:
            print(f"  ...另有 {len(ghosts) - 20} 条")
        # ★ 退出码：有幽灵 = 1（失败）。曾误写成取反。
        return 1

    print(f"layer-map: consumer {total} 条，全部可达（base={BASE}/）")
    return 0


if __name__ == "__main__":
    sys.exit(main())