import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'

/**
 * 包内套件入口：根 vitest.config.ts 已改为 project 清单（unit/plugin/desktop
 * 三 lane），从包目录裸跑 `vitest run` 会在错误的 CWD 下解析那份清单。本包用例
 * 显式声明自身配置即可独立运行（`pnpm --filter dsh-tauri-ssh test`），也避免
 * 依赖根 lane 的装配前提。
 *
 * 组件用例渲染官方 UI primitives，其包带真实 `.module.css` 边车文件、且在 pnpm 下
 * 嵌套了另一份 react@18——而本包面向 workspace 的 react 开发。部署时的 ModuleLoader
 * 里两者共用同一个平台 react，这里用测试期设置复原该环境：primitives 包内联让 Vite
 * 处理它的 CSS import，所有 react specifier 钉到 workspace 那份，使 jsdom 只挂一个
 * 渲染器而不是两个。（组件用例自带 `@vitest-environment jsdom` 文档块。）
 */
const reactRoot = fileURLToPath(new URL('../../node_modules/react', import.meta.url))

export default defineConfig({
  resolve: {
    alias: [
      { find: /^react$/, replacement: reactRoot },
      { find: /^react\/jsx-runtime$/, replacement: `${reactRoot}/jsx-runtime.js` },
      { find: /^react\/jsx-dev-runtime$/, replacement: `${reactRoot}/jsx-dev-runtime.js` },
    ],
  },
  test: {
    environment: 'node',
    css: true,
    server: {
      deps: {
        inline: [/@deepseek-ai\/dsh-client-ui-primitives/],
      },
    },
  },
})
