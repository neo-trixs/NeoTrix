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
  neobot_convo_list: [],
  neobot_member_list: [],
  neobot_convo_messages: [],
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
  // 无返回值命令
  log_frontend: null,
  neobot_api_call: null,
}

const TAURI_STUB = () => {
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd) => Promise.resolve(STUB_RETURNS[cmd] ?? null),
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e9)
      window[`_${id}`] = cb
      return id
    },
    metadata: { currentWindow: { label: 'main' } },
  }
}

/** 加载一页并返回渲染事实。 */
async function probe(browser, label) {
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

  await page.addInitScript(TAURI_STUB)
  let httpStatus = 0
  let children = 0
  let textLen = 0
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
  } catch (e) {
    errors.push(`navigation: ${e.message}`)
  }
  await page.close()
  return { label, httpStatus, children, textLen, errors, badUrls }
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
    await page.addInitScript(TAURI_STUB)
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
