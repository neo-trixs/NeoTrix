/**
 * 主题持久化门 —— 统一队列 #5（`get_dsh_theme` 命名债）的验收。
 *
 * # 它守的是一条**曾经真实存在**的缺陷
 *
 * `desktop.rs:33` 的 `get_dsh_theme()` **恒返回 `Theme::System`**（写死的常量）。
 * 而 `theme-mode.ts` 原先在挂载时 `setThemeMode(后端值)`
 * ⇒ **每次启动都把用户存的主题覆盖成「跟随系统」**
 * ⇒ **主题选择根本不持久**：选了深色，下次打开又变浅。
 *
 * 根因是**自相矛盾**：该模块自己在注释里写下「后端没有写入口 ⇒ 前端是真源」，
 * 却仍在实现里读后端。契约表确实**没有 `set_theme`**。
 *
 * # 判据（先定）
 *
 * |  | 判据 | 理由 |
 * |---|------|------|
 * | A | 选「浅色」→ **刷新后仍是浅色** | 这就是那条缺陷本身 |
 * | B | 选「深色」→ **刷新后仍是深色** | 双向，避免只测一档 |
 * | C | 主题选择器存在且可选 | 没有它谈不上持久 |
 * | D | 自持树**不再调用** `get_dsh_theme` | 命名债已清（代码引用，非注释） |
 *
 * ⛔ **A/B 必须跨「刷新」**：只在同一次会话里比较，
 *   那个覆盖 bug 根本不会显现（它发生在**挂载时**）。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { neobotDistFresh } from './nt_dist_freshness.mjs'

/** ⭐⭐⭐ 2026-10-03 主题/语言控件已移进**设置弹窗**（消除顶栏冗余）。
 * ⭐⭐ 因此控件**不在常驻 DOM** ⇒ 直接 `selectOption` 会失败；
 * 而原调用点 9 处有 6 处是 `.catch(() => {})` ⇒ ⭐⭐ **失败被静默吞掉**，
 * 测试会「通过」却**什么都没测**。
 *
 * ⭐⭐⭐ **实测踩到的第二个坑（本条注释就是它留下的）**：
 *   只「打开」不「关闭」⇒ ⭐⭐ **模态遮罩残留** ⇒ 后续
 *   「日志弹窗未打开 / 活动面板未渲染」等判据**全被挡住**。
 *   实测：smoke 失败项由 **4 → 10**，其中 6 项**与主题/语言无关**，
 *   ⭐⭐ **纯属遮罩残留的连带伤害** ⇒ ⇒ 必须 ⭐⭐**用完即关**。
 *   ⭐ 关闭走 Esc（`useModalBehaviour` 的 `:87`）⇒ ⛔ 不依赖文案与类名。*/
async function ensurePrefs(pg) {
  if (await pg.$('[data-testid="nb-theme-select"]')) return
  await pg.click('[data-testid="nb-settings-open"]')
  await pg.waitForSelector('[data-testid="nb-theme-select"]', { timeout: 5000 })
}

/** ⭐ 选完**立刻关**设置弹窗 ⇒ ⭐ 不留遮罩污染后续步骤。*/
async function closePrefs(pg) {
  if (!(await pg.$('[data-testid="nb-settings-close"]'))) return
  await pg.keyboard.press('Escape')
  await pg.waitForSelector('[data-testid="nb-settings-close"]', { state: 'detached', timeout: 5000 })
}

/** ⭐⭐ 设定一个偏好并关窗（⭐ 组合调用，避免各处再忘关）。*/
async function setPref(pg, testid, value) {
  await ensurePrefs(pg)
  await pg.selectOption(`[data-testid="${testid}"]`, value)
  await closePrefs(pg)
}


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
neobotDistFresh('neobot-check-theme-persist');
const PORT = 8861
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

const STUB = {
  neobot_convo_list: [{
    id: 'c0', kind: 'direct', title: '海豚调试', members: ['u1'], task_count: 0,
    last_active: '2026-10-01T00:00:00Z', muted: false, unread: 0,
  }],
  neobot_convo_messages: [{
    id: 'm1', convo_id: 'c0', role: 'user', text: '你好',
    created_at: '2026-10-01T00:00:00Z',
  }],
  neobot_member_list: [],
  neobot_usage_summary: { days: 1, tokens: 0 },
  neobot_memory_list: { lines: [], bytes: 0, cap: 0, revisions: 0 },
  neobot_core_capabilities: { crystal_version: '0', tool_count: 0, model: '', model_source: '' },
  log_frontend: null, set_language: null, read_run_logs: '', quit_app: null,
  open_external_url: null,
  // ⛔ 刻意仍提供它：模拟「后端还在、但自持 UI 不该问」的处境。
  //    若门不注入，桩缺这项反而会掩盖「是否还在调用」。
  get_dsh_theme: 'system',
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
let fail = 0
const bad = (m) => { console.log(`     ⛔ ${m}`); fail++ }

async function openPage(storage) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 820 } })
  if (storage) {
    await ctx.addInitScript((v) => {
      try { localStorage.setItem('neobot.theme', v) } catch { /* ignore */ }
    }, storage)
  }
  const page = await ctx.newPage()
  await page.addInitScript(initScript, { t: STUB })
  await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
  await page.waitForTimeout(1100)
  return { ctx, page }
}

const themeOf = (page) => page.evaluate(() => document.documentElement.getAttribute('data-theme'))
const hasSel = (page) => page.evaluate(
  () => !!document.querySelector('[data-testid="nb-theme-select"]'))

console.log('主题持久化门\n')

// C：选择器存在（⭐⭐ 2026-10-03：控件已移进**设置弹窗** ⇒ ⛔ 不在常驻 DOM。
//   ⛔ 改前直接 `hasSel(page)` 判存在 ⇒ ⭐⭐ **必然误判红**（去「修」一个没坏的判据）。
//   ⇒ 现在 ⭐⭐ **先开弹窗再判**，且弹窗打不开就 ⭐⭐ **抛错**（⛔ 不静默 pass）。）
{
  const { ctx, page } = await openPage(null)
  try {
    await ensurePrefs(page)
  } catch (e) {
    bad(`点「设置」后仍找不到主题选择器：${String(e).split('\n')[0]}`)
    await ctx.close()
    throw e
  }
  if (await hasSel(page)) console.log('  C 主题选择器存在 ✅（设置弹窗内）')
  else bad('打开设置弹窗后仍找不到 [data-testid="nb-theme-select"]')
  await ctx.close()
}

// A / B：选了之后**刷新**仍在
for (const [choice, label] of [['light', 'A 浅色'], ['dark', 'B 深色']]) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 820 } })
  const page = await ctx.newPage()
  await page.addInitScript(initScript, { t: STUB })
  await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
  await page.waitForTimeout(1100)
  await setPref(page, 'nb-theme-select', choice)
  await page.waitForTimeout(400)
  const beforeReload = await themeOf(page)
  // ⛔ 必须**刷新** —— 覆盖 bug 发生在挂载时，同会话内比较看不出来
  await page.reload({ waitUntil: 'load' })
  await page.waitForTimeout(1100)
  const afterReload = await themeOf(page)
  const ok = beforeReload === choice && afterReload === choice
  console.log(`  ${label}：选后 ${beforeReload} → **刷新后** ${afterReload} ⇒ ${ok ? '✅ 已持久' : '⛔ 未持久'}`)
  if (!ok) bad(`${label}：选 ${choice} 但刷新后是 ${afterReload}（覆盖 bug 未修）`)
  await ctx.close()
}

// D：自持树不再调用 get_dsh_theme
{
  const { ctx, page } = await openPage(null)
  const called = await page.evaluate(() => {
    // 记录本会话所有 invoke 的命令名（重新注入一个记录器后刷新）
    return window.__INVOKED__ || null
  })
  void called
  // 用更可靠的办法：给桩打桩计数
  const ctx2 = await browser.newContext({ viewport: { width: 1280, height: 820 } })
  const p2 = await ctx2.newPage()
  await p2.addInitScript((t) => {
    window.__INVOKED__ = []
    window.__TAURI_INTERNALS__ = {
      invoke: (c) => { window.__INVOKED__.push(c); return Promise.resolve(t[c] ?? null) },
      transformCallback: (cb) => { window._1 = cb; return 1 },
      metadata: { currentWindow: { label: 'main' } },
    }
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
  }, { t: STUB })
  await p2.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
  await p2.waitForTimeout(1200)
  const invoked = await p2.evaluate(() => window.__INVOKED__ || [])
  const bad_ = invoked.filter((c) => c === 'get_dsh_theme')
  console.log(`  D 命名债：本次会话共 invoke ${invoked.length} 次，get_dsh_theme ${bad_.length} 次`
    + ` ⇒ ${bad_.length === 0 ? '✅ 已清' : '⛔ 仍在调用'}`)
  if (bad_.length) bad(`自持 UI 仍调用 get_dsh_theme（DSH 命名债未清）：${bad_.length} 次`)
  await p2.close(); await ctx2.close(); await ctx.close()
}

await browser.close()
srv.close()
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: 主题选择跨刷新持久；自持 UI 不再依赖 DSH 命名命令。')
process.exit(fail ? 1 : 0)
