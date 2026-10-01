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
  // 相对路径。**证据边界**（勿越读）：
  //   · 已实证：`base:'./'` 在**子路径**挂载下正确加载并真实渲染
  //     （scripts/ops/neobot-ui-smoke.mjs，headless Chrome）。
  //   · 未实证：`base:'/'` 在 Tauri v2 自定义协议下是否失效 —— Tauri 把
  //     frontendDist 挂在协议根，`/assets/…` 未必失效。我最初写的
  //     「file:// 下必白屏」属**手推未验证**，已降级为上述边界。
  //   · 仍取相对路径：它在「根挂载」与「子路径」下**都**正确，
  //     是不依赖未验证前提的唯一选择。
  base: './',
  plugins: [react()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    // Tauri 以 file:// 加载，必须用相对路径，否则资源解析会挂
    assetsDir: 'assets',
  },
})
