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
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/frontend/dist");
const W = Number(process.env["NB_W"] || 1280);
const H = Number(process.env["NB_H"] || 840);
const PORT = 9341;

if (!existsSync(join(DIST, "index.html"))) {
  console.error("布局门 FAIL: dist 不存在 —— 先跑 pnpm --dir apps/neobot-desktop/frontend run build");
  process.exit(1);
}

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
      if (cmd === 'neobot_convo_messages') {
        return Array.from({ length: 60 }, (_, i) => ({ id: 'm' + i, convo_id: (args && args.convo_id) || 'c1', role: i % 2 ? 'assistant' : 'user', text: '消息' + i + '：这是一条足够长的测试消息，用来把消息流撑出可滚动的高度。', created_at: new Date().toISOString() }));
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
  await new Promise((r) => setTimeout(r, 1500));

  const targets = await cdpGet("http://127.0.0.1:9333/json");
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
      if (d.ready && d.rows >= 40 && d.bubbles >= 60) break;
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
      bad.push(`选中态与未选中态背景相同（${sel} vs ${unsel}）—— 等于没有选中态`);
    }

    if (d.rows < 40) bad.push(`会话列表只渲染了 ${d.rows} 条（桩给了 40 条）`);
    if (d.bubbles < 60) bad.push(`消息流只渲染了 ${d.bubbles} 个气泡（桩给了 60 条）`);
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
