#!/usr/bin/env node
/**
 * 布局门 v2 —— 用 headless Chrome **实测真界面**，不靠肉眼。
 *
 * # 为什么必须有这道门
 *
 * 静态门（类型 / 逻辑自测 / build）全部能绿，而界面**依然可能是坏的**。
 * 本仓历史上最贵的一类 bug 正是几何：「列表滚不动、输入框被顶出视口」。
 * v1 守旧自研 UI（`.convs-host` / `--nb-*`），旧 UI 删除后退役；
 * 本版探针全部打在新自持区 `[data-testid="neobot-root"]` 上。
 *
 * # 无后端怎么跑起来（stub-boot）
 *
 * 真 Tauri IPC 在纯 Chrome 里不存在。门在页面脚本执行前注入
 * `window.__TAURI_INTERNALS__` 桩（与官方 `@tauri-apps/api/mocks` 同手法，
 * 但 mocks 模块是 ESM，CDP 注入点 import 不了裸包名，故手写最小面）：
 *   - `invoke`：自持 boot 链全量 canned（40 会话 / 60 消息），
 *     未知非 `plugin:*` 调用记入 `__UNMOCKED__` 并抛错 ——
 *     前端新增调用会立刻现形，而不是静默走 `.catch`；
 *   - `plugin:*` 一律回 null（Tauri 插件面与布局无关；`plugin:store|load`
 *     例外抛错 —— spike 实证抛错路径是干净的，回 null 反而把 TypeError
 *     推迟到驱动方法调用时）。
 *   - `__TAURI_EVENT_PLUGIN_INTERNALS__`：空实现。缺它时 `listen` 的
 *     unlisten 路径报 `unregisterListener` 未定义（spike 实测 5 次），
 *     那是桩的错，不是应用的错。
 * 判据含「零 JS 异常」—— 桩盖住的地方应用若还报错，必是真问题。
 *
 * # 判据
 *
 *   ① 自持分支渲染（neobot-root 在、无 iframe）—— 空 service_url 时挂
 *      iframe 会加载不存在的地址，探针直接断言分支本身。
 *   ② 零 JS 异常（error + unhandledrejection）。
 *   ③ 会话列表 40 条注入后真的可滚（scrollHeight > clientHeight）。
 *   ④ 输入框在视口内。
 *   ⑤ 无横向溢出。
 *   ⑥ chrome 翻色：textarea 背景与标题颜色在 light/dark 下不同
 *      （语义 token 经 `html[data-theme]` 翻色；气泡是固定内容色，不参与）。
 *   ⑦ a11y（只查自持区）：img 必有 alt；button 必有可读名；
 *      textarea 必有 label 或 placeholder。
 *   ⑧ Pessoa：未知非 plugin 调用为零（`__UNMOCKED__` 为空）。
 */

import { spawn } from "node:child_process";
import { mkdtempSync, rmSync, existsSync, readFileSync } from "node:fs";
import { createServer, get } from "node:http";
import { tmpdir } from "node:os";
import { join, dirname, extname } from "node:path";
import { neobotDistFresh } from './nt_dist_freshness.mjs';
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/neobot-ui/dist");
// ⭐⭐ 2026-10-07：真源从 `frontend/dist` 改为 `neobot-ui/dist`（**交付树**）。
//
//    ⛔ 改前指向 `apps/neobot-desktop/frontend/dist` —— 那棵树：
//      ① `AGENTS.md` §0 明写是**参考树、不可交付**；
//      ② `.gitignore` 第 334 行**显式忽略** `apps/neobot-desktop/frontend/dist/`
//         （注释写明它是「vite build 的输出」）⇒ CI 里**永远不会被构建**。
//    ⇒ 这道门**从来跑不起来**，而 `STATUS.md` §1.5 却把它列在「门禁（10 个）」
//    里、§5 第 5/6 条还写「已闭合」。**声称与事实分叉**，且分叉方向正是
//    「文档说有门、实际没有」—— 本仓 §4 教训 38 的形状。
//
//    ⭐ 佐证「本意就是交付树」而不是我改了它的口径：门里的 stub 注册的是
//      `neobot_convo_list` / `neobot_core_capabilities` / `neobot_usage_summary` /
//      `neobot_memory_list` —— **全部只存在于 `neobot-ui/` 的自持根**，
//      vendored 树根本没有这些调用。⇒ 只是路径忘了改。
const W = Number(process.env["NB_W"] || 1280);
const H = Number(process.env["NB_H"] || 840);
const PORT = 9341;

if (!existsSync(join(DIST, "index.html"))) {
  console.error("布局门 FAIL: dist 不存在 —— 先跑 cd apps/neobot-desktop/neobot-ui && ./node_modules/.bin/vite build");
  process.exit(1);
}

/** statSync 的安全版：文件可能在遍历途中消失（共享工作树）。 */
function safeStat(p) { try { return statSync(p); } catch { return null; } }

// ⭐⭐ 2026-10-07 P0-1：产物新鲜度断言已抽成**公用件**
//    `scripts/ops/nt_dist_freshness.mjs`（17 道读 dist 的门统一走那一条），
//    本门改为调用它。⛔ 内联副本 = 第二份待腐化的真源。
neobotDistFresh('布局门');

// ── 退役 guard（v1 探针是旧 UI 选择器；本文件已是 v2，留此注释防回退） ──
// 若 `frontend/src/ui/tokens.css` 回来了（旧 UI 回潮），下面的 v2 探针
// 打在旧界面上会全灭 —— 那时先读本注释，再决定修探针还是修界面。
const OLD_UI = existsSync(join(ROOT, "apps/neobot-desktop/frontend/src/ui/tokens.css"));

// ── stub-boot 注入 ─────────────────────────────────────────────
const STUB = `(() => {
  window.__CALLS__ = [];
  window.__UNMOCKED__ = [];
  window.__ERRORS__ = [];
  window.addEventListener('error', (e) => window.__ERRORS__.push(String(e.message).slice(0, 160)));
  window.addEventListener('unhandledrejection', (e) => window.__ERRORS__.push('unhandled:' + String((e.reason && e.reason.message) || e.reason).slice(0, 160)));
  let cbId = 100;
  const convos = Array.from({ length: 40 }, (_, i) => ({ id: 'c' + (i + 1), kind: 'group', title: '会话' + (i + 1), members: ['neo'], task_count: i, last_active: new Date().toISOString(), muted: false, unread: 0 }));
  const MAP = {
    get_dsh_theme: 'system',
    get_runtime_info: { service_url: '', has_service: false, host: 'neobot' },
    runtime_ready: true,
    install_dependencies: null,
    log_frontend: null,
    set_language: null,
    is_dev_build: false,
    get_desktop_about: { version: '0.0.0-gate', published_at: '', copyright: 'gate', repo: '', powered_by: 'gate' },
    read_run_logs: 'gate logs',
    get_dsh_plugins: [],
    check_desktop_update: null,
    neobot_convo_list: convos,
    neobot_core_capabilities: { crystal_version: 'gate', tool_count: 6, model: 'gate-model', model_source: 'gate-src' },
    neobot_usage_summary: { days: 1, input: 0, cached: 0, written: 0, output: 0, requests: 0, tokens: 0, rows: [] },
    neobot_memory_list: { lines: [], bytes: 0, cap: 8192, revisions: 0 },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { registerListener() {}, unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main', windowLabel: 'main' } },
    transformCallback: () => ++cbId,
    unregisterCallback: () => {},
    convertFileSrc: (p) => 'asset://localhost/' + p,
    invoke: async (cmd, args) => {
      window.__CALLS__.push(cmd);
      // ⭐ 2026-10-07 两处修正（都因为**门停了太久**，桩停在旧形态）：
      //  ① 命令改名：\`neobot_convo_messages\` → \`neobot_convo_messages_page\`
      //     （全量命令已删；见 api.rs:117 记的「改名后下游没跟」复发链）。
      //     ⛔ 不改的后果正是本次实测到的失败：门报「未登记调用」，
      //     且因为**没注册就抛 UNMOCKED**，页面拿不到历史 ⇒
      //     「消息流只渲染了 0 个气泡」—— 两个症状其实**同一个原因**。
      //     这是本仓 §4 教训 25 的形状：只处理第一个症状会把第二个当成另一个 bug。
      //  ② 返回形状：\`{messages, hasMore, nextSeq}\`（MessagePage），不是裸数组。
      if (cmd === 'neobot_convo_messages_page') {
        const msgs = Array.from({ length: 60 }, (_, i) => ({ id: 'm' + i, seq: i + 1, convo_id: (args && args.convoId) || 'c1', role: i % 2 ? 'assistant' : 'user', text: '消息' + i + '：这是一条足够长的测试消息，用来把消息流撑出可滚动的高度。', created_at: new Date().toISOString() }));
        // ⛔ hasMore 必须**为真**且给游标，否则「加载更早」不渲染 ⇒ 门验不到那条路径。
        return { messages: msgs, hasMore: true, nextSeq: 1 };
      }
      // ⭐ 轨迹读口（2026-10-07 新增两条命令 ⇒ 桩必须跟，否则门报未登记）。
      //   3 条 run + 每轮 4 步 + 1 条文件改动：够门分别断言
      //   「列表渲染了」「展开能拉到详情」「步与改动都在」。
      if (cmd === 'neobot_run_list') {
        return { runs: Array.from({ length: 3 }, (_, i) => ({ id: 't' + (i + 1), title: '轮次' + (i + 1), status: i === 1 ? 'failed' : 'done', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), error: i === 1 ? 'boom' : null, steps: 4, failed_steps: i === 1 ? 1 : 0 })) };
      }
      if (cmd === 'neobot_run_trace') {
        const id = (args && args.taskId) || 't1';
        return {
          run: { id: id, title: '轮次', status: 'done', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), error: null, steps: 4, failed_steps: 0 },
          steps: Array.from({ length: 4 }, (_, i) => ({ id: i + 1, n: i, tool: i === 0 ? 'bash' : i === 3 ? 'reply' : 'read', ok: i !== 2, output: '步骤输出 ' + i + '\\n' + Array.from({ length: 12 }, (_, k) => '第 ' + (k + 1) + ' 行输出').join('\\n'), tool_call_id: i === 0 ? 'c1' : null })),
          changes: [{ id: 'ch1', at: new Date().toISOString(), path: 'src/lib.rs', kind: 'edit', bytes: 128, content_omitted: false }],
        };
      }
      if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') return 1;
      if (cmd === 'plugin:store|load') throw new Error('UNMOCKED:plugin:store|load');
      if (typeof cmd === 'string' && cmd.startsWith('plugin:')) return null;
      if (cmd in MAP) return MAP[cmd];
      window.__UNMOCKED__.push(cmd);
      throw new Error('UNMOCKED:' + cmd);
    },
  };
})()`;

// ── 页面内探针 ─────────────────────────────────────────────────
const PROBE = `(() => {
  const root = document.querySelector('[data-testid="neobot-root"]');
  if (!root) {
    return { ready: false, errors: window.__ERRORS__ || [], unmocked: window.__UNMOCKED__ || [] };
  }
  const cs = (el, prop) => window.getComputedStyle(el)[prop];
  // 会话列表的滚动容器是侧栏**内层**（侧栏现在是三段 flex：筛选/列表/记忆）。
  // ⛔ 用 testid 而不是 aside 选择器：外层一旦再分一段，门就会去断言一个不滚动的壳，
  //    报「overflow-y=visible」—— 报错对但指错对象，比不报更费时间。
  // ⛔ 本段在模板字符串里，注释中不得出现反引号（会提前结束字符串）。
  const aside = root.querySelector('[data-testid="nb-convo-list"]') || root.querySelector('aside');
  const section = root.querySelector('section');
  const msgs = section ? section.querySelector('div.overflow-y-auto') : null;
  const composer = root.querySelector('textarea');
  const title = root.querySelector('header span');
  const rows = aside ? aside.querySelectorAll('button').length : 0;
  const bubbles = msgs ? msgs.querySelectorAll(':scope > div > div > div').length : 0;
  const r = composer ? composer.getBoundingClientRect() : null;
  const vw = window.innerWidth, vh = window.innerHeight;
  // a11y（只查自持区）：img/button/textarea。
  const a11y = [];
  root.querySelectorAll('img').forEach((img) => {
    if (img.getAttribute('alt') === null) a11y.push('img 缺 alt：' + (img.getAttribute('src') || '?').slice(0, 60));
  });
  root.querySelectorAll('button').forEach((b) => {
    const name = (b.getAttribute('aria-label') || b.textContent || '').trim();
    if (!name) a11y.push('button 无可读名');
  });
  root.querySelectorAll('textarea').forEach((t) => {
    if (!t.getAttribute('aria-label') && !t.getAttribute('placeholder')) a11y.push('textarea 无 label/placeholder');
  });
  const px = (v) => Number(String(v).replace('px', '')) || 0;
  return {
    // 2026-10-07 补诊断字段：selBg 为 null 时**必须能说出为什么**，
    //    否则报告只有一句「选中态与未选中态相同（null vs …）」，读者无从下手
    //    （本轮就为这个 null 反复猜了三轮：端口？时序？半就绪？—— 全是猜）。
    //    ⛔ 「报不出原因的空值」和「断言失败」一样有害。
    ariaCurrentCount: root.querySelectorAll('[aria-current="true"]').length,
    listButtonCount: root.querySelectorAll('[data-testid="nb-convo-list"] button').length,
    selBg: (() => {
      const b = root.querySelector('[aria-current="true"]');
      return b ? window.getComputedStyle(b).backgroundColor : null;
    })(),
    unselBg: (() => {
      const list = root.querySelectorAll('[data-testid="nb-convo-list"] button');
      const b = [...list].find(x => x.getAttribute('aria-current') !== 'true');
      return b ? window.getComputedStyle(b).backgroundColor : null;
    })(),
    ready: true,
    hasIframe: !!document.querySelector('iframe'),
    rows, bubbles,
    aside: aside ? {
      overflowY: cs(aside, 'overflowY'), minHeight: cs(aside, 'minHeight'),
      sh: aside.scrollHeight, ch: aside.clientHeight,
      overflowing: aside.scrollHeight > aside.clientHeight + 1,
    } : null,
    msgs: msgs ? {
      overflowY: cs(msgs, 'overflowY'), minHeight: cs(msgs, 'minHeight'),
      sh: msgs.scrollHeight, ch: msgs.clientHeight,
      overflowing: msgs.scrollHeight > msgs.clientHeight + 1,
    } : null,
    composerInViewport: r ? (r.top >= 0 && r.bottom <= vh + 1) : false,
    hOverflow: document.scrollingElement.scrollWidth > vw + 1,
    appH: document.scrollingElement.scrollHeight,
    vh, vw,
    light: {
      areaBg: composer ? cs(composer, 'backgroundColor') : null,
      titleColor: title ? cs(title, 'color') : null,
    },
    errors: window.__ERRORS__ || [],
    unmocked: window.__UNMOCKED__ || [],
    a11y,
    calls: [...new Set(window.__CALLS__ || [])],
  };
})()`;

// ── CDP  Plumbing（沿用 v1：显式 kill，不靠浏览器善后） ──────────
const CHROME_CANDIDATES = [
  process.env["CHROME_PATH"],
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
].filter(Boolean);

function findChrome() {
  const hit = CHROME_CANDIDATES.find((p) => existsSync(p));
  if (!hit) {
    console.error("布局门: 找不到 Chrome/Edge。用 CHROME_PATH=<路径> 指定。");
    process.exit(2);
  }
  return hit;
}

const MIME = {
  ".html": "text/html;charset=utf-8", ".js": "text/javascript;charset=utf-8",
  ".css": "text/css;charset=utf-8", ".svg": "image/svg+xml",
  ".png": "image/png", ".json": "application/json",
};

const server = createServer((req, res) => {
  const url = (req.url || "/").split("?")[0];
  const p = join(DIST, url === "/" ? "index.html" : url.slice(1));
  if (!p.startsWith(DIST)) { res.writeHead(403).end(); return; }
  if (!existsSync(p)) { res.writeHead(404).end("not found"); return; }
  res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
  res.end(readFileSync(p));
});

/**
 * 等 CDP 端点就绪 —— ⛔ **轮询**，不是固定 sleep。
 *
 * # 为什么必须轮询（实测，非推理）
 *
 * ⛔ 改前是 `await new Promise(r => setTimeout(r, 1500))` 然后直接连。
 * ⓘ 实测：Chrome **冷启动**时 1500ms 不够 ⇒ `connect ECONNREFUSED` ⇒
 *   门报「环境问题」红。热态复跑则绿 ⇒ 表现为**间歇性**失败。
 * ⓘ 而 GitHub Actions 的 runner **每次都是冷的** ⇒ ⛔ 固定 sleep 在 CI 上
 *   近乎必然踩中 ⇒ 门红 ⇒ **跳过后续步骤** ⇒ 正是本轮刚修的那个老问题复发。
 *
 * ✅ 这里改成「连不上就重试，直到超时」，并把**超时**做成硬失败
 *   （⛔ 不把「跑不出结果」当「没问题」）。
 */
async function waitForCdp(port, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let lastErr = null;
  while (Date.now() < deadline) {
    try {
      return await cdpGet(`http://127.0.0.1:${port}/json`);
    } catch (e) {
      lastErr = e;
      await new Promise((r) => setTimeout(r, 250));
    }
  }
  throw new Error(`CDP ${port} 在 ${timeoutMs}ms 内未就绪：${lastErr}`);
}

function cdpGet(url) {
  return new Promise((res, rej) => {
    get(url, (r) => {
      let s = "";
      r.on("data", (c) => (s += c));
      r.on("end", () => res(JSON.parse(s)));
    }).on("error", rej);
  });
}

function connect(url) {
  return new Promise((res, rej) => {
    import("node:stream/web").catch(() => {});
    const ws = new WebSocket(url);
    let id = 0;
    const pending = new Map();
    const send = (method, params) =>
      new Promise((rs, rj) => {
        const mid = ++id;
        pending.set(mid, { rs, rj });
        ws.send(JSON.stringify({ id: mid, method, params }));
        setTimeout(() => {
          if (pending.has(mid)) { pending.delete(mid); rj(new Error(`${method} 超时`)); }
        }, 20000);
      });
    ws.onopen = () => res({ ws, send });
    ws.onerror = rej;
    ws.onmessage = (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && pending.has(m.id)) {
        const { rs, rj } = pending.get(m.id);
        pending.delete(m.id);
        m.error ? rj(new Error(JSON.stringify(m.error))) : rs(m.result);
      }
    };
  });
}

const chrome = findChrome();
const profile = mkdtempSync(join(tmpdir(), "nb-layout-"));
let child = null;
let failed = 2;

try {
  await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
  child = spawn(chrome, [
    "--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run",
    "--disable-extensions", "--disable-background-networking",
    `--user-data-dir=${profile}`,
    `--window-size=${W},${H}`,
    "--remote-debugging-port=9333",
    "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  child.on("error", () => {});
  // ⭐ 2026-10-07：固定 1500ms sleep 改为**轮询就绪**（见 waitForCdp 的理由）。
  const targets = await waitForCdp(9333);
  const page = targets.find((t) => t.type === "page") || targets[0];
  const { ws, send } = await connect(page.webSocketDebuggerUrl);
  try {
    await send("Page.enable");
    await send("Runtime.enable");
    await send("Emulation.setDeviceMetricsOverride", {
      width: W, height: H, deviceScaleFactor: 1, mobile: false,
    });
    await send("Page.addScriptToEvaluateOnNewDocument", { source: STUB });
    await send("Page.navigate", { url: process.env["NB_URL"] || `http://127.0.0.1:${PORT}/` });
    // 等 neobot-root 出现（boot 是异步链），而不是死等固定毫秒。
    let d = null;
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 500));
      const out = await send("Runtime.evaluate", { expression: PROBE, returnByValue: true, awaitPromise: true });
      if (out.exceptionDetails) throw new Error("探针异常：" + (out.exceptionDetails.text || "?"));
      d = out.result.value;
      // ⭐⭐ 2026-10-07 修**门自身的两处竞态**（实测 5 跑里 3 次假红，逐个定位）。
      //
      // 【竞态 1】等待条件在等一个**永远达不到**的数
      //   ⛔ 改前是 `d.rows >= 40 && d.bubbles >= 60`，而两处列表**已窗口化**
      //     （`useVirtualizer`）⇒ DOM 里**永远只有约 20 行 / 16 个气泡**
      //     ⇒ 条件永不成立 ⇒ 循环每次空跑满 20 秒，然后**拿半就绪的页面**断言。
      //   ⛔⛔ 且它**自相矛盾**：断言已改成「rows >= 40 ⇒ 判定窗口化被回退」，
      //     一旦真等到 `rows >= 40`，**门会立刻判自己失败**。
      //   ⇒ 教训：**改断言必须同步改等待条件**，两者是同一不变量的两半。
      //
      // 【竞态 2】⭐ 真正让 `selBg` 时有时无的原因（**加了诊断字段才定位到**）：
      //   桩造 40 个会话，`last_active` 用 `new Date().toISOString()` ——
      //   **毫秒级**。⇒ 有时 40 个拿到同一毫秒（排序稳定，c1 落在顶部），
      //   有时各差 1ms（排序被打乱，**c1 可能掉到第 30 位**）。
      //   ⭐ 而界面**已窗口化**：DOM 里只有约 20 个会话项
      //   ⇒ 选中的 c1 **可能在渲染窗口之外** ⇒ `aria-current` 根本不在 DOM 里
      //   ⇒ `selBg` 读成 `null`。
      //   ⛔ **这不是产品缺陷**：用户滚到那儿就能看到选中态；滚动窗口里
      //     本来就不该期待「任意某个选中项此刻在 DOM 中」。
      //
      // ✅ 修法（不靠运气）：**先点一个「此刻确实渲染着」的行**再等选中态。
      //   点的是 DOM 里第一个会话项 ⇒ 按定义就在窗口内 ⇒ 确定性成立。
      //   ⭐ 顺带把「选中态」从「碰巧成立」变成「**真的点出来的**」——
      //   这比原来更强，不是放松判据。
      if (d.ready && d.rows > 0 && d.bubbles > 0) {
        if (d.ariaCurrentCount === 0) {
          // 此刻 DOM 里没有任何 aria-current ⇒ 点第一个渲染中的会话项
          await send("Runtime.evaluate", {
            expression: `(() => {
              const list = document.querySelector('[data-testid="nb-convo-list"]');
              const first = list && list.querySelector('button[data-testid="nb-convo-item"]');
              if (!first) return false;
              first.click();
              return true;
            })()`,
            returnByValue: true,
          });
          continue; // 点完让 React 重渲染，下一轮再量
        }
        if (d.selBg) break;
      }
    }
    // 暗色翻色：直接切 data-theme（偏好管线 resolveTheme 是纯函数，不在此测）。
    const themeExpr = `(async () => {
      const ta = document.querySelector('[data-testid="neobot-root"] textarea');
      const th = document.querySelector('[data-testid="neobot-root"] header span');
      const cur = document.documentElement.dataset.theme;
      const bg = (el) => el ? window.getComputedStyle(el).backgroundColor : null;
      const fg = (el) => el ? window.getComputedStyle(el).color : null;
      document.documentElement.dataset.theme = 'light';
      await new Promise((r) => setTimeout(r, 200));
      const l = { areaBg: bg(ta), titleColor: fg(th) };
      document.documentElement.dataset.theme = 'dark';
      await new Promise((r) => setTimeout(r, 200));
      const k = { areaBg: bg(ta), titleColor: fg(th) };
      document.documentElement.dataset.theme = cur;
      return { light: l, dark: k };
    })()`;
    const themeOut = await send("Runtime.evaluate", { expression: themeExpr, returnByValue: true, awaitPromise: true });
    if (themeOut.exceptionDetails) throw new Error("暗色探针异常");
    d.theme = themeOut.result.value;

    // ── 判据 ──
    const bad = [];
    if (OLD_UI) bad.push("旧 UI 回来了（tokens.css 存在）：v2 探针不适用，先读门头注释");
    if (!d.ready) bad.push("自持根没渲染（[data-testid=neobot-root] 不存在）");
    if (d.hasIframe) bad.push("自持模式下出现了 iframe（分支错了）");
    for (const e of d.errors) bad.push(`JS 异常：${e}`);
    for (const u of d.unmocked) bad.push(`未登记的调用：${u}（加进桩 MAP 或契约）`);
    // 选中态必须与未选中态**可区分**（背景不同色）。
    // ⛔ 上一轮之前选中态与 hover 态同为 bg-panel-hover，肉眼在截图里看不出差别。
    const sel = d.selBg, unsel = d.unselBg;
    if (!sel || !unsel || sel === unsel) {
      bad.push(
        `选中态与未选中态背景相同（${sel} vs ${unsel}）—— 等于没有选中态` +
          `｜诊断：aria-current 数=${d.ariaCurrentCount} 列表按钮数=${d.listButtonCount}`,
      );
    }

    // ⭐⭐ 2026-10-07：这两条断言**写在窗口化之前**，门停了太久没跟着改。
    //
    // ⛔ 改前断言「DOM 里必须有 40 条会话 / 60 个气泡」，而两处列表
    //    **都已窗口化**（`useVirtualizer`，见 `neobot-root.tsx` 的
    //    `convoRows` 拍平 + `msgVirtualizer`）⇒ DOM 里**永远只有约 20 / 16 个**。
    //    ⇒ 门在「全部渲染」的世界里是对的，在窗口化的世界里是**恒假**。
    //
    // ⛔ 为什么不能把阈值调小来「修好」它：那样门就只在
    //    「窗口里一个都没渲染」时才报 —— 而那恰恰是**该报**的故障。
    //    ⇒ 改成断言**三件真正要保证的事**：
    //      ① 窗口内确实渲染出了东西（> 0）——「一个都没渲染」是真故障；
    //      ② 滚动容器**真的溢出**（已在别处断言）⇒ 窗口化在起作用；
    //      ③ 渲染数**显著小于**桩给的总数 ⇒ 证明没有退化成全量渲染。
    //    ⓘ ③ 是这条断言的**真正价值**：它是「窗口化没被回退」的守卫。
    if (d.rows <= 0) bad.push(`会话列表窗口内一条都没渲染（桩给了 40 条）`);
    if (d.bubbles <= 0) bad.push(`消息流窗口内一个气泡都没渲染（桩给了 60 条）`);
    if (d.rows >= 40) {
      bad.push(`会话列表渲染了全部 40 条 ⇒ 窗口化被回退了（长会话会一次建上千个节点）`);
    }
    if (d.bubbles >= 60) {
      bad.push(`消息流渲染了全部 60 个气泡 ⇒ 消息窗口化被回退了（长会话曾一次塞满 DOM）`);
    }
    for (const [name, r] of [["会话列表", d.aside], ["消息流", d.msgs]]) {
      if (!r) { bad.push(`${name} 容器不存在`); continue; }
      if (r.overflowY !== "auto" && r.overflowY !== "scroll") {
        bad.push(`${name} 的 overflow-y=${r.overflowY}（内容一多就撑破而不是滚动）`);
      }
      if (!r.overflowing) {
        bad.push(`${name} 内容超出（${r.sh} > 容器 ${r.ch}）但不可滚 —— overflow 未生效`);
      }
      // 高度链：纵向滚动容器自己不能是 min-height:auto（grid/flex 下拒绝收缩）。
      if (r.minHeight !== "0px" && r.minHeight !== "0") {
        bad.push(`${name} 的 min-height=${r.minHeight}（grid/flex item 默认 auto 会撑高父级）`);
      }
    }
    if (d.composerInViewport === false) bad.push("输入框不在视口内");
    if (d.hOverflow) bad.push("出现横向溢出（通常是布局 bug 的症状）");
    if (d.theme.light.areaBg === d.theme.dark.areaBg) {
      bad.push(`暗色未翻色：输入框背景 light=${d.theme.light.areaBg} dark=${d.theme.dark.areaBg}`);
    }
    if (d.theme.light.titleColor === d.theme.dark.titleColor) {
      bad.push(`暗色未翻色：标题颜色 light=${d.theme.light.titleColor} dark=${d.theme.dark.titleColor}`);
    }
    const a11y = d.a11y ?? [];
    for (const a of a11y) bad.push(`a11y：${a}`);

    console.log(`  视口 ${W}x${H} · 会话 ${d.rows} 条 · 气泡 ${d.bubbles} 个`);
    console.log(`    选中/未选中底色 ${d.selBg} vs ${d.unselBg}`);
    console.log(`    列表 overflow-y=${d.aside?.overflowY} 超出=${d.aside?.overflowing} (${d.aside?.sh}/${d.aside?.ch}) min-height=${d.aside?.minHeight}`);
    console.log(`    消息 overflow-y=${d.msgs?.overflowY} 超出=${d.msgs?.overflowing} (${d.msgs?.sh}/${d.msgs?.ch}) min-height=${d.msgs?.minHeight}`);
    console.log(`    输入框在视口=${d.composerInViewport} 横向溢出=${d.hOverflow} iframe=${d.hasIframe}`);
    console.log(`    light: ${d.theme.light.areaBg} / ${d.theme.light.titleColor}`);
    console.log(`    dark:  ${d.theme.dark.areaBg} / ${d.theme.dark.titleColor}`);
    console.log(`    调用 ${d.calls.length} 种 · 异常 ${d.errors.length} · 未登记 ${d.unmocked.length}`);

    if (bad.length) {
      console.error(`\n布局门 FAIL（${bad.length} 项）:`);
      for (const b of bad) console.error("  · " + b);
      failed = 1;
    } else {
      console.log("\n布局门 PASS（自持分支渲染、列表消息可滚、无溢出、暗色翻色、a11y、无异常）");
      failed = 0;
    }
    ws.close();
  } catch (e) {
    try { ws.close(); } catch {}
    throw e;
  }
} catch (e) {
  // 「没测到」与「测坏了」必须分开
  console.error("布局门 FAIL: 环境问题（不是布局问题）—— " + (e instanceof Error ? e.message : String(e)));
  console.error("  · 常见病因: Chrome 启动失败 / CDP 端口被占 / dist 未构建");
  failed = 1;
} finally {
  try { server.close(); } catch {}
  if (child) child.kill("SIGKILL");
  try { rmSync(profile, { recursive: true, force: true }); } catch {}
}
process.exit(failed);
