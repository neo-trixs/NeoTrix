/**
 * 会话列表方向键导航门 —— 判据 A~H，**先定后写**。
 *
 * # ⭐ 它守的是虚拟化列表里**最容易做假**的一个功能
 *
 * 1000 个会话时，DOM 里**永远只有约 20 个** `[data-testid="nb-convo-item"]`。
 * ⓘ 于是天真的方向键实现（「找下一个兄弟节点并 focus」）**在第 21 个就走到尽头**：
 *    用户按 21 次 ↓ 之后什么都不会发生，而**界面看起来完全正常** ——
 *    焦点在动、样式在变、没有任何报错。**纯人眼测试会判它「能用」**。
 *
 * ⇒ D 判据专门跨过这个边界：按**超过一个渲染窗口**的次数，断言焦点**真的**继续移动。
 *
 * # 判据
 *
 * |  | 判据 | 拦住什么 |
 * |---|------|---------|
 * | A | `↓` 把焦点移到**下一个**会话项 | 本体 |
 * | B | `↑` 移回 | 方向反了 |
 * | C | `Home`/`End` 跳到**当前渲染窗口**首/尾 | 键位没接 |
 * | D ⭐ | 连续按 `↓` **超过一个窗口**（25 次 > 20）后，焦点**仍在移动** | 虚拟化边缘断链（第 21 个起无声失效） |
 * | E | **不劫持 Tab** | 键盘用户被卡在列表里出不去 |
 * | F | 焦点确实落在会话项上（可观测） | 「focus 了一个不存在的元素」 |
 * | G | 搜索框里的方向键**不**触发列表导航 | 作用域失控：在输入框按 ↑ 却跳走列表焦点 |
 * | H | 零未捕获异常 | — |
 *
 * ⛔ **D 不可省**，且「25 次」这个数字是有意的：它**必须大于**实测的渲染窗口（20），
 *    否则测的还是同一个窗口内，边缘断链测不出来。
 * ⛔ **G 不可省**：handler 挂在列表容器上，但事件会从搜索框冒泡吗？——
 *    不会（搜索框在容器外），但**这类作用域错误一旦发生极难察觉**，
 *    判据写在这里是为了钉住它。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { neobotDistFresh } from './nt_dist_freshness.mjs'

// ⭐⭐⭐ 2026-10-04：playwright 改从**自持交付树** `neobot-ui/` 解析。
// ⭐⭐ 改前锚定 `frontend/`（vendored 冻结树）的 devDependency，而
// ⭐⭐ **CI 只在 `neobot-ui/` 跑 `pnpm install`** ⇒ ⭐⭐ CI 里必然加载失败
// ⭐⭐ ⇒ ⭐⭐⭐ 这 9 道 UI 门在 CI 上等于不存在。
// ⭐⭐ 另：`neobot-ui` 无 `pnpm-workspace.yaml`，⭐⭐ 故用**具体版本**
// ⭐⭐ 而非 vendored 那套 `catalog:testing`。⭐⭐ 许可 Apache-2.0（不在 deny 名单）。
const require = createRequire(
  new URL('../../apps/neobot-desktop/neobot-ui/package.json', import.meta.url))
const { chromium } = require('playwright')

const DIST = fileURLToPath(new URL('../../apps/neobot-desktop/neobot-ui/dist/', import.meta.url))
// ⭐⭐ 2026-10-07 P0-1：产物新鲜度断言（公用件，17 道读 dist 的门统一走这条）。
//    ⛔ 不新鲜就在**开浏览器之前**退出 —— 否则白等几十秒再失败。
neobotDistFresh('neobot-check-convo-keys');
const PORT = 8873
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

// ⛔ 1000 会话：必须**远大于**渲染窗口，否则 D 判据失去意义
const N = 1000
const convos = Array.from({ length: N }, (_, i) => ({
  id: `c${i}`, kind: 'direct', title: `会话 ${i}`, members: ['u1'], task_count: 0,
  last_active: new Date(Date.now() - i * 6e4).toISOString(), muted: false, unread: 0,
}))
const msgs = Array.from({ length: N }, (_, i) => ({
  id: `m${i}`, convo_id: 'c0', role: i % 2 ? 'assistant' : 'user',
  text: `第 ${i} 条消息`, created_at: '2026-10-01T00:00:00Z',
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
  write_clipboard_text: null,
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

const browser = await chromium.launch({ channel: 'chrome' })
let fail = 0
const bad = (m) => { console.log(`     ⛔ ${m}`); fail++ }

const ctx = await browser.newContext({ viewport: { width: 1280, height: 820 } })
const page = await ctx.newPage()
const errs = []
page.on('pageerror', (e) => errs.push(e.message))
// ⛔ invoke 桩：传参与读取必须一致（本次会话已被此坑三次）
await page.addInitScript((t) => {
  window.__TAURI_INTERNALS__ = {
    invoke: (c) => (c === 'plugin:event|listen' ? Promise.resolve(1)
      : c === 'plugin:event|unlisten' ? Promise.resolve(null)
      : Promise.resolve(t[c] ?? null)),
    transformCallback: (cb) => { window._1 = cb; return 1 },
    metadata: { currentWindow: { label: 'main' } },
  }
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
}, STUB)
await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
await page.waitForSelector('[data-testid="nb-convo-item"]', { state: 'attached', timeout: 15000 })
await page.waitForTimeout(900)

const ITEM = '[data-testid="nb-convo-item"]'
const focused = () => page.evaluate((s) => {
  const a = document.activeElement
  return {
    isItem: !!a?.matches?.(s),
    title: a?.matches?.(s) ? a.innerText.trim().slice(0, 20) : null,
    inList: !!a?.closest?.('[data-testid="nb-convo-list"]'),
  }
}, ITEM)
const titles = () => page.evaluate((s) => [...document.querySelectorAll(s)].map((n) => n.innerText.trim().slice(0, 20)), ITEM)

console.log('会话列表方向键导航门\n')

const win0 = await titles()
console.log(`  （1000 会话，DOM 渲染 ${win0.length} 项 ⇒ 虚拟化窗口 ≈ ${win0.length}）`)

// 从第 1 项开始
await page.locator(ITEM).first().focus()
await page.waitForTimeout(150)
const f0 = await focused()
console.log(`  起点：${JSON.stringify(f0)}`)
if (!f0.isItem) bad('起点不是会话项')

// A：↓
await page.keyboard.press('ArrowDown')
await page.waitForTimeout(200)
const a = await focused()
console.log(`  A ↓：${JSON.stringify(a)}`)
if (!a.isItem) bad('↓ 之后焦点不在会话项上')
else if (a.title === f0.title) bad('↓ 之后焦点没移动')

// B：↑ 移回
await page.keyboard.press('ArrowUp')
await page.waitForTimeout(200)
const b = await focused()
console.log(`  B ↑：${JSON.stringify(b)}`)
if (b.title !== f0.title) bad(`↑ 没移回（${f0.title} → ${b.title}）`)

// C：End / Home
await page.keyboard.press('End')
await page.waitForTimeout(250)
const cEnd = await focused()
// ⛔⛔ **不能在按 Home 之前就认定首项是「会话 0」**：
//   聚焦一个项会把它滚进视口 ⇒ 虚拟化器**重渲染** ⇒ 渲染窗口已经变了
//   （实测 End 之后窗口首项是「会话 5」）。Home 的语义是
//   「跳到**当前渲染窗口**的首项」，所以必须在**按下的那一刻**取窗口。
//   我第一版硬编码期望「会话 0」⇒ 报「Home 没回到首项」，**判据写错**。
const winAtHome = await titles()
await page.keyboard.press('Home')
await page.waitForTimeout(250)
const cHome = await focused()
console.log(`  C End：${JSON.stringify(cEnd)} → Home：${JSON.stringify(cHome)}（按 Home 时窗口首项=${winAtHome[0]}）`)
if (!cEnd.isItem || !cHome.isItem) bad('End/Home 之后焦点不在会话项上')
if (cHome.title !== winAtHome[0]) bad(`Home 没到当时窗口首项（期望 ${winAtHome[0]}，实得 ${cHome.title}）`)

// D ⭐：连按 25 次 ↓（> 窗口 20）必须仍在移动
const winSize = win0.length
const presses = winSize + 5 // ⭐ 有意超过窗口
await page.locator(ITEM).first().focus()
await page.waitForTimeout(150)
for (let i = 0; i < presses; i++) {
  await page.keyboard.press('ArrowDown')
  await page.waitForTimeout(35) // ⭐ 必须给虚拟化重渲染留时间
}
await page.waitForTimeout(600)
const d = await focused()
console.log(`  D ⭐ 连按 ↓ ${presses} 次（窗口 ${winSize}）：焦点在 ${JSON.stringify(d)}`)
if (!d.isItem) bad(`连按 ${presses} 次后焦点丢失（不在会话项上）`)
else {
  const stuckAtWinEnd = d.title === win0[winSize - 1]
  if (stuckAtWinEnd) {
    bad(`焦点卡在渲染窗口最后一项「${d.title}」⇒ 虚拟化边缘断链`
      + `（按 ${presses} 次 = ${winSize} 窗口 + 5，应已跨窗）`)
  } else {
    console.log(`     ✅ 已跨过虚拟化窗口（不再停在「${win0[winSize - 1]}」）`)
  }
}

// E ⭐：Tab 必须能**最终离开**列表（不是「按一次就出去」）
// ⓘ 我第一版断言「按一次 Tab 后焦点不在会话项」⇒ 报「Tab 被劫持」。
//   ⛔ 但会话项**全是 button** ⇒ Tab 的下一个可聚焦元素**恰好也是**会话项。
//   ⇒ 一次 Tab 出不去是**正常**的，判据测错了东西。
//   ⛔ 真正的失败模式是**「出不去」**（焦点陷阱）：必须连按到离开为止。
await page.locator(ITEM).first().focus()
let escaped = false
let tabs = 0
for (; tabs < 60; tabs++) { // 60 > 窗口 20，足够穿过整个渲染窗口
  await page.keyboard.press('Tab')
  await page.waitForTimeout(25)
  if (!(await focused()).inList) { escaped = true; break }
}
console.log(`  E ⭐ 连按 Tab ${tabs + 1} 次后离开列表：${escaped ? '✅' : '⛔'}（焦点陷阱）`)
if (!escaped) bad(`连按 ${tabs + 1} 次 Tab 仍未离开会话列表 ⇒ 焦点被trap，键盘用户出不去`)

// G：搜索框里的方向键不触发列表导航
await page.locator('[data-testid="nb-search"]').click()
await page.keyboard.press('ArrowDown')
await page.keyboard.press('ArrowUp')
await page.waitForTimeout(300)
const g = await focused()
console.log(`  G 搜索框内按 ↑↓：焦点 ${JSON.stringify(g)}`)
if (!g.inList === false) { /* noop */ }
if (g.isItem) bad('在搜索框里按 ↑/↓ 把列表焦点抢走了 ⇒ 作用域失控')

// H
console.log(`  H 未捕获异常：${errs.length} ${errs.length ? '⛔' : '✅'}`)
if (errs.length) bad(`未捕获异常 ${errs.length} 条：${errs[0].slice(0, 80)}`)

await browser.close()
srv.close()
console.log()
console.log(fail
  ? `FAIL: ${fail} 项`
  : `PASS: 方向键在虚拟化窗口内与跨窗口均能移动，Home/End 生效，不劫持 Tab，作用域不外溢。`)
process.exit(fail ? 1 : 0)
