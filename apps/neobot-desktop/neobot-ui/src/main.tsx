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
 * `OverlaysProvider`（三个上游 provider）。我方 UI **不依赖**其中任何一个
 * ——它只用 react 内建 hooks + `@tauri-apps/api/core` 的 `invoke`。
 * 故此处不挂 provider，避免把 vendored 依赖重新引进自持树。
 *
 * 参见 `PROVENANCE.md` 与 `docs/architecture/FRONTEND-REBUILD-2026-10-01.md`。
 */
import React from 'react'
import ReactDOM from 'react-dom/client'

import { NeoBotRoot } from './neobot-root'
import { Shell, useExternalLinks } from './shell'
// 副作用导入：模块加载即同步 `document.documentElement.lang`，
// openghost 垫片读它决定代码块复制按钮的 aria-label。
import './i18n'
import './theme.css'
import './nb-scroll.css'

/**
 * 外链拦截（`open_external_url`）。必须在**任何**内容挂载前注册，
 * 且用捕获阶段（`useExternalLinks` 内 `addEventListener(..., true)`）——
 * 否则已被 React 处理的点击不会冒泡到 document。
 */
function Root() {
  useExternalLinks()
  return (
    <Shell>
      <NeoBotRoot />
    </Shell>
  )
}

const host = document.getElementById('root')
if (!host) {
  // 契约：#root 缺失说明 index.html 与本文件不同源，属接线断裂而非运行时故障。
  // 用可定位的报错而非静默退出（静默 = 白屏，无从排查）。
  throw new Error('neobot-ui: #root not found in index.html — 入口与宿主页不同源')
}

ReactDOM.createRoot(host).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
)
