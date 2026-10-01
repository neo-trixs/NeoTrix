/**
 * NeoBot **自持入口** —— 不 import 任何 vendored（dsh-tauri）代码。
 *
 * # 为什么存在这个文件
 *
 * 原先我方 UI（`neobot-root.tsx`）**不是** vite 入口，入口是上游
 * `frontend/src/main.tsx`，我方 UI 只能经
 *   上游 main.tsx → … → 上游 layout/components/webview.tsx
 *     → `selfHosted ? <NeoBotRoot/> : <iframe/>`
 * 被「请」去渲染（开关由上游 `harness/store.ts` 的
 * `selfHosted = !hasService` 计算，后端 `get_runtime_info` 硬编码
 * `has_service:false` ⇒ 该分支为真）。
 *
 * ⇒ 文件级虽零上游 import，**渲染路径却穿过上游**。删 vendored 树时
 *    若只改 `frontendDist`，我方 UI 会**静默变成死代码**，界面无任何异常。
 *    本文件即那条缺失的替代路径。
 *
 * # 与上游入口的差别
 *
 * 上游 `main.tsx` 挂 `QueryClientProvider` / `ToastProvider` /
 * `OverlaysProvider`（三个上游 provider）。我方 UI **不依赖** 其中任何一个
 * ——它只用 react 内建 hooks + `@tauri-apps/api/core` 的 `invoke`。
 * 故此处不挂 provider，避免把 vendored 依赖重新引进自持树。
 *
 * 参见 `PROVENANCE.md` 与 `docs/architecture/FRONTEND-REBUILD-2026-10-01.md`。
 */
import React from 'react'
import ReactDOM from 'react-dom/client'

import { NeoBotRoot } from './neobot-root'

/**
 * 设定 `document.documentElement.lang`。
 *
 * 不是装饰：`vendor/openghost/shim.ts` 的 `lang()` 读
 * `document.documentElement.lang.startsWith('zh')` 来决定代码块复制按钮
 * 的中/英 aria-label（`code.copy`）。不设则该按钮失名。
 * 优先级刻意为「显式 lang 属性 > 浏览器语言」，因为 Tauri 窗口环境下
 * `navigator.language` 可能与用户实际选择不一致。
 */
function applyDocumentLang(): void {
  const fromNavigator = typeof navigator !== 'undefined' ? navigator.language || '' : ''
  const lang = fromNavigator.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
  document.documentElement.lang = lang
}

const host = document.getElementById('root')
if (!host) {
  // 契约：#root 缺失说明 index.html 与本文件不同源，属接线断裂而非运行时故障。
  // 用可定位的报错而非静默退出（静默 = 白屏，无从排查）。
  throw new Error('neobot-ui: #root not found in index.html — 入口与宿主页不同源')
}

applyDocumentLang()
ReactDOM.createRoot(host).render(
  <React.StrictMode>
    <NeoBotRoot />
  </React.StrictMode>,
)
