import { defineConfig } from 'vite'
import solidPlugin from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 1421,
    strictPort: true,
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    rollupOptions: {
      output: {
        // 代码分割：vendor 稳定分包（solid 生态 / tauri 桥 / UI 图标），
        // three·globe·topojson 等重型 3D 依赖独立成包，仅 /globe 懒加载时拉取
        manualChunks: (id) => {
          if (id.includes('node_modules/solid-js') || id.includes('node_modules/@solidjs/router')) {
            return 'vendor-solid'
          }
          if (id.includes('node_modules/@tauri-apps')) {
            return 'vendor-tauri'
          }
          if (
            id.includes('node_modules/lucide-solid') ||
            id.includes('node_modules/clsx') ||
            id.includes('node_modules/tailwind-merge')
          ) {
            return 'vendor-ui'
          }
          if (
            id.includes('node_modules/three') ||
            id.includes('node_modules/globe.gl') ||
            id.includes('node_modules/topojson-client')
          ) {
            return 'vendor-globe'
          }
          return undefined
        },
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
})