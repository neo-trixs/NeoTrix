#!/usr/bin/env node
/**
 * 界面截图 —— 与 nt_check_layout 相同的 HTTP 服务方式。
 *
 * ⛔ **不要用 `file://` 截**。index.html 里样式路径是绝对路径
 *    （`/src/ui/tokens.css`），file:// 下它解析到文件系统根 ⇒ 全部 404 ⇒
 *    截出来是一张**完全没样式**的裸 HTML，看起来像界面崩了。
 *    我第一次就是这么误判的。Tauri 走自己的 server root，路径是对的。
 *
 * 用法：node scripts/ops/nt_shot.mjs [输出路径] [宽] [高]
 */
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { readFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const DIST = join(ROOT, "apps/neobot-desktop/frontend/dist");
const OUT = process.argv[2] || "/tmp/opencode/nb-ui.png";
const W = Number(process.argv[3] || 1280);
const H = Number(process.argv[4] || 840);
const PORT = 9345;

if (!existsSync(join(DIST, "index.html"))) {
  console.error("dist 不存在 —— 先 npm --prefix apps/neobot-desktop/frontend run build");
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

const profile = mkdtempSync(join(tmpdir(), "nb-shot-"));
const child = spawn(CHROME, [
  "--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run",
  "--hide-scrollbars", "--force-device-scale-factor=1",
  `--user-data-dir=${profile}`,
  `--window-size=${W},${H}`,
  "--screenshot=" + OUT,
  `--virtual-time-budget=2500`,
  `http://127.0.0.1:${PORT}/`,
], { stdio: ["ignore", "ignore", "pipe"] });

const t0 = Date.now();
while (Date.now() - t0 < 40000) {
  await new Promise((r) => setTimeout(r, 500));
  if (existsSync(OUT) && readFileSync(OUT).length > 0) break;
  if (child.exitCode !== null) break;
}
try { child.kill("SIGKILL"); } catch {}
server.close();
try { rmSync(profile, { recursive: true, force: true }); } catch {}

if (existsSync(OUT)) {
  console.log(`  ✓ 截图 ${OUT}（${readFileSync(OUT).length} 字节，${W}x${H}，HTTP 服务方式）`);
  process.exit(0);
}
console.error("  ✗ 截图失败（Chrome 提前退出或超时）");
process.exit(2);
