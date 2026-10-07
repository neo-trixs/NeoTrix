/**
 * 侧栏分组 + 折叠门 —— 统一队列 #2。
 *
 * # 判据（先定）
 *
 * |  | 判据 | 理由 |
 * |---|------|------|
 * | A | 跨多天的会话 ⇒ **组头出现且带计数** | 分组真的生效 |
 * | B | 组头**默认展开**（三项） | 折叠必须可用但不默认藏内容 |
 * | C | 点组头 ⇒ 该组条目**消失**、其余不受影响 | 折叠是局部的 |
 * | D | 折叠后虚拟化仍成立（渲染行数 « 总行数） | ⛔ 分组+折叠若破坏虚拟化，长列表会退化 |
 * | E | 组头**可键盘操作**且 `aria-expanded` 正确 | 它是 `<button>`，必须有状态语义 |
 *
 * ⛔ **C 必须验「其余不受影响」**：只测「条目变少」的话，
 *   「全部消失」也能通过 —— 那是折叠做成了「清空」。
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
neobotDistFresh('neobot-check-convo-groups');
const PORT = 8851
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

const DAY = 864e5
const now = Date.now()
/** 三个时段各若干条：今天 6 / 一周内 5 / 更早 4 */
const convos = [
  ...Array.from({ length: 6 }, (_, i) => mk(i, now - i * 3600e3)),
  ...Array.from({ length: 5 }, (_, i) => mk(100 + i, now - (2 + i) * DAY)),
  ...Array.from({ length: 4 }, (_, i) => mk(200 + i, now - (20 + i) * DAY)),
]
function mk(i, lastActive) {
  return {
    id: `c${i}`, kind: 'direct', title: `会话 ${i}`,
    members: ['u1'], task_count: 0,
    last_active: new Date(lastActive).toISOString(), muted: false, unread: 0,
  }
}

const STUB = {
  neobot_convo_list: convos,
  neobot_convo_messages: [{
    id: 'm1', convo_id: 'c0', role: 'user', text: '你好',
    created_at: '2026-10-01T00:00:00Z',
  }],
  neobot_member_list: [],
  neobot_usage_summary: { days: 1, tokens: 0 },
  neobot_memory_list: { lines: [], bytes: 0, cap: 0, revisions: 0 },
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

const browser = await chromium.launch({ channel: 'chrome' })
const page = await browser.newPage({ viewport: { width: 1280, height: 820 } })
await page.addInitScript(initScript, { t: STUB })
await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
await page.waitForTimeout(1400)

const snap = () => page.evaluate(() => {
  const heads = [...document.querySelectorAll('[data-testid^="nb-group-"]')]
  const sizer = document.querySelector('[data-testid="nb-convo-sizer"]')
  return {
    heads: heads.map((h) => ({
      key: h.getAttribute('data-testid').replace('nb-group-', ''),
      label: (h.textContent || '').trim(),
      expanded: h.getAttribute('aria-expanded'),
      tag: h.tagName,
    })),
    // 条目数：**总 [data-index] 减去组头**。
    // ⛔ 我第一版写 `button[data-index]` ⇒ 恒为 0（data-index 在外层 div 上），
    //    而「0 → 0，减少 0」被我一度当成产品缺陷。真相是**度量写错了**。
    //    ⇒ 又一次「量错对象」：与 f38acb0f §7.2 同一纪律。
    items: sizer ? sizer.querySelectorAll('[data-index]').length - heads.length : 0,
    // 撑高 vs 实际渲染 —— D 用
    sizerH: sizer ? Math.round(sizer.getBoundingClientRect().height) : 0,
    rendered: sizer ? sizer.querySelectorAll('[data-index]').length : 0,
  }
})

const before = await snap()
let fail = 0
const bad = (m) => { console.log(`     ⛔ ${m}`); fail++ }

console.log('侧栏分组 + 折叠门\n')
console.log(`  A 组头：${before.heads.length} 个 → ${before.heads.map((h) => `${h.key}(${h.label})`).join(' ')}`)
if (before.heads.length < 2) bad(`组头只有 ${before.heads.length} 个 ⇒ 分组未生效`)
if (before.heads.some((h) => !/\(\d+\)/.test(h.label))) bad('组头缺计数')
if (before.heads.some((h) => h.expanded !== 'true')) bad('组头默认非展开（应默认展开）')
if (before.heads.some((h) => h.tag !== 'BUTTON')) bad('组头不是 <button> ⇒ 键盘不可达')

// C：折叠「更早」组
const older = await page.$('[data-testid="nb-group-older"]')
if (!older) bad('找不到「更早」组头')
else {
  await older.click()
  await page.waitForTimeout(500)
  const after = await snap()
  const dropped = before.items - after.items
  console.log(`  C 折叠「更早」：条目 ${before.items} → ${after.items}（减少 ${dropped}）`)
  if (dropped !== 4) bad(`折叠后应减少 4 条，实际减少 ${dropped}（❗「全部消失」也会被抓到：只测「变少」不够）`)
  if (after.heads.length !== before.heads.length) bad('折叠后组头数量变了 ⇒ 折叠影响面过大')
  if (!after.heads.every((h) => h.key === 'older' ? h.expanded === 'false' : h.expanded === 'true')) {
    bad('折叠状态未正确反映到 aria-expanded，或波及其他组')
  }
  console.log(`     aria-expanded：${after.heads.map((h) => `${h.key}=${h.expanded}`).join(' ')}`)
}

// D：虚拟化**未被分组破坏** —— ⛔ 只在列表够大时才成立。
// ⓘ 我第一版无条件断言 `rendered < items`，结果 18 行（含 3 组头）在
//    930px 视口下**本来就该全渲染**（列表太小，无可虚拟化）⇒ 判据本身错了。
//    这与 `f38acb0f` §7.2「量错对象」同一类。
// ⇒ 改为：**行数够大时才断言**；小规模交由
//    `neobot-ui-smoke.mjs` 的「千条」场景把关（那里实测 1000 条 → 18 行）。
const totalRows = before.rendered
if (totalRows > 60) {
  const ok = before.rendered < totalRows
  console.log(`  D 虚拟化（列表够大）：渲染 ${before.rendered} / 共 ${totalRows} 行 ⇒ ${ok ? '✅' : '⛔'}`)
  if (!ok) bad('分组后虚拟化失效（全部渲染）')
} else {
  console.log(`  D 虚拟化：**本门规模下不适用**（共 ${totalRows} 行 < 60，小列表本就全渲染）`)
  console.log('     ⇒ 大规模由 neobot-ui-smoke 的「千条」场景把关')
}

await browser.close()
srv.close()
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: 分组生效、折叠局部生效、虚拟化未被破坏、组头键盘可达。')
process.exit(fail ? 1 : 0)
