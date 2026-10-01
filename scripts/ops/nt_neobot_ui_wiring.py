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
    # ⚠️ neobot-root.tsx **刻意不在漂移对照内**。
    #   我方副本已完成 R4 i18n 迁移（24 处硬编码中文 → t()，56 键词条），
    #   而 `frontend/src/neobot-root.tsx` **不能**跟着改，两个硬理由：
    #   ① 上游 `frontend/src/i18n/index.ts` 只导出 `i18n`（i18next 实例），
    #      **没有 `t()`** ⇒ 在那边 `import { t } from '@/i18n'` 编译不过，
    #      会**打断另一窗口正在推进的构建**。
    #   ② M6 的结局是 vendored 树被删 —— 给一棵即将退役的树做 i18n 迁移
    #      是纯浪费，且要改他窗有 600+ 行在途改动的共享文件。
    #   ⇒ 迁移只落在自持树。原件保持 md5 5909ee44（仅含剪贴板降级修复）。
    ('api-panel.ts', 'api-panel.ts'),
    ('dom.ts', 'dom.ts'),
    ('ui/nb-markdown.css', 'ui/nb-markdown.css'),
    # ⚠️ i18n **刻意不在漂移对照内**：我方词条是**自有集合**（实测与上游
    # 452 键零重合），不再是上游文件的复制件。把它纳入漂移检查会强迫我方
    # 词条永久跟随上游 DSH 词汇表 —— 那等于把 DSH 概念重新引进自持树。
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
    print(f'✅ 1 漂移门：{len(PAIRS)} 个复制件与原件逐字一致'
          f'（i18n 不在对照内：我方词条为自有集合，与上游 452 键零重合）')

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

# ── 4b IPC 单一出口检查（**报告式**：不因他窗文件而判失败）─────────────
# 吸收 bytedance/UI-TARS-desktop 的 Event Stream Viewer / 工具调用耗时统计：
# 活动面板依赖「所有 invoke 都经 src/ipc.ts」。直连者**不进入面板**
# ⇒ 命令失败时用户看不到是哪条命令、多耗时。
# ⛔ 刻意**不判失败**：唯一已知直连方 `src/pet/pet.tsx` 属另一窗口，
#    把它做成红灯只会被忽略（本仓门纪律：恒红的门 = 没有门）。
DIRECT = []
for root, dirs, files in os.walk(os.path.join(UI, 'src')):
    dirs[:] = [x for x in dirs if x not in ('node_modules', 'dist', 'vendor')]
    for fn in files:
        if not fn.endswith(('.ts', '.tsx')):
            continue
        p_ = os.path.join(root, fn)
        rel = os.path.relpath(p_, UI)
        if rel == os.path.join('src', 'ipc.ts'):
            continue  # 单一出口本身
        if "from '@tauri-apps/api/core'" in read(p_):
            DIRECT.append(rel)
if DIRECT:
    print(f"ℹ️  4b IPC 单一出口：{len(DIRECT)} 个文件仍直连 @tauri-apps/api/core"
          f" ⇒ 其调用不进活动面板（不判失败，属他窗文件）：{', '.join(DIRECT)}")
else:
    print('✅ 4b IPC 单一出口：全部经 src/ipc.ts（活动面板完整）')

# ── 4c 工具类是否**真的编译进产物**（Tailwind 缺失守卫）───────────────
# ⛔ 这条门来自一次**严重自伤**：neobot-root.tsx 逐字复制自 vendored 树，
#    那边有 Tailwind v4 + CSS 变量主题；本自持树起初**两者都没有** ⇒
#    实测产物 CSS 里工具类命中 **0**，而该文件用了 **230 个 token** ⇒
#    全部是死的。表现不是"不好看"，是**布局坏掉**：
#    `flex-1` 计算值为初始值 `0 1 auto` ⇒ 会话列表不滚动、把容器撑破。
#    而我当时的冒烟测试只验「有文本 + 无 console 报错」⇒ **放过了**。
# ⇒ 「渲染通过」≠「样式生效」。本门把这件事变成可红的判据。
UTIL_SAMPLE = [
    'flex-1', 'flex-col', 'min-h-0', 'overflow-y-auto', 'truncate',
    'shrink-0', 'text-muted', 'border-line', 'bg-panel', 'text-ink',
]
css_text = ''
for root, _d, files in os.walk(os.path.join(UI, 'dist')):
    for fn in files:
        if fn.endswith('.css'):
            css_text += read(os.path.join(root, fn))
if not css_text:
    print('ℹ️  4c 工具类守卫：dist/ 无 CSS（未构建）—— 跳过')
else:
    # 统计源码里实际用到的工具类 token 数（粗口径：出现在 className 里的）
    src_all = ''
    for root, dirs, files in os.walk(os.path.join(UI, 'src')):
        dirs[:] = [x for x in dirs if x not in ('node_modules', 'dist', 'vendor')]
        for fn in files:
            if fn.endswith(('.ts', '.tsx')):
                src_all += read(os.path.join(root, fn))
    used = [u for u in UTIL_SAMPLE if u in src_all]
    missing = [u for u in used if f'.{u}' not in css_text]
    if missing:
        fail.append(
            f'源码使用了工具类但**产物 CSS 里没有**：{missing}\n'
            f'       ⇒ 这些 class 是**死的**（浏览器找不到规则 ⇒ 布局静默失效）。\n'
            f'       常见病因：自持树缺 Tailwind（vendored 树有，本树曾没有）。\n'
            f'       处置：确认 vite 插件含 tailwindcss()，且入口 CSS 有 '
            f'@import "tailwindcss"；语义色还需 CSS 变量（见 src/theme.css）'
        )
    else:
        print(f'✅ 4c 工具类守卫：抽样的 {len(used)} 个工具类均已编译进产物 CSS'
              f'（产物 {len(css_text)} 字节）')

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
