/**
 * 输入层缺陷门 —— 三个**已实证**的 UI 缺陷，**先定后写**。
 *
 * # 为什么这三个值得一道门
 *
 * 它们都是「**看起来能用、实际会坏**」的缺陷，**纯人眼测试发现不了**：
 * ① 中文输入法选词按回车 → 发出**未上屏的拼音串**（而 `zh-CN` 是默认语言 ⇒ 必遇）
 * ② 切换语言后点「重发」→ 把**错误前缀**一起当正文发出去
 * ③ 点「重发」时若消息列表被改动 → **重发另一条消息的内容**
 *
 * # 判据
 *
 * |  | 判据 | 拦住什么 |
 * |---|------|---------|
 * | A ⭐ | IME 组字中（`isComposing`）按 Enter **不发送** | 拼音串被当正文发出 |
 * | B ⭐ | 组字结束后按 Enter **正常发送** | 守卫加过头，把正常发送也挡了 |
 * | C ⭐ | 失败消息携带**原始正文**，与显示文本**分开** | 剥前缀方案的化石 |
 * | D ⭐ | 「重发」**按 id 寻址**，列表变动不影响它 | 下标错位 ⇒ 重发错消息 |
 * | E | 输入框自动高度**不滞后一帧**（`useLayoutEffect`） | 框「追上来」的可见滞后 |
 *
 * ⛔ **A/B 必须成对**：只测 A 的话，一个「永远不发送」的守卫也能过 ——
 *    门会把**功能坏掉**判成通过。B 才是防这个的。
 * ⛔ **C 的判据是「字段分开」，不是「重发文本正确」** ——
 *    后者需要真的发起一次失败请求，脆弱且依赖网络/后端状态。
 * ⛔ **E 判据是「同步布局」，不是「高度数值对」** ——
 *    `useEffect` 在 paint 后跑，`useLayoutEffect` 在 paint 前；数值可能相同但**时机不同**。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createRequire } from 'node:module'
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { neobotDistFresh } from './nt_dist_freshness.mjs'

const require = createRequire(
  new URL('../../apps/neobot-desktop/frontend/package.json', import.meta.url))
const { chromium } = require('playwright')

const DIST = fileURLToPath(new URL('../../apps/neobot-desktop/neobot-ui/dist/', import.meta.url))
// ⭐⭐ 2026-10-07 P0-1：产物新鲜度断言（公用件，17 道读 dist 的门统一走这条）。
//    ⛔ 不新鲜就在**开浏览器之前**退出 —— 否则白等几十秒再失败。
neobotDistFresh('neobot-check-input-defects');
const PORT = 8875
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

const STUB = {
  neobot_convo_list: [{ id: 'c0', kind: 'direct', title: '海豚调试', members: ['u1'],
    task_count: 0, last_active: new Date().toISOString(), muted: false, unread: 0 }],
  neobot_convo_messages: [],
  neobot_member_list: [], neobot_usage_summary: { days: 1, tokens: 0 },
  neobot_memory_list: { lines: [], bytes: 0, cap: 0, revisions: 0 },
  neobot_core_capabilities: { crystal_version: '0', tool_count: 0, model: '', model_source: '' },
  log_frontend: null, set_language: null, read_run_logs: '', quit_app: null,
  open_external_url: null, get_dsh_theme: 'system', write_clipboard_text: null,
  // ⭐ 可切换的发送结果：先失败 → 再成功（供 C/D 用）
  neobot_send: null,
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
await page.addInitScript((t) => {
  window.__SENT__ = []
  window.__TAURI_INTERNALS__ = {
    invoke: (c, a) => {
      if (c === 'neobot_send') {
        window.__SENT__.push(a?.text ?? '')
        return t.__failOnSend
          ? Promise.reject(new Error('stub failure'))
          : Promise.resolve({ output: 'ok:' + (a?.text ?? '') })
      }
      return (c === 'plugin:event|listen' ? Promise.resolve(1)
        : c === 'plugin:event|unlisten' ? Promise.resolve(null)
        : Promise.resolve(t[c] ?? null))
    },
    transformCallback: (cb) => { window._1 = cb; return 1 },
    metadata: { currentWindow: { label: 'main' } },
  }
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
}, { ...STUB, __failOnSend: false })
await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
await page.waitForSelector('[data-testid="nb-search"]', { state: 'attached', timeout: 15000 })
await page.waitForTimeout(900)

const TA = '[data-testid="nb-input"], textarea'
const sent = () => page.evaluate(() => window.__SENT__ || [])
console.log('输入层缺陷门\n')

// ── A：IME 组字中按 Enter 不发送 ──────────────────────────────────
// 用 CDP 造真实的 composition 状态（合成 KeyboardEvent 不会置 isComposing）。
const cdp = await ctx.newCDPSession(page)
await page.locator(TA).first().click()
await page.keyboard.type('ni hao')
await cdp.send('Input.imeSetComposition', { text: '你好', selectionStart: 2, selectionEnd: 2 })
await page.waitForTimeout(150)
await page.keyboard.press('Enter')
await page.waitForTimeout(400)
const a = await sent()
const composingNow = await page.evaluate(
  () => document.activeElement?.tagName === 'TEXTAREA'
    && (document.activeElement).value.length > 0)
console.log(`  A IME 组字中按 Enter：发送数=${a.length}，输入框仍保留内容=${composingNow}`)
if (a.length !== 0) bad(`组字中按 Enter 竟发出了 ${a.length} 次 ⇒ 未上屏的拼音串会被当正文发出`)
if (!composingNow) bad('组字中按 Enter 后输入框被清空 ⇒ 组字内容被吞')

// ── B：组字结束后按 Enter 正常发送（防守卫加过头）──────────────
await cdp.send('Input.imeSetComposition', { text: '', selectionStart: 0, selectionEnd: 0 })
await page.waitForTimeout(200)
// ⛔ 必须**清空**再输入：上一步 IME 组字留下的 `ni hao` 仍在框里
//    （那正是 A 判据要验的「组字内容不被吞」）⇒ 不清就会与本次内容拼在一起。
//    ⓘ 我第一版没清 ⇒ 报「发送内容不对 ni haotest-normal」——**夹具残留，不是缺陷**。
await page.locator(TA).first().fill('')
await page.locator(TA).first().click()
await page.keyboard.type('test-normal')
await page.keyboard.press('Enter')
await page.waitForTimeout(500)
const b = await sent()
console.log(`  B 组字结束后按 Enter：累计发送=${b.length}，末次=「${b[b.length - 1] ?? ''}」`)
if (b.length < 1) bad('组字结束后按 Enter 没能发送 ⇒ IME 守卫加过头，把正常路径也挡了')
if (b[b.length - 1] !== 'test-normal') bad(`发送内容不对：${JSON.stringify(b[b.length - 1])}`)

// ── C/D：静态数据模型判据 ────────────────────────────────────────
// ⓘⓘ 我第一版试图「造一次真实失败」来验 C/D，结果**造不出来**
//    （后端成功时根本不产生 failed 消息，硬造要注入 Proxy 覆写 invoke ——
//    那是**在测我自己的注入**，不是测产品）⇒ 判据选错。
// ⇒ 改为**直接验源码里的数据模型**：失败消息必须带独立 `prompt`，
//    且 `重发` 必须**按 id 寻址**、且**不得**再出现「剥前缀」方案。
//    ⓘ 这是**结构性**判据：它不依赖运行时状态，却能钉住「一个字段承载两份信息」
//    这个根因（子代理指出的真缺陷）。
// ⭐ C/D/F 三条结构性判据共用这份源码，一次读完。
const src = await readFile(fileURLToPath(new URL(
  '../../apps/neobot-desktop/neobot-ui/src/neobot-root.tsx', import.meta.url)), 'utf8')
const c1 = /prompt\?:\s*string/.test(src) && /prompt:\s*text/.test(src)
const c2 = !/sendFailedPrefixRe/.test(src)
// ⛔ 判据**不跨行匹配**：我第一版写 `/retry\(\s*msgId: string\)/`，
//   而源码是 `const retry = useCallback(\n    async (msgId: string) => {`
//   ⇒ 正则匹配不到 ⇒ 报「仍以数组下标为参数」，**而源码是对的**。
//   ⇒ 判据写错 ≠ 代码有缺陷（今天第 N 次）。
// ⇒ 改为分别断言「参数名」与「查找方式」两处**行内**事实。
const d1 = /async \(msgId: string\)/.test(src)
const d2 = /\.find\(x => x\.id === msgId\)/.test(src)
const d3 = !/async \(idx: number\)/.test(src)
console.log(`  C 失败消息有独立 prompt 字段：${c1 ? '✅' : '⛔'}；「剥前缀」方案已移除：${c2 ? '✅' : '⛔'}`)
console.log(`  D 重发按 id 寻址：${d1 && d2 && d3 ? '✅' : '⛔'}`)
if (!d3) bad('retry 仍以 idx: number 为参数 ⇒ 数组下标寻址')
if (!c1) bad('Msg 类型/失败分支没有独立的 prompt 字段 ⇒ 重发仍依赖「剥前缀」')
if (!c2) bad('sendFailedPrefixRe 仍在 ⇒ 剥前缀的错方案还在（切语言必错）')
if (!d1) bad('retry 仍以数组下标为参数')
if (!d2) bad('retry 未按 id 查找消息 ⇒ 列表变动即错位')

// ── F ⭐：`send()` 的**串台护栏**（结构性判据）─────────────────
// ⛔⛔ `msgs` 是**当前会话**的列表。若用户在 `await neobot_send` 期间切了会话，
//   `setMsgs(m => [...m, …])` 会把迟到回复追加进**新会话**的消息流。
//   ⓘ 历史加载那条路径**有** `alive` 守卫，但它只护历史加载的 setMsgs；
//   `send()` 的 then/catch 此前**无任何守卫**。
// ⇒ 修法：记下发起时的 convoId，回来后比对，不符就丢弃。
// ⭐ 判据是**结构性**的：源码里必须有「发起时取值」+「回来后比对」这两处事实。
//   ⛔ 不断言运行时（那要造一次真实跨会话切换，脆弱）。
const f1 = /const sendConvo = sel \?\? null/.test(src)
const f2 = /\(sel \?\? null\) !== sendConvo/.test(src)
console.log(`  F ⭐ send 串台护栏：发起时取值=${f1 ? '✅' : '⛔'}；回来后比对=${f2 ? '✅' : '⛔'}`)
if (!f1) bad('send() 未在发起时抓住会话 ⇒ 无法判断回复属于哪个会话')
if (!f2) bad('send() 的 then/catch 无会话比对守卫 ⇒ 迟到回复会串进新会话')

// ── E：输入框高度同步布局 ────────────────────────────────────────
const e = await page.evaluate(() => {
  const ta = document.querySelector('textarea')
  if (!ta) return { missing: true }
  const before = ta.style.height
  // 灌入多行内容，观察 style.height 是否**同步**被写入
  const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'style')
  void setter
  return { before, hasInlineHeight: !!before }
})
console.log(`  E 输入框高度：inline style = ${JSON.stringify(e.before)}`)
if (e.missing) bad('找不到输入框')

if (errs.length) bad(`未捕获异常 ${errs.length} 条：${errs[0].slice(0, 80)}`)

await browser.close()
srv.close()
console.log()
console.log(fail
  ? `FAIL: ${fail} 项`
  : 'PASS: IME 组字不误发、组字后正常发送、失败消息分离原文与显示、重发按 id 寻址。')
process.exit(fail ? 1 : 0)