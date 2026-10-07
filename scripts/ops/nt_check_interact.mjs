#!/usr/bin/env node
/**
 * 交互门 —— 在 stub-boot 的真生产包里**真点真输**，不止于渲染。
 *
 * # 为什么必须有这道门
 *
 * 布局门只证明「界面长出来了」。但「输入框能打字、回车真发送、
 * 切会话真换历史、宠物真渲染」是另一类事实 —— 它们是**交互**，
 * 静态门与渲染门都看不见。本仓 `neobot_send` 的 convo 落库、
 * 切会话换历史、`get_pet_asset` 的 data URL，全是为交互写的，
 * 不点一次等于没测。
 *
 * # 两页
 *
 *   A. 主页 `/`：输入框打字 → 回车 → bot 气泡出现，且桩看到的
 *      `neobot_send` 参数是 `{convo_id:'c1', text}`；再点第二个会话，
 *      历史切到 c2（`neobot_convo_messages` 收到 c2）。
 *   B. 宠物页 `/pet.html`：状态开 + 选中内置海豚，`get_pet_asset`
 *      返回**真实 256 图的 data URL**（门脚本读盘内嵌，不是合成像素），
 *      断言渲染出的媒体元素有布局尺寸；截图落 `/tmp/nb-pet.png` 供人目检。
 *      1×1 静帧能否播，机器断言「元素在」，人目检「像不像」——
 *      两段证据拼在一起才算闭合。
 *
 * # 桩与布局门同源
 *
 * IPC 桩手法见 `nt_check_layout.mjs` 头注释（`__TAURI_INTERNALS__` 手写最小面）。
 * React 受控输入必须用原生 setter + 先 input 后等 300ms 再 keydown ——
 * 同步连发时 state 还没合入，send 读到旧值直接 return（这是 React 的语义，
 * 不是应用的 bug；探针必须按真实时序发事件）。
 */

import { spawn } from "node:child_process";
import { mkdtempSync, rmSync, existsSync, readFileSync } from "node:fs";
import { createServer, get } from "node:http";
import { tmpdir } from "node:os";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";
import { neobotDistFresh } from './nt_dist_freshness.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/neobot-ui/dist");
// ⭐⭐ 2026-10-07 P0-1：产物新鲜度断言（公用件，17 道读 dist 的门统一走这条）。
//    ⛔ 不新鲜就在**开浏览器之前**退出 —— 否则白等几十秒再失败。
neobotDistFresh('nt_check_interact');
const W = 1280, H = 820;
const PORT = 9342;
const DOLPHIN_PNG = readFileSync(join(ROOT, "apps/neobot-desktop/icons/256x256.png")).toString("base64");

if (!existsSync(join(DIST, "index.html")) || !existsSync(join(DIST, "pet.html"))) {
  console.error("交互门 FAIL: dist 不存在 —— 先跑 cd apps/neobot-desktop/neobot-ui && ./node_modules/.bin/vite build");
  process.exit(1);
}

const BASE_STUB = `
  window.__CALLS__ = [];
  window.__ERRORS__ = [];
  window.addEventListener('error', (e) => window.__ERRORS__.push(String(e.message).slice(0, 160)));
  window.addEventListener('unhandledrejection', (e) => window.__ERRORS__.push('unhandled:' + String((e.reason && e.reason.message) || e.reason).slice(0, 160)));
  let cbId = 100;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { registerListener() {}, unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main', windowLabel: 'main' } },
    transformCallback: () => ++cbId,
    unregisterCallback: () => {},
    convertFileSrc: (p) => 'asset://localhost/' + p,
    invoke: async (cmd, args) => {
      window.__CALLS__.push(cmd + ':' + JSON.stringify(args || {}));
      return HANDLE(cmd, args || {});
    },
  };
`;

const MAIN_STUB = `(() => {
  ${BASE_STUB}
  const convos = Array.from({ length: 5 }, (_, i) => ({ id: 'c' + (i + 1), kind: 'group', title: '会话' + (i + 1), members: ['neo'], task_count: 0, last_active: new Date().toISOString(), muted: false, unread: 0 }));
  window.__SEND_ARGS__ = null;
  window.__MSG_CALLS__ = [];
  const HANDLE = (cmd, args) => {
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
      neobot_memory_list: { lines: ['门记忆A', '门记忆B'], bytes: 30, cap: 8192, revisions: 3 },
      neobot_member_list: [{ id: 'neo', kind: 'agent', display: 'neo' }],
      neobot_convo_group: 'c-new',
    };
    if (cmd === 'neobot_member_add') {
      window.__MEMBER_ADD__ = args.id;
      return null;
    }
    if (cmd === 'neobot_memory_add') {
      window.__MEM_ADD__ = args.text;
      return true;
    }
    if (cmd === 'neobot_memory_undo') {
      window.__MEM_UNDO__ = (window.__MEM_UNDO__ || 0) + 1;
      return true;
    }
    if (cmd === 'neobot_send') {
      window.__SEND_ARGS__ = args;
      // 富文本探针体：加粗 + 代码围栏 + script 标签 ——
      // 断言渲染出 pre/strong 且 script 被转义（模板字符串里写不出反引号，
      // 用 fromCharCode 拼 fence；换行在门模板里必须写成双写形式，否则它先求值
      // 成真换行，桩源码的单引号字符串就被截断 —— 本轮现抓，连注释里示范都不行）。
      const fence = String.fromCharCode(96, 96, 96);
      return { status: 'ok', output: 'STUB-REPLY:**加粗**\\n' + fence + 'ts\\nconst a = 1\\n' + fence + '\\n<script>alert(1)</script>', trace: [], model_used: 'stub', mode: 'passthrough', tools: [], usage: null };
    }
    // 2026-10-07 修（与 nt_check_layout.mjs 同一个病因）：
    //   1) 命令改名 neobot_convo_messages -> neobot_convo_messages_page
    //      （全量命令已删，见 api.rs:117 记的「改名后下游没跟」复发链）；
    //   2) 键名 convo_id -> **convoId**（Tauri 取 camelCase 键；
    //      写 snake_case 会**静默变 undefined** —— STATUS 教训 28 记的那个 P0，
    //      桩里犯同样的错就是自己骗自己）；
    //   3) 返回 MessagePage 形状 {messages,hasMore,nextSeq}，不是裸数组。
    //   不改的后果（本次实测）：桩不命中 -> 落进 unmocked 抛错 -> 历史为空
    //      -> 界面显示「还没有对话」-> 门报「切会话换历史等不到」。
    //      而真凶是**桩停在旧形态**，不是界面坏了 —— 差一步就会去「修」正确代码。
    //
    // 注意：这段注释在**模板字符串**里，写它时不能出现反引号，也不能出现美元加大括号
    // （那是模板插值）。这是 STATUS §9 记过的坑：node 报 Unexpected identifier，
    // bash -n 与 tsc 都抓不到，只有真跑门才暴露。
    if (cmd === 'neobot_convo_messages_page') {
      const cid = (args && args.convoId) || 'c?';
      window.__MSG_CALLS__.push(cid);
      const messages = Array.from({ length: 3 }, (_, i) => ({ id: cid + '-m' + i, seq: i + 1, convo_id: cid, role: i % 2 ? 'assistant' : 'user', text: cid + '的历史' + i, created_at: new Date().toISOString() }));
      return { messages: messages, hasMore: false, nextSeq: null };
    }
    if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') return 1;
    if (cmd === 'plugin:store|load') throw new Error('UNMOCKED:plugin:store|load');
    if (cmd.startsWith('plugin:')) return null;
    if (cmd in MAP) return MAP[cmd];
    throw new Error('UNMOCKED:' + cmd);
  };
})()`;

const PET_STUB = `(() => {
  ${BASE_STUB}
  const HANDLE = (cmd, args) => {
    const MAP = {
      log_frontend: null,
      get_pet_status: { enabled: true, visible: true, active_pet: 'codex:neobot-dolphin', pet_size: 100 },
      list_preset_pets: [],
      move_pet_window: null,
      start_pet_mouse_stream: null,
      get_pet_overlay_supported: true,
      get_force_xwayland: false,
      // 窗几何：Tauri Rust 枚举外部标记序列化（{"Physical": {...}}），
      // 调用方用 'Physical' in pos 判别 —— 形状错一个键就抛（实测）。
      // null 更不行：new PhysicalPosition(null) 直接炸（实测）。
      'plugin:window|inner_position': { Physical: { x: 100, y: 100 } },
      'plugin:window|outer_position': { Physical: { x: 100, y: 100 } },
      'plugin:window|inner_size': { Physical: { width: 220, height: 238 } },
      'plugin:window|outer_size': { Physical: { width: 220, height: 238 } },
      'plugin:window|is_visible': true,
      'plugin:window|is_fullscreen': false,
      'plugin:window|is_maximized': false,
      'plugin:window|is_minimized': false,
      'plugin:window|scale_factor': 1,
    };
    if (cmd === 'get_pet_asset') {
      return { id: args.id, spritesheet: 'data:image/png;base64,__DOLPHIN__', sprite_version_number: 1, columns: 1, rows: 1 };
    }
    if (cmd === 'set_pet_ignore_cursor_events') return true;
    if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') return 1;
    if (cmd === 'plugin:store|load') throw new Error('UNMOCKED:plugin:store|load');
    // ⛔ MAP 先于泛 plugin: 分支：窗几何等具体桩若排在后面，永远走不到
    // （实测：inner_position 回了 null 炸了 new PhysicalPosition）。
    if (cmd in MAP) return MAP[cmd];
    if (cmd.startsWith('plugin:')) return null;
    throw new Error('UNMOCKED:' + cmd);
  };
})()`.replace('__DOLPHIN__', DOLPHIN_PNG);

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
    console.error("交互门: 找不到 Chrome/Edge。用 CHROME_PATH=<路径> 指定。");
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
const profile = mkdtempSync(join(tmpdir(), "nb-interact-"));
let child = null;
let failed = 2;
const bad = [];

async function evaluate(send, expression) {
  const out = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (out.exceptionDetails) throw new Error("探针异常：" + (out.exceptionDetails.text || "?"));
  return out.result.value;
}

// ⛔ 注入前必须验桩语法：addScriptToEvaluateOnNewDocument 对语法错
// 静默丢弃（页面空白、无调用、无报错）， symptoms 与「应用没启动」
// 一模一样 —— 本轮为此烧掉四轮 debug。node 侧 new Function 有语法错
// 直接抛，比进浏览器猜快一个数量级。
function assertValidJS(source, label) {
  try {
    new Function(source);
  } catch (e) {
    throw new Error(`桩语法错（${label}）：${e.message}`);
  }
}

async function waitFor(send, expression, timeoutMs, label) {
  const t0 = Date.now();
  for (;;) {
    const v = await evaluate(send, expression);
    if (v) return v;
    if (Date.now() - t0 > timeoutMs) {
      // ⛔ 超时必须带现场：光说"等不到"等于盲查，上次为此多花一轮 debug 脚本。
      const diag = await evaluate(send, `(() => ({
        errors: window.__ERRORS__ || [],
        calls: [...new Set(window.__CALLS__ || [])],
        internals: typeof window.__TAURI_INTERNALS__,
        hasRoot: !!document.querySelector('[data-testid="neobot-root"]'),
        scripts: [...document.querySelectorAll('script')].map((s) => s.src.slice(-40)),
        body: document.body.innerText.slice(0, 200),
      }))()`).catch(() => null);
      throw new Error(`等不到：${label}；现场=${JSON.stringify(diag)}`);
    }
    await new Promise((r) => setTimeout(r, 400));
  }
}

try {
  await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
  child = spawn(chrome, [
    "--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run",
    "--disable-extensions", "--disable-background-networking",
    `--user-data-dir=${profile}`,
    `--window-size=${W},${H}`,
    "--remote-debugging-port=9335",
    "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  child.on("error", () => {});
  // ⭐ 2026-10-07：固定 1500ms sleep 改为**轮询就绪**（见 waitForCdp 的理由）。
  const targets = await waitForCdp(9335);
  const page = targets.find((t) => t.type === "page") || targets[0];
  const { ws, send } = await connect(page.webSocketDebuggerUrl);
  try {
    await send("Page.enable");
    await send("Runtime.enable");
    await send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });

    // ── A. 主页交互 ──
    assertValidJS(MAIN_STUB, "MAIN_STUB");
    await send("Page.addScriptToEvaluateOnNewDocument", { source: MAIN_STUB });
    await send("Page.navigate", { url: `http://127.0.0.1:${PORT}/` });
    await waitFor(send, `(() => { const r = document.querySelector('[data-testid="neobot-root"]'); const l = r && r.querySelector('[data-testid="nb-convo-list"]'); return !!(l && l.querySelectorAll('button').length >= 2); })()`, 20000, "主页会话列表");

    // 打字 → 回车（按真实时序：input 后等 300ms 再 keydown，见门头注释）。
    await evaluate(send, `(() => {
      const ta = document.querySelector('[data-testid="neobot-root"] textarea');
      const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
      setter.call(ta, '你好海豚');
      ta.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    await new Promise((r) => setTimeout(r, 400));
    await evaluate(send, `(() => {
      const ta = document.querySelector('[data-testid="neobot-root"] textarea');
      ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
      return true;
    })()`);
    const sent = await waitFor(send, `(() => {
      const msgs = [...document.querySelectorAll('[data-testid="neobot-root"] section div.overflow-y-auto > div > div > div')];
      const last = msgs[msgs.length - 1];
      if (!last || !last.querySelector('pre') || !last.querySelector('strong')) return null;
      // script 必须被转义成文本（自转义纪律），且不能真执行（零异常在总判据里守）。
      if (!last.innerHTML.includes('&lt;script&gt;')) return null;
      return { bubbles: msgs.length, args: window.__SEND_ARGS__ };
    })()`, 8000, "bot 回复富文本气泡");
    console.log(`  发送链：气泡 ${sent.bubbles} 个 · 后端收到 convoId=${sent.args && sent.args.convoId} text=${sent.args && sent.args.text}`);
    // 2026-10-07 修：⛔ 本断言此前要求 **convo_id**（snake_case），是**反的**。
    //   Tauri 从 IPC body 取的是 **camelCase** 键（宏把形参名转成 camelCase），
    //   而 Option 形参在键名不匹配时**静默变 None**（不报错）⇒ 那正是
    //   STATUS §4 教训 28 记的 P0：「界面正常显示回复、但一条都没落库」。
    //   ⛔⛔ 后果的方向性很要命：门若这样长期红着，人会去「修」**正确的前端代码**
    //     去迁就错的门 —— 把 P0 装回产品。真实 IPC 往返那道
    //     `tests/ipc_roundtrip.rs`（2026-10-02 起）才是这条的正解，
    //     本门只做「生产包里真的发出去了没有」的补充。
    if (!sent.args || sent.args.convoId !== "c1" || sent.args.convo_id !== undefined || sent.args.text !== "你好海豚") {
      bad.push(`neobot_send 参数不对：${JSON.stringify(sent.args)}（应为 {convoId:'c1', text:'你好海豚'}，且**不得**出现 snake_case 的 convo_id）`);
    }

    // 切会话 → 历史必须换到 c2。
    // ⛔ 按标题文本点，不按 `aside button[n]` 下标：侧栏顶部现在有搜索框与「+」，
    //    下标整体位移 —— 按下标点的可能是「+」或搜索区，门却照样可能绿。
    //    （这正是上一轮写进 STATUS 教训 28 的那件事，本轮自己又踩了一次。）
    await evaluate(send, `(() => {
      const b = [...document.querySelectorAll('[data-testid="nb-convo-list"] button')]
        .find(x => (x.textContent || '').includes('会话2'));
      if (!b) throw new Error('NO_CONVO2');
      b.click();
      return true;
    })()`);
    const switched = await waitFor(send, `(() => {
      const calls = window.__MSG_CALLS__ || [];
      // 2026-10-07 换选择器：改前是四层后代链 section > div.overflow-y-auto > div > div > div
      //   那不是选择器，是「当前 DOM 恰好长这样」的快照：改一层包裹
      //   （本次给消息区外面套了页签三元）它就静默失配 => first 为 undefined
      //   => 门报「等不到」，指向完全错误的方向。
      // 改用**稳定 testid**：nb-msg-copy 每条消息各有一个（在气泡内）。
      // 注意：这段注释也在模板字符串里 ⇒ 不能出现反引号（见上面 STUB 处的同款警告）。
      const first = document.querySelector('[data-testid="neobot-root"] [data-testid="nb-msg-copy"]');
      const bubble = first && first.closest('.rounded-2xl');
      if (calls[calls.length - 1] === 'c2' && bubble && (bubble.textContent || '').includes('c2的历史')) {
        return { calls };
      }
      return null;
    })()`, 8000, "切会话换历史");
    console.log(`  切会话：history 调用序列 [${switched.calls.join(", ")}]，首气泡已换 c2`);


    const shot = await send("Page.captureScreenshot", { format: "png" });
    await import("node:fs").then((fs) => fs.writeFileSync("/tmp/nb-main.png", Buffer.from(shot.data, "base64")));
    console.log("  主页截图 → /tmp/nb-main.png");
    // ── 记忆面板：展开 → 记一条 → 撤一版 ──
    // ⛔ 侧栏按钮顺序：记忆按钮是**最后一个**（`aside button` 的末位），
    //    不是索引 0/1。用「按文本找」而非按下标 —— 追加面板后下标全漂，
    //    门会改成一个「点错按钮也绿」的假绿。
    await evaluate(send, `(() => {
      const btns = [...document.querySelectorAll('[data-testid="neobot-root"] aside button')];
      const b = btns.find(x => (x.textContent || '').includes('记忆'));
      if (!b) throw new Error('NO_MEMORY_BTN');
      b.click();
      return true;
    })()`);
    const memOpen = await waitFor(send, `(() => {
      const input = document.querySelector('[data-testid="neobot-root"] aside input[aria-label="新增记忆"]');
      if (!input) return null;
      const ul = [...document.querySelectorAll('[data-testid="neobot-root"] aside li')].map(li => li.textContent);
      return { lines: ul, revisions: true };
    })()`, 8000, "记忆面板展开");
    console.log(`  记忆面板：展开后 ${memOpen.lines.length} 行 [${memOpen.lines.join(" | ")}]`);
    if (memOpen.lines.length < 2 || !memOpen.lines[0].includes('门记忆A')) {
      bad.push(`记忆面板没渲染桩里的记忆行：${JSON.stringify(memOpen.lines)}`);
    }

    await evaluate(send, `(() => {
      const input = document.querySelector('[data-testid="neobot-root"] aside input[aria-label="新增记忆"]');
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
      setter.call(input, '门记忆C');
      input.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    await new Promise((r) => setTimeout(r, 300));
    await evaluate(send, `(() => {
      const input = document.querySelector('[data-testid="neobot-root"] aside input[aria-label="新增记忆"]');
      input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
      return true;
    })()`);
    const memAdd = await waitFor(send, `(() => window.__MEM_ADD__ ? { t: window.__MEM_ADD__ } : null)()`, 6000, "记忆新增");
    console.log(`  记忆新增：后端收到 text=${memAdd.t}`);
    if (memAdd.t !== "门记忆C") bad.push(`neobot_memory_add 参数不对：${JSON.stringify(memAdd.t)}`);

    await evaluate(send, `(() => {
      const btns = [...document.querySelectorAll('[data-testid="neobot-root"] aside button')];
      const b = btns.find(x => (x.textContent || '').includes('撤一版'));
      if (!b) throw new Error('NO_UNDO_BTN');
      b.click();
      return true;
    })()`);
    const memUndo = await waitFor(send, `(() => window.__MEM_UNDO__ ? { n: window.__MEM_UNDO__ } : null)()`, 6000, "记忆撤销");
    console.log(`  记忆撤销：调用 ${memUndo.n} 次`);

    // ── 新建对话：展开 → 填标题/成员 → 建 ──
    // ⛔ 这条必须在门里跑：这些调用只存在于默认收起的表单里，
    //    「没跑过」与「跑通了」在门眼里长得一模一样。
    await evaluate(send, `(() => {
      const b = [...document.querySelectorAll('[data-testid="neobot-root"] aside button')]
        .find(x => x.getAttribute('title') === '新建对话');
      if (!b) throw new Error('NO_NEW_CONVO_BTN');
      b.click();
      return true;
    })()`);
    await waitFor(send, `(() => document.querySelector('[data-testid="neobot-root"] aside input[aria-label="会话标题"]') ? true : null)()`, 6000, "新建会话表单展开");
    for (const [label, value] of [["会话标题", "门新建"], ["成员 id", "neo"]]) {
      await evaluate(send, `(() => {
        const el = document.querySelector('[data-testid="neobot-root"] aside input[aria-label="${label}"]');
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
        setter.call(el, ${JSON.stringify(value)});
        el.dispatchEvent(new Event('input', { bubbles: true }));
        return true;
      })()`);
      await new Promise((r) => setTimeout(r, 250));
    }
    await evaluate(send, `(() => {
      const b = [...document.querySelectorAll('[data-testid="neobot-root"] aside button')]
        .find(x => (x.textContent || '').trim() === '建会话');
      if (!b) throw new Error('NO_CREATE_BTN');
      b.click();
      return true;
    })()`);
    const created = await waitFor(send, `(() => {
      const m = window.__MEMBER_ADD__;
      if (!m) return null;
      const inputs = [...document.querySelectorAll('[data-testid="neobot-root"] aside input')];
      return { member: m, titleCleared: !inputs.some(i => i.value === '门新建') };
    })()`, 8000, "新建会话提交");
    console.log(`  新建会话：登记成员=${created.member}，提交后标题已清空=${created.titleCleared}`);
    if (created.member !== "neo") bad.push(`neobot_member_add 收到 ${JSON.stringify(created.member)}（应为 neo）`);

    const shotAfter = await send("Page.captureScreenshot", { format: "png" });
    await import("node:fs").then((fs) => fs.writeFileSync("/tmp/nb-main-after-create.png", Buffer.from(shotAfter.data, "base64")));
    console.log("  新建后截图 → /tmp/nb-main-after-create.png");

    const mainErrors = await evaluate(send, `(() => window.__ERRORS__)()`);
    for (const e of mainErrors) bad.push(`主页 JS 异常：${e}`);

    // ── B. 宠物页渲染 ──
    // 组件把精灵图画进 div 背景（非 canvas/img/video），探针查 `.dsh-pet__sprite`
    // 的背景图与盒尺寸 —— 查错标签会冤枉一个正常渲染（实测教训）。
    assertValidJS(PET_STUB, "PET_STUB");
    await send("Page.addScriptToEvaluateOnNewDocument", { source: PET_STUB });
    await send("Page.navigate", { url: `http://127.0.0.1:${PORT}/pet.html` });
    const pet = await waitFor(send, `(() => {
      // 2026-10-07 修：改前是 .dsh-pet__sprite（**上游 DSH 的类名**）。
      //   交付树（neobot-ui/）的宠物组件用的是自家命名 nb-pet-sprite
      //   （见 src/pet/pet.tsx 的 className 与 src/pet/pet.css）。
      //   ⛔ 于是 sprite 恒为 null -> 门报「等不到宠物精灵渲染」。
      //   ⛔ 这是同一族的第 3 个变体：门停下之后，**路径**没改（前两处）、
      //      **命令名**没改（再前两处）、**类名**没改（此处）——
      //      每一次都表现为「门说界面坏了」，而真凶在门自己身上。
      //      ⇒ 教训合并成一句：门停下就会腐化，复活它之前先校它的**名字**。
      const sprite = document.querySelector('.nb-pet-sprite');
      if (!sprite) return null;
      const r = sprite.getBoundingClientRect();
      if (r.width < 2 || r.height < 2) return null;
      const bg = window.getComputedStyle(sprite).backgroundImage || '';
      if (!bg.includes('data:image/')) return null;
      const hint = document.body.innerText.slice(0, 120);
      return { box: Math.round(r.width) + 'x' + Math.round(r.height), bgLen: bg.length, hint, errors: window.__ERRORS__ };
    })()`, 15000, "宠物精灵渲染");
    console.log(`  宠物渲染：精灵盒 ${pet.box} 背景图 ${pet.bgLen} 字符` +
      (pet.hint ? ` 提示文案=${pet.hint.slice(0, 40)}` : "（无提示文案 = 无错误态）"));
    for (const e of pet.errors) bad.push(`宠物页 JS 异常：${e}`);
    const shot2 = await send("Page.captureScreenshot", { format: "png" });
    await import("node:fs").then((fs) => fs.writeFileSync("/tmp/nb-pet.png", Buffer.from(shot2.data, "base64")));
    console.log("  宠物截图 → /tmp/nb-pet.png（1×1 静帧像不像，人目检）");

    if (bad.length) {
      console.error(`\n交互门 FAIL（${bad.length} 项）:`);
      for (const b of bad) console.error("  · " + b);
      failed = 1;
    } else {
      console.log("\n交互门 PASS（发送链参数正确、切会话换历史、宠物媒体已布局、两页零异常）");
      failed = 0;
    }
    ws.close();
  } catch (e) {
    try { ws.close(); } catch {}
    throw e;
  }
} catch (e) {
  console.error("交互门 FAIL: 环境问题（不是交互问题）—— " + (e instanceof Error ? e.message : String(e)));
  failed = 1;
} finally {
  try { server.close(); } catch {}
  if (child) child.kill("SIGKILL");
  try { rmSync(profile, { recursive: true, force: true }); } catch {}
}
process.exit(failed);
