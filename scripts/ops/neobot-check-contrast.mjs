/**
 * 深浅两档的**对比度门** —— 补 `STATUS.md` §0 判据② 唯一缺的那道门。
 *
 * # 为什么必须有
 *
 * §0 判据② 明写「**深浅两套都达可读对比**」，而现有门
 * （`nt_check_visual` / `neobot-ui-smoke` …）**只量几何，不量对比度**。
 * ⇒ 「达可读对比」这句话此前**无人执行**。
 *
 * 它也确实抓到了东西：建门前的实测显示 `--nb-muted` 深色 5.50:1 ✅
 * 而**浅色仅 3.32:1 ⛔**（AA 正文需 4.5）—— 那个值是**我**当初补 HeroUI
 * 语义色时随手定的，从没量过。已修（浅色档单独给值，按 4.5:1 反推）。
 *
 * # 判据（先定）
 *
 * 采 WCAG 2.1 相对亮度比：
 *   · 正文级文字 ≥ **4.5:1**（AA normal text）
 *   · 大字 / UI 控件边界 ≥ **3.0:1**（AA large text & non-text）
 *   · 两档（dark / light）**都要过**
 *
 * ⛔ 量的是 **computed color vs 实际生效的背景色**（向上找第一个非透明
 *   background-color），不是 CSS 里写的变量值 —— 变量值不反映叠层与
 *   透明度。这与 `f38acb0f` §7.2「每个量都要问『我量的这个元素，是用户
 *   看到的东西吗』」是同一条纪律。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'

const require = createRequire(
  new URL('../../apps/neobot-desktop/frontend/package.json', import.meta.url))
const { chromium } = require('playwright')

const DIST = fileURLToPath(new URL('../../apps/neobot-desktop/neobot-ui/dist/', import.meta.url))
const PORT = 8841
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

/** 被量的选择器 → 该处文字的最小对比度要求 */
const TARGETS = [
  ['.nb-wordmark', 4.5, '字标'],
  ['.nb-tagline', 4.5, '副标（次要文字）'],
  ['.nb-actions button', 4.5, '顶栏按钮文字'],
  ['.text-muted', 4.5, '次要文字'],
  ['.text-ink', 4.5, '正文'],
  ['.nb-act-empty', 4.5, '空态说明'],
]

const STUB = {
  neobot_convo_list: [{
    id: 'c0', kind: 'direct', title: '海豚调试', members: ['u'], task_count: 0,
    last_active: '2026-10-01T00:00:00Z', muted: false, unread: 0,
  }],
  neobot_convo_messages: [{
    id: 'm1', convo_id: 'c0', role: 'user', text: '你好，这条消息用来量对比度',
    created_at: '2026-10-01T00:00:00Z',
  }],
  neobot_member_list: [],
  neobot_usage_summary: { days: 1, tokens: 0 },
  neobot_memory_list: { lines: ['用户偏好中文'], bytes: 0, cap: 0, revisions: 0 },
  neobot_core_capabilities: { crystal_version: '0', tool_count: 0, model: '', model_source: '' },
  log_frontend: null, set_language: null, read_run_logs: '', quit_app: null,
  open_external_url: null, get_dsh_theme: 'system',
}

const srv = createServer(async (q, r) => {
  const u = new URL(q.url, 'http://x')
  if (!u.pathname.startsWith('/app/')) { r.writeHead(404).end(); return }
  const rel = normalize(u.pathname.slice(5)).replace(/^(\.\.[/\\])+/, '')
  try {
    const bb = await readFile(join(DIST, rel === '' || rel === '/' ? 'index.html' : rel))
    r.writeHead(200, { 'content-type': MIME[extname(rel)] ?? 'application/octet-stream' })
    r.end(bb)
  } catch { r.writeHead(404).end() }
})
await new Promise((r) => srv.listen(PORT, '127.0.0.1', r))

const initScript = ({ t }) => {
  window.__TAURI_INTERNALS__ = {
    invoke: (c) => Promise.resolve(t[c] ?? null),
    transformCallback: (cb) => { window._1 = cb; return 1 },
    metadata: { currentWindow: { label: 'main' } },
  }
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
}

/** WCAG 相对亮度 */
function lum(css) {
  const [r, g, b] = (css.match(/[\d.]+/g) || []).slice(0, 3).map(Number).map((v) => {
    const s = v / 255
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}
function ratio(a, b) {
  const l1 = lum(a); const l2 = lum(b)
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)
}

const browser = await chromium.launch({ channel: 'chrome' })
let fail = 0
let measured = 0

for (const theme of ['dark', 'light']) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 820 } })
  await page.addInitScript(initScript, { t: STUB })
  await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
  await page.evaluate((th) => document.documentElement.setAttribute('data-theme', th), theme)
  await page.waitForTimeout(900)

  const rows = await page.evaluate((sels) => {
    const bgOf = (el) => {
      let n = el
      while (n) {
        const bg = getComputedStyle(n).backgroundColor
        if (bg && !/rgba\(0, 0, 0, 0\)|transparent/.test(bg)) return bg
        n = n.parentElement
      }
      return getComputedStyle(document.body).backgroundColor
    }
    return sels.map((sel) => {
      const el = document.querySelector(sel)
      if (!el) return null
      return { sel, fg: getComputedStyle(el).color, bg: bgOf(el) }
    }).filter(Boolean)
  }, TARGETS.map((t) => t[0]))

  console.log(`\n  ── ${theme} ──`)
  for (const [sel, min, label] of TARGETS) {
    const r = rows.find((x) => x.sel === sel)
    if (!r) { continue }   // 该元素不存在（页面未渲染到）⇒ 不计入，也不算过
    const got = ratio(r.fg, r.bg)
    measured++
    const ok = got >= min
    if (!ok) fail++
    console.log(`     ${ok ? '✅' : '⛔'} ${label.padEnd(14)} ${got.toFixed(2)}:1`
      + `  (需 ≥${min}:1)  ${r.fg} on ${r.bg}`)
  }
  await page.close()
}
await browser.close()
srv.close()

console.log()
if (measured === 0) {
  console.log('⛔ 一处都没量到 —— 判据形同不存在（选择器全失效）。拒绝报 PASS。')
  process.exit(1)
}
if (fail) {
  console.log(`FAIL: ${fail}/${measured} 处对比度不足`)
  process.exit(1)
}
console.log(`PASS: 深浅两档共 ${measured} 处文字均达 WCAG AA。`)
