#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""功能存活性门 —— 把「下架哪些功能」从**意见**变成**可复算的结论**。

## 要解决的问题

商用重构要切入口（`tauri.conf.json` → `neobot-ui/dist`）。切之前必须回答：
**会丢掉什么？** 人工判断不可靠 —— 会重复本仓已犯 3 次的错：
「导出 ≠ 调用」「看起来在用其实没接线」。

## 判据（全部来自实测，不含意见）

对 vendored 前端**每个功能域**，统计它 `invoke` 的命令在
`apps/neobot-desktop/src/api.rs` 契约表里的状态：

| 域的状态 | 含义 | 处置 |
|---|---|---|
| **ALIVE**（全部已实现） | 该功能**现在真的能用** | ⚠️ 切入口会**真丢**功能 ⇒ 必须有自持对应物或显式下架决定 |
| **DEAD**（无任何已实现命令） | 该功能**对着我们的 Rust 后端本来就是死的** | 下它**不丢功能**（它今天就不工作） |
| **MIXED** | 部分可用 | 逐条看，**按命令**而非按域裁决 |

## 「可用」≠「已建」——本门区分三态，而非两态

初版把「命令在契约表里已实现」直接等同于「切入口会丢功能」，**不精确**：
一个域的命令可用，但该功能在自持树里**根本还没建**（如 `layout` 的外壳命令
属里程碑 R2、`i18n` 的 `set_language` 属 R4）—— 那不是「被丢弃」，
是「还没做」。混为一谈会把「待建」也报成「丢失」，门就会恒红。

故本门三态判定：

| 归类 | 含义 | 门行为 |
|---|---|---|
| `DROPPED` | 显式决定下架，附理由 | 放行（可审计） |
| `PENDING` | 该功能**尚未建**，挂里程碑 | 放行，但**列出**以防烂尾 |
| （两者都无） | 活着、无自持对应物、又被漏登记 | ⛔ **门红** |

⇒ 强制的唯一不变量：
> **不允许静默丢弃任何「今天真的能用」的功能** ——
> 「丢弃」指已存在的能力被移除；「尚未建」须以 `PENDING` 显式登记。

## 为什么这个门值钱

它是**后端实现的函数**：哪天我们实现了 `create_profile`，
`ui` 域会从 DEAD/MIXED 变向 ALIVE，本门**立刻**把「不可下架」标出来。
⇒ 功能下架清单不是一次拍板，而是随契约表自动漂移的可审计结论。

退出码：0 = PASS，1 = FAIL。
"""
from __future__ import annotations

import importlib.util
import os
import re
import sys
from typing import Dict, Set

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, '..', '..'))
FRONTEND = os.path.join(REPO, 'apps/neobot-desktop/frontend/src')
SELF_HOSTED = os.path.join(REPO, 'apps/neobot-desktop/neobot-ui/src')
API_RS = os.path.join(REPO, 'apps/neobot-desktop/src/api.rs')

# 显式下架决定（D1–D10）。**只有**写进这里，下架才是「被声明的动作」而非
# 「无人注意的丢失」。每条必须带原因，否则后人会以为漏了。
DROPPED: Dict[str, str] = {
    'store': 'profile/plugin/DSH 服务编排：24/29 个调用指向契约表 Stub/Planned，'
             '对着我方 Rust 后端**今天就不工作** ⇒ 下它不丢功能',
    'ui': 'profile/插件/DSH 管理界面：22/33 个调用指向 Stub/Planned ⇒ 同上',
    'config': '上游 DSH 配置项，我方无对应后端',
    'hooks': '上游数据同步 hook，多为 profile/pet-mouse-stream 等未实现命令',
    # ⛔ 下面这条是**本门逼出来的决策**，不是默认下架：
    # pet 的 4 个命令**全部已实现**（get_pet_asset / get_pet_status /
    # list_preset_pets / move_pet_window）⇒ 切入口会**真丢**一个能用的功能。
    # 决定下架，理由三条（可被审计、可被推翻）：
    #   ① **许可阻断**：实现是 dsh-tauri 上游代码，随受限树一并不可商用分发；
    #      自研重做一个桌面宠物不在商用关键路径上。
    #   ② 纯装饰性：不影响 chat/api-panel/markdown 任一主链路。
    #   ③ 阻塞成本不对称：保留它会阻塞**整个产品**交付。
    # 若日后要恢复：删掉本行即可，门会立刻因 4 个可用命令缺失而转红提醒。
    # ⓘ 这条域是**本门按 md5 判归属后才分出来的**（初版按文件名判，把上游
    #    main.tsx 误认成我方，随后 continue 跳过，掩盖了下面这件事）：
    '(根·上游未迁入)': '上游 vite 入口 frontend/src/main.tsx 本身，M6 随 vendored 树'
                      '**整体删除**。它唯一未覆盖的可用命令 `get_dsh_theme` —— '
                      '我们的上游入口去问「DSH 的主题」，是**DSH 概念泄漏**，'
                      '正是本次商用重构要清掉的东西，不是待补的能力。',
    # 🚩 **冲突标记 —— 这不是决策，是给桌宠功能作者的告示**
    #    本条写于「pet 只有上游实现」之时，理由是「实现属 dsh-tauri 上游、
    #    许可阻断、自研重做不在关键路径」。**该前提已不成立**：
    #    另一窗口已在 `neobot-ui/src/pet/` 提交**自研**桌宠
    #    （commit efe75ded，文件内注释明写「自研（非上游移植）」），
    #    实测其调用 get_pet_asset / get_pet_status，自持树现覆盖 5 个
    #    pet 命令中的 4 个。
    #    ⇒ **本条已过期**，但**由桌宠功能作者自行收敛**（见用户指示
    #      「由他窗自己改」），我**不擅自改**。
    #    ⚠️ 保留此标记的**唯一目的**：防止后来人读到「pet 已下架」而
    #    **删掉那段能跑的自研代码**。要恢复/改判，请由该作者处理。
    'pet': '桌面宠物：4/4 命令可用，但实现属 dsh-tauri 上游（许可阻断），'
           '自研重做不在商用关键路径；纯装饰性，不影响 chat/api-panel/markdown 主链路。'
           '**显式决策**，恢复只需删本行',
}

# 逐命令的「不在本产品范围内」登记 —— **最细粒度**。
#
# 与 DROPPED/PENDING 的区别：
#   DROPPED[域]  = 整个域随 vendored 树删除
#   PENDING[域]  = 该域功能挂里程碑、尚未建
#   NOT_NEEDED[] = **后端已实现**，但本产品**没有它的 UI 场景**
#
# ⛔ 判据纪律：写进这里必须有**站得住的理由**，不能是「暂时没空」。
#    下面每条都指明「为什么这个产品不需要它」或「接了会更糟」。
NOT_NEEDED: Dict[str, str] = {
    # 多窗口：自持 UI 是**单窗口**应用（一个 #root 挂 NeoBotRoot），
    # 无第二个窗口可开。接了只能做出「点了没反应」的按钮。
    'create_app_window': '多窗口：自持 UI 为单窗口架构，无第二窗口可开',
    'remote_open_window': '多窗口：同上',
    # ⛔ 契约表自注：「arboard 未暴露像素读取（image-data feature 未开）
    #    ⇒ **诚实报错**而非返空 data URL」。
    #    ⇒ 命令在、调用必失败。接它 = 交付一个点了就报错的按钮，
    #    比不接更糟（用户会以为功能坏了）。
    'read_clipboard_image': '后端 arboard 未开 image-data feature，调用**必然失败**；'
                            '接它等于交付「点了就报错」的按钮',
    # 以下两条后端确实可用，但当前自持产品（chat + api-panel + 顶栏）
    # **没有对应 UI 场景**。不是「做不到」，是「不在范围内」。
    # 若日后加「导出日志到文件」「有新消息时原生通知」再接线即可。
    'reveal_in_folder': '后端可用，但本产品无「在访达中显示」场景（无文件导出/打开路径 UI）',
    'show_native_notification': '后端可用，但本产品无「原生通知」场景'
                                '（chat 在应用内，无后台提醒需求）',
}

# 「已建但未建」的显式登记 —— 挂里程碑，不挂意愿。
# 语义：该域的命令在契约表里可用，但该功能在自持树**还没实现**（非「被移除」）。
# 删除某行 = 承诺「不再做它」，届时门会因「活着且无自持对应物」而转红。
# R2（外壳）与 R4（i18n）**已实现并经运行时验证**（见 neobot-ui-smoke.mjs：
# 语言切换实测 documentElement.lang zh-CN→en-US 且按钮文案随之改变；
# read_run_logs 弹窗返回真实内容）⇒ 两个域已从 PENDING 移出。
PENDING: Dict[str, str] = {}

# `invoke` 提取正则 —— **刻意复用 nt_api_contract.py 的那一条**。
# 教训（M3）：本仓已两次因自写正则抽不出调用点而误判（一次报 32 条假缺陷、
# 一次各域全 0 调用）。同一事实只允许有一个提取实现。
INVOKE_RE = re.compile(r'invoke(?:<[^>]*>)?\s*\(\s*[\'"]([A-Za-z0-9_]+)[\'"]')

# 归属判据：**md5 内容同一性**，不是「文件名是否存在」。
#
# ⛔ 初版按文件名判，把上游 `frontend/src/main.tsx` 误判成「我方」——
#    因为 `neobot-ui/src/main.tsx` 存在同名文件。二者**同名但内容不同**：
#    上游那个 import OverlaysProvider/QueryClientProvider/ToastProvider，
#    我方那个只 import react + 自持 root。
#    ⇒ 误判 + 随后的 `continue` 跳过，**掩盖了上游入口依赖 get_dsh_theme**。
#    这正是 AGENTS.md 警告的「同名 ≠ 同一符号」。
_MIGRATED: Set[str] = set()


def _md5(path: str) -> str:
    import hashlib
    with open(path, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()


def init_migrated() -> None:
    """记录「已逐字迁入自持树」的文件内容哈希。"""
    _MIGRATED.clear()
    for root, dirs, files in os.walk(SELF_HOSTED):
        dirs[:] = [d for d in dirs if d not in ('node_modules', 'dist', 'vendor')]
        for fn in files:
            if fn.endswith(('.ts', '.tsx')):
                _MIGRATED.add(_md5(os.path.join(root, fn)))


def is_ours(path: str) -> bool:
    """该 vendored 文件是否就是自持树里那份（逐字相同）。"""
    return _md5(path) in _MIGRATED


def load_contract() -> Dict[str, str]:
    """契约表 → 命令 → 状态。

    **优先复用 `nt_api_contract.load_specs()`**（它用括号配平 + 字符串/注释感知
    逐条切分 `ApiSpec::new(...)`，是本仓唯一被验证正确的契约表解析实现）。
    刻意不在此另写一份正则 —— M3 教训：自写正则抽结构化数据已两次误判
    （一次报 32 条假缺陷、一次各域全 0 调用）。
    """
    spec = importlib.util.spec_from_file_location(
        '_contract', os.path.join(HERE, 'nt_api_contract.py'))
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except SystemExit:
        pass  # 该模块顶层会 SystemExit；解析函数此时已定义
    if hasattr(mod, 'load_specs'):
        # 返回 [{'name','group','params','ret','status','note'}, ...]
        return {s['name']: s['status'] for s in mod.load_specs() if s.get('name')}

    # 回退（仅在 nt_api_contract 结构变化时触发）：就地粗解析。
    txt = open(API_RS, encoding='utf-8', errors='ignore').read()
    out: Dict[str, str] = {}
    for m in re.finditer(r'ApiSpec::new\(\s*"([A-Za-z0-9_]+)"(.*?)\)\s*[,;]', txt, re.S):
        body = m.group(2)
        out[m.group(1)] = ('Implemented' if 'Implemented' in body
                           else 'Stub' if 'Stub' in body
                           else 'Planned' if 'Planned' in body else '?')
    return out


def domain_commands() -> Dict[str, Set[str]]:
    out: Dict[str, Set[str]] = {}
    for root, dirs, files in os.walk(FRONTEND):
        dirs[:] = [d for d in dirs if d not in ('node_modules', 'dist', 'vendor')]
        for fn in files:
            if not fn.endswith(('.ts', '.tsx')):
                continue
            p = os.path.join(root, fn)
            rel = os.path.relpath(p, FRONTEND)
            if os.sep in rel:
                dom = rel.split(os.sep)[0]
            else:
                # 根级：只有**内容逐字相同**才算我方已迁入的那份
                dom = '(根·我方已迁入)' if is_ours(p) else '(根·上游未迁入)'
            for m in INVOKE_RE.finditer(open(p, encoding='utf-8', errors='ignore').read()):
                out.setdefault(dom, set()).add(m.group(1))
    return out


def self_hosted_commands() -> Set[str]:
    out: Set[str] = set()
    for root, dirs, files in os.walk(SELF_HOSTED):
        dirs[:] = [d for d in dirs if d not in ('node_modules', 'dist', 'vendor')]
        for fn in files:
            if fn.endswith(('.ts', '.tsx')):
                txt = open(os.path.join(root, fn), encoding='utf-8', errors='ignore').read()
                out |= set(INVOKE_RE.findall(txt))
    return out


def main() -> int:
    contract = load_contract()
    doms = domain_commands()
    init_migrated()
    mine = self_hosted_commands()

    counts = {k: sum(1 for v in contract.values() if v == k)
              for k in ('Implemented', 'Planned', 'Stub')}
    print('功能存活性门 —— 下架决策由契约表推导，不由意见决定\n')
    print(f"契约表 {len(contract)} 条：Implemented={counts['Implemented']} "
          f"Planned={counts['Planned']} Stub={counts['Stub']}")
    print(f"自持树已实现命令 {len(mine)} 个\n")

    if not contract:
        print('⛔ 契约表解析为空 —— 判据失效，拒绝给结论')
        return 1

    fail: list[str] = []
    print(f'{"功能域":<16}{"调用":>5}{"已实现":>7}{"死":>5}  状态        处置')
    print('-' * 78)
    for dom in sorted(doms, key=lambda d: (-len(doms[d]), d)):
        cmds = doms[dom]
        alive = {c for c in cmds if contract.get(c) == 'Implemented'}
        dead = {c for c in cmds if contract.get(c) in ('Stub', 'Planned')}
        unknown = set(cmds) - set(contract)

        if not alive:
            state = 'DEAD'
        elif dead or unknown:
            state = 'MIXED'
        else:
            state = 'ALIVE'

        # 显示必须与判定一致：初版对**已全覆盖**的域也印「会真丢」，
        # 属自己输出里的假警报，会让人不信任这张表。
        if state == 'DEAD':
            act = f'下架无损失（{len(dead)}/{len(cmds)} 调用本就失败）'
        elif not (alive - mine):
            act = '✅ 可用命令已全部被自持树覆盖'
        elif state == 'MIXED':
            act = f'部分覆盖：缺 {len(alive - mine)}/{len(alive)} 个可用命令'
        else:
            act = f'⚠️ 全可用且未覆盖（缺 {len(alive - mine)}）⇒ 切入口会真丢'
        print(f'{dom:<16}{len(cmds):>5}{len(alive):>7}{len(dead):>5}  {state:<11}  {act}')

        # ── 唯一不变量：不允许静默丢弃「今天真能用」的功能 ──
        if state in ('ALIVE', 'MIXED') and dom not in DROPPED and dom not in PENDING:
            covered = alive & mine
            # 逐命令豁免：NOT_NEEDED 里登记过的**不算「丢失」**
            missing = alive - mine - set(NOT_NEEDED)
            if dom == '(根·我方已迁入)':
                continue  # 我方自持代码本身，逐字已进自持树
            if missing:
                fail.append(
                    f'域 {dom!r} 有 {len(missing)} 个**今天可用**的命令，'
                    f'自持树无对应物，且未在 DROPPED 显式下架\n'
                    f'       缺失命令（例）：{", ".join(sorted(missing)[:6])}\n'
                    f'       处置二选一：① 在自持树实现其中之一（哪怕占位可达）\n'
                    f'                 ② 把 {dom!r} 写进 DROPPED 并写明原因'
                    f'\n       ⛔ 静默丢弃「真能用」的功能不允许'
                )
    print()
    for dom, why in sorted(DROPPED.items()):
        if dom in doms:
            n = len(doms[dom])
            a = len([c for c in doms[dom] if contract.get(c) == 'Implemented'])
            print(f'ℹ️  已声明下架 {dom}（{a}/{n} 可用）：{why}')
    for dom, why in sorted(PENDING.items()):
        if dom in doms:
            n = len(doms[dom])
            a = len([c for c in doms[dom] if contract.get(c) == 'Implemented'])
            print(f'⏳ 待建 {dom}（{a}/{n} 可用，尚未实现而非被移除）：{why}')
    if NOT_NEEDED:
        print(f'\n逐命令「不在本产品范围」{len(NOT_NEEDED)} 条（后端已实现但无 UI 场景）：')
        for cmd, why in sorted(NOT_NEEDED.items()):
            st = contract.get(cmd, '?')
            print(f'  · {cmd} [{st}] —— {why}')

    if fail:
        print(f'\nFAIL: {len(fail)} 项')
        for f in fail:
            print(f'  ⛔ {f}')
        return 1
    print('\nPASS: 无「今天真能用」的功能被静默丢弃；'
          '下架项与待建项均已显式登记且有据。')
    return 0


if __name__ == '__main__':
    sys.exit(main())
