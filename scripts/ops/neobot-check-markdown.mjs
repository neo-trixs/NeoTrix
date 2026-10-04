/**
 * markdown / 代码块门 —— 判据 A~H，**先定后写**。
 *
 * # ⭐ 它守的是一个**至今无人验证**的前提
 *
 * `neobot-root.tsx` 的 `renderBot()` 依赖 `window.Markdown`（vendored
 * `openghost/markdown.js` 挂到全局）。一旦它没了，`renderBot` 静默返回
 * `null` ⇒ **每条 bot 消息退回显示 markdown 原文**（`**粗体**`、`# 标题`、
 * ```` ```代码 ```` 全部裸露），而：
 *   - 界面**不报错**、**不白屏**；
 *   - 消息照常收发、列表照常虚拟化；
 *   - ⓘ **那 100 行 `.nb-md`/`.md-code` CSS 会变成无人察觉的死 CSS**。
 *
 * ⇒ 此前只有**人眼**能发现。本门把它变成可执行判据。
 *
 * # 判据（先定）
 *
 * |  | 判据 | 拦住什么 |
 * |---|------|---------|
 * | A | markdown **真的渲染成 HTML**（有 `.nb-md`），且**看不到原始标记** | `renderBot` 退化成纯文本 |
 * | B | 标题/粗体/列表/表格/引用/链接/行内码 **都真的产生了元素** | 渲染器只覆盖部分语法 |
 * | C | 代码块**不撑破气泡**（宽度 ≤ 气泡） | 长代码把布局顶飞 |
 * | D | 长行**横向滚动**（`overflow-x: auto` 且 `scrollWidth > clientWidth`），且**不把块撑宽** | 整页出现横向滚动条 |
 * | E | 复制按钮在位，aria-label **非空** | 复制按钮无名 ⇒ 无障碍失效 |
 * | F | 代码文字对比度 **≥ 4.5:1**，**深浅两套主题都查** | ⓘ 既有对比度门**完全没覆盖代码块**（实测 rg 零命中） |
 * | G | 复制按钮文案**跟随应用语言**（切 en ⇒ 变英文） | vendored 垫片的 `lang()` 脱钩 |
 * | H | 零未捕获异常 | 渲染期炸了却只显示半截 |
 *
 * ⛔ **F 必须两套主题都查**：深浅是**两套** token，只测一套等于没测。
 * ⛔ **A 的判据是「看不到原始标记」**，不是「有 `.nb-md`」——
 *    两者不等价：部分降级会渲染出容器但把内容原样塞进去。
 * ⛔ **D 不接受「没撑宽」当通过**：那也可能是 `overflow` 被写成了 `hidden`
 *    （内容被**裁掉**而不是可滚动）—— 用户同样看不到。
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
const PORT = 8863
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }

const LONG = 'fn main() { let s = "' + 'x'.repeat(120) + '"; println!("{}", s); }'
const MD = [
  '# 标题', '',
  '**粗体** 与 `行内码` 与 [链接](https://example.com)', '',
  '```rust', LONG, '```', '',
  '- 项目一', '- 项目二', '',
  '> 引用', '',
  '| a | b |', '| --- | --- |', '| 1 | 2 |', '',
].join('\n')

const N = 1000
const convos = Array.from({ length: N }, (_, i) => ({
  id: `c${i}`, kind: 'direct', title: `会话 ${i}`, members: ['u1'], task_count: 0,
  last_active: new Date(Date.now() - i * 6e4).toISOString(), muted: false, unread: 0,
}))
// ⛔⛔ markdown **只放 assistant 消息**。
//   我第一版在末尾 6 条里都放 MD，而其中 3 条是 `user` 角色 ——
//   **user 消息按设计就显示原文**（`m.who === 'me' ? <span>原文</span>`），
//   用户在聊天里敲的 markdown 不该被渲染。
//   ⇒ 我的 A 判据（"看不到原始标记"）被**自己的夹具**推翻：
//   它在 assistant 消息里找残留标记，却总是撞见 user 消息的**正确**原文。
//   ⇒ 判据没错，夹具错了。**判据错 ≠ 代码有缺陷，夹具错同理。**
const msgs = Array.from({ length: N }, (_, i) => ({
  id: `m${i}`, convo_id: 'c0', role: 'assistant',
  text: i >= N - 4 ? MD : `第 ${i} 条消息`.repeat(3),
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

/** sRGB 相对亮度 → WCAG 对比度。 */
function contrast(fg, bg) {
  const lum = (c) => {
    const [r, g, b] = c.match(/\d+(\.\d+)?/g).slice(0, 3).map((v) => {
      const s = Number(v) / 255
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
    })
    return 0.2126 * r + 0.7152 * g + 0.0722 * b
  }
  const a = lum(fg), b = lum(bg)
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05)
}

async function open(theme) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 820 } })
  const page = await ctx.newPage()
  const errs = []
  page.on('pageerror', (e) => errs.push(e.message))
  await page.addInitScript((a) => {
    try { localStorage.setItem('neobot.theme', a.theme) } catch { /* ignore */ }
    window.__TAURI_INTERNALS__ = {
      invoke: (c) => (c === 'plugin:event|listen' ? Promise.resolve(1)
        : c === 'plugin:event|unlisten' ? Promise.resolve(null)
        : Promise.resolve(a.t[c] ?? null)),
      transformCallback: (cb) => { window._1 = cb; return 1 },
      metadata: { currentWindow: { label: 'main' } },
    }
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
  }, { theme, t: STUB })
  await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, { waitUntil: 'load' })
  await page.waitForTimeout(1800)
  // ⛔ 滚到底**并重试**：虚拟列表默认停在底部，但滚容器上溯在实测中不稳定
  //   （从 sizer 上溯有时找不到 overflow-y:auto/scroll 的祖先）。
  //   ⛔ 固定 sleep 当等待是竞态 ⇒ 找不到就如实报错，不假装通过。
  for (let i = 0; i < 6; i++) {
    const n = await page.evaluate(() => {
      let sc = document.querySelector('[data-testid="nb-msg-sizer"]'), hops = 0
      while (sc && hops < 16) {
        const oy = getComputedStyle(sc).overflowY
        if (oy === 'auto' || oy === 'scroll') break
        sc = sc.parentElement; hops++
      }
      if (sc) sc.scrollTop = sc.scrollHeight
      return document.querySelectorAll('.nb-md').length
    })
    if (n > 0) break
    await page.waitForTimeout(600)
  }
  await page.waitForTimeout(700)
  return { ctx, page, errs }
}

console.log('markdown / 代码块门\n')

for (const theme of ['light', 'dark']) {
  const { ctx, page, errs } = await open(theme)
  const d = await page.evaluate(() => {
    const blocks = [...document.querySelectorAll('.nb-md .md-code')]
    // ⛔ 只在**已渲染的 .nb-md 内**找残留标记；扫整个 .nb-main 会把
    //   user 消息的**正确**原文也算进来（见夹具注释）。
    const txt = [...document.querySelectorAll('.nb-md')].map((n) => n.innerText).join('\n')
    const first = blocks[0]
    const pre = first?.querySelector('pre')
    const bubble = first?.closest('[class*="max-w-"]') ?? first?.closest('[class*="rounded-2xl"]')
    const codeEl = first?.querySelector('pre code')
    return {
      theme: document.documentElement.getAttribute('data-theme'),
      nbMd: document.querySelectorAll('.nb-md').length,
      h1: document.querySelectorAll('.nb-md h1').length,
      strong: document.querySelectorAll('.nb-md strong').length,
      ul: document.querySelectorAll('.nb-md ul').length,
      table: document.querySelectorAll('.nb-md table').length,
      quote: document.querySelectorAll('.nb-md blockquote').length,
      a: document.querySelectorAll('.nb-md a').length,
      inlineCode: document.querySelectorAll('.nb-md code').length,
      // A 的真判据：原始标记**可见**吗
      rawFence: txt.includes('```'), rawBold: txt.includes('**粗体**'), rawHash: /^# 标题/m.test(txt),
      rawPipe: txt.includes('| --- |'),
      blkW: first ? Math.round(first.getBoundingClientRect().width) : 0,
      bubW: bubble ? Math.round(bubble.getBoundingClientRect().width) : 0,
      ofx: pre ? getComputedStyle(pre).overflowX : null,
      scrollW: pre?.scrollWidth ?? 0, clientW: pre?.clientWidth ?? 0,
      codeColor: codeEl ? getComputedStyle(codeEl).color : null,
      codeBg: first ? getComputedStyle(first).backgroundColor : null,
      copyLabel: first?.querySelector('.md-copy')?.getAttribute('aria-label') ?? null,
    }
  })

  console.log(`  ── ${theme} ──`)
  // A
  if (d.nbMd === 0) bad(`[${theme}] 没有 .nb-md ⇒ window.Markdown 没了，bot 消息在显示 markdown 原文`)
  const leaked = [['```', d.rawFence], ['**粗体**', d.rawBold], ['# 标题', d.rawHash], ['| --- |', d.rawPipe]]
    .filter(([, v]) => v).map(([k]) => k)
  if (leaked.length) bad(`[${theme}] 原始标记仍可见：${leaked.join(' / ')} ⇒ 渲染被降级`)
  console.log(`  A markdown 渲染成 HTML：.nb-md=${d.nbMd} ${d.nbMd ? '✅' : '⛔'}（原始标记残留 ${leaked.length} 处 ${leaked.length ? '⛔' : '✅'}）`)
  // B
  const miss = Object.entries({ h1: d.h1, strong: d.strong, ul: d.ul, table: d.table, quote: d.quote, a: d.a, inlineCode: d.inlineCode })
    .filter(([, v]) => !v).map(([k]) => k)
  if (miss.length) bad(`[${theme}] 这些语法没产生元素：${miss.join(', ')}`)
  console.log(`  B 语法覆盖：${miss.length ? `⛔ 缺 ${miss.join(',')}` : '✅ 标题/粗体/列表/表格/引用/链接/行内码齐全'}`)
  // C
  if (d.blkW && d.bubW && d.blkW > d.bubW + 1) bad(`[${theme}] 代码块撑破气泡（${d.blkW} > ${d.bubW}）`)
  console.log(`  C 不撑破气泡：块 ${d.blkW}px / 气泡 ${d.bubW}px ${d.blkW > d.bubW + 1 ? '⛔' : '✅'}`)
  // D
  const dOk = d.ofx === 'auto' && d.scrollW > d.clientW
  if (!dOk) bad(`[${theme}] 长行不可横向滚动（overflow-x=${d.ofx} scrollW=${d.scrollW} clientW=${d.clientW}）`
    + ` ⇒ 内容可能被**裁掉**（overflow:hidden）而不是可滚动`)
  console.log(`  D 长行可横向滚动：overflow-x=${d.ofx} ${d.scrollW}/${d.clientW} ${dOk ? '✅' : '⛔'}`)
  // E
  if (!d.copyLabel || !d.copyLabel.trim()) bad(`[${theme}] 复制按钮 aria-label 为空 ⇒ 无障碍失效`)
  console.log(`  E 复制按钮 aria-label：「${d.copyLabel}」${d.copyLabel ? '✅' : '⛔'}`)
  // F
  if (d.codeColor && d.codeBg) {
    const cr = contrast(d.codeColor, d.codeBg)
    if (cr < 4.5) bad(`[${theme}] 代码文字对比度 ${cr.toFixed(2)}:1 < 4.5（${d.codeColor} on ${d.codeBg}）`)
    console.log(`  F 代码文字对比度：${cr.toFixed(2)}:1 ${cr >= 4.5 ? '✅' : '⛔'}`)
  } else bad(`[${theme}] 取不到代码文字/背景色`)
  // H
  if (errs.length) bad(`[${theme}] ${errs.length} 条未捕获异常：${errs[0].slice(0, 80)}`)
  console.log(`  H 未捕获异常：${errs.length} ${errs.length ? '⛔' : '✅'}`)

  await ctx.close()
}

// G：复制按钮文案跟随应用语言（vendored 垫片的 lang() 端到端）
{
  const { ctx, page } = await open('light')
  const sel = '[data-testid="nb-lang-select"]'
  const zh = await page.evaluate(() => document.querySelector('.nb-md .md-copy')?.getAttribute('aria-label') ?? null)
  // ⛔⛔ 不 `.catch(() => {})` 吞掉失败：上一版正是这么把「选择器 disabled 导致
  //   selectOption 超时」变成了「文案没变」的**假象**，差点去「修」一个没坏的地方。
  //   那个 select 带 `disabled={busy}` ⇒ Playwright 会等它可交互直到超时。
  //   ⇒ 先证明语言**真的**切了，再判文案是否跟随。
  let selErr = null
  // ⭐⭐ 2026-10-03：控件已移进设置弹窗 ⇒ 不在常驻 DOM。
  // ⭐⭐⭐ 而上面那段注释正是在警告「`.catch()` 把失败变成假象」——
  // ⭐⭐ **我本轮在另外两个脚本里恰好犯了它自己警告过的错**（已一并修），
  // ⭐⭐ 此处 ⛔ 不再吞：失败要么抛，要么记进 `selErr` 并被下方判据看见。
  if (!(await page.$(sel))) {
    await page.click('[data-testid="nb-settings-open"]')
    await page.waitForSelector(sel, { timeout: 8000 })
  }
  await page.selectOption(sel, 'en-US', { timeout: 8000 }).catch((e) => { selErr = String(e).split('\n')[0] })
  // ⭐⭐⭐ 必须**关掉设置弹窗**：否则 ⭐⭐ 模态遮罩残留 ⇒ 后续判据全被挡
  // （实测：neobot-ui-smoke 因遮罩残留，失败项 4 → 10）
  if (await page.$('[data-testid="nb-settings-close"]')) {
    await page.keyboard.press('Escape')
    await page.waitForSelector('[data-testid="nb-settings-close"]', { state: 'detached', timeout: 8000 })
  }
  await page.waitForTimeout(1000)
  const after = await page.evaluate(() => ({
    lang: document.documentElement.lang,
    sel: document.querySelector('[data-testid="nb-lang-select"]')?.value ?? null,
    label: document.querySelector('.nb-md .md-copy')?.getAttribute('aria-label') ?? null,
  }))
  console.log(`  ── 语言跟随 ──`)
  console.log(`  G 切换：selectOption ${selErr ? `⛔ 失败(${selErr.slice(0, 50)})` : '✅'}`)
  console.log(`    documentElement.lang=${after.lang} select.value=${after.sel}`)
  console.log(`    复制按钮文案：zh「${zh}」→ 「${after.label}」`)
  if (selErr) bad(`G 切语言失败：${selErr.slice(0, 70)}（select 可能处于 disabled）`)
  else if (!after.lang?.toLowerCase().startsWith('en')) bad(`G 语言没真的切到英文（lang=${after.lang}）⇒ 不能据此判文案`)
  else if (!zh || !after.label) bad('G 取不到复制按钮文案（代码块未渲染？）')
  else if (zh === after.label) bad(`G 语言已切到 en，但文案仍是「${after.label}」⇒ vendored 垫片的 lang() 与应用语言脱钩`)
  else if (!/[A-Za-z]/.test(after.label)) bad(`G 英文文案不是英文：「${after.label}」`)
  await ctx.close()
}

await browser.close()
srv.close()
console.log()
console.log(fail ? `FAIL: ${fail} 项` : 'PASS: markdown 真渲染、代码块不撑破布局、长行可滚、复制按钮有名且跟随语言、对比度达标、零异常。')
process.exit(fail ? 1 : 0)
