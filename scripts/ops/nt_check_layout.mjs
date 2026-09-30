#!/usr/bin/env node
/**
 * 布局门 —— 用 headless Chrome **实测盒模型**，不靠肉眼。
 *
 * # 为什么必须有这道门
 *
 * 本项目至今只有静态门（token 门 / 字节门 / 类型 / 逻辑自测 / build）。
 * 它们全部能绿，而界面**依然可能是坏的** —— 静态门不看几何。
 * 本仓历史上最贵的一类 bug 正是几何：「会话列表滚不动、输入框被顶出视口」，
 * `overflow-y: auto` 明明写着，却因为高度链上有一环缺 `min-height: 0`
 * 而永不触发（grid/flex item 默认 `min-height: auto`，拒绝收缩到内容以下，
 * 把父级撑高）。
 *
 * # 为什么用 CDP 而不是 --dump-dom / --screenshot
 *
 * 实测（2026-09-30）：本机 Chrome 的 `--dump-dom`、`--screenshot`、
 * `--headless=new` **全部挂死**，无论加不加 `--virtual-time-budget`，
 * 两次各烧掉 30 分钟超时。⇒ 不能依赖「Chrome 自己退出」。
 *
 * CDP（`--remote-debugging-port`）实测 1s 就绪，且**生命周期由我控制**：
 * 拿到结果后显式 `kill`，不靠浏览器善后。Node 22+ 有原生 WebSocket，
 * 所以本门**零依赖**。
 *
 * # 判据
 *
 *   ① 高度链每一环 min-height 必须是 0（否则纵向滚动失效）
 *   ② 侧栏列表与线程在内容超出时**真的**可滚（scrollHeight > clientHeight）
 *
 *   ⚠️ 侧栏选择器是 `.convs-host` 而**不是** `.convs`：
 *      列表被三态容器（加载/失败/空/内容）包住，滚动责任在 host 上。
 *      换成三态那轮忘了改这里，门立刻报「不存在」——
 *      门报「不存在」时先怀疑**自己代码结构变了**，而不是门过时了。
 *   ③ 语义 token 在**计算样式**里真的解析出值（不是空串/未解析）
 *   ④ 关键元素不溢出视口（输入框必须在视口内可见）
 *   ⑤ 无横向溢出（横向滚动条通常是布局 bug 的症状）
 */

import { spawn } from "node:child_process";
import { mkdtempSync, rmSync, existsSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/frontend/dist");
const W = Number(process.env["NB_W"] || 1280);
const H = Number(process.env["NB_H"] || 820);
const PORT = 9341;

if (!existsSync(join(DIST, "index.html"))) {
  console.error("布局门 FAIL: dist 不存在 —— 先跑 npm --prefix apps/neobot-desktop/frontend run build");
  process.exit(1);
}

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
  ".html": "text/html;charset=utf-8",
  ".js": "text/javascript;charset=utf-8",
  ".css": "text/css;charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".json": "application/json",
};

// ── 静态服务（只读 dist） ──────────────────────────────────────
const server = createServer((req, res) => {
  const url = (req.url || "/").split("?")[0];
  let p = join(DIST, url === "/" ? "index.html" : url);
  if (!p.startsWith(DIST)) { res.writeHead(403).end(); return; }
  if (!existsSync(p)) { res.writeHead(404).end("not found"); return; }
  res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
  res.end(readFileSync(p));
});

// ── 页面内探针 ──────────────────────────────────────────────────
const PROBE = `(() => {
  const g = (s) => document.querySelector(s);
  // ⛔ 必须先注入足量内容再量。原因：只有 3 条会话时 scrollHeight == clientHeight
  //    是**正常**的（内容装得下），此时测不出 overflow 是否生效。
  //    本仓历史上最贵的 bug 正是「overflow-y: auto 明明写了却不触发」，
  //    那种 bug 只有在内容**真的超出**时才显形。
  const CONVOS = 40, MSGS = 60;
  try {
    const convs = g(".convs-host");
    if (convs) {
      convs.replaceChildren();
      for (let i = 0; i < CONVOS; i++) {
        const row = document.createElement("div");
        row.className = "convo";
        const t = document.createElement("div");
        t.className = "convo-txt";
        const ttl = document.createElement("div");
        ttl.className = "convo-title";
        ttl.textContent = "注入会话 " + i;
        t.appendChild(ttl);
        row.appendChild(t);
        convs.appendChild(row);
      }
    }
    const th = g(".thread");
    if (th) {
      for (let i = 0; i < MSGS; i++) {
        const m = document.createElement("div");
        m.className = "msg msg--bot";
        const b = document.createElement("div");
        b.className = "msg-bubble";
        b.textContent = "注入消息 " + i;
        m.appendChild(b);
        th.appendChild(m);
      }
    }
  } catch (e) { /* 注入失败照常量，判据里会体现 */ }
  const cs = (s, p) => { const e = g(s); return e ? getComputedStyle(e)[p] : "MISSING"; };
  const chain = [".app", ".side", ".convs-host", ".thread-col", ".thread"].map((sel) => {
    const e = g(sel);
    if (!e) return { sel, missing: true };
    const c = getComputedStyle(e);
    return { sel, minHeight: c.minHeight, h: Math.round(e.getBoundingClientRect().height) };
  });
  const scrollable = (sel) => {
    const e = g(sel);
    if (!e) return null;
    const c = getComputedStyle(e);
    return {
      overflowY: c.overflowY,
      overflowing: e.scrollHeight > e.clientHeight,
      sh: e.scrollHeight, ch: e.clientHeight,
    };
  };
  const tokens = {};
  for (const t of ["--nb-bg-surface","--nb-text-primary","--nb-type-msg","--nb-radius-control","--nb-font-mono"]) {
    tokens[t] = getComputedStyle(document.documentElement).getPropertyValue(t).trim();
  }
  const composer = g(".composer-input textarea");
  const cr = composer ? composer.getBoundingClientRect() : null;
  return {
    ready: !!g(".app"),
    chain,
    convs: scrollable(".convs-host"),
    thread: scrollable(".thread"),
    tokens,
    hostBadge: g("#host-badge") ? g("#host-badge").textContent : null,
    panels: [...document.querySelectorAll(".side-panel-btn")].map(b => b.textContent),
    convos: [...document.querySelectorAll(".convo-title")].map(b => b.textContent),
    vh: innerHeight,
    appH: g(".app") ? Math.round(g(".app").getBoundingClientRect().height) : null,
    composerInViewport: cr ? (cr.bottom <= innerHeight + 1 && cr.top >= -1) : false,
    composerH: cr ? Math.round(cr.height) : null,
    hOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
  };
})()`;

// ── CDP 最小客户端（原生 WebSocket，零依赖） ────────────────────
function cdpGet(url, tries = 20) {
  return new Promise((resolve, reject) => {
    let n = 0;
    const tick = () => {
      fetch(url)
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
        .then(resolve)
        .catch(() => {
          if (++n >= tries) reject(new Error(`CDP 无响应：${url}`));
          else setTimeout(tick, 300);
        });
    };
    tick();
  });
}

async function evaluate(wsUrl, expr, vw, vh) {
  const ws = new WebSocket(wsUrl);
  const pending = new Map();
  let id = 0;
  const send = (method, params) =>
    new Promise((res, rej) => {
      const mid = ++id;
      pending.set(mid, { res, rej });
      ws.send(JSON.stringify({ id: mid, method, params }));
      setTimeout(() => {
        if (pending.has(mid)) { pending.delete(mid); rej(new Error(`${method} 超时`)); }
      }, 20000);
    });
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.id && pending.has(m.id)) {
      const { res, rej } = pending.get(m.id);
      pending.delete(m.id);
      m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
    }
  };
  await send("Page.enable");
  await send("Runtime.enable");
  // 直接设视口。--window-size 改的是窗口，headless 下内容视口不跟着变
  // （实测 820 → innerHeight 仍 413），会让人以为在 820 下测过了。
  await send("Emulation.setDeviceMetricsOverride", {
    width: vw, height: vh, deviceScaleFactor: 1, mobile: false,
  });
  const url = process.env["NB_URL"] || `http://127.0.0.1:${PORT}/`;
  await send("Page.navigate", { url });
  // 等 load 事件而不是死等固定毫秒
  await new Promise((r) => setTimeout(r, 1500));
  const out = await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true });
  ws.close();
  if (out.exceptionDetails) throw new Error(out.exceptionDetails.text || "页面内异常");
  return out.result.value;
}

// ── 主流程 ──────────────────────────────────────────────────────
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

  const targets = await cdpGet("http://127.0.0.1:9333/json");
  const page = targets.find((t) => t.type === "page") || targets[0];
  const d = await evaluate(page.webSocketDebuggerUrl, PROBE, W, H);

  // ── 判据 ──
  const bad = [];
  if (!d.ready) bad.push("页面没就绪（.app 不存在）");

  for (const c of d.chain) {
    if (c.missing) { bad.push(`${c.sel} 不存在`); continue; }
    // min-height 必须是 0（0px 或 normal→0）。缺这环纵向滚动必失效。
    const mh = c.minHeight;
    if (mh !== "0px" && mh !== "0") {
      bad.push(`${c.sel} 的 min-height=${mh}（应为 0px；缺这环则 overflow-y 不触发）`);
    }
  }
  for (const [name, r] of [["侧栏列表", d.convs], ["对话流", d.thread]]) {
    if (!r) { bad.push(`${name} 不存在`); continue; }
    if (r.overflowY !== "auto" && r.overflowY !== "scroll") {
      bad.push(`${name} 的 overflow-y=${r.overflowY}（应为 auto/scroll，否则内容一多就撑破而不是滚动）`);
    }
    // 注入了 40/60 条，内容必然超出 ⇒ 滚不动是可判定的
    if (!r.overflowing) {
      bad.push(`${name} 注入后内容超出（${r.sh} > 容器 ${r.ch}）但不可滚 —— overflow 未生效`);
    }
  }
  // 整个栅格不得超出视口：超出即「输入框被顶出视口」那类 bug
  if (d.appH > d.vh + 1) {
    bad.push(`.app 高 ${d.appH}px 超出视口 ${d.vh}px（高度链某环把父级撑高了）`);
  }
  for (const [name, t] of Object.entries(d.tokens)) {
    if (!t) bad.push(`token ${name} 在计算样式里为空（未解析）`);
  }
  if (d.composerInViewport === false) bad.push(`输入框不在视口内（高 ${d.composerH}px）`);
  if (d.hOverflow) bad.push("出现横向溢出（通常是布局 bug 的症状）");

  console.log(`  视口 ${W}x${H} · 宿主 ${d.hostBadge ?? "?"}`);
  console.log(`  面板 ${JSON.stringify(d.panels)} · 会话 ${d.convos.length} 条`);
  for (const c of d.chain) {
    console.log(`    ${c.sel.padEnd(12)} min-height=${String(c.minHeight).padEnd(5)} h=${c.h ?? "-"}`);
  }
  for (const [nm, r] of [["convs-host", d.convs], ["thread", d.thread]])
    console.log(`    .${nm.padEnd(6)} overflow-y=${String(r?.overflowY).padEnd(7)} 超出=${r?.overflowing} (${r?.sh}/${r?.ch})`);
  console.log(`    .app 高 ${d.appH} / 视口高 ${d.vh}`);

  if (bad.length) {
    console.error(`\n布局门 FAIL（${bad.length} 项）:`);
    for (const b of bad) console.error("  · " + b);
    failed = 1;
  } else {
    console.log("\n布局门 PASS（高度链完整、token 已解析、无溢出）");
    failed = 0;
  }
} catch (e) {
  // 「没测到」与「测坏了」必须分开
  console.error("布局门 FAIL: 环境问题（不是布局问题）—— " + (e instanceof Error ? e.message : String(e)));
  console.error("  · 常见病因: Chrome 启动失败 / CDP 端口被占 / dist 未构建");
  failed = 2;
} finally {
  // 显式关停。不靠浏览器自己退出 —— 本机实测它不退。
  try { if (child) child.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(profile, { recursive: true, force: true }); } catch {}
}
process.exit(failed);
