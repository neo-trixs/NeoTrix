/**
 * 气泡悬停/键盘复制门 —— 判据 A/B/C/D/E，**先定后写**。
 *
 * # 它守什么
 * 消息气泡右上角的复制按钮：悬停出现、键盘可达、复制真生效、
 * 且**不改变气泡高度**。
 *
 * # ⭐ 本门第一版崩在**自己的桩**上（不是产品）
 *
 * 症状：`TypeError: Cannot read properties of null (reading '0')`，
 * React 根本没挂载（`#root` 为空）⇒ 门报「按钮未渲染」。
 *
 * 真因：我的 `invoke` 桩**漏了 Tauri 事件插件**：
 * ```js
 *   // 错的（我写的）
 *   invoke: (c, a) => Promise.resolve(t[c] ?? null)     // listen ⇒ null
 *   // 对的（neobot-msg-virtual.mjs 的，已被两轮证明能渲染）
 *   invoke: (c) => (c === 'plugin:event|listen' ? Promise.resolve(1) ...
 * ```
 * `listen()` 必须拿到一个**数字**事件 id；返回 `null` 后应用继续解引用
 * ⇒ 挂载期崩。`neobot-msg-virtual.mjs` 的注释写着**这个坑作者栽了第 4 次**
 * （每写一个新脚本就重犯），我照搬时把它丢了。
 *
 * ⇒ **本门因此内建「零未捕获异常」断言（判据 F）**：
 *    桩不完整会**先**在这里报出来，而不是伪装成「功能没渲染」。
 *    上一版缺的正是这条，它让我把桩 bug 误判成了产品缺陷。
 *
 * # 判据（先定）
 *
 * |  | 判据 | 为什么 |
 * |---|------|--------|
 * | A | 默认 `opacity: 0` **且** `pointer-events: none` | 不占视觉、不误触 |
 * | B | 悬停 ⇒ 出现 | 功能本体 |
 * | C | **`focus-within`** 也让它出现 | ⛔ 只做 hover ⇒ 键盘用户永远看不到 |
 * | D | 复制**真写进剪贴板**（读回内容） | ⛔ 「有按钮」≠「能用」 |
 * | E | 加按钮**不改变气泡高度** | ⛔ `measureElement` 量的就是它 |
 * | F | **零未捕获异常** | 桩/环境错会伪装成「功能没渲染」 |
 *
 * ⛔ **C 不可省**：`opacity: 0` 的元素**仍在 Tab 序里**，
 *    完全不可见 ⇒ 对键盘用户**比没有更糟**。
 * ⛔ **E 是结构性的**：按钮若进文档流，会污染虚拟化测到的高度与滚动条。
 * ⛔ **D 不接受「点击没报错」当通过**，必须读回内容。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'

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
const PORT = 8823
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

// ⛔ 桩形状照抄 neobot-msg-virtual.mjs（已被证明可渲染）。**不要**随手改成
//    「1 个会话 + 1 条消息」去缩小规模 —— 那会走进另一条未覆盖的分支。
const N = 1000
const MARK = 'COPYME-ABC-123'
const convos = Array.from({ length: N }, (_, i) => ({
  id: `c${i}`, kind: 'direct', title: `会话 ${i}`,
  members: ['u1'], task_count: 0,
  last_active: new Date(Date.now() - i * 6e4).toISOString(),
  muted: false, unread: 0,
}))
const msgs = Array.from({ length: N }, (_, i) => ({
  id: `m${i}`, convo_id: 'c0', role: i % 2 ? 'assistant' : 'user',
  text: i === 0 ? `首条 ${MARK}` : `第 ${i} 条消息`.repeat(3),
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
  write_clipboard_text: null, // 复制降级链第二跳（Tauri）
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
// ⛔ 用 newContext 授真剪贴板权限：判据 D 要**读回真实内容**。
const ctx = await browser.newContext({
  viewport: { width: 1280, height: 820 },
  permissions: ['clipboard-read', 'clipboard-write'],
})
const page = await ctx.newPage()
const pageErrors = []
page.on('pageerror', (e) => pageErrors.push(e.message))
// ⛔ 降级链命中记录：用来区分「走浏览器 API」还是「回落 Tauri」
const tauriClip = []
await page.addInitScript((t) => {
  window.__TAURI_CLIP__ = []
  window.__TAURI_INTERNALS__ = {
    invoke: (c, a) => {
      if (c === 'write_clipboard_text') { window.__TAURI_CLIP__.push(a?.text ?? ''); return Promise.resolve(null) }
      return (c === 'plugin:event|listen' ? Promise.resolve(1)
        : c === 'plugin:event|unlisten' ? Promise.resolve(null)
        : Promise.resolve(t[c] ?? null))
    },
    transformCallback: (cb) => { window._1 = cb; return 1 },
    metadata: { currentWindow: { label: 'main' } },
  }
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
}, STUB)
await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
// ⛔ 确定性等待：按钮出现即继续。固定 sleep 是竞态（上一版 1400ms 就踩了）。
const SEL = '[data-testid="nb-msg-copy"]'
const appeared = await page.waitForSelector(SEL, { timeout: 15000 }).then(() => true).catch(() => false)
await page.waitForTimeout(400)

let fail = 0
const bad = (m) => { console.log(`     ⛔ ${m}`); fail++ }
console.log('气泡悬停/键盘复制门\n')

// ── F 先判：桩/环境错会伪装成「功能没渲染」，必须先排除 ──────────────
const rootOk = await page.evaluate(() => (document.getElementById('root')?.children.length ?? 0) > 0)
if (!rootOk) bad('#root 为空 ⇒ React 未挂载：**桩或环境有问题**，不是功能缺陷（先查 invoke 是否处理 plugin:event|listen）')
if (pageErrors.length) bad(`页面有 ${pageErrors.length} 条未捕获异常：${pageErrors[0].slice(0, 90)}`)

if (!appeared) {
  bad(`等不到 ${SEL}`)
} else {
  // ⛔⛔⛔ 必须挑一个**真在视口内**的按钮，不能用「DOM 第一个」。
  //   我第一版用 `.first()` ⇒ 拿到的是**已滚出屏幕上方**的那条
  //   （诊断实测 `gRect=[388, -312, 274, 55]`，Y 为负）
  //   ⇒ 鼠标移到 y=-312 落空 ⇒ `.group:hover` 不触发 ⇒ opacity 0；
  //      点击也落空。看起来像「悬停功能坏了」，其实**一次都没点着过**。
  //   列表会**自动滚到底部**（见 neobot-msg-virtual 判据 C），
  //   所以 DOM 第一个 ≠ 屏幕第一个。
  //   ⓘ 期望值也不用硬编码 MARK，而是取**被点那条自己的气泡文本** ——
  //      否则虚拟化/排序一变就假失败（我第一版就栽在这：门读回
  //      「第 962 条消息」而我断言的是第 0 条的 COPYME）。
  const vh = page.viewportSize().height
  // ⓘ 判据再加一层：**中心点最顶层必须就是它**（或其后代）。
  //   只判「在视口内」还不够 —— 列表会从**吸顶栏下方**滚过，
  //   顶部那条虽然在视口内，却被 `h-12 shrink-0 border-b` 的栏**盖住**
  //   （实测落点 = 该栏）⇒ 鼠标中心落在栏上 ⇒ `.group:hover` 不触发。
  //   这不是产品缺陷（吸顶栏压住滚动内容是正常的），
  //   但它意味着「那条消息此刻**点不到**」—— 真实用户也不会去点被盖住的地方。
  const pick = await page.evaluate(({ sel, vh }) => {
    const all = [...document.querySelectorAll(sel)]
    for (const el of all) {
      const r = el.getBoundingClientRect()
      if (r.top < 0 || r.bottom > vh || r.width <= 0) continue
      const cx = r.x + r.width / 2, cy = r.y + r.height / 2
      const top = document.elementFromPoint(cx, cy)
      const g = el.closest('.group')
      if (!top || !g || !(g === top || g.contains(top) || top.contains(g))) continue
      return { idx: all.indexOf(el), y: Math.round(r.y),
               // ⛔ 期望值必须取**消息正文**，不能取整个气泡的 innerText：
               //   气泡里还含复制按钮的 `⧉` 字形 ⇒ innerText = "⧉\n第 990 条…"
               //   ⇒ 拿它去比对剪贴板（只含正文）**必然不匹配**。
               //   我第一版就栽在这：门明明复制成功了却报「未写入」。
               text: (el.closest('[class*="rounded-2xl"]')
                        ?.querySelector('[class*="whitespace-pre-wrap"], .nb-md')
                        ?.innerText ?? '').trim() }
    }
    return null
  }, { sel: SEL, vh })
  if (!pick) bad('找不到**可悬停**的复制按钮 ⇒ 判据无法执行')
  if (pick && !pick.text) bad('取不到消息正文 ⇒ 期望值无法确定（选择器可能已变）')
  const first = page.locator(SEL).nth(pick?.idx ?? 0)
  const WANT = pick?.text ?? ''
  console.log(`  （DOM 共 ${await page.locator(SEL).count()} 个；取**可悬停**第 ${pick?.idx} 个（y=${pick?.y}），`
    + `其气泡文本 ${JSON.stringify(WANT.slice(0, 24))}）`)

  // A：默认不可见
  const a = await first.evaluate((el) => {
    const cs = getComputedStyle(el)
    return { opacity: Number(cs.opacity), pe: cs.pointerEvents, pos: cs.position }
  })
  console.log(`  A 默认态：opacity=${a.opacity} pointer-events=${a.pe} position=${a.pos}`)
  if (a.opacity > 0.05) bad(`按钮默认可见（opacity=${a.opacity}）⇒ 未隐藏，会一直占视觉`)
  if (a.pe !== 'none') bad(`默认仍吃点击（pointer-events=${a.pe}）`)
  // ⛔ E 的前置：必须绝对定位，否则会进文档流
  if (a.pos !== 'absolute') bad(`position=${a.pos}（非 absolute ⇒ 进文档流 ⇒ 改变气泡高度，污染虚拟化测量）`)

  const bubbleH = () => first.evaluate((el) => Math.round(
    el.closest('[class*="rounded-2xl"]')?.getBoundingClientRect().height ?? 0))
  const hBefore = await bubbleH()

  // B：悬停**气泡**⇒ 按钮浮现
  // ⛔⛔ 必须悬停**气泡（.group）**，不能悬停按钮自己：
  //   按钮隐藏时 `pointer-events: none` ⇒ 鼠标**永远落不到它**
  //   ⇒ Playwright 的 hover()/click() 会一直等「元素可接收事件」然后超时。
  //   我第一版 `first.hover()` 就是这么错的（还被 `.catch()` 吞掉 ⇒ 报 opacity=0，
  //   看起来像「悬停功能坏了」，其实**一次都没悬停上**）。
  //   真实用户的路径是：悬停消息 → 按钮出现 → 点它。门必须走同一条路。
  await first.evaluate((el) => {
    const g = el.closest('.group') ?? el
    g.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))
    g.dispatchEvent(new MouseEvent('mouseenter', { bubbles: false }))
  }).catch(() => {})
  // 用真实鼠标移到气泡中心（:hover 是真实指针状态，派发事件**不会**触发它）
  const bubble = first.locator('xpath=ancestor::div[contains(@class,"group")][1]')
  await bubble.hover({ force: true }).catch(() => {})
  await page.waitForTimeout(400)
  const bOp = await first.evaluate((el) => Number(getComputedStyle(el).opacity))
  // ⓘ 诊断：鼠标到底落在哪、.group 是否真被 :hover 命中
  const hDiag = await first.evaluate((el) => {
    const g = el.closest('.group')
    const gr = g.getBoundingClientRect(); const br = el.getBoundingClientRect()
    return { groupHover: g.matches(':hover'),
             underCursor: (document.elementFromPoint(gr.x + gr.width / 2, gr.y + gr.height / 2) || {}).className?.toString?.().slice(0, 40),
             gRect: [Math.round(gr.x), Math.round(gr.y), Math.round(gr.width), Math.round(gr.height)],
             bRect: [Math.round(br.x), Math.round(br.y), Math.round(br.width), Math.round(br.height)] }
  })
  console.log(`     （诊断 groupHover=${hDiag.groupHover} gRect=${JSON.stringify(hDiag.gRect)} 落点=${hDiag.underCursor}）`)
  const bPe = await first.evaluate((el) => getComputedStyle(el).pointerEvents)
  console.log(`  B 悬停气泡：opacity=${bOp} pointer-events=${bPe}`)
  if (bOp < 0.9) bad(`悬停气泡后按钮未出现（opacity=${bOp}）`)
  if (bPe === 'none') bad(`按钮已可见但仍 pointer-events:none ⇒ 点不到`)

  // C：键盘可达（不依赖 hover）
  const cOp = await first.evaluate((el) => new Promise((res) => {
    el.focus()
    setTimeout(() => res(Number(getComputedStyle(el).opacity)), 350)
  }))
  console.log(`  C 键盘聚焦：opacity=${cOp}`)
  if (cOp < 0.9) bad(`键盘聚焦后仍不可见（opacity=${cOp}）⇒ 只做了 hover，键盘用户看不到`)

  // D：复制真生效 —— 读回剪贴板内容
  // ⓘ B 之后按钮已 pointer-events:auto ⇒ 此时点击才可达。
  //   先断言**点击确实触发了 handler**：应用在复制失败时是**静默**的
  //   （`copyText` 返回 false ⇒ 不加反馈类，这是有意的设计），
  //   所以「剪贴板空」无法区分「没点到」与「点到但失败」⇒ 必须先看信号。
  await first.click({ timeout: 5000 }).catch((e) => bad(`点击失败：${String(e).slice(0, 60)}`))
  await page.waitForTimeout(200)
  const gotFocus = await first.evaluate((el) => document.activeElement === el)
  await page.waitForTimeout(400)
  const clip = await page.evaluate(async () => {
    try { return await navigator.clipboard.readText() } catch (e) { return `ERR:${e}` }
  })
  const viaT = await page.evaluate(() => window.__TAURI_CLIP__ || [])
  const okReal = String(clip).includes(WANT.slice(0, 12))
  const okTauri = viaT.some((t) => String(t).includes(WANT.slice(0, 12)))
  // ⭐ 用户可见的反馈：成功才加 `is-copied`。没有它 = 用户按了**什么都没发生**。
  const flashed = await first.evaluate((el) => el.classList.contains('is-copied'))
  console.log(`  D 剪贴板：${okReal ? '✅ 真剪贴板含该条原文' : okTauri ? '✅ Tauri 降级链含原文' : '⛔ 未写入'}`
    + `（路径=${okReal ? '浏览器 API' : okTauri ? 'write_clipboard_text' : '无'}；已聚焦=${gotFocus}）`)
  console.log(`  D2 成功反馈 is-copied：${flashed ? '✅ 有' : '⛔ 无 ⇒ 用户按了看不到任何变化'}`)
  if (!gotFocus) bad('点击后按钮未获焦 ⇒ 事件可能没派发到按钮')
  if (!okReal && !okTauri) bad(`两条复制路径都没写入（readText=${JSON.stringify(String(clip).slice(0, 50))}）`)
  if (!flashed && (okReal || okTauri)) bad('复制成功但无 is-copied 反馈 ⇒ 用户不知道成功了')

  // E：高度未变
  const hAfter = await bubbleH()
  console.log(`  E 气泡高度：${hBefore}px → ${hAfter}px`)
  if (Math.abs(hAfter - hBefore) > 1) bad(`按钮改变了气泡高度（${hBefore}→${hAfter}）⇒ 污染虚拟化测量`)
}

// F 收尾（渲染后可能新增异常）
const lateErr = pageErrors.length
console.log(`  F 未捕获异常：${lateErr} 条 ${lateErr ? '⛔' : '✅'}`)
if (lateErr) { bad(`页面有 ${lateErr} 条未捕获异常`); fail += 0 }

await browser.close()
srv.close()
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: 悬停/键盘均可复制，剪贴板真写入，未改气泡高度，零未捕获异常。')
process.exit(fail ? 1 : 0)
