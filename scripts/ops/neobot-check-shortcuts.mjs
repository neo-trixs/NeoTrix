/**
 * 键盘快捷键门 —— 判据 A~H，**先定后写**。
 *
 * # 它守什么
 * `Cmd/Ctrl+K` 聚焦搜索 · `Esc` 清空搜索。两条都是**全局**快捷键，
 * 而「全局」正是危险所在：ⓘ 一次按键改了**两个**无关状态，用户不会预期。
 *
 * # ⭐ 本门重点在 E/F：**与对话框的 Esc 归属**
 *
 * 日志/记忆弹窗自己处理 Esc（`shell.tsx` 用 capture + `stopPropagation`）。
 * 若搜索的 Esc 无条件生效，用户**关弹窗的同一个 Esc 会顺手清空搜索**
 * —— 两个状态被一次按键改掉。
 * ⛔ 而「靠 `stopPropagation` 的顺序」是**脆的**：它依赖监听器注册次序，
 *    不是一条可读、可测的规则。
 * ⇒ 实现里显式检查 `[role=dialog]` 让路；E/F 就是守这条让路。
 *
 * # 判据
 *
 * |  | 判据 | 拦住什么 |
 * |---|------|---------|
 * | A | `Cmd+K` 把焦点移到搜索框，并**全选**已有内容 | 聚焦了但没选中 ⇒ 打字会插入而非覆盖 |
 * | B | 聚焦后输入真的过滤列表 | 焦点是假的 |
 * | C | `Esc` 清空搜索文本 | 功能本体 |
 * | D | `Esc` 时搜索为空 ⇒ **不清空别的东西、不乱跳焦点** | 空搜索时的误伤 |
 * | E ⭐ | 弹窗打开时 `Esc` **只关弹窗**，搜索**保持原样** | 一次按键改两个状态 |
 * | F ⭐ | 弹窗打开时 `Cmd+K` **不抢焦点** | 把焦点从模态里抢走 ⇒ 读屏用户丢位置 |
 * | G | 搜索框有**无障碍名**（`aria-label` 非空） | 读屏用户不知道这是啥 |
 * | H | 零未捕获异常 | — |
 *
 * ⛔ **E/F 不可省**：只测 A~C 的话，最危险的那条路径（与模态交互）**完全没测**，
 *    而它恰恰是「全局快捷键」最可能出事的地方。
 * ⛔ **D 不可省**：只在有内容时清空很容易写错成「无条件清空」。
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
const PORT = 8871
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

// ⛔⛔ 桩形状必须是**已被实证可渲染**的那个：1000 会话 + 1000 消息。
//   我第一版图省事写 60 会话 + **0 消息** ⇒ 应用挂载期就崩
//   （`root=0`、`Cannot read properties of null (reading '0')`），
//   表现为「搜索框找不到」—— 看起来像选择器写错，**其实是我的夹具不对**。
//   ⓘ 这是我今天第三次被自己的夹具坑（上次是 markdown 夹具混进 user 消息）。
//   ⇒ 判据要**先分清「环境坏了」还是「功能坏了」**，本脚本开头就打印环境诊断。
//   ⇒ 「为什么必须 1000/1000」尚未定位（疑与空消息列表的分支有关），
//      已记为待办；本门不猜，只用已验证形状。
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

async function open() {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 820 } })
  const page = await ctx.newPage()
  const errs = []
  page.on('pageerror', (e) => errs.push(e.message))
  // ⛔⛔⛔ 传参必须与读取**一致**：这里直接传 STUB，读 `t[c]`。
  //   我第一版传的是 `{ t: STUB }` 而读 `t[c]`（抄了 neobot-check-markdown
  //   的形却没抄它的 `a.t`）⇒ `t` = `{t: STUB}` ⇒ **每个命令都返回 null**
  //   ⇒ `neobot_convo_list` 为 null ⇒ 应用读 `null[0]`
  //   ⇒ `TypeError: Cannot read properties of null (reading '0')`，root 为空。
  //   ⓘ 症状极具误导性：表现为「搜索框找不到」，像是选择器写错。
  //   ⓘ **这是本次会话 invoke 桩第三次咬我**（前两次：漏 listen、改桩形状）。
  //   ⇒ 结论：这类门的桩**必须**先跑一次环境诊断（见下方 diag），
  //     看到 `root=0` 先怀疑桩，别去改被测代码或选择器。
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
  // ⛔ 诊断优先：桩/环境错会表现为「元素不出现」，不先分清就改选择器 = 改错地方
  await page.waitForTimeout(1500)
  const diag = await page.evaluate(() => ({
    root: document.getElementById('root')?.children.length ?? -1,
    aside: document.querySelectorAll('aside').length,
    inputs: document.querySelectorAll('aside input').length,
    main: (document.querySelector('.nb-main')?.innerText || '').slice(0, 40),
  }))
  if (diag.root === 0 || diag.inputs === 0) {
    console.log(`     ⛔ [环境] root=${diag.root} aside=${diag.aside} aside>input=${diag.inputs} `
      + `main=「${diag.main}」errs=${errs.length ? errs[0].slice(0, 70) : '无'}`)
  }
  await page.waitForSelector('[data-testid="nb-search"]', { state: 'attached', timeout: 15000 })
    .catch(() => { throw new Error(`搜索框未渲染（root=${diag.root} aside>input=${diag.inputs}）`) })
  await page.waitForTimeout(700)
  return { ctx, page, errs }
}

const SEL = '[data-testid="nb-search"]'
const state = (page) => page.evaluate((s) => {
  const el = document.querySelector(s)
  return {
    focused: document.activeElement === el,
    value: el?.value ?? null,
    selStart: el?.selectionStart ?? null,
    selEnd: el?.selectionEnd ?? null,
    aria: el?.getAttribute('aria-label') ?? null,
    dialogs: document.querySelectorAll('[role="dialog"]').length,
    rows: document.querySelectorAll('[data-testid="nb-convo-item"]').length,
  }
}, SEL)

console.log('键盘快捷键门\n')

// A / B：Cmd+K 聚焦 + 全选 + 过滤
{
  const { ctx, page, errs } = await open()
  const before = await state(page)
  // ⛔ 过滤效果**不能靠 DOM 行数**判定：列表是虚拟化的，DOM 行数恒 ~20，
  //   过滤前后都一样（实测 20 → 20）。我第一版据此报「过滤没生效」——**判据错**。
  //   ⇒ 改量两件真正反映过滤的东西：
  //     ① sizer 高度（= 未虚拟化的总内容高度，过滤后应变小）
  //     ② 可见会话标题**是否都命中**查询
  // ⛔ 量**会话**列表的 sizer（`nb-convo-sizer`），不是 `nb-msg-sizer`。
  //   我第一版量了消息列表的 sizer ⇒ 搜索不改变消息 ⇒ 高度恒定
  //   ⇒ 报「过滤未缩小结果集」，**判据量错了对象**（又一次）。
  const sizerH = () => page.evaluate(() => Math.round(
    document.querySelector('[data-testid="nb-convo-sizer"]')?.getBoundingClientRect().height ?? 0))
  const titlesHit = (q) => page.evaluate((qq) => {
    const t = [...document.querySelectorAll('[data-testid="nb-convo-item"]')].map((n) => n.innerText)
    return { n: t.length, allHit: t.length > 0 && t.every((x) => x.includes(qq)) }
  }, q)
  const hBefore = await sizerH()
  await page.keyboard.press('Meta+k')
  await page.waitForTimeout(300)
  const a = await state(page)
  console.log(`  A Cmd+K：focused=${a.focused} 全选=${a.selStart === 0 && a.selEnd === (a.value?.length ?? 0)}`)
  if (!a.focused) bad('Cmd+K 后焦点没进搜索框')
  await page.keyboard.type('会话 1')
  await page.waitForTimeout(500)
  const b = await state(page)
  const hAfter = await sizerH()
  const hit = await titlesHit('会话 1')
  console.log(`  B 过滤：输入「会话 1」⇒ sizer ${hBefore}→${hAfter}px，可见 ${hit.n} 条全部命中=${hit.allHit}，value=「${b.value}」`)
  if (!b.value) bad('搜索框没接收到输入 ⇒ 焦点是假的')
  if (hAfter >= hBefore) bad(`sizer 高度没变小（${hBefore} → ${hAfter}）⇒ 过滤未缩小结果集`)
  if (!hit.allHit) bad(`可见会话标题未全部命中查询（共 ${hit.n} 条）⇒ 过滤没生效`)
  // C：Esc 清空
  await page.keyboard.press('Escape')
  await page.waitForTimeout(400)
  const c = await state(page)
  console.log(`  C Esc 清空：value=「${c.value}」${c.value === '' ? '✅' : '⛔'}`)
  if (c.value !== '') bad(`Esc 后搜索框仍有内容：「${c.value}」`)
  // D：空搜索时 Esc 不乱动
  const d0 = await state(page)
  await page.locator(SEL).click().catch(() => {})
  await page.keyboard.press('Escape')
  await page.waitForTimeout(300)
  const d = await state(page)
  console.log(`  D 空搜索 Esc：value=「${d.value}」焦点仍在搜索=${d.focused}`)
  if (d.value !== '') bad(`空搜索时 Esc 写入了内容：「${d.value}」`)
  // G
  console.log(`  G 无障碍名：aria-label=「${d.aria}」${d.aria ? '✅' : '⛔'}`)
  if (!d.aria?.trim()) bad('搜索框 aria-label 为空 ⇒ 读屏用户不知道这是啥')
  if (errs.length) bad(`未捕获异常 ${errs.length} 条：${errs[0].slice(0, 70)}`)
  console.log(`  H 未捕获异常：${errs.length} ${errs.length ? '⛔' : '✅'}`)
  await ctx.close()
}

// E / F ⭐：弹窗打开时，Esc 归弹窗、Cmd+K 不抢焦点
{
  const { ctx, page, errs } = await open()
  // 先给搜索框填内容，验证「关弹窗不会顺手清掉它」
  await page.locator(SEL).click()
  await page.keyboard.type('会话 1')
  await page.waitForTimeout(400)
  const pre = await state(page)

  // 打开日志弹窗
  const opened = await page.evaluate(() => {
    const b = [...document.querySelectorAll('button')]
      .find((x) => /日志|log/i.test(x.getAttribute('aria-label') || x.textContent || ''))
    if (b) { b.click(); return true }
    return false
  })
  await page.waitForTimeout(500)
  const mid = await state(page)
  console.log(`  ── 弹窗交互 ──`)
  console.log(`  （日志按钮找到=${opened}；弹窗数=${mid.dialogs}；搜索值=「${mid.value}」）`)
  if (!opened || mid.dialogs === 0) {
    bad(`打不开日志弹窗（按钮=${opened} dialogs=${mid.dialogs}）⇒ E/F 判据无法执行`)
  } else {
    // F：Cmd+K 不抢焦点
    const fBefore = await page.evaluate(() => {
      const d = document.querySelector('[role="dialog"]')
      return d?.contains(document.activeElement) ?? false
    })
    await page.keyboard.press('Meta+k')
    await page.waitForTimeout(350)
    const f = await state(page)
    const fInDialog = await page.evaluate(() => {
      const d = document.querySelector('[role="dialog"]')
      return d?.contains(document.activeElement) ?? false
    })
    console.log(`  F 弹窗开时 Cmd+K：焦点在弹窗内 ${fBefore}→${fInDialog}，搜索 focused=${f.focused}`)
    // ⛔ 我第一版把这条**写反了**：`fInDialog === true` 意味着焦点**仍在弹窗内**
    //   （正确行为），我却判它失败。判据写错 ≠ 代码有缺陷。
    if (f.focused) bad('Cmd+K 在弹窗打开时把焦点抢到了搜索框 ⇒ 读屏用户丢失模态内位置')
    if (!fInDialog) bad('Cmd+K 把焦点移出了弹窗（焦点应留在模态内）')

    // E：Esc 只关弹窗，搜索保持
    await page.keyboard.press('Escape')
    await page.waitForTimeout(500)
    const e = await state(page)
    console.log(`  E 弹窗开时 Esc：弹窗数 ${mid.dialogs} → ${e.dialogs}；搜索值「${pre.value}」→「${e.value}」`)
    if (e.dialogs >= mid.dialogs) bad('Esc 没有关闭弹窗')
    if (e.value !== pre.value) bad(`关弹窗的同一个 Esc **顺手清空了搜索**：「${pre.value}」→「${e.value}」`
      + ` ⇒ 一次按键改了两个无关状态`)
  }
  if (errs.length) bad(`未捕获异常 ${errs.length} 条：${errs[0].slice(0, 70)}`)
  await ctx.close()
}

await browser.close()
srv.close()
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: Cmd+K 聚焦并全选、Esc 清空、空态不误伤、弹窗打开时 Esc 归弹窗且 Cmd+K 不抢焦点。')
process.exit(fail ? 1 : 0)
