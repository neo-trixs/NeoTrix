#!/usr/bin/env node
// **视觉/布局**门 —— 专治「构建全绿、类型全对、界面却不能用」。
//
// ## 为什么必须单独一道
//
// 本仓此前的所有前端门（`nt_check_layout` / `nt_check_interact` / 各 smoke）
// 检查的是**结构**：元素在不在、testid 有没有、overflow 生不生效、异常是不是 0。
// 于是 2026-10-01 这个状态能全绿通过，而人眼看到的是：
//   · 页面下半屏全黑、侧栏右边框在半途断掉、输入框浮在半空
//     （`.nb-main` 少了 `flex: 1 1 auto` ⇒ `flex-grow` 停在初始值 **0**，
//       实测 shell 840px / main **213px**）；
//   · 顶栏右端一簇控件上下参差、被读成「挤在一起」；
//   · 会话项行高只有 34px，两行文字贴着上下边缘。
//
// 这三类都不是「结构错」，所以结构门看不见。本门用**可量的量**表达它们：
//
// ① **高度链**：`.nb-shell` / `.nb-main` / `[data-testid=neobot-root]` / `aside`
//    必须与视口同高（差 ≤ 2px）。这一条就足以抓住上面那个 flex-grow 缺陷。
// ② **控件等高**：顶栏内所有 button/select 的 `getBoundingClientRect().height`
//    极差必须 = 0。这是「挤在一起」的定义。
// ③ **基线对齐**：同簇控件的垂直中心极差 ≤ 1px。
// ④ **行高下限**：会话项高度 ≥ 40px（两行文字 + 呼吸）。
// ⑤ **正文列宽上限**：消息列 ≤ 880px（超宽屏上一行几百字没法读）。
// ⑥ **无横向溢出** + 输入框在视口内。
//
// 用法：`node scripts/ops/nt_check_visual.mjs`

import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { readFileSync, existsSync, mkdirSync } from "node:fs";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/neobot-ui/dist");
const PORT = Number(process.env.NB_PORT || 8799);
const W = Number(process.env.NB_W || 1280);
const H = Number(process.env.NB_H || 840);
const OUT = process.env.NB_OUT || "/tmp/nb-visual.png";

if (!existsSync(join(DIST, "index.html"))) {
  console.error("视觉门 FAIL: neobot-ui/dist 不存在 —— 先跑 pnpm --dir apps/neobot-desktop/neobot-ui run build");
  process.exit(1);
}

/**
 * 桩：覆盖 neobot-ui 会调的全部命令。
 *
 * ⛔ 用 `window.__TAURI_INTERNALS__`（Tauri v2 的真实形状），**不要**自造
 *    `window.__TAURI__` —— 那样 app 读不到 `invoke`，UI 渲染成错误态，
 *    而本门量的是**布局**，错误态的布局与正常态不是一回事。
 *    （我自己就是这么测错了一次：截图里满屏 TypeError，却以为那是界面坏掉。）
 */
const STUB = `(() => {
  const convos = Array.from({ length: 9 }, (_, i) => ({
    id: 'c' + (i + 1), kind: 'group', title: '海豚调试会话 ' + (i + 1),
    members: ['neo', 'kai'], task_count: i,
    last_active: new Date(Date.now() - i * 3600e3).toISOString(),
    muted: i === 4, unread: i === 2 ? 3 : 0,
  }));
  const now = new Date().toISOString();
  const MAP = {
    log_frontend: null, set_language: null, read_run_logs: 'stub logs', quit_app: null,
    open_external_url: null, write_clipboard_text: null,
    get_pet_status: { enabled: false, visible: false, active_pet: '', pet_size: null },
    get_pet_asset: null, move_pet_window: null, set_pet_ignore_cursor_events: null,
    neobot_convo_list: convos,
    neobot_member_list: [{ id: 'neo', kind: 'agent', display: 'neo' }],
    neobot_core_capabilities: { crystal_version: 'v', tool_count: 6, model: 'qwen3-max', model_source: 'provider' },
    neobot_usage_summary: { days: 1, input: 12000, cached: 0, written: 0, output: 3400, requests: 9, tokens: 15400, rows: [] },
    neobot_memory_list: { lines: ['喜欢深色模式', '用 pnpm 不用 npm'], bytes: 42, cap: 8192, revisions: 3 },
    neobot_convo_group: 'c-new', neobot_member_add: null, neobot_memory_add: true, neobot_memory_undo: true,
    neobot_api_specs: { total: 118, implemented: 63, refused: 11, stub: 39, planned: 5, specs: [] },
    neobot_api_call: { ok: true, status: 'implemented', reason: '', data: null },
  };
  const HANDLE = (cmd, args) => {
    if (cmd === 'neobot_convo_messages') {
      return Array.from({ length: 6 }, (_, i) => ({
        id: 'm' + i, convo_id: args.convo_id,
        role: i % 3 === 2 ? 'assistant' : 'user',
        text: i % 3 === 2 ? '这是**第 ' + i + ' 条**回复' : '第 ' + i + ' 条提问',
        created_at: now,
      }));
    }
    if (cmd === 'neobot_send') {
      return { status: 'ok', output: '收到：' + String(args.text || '').slice(0, 10), trace: [], model_used: 'qwen3-max', mode: 'passthrough', tools: [], usage: null };
    }
    if (cmd && cmd.startsWith('plugin:')) return null;
    if (cmd in MAP) return MAP[cmd];
    throw new Error('UNMOCKED:' + cmd);
  };
  window.__ERRORS__ = [];
  window.addEventListener('error', (e) => window.__ERRORS__.push(String(e.message)));
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      if (cmd === 'plugin:event|listen') return Promise.resolve(1);
      if (cmd === 'plugin:event|unlisten') return Promise.resolve(null);
      try { return Promise.resolve(HANDLE(cmd, args)); }
      catch (e) { return Promise.reject(e); }
    },
    transformCallback: (cb) => { const id = Math.floor(Math.random() * 1e9); window['_' + id] = cb; return id; },
  };
})()`;

const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml", ".png": "image/png", ".json": "application/json" };
const server = createServer((req, res) => {
  const p = join(DIST, ((req.url || "/").split("?")[0] === "/" ? "/index.html" : (req.url || "").split("?")[0]));
  if (!existsSync(p)) { res.writeHead(404); res.end("nf"); return; }
  res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
  res.end(readFileSync(p));
});

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const get = (url) => new Promise((res, rej) => {
  import("node:http").then(({ get: g }) => g(url, (r) => { let b = ""; r.on("data", (c) => (b += c)); r.on("end", () => res(b)); }).on("error", rej));
});

let chrome;
let chromeProc;
const send = (method, params = {}, sessionId) =>
  new Promise((res, rej) => {
    const msg = { id: Math.floor(Math.random() * 1e9), method, params };
    if (sessionId) msg.sessionId = sessionId;
    // ⛔ Node 内置 WebSocket 只有 addEventListener，没有 ws 包的 `.on()`。
    const on = (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id !== msg.id) return;
      chrome.removeEventListener("message", on);
      if (m.error) rej(new Error(m.error.message || "CDP error"));
      else res(m.result);
    };
    chrome.addEventListener("message", on);
    chrome.send(JSON.stringify(msg));
    setTimeout(() => { chrome.removeEventListener("message", on); rej(new Error("CDP timeout " + method)); }, 20000);
  });

const bad = [];
try {
  await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
  // ⛔ mkdirSync 返回 undefined，不能拿它做 && 的左值（我第一版这么写，dir 直接是 undefined）。
  const dir = join(tmpdir(), `nb-chrome-${Date.now()}`);
  mkdirSync(dir, { recursive: true });
  // ⛔ 候选列表而不是写死 `google-chrome`：macOS 上可执行名通常是
  //    "Google Chrome"（含空格、在 .app 里），写死会得到「找不到文件」。
  const CANDIDATES = [
    process.env["CHROME_PATH"],
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
  ].filter(Boolean);
  const bin = CANDIDATES.find((p) => existsSync(p));
  if (!bin) throw new Error("找不到 Chrome/Edge。用 CHROME_PATH=<路径> 指定。");
  chromeProc = spawn(bin, [
    "--headless=new", `--remote-debugging-port=${PORT + 1}`, "--disable-gpu", "--no-first-run",
    `--user-data-dir=${dir}`, `--window-size=${W},${H}`, "about:blank",
  ], { stdio: "ignore" });

  let ver = null;
  for (let i = 0; i < 40 && !ver; i++) {
    await sleep(300);
    try { ver = JSON.parse(await get(`http://127.0.0.1:${PORT + 1}/json/version`)); } catch {}
  }
  if (!ver) throw new Error("Chrome 起不来");

  // ⛔ 用 Node 22+ 内置的全局 WebSocket，不引 ws 包 —— 门不该给仓库加依赖。
  chrome = new WebSocket(ver.webSocketDebuggerUrl);
  await new Promise((r, j) => { chrome.addEventListener("open", r, { once: true }); chrome.addEventListener("error", j, { once: true }); });

  const { targetId } = await send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
  await send("Page.enable", {}, sessionId);
  await send("Runtime.enable", {}, sessionId);
  await send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false }, sessionId);
  await send("Page.addScriptToEvaluateOnNewDocument", { source: STUB }, sessionId);
  await send("Page.navigate", { url: `http://127.0.0.1:${PORT}/` }, sessionId);

  // 等真实内容出现（而不是死等毫秒）
  let ready = false;
  for (let i = 0; i < 40 && !ready; i++) {
    await sleep(300);
    const r = await send("Runtime.evaluate", {
      returnByValue: true,
      expression: `(() => { const l = document.querySelector('[data-testid="nb-convo-list"]'); return !!(l && l.querySelectorAll('button').length >= 3); })()`,
    }, sessionId);
    ready = !!r.result.value;
  }
  if (!ready) { bad.push("渲染未就绪：会话列表没出现（先看是不是桩不完整）"); }

  const probe = await send("Runtime.evaluate", {
    returnByValue: true,
    expression: `(() => {
      const q = (s) => document.querySelector(s);
      const h = (el) => el ? Math.round(el.getBoundingClientRect().height) : null;
      const shell = q('.nb-shell'), main = q('.nb-main');
      const root = q('[data-testid="neobot-root"]'), aside = q('aside');
      const bar = q('.nb-bar');
      const ctrls = [...document.querySelectorAll('.nb-bar button, .nb-bar select')]
        .filter(el => el.offsetParent !== null)
        .map(el => { const r = el.getBoundingClientRect(); return { h: Math.round(r.height), cy: Math.round(r.top + r.height / 2) }; });
      // ⛔⛔ 原来量「nb-convo-list 下所有 button」。侧栏新增「按最近活跃分组」的
      //   **组头**（py-1 text-[11px]，约 25px）后，组头被当成「会话项」
      //   ⇒ 本门报「行高 25px < 40px」。
      //   **门没错在阈值，是量错了对象** —— 与本文件头 §7.2 同一纪律，
      //   只不过这次是**另一侧加功能**暴露了选择器的位置假设。
      // ⇒ 改量会话项专属钩子；钩子不存在时回退旧选择器（避免选择器失效 ⇒ 恒 0 判过）。
      // ⚠️ 本段在**模板字符串**内 ⇒ 注释里**绝不可用反引号**，
      //    否则提前终止模板 ⇒ SyntaxError（本轮踩过）。用「」代替。
      const nbItems = document.querySelectorAll('[data-testid="nb-convo-item"]')
      const rowSel = nbItems.length ? '[data-testid="nb-convo-item"]' : '[data-testid="nb-convo-list"] button'
      const rows = [...document.querySelectorAll(rowSel)]
        .map(el => Math.round(el.getBoundingClientRect().height));
      // ⛔ 上一版量「section 下所有 div 的宽度」，量到的是**滚动容器本身**
      //    （1024px 全宽）—— 那不是正文列。真正决定可读性的是居中那列
      //    （Tailwind 的 mx-auto），量错对象就会给假警或漏警。
      const col = [...document.querySelectorAll('[data-testid="neobot-root"] section div.mx-auto')]
        .map(el => Math.round(el.getBoundingClientRect().width))
        .filter(w => w > 0);
      const ta = q('textarea');
      const tr = ta ? ta.getBoundingClientRect() : null;
      return JSON.stringify({
        viewport: window.innerHeight,
        shell: h(shell), main: h(main), root: h(root), aside: h(aside), bar: h(bar),
        mainGrow: main ? getComputedStyle(main).flexGrow : null,
        ctrls, rows,
        maxCol: col.length ? Math.round(Math.max(...col)) : null,
        composerInView: tr ? tr.bottom <= window.innerHeight + 1 && tr.top >= 0 : null,
        hOverflow: document.documentElement.scrollWidth > window.innerWidth + 1,
        // 消息分组：连续同作者应聚合 ⇒ 间距必须**不等**（组内小、组间大）。
        gaps: (() => {
          // ⛔ 必须限定在消息列内：data-index 是**两个虚拟化列表共用**的属性
          //    （会话列表 + 消息列表），不限定就会把两列的 top 混在一起排序，
          //    算出「负间距」（实测 -53/-61…）—— 门于是对着垃圾数据判 PASS。
          //    这与之前「量错对象」是同一类错：根/侧栏、列宽、[data-index] 各犯一次。
          // ⛔ 量**气泡**而不是 item 外框：虚拟化下 item 是 absolute 定位、
          //    首尾相接（vi.start 累加高度），所以外框之间的间距**恒为 0**，
          //    分组间距实际写在 item 自己的 paddingTop 里 ⇒ 量外框永远量不到。
          const items = [...document.querySelectorAll('[data-testid="nb-msg-sizer"] [data-index]')]
            .map(el => el.firstElementChild?.firstElementChild)
            .filter(Boolean)
            .map(el => el.getBoundingClientRect())
            .sort((a, b) => a.top - b.top);
          const out = [];
          for (let i = 1; i < items.length; i++) {
            out.push(Math.round(items[i].top - items[i - 1].bottom));
          }
          return out;
        })(),
        errors: window.__ERRORS__ || [],
      });
    })()`,
  }, sessionId);
  const d = JSON.parse(probe.result.value);

  // ① 高度链：差 ≤ 2px（.nb-main 要减掉顶栏高度，故与 shell 比用「接近视口」判）
  // ⛔ 根与侧栏在**顶栏之下**，所以判据是「≈ 视口 − 顶栏」，不是「≈ 视口」。
  //    我第一版写成后者，于是门把自己刚修好的布局判成 FAIL —— 门的第一版
  //    断言和它要抓的 bug 一样，都属于「没看清事实就下结论」。
  const body = d.viewport - (d.bar ?? 0);
  const near = (v) => v != null && Math.abs(v - body) <= 2;
  if (!near(d.root)) bad.push(`对话根高度 ${d.root} ≠ 视口−顶栏 ${body}（高度链断了）`);
  if (!near(d.aside)) bad.push(`侧栏高度 ${d.aside} ≠ 视口−顶栏 ${body}（侧栏没被拉伸）`);
  if (d.mainGrow === "0") bad.push(".nb-main 的 flex-grow=0 ⇒ 主区停在内容高度（本门就是为这条而写）");

  // ② 控件等高
  if (d.ctrls.length >= 2) {
    const hs = d.ctrls.map((c) => c.h);
    const spread = Math.max(...hs) - Math.min(...hs);
    if (spread !== 0) bad.push(`顶栏控件高度参差 ${spread}px：${hs.join("/")}（这就是「挤在一起」）`);
    const cys = d.ctrls.map((c) => c.cy);
    const cspread = Math.max(...cys) - Math.min(...cys);
    if (cspread > 1) bad.push(`顶栏控件中心线错位 ${cspread}px`);
  } else {
    bad.push(`顶栏只找到 ${d.ctrls.length} 个控件 —— 探针失效或界面真的空了`);
  }

  // ③ 会话行高
  if (d.rows.length) {
    const min = Math.min(...d.rows);
    if (min < 40) bad.push(`会话项行高 ${min}px < 40px（两行文字会贴边，读起来是「挤」）`);
  }

  // ④ 正文列宽上限
  if (d.maxCol && d.maxCol > 880) bad.push(`消息列宽 ${d.maxCol}px > 880px（超宽屏上一行几百字）`);

  // ⑤ 溢出与输入框
  if (d.hOverflow) bad.push("出现横向溢出");
  if (d.composerInView === false) bad.push("输入框不在视口内");
  for (const e of d.errors) bad.push(`JS 异常：${e}`);

  const shot = await send("Page.captureScreenshot", { format: "png" }, sessionId);
  const fs = await import("node:fs");
  fs.writeFileSync(OUT, Buffer.from(shot.data, "base64"));

  console.log(`  视口 ${d.viewport} · shell ${d.shell} · main ${d.main}(grow=${d.mainGrow}) · 根 ${d.root} · 侧栏 ${d.aside} · 顶栏 ${d.bar}`);
  console.log(`  顶栏控件 ${d.ctrls.length} 个，高度 ${[...new Set(d.ctrls.map((c) => c.h))].join("/")} · 会话行高 ${d.rows.length ? Math.min(...d.rows) + ".." + Math.max(...d.rows) : "n/a"} · 列宽 ${d.maxCol}`);
  if (d.gaps && d.gaps.length) console.log(`  消息间距 ${d.gaps.join("/")}（组内小/组间大）`);
  console.log(`  截图 → ${OUT}`);

  if (bad.length) {
    console.error(`\n视觉门 FAIL（${bad.length} 项）:`);
    for (const b of bad) console.error(`  · ${b}`);
    process.exitCode = 1;
  } else {
    console.log("视觉门 PASS（高度链通 + 控件等高对齐 + 行高达标 + 列宽有上限 + 无溢出）");
  }
} catch (e) {
  console.error("视觉门 FAIL（环境问题）:", e.stack || e.message);
  process.exitCode = 1;
} finally {
  try { chrome?.close?.(); } catch {}
  try { chromeProc?.kill?.(); } catch {}
  server.close();
}
