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
    # ⚠️ api-panel.ts **刻意不在漂移对照内**（与 neobot-root.tsx 同理）。
    #   我方副本已完成 R4 i18n 迁移（用户可见文案 8 处 → t()，8 个新键），
    #   且已改走 src/ipc.ts 单一出口。`frontend/src/api-panel.ts` **不能**跟着改：
    #   ① 它 import 的是 `@/i18n`（上游模块，只导出 i18next 实例、无 `t()`），
    #      跟着改会**打断另一窗口的构建**；
    #   ② M6 结局是 vendored 树被删，给即将退役的树做迁移是纯浪费。
    #   ⇒ 迁移只落自持树。此条曾被漂移门正确报出（md5 不同），
    #     故在此登记为**有意分叉**，不是静默漂移。
    ('dom.ts', 'dom.ts'),
    # ⚠️ `ui/nb-markdown.css`：**锚点已改到自持树**（2026-10-03 裁决，见 ANCHORED）。
    #   实测差异**只有 2 行**：`.md-code-lang` 与 `.md-copy` 的
    #   `font-size: 11px → 12px`。
    #   ⭐⭐ 裁决依据（实测，非品味）：自持树 `src/` 的 font-size 分布是
    #   **12px × 9 / 13px × 1 / 11px × 0** ⇒ ⭐ **11px 是离群值**，
    #   副本的 12px **对齐设计系统** ⇒ **副本是修正，不是回归**。
    #   ⇒ ⭐ **从原件重新同步反而会引入离群值**（那正是门建议的动作 ⛔ 不做）。
    #   ⛔ 刻意**不**把修改推到 `frontend/src/`：该树是**他窗在途**资产
    #   （门自己写着「勿贸然同步（会与其竞速）」），且「M6 结局是 vendored 树被删」
    #   ⇒ ⭐ 给即将退役的树做同步是纯浪费。
    #   ⓘ 时间证据：副本 2026-10-02 20:11 **新于** 原件 2026-09-30 21:21（**2 天**）
    #   ⇒ 门判「复制件更新 ⇒ 应从原件重新同步」在**归属上是对的**，
    #   ⛔ 但**建议本身在这里是错的**（会退回离群值）。
    #   ⇒ 故登记为有意分叉，保留该文件在对照内（门继续盯其它可能的漂移）。
    ('ui/nb-markdown.css', 'ui/nb-markdown.css'),
    # ⚠️ i18n **刻意不在漂移对照内**：我方词条是**自有集合**（实测与上游
    # 452 键零重合），不再是上游文件的复制件。把它纳入漂移检查会强迫我方
    # 词条永久跟随上游 DSH 词汇表 —— 那等于把 DSH 概念重新引进自持树。
    # ⚠️ vendor/openghost/shim.ts **刻意移出漂移对照**（2026-10-01）。
    #   理由与上方 neobot-root.tsx / api-panel.ts 同源，但有一条**更硬**：
    #   ① **shim.ts 是我们自己的代码，不是上游复制件。** 上游 md5 锁只锁
    #      markdown.js / highlight.js / tex.js 三个文件（见
    #      neobot-ui/src/vendor/openghost/VENDOR-OPENGHOST.md 的 md5 表）；
    #      shim.ts 是为「补两个全局」自写的垫片，本就该随宿主演进。
    #   ② 我在自持树修了它的**真缺陷**：`lang()` 的回退链
    #      `documentElement.lang 落空 → navigator.language` 会**覆盖宿主
    #      显式设置**，导致「应用选英文 + 系统中文」时代码块复制按钮永远中文。
    #      修在原件上治不好自持树（交付物是自持树），而原件即将退役。
    #   ⇒ 三个**上游**文件仍在对照内（字节锁 + 许可边界），门的牙齿
    #      留在真正需要它的地方；本次移动**不放松**上游文件的约束。
    #   ⓘ 负向测试：把 shim.ts 的修复撤掉，neobot-check-markdown 门 rc=1。
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


# ── ⭐ 锚点覆盖：PAIRS 里仍有该文件，但比对基准从「原件」改为「自持树的期望 md5」──
# ⭐ 理由：把文件移出 PAIRS（既有先例）会**丢掉这颗牙**；改锚点则**牙还在**
# —— 任何人再改这个文件都会被抓，而**刻意的那 2 行修正**不会误报。
# ⭐ 裁决依据（实测，非品味）：自持树 `src/` 的 font-size 分布为
# **12px × 9 / 13px × 1 / 11px × 0** ⇒ **11px 是离群值**，自持树的 12px
# **对齐设计系统** ⇒ ⭐ **从原件重新同步反而会引入离群值**
# ⇒ ⭐ 故同步**方向**是错的。⛔ 刻意**不**推到 `frontend/src/`（他窗在途，
# 且「M6 结局是 vendored 树被删」）。
ANCHORED: dict[str, str] = {
    # 上面的登记块记录了完整裁决与时间证据
    'ui/nb-markdown.css': '9c0b5ae2179ee59f50471219b906b165',
}


def anchor_verdict(expect: str, got: str) -> str | None:
    """⭐ 锚点比对（**纯函数**：只吃两个 md5 字符串 ⇒ 可用内联样本自测）。

    返回 `None` = 一致；返回字符串 = 漂移说明（含 file:line 级可读性）。
    """
    if got.startswith(expect):
        return None
    return (
        f'锚点漂移（基准=自持树期望值）\n'
        f'       期望前缀={expect[:8]}  实际={got[:8]}\n'
        f'       ⇒ 该文件已登记为「有意分叉」并锚定自持树期望值；\n'
        f'         任何人再改动它都会被抓 ⇒ 若改动是有意的，请同步更新 ANCHORED。'
    )


def selftest() -> int:
    """⭐⭐ 内联样本自测（**不读仓库**，对齐 `nt_docclaims.py` 的既有范式）。

    ⭐ 覆盖：绿 / 红（单字符改动）/ 红（完全不同）/ 空前缀边界。
    ⭐⭐ **变异方向必须与被测性质相关**：本门度量「字节漂移」
    ⇒ 变异**必须改字节**，⛔ 不是改期望值（改期望值是「更新基线」，不是漂移）。
    """
    EXPECT = '9c0b5ae2179ee59f50471219b906b165'
    cases: list[tuple[str, bool]] = [
        ('锚点绿（完全一致）', anchor_verdict(EXPECT, EXPECT) is None),
        ('锚点绿（前缀一致+后续不同）',
         anchor_verdict(EXPECT[:8], EXPECT + 'deadbeef') is None),
        # ⭐ 变异：改**实际字节**的最后一个十六进制位 ⇒ 必须红
        ('锚点红（末位改动 1 bit）',
         anchor_verdict(EXPECT, EXPECT[:-1] + ('0' if EXPECT[-1] != '0' else '1')) is not None),
        ('锚点红（完全不同）', anchor_verdict(EXPECT, 'f' * 32) is not None),
        ('锚点红（空串）', anchor_verdict(EXPECT, '') is not None),
    ]
    ok = True
    for name, good in cases:
        print(f'{"✅" if good else "⛔"} 自测 {name}')
        ok = ok and good
    print()
    print('SELFTEST OK: 锚点判定 绿/红 均可复现' if ok else 'SELFTEST FAIL')
    return 0 if ok else 1


def md5(path: str) -> str:
    with open(path, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()


def read(path: str) -> str:
    with open(path, encoding='utf-8', errors='ignore') as f:
        return f.read()


if '--self-test' in sys.argv:
    sys.exit(selftest())

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
    # ⭐ 锚点覆盖优先（见 ANCHORED 的裁决记录）
    expect = ANCHORED.get(mine)
    if expect is not None:
        err = anchor_verdict(expect, md5(a))
        if err:
            drift += 1
            fail.append(err)
        else:
            print(f'✅ 1b 锚点一致 src/{mine}（有意分叉，基准=自持树期望值）')
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
        # ⛔⛔ 原来只匹配**单引号** ⇒ `api-panel.ts` 用双引号 ⇒ **门漏报**。
        #   这正是本项目反复出现的「按字面匹配 ⇒ 假阴性」：门说了「1 个文件直连」，
        #   实际有 2 个。**判据要覆盖写法变体，不能假设引号统一。**
        if re.search(r"""from\s+['"]@tauri-apps/api/core['"]""", read(p_)):
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

# ── 4d i18n 键完整性（**注释感知**）────────────────────────────────────
# ⛔ 这条门来自我亲手犯的缺陷：i18n 迁移时改了 42 处 `t('...')` 调用，
#    却漏加 2 个键（chat.historyPaused / chat.inputPlaceholder）⇒
#    界面**显示原始键名**。我此前只验「有文本、无报错」⇒ 放过了。
#    而 `t()` 的诚实回退（缺键返回键名）**让缺陷看起来像功能**。
#
# ⚠️ 扫描必须**去注释**：本门第一版不感知注释 ⇒ 把注释里的 `t('x')`
#    当成缺失键（我今天第三次栽在「用字面匹配解析代码」上）。
def _strip_js_comments(src: str) -> str:
    src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    return re.sub(r'//[^\n]*', '', src)

_locales = {}
for _loc in ('zh-CN', 'en-US'):
    _p = os.path.join(UI, 'src/i18n/locales', f'{_loc}.json')
    if os.path.isfile(_p):
        try:
            _locales[_loc] = __import__('json').load(open(_p, encoding='utf-8'))
        except ValueError:
            fail.append(f'i18n 词条 {_loc}.json **不是合法 JSON**')

_used_keys = set()
for _root, _dirs, _files in os.walk(os.path.join(UI, 'src')):
    _dirs[:] = [x for x in _dirs if x not in ('node_modules', 'dist', 'vendor')]
    for _fn in _files:
        if _fn.endswith(('.ts', '.tsx')):
            _used_keys |= set(re.findall(
                # ⛔ 词边界必须在字符类**外面**：写成 `[\b]` 是**退格符**(0x08)
                #    不是锚点 ⇒ 永不匹配 ⇒ 扫出 0 键却判 PASS（绿色的谎言）。
                #    且下方有最小数量断言兜底。
                r"""\bt\(\s*['"]([A-Za-z0-9_.]+)['"]""",
                _strip_js_comments(read(os.path.join(_root, _fn)))))
# ⛔ 扫出 0 个键几乎必然是**扫描器坏了**（我已栽：`[\b]` 误写）。
#    0 结果若判 PASS，门就成了「绿色的谎言」—— 比没有门更坏。
if len(_used_keys) < 10:
    fail.append(
        f'i18n 键扫描只找到 **{len(_used_keys)}** 个调用键（预期 ≥10）\n'
        f'       ⇒ **扫描器本身坏了**，不是「没有缺失键」。\n'
        f'       本门第一版即栽在此：`[\\b]` 写成字符类里的退格符，扫出 0 键却判 PASS。'
    )
elif _locales:
    for _loc, _tbl in sorted(_locales.items()):
        _missing = sorted(_k for _k in _used_keys if _k not in _tbl)
        if _missing:
            fail.append(
                f'i18n[{_loc}] 源码用了但**词条缺失**：{_missing}\n'
                f'       ⇒ 界面会显示原始键名（t() 的诚实回退把它伪装成正常文案）'
            )
        else:
            print(f'✅ 4d i18n 键完整性[{_loc}]：{len(_used_keys)} 个调用键全部有词条')

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
