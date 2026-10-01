/**
 * neobot-ui 运行时冒烟测试 —— 证明「产物真的渲染」，而非「构建通过」。
 *
 * # 为什么必须做（AGENTS.md R-SCAN-2：手推 ≠ 实证）
 *
 * 上一轮我断言「vite 默认 `base:'/'` 在 Tauri 的 file:// 加载下会**白屏**」——
 * 那是**手推**，未实证。本脚本就是来**证实或证伪**它的。
 * 若只按「`vite build` 成功 + `tsc` 通过」验收，就会交付一个开屏即白的产品。
 *
 * # 判据
 *
 *   1. `#root` 被真实填充（子元素数 > 0）—— 证伪「白屏」
 *   2. 无未捕获 console error —— 证伪「静默失败」
 *   3. base 变体对照：同一服务、同一路径下分别加载 `base:'./'` 与 `base:'/'`
 *      两份产物，验证 base 到底影不影响渲染
 *
 * # 为什么注入 Tauri IPC 桩
 *
 * `neobot-root.tsx` 在 mount 时 `invoke('log_frontend', ...)`，而
 * `@tauri-apps/api/core` 依赖 `window.__TAURI_INTERNALS__`，浏览器里不存在。
 * 本脚本测的是**产物能否渲染**，不是 Tauri 桥（桥在 Rust 侧，浏览器测不了）。
 * 故在模块执行前用 `addInitScript` 注入桩，把桥接因素从渲染因素中剥离。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { createRequire } from 'node:module'

// playwright 装在 vendored 的 frontend 里，而本脚本在 scripts/ops/。
// ⛔ 不用「在仓库根建 node_modules 软链」来解析 —— 那是**根级新项**，
//   会被命名门拦下（实测：FAIL(strict) 新增违规 1）。故用 createRequire
//   锚定到 frontend/package.json，零新增根级项。
const require = createRequire(
  new URL('../../apps/neobot-desktop/frontend/package.json', import.meta.url),
)
const { chromium } = require('playwright')

const UI = new URL('../../apps/neobot-desktop/neobot-ui/', import.meta.url).pathname
const DIST = join(UI, 'dist')
const PORT = 8731

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
}

// 只服务 neobot-ui/dist，且**挂在子路径 /app/ 下** ——
// 子路径是本测试的关键：base:'/' 与 base:'./' 只在「非根路径」下才有区别。
const server = createServer(async (req, res) => {
  const url = new URL(req.url, 'http://x')
  if (!url.pathname.startsWith('/app/')) {
    res.writeHead(404).end('not found')
    return
  }
  const rel = normalize(url.pathname.slice('/app/'.length)).replace(/^(\.\.[/\\])+/, '')
  const file = join(DIST, rel === '' || rel === '/' ? 'index.html' : rel)
  try {
    const body = await readFile(file)
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' })
    res.end(body)
  } catch {
    console.log('     [404] ' + url.pathname)
    res.writeHead(404).end('missing')
  }
})

await new Promise((r) => server.listen(PORT, '127.0.0.1', r))

/**
 * 契约忠实的 Tauri IPC 桩。
 *
 * ⚠️ 第一版这里 `invoke: () => Promise.resolve(null)` —— **不忠实**，导致
 * 冒烟测试报出「未渲染」。核验后确认那是**桩的错，不是产品的错**：
 * `neobot-root.tsx:215` 声明 `invoke<ConvoView[]>`，Rust 侧返回 `Vec`
 * 序列化成 `[]`，**永不为 null**；桩却返回 null ⇒ `v[0]` 抛
 * `Cannot read properties of null`。同理 `:157/:172/:319` 读的是对象字段。
 *
 * ⇒ 冒烟测试的桩必须**按声明类型**建模，否则测的是桩的 bug。
 *   字段取自 neobot-root.tsx 自身的 TS 接口（UsageSummary / MemoryView /
 *   CapabilitySnapshot），不臆造。
 */
const STUB_RETURNS = {
  // 数组型（声明为 T[]，Rust 侧 serde 序列化为 []）
  // 长列表用于验证「溢出渐隐」：__LONG_CONVOS__ 是开关占位，
  // 由 probe 按需替换为 60 条，触发容器溢出。
  // ⚠️ 基础夹具**必须有会话**：否则不会选中任何 convo ⇒ 消息永远不加载
  //    ⇒ 5b 断言报「消息不可见」，而那是**夹具缺口**不是产品缺陷。
  //    （我第一次就踩了：空列表 + 真断言 = 假失败。夹具与判据要一起设计。）
  //    字段严格对齐 Rust 的 ConvoView：id/kind/title/members/task_count/
  //    last_active/muted/unread —— 少字段会出现 `undefined 个任务` 这类假象。
  neobot_convo_list: [{
    id: 'c0', kind: 'direct', title: '海豚调试', members: ['u1'],
    task_count: 2, last_active: '2026-10-01T00:00:00Z', muted: false, unread: 0,
  }],
  neobot_member_list: [],
  // ⛔ 这里曾是 `[]`，于是**消息区从未被真正断言过** —— 我据此交付过一个
  //    「消息区渲染出空白气泡」的界面而全门绿（当时桩把 ChatMessage 的
  //    `text` 写成 `content`，产品没坏、桩坏了，但**门没能力发现**）。
  //    现给两条真实消息，字段严格对齐 neobot-root.tsx 的 interface：
  //      { id, convo_id, role, text, created_at }
  neobot_convo_messages: [
    { id: 'm1', convo_id: 'c0', role: 'user', text: '为什么界面是乱的', created_at: '2026-10-01T00:00:00Z' },
    { id: 'm2', convo_id: 'c0', role: 'assistant', text: 'main 是 display:block，flex 失去约束', created_at: '2026-10-01T00:00:01Z' },
  ],
  // 布尔型
  neobot_memory_add: true,
  neobot_memory_undo: true,
  neobot_member_add: true,
  // 字符串型
  neobot_convo_group: '',
  // 对象型（字段取自 neobot-root.tsx 的 TS 接口）
  neobot_usage_summary: {
    days: 1, input: 0, cached: 0, written: 0, output: 0, requests: 0, tokens: 0,
  },
  neobot_memory_list: { lines: [], bytes: 0, cap: 0, revisions: 0 },
  neobot_core_capabilities: {
    crystal_version: '0.0.0', tool_count: 0, model: '', model_source: '',
  },
  neobot_send: {},
  // 主题：契约表只有 get_dsh_theme（能报不能改），返回 'system' 让前端跟随系统
  get_dsh_theme: 'system',
  // R2 外壳用到的已实现命令
  set_language: null,
  read_run_logs: 'line-1 harness ready\nline-2 self-hosted ui mounted\n',
  quit_app: null,
  open_external_url: null,
  // 无返回值命令
  log_frontend: null,
  neobot_api_call: null,
}

/**
 * ⚠️ 这里**必须**把词表当**参数**传进来，不能用闭包。
 * `page.addInitScript(fn)` 只序列化 `fn.toString()` —— Node 侧的模块级
 * `const STUB_RETURNS` 在浏览器里是 `undefined`，于是每次 invoke 都抛
 * `ReferenceError`，而 UI 把错误 `.catch` 掉照样渲染 ⇒ **测出的是错误态**。
 * 这个坑我踩过一次：首版桩内联（无闭包）是对的，改成查表后反而错，
 * 且因为错误被吞，「渲染通过」的假象一直到我加功能断言才暴露。
 */
// ⚠️ `addInitScript(fn, arg)` 只接受**一个** arg（Playwright 限制）。
//   我曾传 `[table, failList]` ⇒ 形参 table 收到整个数组 ⇒ `table[cmd]` 恒
//   undefined ⇒ 全部返回 null ⇒ 复现了 `v[0]` 崩溃 ⇒ **测的是 harness 的 bug**。
//   故合并成单个对象传递。
const TAURI_STUB = ({ table, failList }) => {
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      if (failList && failList.includes(cmd)) {
        return Promise.reject(new Error(`induced failure: ${cmd}`))
      }
      // Tauri 事件插件：`listen` 必须回一个数字 id，`unlisten` 回 null
      if (cmd === 'plugin:event|listen') return Promise.resolve(1)
      if (cmd === 'plugin:event|unlisten') return Promise.resolve(null)
      void args
      return Promise.resolve(table[cmd] ?? null)
    },
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e9)
      window[`_${id}`] = cb
      return id
    },
    metadata: { currentWindow: { label: 'main' } },
  }
  // ⛔ Tauri **事件插件**内部对象：缺它会让 `listen()` 的 unlisten 抛
  //    `Cannot read properties of undefined (reading 'unregisterListener')`。
  //    实测教训：另一窗口给 shell 加了 `listen('macos-menu-action')`
  //    （macOS 原生菜单必需，**它的实现是对的**），我的桩没提供这个对象
  //    ⇒ 两个门误报红。**是我 harness 不完整，不是产品缺陷。**
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {},
  }
}

/** 加载一页并返回渲染事实。 */
async function probe(browser, label, failList, act, convoCount = 0) {
  const page = await browser.newPage()
  const errors = []
  const badUrls = []
  // 资源加载失败按 **URL** 判定，不靠 console 文本 ——
  // 实测 console 文本是泛化的「Failed to load resource: ... 404」，
  // **不含 URL**，按文本过滤必然失效（我第一版就这么错的）。
  page.on('response', (r) => {
    // /favicon.ico 由**浏览器自动发起**：我方 index.html 未声明 favicon，
    // 且 Tauri 窗口不按网页方式取 favicon ⇒ 不计为应用错误。
    if (r.status() >= 400 && !r.url().includes('/favicon.ico')) {
      badUrls.push(`${r.status()} ${r.url()}`)
    }
  })
  page.on('console', (m) => {
    const t = m.text()
    // 「Failed to load resource」由上面的 response 监听按 URL 负责，不重复计
    if (m.type() === 'error' && !t.startsWith('Failed to load resource')) errors.push(t)
  })
  page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`))

  const table = { ...STUB_RETURNS }
  if (convoCount > 0) {
    // 造足够多的会话把列表撑出容器 —— 用**合法字段**，
    // 避免用一个前端没预期的形状制造出与本题无关的崩溃。
    table.neobot_convo_list = Array.from({ length: convoCount }, (_, i) => ({
      id: `c${i}`,
      title: `会话 ${i} —— 一个足够长以便在列表里换行的标题`,
      kind: i % 3 === 0 ? 'group' : 'direct',
      members: [1, 2, 3],
      last_active: new Date(Date.now() - i * 60000).toISOString(),
      unread: 0,
    }))
  }
  await page.addInitScript(TAURI_STUB, { table, failList: failList || [] })
  let httpStatus = 0
  let children = 0
  let textLen = 0
  let r_children = 0
  try {
    const resp = await page.goto(`http://127.0.0.1:${PORT}/app/index.html`, {
      waitUntil: 'load',
      timeout: 15000,
    })
    httpStatus = resp?.status() ?? 0
    // 等 React 有机会挂载
    await page.waitForTimeout(1200)
    const info = await page.evaluate(() => {
      const r = document.getElementById('root')
      return { children: r ? r.children.length : -1, textLen: (r?.innerText ?? '').length }
    })
    children = info.children
    textLen = info.textLen
    r_children = info.children
  } catch (e) {
    errors.push(`navigation: ${e.message}`)
  }
  // ── R2/R4 功能断言：外壳渲染 + 语言切换**真的生效** ──
  // ⛔ 失败路径**跳过**功能断言：那些断言会 Esc 关掉日志弹窗，之后再取
  //    body.innerText 就取不到 `read_run_logs` 的失败文本了 ——
  //    探针自己把被测证据抹掉了（我第一版就这么错，误判成「无失败提示」）。
  const feat = {}
  const isFailCase = label.startsWith('失败路径：')
  if (label.startsWith('溢出渐隐：')) {
    feat.overflow = await page.evaluate(() => {
      const el = document.querySelector('[data-testid="nb-convo-list"]')
      if (!el) return null
      // 窗口化事实：撑高容器的高度 vs 实际渲染出的行数
      const sizer = document.querySelector('[data-testid="nb-convo-sizer"]')
      const rowsNow = el.querySelectorAll('[data-index]').length
      const cs = getComputedStyle(el, '::after')
      return {
        scrollH: el.scrollHeight,
        clientH: el.clientHeight,
        // 纯 CSS scroll-shadow 探针：应有 4 层 background（2 local + 2 scroll）
        bgLayers: (getComputedStyle(el).backgroundImage.match(/gradient\(/g) || []).length,
        hasLocal: (getComputedStyle(el).backgroundImage.match(/local/g) || []).length,
        scrolls: el.scrollHeight > el.clientHeight,
        opacity: cs.opacity,
        hasScrollClass: el.classList.contains('nb-scroll'),
        sizerH: sizer ? Math.round(sizer.getBoundingClientRect().height) : 0,
        renderedRows: rowsNow,
      }
    }).catch(() => null)
  }
  if (isFailCase) {
    // 只跑该案例自己的动作（功能断言会 Esc 关弹窗、抹掉失败证据，故不走）
    if (act) await act(page)
    await page.waitForTimeout(600)
    // 活动面板的**核心价值**是暴露失败：开日志弹窗看是否有失败行。
    // ⛔ 只有经 src/ipc.ts 的调用才会被记录；桌宠页仍直连（他窗文件），
    //    故只对「已知走单一出口」的失败命令断言。
    if (act) {
      await page.click('.nb-actions button').catch(() => {})
      await page.waitForTimeout(400)
      feat.badRows = await page.evaluate(() =>
        [...document.querySelectorAll('.nb-act-list li.bad')].map((li) => ({
          cmd: li.querySelector('code')?.textContent ?? null,
          ms: li.querySelector('.nb-act-ms')?.textContent ?? null,
          err: li.querySelector('.nb-act-err')?.textContent ?? null,
        })),
      ).catch(() => [])
    }
  } else if (r_children > 0) {
    feat.shell = await page.evaluate(() => ({
      wordmark: document.querySelector('.nb-wordmark')?.textContent ?? null,
      hasLangSelect: !!document.querySelector('.nb-lang select'),
      buttons: [...document.querySelectorAll('.nb-actions button')].map((b) => b.textContent),
    })).catch(() => null)
    // 语言切换：改 select → 断言 documentElement.lang 与可见文案都变了
    feat.langBefore = await page.evaluate(() => document.documentElement.lang)
    feat.textBefore = await page.evaluate(
      () => document.querySelector('.nb-actions button')?.textContent ?? '')
    await page.selectOption('[data-testid="nb-lang-select"]', 'en-US').catch(() => {})
    await page.waitForTimeout(500)
    feat.langAfter = await page.evaluate(() => document.documentElement.lang)
    feat.textAfter = await page.evaluate(
      () => document.querySelector('.nb-actions button')?.textContent ?? '')
    // ── 日志弹窗（走 read_run_logs）+ 模态行为**实证** ──
    const trigger = await page.$('.nb-actions button')
    await trigger?.click().catch(() => {})
    await page.waitForTimeout(400)
    feat.logsModal = await page.evaluate(() => {
      const pre = document.querySelector('.nb-modal-box pre')
      return pre ? pre.textContent.slice(0, 40) : null
    })
    // ── 5b 消息正文**真的看得见**（此前完全没断言）──
    // ⛔ 来自真实事故：我交付过一个「消息区只有空白气泡」的聊天界面，
    //    而门全绿 —— 因为我只验「#root 有子元素 + 无 console 报错」。
    //    **「渲染了」与「用户看得到内容」是两件事，必须分别断言。**
    // ⚠️ 采集必须写在 probe 内（`page` 只在这里有）；写进报告循环是作用域错误，
    //    我今天在同一个文件里已犯过两次。
    feat.msgVisible = await page.evaluate(() => {
      const txt = (document.querySelector('.nb-main')?.innerText || '').replace(/\s+/g, ' ')
      return {
        mainTextLen: txt.length,
        seesUserText: txt.includes('为什么界面是乱的'),
        seesBotText: txt.includes('flex 失去约束'),
      }
    }).catch(() => null)

    // ── 主题切换（把已移植却够不着的浅色盘接活）──
    // ⛔ 双向：浅→深必须**真的**变。单向只测「能变浅」的话，
    //    「恒为 dark」也能通过。
    feat.theme = await page.evaluate(() => {
      const q = () => document.documentElement.getAttribute('data-theme')
      const cs = () => getComputedStyle(document.body).backgroundColor
      return { initial: q(), initialBg: cs() }
    }).catch(() => null)
    // ⚠️ 必须用 **稳定钩子**（data-testid），不能用位置或文本匹配：
    //    我加主题选择器时，语言选择器从第 1 个变成第 2 个 ⇒
    //    位置式探针**静默地**改去操作主题选择器 ⇒ 报「语言切换坏了」，
    //    而产品没坏。**选择器要抗布局变化。**
    await page.selectOption('[data-testid="nb-theme-select"]', 'light').catch(() => {})
    const sels = { theme: null, lang: null }
    await page.waitForTimeout(400)
    feat.themeLight = await page.evaluate(() => ({
      attr: document.documentElement.getAttribute('data-theme'),
      bg: getComputedStyle(document.querySelector('.nb-shell') || document.body).backgroundColor,
    })).catch(() => null)
    await page.selectOption('[data-testid="nb-theme-select"]', 'dark').catch(() => {})
    await page.waitForTimeout(400)
    feat.themeDark = await page.evaluate(() => ({
      attr: document.documentElement.getAttribute('data-theme'),
      bg: getComputedStyle(document.querySelector('.nb-shell') || document.body).backgroundColor,
    })).catch(() => null)

    // a11y-1 role/aria 必须落在 box 上，**不是**遮罩
    feat.dialogOnBox = await page.evaluate(() => {
      const box = document.querySelector('.nb-modal-box')
      const mask = document.querySelector('.nb-modal')
      return {
        boxRole: box?.getAttribute('role') ?? null,
        boxAriaModal: box?.getAttribute('aria-modal') ?? null,
        maskRole: mask?.getAttribute('role') ?? null,
      }
    })
    // a11y-2 初始焦点应落在 data-autofocus（关闭按钮）上
    feat.initialFocus = await page.evaluate(() => {
      const a = document.activeElement
      return { tag: a?.tagName ?? null, isAutofocus: a?.hasAttribute('data-autofocus') ?? false }
    })
    // ⚠️ 顺序纪律：活动面板**必须在此处（Esc 之前）**查 —— 弹窗里的
    //    DOM 在 Esc 之后已卸载。我第一版放在 Esc 之后 ⇒ 报「未渲染」，
    //    那是探针顺序错，不是面板没做。（与 read_run_logs 那次同源。）
    // ── 活动面板（吸收 UI-TARS 的 Event Stream Viewer / 耗时统计）──
    // 判据：挂载时 neobot_convo_list 等已发过 ⇒ 面板须有行、须带命令名与耗时
    feat.activity = await page.evaluate(() => {
      const rows = [...document.querySelectorAll('.nb-act-list li')]
      return {
        open: !!document.querySelector('.nb-act'),
        count: rows.length,
        firstCmd: rows[0]?.querySelector('code')?.textContent ?? null,
        firstMs: rows[0]?.querySelector('.nb-act-ms')?.textContent ?? null,
        hasBad: !!document.querySelector('.nb-act-list li.bad'),
      }
    }).catch(() => null)
    // a11y3 Esc 必须能关（键盘用户出得来）
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
    feat.escClosed = await page.evaluate(() => document.querySelector('.nb-modal-box') === null)
    // a11y-4 关闭后焦点须归还给触发元素，否则键盘用户失位
    feat.focusRestored = await page.evaluate(() => {
      const a = document.activeElement
      return { tag: a?.tagName ?? null, inBar: !!a?.closest?.('.nb-actions') }
    })
    // ⛔ 关键缺陷探测：切到 en-US 后，**聊天区**是否也变了？
    //    只换外壳不换正文 = 语言切换器是半成品。
    await page.selectOption('[data-testid="nb-lang-select"]', 'zh-CN').catch(() => {})
    await page.waitForTimeout(400)
    feat.chatZh = await page.evaluate(() => {
      const m = document.querySelector('.nb-main')
      return (m?.innerText ?? '').replace(/\s+/g, ' ').slice(0, 60)
    })
    await page.selectOption('[data-testid="nb-lang-select"]', 'en-US').catch(() => {})
    await page.waitForTimeout(400)
    feat.chatEn = await page.evaluate(() => {
      const m = document.querySelector('.nb-main')
      return (m?.innerText ?? '').replace(/\s+/g, ' ').slice(0, 60)
    })
  }
  const errText = await page.evaluate(() => document.body.innerText).catch(() => '')
  await page.close()
  return { label, httpStatus, children: r_children, textLen, errors, badUrls, feat, errText }
}

// 用系统 Chrome：免去 npx playwright install 的 150MB 下载（本仓已装 Chrome）
const browser = await chromium.launch({ channel: 'chrome' })
const rows = []
try {
  // 变体 1：当前产物（base:'./'）
  rows.push(await probe(browser, "base:'./' (当前产物)"))
  // 变体 2：**字面**的 base:'/' 产物 —— `src="/assets/x.js"`（根相对，非子路径）。
  // ⚠️ 初版把它改写成 `/app/assets/…`（顺手"修好"了路径）⇒ 那不是 base:'/' 的
  //    真实输出，实验**证明不了任何事**。此处按字面重写。
  const html = await readFile(join(DIST, 'index.html'), 'utf8')
  const abs = html.replace(/(src|href)="\.\//g, '$1="/')
  if (abs !== html) {
    await (await import('node:fs/promises')).writeFile(join(DIST, 'index.abs.html'), abs)
    const page = await browser.newPage()
    const errors = []
    page.on('console', (m) => m.type() === 'error' && errors.push(m.text()))
    page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`))
    await page.addInitScript(TAURI_STUB, { table: STUB_RETURNS, failList: [] })
    let children = 0
    try {
      const resp = await page.goto(`http://127.0.0.1:${PORT}/app/index.abs.html`, {
        waitUntil: 'load',
        timeout: 15000,
      })
      void resp
      await page.waitForTimeout(1200)
      children = await page.evaluate(
        () => document.getElementById('root')?.children.length ?? -1,
      )
    } catch (e) {
      errors.push(`navigation: ${e.message}`)
    }
    await page.close()
    rows.push({
      label: "base:'/' 对照（字面根相对 /assets/…）",
      httpStatus: 0,
      children,
      textLen: 0,
      errors,
      badUrls: [],
    })
  }
  // ── 溢出渐隐探针（吸收 OverlayScrollbars 的设计意图）──
  // 判据：短列表**不**显示渐隐；60 条列表**显示**渐隐。
  // ⛔ 若只测长列表，「渐隐恒显」也会通过 —— 必须两个方向都测。
  // 窗口化：1000 条时 DOM 节点数必须远小于条数
  for (const [name, n] of [['短列表(3)', 3], ['长列表(60)', 60], ['千条(1000)', 1000]]) {
    rows.push(await probe(browser, `溢出渐隐：${name}`, [], null, n))
  }

  // ── 失败路径探针（此前**完全没有覆盖**：桩对每个命令都返回成功）──
  // 诚实性检查：故障时 UI 必须**说清失败**，不得静默空白或崩溃。
  // ⛔ 每个案例必须自带 **act**：有些故障只在**用户动手**时才发生
  //   （set_language 不点就不调、read_run_logs 不点就不调）。
  //   我曾一刀切「失败路径跳过全部断言」⇒ 动作没执行 ⇒ 误判成「无失败提示」。
  //   —— 探针自己没触发故障，却报告故障处理有问题。
  const FAIL_CASES = [
    { name: 'neobot_convo_list 失败（挂载即触发）', fail: ['neobot_convo_list'] },
    { name: 'set_language 失败（需点切换）', fail: ['set_language'],
      act: async (pg) => { await pg.selectOption('[data-testid="nb-lang-select"]', 'en-US').catch(() => {}) } },
    { name: 'read_run_logs 失败（需点按钮）', fail: ['read_run_logs'],
      act: async (pg) => { await pg.click('.nb-actions button').catch(() => {}) } },
    { name: 'neobot_core_capabilities 失败（挂载即触发）', fail: ['neobot_core_capabilities'] },
  ]
  for (const c of FAIL_CASES) {
    rows.push(await probe(browser, `失败路径：${c.name}`, c.fail, c.act))
  }
} finally {
  await browser.close()
  server.close()
}

let fail = 0
console.log('neobot-ui 运行时冒烟测试（HTTP 子路径 /app/ 加载）\n')
for (const r of rows) {
  const rendered = r.children > 0
  const status = rendered ? '✅ 已渲染' : '⛔ 未渲染'
  console.log(`  ${r.label}`)
  console.log(`     #root 子元素 = ${r.children} · 文本长度 = ${r.textLen}  ${status}`)
  if (r.badUrls.length) {
    console.log(`     资源失败 × ${r.badUrls.length}:`)
    for (const u of r.badUrls.slice(0, 3)) console.log(`       · ${u.slice(0, 110)}`)
  }
  if (r.errors.length) {
    console.log(`     console/page error × ${r.errors.length}:`)
    for (const e of r.errors.slice(0, 3)) console.log(`       · ${e.slice(0, 110)}`)
  }
  if (r.feat && r.feat.shell) {
    const f = r.feat
    console.log(`     外壳：wordmark=${JSON.stringify(f.shell.wordmark)} `
      + `语言选择器=${f.shell.hasLangSelect ? '✅' : '⛔'} `
      + `按钮=${JSON.stringify(f.shell.buttons)}`)
    console.log(`     语言切换：documentElement.lang ${f.langBefore} → ${f.langAfter}`
      + ` · 按钮文案 ${JSON.stringify(f.textBefore)} → ${JSON.stringify(f.textAfter)}`)
    console.log(`     日志弹窗：${f.logsModal === null ? '⛔ 未打开' : '✅ ' + JSON.stringify(f.logsModal)}`)
    const th = f.theme
    if (th) {
      console.log(`     主题：初始 ${th.initial} (${th.initialBg})`
        + ` → 浅 ${f.themeLight?.attr} (${f.themeLight?.bg})`
        + ` → 深 ${f.themeDark?.attr} (${f.themeDark?.bg})`)
    }
    const mv = f.msgVisible
    console.log(`     消息可见性：可见文本 ${mv?.mainTextLen ?? '?'} 字 · `
      + `含用户原文=${mv?.seesUserText ? '✅' : '⛔'} 含 bot 原文=${mv?.seesBotText ? '✅' : '⛔'}`)
    if (!mv) {
      console.log('     ⛔ 未取到消息可见性事实'); fail++
    } else if (!mv.seesUserText || !mv.seesBotText) {
      console.log('     ⛔ 消息正文**不可见** —— 界面渲染了但用户看不到内容'); fail++
    }
    const d = f.dialogOnBox || {}
    console.log(`     模态语义：box[role=${d.boxRole} aria-modal=${d.boxAriaModal}] `
      + `遮罩[role=${d.maskRole ?? '（无，正确）'}]`)
    console.log(`     初始焦点：${JSON.stringify(f.initialFocus)}`)
    console.log(`     Esc 关闭：${f.escClosed ? '✅' : '⛔ 键盘用户出不来'}`
      + ` · 焦点归还：${JSON.stringify(f.focusRestored)}`)
    const a = f.activity
    if (a) {
      console.log(`     活动面板：行数=${a.count} 最新=${a.firstCmd} ${a.firstMs}`
        + ` 有失败行=${a.hasBad ? '是' : '否'}`)
    }
    if (r.label.startsWith('溢出渐隐：')) {
    const o = r.feat?.overflow
    if (!o) { console.log('     ⛔ 未取到溢出状态'); fail++ }
    else {
      const want = r.label.includes('长列表') || r.label.includes('千条')
      const isBig = r.label.includes('千条')
      // 判据：长列表必须**真的可滚**（高度被约束），短列表不必
      const ok = o.scrolls === want
      console.log(`     ${r.label}：scrollH=${o.scrollH} clientH=${o.clientH} `
        + `可滚=${o.scrolls ? '✅' : '⛔'} · scroll-shadow 层=${o.bgLayers}(local ${o.hasLocal}) ⇒ `
        + (ok ? '✅ 符合预期' : '⛔ 不符合预期'))
      if (!ok) fail++
      if (isBig) {
        console.log(`     窗口化：撑高 ${o.sizerH}px · 实际渲染 ${o.renderedRows} 行 `
          + `⇒ ${o.renderedRows < 120 && o.sizerH > 2000 ? '✅ 已窗口化' : '⛔ 仍在全量渲染'}`)
        if (o.renderedRows >= 1000) {
          console.log('     ⛔ 1000 条全部渲染 ⇒ 窗口化未生效'); fail++
        }
        if (o.sizerH <= 2000) {
          console.log('     ⛔ 撑高容器不足 2000px ⇒ 滚动条会算错，滚不动'); fail++
        }
      }
      if (o.bgLayers < 4) {
        console.log(`     ⛔ scroll-shadow 层不足（${o.bgLayers} < 4）⇒ 溢出无视觉提示`); fail++
      }
      if (want && o.scrollH <= o.clientH) {
        console.log('     ⛔ 60 条会话仍未受高度约束 ⇒ 布局回归（列表会撑破页面）'); fail++
      }
    }
  }
  if (r.label.startsWith('失败路径：')) {
    // 判据：① 仍渲染出内容（不是白屏）② 无未捕获 pageerror
    //      ③ 页面文本里能看到失败痕迹（说明**说清了**，而非静默）
    const hasCrash = r.errors.some((e) => e.startsWith('pageerror:'))
    if (r.feat?.badRows) {
      console.log(`     活动面板失败行：${r.feat.badRows.length} 条`
        + (r.feat.badRows[0]
          ? ` 例：${r.feat.badRows[0].cmd} ${r.feat.badRows[0].ms} `
            + `err=${JSON.stringify((r.feat.badRows[0].err || '').slice(0, 40))}`
          : '（无 —— 面板未暴露本次失败）'))
    }
    const saysFail = /失败|failed|诱导|induced|错误|error/i.test(r.errText || '')
    console.log(`     仍渲染=${r.children > 0 ? '✅' : '⛔ 白屏'} `
      + `未捕获异常=${hasCrash ? '⛔ ' + r.errors.find((e) => e.startsWith('pageerror:')) : '✅ 无'} `
      + `可见失败提示=${saysFail ? '✅' : '⚠️ 未见'}`)
    if (r.children <= 0) { console.log('     ⛔ 故障导致白屏'); fail++ }
    if (hasCrash) { console.log('     ⛔ 故障导致未捕获异常'); fail++ }
    // ⛔ `act` 不在报告循环的作用域（它是 probe 的参数）—— 我第一版
    //    在这里写 `act &&` ⇒ ReferenceError。判据应只看 r.feat。
    if (r.feat && Array.isArray(r.feat.badRows) && r.feat.badRows.length === 0) {
      console.log('     ⛔ 活动面板未暴露本次失败 —— 面板的核心价值失效')
      fail++
    }
    if (!saysFail) {
      console.log(`     ⛔ 故障**静默**：用户看不到任何失败提示`)
      console.log(`        可见文本：${JSON.stringify((r.errText || '').slice(0, 90))}`)
      fail++
    }
  }
  if (r.label.startsWith("base:'./'")) {
      if (!f.shell.wordmark) { console.log('     ⛔ 外壳未渲染'); fail++ }
      if (!f.shell.hasLangSelect) { console.log('     ⛔ 语言选择器缺失'); fail++ }
      if (f.langAfter !== 'en-US') { console.log('     ⛔ 语言切换未改 document lang'); fail++ }
      if (f.textBefore === f.textAfter) { console.log('     ⛔ 语言切换未改可见文案'); fail++ }
      if (f.logsModal === null) { console.log('     ⛔ 日志弹窗未打开'); fail++ }
      if (d.boxRole !== 'dialog' || d.boxAriaModal !== 'true') {
        console.log('     ⛔ role/aria-modal 未落在 box 上'); fail++
      }
      if (d.maskRole !== null) {
        console.log('     ⛔ 遮罩被当成 dialog（读屏会读错）'); fail++
      }
      if (!f.initialFocus?.isAutofocus) {
        console.log('     ⛔ 无初始焦点（data-autofocus 未生效）'); fail++
      }
      if (!f.escClosed) { console.log('     ⛔ Esc 关不掉对话框'); fail++ }
      if (!f.focusRestored?.inBar) {
        console.log('     ⛔ 关闭后焦点未归还给触发元素'); fail++
      }
      const L = f.themeLight, D = f.themeDark
      if (!L || L.attr !== 'light') {
        console.log(`     ⛔ 切浅色未生效：${JSON.stringify(L)}`); fail++
      } else if (!D || D.attr !== 'dark') {
        console.log(`     ⛔ 切深色未生效：${JSON.stringify(D)}`); fail++
      } else if (L.bg === D.bg) {
        console.log('     ⛔ 深浅两档背景色相同 ⇒ 浅色盘仍是死代码'); fail++
      }
      const a = f.activity
      if (!a || !a.open) { console.log('     ⛔ 活动面板未渲染'); fail++ }
      else if (a.count === 0) {
        console.log('     ⛔ 活动面板无记录（挂载时已发过 neobot_convo_list 等）'); fail++
      } else if (!a.firstMs || !/\d+\s*ms/.test(a.firstMs)) {
        console.log(`     ⛔ 活动行缺耗时统计：${JSON.stringify(a.firstMs)}`); fail++
      }
      if (f.chatZh === f.chatEn) {
        console.log(`     ⛔ 聊天区语言未随切换变化（半成品切换器）`)
        console.log(`        zh: ${JSON.stringify(f.chatZh)}`)
        console.log(`        en: ${JSON.stringify(f.chatEn)}`)
        fail++
      } else {
        console.log(`     聊天区语言：zh=${JSON.stringify(f.chatZh)}`)
        console.log(`                    en=${JSON.stringify(f.chatEn)}`)
      }
    }
  }
  if (r.label.startsWith('溢出渐隐：')) {
    const o = r.feat?.overflow
    if (!o) { console.log('     ⛔ 未取到溢出状态'); fail++ }
    else {
      const want = r.label.includes('长列表') || r.label.includes('千条')
      const isBig = r.label.includes('千条')
      // 判据：长列表必须**真的可滚**（高度被约束），短列表不必
      const ok = o.scrolls === want
      console.log(`     ${r.label}：scrollH=${o.scrollH} clientH=${o.clientH} `
        + `可滚=${o.scrolls ? '✅' : '⛔'} · scroll-shadow 层=${o.bgLayers}(local ${o.hasLocal}) ⇒ `
        + (ok ? '✅ 符合预期' : '⛔ 不符合预期'))
      if (!ok) fail++
      if (isBig) {
        console.log(`     窗口化：撑高 ${o.sizerH}px · 实际渲染 ${o.renderedRows} 行 `
          + `⇒ ${o.renderedRows < 120 && o.sizerH > 2000 ? '✅ 已窗口化' : '⛔ 仍在全量渲染'}`)
        if (o.renderedRows >= 1000) {
          console.log('     ⛔ 1000 条全部渲染 ⇒ 窗口化未生效'); fail++
        }
        if (o.sizerH <= 2000) {
          console.log('     ⛔ 撑高容器不足 2000px ⇒ 滚动条会算错，滚不动'); fail++
        }
      }
      if (o.bgLayers < 4) {
        console.log(`     ⛔ scroll-shadow 层不足（${o.bgLayers} < 4）⇒ 溢出无视觉提示`); fail++
      }
      if (want && o.scrollH <= o.clientH) {
        console.log('     ⛔ 60 条会话仍未受高度约束 ⇒ 布局回归（列表会撑破页面）'); fail++
      }
    }
  }
  if (r.label.startsWith('失败路径：')) {
    // 判据：① 仍渲染出内容（不是白屏）② 无未捕获 pageerror
    //      ③ 页面文本里能看到失败痕迹（说明**说清了**，而非静默）
    const hasCrash = r.errors.some((e) => e.startsWith('pageerror:'))
    if (r.feat?.badRows) {
      console.log(`     活动面板失败行：${r.feat.badRows.length} 条`
        + (r.feat.badRows[0]
          ? ` 例：${r.feat.badRows[0].cmd} ${r.feat.badRows[0].ms} `
            + `err=${JSON.stringify((r.feat.badRows[0].err || '').slice(0, 40))}`
          : '（无 —— 面板未暴露本次失败）'))
    }
    const saysFail = /失败|failed|诱导|induced|错误|error/i.test(r.errText || '')
    console.log(`     仍渲染=${r.children > 0 ? '✅' : '⛔ 白屏'} `
      + `未捕获异常=${hasCrash ? '⛔ ' + r.errors.find((e) => e.startsWith('pageerror:')) : '✅ 无'} `
      + `可见失败提示=${saysFail ? '✅' : '⚠️ 未见'}`)
    if (r.children <= 0) { console.log('     ⛔ 故障导致白屏'); fail++ }
    if (hasCrash) { console.log('     ⛔ 故障导致未捕获异常'); fail++ }
    // ⛔ `act` 不在报告循环的作用域（它是 probe 的参数）—— 我第一版
    //    在这里写 `act &&` ⇒ ReferenceError。判据应只看 r.feat。
    if (r.feat && Array.isArray(r.feat.badRows) && r.feat.badRows.length === 0) {
      console.log('     ⛔ 活动面板未暴露本次失败 —— 面板的核心价值失效')
      fail++
    }
    if (!saysFail) {
      console.log(`     ⛔ 故障**静默**：用户看不到任何失败提示`)
      console.log(`        可见文本：${JSON.stringify((r.errText || '').slice(0, 90))}`)
      fail++
    }
  }
  if (r.label.startsWith("base:'./'")) {
    if (!rendered) {
      console.log('     ⛔ 当前产物未渲染 ⇒ 门应判失败')
      fail++
    }
    if (r.errors.length || r.badUrls.length) {
      console.log('     ⛔ 存在未捕获错误或资源加载失败')
      fail++
    }
  }
}
console.log()
if (fail) {
  console.log(`FAIL: ${fail} 项`)
  process.exit(1)
}
console.log('PASS: 产物在非根路径下真实渲染，且无未捕获错误。')
console.log('')
console.log('关于 base 的证据边界（勿越读）：')
console.log('  · 已**实证**：`base:\'./\'`（相对）在**子路径**下正确加载并渲染。')
console.log('  · 未**实证**：`base:\'/\'`（根相对）在 Tauri v2 自定义协议下是否会白屏。')
console.log('    Tauri 把 frontendDist 挂在协议根，`/assets/…` 未必失效 ——')
console.log('    我上一轮「必白屏」的表述属**手推未验证，已降级**。')
console.log('  · 仍保留 `base:\'./\'`：相对路径在「根挂载」与「子路径」下**都**正确，')
console.log('    是不依赖未验证前提的唯一选择。')
