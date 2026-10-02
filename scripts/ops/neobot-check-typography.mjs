/**
 * 字号阶梯门 —— 吸收 `liseami/ChunUI`（MIT）的**硬约束写法**。
 *
 * # 为什么必须有
 *
 * `docs/architecture/ABSORPTION-2026-10-01-CHUNUI-GROWTH-PI.md` §5 记录了实测：
 * `neobot-ui` 的 `font-size` 有 **8 种取值**，同时**又**在用 Tailwind 的
 * `text-xs` / `text-sm` —— 两套体系并存；`neobot-root.tsx` 里另有 **30 处
 * Tailwind 任意值字号**（`text-[10px]` / `[11px]` / `[13px]` / `[15px]` …）
 * **完全绕过 token 体系**。而 `skills/design/design-core/SKILL.md`
 * 只要求「生成 typography scale」，**不强制**。
 *
 * ChunUI 的做法正好补这个洞：字号是**定档铁律**，枚举定档 + 全局校验，
 * 违反即失败 —— 即「规则被机器执行」而非「写在文档里等人遵守」。
 *
 * ⚠️ **本门只查我们自己的代码。** `apps/neobot-desktop/frontend/` 是
 * **vendored 第三方代码**（dsh-harness-desktop 0.19.1, MIT，见其 `VENDOR.md`），
 * 用我们的规矩去判它的字号是错的 —— vendored 改动必须走 vendor 流程。
 * ⇒ 扫描范围显式排除它。
 *
 * # 判据（先定，再写实现）
 *
 * ① **px 字号必须在定档集合内**，且**下限 12px**。
 *    为什么卡 px：绝对字号是「随手写一个数字」的产物，漂移都积在这里。
 *    为什么下限 12px：11px 已在仓库里出现多处，而本仓已有对比度门
 *    （`neobot-check-contrast.mjs`）明确按 WCAG AA 取值 —— 11px 连正文
 *    可读性都谈不上，却能被自由写出，说明缺的是**机器约束**。
 *
 * ② **em / rem 相对字号允许，但必须落在 [0.80, 2.00] 带内**。
 *    为什么允许：`nb-markdown.css` 里的 0.94em / 1.18em 是
 *    **markdown 标题层级**（h1 比正文大一点），那是**语义**不是绝对尺寸，
 *    强行压成 px 会破坏层级语义。
 *    为什么还要卡带：相对值没有定档就会出 `1.8em` 这种手滑。
 *
 * ③ / ③b **Tailwind 字号类必须在允许集合内**；**任意值 `text-[13px]` 同样受判据**。
 *    ③b 是补**我自己门的漏洞**：原正则只认 `text-<词>`，于是
 *    `text-[10px]` 这类**任意值**整个绕过门 —— 而 `neobot-root.tsx` 里有 30 处。
 *    发现方式不是读代码，是**看构建产物**：门报「PASS 24 处」，
 *    而 `dist/assets/main-*.css` 里仍有 `.text-\[11px\]{font-size:11px}`
 *    ⇒ **门漏了它没看见的东西**。
 *    ⇒ 纪律：门必须以**产物/实测量**为准，扫源文件不等于覆盖。
 *
 * ④ **⛔ 零命中 ⇒ FAIL**，拒绝报 PASS。
 *    判据形同不存在时必须显式失败 —— 与 `neobot-check-contrast.mjs` 同纪律。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { readdir, readFile } from 'node:fs/promises'
import { join, relative, extname } from 'node:path'

const ROOT = 'apps/neobot-desktop/neobot-ui/src'

// 判据①：绝对字号定档。下限 12px（见文件头「为什么下限 12px」）。
const ALLOWED_PX = new Set([12, 13, 14, 16, 18, 20, 22, 24, 28, 32])
// 判据②：相对字号带，防手滑。
const EM_MIN = 0.8
const EM_MAX = 2.0
// 判据③：Tailwind 具名字号类允许集合。
const ALLOWED_TW = new Set([
  'text-xs', 'text-sm', 'text-base', 'text-lg', 'text-xl',
  'text-2xl', 'text-3xl', 'text-4xl',
])

const violations = []
let pxSeen = 0
let emSeen = 0
let twSeen = 0

async function* walk(dir) {
  for (const e of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory()) yield* walk(p)
    else if (['.css', '.tsx', '.ts', '.jsx', '.js', '.html'].includes(extname(e.name))) yield p
  }
}

for await (const file of walk(ROOT)) {
  const src = await readFile(file, 'utf8')
  const lines = src.split('\n')
  const rel = relative('.', file)

  lines.forEach((line, i) => {
    // ⛔ 跳过注释行：注释里的 font-size 不是声明（AGENTS.md R-SCAN-1b 同源纪律 ——
    // 本仓 nt_meta/scanner.rs 就因把禁词当数据持有而误报）。
    const t = line.trim()
    if (t.startsWith('/*') || t.startsWith('*') || t.startsWith('//')) return

    // 判据①：px
    for (const m of line.matchAll(/font-size:\s*(\d+(?:\.\d+)?)px/g)) {
      pxSeen++
      const v = Number(m[1])
      if (!ALLOWED_PX.has(v)) {
        violations.push(`${rel}:${i + 1}  font-size:${v}px 不在定档集合 [${[...ALLOWED_PX].join(',')}]`)
      }
    }
    // 判据②：em / rem
    for (const m of line.matchAll(/font-size:\s*(\d+(?:\.\d+)?)(em|rem)/g)) {
      emSeen++
      const v = Number(m[1])
      if (v < EM_MIN || v > EM_MAX) {
        violations.push(`${rel}:${i + 1}  font-size:${v}${m[2]} 越界（允许 ${EM_MIN}~${EM_MAX}）`)
      }
    }
    // 判据③：Tailwind 具名字号类
    for (const m of line.matchAll(/\b(text-(?:xs|sm|base|lg|xl|\dxl))\b/g)) {
      twSeen++
      if (!ALLOWED_TW.has(m[1])) {
        violations.push(`${rel}:${i + 1}  ${m[1]} 不在 Tailwind 字号允许集合`)
      }
    }
    // 判据③b：Tailwind 任意值字号（判据③的同一约束，不得绕过）
    for (const m of line.matchAll(/\btext-\[(\d+(?:\.\d+)?)(px|rem|em)\]/g)) {
      twSeen++
      const v = Number(m[1])
      const unit = m[2]
      if (unit === 'px' && !ALLOWED_PX.has(v)) {
        violations.push(`${rel}:${i + 1}  text-[${v}px] 任意值绕过定档（应改为定档值）`)
      } else if (unit !== 'px' && (v < EM_MIN || v > EM_MAX)) {
        violations.push(`${rel}:${i + 1}  text-[${v}${unit}] 越界（允许 ${EM_MIN}~${EM_MAX}）`)
      }
    }
  })
}

console.log('=== NeoTrix 字号阶梯门（ChunUI 硬约束写法）===')
console.log(`扫描范围: ${ROOT}（⛔ 已排除 vendored 的 apps/neobot-desktop/frontend/）`)
console.log(`  px 声明 ${pxSeen} 处 · em/rem 声明 ${emSeen} 处 · Tailwind 字号类 ${twSeen} 处`)

// 判据④
const total = pxSeen + emSeen + twSeen
if (total === 0) {
  console.log('⛔ 一处字号声明都没扫到 —— 判据形同不存在（范围写错 or 全被注释）。拒绝报 PASS。')
  process.exit(1)
}

if (violations.length) {
  console.log(`\nFAIL: ${violations.length}/${total} 处字号违反定档`)
  for (const v of violations) console.log(`  · ${v}`)
  console.log('\n修法：绝对值收敛到定档（px 下限 12px），或用 rem/em 表达层级。')
  process.exit(1)
}
console.log(`PASS: ${total} 处字号均在定档内（px 下限 12px · em 带 ${EM_MIN}~${EM_MAX}）。`)