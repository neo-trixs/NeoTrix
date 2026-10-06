#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
nt_claims_numbers.py — 验「文档里写的**数字**」是否还等于实测值。

================================ 判据 ================================
**吸收路线 N-5**（`FEATURE-MAP-TASKS-2026-09-29.md` 批次 A，来源
`kev/scripts/verify_claims.py`）：

> `nt_manifest.py`（Context Manifest）只验 `file:line` 指向是否还存在于实文件
> —— 实测确认：5 claims / 1 个 file:line 有效。
> ⛔ **它不查数字** ⇒ 「本文档说的 79 条」在变成 80 条后仍然是「79 条」，
> 门不会响。而数字恰恰是 agent 最常直接采信的东西。

**本门补的就是这个洞**：把「文档里的数字」变成可复算的断言。

================================ 本会话的实证 ================================
这不是假想需求。**同一场会话内踩了 4 次陈旧数字**：

| 文档写的 | 实测 | 谁发现的 |
|---|---|---|
| `AGENTS.md` 「75 条索引」 | 79（后又 80） | 我，`nt_find --audit` |
| `FINAL-ROADMAP` 「下一步仍是 A3」 | A3 已 ✅ 且带证伪记录 | 我，读 FEATURE-MAP 时 |
| `FEATURE-MAP` N-4 = ⬜ | `visible_to_model` 已接生产 | 我，grep 实际代码 |
| `gate-registry.tsv` 「恒红 NEW 5」 | 实测 NEW 24 | 我，跑门时 |

⇒ 共性：**陈旧的数字比没有数字更贵**，因为它看起来是可采信的。

================================ 设计约束 ================================
1. **零副作用**：只做**纯文本提取**（读文件 + 正则 + 整数比较）。
   ⛔⛔ **绝不执行被检查的命令**。这是 R-SCAN-4 的直接应用：
   `check-disk.sh` 初版把反引号示例当命令真的执行，删掉 115.2 GiB。
   ⇒ 只查「文档里的数字 vs **权威源里的数字**」这种可复算的量。

1b. ⛔⛔ **注册表存「指向源的指针」，不存字面量**（2026-10-06 建门当日自查抓到的设计缺陷）：
   初版注册表第 3 列是 `expected` **字面量**，判据是「文档数字 == expected」。
   ⇒ **文档与 expected 同时陈旧时门照样绿**：实测把 task-index 加到 83 条后，
   AGENTS.md 写 82、expected 也写 82 ⇒ 门 rc=0，而现实是 83。
   **这是最坏形态：两处陈旧互相印证。**
   ⇒ 改为每条 claim 登记**两个** pattern：
   `doc + doc_pattern`（文档怎么说）与 `src + src_pattern`（权威源是什么），
   门**同一次运行里各自提取再互比** ⇒ 真源是三方的（文档 / 权威源 / 门自己），
   字面量退出设计 ⇒ 无法「一起陈旧」。
2. **注册制**：登记写在 `scripts/claims-numbers.tsv`，门只读它
   （对齐 `gate-registry.tsv` 的做法：不替每条规则写注入脚本）。
3. **防空转**（抄 better-sidebar「a glob that silently matches nothing would
   make this contract vacuous」）：
   ① 注册表为空 ⇒ 判红 ② **每条 claim 的 pattern 必须真的在目标文档里命中**
   —— 命中不到说明**文档改了形状**，这条断言已经失去意义，同样判红。
   ⚠️ 这一条是本门的核心：否则「文档把那句话删了」会让门静默变绿。

用法: nt_claims_numbers.py [--strict] [--list]
  默认 advisory（报告，exit 0）；--strict 任一失败 exit 1（供 CI）。
"""
from __future__ import annotations

import argparse
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REGISTRY = os.path.join(REPO, "scripts", "claims-numbers.tsv")

# ── 路径白名单：只允许这三类被引用文件 ─────────────────────────────
#   理由：claims 数字的价值在于「数字来自某个可复算的源」。若允许引用
#   任意路径，就会有人登记「README.md 里写 2026」这种不可复算的断言，
#   门变成永绿摆设（= 没有门）。故源头必须是**台账/源码**。
ALLOWED_PREFIX = ("scripts/", "neotrix-core/", "crates/", "docs/architecture/",
                  "docs/plans/", "docs/standards/", "sessions/", ".neotrix/",
                  ".github/", "AGENTS.md", "TODO.md",
                  "REVIEW.md", "README.md", "Makefile", "Cargo.toml")


def parse_registry(path: str) -> list[dict]:
    rows = []
    with open(path, encoding="utf-8") as fh:
        for ln, raw in enumerate(fh, 1):
            line = raw.rstrip("\n")
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            parts = line.split("\t")
            # 5 列：doc, doc_pattern, src, src_pattern, note
            if len(parts) < 5:
                print(f"claims-numbers: {path}:{ln} 列数不足（需 5："
                      f"doc/doc_pattern/src/src_pattern/note）", file=sys.stderr)
                continue
            doc, dpat, src, spat = parts[0], parts[1], parts[2], parts[3]
            note = parts[4] if len(parts) > 4 else ""
            rows.append({"line": ln, "doc": doc, "pattern": dpat,
                         "src": src, "src_pattern": spat, "note": note})
    return rows


def extract(doc_abs: str, pattern: str) -> list[int]:
    """Return the integers a claim asserts about `doc_abs`.

    Two modes:
      · ``count:<regex>``  → 返回匹配行数（真源常是「有多少条」而非某个字面量）
      · ``<regex 含一个纯数字捕获组>`` → 返回捕获到的整数

    ⛔ 两种模式都只是**读文本**，不执行任何东西（R-SCAN-4）。
    """
    try:
        with open(doc_abs, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
    except OSError:
        return []
    if pattern.startswith("count:"):
        rx = pattern[len("count:"):]
        return [len(re.findall(rx, text, re.MULTILINE))]
    out = []
    for m in re.finditer(pattern, text):
        for g in m.groups():
            if g and re.fullmatch(r"\d+", g.strip()):
                out.append(int(g.strip()))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--strict", action="store_true")
    ap.add_argument("--list", action="store_true", help="只列注册表，不判")
    a = ap.parse_args()

    if not os.path.isfile(REGISTRY):
        print(f"claims-numbers: 注册表不存在: {REGISTRY}")
        return 2
    rows = parse_registry(REGISTRY)
    if a.list:
        for r in rows:
            print(f"{r['doc']}\t{r['pattern']}\t{r['expected']}\t{r['note']}")
        return 0

    print("=== claims 数字追溯门（N-5）===")
    print(f"注册表: scripts/claims-numbers.tsv（{len(rows)} 条）\n")

    # ── 防空转 ①：注册表为空 ⇒ 判红 ────────────────────────────────
    if not rows:
        print("⛔ 注册表为空 ⇒ 本门什么都没查（空门比没有门更坏）")
        return 1 if a.strict else 0

    fails = 0
    vacuous = 0
    for r in rows:
        tag = f"scripts/claims-numbers.tsv:{r['line']}"
        # ── 两侧都必须是白名单内的可复算文件 ──────────────────────
        for label, rel in (("doc", r["doc"]), ("src", r["src"])):
            if not rel.startswith(ALLOWED_PREFIX):
                print(f"  ❌ {tag}: {label} 不在白名单内（{rel}）⇒ 不可复算的断言不收")
                fails += 1
        if fails:
            continue
        doc_abs = os.path.join(REPO, r["doc"])
        src_abs = os.path.join(REPO, r["src"])
        for label, abs_p in (("doc", doc_abs), ("src", src_abs)):
            if not os.path.isfile(abs_p):
                print(f"  ❌ {tag}: {label} 文件不存在: "
                      f"{r[label] if label == 'doc' else r['src']}")
                fails += 1
        if fails:
            continue
        dnums = extract(doc_abs, r["pattern"])
        snums = extract(src_abs, r["src_pattern"])
        # ── 防空转：任一侧 pattern 命中不到 ⇒ 断言已失去意义 ────────
        if not dnums:
            print(f"  ⛔ {tag}: doc_pattern 在 {r['doc']} 里**一个都没命中** ⇒ 断言已失效")
            vacuous += 1
            continue
        if not snums:
            print(f"  ⛔ {tag}: src_pattern 在 {r['src']} 里**一个都没命中** ⇒ 真源失效")
            vacuous += 1
            continue
        common = sorted(set(dnums) & set(snums))
        if not common:
            print(f"  ❌ {r['doc']} 说 {sorted(set(dnums))}，"
                  f"{r['src']} 实为 {sorted(set(snums))}，无交集"
                  f"　← 门: claims-numbers.tsv:{r['line']}（{r['note'] or '无备注'}）")
            fails += 1
        else:
            print(f"  ✅ {r['doc']} 的 {common} 与 {r['src']} 一致")
    print()
    if vacuous:
        print(f"⛔ {vacuous} 条 claim 的 pattern 已失效 ⇒ **不可复算的断言**")
        print("   处置：更新 pattern 到文档现状，或删掉该行。⛔ 不要改 expected 去迎合。")
    if fails == 0 and vacuous == 0:
        print(f"claims-numbers: PASS（{len(rows)} 条全部一致且非空转）")
        return 0
    total = fails + vacuous
    if a.strict:
        print(f"claims-numbers: FAIL — {total} 项"
              f"（不符 {fails} + 空转 {vacuous}）")
        return 1
    print(f"claims-numbers: DONE(advisory) — {total} 项，使用 --strict 阻断")
    return 0


if __name__ == "__main__":
    sys.exit(main())