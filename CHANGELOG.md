# NeoTrix Desktop V2 — Changelog

## v0.19.0-rc2 (2026-09-08)

### Chat Plugin: 对话打通 + Tauri 事件发射
- **问题**: 前端调用 `neocodex_send_message_stream` 报 "Command not found"；流式事件未发射导致前端卡在生成中
- **根因**: ChatPlugin 只有 `send` action，没有 `send_message_stream`/`stop_stream`/`get_session_messages`；无 Tauri 事件发射
- **修复**: 
  - ChatPlugin 新增 3 个 action，`send_message_stream` 调用本地 llama.cpp (`http://127.0.0.1:8080/v1/chat/completions`)
  - 使用 `OnceLock<AppHandle>` 存储 app handle，通过 `setup` hook 注入
  - `call_llm` 发射 `neocodex_stream_start`/`neocodex_stream_token`/`neocodex_stream_end`/`neocodex_stream_done` 事件
- **涉及文件**: `src-tauri/src/domain/plugins/chat.rs`, `src-tauri/src/domain/plugins/mod.rs`, `src-tauri/src/main.rs`

### 意识核心集成尝试 (已回退)
- **目标**: 将 chat plugin 连接到 neotrix-core 的意识核心 (ConsciousnessCore)，实现任务分解 → 模型路由 → 自动执行
- **发现**: neotrix-core 有 87 个编译错误，无法作为依赖引入 Tauri 应用
- **当前状态**: chat plugin 使用简化版本 — 直接调用 llama.cpp，系统提示中加入任务分解指令
- **后续**: 需要先修复 neotrix-core 的编译错误，才能完整集成意识核心

### ModelSwitcher: 动态模型池加载
- **问题**: 模型名硬编码为 `neotrix-core` 或 GGUF 文件名
- **修复**: ModelSwitcher 并行加载 `providerConfig()` (config.toml) + `getModelPoolStatus()` (provider_pool.toml)，合并去重后动态展示所有可用模型
- **涉及文件**: `src/components/ModelSwitcher.tsx`

### Theme Consistency Fix
- **问题**: 对话界面 (main) 变黑，左侧栏 (sidebar) 保持白色，主题不一致
- **根因**: `body` 背景用硬编码渐变，`glass-side` 用硬编码 `#fbfbfa`，`--color-canvas`/`--color-panel` 变量存在但未被任何组件引用
- **修复**:
  - `body` 背景改为 `var(--color-canvas, #ffffff)` — 主题变量控制
  - `.glass-side` 背景改为 `var(--color-panel, #f9fafb)` — 主题变量控制
  - `:root` 定义完整 surface tokens（canvas/panel/line/ink 等），所有主题继承
  - 删除死文件 `src/style/main.css`（v4 Tailwind 暗色主题，从未 import）
  - 删除死文件 `src/styles/design-tokens.css`（从未 import）
- **涉及文件**: `src/styles/index.css`

### Model Name Display Fix
- **问题**: 模型切换器显示原始 GGUF 文件名 `Agents-A1-4B-kimi-Preview-heretic-IQ4_NL`，截断后为 `Agents-A1-4B-k...`
- **修复**: `pillModel()` 去掉 GGUF 后缀 (`heretic`/`IQ4_NL`/`gguf`)，`-`/`_` 转空格，显示 `Agents A1 4B kimi Preview`
- **涉及文件**: `src/components/ModelSwitcher.tsx`

### Tauri Window Rounded Corners Fix
- **问题**: 窗口失去圆角
- **根因**: `html`/`body` 缺少 `overflow: hidden` + `border-radius: 12px`，与 Tauri `windowEffects.radius: 12` 不匹配
- **修复**: `html` 和 `body` 都加了 `overflow: hidden; border-radius: 12px`
- **涉及文件**: `src/styles/index.css`

### Agent Stub Plugin (Real Data)
- **问题**: Agent 插件 `provider_config`/`provider_status`/`pool_sufficiency`/`discover_models` 返回空 stub
- **修复**: 读取 `~/.config/neotrix/config.toml` 返回真实数据
- **涉及文件**: `src-tauri/src/domain/plugins/stubs.rs`

---

## v0.19.0-rc1 (2026-09-08)

### Architecture
- 12 Domain Plugins registered via `DomainRegistry`
- All frontend API routed through `adapter.ts` → `enhancedInvoke` → `domain_call`
- Tauri 2 with `decorations: false`, `transparent: true`, `windowEffects.radius: 12`

### API Routing Migration
- 9 API files changed from `call()` to `enhancedInvoke()`
- Adapter `DOMAIN_MAP` expanded with `memory_*`, `kb_doc_*`, `save_api_key`/`has_api_key`/`delete_api_key`
- `mapAction()` strips prefixes: `neocodex_`, `kb_`, `canvas_`, `provider_`, `pool_`, `discover_`, `probe_`, `memory_`

### Config
- `~/.config/neotrix/config.toml`: `provider = "llamacpp"`, `default_model = "Agents-A1-4B-kimi-Preview-heretic-IQ4_NL"`

---

## Regression Rules

1. **CSS 变量单一事实源**: 所有背景/边框/文字色必须用 CSS 变量，禁止硬编码 hex
2. **主题继承**: `:root` 定义默认值，`[data-theme]` 仅覆盖 accent 色
3. **死文件清理**: 未 import 的 CSS/JS 文件必须删除，防止混淆
4. **模型名显示**: ModelSwitcher 从 config.toml + provider_pool.toml 动态加载，禁止硬编码模型名
5. **窗口圆角**: `html`/`body` 必须有 `overflow: hidden; border-radius: 12px`
6. **构建验证**: 每次修改 CSS/组件后，必须 `npx vite build` + `cargo build --release`，检查 dist 产物
7. **Chat action 命名**: 前端 `neocodex_send_message_stream` → adapter 剥离前缀 → `send_message_stream`，chat plugin 必须有此 action
8. **动态模型列表**: ModelSwitcher 必须并行加载 `providerConfig()` + `getModelPoolStatus()`，合并去重
