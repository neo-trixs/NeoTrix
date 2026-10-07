#!/usr/bin/env node
/**
 * 按选择器点一下再截图 —— 用来验「点开之后长什么样」。
 *
 * # 为什么要这个工具（而不是给 nt_shot 加个参数）
 *
 * Chrome 的 `--screenshot` flag **只能截首屏**，不能交互。而本项目有一半的
 * 界面（设置面板、会话详情、决策面板）是**点开才出现**的。
 * ⇒ 上一轮「设置面板做完了」这句话是**没有截图支撑**的：
 *   我改了代码、build 通过、typecheck 0，但设置面板**一次都没被看到过**。
 *   这跟本仓记了七次的假绿是同一类：把「没观察到失败」当成「验证通过」。
 *
 * 用法：
 *   node scripts/ops/nt_shot_state.mjs <输出png> <选择器> [选择器...]
 * 例：
 *   node scripts/ops/nt_shot_state.mjs /tmp/s.png "#rail-bottom button"
 *
 * ⚠️ 走 HTTP 服务 dist，**不是 file://**。file:// 下 `/assets/x.css` 这类
 *    绝对路径全部 404 ⇒ 截出一张「裸 HTML」，看起来像布局崩了。
 */

import { createServer } from "node:http";
import { existsSync, readFileSync, mkdtempSync, rmSync } from "node:fs";
import { join, extname, dirname } from "node:path";
import { tmpdir } from "node:os";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { neobotDistFresh } from './nt_dist_freshness.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/neobot-ui/dist");
// ⭐⭐ 2026-10-07 P0-1：产物新鲜度断言（公用件，17 道读 dist 的门统一走这条）。
//    ⛔ 不新鲜就在**开浏览器之前**退出 —— 否则白等几十秒再失败。
neobotDistFresh('nt_shot_state');

const OUT = process.argv[2];
const SELECTORS = process.argv.slice(3);
const W = 1280;
const H = 840;
const PORT = 9347;

if (!OUT) {
  console.error("用法：nt_shot_state.mjs <输出png> <选择器> [选择器...]");
  process.exit(1);
}
if (SELECTORS.length === 0) {
  console.error("至少给一个选择器 —— 本工具的意义就是「点开再截」");
  process.exit(1);
}
if (!existsSync(join(DIST, "index.html"))) {
  console.error("dist 不存在 —— 先在 apps/neobot-desktop/frontend 里 build");
  process.exit(1);
}

const MIME = {
  ".html": "text/html;charset=utf-8", ".js": "text/javascript;charset=utf-8",
  ".css": "text/css;charset=utf-8", ".svg": "image/svg+xml",
  ".png": "image/png", ".json": "application/json",
};
const server = createServer((req, res) => {
  const url = (req.url || "/").split("?")[0];
  const p = join(DIST, url === "/" ? "index.html" : url);
  if (!p.startsWith(DIST) || !existsSync(p)) { res.writeHead(404).end("nf"); return; }
  res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
  res.end(readFileSync(p));
});
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const CHROME = [
  process.env.CHROME_PATH,
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
].filter(Boolean).find((p) => existsSync(p));
if (!CHROME) { console.error("找不到 Chrome"); server.close(); process.exit(2); }

const profile = mkdtempSync(join(tmpdir(), "nb-shotstate-"));
const child = spawn(CHROME, [
  "--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run",
  "--hide-scrollbars", "--force-device-scale-factor=1",
  `--remote-debugging-port=9333`, `--user-data-dir=${profile}`,
  `--window-size=${W},${H}`, "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** 等 DevTools 端点就绪。 */
async function findTarget() {
  for (let i = 0; i < 60; i++) {
    try {
      const r = await fetch("http://127.0.0.1:9333/json/list");
      const list = await r.json();
      const page = list.find((t) => t.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch { /* 还没起来 */ }
    await sleep(250);
  }
  throw new Error("DevTools 端点 15 秒内没就绪");
}

/** CDP 最小客户端：原生 WebSocket，零依赖。 */
function cdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  let id = 0;
  const waiting = new Map();
  ws.addEventListener("message", (e) => {
    const m = JSON.parse(e.data);
    if (m.id && waiting.has(m.id)) { waiting.get(m.id)(m); waiting.delete(m.id); }
  });
  const ready = new Promise((r) => ws.addEventListener("open", r, { once: true }));
  const listeners = new Map();
  ws.addEventListener("message", (e) => {
    const m = JSON.parse(e.data);
    if (m.id && waiting.has(m.id)) { waiting.get(m.id)(m); waiting.delete(m.id); }
    else if (m.method && listeners.has(m.method)) listeners.get(m.method)(m.params);
  });
  const send = (method, params = {}) =>
    new Promise((res, rej) => {
      const mid = ++id;
      waiting.set(mid, (m) => (m.error ? rej(new Error(m.error.message)) : res(m.result)));
      ws.send(JSON.stringify({ id: mid, method, params }));
    });
  const on = (method, fn) => listeners.set(method, fn);
  return { ready, send, on, close: () => ws.close() };
}

let code = 0;
try {
  const wsUrl = await findTarget();
  const c = cdp(wsUrl);
  await c.ready;
  await c.send("Page.enable");
  await c.send("Runtime.enable");
  // ⛔ 「点了没反应」有两种可能：按钮没接上，或处理器抛了错。
  //    只看截图分不出这两者 —— 截图里两者都是「没变化」。
  //    所以把异常显式抓出来。consoleAPICalled + exceptionThrown 两条都收。
  c.on("Runtime.exceptionThrown", (p) => {
    const d = p.exceptionDetails;
    console.error(`  ✗ 页面抛异常：${d.text} ${d.exception?.description ?? ""}`.slice(0, 200));
    code = 4;
  });
  c.on("Runtime.consoleAPICalled", (p) => {
    if (p.type === "error" || p.type === "warning") {
      const txt = p.args.map((a) => a.description ?? a.value ?? "").join(" ");
      console.error(`  ✗ console.${p.type}：${txt}`.slice(0, 200));
      if (p.type === "error") code = 4;
    }
  });
  await c.send("Page.navigate", { url: `http://127.0.0.1:${PORT}/` });
  // 等首屏与模块加载完
  await sleep(1800);

  // 逐个点。⛔ 用**真实 click 事件**而不是直接调函数 ——
  //    直接调函数会绕过事件监听器，截出来的图「像是点开了」，
  //    而按钮其实没接上 click。那是又一次「以为验过了」。
  for (const sel of SELECTORS) {
    const expr = `(() => {
      const n = document.querySelector(${JSON.stringify(sel)});
      if (!n) return "MISS:" + ${JSON.stringify(sel)};
      n.click();
      return "OK:" + ${JSON.stringify(sel)};
    })()`;
    const r = await c.send("Runtime.evaluate", { expression: expr, returnByValue: true });
    const v = r.result?.value ?? "";
    if (String(v).startsWith("MISS:")) {
      console.error(`  ✗ 选择器没命中：${sel}`);
      code = 3;
    } else {
      console.log(`  ✓ 已点击 ${sel}`);
    }
    await sleep(500);
  }

  // ⛔ 必须显式设 viewport。`--window-size` 在 headless=new 下**不保证**
  //    作用到 Page.captureScreenshot —— 实测请求 1280x820 拿到的是 756x413，
  //    于是截出来的「设置面板」只有半屏，右侧整块空白，
  //    看起来像面板没打开/没布局。**先量再信**：不设就量不准。
  await c.send("Emulation.setDeviceMetricsOverride", {
    width: W, height: H, deviceScaleFactor: 1, mobile: false,
  });
  await sleep(400);
  const dims = await c.send("Runtime.evaluate", {
    expression: "innerWidth + 'x' + innerHeight", returnByValue: true,
  });
  console.log(`  · 视口 ${dims.result?.value}`);
  const shot = await c.send("Page.captureScreenshot", { format: "png" });
  const buf = Buffer.from(shot.data, "base64");
  const { writeFileSync } = await import("node:fs");
  writeFileSync(OUT, buf);
  console.log(`  ✓ 截图 ${OUT}（${buf.length} 字节，${W}x${H}，点击后）`);
  c.close();
} catch (e) {
  console.error(`  ✗ ${e.message}`);
  code = 2;
} finally {
  try { child.kill("SIGKILL"); } catch {}
  server.close();
  try { rmSync(profile, { recursive: true, force: true }); } catch {}
}
process.exit(code);
