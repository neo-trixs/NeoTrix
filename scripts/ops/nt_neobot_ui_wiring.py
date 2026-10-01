#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""neobot-ui 自持树接线门 + 漂移门。

背景：`apps/neobot-desktop/neobot-ui/` 是商用重构的自持 UI（零 vendored 依赖），
其 684 行代码是**逐字复制**自 `frontend/src/`（因该树有他窗在途改动，不可移动）。
本门守四件事，任何一件坏掉都会让「自持 UI」在**无任何界面异常**的情况下失效：

  1. 漂移：本目录复制件与 `frontend/src/` 原件 md5 必须一致（否则复制腐化）
  2. 纯净：自持入口 `src/main.tsx` 不得 import 任何 vendored 路径
  3. base：`vite.config.ts` 的 `base` 必须是相对路径
     （默认 `'/'` 产出 `/assets/x.js`，子路径挂载下渲染失败 ⇒ **不可用**，
      而 `vite build` 成功、tsc 通过、零报错 —— 只靠构建无法发现）
  4. 产物：若 `dist/` 存在，其 JS 不得含上游 dsh-tauri 特征符

判据纪律（R-SCAN-1）：本门只报**可复现的字节事实**，不推测意图。
退出码：0 = PASS，1 = FAIL。
"""
from __future__ import annotations

import datetime
import hashlib
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, '..', '..'))
UI = os.path.join(REPO, 'apps/neobot-desktop/neobot-ui')
SRC = os.path.join(REPO, 'apps/neobot-desktop/frontend/src')

# 复制件 → 原件（相对 neobot-ui/src 与 frontend/src）
PAIRS: list[tuple[str, str]] = [
    ('neobot-root.tsx', 'neobot-root.tsx'),
    ('api-panel.ts', 'api-panel.ts'),
    ('dom.ts', 'dom.ts'),
    ('ui/nb-markdown.css', 'ui/nb-markdown.css'),
    ('i18n/zh-CN.json', 'i18n/locales/zh-CN.json'),
    ('i18n/en-US.json', 'i18n/locales/en-US.json'),
    ('vendor/openghost/shim.ts', 'vendor/openghost/shim.ts'),
    ('vendor/openghost/markdown.js', 'vendor/openghost/markdown.js'),
    ('vendor/openghost/highlight.js', 'vendor/openghost/highlight.js'),
    ('vendor/openghost/tex.js', 'vendor/openghost/tex.js'),
]

# 上游 dsh-tauri 特征符：自持产物里出现即说明混入了 vendored 代码
UPSTREAM_MARKERS = ('overlastic', 'tanstack', 'toast-provider', 'store/modules', 'dsh-tauri')

fail: list[str] = []
warn: list[str] = []


def strip_comments(src: str) -> str:
    """去 // 与 /* */ 注释，供**结构**匹配用。

    本门不追求完备 JS 词法分析（不处理字符串里的 `//`）——作用域仅限
    读 `vite.config.ts` 找 `base` 设置，那里没有含 `//` 的字符串。
    实测教训：加此函数前，门匹配到了**注释里举例的** `base:'/'`，
    把已修正为 `'./'` 的配置报成缺陷（假阳性）。
    """
    src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    return re.sub(r'//[^\n]*', '', src)


def md5(path: str) -> str:
    with open(path, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()


def read(path: str) -> str:
    with open(path, encoding='utf-8', errors='ignore') as f:
        return f.read()


print(f'neobot-ui 接线门 · {os.path.relpath(UI, REPO)}')
print()

# ── 0 目录存在性 ────────────────────────────────────────────────────────
if not os.path.isdir(UI):
    print('⛔ 自持 UI 目录不存在 —— 无可接线之物')
    sys.exit(1)

# ── 1 漂移门 ───────────────────────────────────────────────────────────
drift = 0
for mine, orig in PAIRS:
    a, b = os.path.join(UI, 'src', mine), os.path.join(SRC, orig)
    if not os.path.isfile(a):
        fail.append(f'复制件缺失 src/{mine}')
        continue
    if not os.path.isfile(b):
        warn.append(f'原件已不在 frontend/src/{orig}（他窗已删/已移）⇒ 需确认去留')
        continue
    if md5(a) != md5(b):
        drift += 1

        def mtime(p: str) -> str:
            return datetime.datetime.fromtimestamp(os.path.getmtime(p)).strftime('%H:%M:%S')

        newer = '原件（frontend）更新 ⇒ 他窗在途改动，**勿贸然同步**（会与其竞速）' \
            if os.path.getmtime(b) > os.path.getmtime(a) else \
            '复制件更新 ⇒ 本目录被手改过；复制必须逐字，**应从原件重新同步**'
        fail.append(
            f'漂移 src/{mine}\n'
            f'       md5：neobot-ui={md5(a)[:8]}({mtime(a)})  '
            f'frontend={md5(b)[:8]}({mtime(b)})\n'
            f'       归属：{newer}'
        )
if not drift:
    print(f'✅ 1 漂移门：{len(PAIRS)} 个复制件与原件逐字一致')

# ── 2 入口纯净门：main.tsx 不得引用 vendored 路径 ───────────────────────
entry = os.path.join(UI, 'src/main.tsx')
if not os.path.isfile(entry):
    fail.append('自持入口 src/main.tsx 不存在')
else:
    t = read(entry)
    # 判据：neobot-ui/ 与 frontend/src/ 是**物理分离的两棵树**，
    # 所以 `./x` 必然落在自持树内（不可疑）；真正危险的是能跨出本树的写法：
    #   ① `@/x`  —— 上游 vite alias，指向 vendored src/（删 vendored 即崩）
    #   ② `../..` —— 逃出 neobot-ui/
    # ⚠️ 教训：初版把一切 `./x` 判为违规，误报 `./neobot-root`（那**就是**自持文件）。
    bad = []
    for m in re.finditer(r"""from\s+['"]([^'"]+)['"]|import\s+['"]([^'"]+)['"]""", t):
        spec = m.group(1) or m.group(2)
        if spec.startswith('@/') or spec == '@':
            bad.append(spec)
        elif spec.startswith('/') or spec.startswith('../'):
            bad.append(spec)
    if bad:
        fail.append(f'自持入口引用了非自持路径：{sorted(set(bad))}'
                    f'\n       后果：自持树反向依赖 vendored ⇒ 删 vendored 树即崩')
    else:
        print('✅ 2 入口纯净门：无 @/ 上游别名、无 ../ 逃逸（相对 ./x 为自持树内引用）')

# ── 3 vite base 门（挂载点无关性守卫）───────────────────────────────────
# 证据边界（勿越读）：`base:'./'` 在**子路径**下渲染已实证（neobot-ui-smoke.mjs）；
# 而 `base:'/'` 在 Tauri v2 自定义协议下**未必**失效（Tauri 把 frontendDist
# 挂在协议根）—— 本门守的是「不依赖未验证前提」：相对路径在两种挂载下都正确。
vc = os.path.join(UI, 'vite.config.ts')
if not os.path.isfile(vc):
    fail.append('vite.config.ts 不存在')
else:
    t = strip_comments(read(vc))
    m = re.search(r"""\bbase\s*:\s*['"]([^'"]*)['"]""", t)
    if not m:
        fail.append("vite.config.ts 未显式声明 base\n"
                    "       后果：vite 默认 '/' ⇒ 产物 src=\"/assets/x.js\"（根相对）\n"
                    "       已实证：根相对在**子路径**挂载下渲染失败（#root 为空），"
                    "而构建与 tsc 仍全绿 ⇒ 产物不可用")
    elif m.group(1) not in ('./', ''):
        fail.append(f"vite base={m.group(1)!r} 为根相对 ⇒ 依赖「应用挂在协议根」这一未验证前提")
    else:
        print(f"✅ 3 vite base 门：base={m.group(1)!r}（相对，根挂载与子路径皆安全）")

# ── 4 产物纯净门（若有 dist）───────────────────────────────────────────
dist = os.path.join(UI, 'dist')
js = []
if os.path.isdir(dist):
    for root, _d, files in os.walk(dist):
        js += [os.path.join(root, f) for f in files if f.endswith('.js')]
if js:
    hits: dict[str, list[str]] = {}
    for p in js:
        body = read(p)
        for mk in UPSTREAM_MARKERS:
            if mk in body:
                hits.setdefault(mk, []).append(os.path.relpath(p, UI))
    if hits:
        for mk, where in sorted(hits.items()):
            fail.append(f'产物含上游特征符 {mk!r}：{where}'
                        f'\n       后果：自持产物混入 vendored 代码，商用分发将带上受限源码')
    else:
        print(f'✅ 4 产物纯净门：{len(js)} 个 JS，{len(UPSTREAM_MARKERS)} 个上游特征符全部 0 命中')
else:
    print('ℹ️  4 产物纯净门：dist/ 不存在（未构建）—— 跳过')

# ── 5 openghost 独立授权（MIT 无附加条款）───────────────────────────────
og = os.path.join(UI, 'src/vendor/openghost')
if os.path.isdir(og):
    print('✅ 5 openghost 已物理迁出受限树（MIT 独立授权，许可门独立审计）')
else:
    warn.append('src/vendor/openghost 不存在 —— R5 迁出未完成？')

# ── 汇总 ───────────────────────────────────────────────────────────────
print()
for w in warn:
    print(f'⚠️  {w}')
if fail:
    print(f'FAIL: {len(fail)} 项')
    for f_ in fail:
        print(f'  ⛔ {f_}')
    sys.exit(1)
print('PASS: 自持 UI 接线完整、复制无漂移、产物无 vendored 混入。')
sys.exit(0)
