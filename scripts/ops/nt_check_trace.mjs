#!/usr/bin/env node
/**
 * 轨迹门 —— 在 **生产包**（`neobot-ui/dist`）里真点「轨迹」页签，逐条断言。
 *
 * # 为什么要有这道门（2026-10-07 新增）
 *
 * 「轨迹页」是本轮补的**第二大缺口**：`neobot_send` 的返回里带着
 * `AgentRunResult.trace`，而界面此前只取 `output` ⇒ **整段被丢掉**；
 * 同时 `tasks`/`steps`/`file_changes` 三张表**没有任何读口**把它们端到端取出来
 * ⇒ 「跑过什么」完全看不见（这是本仓第 N 次「导出 ≠ 接入」，R-P79）。
 *
 * 补完之后，若只有类型检查与 cargo 测试，**界面上的六件事仍然没人验**：
 *   ① 三态（还在读 / 读不到 / 真的没有）是否被说成三句不同的话；
 *   ② 轮次行是否渲染、状态是否**原样**显示（库侧不解析，前端也不映射）；
 *   ③ 展开是否真去调 `neobot_run_trace` 并画出每一步 + 改了哪些文件；
 *   ④ 长输出是否折叠成「还有 N 行」且可展开；
 *   ⑤ ⭐ **轨迹页是否真的没有用量数字**（库里每轮没有可信费用可摊，
 *      一个会话级数字摆在逐轮行旁边会诱导用户拿它去除）；
 *   ⑥ 跑轮信号是否真的换掉了「正在想…」（`started` 应显示「开始跑…」）。
 *
 * ⛔ **不复用 `nt_check_layout` / `nt_check_interact` 的断言位置**：那两道门
 *    刚被复活（它们此前指向**冻结**的 `frontend/dist`，`.gitignore` 第 334 行
 *    显式忽略 ⇒ CI 永远构建不出来 ⇒ 从来没跑过）。复活它们时实测发现它们的
 *    路径、命令名、CSS 类名、键名**全部**停在旧形态 —— 这道门因此**自己持有**
 *    自己的选择器与桩，不与任何别的门共享字符串，避免再被「同一处腐化」牵连。
 *
 * # 判据纪律
 *
 * · 全部断言都基于**稳定 testid**（`nb-tab-trace` / `nb-trace-run` …）或
 *   **量 computed style**，⛔ 不用「四层后代链」那种 DOM 快照式选择器
 *   （`nt_check_interact` 那处就是这么坏的，见它的注释）。
 * · **零 JS 异常**是总判据之一；任何 `waitFor` 等不到都报「环境问题」并
 *   附现场（调用序列 + body 文本 + 异常），⛔ 不把「没等到」说成「界面坏」。
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
neobotDistFresh('nt_check_trace');
const W = Number(process.env["NB_W"] || 1280);
const H = Number(process.env["NB_H"] || 840);
const PORT = 9351;
const CDP_PORT = 9352;

if (!existsSync(join(DIST, "index.html"))) {
  console.error("轨迹门 FAIL: 产物不存在 —— 先跑 cd apps/neobot-desktop/neobot-ui && ./node_modules/.bin/vite build");
  process.exit(1);
}

const bad = [];

// ── stub-boot ────────────────────────────────────────────────────
// ⛔ 本段在**模板字符串**里 ⇒ 注释中不得出现反引号或美元加大括号
//    （STATUS §9 记过的坑：node 报 Unexpected identifier，bash -n 与 tsc 都抓不到）。
const STUB = `(() => {
  window.__CALLS__ = [];
  window.__ERRORS__ = [];
  window.__FAIL__ = {};
  window.addEventListener('error', (e) => window.__ERRORS__.push(String(e.message).slice(0, 160)));
  window.addEventListener('unhandledrejection', (e) => window.__ERRORS__.push('unhandled:' + String((e.reason && e.reason.message) || e.reason).slice(0, 160)));
  let cbId = 500;
  // ⭐ 失败开关由门在导航前设置（window.__NB_FAIL__）—— 用来分别验
  //   「读不到」与「真的没有」两种处境。若只有一个固定桩，就永远验不到其中一种。
  const convos = Array.from({ length: 3 }, (_, i) => ({ id: 'c' + (i + 1), kind: 'group', title: '会话' + (i + 1), members: ['neo'], task_count: i, last_active: new Date().toISOString(), muted: false, unread: 0 }));
  const RUNS = [
    { id: 't1', title: '第一轮', status: 'done', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), error: null, steps: 12, failed_steps: 0 },
    { id: 't2', title: '第二轮', status: 'failed', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), error: 'boom', steps: 5, failed_steps: 2 },
    { id: 't3', title: '第三轮（库里没有这个状态）', status: 'quantum', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), error: null, steps: 0, failed_steps: 0 },
  ];
  const STEP_OUT = Array.from({ length: 14 }, (_, k) => '第 ' + (k + 1) + ' 行输出').join('\\n');
  const map = {
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
    neobot_usage_summary: { days: 1, input: 0, cached: 0, written: 0, output: 0, requests: 0, tokens: 4242, rows: [] },
    neobot_memory_list: { lines: [], bytes: 0, cap: 8192, revisions: 0 },
  };
  // 2026-10-07：桩要能**真的派发**事件，否则「跑轮信号换了文案」这条断言
  //   根本验不到（无 Rust 时没有任何东西会调那个 callback）。
  //   做法照 @tauri-apps/api v2 的真实路径：listen 经 invoke 把
  //   transformCallback(handler) 得到的**数字 id** 交给后端，后端回调时按 id 取函数。
  //   ⇒ 这里记录 (event -> handlerId)，再用 __EMIT__ 按 id 取回并调用。
  const CB = new Map();
  const LISTENS = {};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    registerListener(ev) { delete LISTENS[ev]; return Promise.resolve(); },
    unregisterListener(ev) { delete LISTENS[ev]; return Promise.resolve(); },
  };
  window.__EMIT__ = (ev, payload) => {
    const id = LISTENS[ev];
    if (id === undefined) return false;
    const fn = CB.get(id);
    if (!fn) return false;
    fn({ event: ev, id: id, payload: payload });
    return true;
  };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main', windowLabel: 'main' } },
    transformCallback: (cb) => { const id = ++cbId; CB.set(id, cb); return id; },
    unregisterCallback: () => {},
    convertFileSrc: (p) => 'asset://localhost/' + p,
    invoke: async (cmd, args) => {
      window.__CALLS__.push(cmd);
      if (window.__NB_FAIL__ && window.__NB_FAIL__[cmd]) {
        throw new Error(window.__NB_FAIL__[cmd]);
      }
      if (cmd === 'neobot_convo_messages_page') {
        const cid = (args && args.convoId) || 'c1';
        const messages = [{ id: cid + '-m0', seq: 1, convo_id: cid, role: 'user', text: '问', created_at: new Date().toISOString() }];
        return { messages: messages, hasMore: false, nextSeq: null };
      }
      if (cmd === 'neobot_run_list') {
        // ⭐ 失败开关优先于空列表：这样「读不到」与「真的没有」能被分别验到。
        if (window.__NB_RUNS_EMPTY__) return { runs: [] };
        return { runs: RUNS };
      }
      if (cmd === 'neobot_run_trace') {
        const id = (args && args.taskId) || 't1';
        return {
          run: RUNS.find((r) => r.id === id) || RUNS[0],
          steps: [
            { id: 1, n: 0, tool: 'bash', ok: true, output: '短输出', tool_call_id: 'c1' },
            { id: 2, n: 1, tool: 'read', ok: true, output: STEP_OUT, tool_call_id: null },
            { id: 3, n: 2, tool: 'web_fetch', ok: false, output: '失败了', tool_call_id: null },
            { id: 4, n: 3, tool: 'reply', ok: true, output: '答复正文', tool_call_id: null },
          ],
          changes: [
            { id: 'ch1', at: new Date().toISOString(), path: 'src/lib.rs', kind: 'edit', bytes: 128, content_omitted: false },
            { id: 'ch2', at: new Date().toISOString(), path: 'vendor/huge.bin', kind: 'write', bytes: 999999, content_omitted: true },
          ],
        };
      }
      if (cmd === 'neobot_send') {
        // 必须**慢**：改前这个桩同步 resolve ⇒ busy 只存在一个 tick，
        // 轮询（200ms 一跳）**必然错过** ⇒ 门报「没有出现正在想…」。
        // 而那不是产品缺陷，是**桩太快**。用一个可观的时长把瞬态撑开。
        // 注意：这段注释在模板字符串里，不能出现反引号。
        await new Promise((r) => setTimeout(r, 2500));
        return { status: 'ok', output: 'STUB-REPLY', trace: [{ kind: 'plan', detail: '先读再写' }], model_used: 'stub', mode: 'passthrough', tools: [], usage: null };
      }
      if (cmd === 'plugin:event|listen') {
        LISTENS[args && args.event] = args && args.handler;
        return 1;
      }
      if (cmd === 'plugin:event|unlisten') {
        if (LISTENS[args && args.event] === (args && args.eventId)) delete LISTENS[args && args.event];
        return 1;
      }
      if (cmd === 'plugin:store|load') throw new Error('UNMOCKED:plugin:store|load');
      if (typeof cmd === 'string' && cmd.startsWith('plugin:')) return null;
      if (cmd in map) return map[cmd];
      throw new Error('UNMOCKED:' + cmd);
    },
  };
})()`;

/** 点页签进轨迹页并等它渲染完。 */
const GO_TRACE = `(() => {
  const tab = document.querySelector('[data-testid="nb-tab-trace"]');
  if (!tab) return { err: 'NO_TAB' };
  tab.click();
  return { ok: true };
})()`;

/** 轨迹页快照（只读稳定 testid 与量 computed style）。 */
const PROBE_TRACE = `(() => {
  const root = document.querySelector('[data-testid="neobot-root"]');
  if (!root) return null;
  const pane = document.querySelector('[data-testid="nb-trace"]');
  const runs = [...document.querySelectorAll('[data-testid="nb-trace-run"]')];
  const skeleton = document.querySelector('[data-testid="nb-trace-skeleton"]');
  const rows = runs.map((b) => {
    const spans = [...b.querySelectorAll('span')].map((s) => (s.textContent || '').trim());
    const statusSpan = b.querySelector('[title]');
    return {
      text: (b.textContent || '').trim(),
      expanded: b.getAttribute('aria-expanded') === 'true',
      // ⭐ 失败步数那一格用危险色：量出来而不是看 class（class 名会变）。
      danger: [...b.querySelectorAll('span')].some((s) => {
        const c = window.getComputedStyle(s).color;
        return c === 'rgb(163, 39, 43)';
      }),
      status: statusSpan ? (statusSpan.getAttribute('title') || '') : '',
    };
  });
  const paneText = pane ? (pane.textContent || '') : '';
  // ⭐ 2026-10-07 补：**整棵根**的文本，与面板文本**分开**报。
  //   ⛔⛔ 这是被变异测试逼出来的：改前只报 paneText，而那个 token 数字
  //   实际渲染在**头栏**（那段 view==='chat' 的条件在 header 里，不在面板里）
  //   ⇒ 我把「轨迹页出现用量数字」写成面板内断言 ⇒ 变异「头栏两个页签都显示」
  //   **全绿通过**。门犯的错和它要抓的错是同一种：**量错了作用域**。
  //   ⇒ 不变量说的是「这一页**任何位置**都没有用量数字」，量就必须覆盖整页。
  const rootText = (root.textContent || '');
  return {
    ok: !!pane,
    hasPane: !!pane,
    skeleton: !!skeleton,
    rows: rows,
    // ⭐ 轨迹页**不得**出现任何 token/用量数字（库里每轮没有可信费用可摊）。
    paneText: paneText,
    rootText: rootText,
    errors: window.__ERRORS__,
  };
})()`;

/** 展开某一轮并等详情。 */
const OPEN_RUN = `(() => {
  const b = [...document.querySelectorAll('[data-testid="nb-trace-run"]')]
    .find((x) => (x.textContent || '').includes('第二轮'));
  if (!b) return { err: 'NO_RUN2' };
  b.click();
  return { ok: true };
})()`;

const PROBE_DETAIL = `(() => {
  const pane = document.querySelector('[data-testid="nb-trace"]');
  if (!pane) return null;
  const txt = pane.textContent || '';
  // 「还有 N 行」按钮：折叠态才有（长输出折叠）。
  const moreBtn = [...pane.querySelectorAll('button')].find((x) => /^还有 \\d+ 行$/.test((x.textContent || '').trim()));
  return {
    ok: !!pane,
    text: txt,
    hasStepsHeading: /步/.test(txt),
    hasChangeHeading: /改动的文件/.test(txt),
    // ⭐ 14 行输出被折叠 ⇒ 界面上不应出现最后一行。
    collapsedHidesTail: !/第 14 行输出/.test(txt) && !!moreBtn,
    moreLabel: moreBtn ? (moreBtn.textContent || '').trim() : '',
    errors: window.__ERRORS__,
  };
})()`;

const EXPAND_ALL = `(() => {
  const pane = document.querySelector('[data-testid="nb-trace"]');
  if (!pane) return { err: 'NO_PANE' };
  const btn = [...pane.querySelectorAll('button')].find((x) => /^还有 \\d+ 行$/.test((x.textContent || '').trim()));
  if (!btn) return { err: 'NO_MORE_BTN' };
  btn.click();
  return { ok: true };
})()`;

const PROBE_EXPANDED = `(() => {
  const pane = document.querySelector('[data-testid="nb-trace"]');
  if (!pane) return null;
  const txt = pane.textContent || '';
  return { showsTail: /第 14 行输出/.test(txt), hasCollapseBtn: /收起/.test(txt), errors: window.__ERRORS__ };
})()`;

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

function findChrome() {
  const cands = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  ];
  for (const c of cands) if (existsSync(c)) return c;
  console.error("轨迹门 FAIL: 找不到 Chrome/Chromium/Edge");
  process.exit(1);
  return "";
}

/** 等探针满足条件；等不到抛「等不到」并附现场（⛔ 不说成「界面坏」）。 */
async function waitFor(send, expr, what, tries = 30) {
  let last = null;
  for (let i = 0; i < tries; i++) {
    await new Promise((r) => setTimeout(r, 300));
    const out = await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true });
    if (out.exceptionDetails) throw new Error("探针异常：" + (out.exceptionDetails.text || "?"));
    last = out.result.value;
    if (last && last.ok) return last;
    if (last && last.err) throw new Error(`${what}：桩/选择器没找到（${last.err}）`);
  }
  const scene = await send("Runtime.evaluate", {
    expression: `({ calls: window.__CALLS__, errors: window.__ERRORS__, body: (document.body.innerText || '').slice(0, 400) })`,
    returnByValue: true,
  });
  throw new Error(`等不到：${what}；现场=${JSON.stringify(scene.result.value)}`);
}

const chrome = findChrome();
const profile = mkdtempSync(join(tmpdir(), "nb-trace-"));
let child = null;
let failed = 2;

try {
  await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
  child = spawn(chrome, [
    "--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run",
    "--disable-extensions", "--disable-background-networking",
    `--user-data-dir=${profile}`,
    `--window-size=${W},${H}`,
    `--remote-debugging-port=${CDP_PORT}`,
    "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  child.on("error", () => {});
  await new Promise((r) => setTimeout(r, 1500));

  const targets = await cdpGet(`http://127.0.0.1:${CDP_PORT}/json`);
  const page = targets.find((t) => t.type === "page") || targets[0];
  const { ws, send } = await connect(page.webSocketDebuggerUrl);
  try {
    await send("Page.enable");
    await send("Runtime.enable");
    await send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
    await send("Page.addScriptToEvaluateOnNewDocument", { source: STUB });
    await send("Page.navigate", { url: `http://127.0.0.1:${PORT}/` });

    // 等自持根出现。
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 300));
      const out = await send("Runtime.evaluate", {
        expression: `!!document.querySelector('[data-testid="nb-tab-trace"]')`,
        returnByValue: true,
      });
      if (out.result.value) break;
    }

    // ── ① 页签存在且可进 ─────────────────────────────────────────
    await waitFor(send, GO_TRACE, "进轨迹页签");
    let snap = null;
    snap = await waitFor(send, PROBE_TRACE, "轨迹页渲染");
    if (!snap) throw new Error("轨迹页探针返回空");
    console.log(`  轨迹页：${snap.rows.length} 条轮次 · 骨架=${snap.skeleton}`);

    // ② 三轮都渲染，且状态**原样**（含库里那个界面不认识的 quantum）。
    if (snap.rows.length !== 3) bad.push(`轨迹列表应有 3 行，实际 ${snap.rows.length}`);
    const quantum = snap.rows.find((r) => r.text.includes("quantum"));
    if (!quantum) {
      bad.push(
        "库里那个「界面不认识的状态(quantum)」必须**原样显示** ⇒ 界面把它映射成已知状态了，" +
          "那会把「新的」说成「没有」（nt_run_trace 不变量 1）",
      );
    }
    // 失败轮才用危险色；成功轮不得带危险色。
    const failedRow = snap.rows.find((r) => r.text.includes("第二轮"));
    const okRow = snap.rows.find((r) => r.text.includes("第一轮"));
    if (!failedRow || !failedRow.danger) bad.push("失败步数 > 0 的那一行应当用危险色标出");
    if (okRow && okRow.danger) bad.push("0 个失败步的行不该出现危险色（那是纯噪声）");

    // ③ ⭐ 轨迹页**不得**出现用量数字。
    if (/tokens|\b4242\b/.test(snap.rootText)) {
      bad.push(
        `轨迹页出现了用量数字：${(snap.rootText.match(/.{0,20}(tokens|4242).{0,20}/) || ['?'])[0]} ` +
          "⇒ 库里每轮没有可信费用可摊，一个会话级数字摆在逐轮行旁边会诱导用户拿它去除",
      );
    }
    // 但**必须**有一句说明为什么没有（需要解释的缺席）。
    if (!/按天/.test(snap.paneText)) bad.push("轨迹页没有解释「为什么这里没有每轮费用」");

    // ── ④ 展开第二轮 → 步与改动 ─────────────────────────────────
    await waitFor(send, OPEN_RUN, "展开第二轮");
    const detVal = await waitFor(send, PROBE_DETAIL, "第二轮详情");
    console.log(`  详情：步标题=${detVal.hasStepsHeading} 改动标题=${detVal.hasChangeHeading} 折叠=${detVal.collapsedHidesTail}`);
    if (!detVal.hasStepsHeading) bad.push("展开后没有步骤区");
    if (!detVal.hasChangeHeading) bad.push("展开后没有「改动的文件」区");
    if (!detVal.collapsedHidesTail) {
      bad.push(
        `长输出（14 行）没有按行折叠成「还有 N 行」（实测 ${detVal.moreLabel || "没有按钮"}）` +
          " ⇒ 折叠按行数不按字符：字符截断丢掉的是尾部且「多大」这个信息也丢了",
      );
    }
    // ⭐⭐ `content_omitted` 必须**被显示**。
    //
    // ⛔ 这条是 2026-10-07 由 `check-dead-config-flag` 反向发现的：
    //   store 段头写明「UI 据此说『内容已略去』」，而界面把它一路传到
    //   TS 接口后**从不渲染** ⇒ 只写不读 ⇒ 门报「新增死开关」。
    //   ⇒ 与本轮开头修的 `AgentRunResult.trace` **同型**：数据到了最后一格被丢。
    //   门抓的是真缺陷；本断言让它**不能再退化**（桩里第 2 条就是它）。
    if (!/略去|omitted/.test(detVal.text)) {
      bad.push(
        "展开后看不到「内容已略去」的标记 ⇐ `content_omitted` 传到了界面却没渲染。" +
          "（这条曾被 check-dead-config-flag 抓成新增死开关；store 的意图就是让 UI 说这句）",
      );
    }

    // 失败步必须有可见的失败标记。
    if (!/失败/.test(detVal.text)) bad.push("展开后看不到哪一步失败了（只有 ok=true 才不算表达出来）");

    // ⑤ 展开全文
    await waitFor(send, EXPAND_ALL, "展开全文");
    let exp = null;
    for (let i = 0; i < 12; i++) {
      await new Promise((r) => setTimeout(r, 200));
      exp = (await send("Runtime.evaluate", { expression: PROBE_EXPANDED, returnByValue: true })).result.value;
      if (exp && exp.showsTail) break;
    }
    if (!exp || !exp.showsTail) bad.push("点了「还有 N 行」之后长输出没有真正展开");
    if (exp && !exp.hasCollapseBtn) bad.push("展开后没有「收起」按钮（不可逆的展开同样是缺陷）");

    // ── ⑥ 三态：真的没有 ─────────────────────────────────────────
    await send("Runtime.evaluate", { expression: `window.__NB_RUNS_EMPTY__ = true` });
    await send("Runtime.evaluate", { expression: GO_TRACE });
    await new Promise((r) => setTimeout(r, 600));
    // 重新进页签才会重拉（轨迹是按需拉的）⇒ 先切回对话再切回来。
    await send("Runtime.evaluate", { expression: `document.querySelector('[data-testid="nb-tab-chat"]').click()` });
    await new Promise((r) => setTimeout(r, 200));
    await send("Runtime.evaluate", { expression: GO_TRACE });
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 250));
      const out = await send("Runtime.evaluate", { expression: PROBE_TRACE, returnByValue: true });
      const v = out.result.value;
      if (v && v.hasPane && v.rows.length === 0) break;
    }
    let emptySnap = (await send("Runtime.evaluate", { expression: PROBE_TRACE, returnByValue: true })).result.value;
    if (emptySnap.rows.length !== 0) {
      bad.push(`把桩切成「真的没有」之后仍有 ${emptySnap.rows.length} 行 ⇒ 空态没生效`);
    } else if (/读不到|读取轨迹失败/.test(emptySnap.paneText)) {
      bad.push("空态说成了「读不到」⇒ 两种处境被混成一句（这是本仓反复吃过的坑）");
    } else if (!/还没有跑过/.test(emptySnap.paneText)) {
      bad.push(`空态文案没说清「没跑过」，实际：${emptySnap.paneText.slice(0, 120)}`);
    }

    // ── ⑦ 三态：读不到 ───────────────────────────────────────────
    await send("Runtime.evaluate", {
      expression: `window.__NB_RUNS_EMPTY__ = false; window.__NB_FAIL__ = { neobot_run_list: 'gate: 故意让读失败' };`,
    });
    await send("Runtime.evaluate", { expression: `document.querySelector('[data-testid="nb-tab-chat"]').click()` });
    await new Promise((r) => setTimeout(r, 200));
    await send("Runtime.evaluate", { expression: GO_TRACE });
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 250));
      const out = await send("Runtime.evaluate", { expression: PROBE_TRACE, returnByValue: true });
      const v = out.result.value;
      if (v && v.hasPane && /读不到|读取轨迹失败/.test(v.paneText)) break;
    }
    let failSnap = (await send("Runtime.evaluate", { expression: PROBE_TRACE, returnByValue: true })).result.value;
    if (!/读取轨迹失败/.test(failSnap.paneText)) {
      bad.push(`读失败时应当说「读不到」并给重读，实际：${failSnap.paneText.slice(0, 140)}`);
    }
    if (!/重读/.test(failSnap.paneText)) bad.push("读失败时没有给「重读」按钮（不可恢复的失败态等于没有错误提示）");
    await send("Runtime.evaluate", { expression: `window.__NB_FAIL__ = {}` });

    // ── ⑧ 跑轮信号：先把「正在想…」换成「开始跑…」 ─────────────
    // ⛔ 必须先回到**对话页**：第 ⑦ 步结束时屏幕停在轨迹页（且还挂着「读不到」态），
    //   而「正在想…」那个气泡只在消息流里 ⇒ 在轨迹页上验它等于验错对象
    //   （这与 STATUS §7 记的「门报错对但指错对象」是同一条纪律）。
    await send("Runtime.evaluate", {
      expression: `document.querySelector('[data-testid="nb-tab-chat"]').click()`,
      returnByValue: true,
    });
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 200));
      const out = await send("Runtime.evaluate", {
        expression: `!document.querySelector('[data-testid="nb-trace"]')`,
        returnByValue: true,
      });
      if (out.result.value) break;
    }
    // ⛔ 先真发一条：改前的第 ⑧ 步**没有发送**就去查文案 ⇒ 那是在断言
    //   一个从未发生过的状态。门里的这种断言要么恒真、要么恒假，
    //   两种都提供不了信息（STATUS §4 教训 7：空洞的测试）。
    await send("Runtime.evaluate", {
      expression: `(() => {
        const ta = document.querySelector('[data-testid="neobot-root"] textarea');
        if (!ta) return { err: 'NO_TEXTAREA' };
        const setter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value').set;
        setter.call(ta, '门内自测消息');
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        return { ok: true };
      })()`,
      returnByValue: true,
    });
    await send("Runtime.evaluate", {
      expression: `[...document.querySelectorAll('[data-testid="neobot-root"] button')]
        .find((b) => (b.textContent || '').trim() === '发送').click()`,
      returnByValue: true,
    });
    let preSignal = null;
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 200));
      const out = await send("Runtime.evaluate", {
        expression: `({ text: (document.querySelector('[data-testid="neobot-root"]')||{}).innerText || '' })`,
        returnByValue: true,
      });
      preSignal = out.result.value;
      if (/正在想/.test(preSignal.text)) break;
    }
    if (!preSignal || !/正在想/.test(preSignal.text)) {
      bad.push("发送后没有出现「正在想…」（本地 await 未回来时也该有运行态提示）");
    }

    // ⭐ 派发**真实**的 started 信号：文案必须从「正在想…」变成「开始跑…」。
    //   ⛔ 这一步验的是「信号真被接上并改了渲染」，不是「有个 listen 调用」——
    //   后者只证明代码跑到了那一行。
    const emitted = (await send("Runtime.evaluate", {
      expression: `window.__EMIT__('neobot:run', { phase: 'started', convo_id: 'c1', elapsed_ms: 0, trace: [], error: null })`,
      returnByValue: true,
    })).result.value;
    if (emitted !== true) bad.push("桩没能派发 neobot:run（说明界面的 listen 没注册成功）");
    await new Promise((r) => setTimeout(r, 300));
    const postSignal = (await send("Runtime.evaluate", {
      expression: `({ text: (document.querySelector('[data-testid="neobot-root"]')||{}).innerText || '' })`,
      returnByValue: true,
    })).result.value;
    if (!/开始跑/.test(postSignal.text)) {
      bad.push(
        `收到 started 信号后文案没有变成「开始跑…」（实际含：${postSignal.text.slice(-80)}）` +
          " ⇒ 信号没接到渲染，「正在想…」仍然是猜的而不是后端说的",
      );
    }

    // ── ⑨ 零 JS 异常（两页合计） ─────────────────────────────────
    const errs = (await send("Runtime.evaluate", { expression: `window.__ERRORS__`, returnByValue: true })).result.value;
    for (const e of errs || []) bad.push(`页面 JS 异常：${e}`);

    ws.close();
  } catch (e) {
    try { ws.close(); } catch { /* 关闭失败不影响判据 */ }
    throw e;
  }
} catch (e) {
  console.error("轨迹门 FAIL: 环境问题（不是轨迹页问题）—— " + (e instanceof Error ? e.message : String(e)));
  failed = 1;
} finally {
  try { server.close(); } catch { /* 忽略 */ }
  if (child) child.kill("SIGKILL");
  try { rmSync(profile, { recursive: true, force: true }); } catch { /* 忽略 */ }
}

if (failed === 2) {
  if (bad.length) {
    console.error(`\n轨迹门 FAIL（${bad.length} 项）:`);
    for (const b of bad) console.error("  · " + b);
    process.exit(1);
  }
  console.log("\n轨迹门 PASS（三态可辨、状态原样透传、步与改动可展开、长输出按行折叠、无每轮费用、零异常）");
  process.exit(0);
}
process.exit(failed);
