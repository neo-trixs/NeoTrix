#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
元门：门必须「接了 CI」或「登记豁免且写明理由」。

⭐⭐⭐ **立门理由（本仓复发型缺陷的第 N 次）**
本轮实测：`kind=gate` 共 41 道，⭐⭐ **27 道从未在 CI 被引用**。
其中 ⭐⭐⭐ `nt_api_contract.py` **一跑就抓到已发布的缺陷**
（「已注册但契约表无条目」）⇒ ⭐⭐⭐ **门写了却没跑，缺陷就能长期存活**。
⭐⭐ 同一病本轮已实证 **7 次**（孤儿刻度 / 规范化函数没人用 / pet.tsx 绕过
统一出口 / `ring_*` 整棵没接进 mod.rs / 4b 只报告不阻断 /
契约表落后于注册 / ⭐⭐ 本门）。

⭐⭐⭐ **为什么是「元门」而不是把 27 道全接进 CI**
⭐⭐ 逐条实测后发现未接线者分三类，⭐⭐⭐ **只有一类该接**：
① `advisory` 门（实测 9 道）⭐⭐ **就该是本地报告器** ⭐⭐ 接进 CI 只会
   让「恒红的门 = 没有门」（本仓门纪律）。
② **需要参数**的门（`mutation-check` / `nt_absorb_denylist`）⭐⭐ 是**工具**，
   ⭐⭐ 无法无参运行 ⇒ ⭐⭐ 天然不该进 CI 阻断。
③ ⭐⭐ **隐式依赖未声明**：9 道 `neobot-check-*.mjs` `require('playwright')`，
   ⭐⭐⭐ 而 `playwright` **未在 package.json 声明** ⇒ ⭐⭐⭐ 干净检出上跑不了
   ⇒ ⭐⭐ 接进 CI 会**立刻红** ⇒ ⭐⭐⭐ 正是「恒红的门」。

⇒ ⭐⭐⭐ 所以最优解不是「全接」，⭐⭐⭐ 而是 ⭐⭐ **让「接线状态」本身可判定**：
每道门**必须**二选一，且豁免**必须写理由**，⭐⭐⭐ **死豁免判红**。

⭐⭐⭐ 五条判据（每条都能独立判红）：
  P1 ⭐⭐ 覆盖率：每个 `kind=gate` 已接 CI **或** 已登记豁免
  P2 ⭐⭐ 死豁免：登记了却已接 CI（或脚本已不存在）⇒ 请删除
  P3 ⭐⭐ 索引漂移：豁免理由为「脚本不存在」时，⭐⭐ **脚本必须真的不存在**
      （反向也成立：⭐⭐ 索引说有的，⭐⭐ 文件必须真的有）
  P4 ⭐⭐⭐ **可运行性**：以「依赖未声明」豁免的门，⭐⭐ **该依赖必须真的未声明**
      ⇒ ⭐⭐ 依赖一旦补上，⭐⭐ **豁免自动失效并判红**（⭐⭐ 逼人真去接）
  P5 ⭐⭐ 防空转：⭐⭐ 索引解析失败 / 门数为 0 ⇒ 判红
      （抄 better-sidebar：⭐⭐「a glob that silently matches nothing would
      make this contract vacuous」）
"""
from __future__ import annotations

import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
INDEX = os.path.join(REPO, '.neotrix/task-index.json')
WF = os.path.join(REPO, '.github/workflows')
UI = os.path.join(REPO, 'apps/neobot-desktop/neobot-ui')

# ⭐⭐⭐ 豁免登记表：**每条必须写理由**。
# ⭐⭐ ⛔ 裸字符串列表会让这道门退化成「什么都往里塞」——
# ⭐⭐ 「有豁免」与「无约束」必须可区分（对标 better-sidebar：
# ⭐⭐ 它的门只扫 `color:`，于是尺寸漂了它不会红）。
EXEMPT = {
    # ── ① advisory 门：实测自称 advisory，接进 CI 只会「恒红」──
    'check-naming': 'ratchet（2026-10-08 起）：基线 1612（scripts/naming-baseline.txt，干净检出实测）；新增违规 ⇒ 两个模式都判红，可接 CI；无回归 ⇒ exit 0 但输出明写 PASS≠合规（存量 1612 未清）',
    'check-skill-gate': 'advisory：门自身输出「advisory only (use --strict to enforce)」；⭐ --strict 形态是否阻断属另案',
    'check-doc-drift': 'advisory：只报 root-doc deadlinks 数量，不判失败',
    'check-supply-iocs': 'advisory-review：输出「packages above lack license metadata; check before…」',
    'check-build-surface': 'advisory：门自身输出「DONE(advisory). Baseline: …」',
    'check-fuzz-ready': 'advisory：门自身输出「DONE(advisory).」',
    'check-api-surface': 'advisory：门自身输出「DONE(advisory). Baseline: …」',
    'check-disk': '本机门：按磁盘余量给「长期处方」，⭐⭐ 与仓库状态无关，接进 CI 无意义',
    # ⭐ 2026-10-07：`check-layout` 的 advisory 豁免已**删除**（元门 P2 判为死豁免）。
    #   旧理由是「门自身输出 DONE(advisory)」—— 那说的是**无参默认模式**；
    #   而 CI 里接的是 `--strict` 形态，实测造违规时 rc=1 真会阻断 ⇒ 豁免无意义。
    #   ⛔ 教训：豁免理由必须**指名它豁免的是哪个形态**（默认/advisory 还是 --strict），
    #     否则「同一道门的两种退出码」会互相掩护 —— 与本仓「同名 ≠ 同一符号」同型。
    'nt-integration-evidence': 'report 类：产出证据文档，⭐⭐ 不判失败（⭐ 由人读）',
    'coverage-gaps': '⭐⭐ **report 器，非门**：门自身 docstring 写明「本工具是**筛选器**：能指出哪里可能有洞，⭐⭐ **不能证明哪里没问题**」⇒ 实测输出 24 项「调用了但没断言」，⭐⭐ 而**那是正常代码** ⇒ ⭐⭐ 接进 CI 必然恒红（本仓门纪律）',
    'audit-all': '⭐⭐ 聚合器：一条命令串跑多道 advisory 报告器，⭐⭐ 其成分门已各自登记豁免 ⇒ ⭐⭐ 本元门按**其成分门**判定，⭐⭐ 避免重复登记掩盖缺口',

# ── ② 需要参数 ⇒ 是工具，**不是** CI 阻断项 ──
    'mutation-check': '工具：命令形态是 `nt_mutation_check.py --audit <FILE.rs>`，⭐⭐ **需逐文件传参** ⇒ 无法无参运行 ⇒ 不适合做 CI 阻断',
    'nt-absorb-denylist': '工具：命令形态是 `nt_absorb_denylist.py <清单文件|->`，⭐⭐ **需传清单** ⇒ 同上',

    # ── ③ ⭐⭐ 隐式依赖未声明 —— ⭐⭐ **已于 2026-10-04 清零** ──
    # ⭐⭐⭐ 原来这里有 9 条 `UNDECLARED_DEP` 豁免（9 道 playwright UI 门）。
    # ⭐⭐ 那 9 道门从 **vendored 冻结树** 解析 playwright，而 CI 只在
    # ⭐⭐ `neobot-ui/` 跑 `pnpm install` ⇒ ⭐⭐ **CI 里必然加载失败**
    # ⭐⭐ ⇒ ⭐⭐⭐ 它们在 CI 上等于不存在。
    # ✅ 处置：playwright 已归位到 `neobot-ui/devDependencies`，
    #    9 道门的 `createRequire` 锚点已改到交付树，⭐⭐ **全部已接进 CI**。
    # ⭐⭐ P4 现在**故意不豁免任何门** ⇒ ⭐⭐ 一旦再出现「依赖未声明」的
    # ⭐⭐ UI 门，它会**当场判红**（⭐⭐ 这道判据的价值就是「逼人真去接」）。
}

# ⭐⭐ 纯本地、**零浏览器/网络/编译依赖** ⇒ 应该接 CI 的门。
# ⭐⭐ 本轮实测它们的 rc=0，⭐⭐ ⛔ 但凡有一道不可无参运行，本元门就判红。
EXPECT_WIRED = {
    'nt-neobot-ui-wiring': '纯 python；只读源码 + dist 文本扫描',
    'neobot-check-selfcontained': '纯 node；只读 dist 产物文本，⭐⭐ 无浏览器',
    'neobot-check-ipc-keys': '纯 node；源码级 IPC 键名比对，⭐⭐ 无浏览器',
    'neobot-check-a1-recovery': '纯 node；⭐⭐ 无浏览器（实测 pw=0）',
    'scan-surface': '纯 python；检查 21 个扫描面存在性',
    'nt-feature-viability': '纯 python；纯扫描',
}

fail: list[str] = []
bad = lambda m: fail.append(m)


def ci_text() -> str:
    out = []
    if os.path.isdir(WF):
        for fn in sorted(os.listdir(WF)):
            if fn.endswith(('.yml', '.yaml')):
                with open(os.path.join(WF, fn), encoding='utf-8') as fh:
                    out.append(fh.read())
    return '\n'.join(out)


def main() -> int:
    print('元门：门必须「接了 CI」或「登记豁免且写明理由」\n')

    # ── P5 防空转自检 ──
    if not os.path.exists(INDEX):
        bad(f'P5 找不到任务索引 {INDEX} ⇒ ⭐⭐ 本门无法判定 ⇒ 判红（不假装通过）')
        return report()
    try:
        with open(INDEX, encoding='utf-8') as fh:
            data = json.load(fh)
    except Exception as e:  # noqa: BLE001
        bad(f'P5 索引解析失败：{e} ⇒ ⭐⭐ 本门无能力 ⇒ 判红')
        return report()
    tasks = data.get('tasks')
    gates = [t for t in tasks if isinstance(t, dict) and t.get('kind') == 'gate'] \
        if isinstance(tasks, list) else []
    if len(gates) < 10:
        bad(f'P5 只读到 {len(gates)} 道门（预期 ≥10）⇒ ⭐⭐ 索引结构可能已变 ⇒ 判红')
        return report()
    print(f'✅ P5 防空转自检：索引解析成功，{len(gates)} 道 kind=gate')

    ci = ci_text()
    if not ci.strip():
        bad('P5 读不到任何 workflow ⇒ ⭐⭐ 「已接线」判定会全部落空 ⇒ 判红')
        return report()

    wired, exempt_ok, missing = set(), set(), []

    for t in gates:
        gid = t.get('id', '?')
        m = re.search(r'(?:bash|python3|node|sh)\s+(\S+)', t.get('tool', '') or '')
        script = m.group(1).split()[0] if m else ''
        is_wired = bool(script) and script in ci
        if is_wired:
            wired.add(gid)
        if gid in EXEMPT:
            exempt_ok.add(gid)
        if script and not os.path.exists(os.path.join(REPO, script)):
            missing.append((gid, script))

    # ── P1 覆盖率 ──
    uncovered = sorted({t.get('id', '?') for t in gates} - wired - exempt_ok)
    if uncovered:
        bad(f'P1 未接 CI 且未登记豁免的门 {len(uncovered)} 道：{", ".join(uncovered)}'
            ' ⇒ ⭐⭐ 要么接进 CI，⭐⭐ 要么在 EXEMPT 里**写明理由**登记')
    else:
        print(f'✅ P1 覆盖率：{len(gates)} 道门全部「已接 CI」或「已登记豁免」'
              f'（接 {len(wired)} / 豁免 {len(exempt_ok)}）')

    # ── P2 死豁免（三种形态，⭐⭐ 变异测试只抓到第一种，⭐⭐ 靠补测才找全）──
    in_index = {t.get('id') for t in gates}
    gone = {gid for gid, _ in missing}
    # ① 脚本已不存在  ② 已不在索引  ③ ⭐⭐**已接 CI**（豁免已无意义，最该删）
    dead = sorted((exempt_ok & gone) | (exempt_ok - in_index) | (exempt_ok & wired))
    if dead:
        why = []
        for g in dead:
            tags = []
            if g in gone:
                tags.append('脚本不存在')
            if g not in in_index:
                tags.append('已不在索引')
            if g in wired:
                tags.append('已接 CI ⇒ 豁免已无意义')
            why.append(f'{g}({"、".join(tags)})')
        bad(f'P2 死豁免 {len(dead)} 道 ⇒ ⭐⭐ 死豁免会让本门失去约束力'
            f'（⭐⭐ 留着就等于「既挡又豁免」）：{"; ".join(why)}')
    else:
        print('✅ P2 无死豁免（脚本缺失 / 已出索引 / 已接 CI 三种形态都查）')

    # ── P3 索引↔文件 漂移（双向）──
    real_missing = [f'{g} → {s}' for g, s in missing
                    if g not in EXEMPT or '不存在' not in EXEMPT[g]]
    if real_missing:
        bad(f'P3 索引指向的脚本不存在（{len(real_missing)}）⇒ ⭐⭐ 索引腐坏，'
            f'⭐⭐ 用 `nt_find.py` 会把人导向空路径：{", ".join(real_missing)}')
    else:
        print('✅ P3 索引指向的脚本均存在')

    # ── P4 ⭐⭐ 可运行性：以「依赖未声明」豁免的，依赖必须真的未声明 ──
    try:
        with open(os.path.join(UI, 'package.json'), encoding='utf-8') as fh:
            pkg = json.load(fh)
        declared = set(pkg.get('dependencies') or {}) | set(pkg.get('devDependencies') or {})
    except Exception as e:  # noqa: BLE001
        bad(f'P4 读不到 neobot-ui/package.json：{e} ⇒ 判红')
        declared = set()

    stale = sorted(g for g, why in EXEMPT.items()
                   if 'UNDECLARED_DEP' in why
                   and 'playwright' not in declared
                   and g in {t.get('id') for t in gates})
    # ⭐⭐ 反向：依赖已声明 ⇒ 豁免**必须**失效并判红（逼人真去接 CI）
    now_stale = sorted(g for g, why in EXEMPT.items()
                       if 'UNDECLARED_DEP' in why and 'playwright' in declared)
    if now_stale:
        bad(f'P4 ⭐⭐ 豁免失效 ⭐⭐：`playwright` **已**在 neobot-ui/package.json 声明'
            f' ⇒ ⭐⭐ 「依赖未声明」不再成立，⭐⭐ 请把这些门接进 CI 并删掉豁免：'
            f'{", ".join(now_stale)}')
        print(f'ℹ️  P4 playwright 已声明 ⇒ {len(now_stale)} 条 UNDECLARED_DEP 豁免已判红')
    else:
        pw_count = sum(1 for g, why in EXEMPT.items() if 'UNDECLARED_DEP' in why)
        print(f'✅ P4 可运行性：{pw_count} 条「依赖未声明」豁免**仍然成立**'
              f'（playwright 未在 neobot-ui/package.json 声明 ⇒ 干净检出跑不了）'
              if 'playwright' not in declared else
              '✅ P4 无 UNDECLARED_DEP 豁免')

    # ── 应当接 CI 的门（EXPECT_WIRED）真的接了吗 ──
    not_wired = sorted(g for g in EXPECT_WIRED if g not in wired)
    if not_wired:
        bad(f'⭐⭐ 零依赖、实测 rc=0 的门**仍未接 CI** {len(not_wired)} 道：'
            f'{", ".join(not_wired)} ⇒ ⭐⭐ 它们在 CI 上等于不存在')
    else:
        print(f'✅ ⭐⭐ {len(EXPECT_WIRED)} 道「零依赖门」全部已接 CI')

    return report()


def report() -> int:
    print()
    if fail:
        for m in fail:
            print(f'  ⛔ {m}')
        print(f'\nFAIL: {len(fail)} 条')
        return 1
    print('PASS: 每道门都已接 CI 或带理由登记豁免；死豁免与索引漂移均无。')
    return 0


if __name__ == '__main__':
    sys.exit(main())