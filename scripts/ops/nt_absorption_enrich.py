#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""台账元数据补全器 —— 把 493 条「只有名字」变成「可评估的项目」。

# 为什么必须做

实测 `docs/architecture/absorption-sources/repos.csv`（2026-10-01）：

| 指标 | 值 |
|---|---|
| 条目 | 493 |
| ★=0 **且** language 为空 | **390（79%）** |
| ★≥100 | 90 |
| **有 license 字段** | **0** |

两个后果，都很硬：

1. **79% 的条目根本无法评估** —— 没星数、没语言，等于只有名字。
   谈「吸收」无从谈起。
2. ⛔ **没有 license 字段** —— 而本项目正走在**商用**路径上
   （见 `docs/architecture/FRONTEND-REBUILD-2026-10-01.md`）。
   **license 恰恰决定一段代码能不能被吸收商用。**
   我们的商用重构刚被 dsh-tauri 的附加条款卡了一整轮，
   而台账里另外 492 个项目的许可状况**一条未知**。

⇒ 本工具补 `stars / language / license / archived / pushed_at / topics`，
   让「能不能吸收」变成**可查的事实**而不是猜测。

# 配额现实（必须诚实）

GitHub 未认证 API = **60 次/小时**。493 条 > 60 ⇒：
· **不可能一次跑完**，本工具按配额自觉停，并**报告覆盖度**；
· 结果**落盘缓存**（`.neotrix/absorption-cache.json`），下次**增量**续跑；
· 已实测本轮配额为 **0**，故本轮只能交付工具、**数据覆盖度如实报 0**。

# 用法

    python3 scripts/ops/nt_absorption_enrich.py            # 按配额能抓多少抓多少
    python3 scripts/ops/nt_absorption_enrich.py --limit 20  # 只抓 20 条（试跑）
    python3 scripts/ops/nt_absorption_enrich.py --report   # 只出报告，不发请求

退出码：0 = 完成（可能部分覆盖，见报告）；2 = 配额耗尽且无新增。
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import sys
import time
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, '..', '..'))
LEDGER = os.path.join(REPO, 'docs/architecture/absorption-sources/repos.csv')
CACHE = os.path.join(REPO, '.neotrix/absorption-cache.json')
UA = 'neotrix-absorption-enrich/1.0'


def load_cache() -> dict:
    if os.path.isfile(CACHE):
        try:
            with open(CACHE, encoding='utf-8') as f:
                return json.load(f)
        except (json.JSONDecodeError, OSError):
            pass
    return {}


def save_cache(c: dict) -> None:
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    with open(CACHE, 'w', encoding='utf-8') as f:
        json.dump(c, f, ensure_ascii=False, indent=1, sort_keys=True)
        f.write('\n')


def rate_limit() -> tuple[int, int]:
    """返回 (remaining, limit)。取不到就报 (0, 0) —— 不猜。"""
    try:
        req = urllib.request.Request('https://api.github.com/rate_limit', headers={'User-Agent': UA})
        with urllib.request.urlopen(req, timeout=15) as r:
            core = json.load(r)['resources']['core']
            return int(core['remaining']), int(core['limit'])
    except Exception:
        return 0, 0


def fetch(slug: str) -> dict | None:
    """取单个仓库元数据。404/无 LICENSE 返回 None。"""
    url = f'https://api.github.com/repos/{slug}'
    req = urllib.request.Request(url, headers={'User-Agent': UA, 'Accept': 'application/vnd.github+json'})
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            d = json.load(r)
    except urllib.error.HTTPError as e:
        return {'_error': f'HTTP {e.code}'}
    except Exception as e:  # noqa: BLE001
        return {'_error': type(e).__name__}
    lic = (d.get('license') or {}).get('spdx_id')
    return {
        'stars': d.get('stargazers_count'),
        'language': d.get('language'),
        # ⛔ 没有 license 就明确记 None —— **不猜、不用「未知」糊过去**。
        #    「未知」与「无限制」在商用决策里是天壤之别。
        'license': lic,
        'archived': bool(d.get('archived')),
        'pushed_at': d.get('pushed_at'),
        'topics': (d.get('topics') or [])[:8],
        'description': (d.get('description') or '')[:160],
    }


# ⛔ 商用吸收必须警惕的许可（不构成法律意见，是**人工复核触发器**）
# ⛔⛔ **必须小写归一化** —— 这是我自己的工具里一个**危险假阴性**：
#   GitHub API 返回**大写** SPDX（'MIT' / 'Apache-2.0' / 'AGPL-3.0'），
#   而我初版集合用小写 ⇒ 三个分类**全部恒为 0**，报告遂称
#   「宽松 0 / 需复核 0」——**其中实际藏着一个 AGPL-3.0（copyleft，商用有风险）**。
#   ⇒ 假阴性的代价：让不能商用的代码被当成可商用吸收。
#   ⇒ 教训：枚举外部取值比较时，**归一化大小写**不是洁癖，是正确性。
REVIEW_LICENSES = {'agpl-3.0', 'agpl-3.0-only', 'sspl-1.0', 'sspl-1.0.txt',
                   'cc-by-nc-4.0', 'cc-by-nc-sa-4.0', 'cc-by-nc-3.0', 'cc-by-nc-2.0',
                   'prosperity-public-license-3.0.0', 'bsl-1.0', 'elastic-2.0',
                   'epl-2.0', 'eupl-1.2', 'osl-3.0', 'cc-by-sa-4.0',
                   # GitHub 无法判定 ⇒ **按需复核处理**，绝不当作「无限制」
                   'noassertion', 'other', 'null',
                   # CC-BY 可商用但**须署名** ⇒ 不算宽松，归人工复核
                   'cc-by-4.0', 'cc-by-3.0', 'cc-by-2.0', 'cc-by-sa-3.0'}
PERMISSIVE = {'mit', 'apache-2.0', 'bsd-2-clause', 'bsd-3-clause', 'isc',
              'unlicense', '0bsd', 'mit-0', 'apache-2.0-with-llvm-exception'}


def norm_lic(v):
    return (v or '').strip().lower() or 'noassertion'


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--limit', type=int, default=0, help='本次最多抓几条（0=按配额尽量抓）')
    ap.add_argument('--report', action='store_true', help='只出报告，不发任何请求')
    args = ap.parse_args()

    with open(LEDGER, encoding='utf-8') as f:
        rows = list(csv.DictReader(f))
    cache = load_cache()

    # 优先级：user-list 优先（用户明确给的，最可能被吸收），
    # 其次按星数降序（星数高的先抓，因为它们最值得判 license）。
    # ⛔ 不按台账原序 —— 原序会把 375 条 trendshift 噪声排前面，
    #    配额宝贵时先抓它们等于浪费。
    def _prio(r):
        st = r.get('stars') or ''
        try:
            st = int(st)
        except ValueError:
            st = 0
        user_first = 0 if r['source'].startswith('user') else 1
        return (user_first, -st)
    todo = [r['repo'] for r in sorted(rows, key=_prio) if r['repo'] not in cache]
    covered = len(rows) - len(todo)

    print('台账元数据补全器\n')
    print(f'  台账 {len(rows)} 条 · 已缓存 {covered} 条 · 待抓 {len(todo)} 条')

    # 现状报告（不依赖网络）
    lic_known = sum(1 for c in cache.values() if isinstance(c, dict) and c.get('license'))
    print(f'  **有 license 记录：{lic_known}/{len(rows)}**'
          f'（0 = 商用吸收无法判定，本工具存在的理由）')
    dead = sum(1 for c in cache.values() if isinstance(c, dict) and c.get('_error') == 'HTTP 404')
    print(f'  已缓存中 404（仓库不存在/改名）：{dead}')

    rem, lim = rate_limit()
    print(f'  GitHub 配额：剩余 {rem}/{lim} 次/小时'
          + ('　⇒ **本轮无法抓取**（未认证上限 60/h）' if rem == 0 else ''))

    if args.report:
        print('\n--report 模式：不发请求。')
        return 0

    budget = min(rem - 1 if rem else 0, args.limit) if args.limit else max(0, rem - 1)
    if budget <= 0:
        print('\n⛔ 配额不足，本次未抓取任何条目。')
        print('   工具已就绪，**可随时增量续跑**（结果落盘缓存）：')
        print(f'   {os.path.relpath(CACHE, REPO)}')
        print('   覆盖全部 493 条需 ~8 小时（60/h）或配置 GITHUB_TOKEN 后数分钟。')
        print('   ⛔ 本工具**不会**去寻找或使用未授权凭证。')
        return 2

    print(f'\n  本次预算 {budget} 条，开始抓取…')
    done = 0
    for slug in todo[:budget]:
        cache[slug] = fetch(slug)
        done += 1
        if done % 20 == 0:
            save_cache(cache)
            r2, _ = rate_limit()
            print(f'    …{done}/{budget}（配额剩 {r2}）')
        time.sleep(0.4)  # 礼貌限速
    save_cache(cache)
    print(f'  已抓 {done} 条并落盘。')

    lic = [c for c in cache.values() if isinstance(c, dict) and c.get('license')]
    # ⛔⛔ 必须走 `norm_lic()`：GitHub 返回**大写** SPDX（'MIT'/'AGPL-3.0'），
    #   直接用 `in REVIEW_LICENSES`（小写集合）比较 ⇒ **恒为 0**。
    #   ⓘ 我"修"过这个问题一次，但**只加了 norm_lic 函数、报告段的替换没落盘**
    #   （Python 字符串 replace 的锚点没匹配），而我没验证 ⇒ 工具继续报假数据。
    #   ⇒ 这也是本项目第 3 次栽在「字符串替换未落盘却以为成功」。
    #   ⇒ 对策：**每次改完必须比对输出与原始分布**，不能只看 rc。
    # ⛔⛔ 必须显式跳过 `_error`（404 / 网络失败）条目：
    #   `norm_lic(None)` 会返回 'noassertion' 落进 REVIEW ⇒ 把
    #   「**从未取到数据**」与「**取到了、GitHub 判不出**」混为一谈。
    #   后者更有误导性：它让人以为「查过了，结论是判不出」，
    #   而真相是「根本没查成」。这是我自检刚抓出来的。
    ok = {s: c for s, c in cache.items() if isinstance(c, dict) and not c.get('_error')}
    failed = [s for s, c in cache.items() if isinstance(c, dict) and c.get('_error')]
    review = [s for s, c in ok.items()
              if norm_lic(c.get('license')) in REVIEW_LICENSES]
    permissive = [s for s, c in ok.items()
                  if norm_lic(c.get('license')) in PERMISSIVE]
    nolic = [s for s, c in ok.items() if not c.get('license')]
    print(f'\n  覆盖：{len(cache)}/{len(rows)} 条已抓 · 有 license {len(lic)}')
    print(f'  🟢 宽松许可（可商用吸收，**仅初筛**）：{len(permissive)}')
    print(f'  🔴 需**人工复核**（copyleft / 源码可见 / GitHub 判不出）：{len(review)}')
    for s in review[:12]:
        print(f'     · {s} — {cache[s].get("license")}')
    print(f'  ⚪ 无 license 字段（须人工查 LICENSE 文件）：{len(nolic)}')
    print(f'  ⚫ 抓取失败（**无数据**，非「判不出」）：{len(failed)}')
    for s in failed[:8]:
        print(f'     · {s} — {cache[s].get("_error")}')

    # ── 自检：分类数必须与原始分布对得上，否则说明分类逻辑又坏了 ──
    import collections
    raw = collections.Counter(
        (v.get('license') or '').strip().lower() for v in cache.values()
        if isinstance(v, dict))
    unclassified = sum(n for k, n in raw.items()
                       if k and k not in REVIEW_LICENSES and k not in PERMISSIVE)
    print(f'\n  ℹ️ 原始分布 {dict(raw)}')
    print(f'  ℹ️ 未归类 {unclassified} 种'
          + ('（⚠️ 应为 0，否则 PERMISSIVE/REVIEW 集合漏了取值）'
             if unclassified else ' ✅'))
    if unclassified:
        print('     ⇒ 这些 license 既不在「宽松」也不在「需复核」里 ⇒ **被静默漏过**')
        for k in raw:
            if k and k not in REVIEW_LICENSES and k not in PERMISSIVE:
                print(f'       · {k}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
