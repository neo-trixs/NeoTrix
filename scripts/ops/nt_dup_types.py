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

# 「刻意镜像」的设计意图标记 —— 命中任一即判定为**刻意**，不应融合。
#
# 来源：AwarenessReport 一组的裁决（2026-09-29）。该组字段集与 derive 完全相同，
# 但 l1_action/nt_act/nt_act_autonomy/types.rs:1-5 明写
#   「These mirror the L5 ... to preserve the dependency direction:
#     L1 must NOT depend on L5. When L5 evolves, these stay stable
#     as the interface contract for the oracle gate.」
# ⇒ 副本是**刻意的接口隔离层**，不是疏忽。
#
# 这类注释是**作者写下的设计意图**，比任何静态分析都权威。
# 故本工具把它们自动识别出来，避免下一个 agent 逐个人工阅读 122 组注释。
DELIBERATE_MARKERS = (
    "mirror",
    "mirrors",
    "mirroring",
    "keep stable",
    "stay stable",
    "stays stable",
    "interface contract",
    "must NOT depend on",
    "must not depend on",
    "l1-local",
    "local equivalent",
    "locally equivalent",
    "deliberately",
    "intentionally",
    "on purpose",
    "刻意",
    "故意",
)

# 每处定义**上方 12 行内**的注释/文档（刻意声明通常紧邻定义）
DOC_WINDOW = 12


def deliberate_note(path, name):
    """返回命中设计意图标记的注释行；无则返回 None。"""
    full = os.path.join(SRC, path)
    try:
        text = open(full, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    m = re.search(r"pub struct " + re.escape(name) + r"\b", text)
    if not m:
        m = re.search(r"pub enum " + re.escape(name) + r"\b", text)
    if not m:
        return None
    # ① 定义紧邻的注释（DOC_WINDOW 行内）
    window = text[max(0, m.start() - DOC_WINDOW * 80): m.start()]
    candidates = window.split("\n")[-DOC_WINDOW:]
    # ② **文件头**的模块级说明 —— 刻意声明常写在那里。
    #    2026-09-29 实测踩坑：AwarenessReport 的刻意声明在 types.rs:1-5
    #    （「L1-local type equivalents ... mirror the L5 types ...」）
    #    而定义在第 28 行，只扫紧邻窗口会漏判 False（错把刻意当疏忽）。
    head_end = text.find("\n\n", 0)
    if head_end > 0:
        candidates += text[:head_end].split("\n")
    for line in candidates:
        low = line.lower()
        if not low.strip().startswith(("//", "///", "//!", "*")):
            continue
        for marker in DELIBERATE_MARKERS:
            if marker in low:
                return line.strip().lstrip("/!*").strip()
    return None


# 已裁决组（2026-09-29）：**字段集相同但不该融合**，连同理由一并记下，
# 避免下一个 agent 重复调查 122 组候选。
#
# 教训：字段集相同只是**必要条件**，不是充分条件。真正的判据还要读
# **代码注释里的设计意图** —— 本例 L1 那份副本是刻意的接口隔离层。
ADJUDICATED = {
    "AwarenessReport": {
        "verdict": "不融合",
        "reason": (
            "三处字段+derive 完全一致，但 L1 那份是**刻意**的接口隔离："
            "l1_action/nt_act/nt_act_autonomy/types.rs:1-5 明写"
            "「mirror the L5 `awareness_monitor` types to preserve the dependency "
            "direction: L1 must NOT depend on L5. When L5 evolves, these stay stable "
            "as the interface contract for the oracle gate」。"
            "改成 use L0 会让 L1 与 L5 共享同一类型 ⇒ L5 演进直接波及 L1，"
            "**破坏原作者写下的契约**。"
        ),
        "next_action": (
            "唯一可疑的是 L0 那份（零消费者，只有 mod.rs:92 声明）—— "
            "但它是否被下游 crate 使用需另查，**不要顺手删**。"
        ),
        "groups_with": ["CapabilityGap", "GapSeverity"],  # 同源，同裁决
    },
}

def _wrap(text, width):
    """按显示宽度折行（中文按 2 列算，避免表格错位）。"""
    import unicodedata
    def w(s):
        return sum(2 if unicodedata.east_asian_width(c) in "WF" else 1 for c in s)
    out, cur = [], ""
    for ch in text:
        # 必须在**空格**处断行，不能逐字硬切 —— 否则中文会被切坏。
        if w(cur + ch) > width:
            cut = cur.rfind(" ")
            if cut > width // 2:
                out.append(cur[:cut])
                cur = cur[cut + 1:] + ch
            else:
                out.append(cur)
                cur = ch
            continue
        cur += ch
    if cur:
        out.append(cur)
    return out or [""]


def real_module(rel):
    """模块边界 = 从文件往上最近的存在 `mod.rs` 的目录。

    不能用「同一父目录」当模块判据 —— 2026-09-29 实测踩坑：
    `provider_abstraction/{models.rs, provider.rs}` 的父目录是同一层，
    但 `nt_io/{nt_io_avatar_channel.rs, nt_io_user_avatar.rs}` 的父目录
    更大却**不在任何 mod.rs 下**。用父目录会把 40 组「跨模块」误判成
    「同模块」。故必须走真实的 `mod.rs` 边界。
    """
    parts = rel.split("/")
    for i in range(len(parts) - 1, 0, -1):
        cand = "/".join(parts[:i])
        if os.path.exists(os.path.join(SRC, cand, "mod.rs")):
            return cand
    return "/".join(parts[:-1])


def layer_of(rel):
    for seg in rel.split("/"):
        if len(seg) > 1 and seg[0] == "l" and seg[1].isdigit():
            return seg[:2]
    return "xx"


def automatable(grp):
    """这一组能否按「同模块 + 同层 + 无刻意声明」自动处理。

    返回 (bool, 理由)。**默认 False** —— 判不准时一律不动。
    """
    if grp["name"] in ADJUDICATED:
        return False, "已人工裁决"
    if grp["deliberate"]:
        return False, "注释自证是刻意镜像"
    mods = {real_module(p) for p in grp["paths"]}
    if len(mods) > 1:
        return False, f"跨模块（{len(mods)} 个真实 mod.rs 边界）⇒ 需定归属层"
    layers = {layer_of(p) for p in grp["paths"]}
    if len(layers) > 1:
        return False, f"跨层（{'/'.join(sorted(layers))}）⇒ 会触发层归属裁决"
    return True, "同模块 + 同层 + 无刻意声明"


STRUCT_RE = re.compile(r"pub struct (\w+)\s*(?:<[^>]*>)?\s*\{([^}]*)\}")
FIELD_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z_0-9]*)\s*:", re.M)
# 字段名 + 类型（融合判据必须比这个，见 collect() 里的注释）
FIELD_TYPED_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z_0-9]*)\s*:\s*([^,\n]+)", re.M
)
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
                # **必须比字段名+类型**，不能只比字段名 ——
                # 2026-09-29 实测踩坑：`nt_act_trade::{data_model,unified_types}`
                # 的 `Order`，dm 是 {id,order_no,payment_terms,status,…}、
                # ut 是 {id,quote_id,items,created_at,…}，两者**交集 8/11**
                # 但绝不相同。只比名字的判据会把它误判成「字段集相同」。
                # 更糟的是它会诱导人去删 —— 实测按该判据删 15 个类型后
                # cargo check 报 E0119（Default 冲突）+ E0560（no field order_no），
                # 已回滚。
                fields = frozenset(
                    f"{n}:{t.strip()}"
                    for n, t in FIELD_TYPED_RE.findall(body)
                )
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

    # 打「刻意镜像」标记：任一处定义旁的注释表明是刻意副本 ⇒ 该组不是疏忽
    for g in exact_groups:
        hits = []
        for p in g["paths"]:
            note = deliberate_note(p, g["name"])
            if note:
                hits.append((p, note))
        g["deliberate"] = bool(hits)
        g["deliberate_notes"] = hits
        ok, why = automatable(g)
        g["automatable"] = ok
        g["automatable_reason"] = why
        g["module"] = sorted({real_module(p) for p in g["paths"]})
        g["layers"] = sorted({layer_of(p) for p in g["paths"]})

    # 排序：先真疏忽（可动）→ 刻意镜像（别动）→ 已裁决
    # 排序：可自动 → 真疏忽(需人工) → 刻意 → 已裁决
    exact_groups.sort(key=lambda g: (
        0 if g["automatable"] else (1 if not g["deliberate"] and g["name"] not in ADJUDICATED else 2),
        -len(g["paths"]), g["name"]))

    if args.json:
        print(json.dumps({
            "same_name_types": same_name_total,
            "exact_duplicate_groups": len(exact_groups),
            "groups": exact_groups,
        }, ensure_ascii=False, indent=2))
        return 0

    n_auto = sum(1 for g in exact_groups if g["automatable"])
    print(f"             ├─ 🤖 可自动处理（同模块+同层+无声明） {n_auto} 组")
    n_adj = sum(1 for g in exact_groups if g["name"] in ADJUDICATED)
    n_delib = sum(1 for g in exact_groups
                  if g["deliberate"] and g["name"] not in ADJUDICATED)
    n_plain = len(exact_groups) - n_adj - n_delib
    print(f"nt-dup-types: 同名类型 {same_name_total} 个（分布在多文件）")
    print(f"             其中**同名 + 字段集完全相同** = {len(exact_groups)} 组")
    print(f"             其余 {same_name_total - len(set(g['name'] for g in exact_groups))} 个是"
          f"**合理同名**（字段集不同 ⇒ 领域不同，不该动）")
    print()
    print(f"             ├─ ✅ 真疏忽候选（无刻意声明）  {n_plain} 组 ⬅ 可逐组取证")
    print(f"             ├─ ⛔ 刻意镜像（注释自证设计）   {n_delib} 组 别动")
    print(f"             └─ ⚖️  已人工裁决                {n_adj} 组")
    print()
    for g in exact_groups[: args.top]:
        verdict = ADJUDICATED.get(g["name"])
        if verdict:
            tag = f"  ⚖️ 已裁决: {verdict['verdict']}"
        elif g["deliberate"]:
            tag = "  ⛔ 刻意镜像（注释自证设计）"
        elif g["automatable"]:
            tag = f"  🤖 可自动处理（{g['module'][0].split('/')[-1]}）"
        else:
            tag = f"  ✅ 无刻意声明但需人工 —— {g['automatable_reason']}"
        print(f"── {g['name']}  ({len(g['paths'])} 处, {len(g['fields'])} 字段: "
              f"{', '.join(g['fields'][:5])}{'…' if len(g['fields']) > 5 else ''}){tag}")
        print(f"   derive: {', '.join(g['derives'])}")
        for p in g["paths"][:5]:
            print(f"     {p}")
        if len(g["paths"]) > 5:
            print(f"     …另 {len(g['paths']) - 5} 处")
        if g["deliberate"] and not verdict:
            for pth, note in g["deliberate_notes"][:2]:
                print(f"     ▸ 注释自证: {pth}")
                print(f"       「{note[:74]}」")
        if verdict:
            for line in _wrap(verdict["reason"], 92):
                print(f"   ▸ {line}")
            print(f"   ▸ 下一步: {verdict['next_action']}")
            if verdict.get("groups_with"):
                print(f"   ▸ 同裁决同源类型: {', '.join(verdict['groups_with'])}")
        print()
    print()
    print("判据提醒（三条都必须成立，缺一不可）：")
    print("  ① 同名  ② 字段**名+类型**完全相同  ③ 无刻意镜像声明")
    print("以上三条是**必要条件，不是充分条件**。仍需人工确认：")
    print("  ④ 两侧有无 impl 块（工具**不检查** —— 2026-09-29 实测：`Order` 两处字段"
          "名交集 8/11")
    print("     却根本不同，删 15 个类型后 cargo check 报 E0119+E0560；"
          "`unified_types` 多出 9 个 impl 块致 Default 冲突）")
    print("  ⑤ 依赖链：本文件内哪些类型引用它（那些类型可能因字段不同而无法一起融合）")
    print()
    print("⛔ 本工具**不自动改**，且不建议批量自动执行。理由：④⑤ 两项工具查不到，")
    print("   而它们恰恰是「删了才发现」的那类。逐组人工验证，每组独立提交。")
    print("   依据：本仓「导出 ≠ 调用」已误删 3~5 次（AGENTS.md）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
