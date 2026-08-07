# E2E 测试机制

## 两层测试架构

| 层 | 工具 | 覆盖 | 命令 |
|----|------|------|------|
| 单测 | Vitest + jsdom + Testing Library | 组件交互、状态、IPC 参数断言 | `npm test` |
| E2E | Playwright (chromium + webkit) | 真实浏览器内标签点击 / 键盘 / 面板开关 | `npm run e2e` |

E2E 通过 `playwright.config.ts` 的 `webServer` 启动 Vite dev server
(`npm run dev -- --port 1420`)，对前端做端到端交互测试。此模式被称为
**browser mode**（Tauri 官方文档也推荐此作为快速的 renderer-only 测试路径）。

## 为什么 E2E 跑在浏览器而非 Tauri 原生 webview

Tauri 2 的原生 WebView 自动化依赖 WebDriver：

- **Windows/Linux**: `tauri-driver`（官方 WebDriver wrapper）可用。
- **macOS**: WKWebView 没有原生 WebDriver，官方 WebdriverIO 服务
  (`@wdio/tauri-service` + `tauri-plugin-wdio-webdriver` embedded driver) 支持，
  但 CrabNebula 的付费路线需要订阅。

因此本仓库采用「浏览器模式 E2E + 单测」双层策略：

- 浏览器模式覆盖所有 UI 交互（标签点击、快捷键、面板开关、错误态）。
- 真实 Rust 后端行为由 `cargo test`（含命令层纯逻辑测试）覆盖。
- IPC 的参数/返回契约由 Vitest 的 `mockInvoke` 断言。

## 关键约定

- **错误过滤规则**：`no uncaught errors` 测试忽略
  `/favicon|tailwind|websocket|tauri|invoke|resizeobserver|transformCallback|__TAURI__|404 (Not Found)/`，
  这些是 dev server / Tauri mock 的预期噪声。
- **标签点击类测试**全部用 `data-testid`（如 `sidebar-tab-files`、`views-menu-terminal`、
  `diff-scope-staged`）以稳定选择器。
- 新增 UI 面板/交互时必须同步：单测 + e2e 各加一例，否则视为未完成。

## 升级路径（未来需要原生 WebView 自动化时）

1. `cargo install tauri-driver`（仅 Windows/Linux）。
2. 或接入 `@wdio/tauri-service` + `tauri-plugin-wdio`（embedded driver，macOS 可用，
   需引入 Rust 插件并注册）。
3. 现有 Playwright 用例可保留为 CI 快路径；WebDriver 用例负责原生 IPC/fs 真实验证。
