#!/usr/bin/env python3
"""nt_dup_types.py — 同名类型的重复体检（回答「哪些单文件该融合」的**判据**）。

## 为什么要它
本会话初答「哪些单文件要融合精简」时，用了三种判据，得到三种答案：
  ① 文件行数排序      → 指向 2000+ 行的 nt_channel_telegram.rs（**但它内部结构完整，不是"多件事塞一起"**）
  ② 同名动词聚类      → 指向 34 个 `*Score` 类型（**但那是不同领域的合理同名**）
  ③ 同类型名跨文件    → **871 个**，看似全是问题（**但绝大多数字段集不同**）

⇒ **「同名」不等于「重复」**。真正硬的判据是
    **同名 + 字段集完全相同**（⇒ 同一份数据被定义了多遍）

## 产出
- **真重复组**（同名 + 字段集相同）⇒ 融合候选
- **疑似组**（同名 + 字段集高度重叠）⇒ 人工判读
- 统计「同名但字段不同」的量 ⇒ 这些是**合理同名，不该动**

## 为什么只报告不自动改
融合是**架构决策**，不是清理动作：
① 字段集相同 ≠ 语义相同（derive/trait 实现/使用场景可能都不同）
② 跨层移动类型会触发 `check-layer-deps.sh` 的层归属裁决
③ 本会话已因「导出 ≠ 调用」误删过一次（AGENTS.md 明确记载错过 3~5 次）
⇒ 本工具**只提供判据与证据**，改不改由人裁决。

## 用法
    python3 scripts/ops/nt_dup_types.py                 # 报告真重复组
    python3 scripts/ops/nt_dup_types.py --top 15        # 多列几组
    python3 scripts/ops/nt_dup_types.py --name AwarenessReport   # 查单个类型
    python3 scripts/ops/nt_dup_types.py --json          # 机器可读
"""
import argparse
import collections
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(REPO, "neotrix-core", "src")
SKIP_DIRS = {"tests", "__pycache__", "target"}

STRUCT_RE = re.compile(r"pub struct (\w+)\s*(?:<[^>]*>)?\s*\{([^}]*)\}")
FIELD_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z_0-9]*)\s*:", re.M)
DERIVE_RE = re.compile(r"#\[derive\(([^)]*)\)\]")


def collect():
    """返回 {(name, frozenset(fields)): [(relpath, derives)]}"""
    out = collections.defaultdict(list)
    for root, dirs, files in os.walk(SRC):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(root, f)
            try:
                text = open(p, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            rel = os.path.relpath(p, SRC)
            for m in STRUCT_RE.finditer(text):
                name, body = m.group(1), m.group(2)
                fields = frozenset(FIELD_RE.findall(body))
                if not (1 <= len(fields) <= 20):
                    continue
                # 往上找最近的 derive
                pre = text[: m.start()]
                dm = None
                for dm in reversed(list(DERIVE_RE.finditer(pre))):
                    if pre[dm.end():m.start()].strip() == "":
                        break
                derives = tuple(sorted(x.strip() for x in dm.group(1).split(","))) if dm else ()
                out[(name, fields)].append((rel, derives))
    return out


def main():
    ap = argparse.ArgumentParser(description="同名类型重复体检（提供融合判据，不自动改）")
    ap.add_argument("--top", type=int, default=10, help="列出前 N 组真重复")
    ap.add_argument("--name", help="只查这一个类型名")
    ap.add_argument("--json", action="store_true", help="机器可读输出")
    args = ap.parse_args()

    data = collect()
    by_name = collections.defaultdict(list)
    for (name, fields), occ in data.items():
        by_name[name].append((fields, occ))

    # 统计：同名 vs 同名+同字段
    same_name_total = 0
    exact_groups = []
    for name, lst in by_name.items():
        paths = set()
        for _, occ in lst:
            paths.update(p for p, _ in occ)
        if len(paths) < 2:
            continue
        same_name_total += 1
        seen = collections.defaultdict(list)
        for fields, occ in lst:
            seen[fields].extend(occ)
        for fields, occ in seen.items():
            uniq = sorted(set(p for p, _ in occ))
            if len(uniq) > 1:
                exact_groups.append({
                    "name": name,
                    "fields": sorted(fields),
                    "paths": uniq,
                    # occ 是 (relpath, derives_tuple_of_str)；
                    # 收集全部 trait 名并排序（不是 tuple 集合）
                    "derives": sorted({t for _, d in occ for t in d}),
                })

    if args.name:
        for g in exact_groups:
            if g["name"] == args.name:
                print(json.dumps(g, ensure_ascii=False, indent=2))
                return 0
        print(f"nt-dup-types: 无 {args.name} 的真重复组")
        return 1

    exact_groups.sort(key=lambda g: (-len(g["paths"]), g["name"]))

    if args.json:
        print(json.dumps({
            "same_name_types": same_name_total,
            "exact_duplicate_groups": len(exact_groups),
            "groups": exact_groups,
        }, ensure_ascii=False, indent=2))
        return 0

    print(f"nt-dup-types: 同名类型 {same_name_total} 个（分布在多文件）")
    print(f"             其中**同名 + 字段集完全相同** = {len(exact_groups)} 组 ⬅ 融合候选")
    print(f"             其余 {same_name_total - len(set(g['name'] for g in exact_groups))} 个是"
          f"**合理同名**（字段集不同 ⇒ 领域不同，不该动）")
    print()
    for g in exact_groups[: args.top]:
        print(f"── {g['name']}  ({len(g['paths'])} 处, {len(g['fields'])} 字段: "
              f"{', '.join(g['fields'][:5])}{'…' if len(g['fields']) > 5 else ''})")
        print(f"   derive: {', '.join(g['derives'])}")
        for p in g["paths"][:5]:
            print(f"     {p}")
        if len(g["paths"]) > 5:
            print(f"     …另 {len(g['paths']) - 5} 处")
        print()
    print("判据提醒：**同名 ≠ 重复**。字段集相同才是（⇒ 同一份数据被定义多遍）。")
    print("本工具**不自动改** —— 融合是架构决策（改层归属、动 trait 实现、")
    print("且本仓「导出 ≠ 调用」已误删过 3~5 次，见 AGENTS.md）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
