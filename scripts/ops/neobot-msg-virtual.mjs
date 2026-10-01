/**
 * 消息列表窗口化门 —— 判据 A/B/C/D，**先定后写**。
 *
 * # 为什么单独一个门（而不是塞进 neobot-ui-smoke.mjs）
 *
 * 我试过把它加进那个文件，结果在 600 行里反复做括号手术后**失衡**，
 * 且一度出现「报告层被 `if (mvz)` 整块跳过 ⇒ rc=0 其实是**没跑**」。
 * ⇒ 一个门只管一件事。本脚本只回答一个问题：
 *   **1000 条消息时，虚拟化是否正确、用户是否仍够得着内容。**
 *
 * # 判据（先定；含我第一版写错的那条）
 *
 * |   | 判据 | 为什么 |
 * |---|------|--------|
 * | A | 1000 条 → DOM 行数 « 1000 | 窗口化生效 |
 * | B | 撑高 > 2000px | 否则滚动条按「可见项数」算高 ⇒ 根本滚不动 |
 * | C | 加载后**末条**在视口内 | 自动滚底生效 |
 * | D | 滚到顶后**首条**可见 | 内容**可达**（这才是「用户看得到」） |
 *
 * ⛔ **C 我第一版写成「首条可见」** ⇒ 对大列表**必然假失败**：
 *    应用此时正确地停在底部，视口里就是最后几条。判据写错 ≠ 产品有缺陷。
 *
 * # 判据 D 为什么必需（A/B/C 全过还不够）
 *
 * 窗口化最危险的失败模式是**「滚不回去」**：DOM 里只有尾部几条，
 * 用户往上滚却永远看不到早期内容 —— 界面看起来完全正常。
 * A/B/C 都测不出来，**只有 D 能测**。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'

// playwright 装在 vendored 的 frontend 里；⛔ 不在仓库根建 node_modules
// 软链（那是根级新项，会被命名门拦下 —— 实测过）。
const require = createRequire(
  new URL('../../apps/neobot-desktop/frontend/package.json', import.meta.url))
const { chromium } = require('playwright')

const DIST = fileURLToPath(new URL('../../apps/neobot-desktop/neobot-ui/dist/', import.meta.url))
const PORT = 8811
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

const N = 1000
const convos = Array.from({ length: N }, (_, i) => ({
  id: `c${i}`, kind: 'direct', title: `会话 ${i}`,
  members: ['u1'], task_count: 0,
  last_active: new Date(Date.now() - i * 6e4).toISOString(),
  muted: false, unread: 0,
}))
const msgs = Array.from({ length: N }, (_, i) => ({
  id: `m${i}`, convo_id: 'c0', role: i % 2 ? 'assistant' : 'user',
  // 首末条留确定性标记，供 C/D 断言用
  text: i === 0 ? 'FIRSTMSG' : i === N - 1 ? 'LASTMSG' : `第 ${i} 条消息`.repeat(3),
  created_at: '2026-10-01T00:00:00Z',
}))

const STUB = {
  neobot_convo_list: convos,
  neobot_convo_messages: msgs,
  neobot_member_list: [],
  neobot_usage_summary: { days: 1, tokens: 0 },
  neobot_memory_list: { lines: [], bytes: 0, cap: 0, revisions: 0 },
  neobot_core_capabilities: { crystal_version: '0', tool_count: 0, model: '', model_source: '' },
  log_frontend: null, set_language: null, read_run_logs: '',
  quit_app: null, open_external_url: null, get_dsh_theme: 'system',
}

const srv = createServer(async (q, r) => {
  const u = new URL(q.url, 'http://x')
  if (!u.pathname.startsWith('/app/')) { r.writeHead(404).end(); return }
  const rel = normalize(u.pathname.slice(5)).replace(/^(\.\.[/\\])+/, '')
  try {
    const b = await readFile(join(DIST, rel === '' || rel === '/' ? 'index.html' : rel))
    r.writeHead(200, { 'content-type': MIME[extname(rel)] ?? 'application/octet-stream' })
    r.end(b)
  } catch { r.writeHead(404).end() }
})
await new Promise((r) => srv.listen(PORT, '127.0.0.1', r))

const browser = await chromium.launch({ channel: 'chrome' })
const page = await browser.newPage({ viewport: { width: 1280, height: 820 } })
const pageErrors = []
page.on('pageerror', (e) => pageErrors.push(e.message))
await page.addInitScript((t) => {
  window.__TAURI_INTERNALS__ = {
    invoke: (c) => (c === 'plugin:event|listen' ? Promise.resolve(1)
      : c === 'plugin:event|unlisten' ? Promise.resolve(null)
      : Promise.resolve(t[c] ?? null)),
    transformCallback: (cb) => { window._1 = cb; return 1 },
    metadata: { currentWindow: { label: 'main' } },
  }
  // ⛔ Tauri **事件插件**内部对象：缺它，`listen()` 的 unlisten 会抛
  //   `Cannot read properties of undefined (reading 'unregisterListener')`。
  //   同一问题我栽了第 4 次（每写一个新脚本就重犯一次）；这次是**门抓到的**
  //   （本门显式断言「页面有未捕获异常」），不是漏过去的。
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
}, STUB)
await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
await page.waitForTimeout(1800)

const txt = () => page.evaluate(
  () => (document.querySelector('.nb-main')?.innerText || '').replace(/\s+/g, ' '))
const a = await page.evaluate(() => {
  const s = document.querySelector('[data-testid="nb-msg-sizer"]')
  if (!s) return { missing: true }
  return { sizerH: Math.round(s.getBoundingClientRect().height), rows: s.querySelectorAll('[data-index]').length }
})
const cOk = (await txt()).includes('LASTMSG')
// D：滚到顶
await page.evaluate(() => {
  const el = document.querySelector('[class*="overflow-y-auto"][class*="relative"]')
  if (el) el.scrollTop = 0
})
await page.waitForTimeout(700)
const dOk = (await txt()).includes('FIRSTMSG')

await browser.close()
srv.close()

let fail = 0
console.log(`消息列表窗口化门（${N} 条消息）\n`)
if (a.missing) {
  console.log('  ⛔ 未找到 [data-testid="nb-msg-sizer"] ⇒ 窗口化未渲染，判据无法执行')
  fail++
} else {
  console.log(`  A 渲染 ${a.rows} 行 / ${N} 条 ⇒ ${a.rows < N / 4 ? '✅ 已窗口化' : '⛔ 全量渲染'}`)
  console.log(`  B 撑高 ${a.sizerH}px ⇒ ${a.sizerH > 2000 ? '✅ 滚动条高度可信' : '⛔ 撑高不足，滚不动'}`)
  console.log(`  C 加载后末条在视口 ⇒ ${cOk ? '✅ 自动滚底生效' : '⛔ 未停在底部'}`)
  console.log(`  D 滚到顶后首条可见 ⇒ ${dOk ? '✅ 内容可达' : '⛔ 滚不回去（最危险的静默故障）'}`)
  if (a.rows >= N / 4) fail++
  if (a.sizerH <= 2000) fail++
  if (!cOk) fail++
  if (!dOk) fail++
}
if (pageErrors.length) {
  console.log(`  ⛔ 页面有未捕获异常 × ${pageErrors.length}：${pageErrors[0].slice(0, 90)}`)
  fail++
}
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: 窗口化生效，且用户仍够得着全部内容。')
process.exit(fail ? 1 : 0)
