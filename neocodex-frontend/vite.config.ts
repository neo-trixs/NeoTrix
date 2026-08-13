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
          // three 核心（尾部带 / 仅匹配 three 本体，不误吞 three-* 子包）
          if (id.includes('node_modules/three/')) {
            return 'vendor-three'
          }
          // three-globe 生态：球体渲染与几何辅助
          if (
            id.includes('node_modules/three-globe') ||
            id.includes('node_modules/three-render-objects') ||
            id.includes('node_modules/three-slippy-map-globe') ||
            id.includes('node_modules/three-conic-polygon-geometry') ||
            id.includes('node_modules/three-geojson-geometry') ||
            id.includes('node_modules/h3-js') ||
            id.includes('node_modules/tinycolor2') ||
            id.includes('node_modules/data-bind-mapper') ||
            id.includes('node_modules/frame-ticker') ||
            id.includes('node_modules/index-array-by')
          ) {
            return 'vendor-three-globe'
          }
          // d3 生态（地理投影/网格/刻度）与其他 globe 支撑库
          if (
            id.includes('node_modules/d3-') ||
            id.includes('node_modules/@tweenjs') ||
            id.includes('node_modules/accessor-fn') ||
            id.includes('node_modules/kapsule')
          ) {
            return 'vendor-globe-d3'
          }
          // 剩余 globe 外壳与按需加载的 topojson
          if (id.includes('node_modules/globe.gl') || id.includes('node_modules/topojson-client')) {
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