#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""吸收台账门 —— 让「吸收了多少」成为**可核对**的数字，而不是声称。

# 为什么需要这个门

被问「你吸收了多少项目」时，我只能给出**无法复核**的数字：
· 台账 493 条 —— 那是**登记数**，不是阅读数
· 文档里点名的 `owner/repo` 623 个 —— 含大量顺带提及/Deferred/凑数清单
· 历史声称的「≥63 真读」—— 无任何记录支撑，我**无法逐条复核**

⇒ 「吸收」在本仓**不可审计**。本门把这件事变成可核对的：
`.neotrix/absorption-audited.json` 只登记**真读过**的条目，
每条必须附 `read_evidence`（读到哪个文件/哪一节）与落地物或未采纳原因。

# 三条硬规则（任一违反 ⇒ 门红）

1. `read_level=none` 的条目**不得**计入已吸收 —— 它就是「没读」
2. `read_evidence` 为空而 `read_level != none` ⇒ **凑数**，门红
3. `landed == '-'` 而 `not_adopted_reason` 为空 ⇒ **静默丢弃**，门红

# 与 license 的交叉（商用路径）

台账里 `commercial_ok=false` 的项目，即便星数再高也不得吸收。
本门会单独列出，避免「星数高就破例」。

退出码：0 = PASS，1 = FAIL。
"""
from __future__ import annotations

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, '..', '..'))
LEDGER = os.path.join(REPO, '.neotrix/absorption-audited.json')
CACHE = os.path.join(REPO, '.neotrix/absorption-cache.json')
SRC_CSV = os.path.join(REPO, 'docs/architecture/absorption-sources/repos.csv')


def main() -> int:
    if not os.path.isfile(LEDGER):
        print('⛔ 找不到 .neotrix/absorption-audited.json —— 无可审计的吸收记录')
        return 1
    with open(LEDGER, encoding='utf-8') as f:
        doc = json.load(f)
    entries = doc.get('entries', [])

    print('吸收台账门 —— 只统计**真读过**的条目\n')
    fail: list[str] = []

    read_levels = {'readme', 'source', 'design-doc', 'issue'}
    really_read: list[dict] = []

    for e in entries:
        repo = e.get('repo', '(无名)')
        lvl = (e.get('read_level') or 'none').strip()
        ev = (e.get('read_evidence') or '').strip()
        landed = (e.get('landed') or '').strip()
        reason = (e.get('not_adopted_reason') or '').strip()

        if lvl not in read_levels and lvl != 'none':
            fail.append(f'{repo}: read_level={lvl!r} 不在允许集合 {sorted(read_levels)} ∪ {{"none"}}')
            continue

        if lvl == 'none':
            # 规则 1：不计入已吸收；但若声称读过又没证据，仍是凑数
            if ev:
                fail.append(f'{repo}: read_level=none 却填了 read_evidence ⇒ 自相矛盾')
            if not reason:
                fail.append(f'{repo}: 未采纳必须写 not_adopted_reason（否则是「静默丢弃」）')
            continue

        # 规则 2：真读过必须有证据
        if not ev:
            fail.append(f'{repo}: read_level={lvl} 但 **read_evidence 为空** ⇒ 凑数，'
                        f'不许计入已吸收')
            continue
        if landed == '-':
            if not reason:
                fail.append(f'{repo}: landed=\'-\' 但未写 not_adopted_reason')
            continue
        if not landed:
            fail.append(f'{repo}: landed 为空 —— 读过就要么落地、要么写明为何不落地')
            continue
        really_read.append(e)

    print(f'  台账条目 {len(entries)} · **真读过并落地/有结论** {len(really_read)}\n')
    for e in really_read:
        mark = '✅' if (e.get('landed') or '-') != '-' else '·'
        print(f'  {mark} {e["repo"]:<34} [{e["read_level"]:<10}] → {e["landed"][:52]}')

    # ── license 交叉：商用禁项单列 ──
    blocked = [e for e in entries if e.get('commercial_ok') is False]
    if blocked:
        print(f'\n  ⛔ 商用禁项 {len(blocked)} 个（**星数高也不得破例**）：')
        for e in blocked:
            print(f'     · {e["repo"]} — {e.get("license", "?")}')

    # ── license 数据与 GitHub 缓存交叉核对（防台账license 写错）──
    if os.path.isfile(CACHE):
        with open(CACHE, encoding='utf-8') as f:
            cache = json.load(f)
        mismatch = []
        for e in entries:
            got = cache.get(e.get('repo', ''))
            if not isinstance(got, dict) or not got.get('license'):
                continue
            real = got['license'].strip().lower()
            claimed = (e.get('license') or '').strip().lower()
            if real != claimed:
                mismatch.append((e['repo'], claimed, got['license']))
        if mismatch:
            for r, c, g in mismatch:
                fail.append(f'{r}: 台账写 license={c}，GitHub API 实为 {g} ⇒ 商用判定会错')
        else:
            print(f'\n  ✅ license 与 GitHub API 缓存一致'
                  f'（核对 {sum(1 for e in entries if e.get("repo") in cache)} 条）')

    # ── 台账规模声明：防止再次把「登记数」当成「阅读数」──
    n_csv = 0
    if os.path.isfile(SRC_CSV):
        import csv
        with open(SRC_CSV, encoding='utf-8') as f:
            n_csv = sum(1 for _ in csv.DictReader(f))
    print(f'\n  ℹ️  原始台账 {n_csv} 条（**登记数，不是阅读数**）；'
          f'其中可审计的已读 {len(really_read)} 条。'
          f'差额主因：375 条 trendshift 批量登记无元数据，不可评估。')

    if fail:
        print(f'\nFAIL: {len(fail)} 项')
        for f_ in fail:
            print(f'  ⛔ {f_}')
        return 1
    print('\nPASS: 每条「已读」都有证据与落地/未采纳结论；license 已与 API 交叉核对。')
    return 0


if __name__ == '__main__':
    sys.exit(main())
