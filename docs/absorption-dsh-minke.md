# 吸收分析 — DSH Desktop & Minke → NeoTrix

> 来源: [deepseek-harness-desktop](https://github.com/dsh-tauri-desk/deepseek-harness-desktop) (Tauri 2 + React, 1.1k★)
> [Minke](https://github.com/lencx/Minke) (Electron Forge, lencx/ChatGPT 作者, 483★)
> 日期: 2026-08-25 · 遵循 external-absorption C1-C6 契约

## 一、特性对比矩阵

| 特性 | DSH | Minke | NeoTrix 现状 | 缺口评级 | 行动 |
|------|-----|-------|-------------|---------|------|
| PTY 终端 UI | — | ✅ 右栏 Tab | 后端 4 命令+事件**完备**，前端无 UI | 🔴 高 | **本次实现** |
| 插件市场 | ✅ preset-plugins.json | ✅ GitHub 发现/安装/修复 | PluginMarketplace + marketplace_* 命令已注册 | 🟡 检查接线 | 待验证 |
| Profile 档案隔离 | ✅ service/profile | — | 无 | 🟡 中 | 路线图 |
| Onboarding 向导 | ✅ 首启插件选择 | — | 仅 CLI provider wizard，桌面端缺 | 🟡 中 | 路线图 |
| Files 导航/编辑/Diff | — | ✅ 语法高亮预览 | RightBar 文件树 + FilePreview 已有 | 🟢 基本齐 | 增强 diff 视图 |
| 远程 Web 工作区 (PWA) | — | ✅ Host 投影非投屏 | nt_io_web H5 页面（原型级） | 🟡 中 | 路线图 |
| 本地模型发现 | — | ✅ LM Studio/Ollama 探测 | Ollama provider 支持 | 🟢 | 加自动发现 |
| 自更新 | ✅ service/update | ✅ forge updater | tauri-plugin-updater 已挂 + check_update | 🟢 | — |
| 数据迁移向导 | — | ✅ 预览/合并/去重 | 无 | 🟢 低优 | 路线图 |
| CLI shim 注册 | ✅ dsh 命令 | — | neotrix CLI 独立二进制 | ⚪ N/A | — |
| 多版本内核管理 | ✅ | ✅ harness:stage | N/A（原生二进制） | ⚪ N/A | — |
| ⌘K 命令面板 | — | ✅ Mod+K | CommandPalette 已有 | 🟢 | — |

## 二、架构模式吸收

### DSH 的服务化后端分层（值得借鉴）

```
src-tauri/services/
  download.rs   安装器+解压
  core.rs       内核多版本管理
  profile.rs    档案管理
  plugin.rs     插件卸载/升级
  cli.rs        CLI shim + PATH
  update.rs     自更新
  workflow.rs   进程生命周期
```

NeoTrix 对应: `src-tauri/src/commands/*` 已按域拆分（58 文件），但 main.rs 的
`invoke_handler` 注册了 **~400 个命令平铺**。DSH 模式启示: **命令按 service 分组注册**
（`generate_handler![pty::*]` 宏聚合），可将 main.rs 从 733 行缩至 <100 行。

### Minke 的 Host 投影模式（差异化方向）

Minke 远程访问 = 响应式 Web UI 直连 host 服务（非像素流），手机可完整操作 agent。
NeoTrix 的 `nt_io_web` H5 页面是同类原型但仅覆盖聊天。若走此方向:
- Web 端复用 SolidJS 组件（同构渲染）
- Tauri 命令 ↔ HTTP API 双通道（axum 已内嵌）

### 数据迁移向导（Minke 独有亮点）

预览→合并→去重→冲突保留→重启时切换。对 NeoTrix 的 `knowledge.db` 与
session 存储同样适用（用户换机/多 profile 场景）。

## 三、本次落地: Terminal 面板（PTY + xterm.js）

**后端已有**: `pty_spawn/write/resize/close` 命令 + `pty-output-{id}` / `pty-exit-{id}` 事件
（main.rs:653-667 mpsc→emit 转发）。

**新增前端**:
- `src/api/pty.ts` — 类型化 IPC 封装
- `src/components/TerminalPanel.tsx` — xterm.js 生命周期管理
- Chat.tsx PanelId 扩展 `'terminal'`

## 四、路线图（后续会话）

1. **Profile 档案隔离** — config.toml 多 profile + 切换 UI（借 DSH service/profile）
2. **桌面 Onboarding 向导** — 首启 provider 配置 + 可选功能勾选（借 DSH 向导状态机）
3. **远程工作区** — nt_io_web H5 扩展为响应式全功能投影（借 Minke Host 模式）
4. **本地模型自动发现** — 探测 localhost:11434(Ollama)/1234(LM Studio) 并注册 provider
5. **数据迁移向导** — knowledge.db/session 迁移预览-合并流程（借 Minke）
