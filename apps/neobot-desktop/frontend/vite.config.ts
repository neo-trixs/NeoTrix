import { defineConfig } from "vite";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
// 源码在 frontend/src，图标在 frontend/../icons —— Vite 默认只服务 root 之下，
// 不配 fs.allow 的话 /neobot/icons/*.svg 会 404，而 404 只在运行期出现，
// tsc 那关拦不住。
// 图标源在 frontend/../icons（tauri.conf.json 也引用同一份）。
// 用 publicDir 直挂，而不是拷一份进 frontend/public ——
// 「各平台各画/各存一份」正是本仓 icon-design 明令禁止的。
const icons = resolve(root, "..", "icons");

export default defineConfig({
  clearScreen: false,
  publicDir: icons,
  server: { port: 1423, strictPort: true, fs: { allow: [root, icons] } },
  build: { outDir: "dist", target: "es2022", emptyOutDir: true },
});
