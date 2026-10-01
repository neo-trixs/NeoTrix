import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

/**
 * 自持 UI 构建配置。
 *
 * `root`/`build.outDir` 都相对本目录 —— 产物落在 `neobot-ui/dist`，
 * 与 vendored 的 `frontend/dist` **物理分离**，切换只需改
 * `tauri.conf.json` 的 `frontendDist` 指向，不动 vendored 树。
 */
export default defineConfig({
  // ⚠️ Tauri 以 `file://` 加载 dist。默认 `base:'/'` 会产出
  // `src="/assets/index-xxx.js"`，在 file:// 下 `/assets` 解析到**文件系统根**
  // ⇒ 白屏，且**构建成功不报错**（实测踩到过）。必须相对路径。
  base: './',
  plugins: [react()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    // Tauri 以 file:// 加载，必须用相对路径，否则资源解析会挂
    assetsDir: 'assets',
  },
})
