#!/usr/bin/env node
/**
 * ⭐⭐⭐ 栏族尺度门（2026-10-04）—— 抄 deepseek-harness 的 CI 契约手法。
 *
 * ⭐⭐⭐ **立门理由（三家对标研究的核心结论）**：
 *   对标 DSH-better-sidebar 有唯一一份「冻结尺度表」
 *   （`docs/plans/2026-09-27-files-changes-uiux.md` §1），
 *   ⭐⭐ **但它的 `tests/theme.spec.ts` 只扫 `color:` ⇒ ⭐⭐ 尺寸漂了它不会红**。
 *   ⇒ ⭐⭐ 「写下来」不等于「守得住」。
 *   ⭐⭐ 真正被 CI 强制执行的是 dsh 的 `builder.rs:1213`
 *   `shell_nav_tests::shell_nav_height_matches_navbar_height_class` ——
 *   ⭐⭐ 它执行的不是某个数字，是**原则**：「栏高必须是 4px 刻度的整数倍」。
 *   ⇒ ⭐⭐ 本门就是那个原则的移植。
 *
 * ⭐⭐⭐ 本门执行 5 条原则（每条都能独立判红）：
 *   P1 刻度无孤儿：⭐⭐ 尺度变量声明了必须被消费（⭐ 本轮实测曾有 5 个孤儿）
 *   P2 栏归栏族：⭐⭐ 每根栏挂 `.nb-band`，⛔ 不得自行声明 height/padding-block
 *   P3 列宽单一真源：⛔ 禁 `max-w-[<N>px]` 形式的字面列宽（本轮实测 3 处）
 *   P4 颜色语义化：⛔ CSS 规则里禁字面色（注释不算）
 *   P5 ⭐⭐ 跨语言契约：⭐⭐ 交通灯 y + 灯径 ≤ 栏高（⭐ dsh issue #524 同型）
 *      —— ⭐⭐ 这是**唯一能跨「CSS 改了但没人发现」**的断言
 *
 * ⭐⭐⭐ **P0 自检（⭐⭐ 抄 better-sidebar 的教训）**：
 *   它自己的门写了 `expect(sheets.length).toBeGreaterThan(5)`，
 *   注释：「Guard the scan itself: a glob that silently matches nothing
 *   would make this contract vacuous.」
 *   ⭐⭐ 本门同理：⭐⭐ **扫描器自己空转 ⇒ 契约作废** ⇒ 必须先证明扫到了东西。
 */
import { readFileSync, existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..')
const UI = join(ROOT, 'apps/neobot-desktop/neobot-ui/src')
const css = (f) => readFileSync(join(UI, f), 'utf8')
const tsx = (f) => readFileSync(join(UI, f), 'utf8')

let fail = 0
const bad = (m) => { console.log(`  ⛔ ${m}`); fail += 1 }
const ok = (m) => console.log(`  ✅ ${m}`)

const shellCss = css('shell.css')
const themeCss = css('theme.css')
const rootTsx = tsx('neobot-root.tsx')

// ═══ P0 自检：⭐⭐ 先证明扫描器不是空转 ═══
console.log('栏族尺度门\n')
const SCALE_VARS = [
  '--nb-bar-h', '--nb-bar-h-sub', '--nb-bar-pad-y', '--nb-bar-pad-x',
  '--nb-row-h', '--nb-row-2-h', '--nb-ctl-h', '--nb-col-w', '--nb-side-w',
]
const declared = SCALE_VARS.filter((v) => themeCss.includes(`${v}:`))
if (declared.length < SCALE_VARS.length) {
  bad(`P0 自检失败：只声明了 ${declared.length}/${SCALE_VARS.length} 个尺度变量`
    + ' ⇒ ⭐⭐ **扫描器与尺度族已漂移**，本门会失去意义')
} else {
  ok(`P0 扫描器自检：${declared.length}/${SCALE_VARS.length} 个尺度变量已定位`)
}
if (shellCss.length < 500 || rootTsx.length < 10000) {
  bad(`P0 自检失败：源文件异常小（shell.css=${shellCss.length} root.tsx=${rootTsx.length}）`
    + ' ⇒ ⭐⭐ 扫描器读到了空/错文件')
}

// ═══ P1 刻度无孤儿 ═══
{
  const orphans = SCALE_VARS.filter(
    (v) => !shellCss.includes(`var(${v}`) && !rootTsx.includes(`var(${v}`),
  )
  if (orphans.length) {
    bad(`P1 尺度变量声明了却**零消费**：${orphans.join(', ')}`
      + ' ⇒ ⭐⭐ 「声明了没人用的档比没有更危险」'
      + '（下一个 agent 会以为栏族已统一，而实际一个栏都没接上）')
  } else {
    ok(`P1 ${SCALE_VARS.length} 个尺度变量全部被消费`)
  }
}

// ═══ P2 栏归栏族 ═══
{
  if (!shellCss.includes('.nb-band')) { bad('P2 `.nb-band` 配方不存在 ⇒ 栏族无处可挂') }
  else {
    const users = (rootTsx.match(/nb-band/g) || []).length
    const shellUsers = (shellCss.match(/^\.nb-[a-z-]*[ ,]/gm) || []).length
    if (users === 0) {
      bad('P2 `.nb-band` **没有任何 TSX 消费** ⇒ 定义了但没接上')
    } else {
      ok(`P2 .nb-band 已被 ${users} 处消费（TSX）`)
    }
    if (shellUsers < 3) {
      bad(`P2 shell.css 里使用 .nb-band 的规则只有 ${shellUsers} 条 ⇒ ⭐⭐ 覆盖不足`)
    } else {
      ok(`P2 shell.css 内 ${shellUsers} 条规则挂上栏族`)
    }
  }
  // ⛔ 侧栏/次级栏⛔ 不得自行写 h-12 之类（⭐ 本轮实测 h-12 就是不统一的实锤）
  const strayHeights = [...rootTsx.matchAll(/<header className="[^"]*\bh-\d+/g)]
  if (strayHeights.length) {
    bad(`P2 TSX 里有 ${strayHeights.length} 处 <header> 自行声明 Tailwind 高度`
      + ' ⇒ ⭐⭐ 应挂 .nb-band（h-12=48px 会让会话头与顶栏等高）')
  } else {
    ok('P2 无 <header> 自行声明高度')
  }
}

// ═══ P3 列宽单一真源 ═══
{
  // ⭐⭐⭐ 判据**必须排除说明性文字**：⭐ 本轮实测 `max-w-[320px]`（`:1162`）
  //   是**一段说明文字的行宽**，⛔ **不是消息列宽**。
  //   ⇒ ⭐⭐ 若不排除，我会去「修」一处**完全正确**的代码
  //   （AGENTS.md §5：扫描器告警 ≠ 缺陷；⭐⭐ 同型病第 N 次）。
  // ⭐⭐ 判据：⭐ **只查承载消息/会话的容器**（class 里带 msg/chat/composer/canvas/thread），
  //   ⭐⭐ 而不是全文扫 `max-w-[Npx]` —— 后者必然误伤说明文字与图标。
  const COL_W_HOSTS = /className="[^"]*\b(?:msg|chat|composer|canvas|thread|convo)[^"]*"[^>]*max-w-\[(\d+)px\]/g
  const lit = [...rootTsx.matchAll(COL_W_HOSTS)].map((m) => m[0])
  if (lit.length) bad(`P3 消息列宽字面量 ${lit.length} 处 ⇒ 应走 --nb-col-w`)
  else ok('P3 消息/会话容器列宽走 --nb-col-w（⭐ 说明性文字已正确排除）')
  const sideLit = [...rootTsx.matchAll(/\bw-(\d+)\b(?=[^"]*shrink-0[^"]*border-r)/g)]
  if (sideLit.length) bad(`P3 侧栏宽用 Tailwind 字面量：${sideLit.map((m) => m[0]).join(', ')} ⇒ 应走 --nb-side-w`)
  else ok('P3 侧栏宽走 --nb-side-w')
}

// ═══ P4 颜色语义化（⭐⭐ 抄 better-sidebar theme.spec.ts 的扫法，但⭐⭐ 扩大到全部颜色属性）═══
{
  // ⭐⭐ 剥注释：⭐⭐ better-sidebar 的门**只扫 color:** ⇒ 尺寸/边框漂了它不会红。
  // ⭐⭐ 本门扫 `color` / `background` / `border*`，且 ⭐⭐ 先剥注释（否则注释里的
  // ⭐⭐ 「改前是 #fbfbfd」这类考古记录会误判红）。
  const stripComments = (s) => s.replace(/\/\*[\s\S]*?\*\//g, '')
  const bare = stripComments(shellCss)
  const literals = [...bare.matchAll(/(?:^|[\s;{])(?:color|background|border[a-z-]*)\s*:[^;{}]*?(#[0-9a-f]{3,8})/g)]
    .map((m) => m[1])
  if (literals.length) {
    bad(`P4 CSS 规则里出现字面色：${[...new Set(literals)].join(', ')} ⇒ ⭐⭐ 应走 --nb-color-*`)
  } else {
    ok('P4 shell.css 无字面色（已剥注释）')
  }
}

// ═══ P5 ⭐⭐ 跨语言契约（dsh issue #524 同型）═══
{
  const confPath = join(ROOT, 'apps/neobot-desktop/tauri.conf.json')
  if (!existsSync(confPath)) {
    bad('P5 找不到 tauri.conf.json ⇒ 无法验证跨语言契约')
  } else {
    const conf = JSON.parse(readFileSync(confPath, 'utf8'))
    const barH = Number((themeCss.match(/--nb-bar-h:\s*(\d+)px/) || [])[1] || 0)
    const y = conf?.app?.windows?.[0]?.trafficLightPosition?.y
    const x = conf?.app?.windows?.[0]?.trafficLightPosition?.x
    if (typeof y !== 'number' || !barH) {
      bad(`P5 取不到 trafficLightPosition.y(${y}) 或 --nb-bar-h(${barH})`)
    } else {
      // ⭐⭐ macOS 交通灯直径 12px（Apple HIG）⇒ 底边 = y + 12
      const bottom = y + 12
      // ⭐⭐⭐ 修**门自身**的缺陷（⭐ 第一版就抓错了地方）：
      //   ⭐ `--nb-traffic-inset` 的**默认值 0px 在 theme.css**，
      //   ⭐⭐ 而**平台值 78px 在 shell.css 的 `[data-platform='macos']`** ——
      //   ⭐⭐ 两者都必须读，⛔ 只读 theme.css 会得出「让位 0px」的**假结论**。
      //   ⇒ 门必须表达「默认在此、平台值在平台选择器」这个**真实契约**。
      const insetDefault = Number((themeCss.match(/--nb-traffic-inset:\s*(\d+)px/) || [])[1] ?? NaN)
      const insetMac = Number((shellCss.match(/--nb-traffic-inset:\s*(\d+)px/) || [])[1] ?? NaN)
      const inset = Number.isFinite(insetMac) ? insetMac : insetDefault
      if (!Number.isFinite(insetDefault)) {
        bad('P5 theme.css 缺 `--nb-traffic-inset` 默认值'
          + ' ⇒ ⭐⭐ 非 macOS 上 var() 无定义 ⇒ 整条 padding 失效')
      }
      // ⭐⭐ 三灯总宽 ≈ 3×12 + 2×8 ⇒ 需要 ≥ 14+52 = 66px 的左让位
      const needW = (x ?? 14) + 3 * 12 + 2 * 8
      if (bottom > barH) {
        bad(`P5 ⭐⭐ 跨语言契约破：交通灯底边 ${bottom}px > 栏高 ${barH}px`
          + ' ⇒ ⭐⭐ 改 CSS 忘了改配置（或反之），界面必然遮挡')
      } else if (inset < needW) {
        bad(`P5 ⭐⭐ 左让位不足：--nb-traffic-inset=${inset}px < 需要 ${needW}px`
          + `（x=${x} + 3×12 + 2×8）⇒ 交通灯会压住品牌区`)
      } else {
        ok(`P5 跨语言契约成立：交通灯底边 ${bottom}px ≤ 栏高 ${barH}px；`
          + `左让位 ${inset}px ≥ 需要 ${needW}px`)
      }
    }
  }
}

console.log(fail ? `\nFAIL: ${fail} 条` : '\nPASS: 栏族尺度与跨语言契约全部成立。')
process.exit(fail ? 1 : 0)
