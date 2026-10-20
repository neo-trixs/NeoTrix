# NeoTrix Desktop — Terax 对标分析 & 优化方案

> 参照项目: [crynta/terax-ai](https://github.com/crynta/terax-ai) (2.6k⭐, Tauri 2 + Rust + React 19, 7MB AI 终端)

---

## 1. 核心架构差距

| 维度 | Terax | NeoTrix(当前) | 建议方案 |
|------|-------|--------------|---------|
| 终端渲染 | xterm.js + WebGL | 自研 wgpu GLSL | **替换为 xterm.js** |
| PTY 后端 | portable-pty crate | 无 PTY | **添加 portable-pty** |
| 密钥存储 | OS keychain (keyring) | 环境变量 | **更换为 keyring** |
| 前端框架 | shadcn/ui + Tailwind v4 | 原生 HTML/CSS | **采用 shadcn/ui** |
| 状态管理 | Zustand | 无 | **引入 Zustand** |
| AI SDK | Vercel AI SDK v6 | 自定义 | **可选集成** |
| 模块化 | modules/<area>/ | 单文件 | **采用模块结构** |
| 跨平台 | macOS/Linux/Windows | macOS only | **补齐平台** |

## 2. 终端子系统重设计

**当前问题**: NeoTrix 在 `src/neotrix/terminal/` 自研了 wgpu GPU 终端渲染器（7+文件、WGSL shader），但：
- 没有 PTY 支持，无法运行真实 shell
- 输入编辑器简陋
- 没有分屏、标签、搜索

**Terax 方案**（已验证、7MB、生产级）:
```
前端: xterm.js (WebGL) ↔ Tauri Channel<PtyEvent> ↔ Rust: portable-pty
```

### 实施步骤
1. `cargo add portable-pty` (Rust PTY 后端)
2. 前端 `npm install @xterm/xterm @xterm/addon-fit @xterm/addon-webgl`
3. 创建 `src-tauri/src/modules/pty/` — PTY 会话管理
4. 创建 `src/modules/terminal/` — xterm.js 标签页

## 3. AI 集成架构

Terax 采用 BYOK + Vercel AI SDK v6 模式。NeoTrix 已有 ReasoningEngine 但缺少：
- **OS keychain** 存储 API key（`keyring` crate）
- **工具审批流**（危险操作需用户确认）
- **多 Agent 子代理**（main agent → sub-agents）

### 实施步骤
1. `cargo add keyring` + secrets commands
2. 前端 <PermissionDialog/> 组件
3. agent/tools.rs 添加审批标记

## 4. 前端组件化

参照 Terax 的模块布局:
```
src/modules/
├── terminal/    # xterm.js + PTY bridge
├── ai/          # AI chat + agent + tools
│   ├── lib/     # agent.ts, sessions.ts, composer.tsx
│   ├── tools/   # tools.ts, security.ts
│   └── agents/  # registry.ts, runSubagent.ts
├── explorer/    # file tree + fuzzy search
├── editor/      # CodeMirror 6
├── settings/    # settings store + UI
├── tabs/        # tab management
└── statusbar/   # cwd + AI status
```

## 5. 跨平台优化

| 平台 | Terax | NeoTrix 所需改动 |
|------|-------|-----------------|
| macOS | native traffic lights (titleBarStyle: Overlay) | ✅ 已有 |
| Linux | custom WindowControls + decorations: false | 添加 tauri.linux.conf.json |
| Windows | NSIS installer (currentUser) + WebView2 embed | 添加 tauri.windows.conf.json |
| Auto-update | tauri-plugin-updater + minisign | 添加 updater |

## 6. 架构升级清单

```
Phase 1: PTY 终端 (1周)
  - cargo add portable-pty
  - npm install @xterm/xterm @xterm/addon-webgl
  - 创建 pty/ module (session.rs, shell_init.rs)
  - 创建 frontend terminal/ module

Phase 2: 密钥存储 (2天)
  - cargo add keyring
  - secrets_* Tauri commands
  - 前端 SettingsPanel 对接

Phase 3: 前端组件化 (1周)
  - npm install shadcn/ui tailwindcss zustand
  - 拆分单文件 → modules/ 结构
  - 实现 Tab 系统 + 状态管理

Phase 4: 跨平台 (3天)
  - tauri.linux.conf.json + tauri.windows.conf.json
  - WindowControls 组件
  - CI 交叉编译

Phase 5: 发布 (2天)
  - tauri-plugin-updater
  - GitHub Release CI
  - Homebrew tap
```
