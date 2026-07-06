# NeoTrix Release Checklist

对标: Codex CLI (80K★, Rust, kernel sandbox, CI/CD native)
    OpenCode (165K★, Go, 75+ providers, LSP)
    Pi (45K★, TS, minimal self-extending)
    Claude Code (proprietary, best multi-file refactoring)
    Aider (44K★, Python, git-native, 100+ LLMs)

## 架构总览

NeoTrix: 94K LOC Rust · 373 source files · 46 TUI commands · 17 CLI subcommands · 3451 tests
Desktop: Tauri 2 + React 18 + Zustand + 26 components
CI/CD: 7 workflows · 14 scripts · 1 Dockerfile · 1 Homebrew formula

---

## P0 — 发布拦截器（未解决则不可发布）

### 1. 代码签名 + 平台分发

| # | 问题 | 当前状态 | 差距 | 工作量 |
|---|------|---------|------|--------|
| 1.1 | macOS 签名 | ❌ 未配置 | Developer ID + codesign + notarization | 1-2d |
| 1.2 | Windows 签名 | ❌ 未配置 | EV Code Signing cert + signtool | 1-2d |
| 1.3 | Linux AppImage | ❌ 无 | `tauri build --bundles appimage` | 0.5d |
| 1.4 | macOS .dmg | ❌ 无 | `tauri build --bundles dmg` | 0.5d |
| 1.5 | Windows .msi | ❌ 无 | `tauri build --bundles msi` (需 wix) | 0.5d |
| 1.6 | Homebrew cask (desktop) | ❌ 仅 CLI formula | 增加 `neotrix-desktop` cask | 0.5d |
| 1.7 | apt/yum repo | ❌ 无 | 发布 .deb/.rpm + repo signing | 1d |

### 2. 安全沙箱

| # | 问题 | 差距 | 工作量 |
|---|------|------|--------|
| 2.1 | OS 级内核沙箱 | 无 Seatbelt/Landlock/seccomp。`sandbox` 模块存在但空壳 | 3-5d |
| 2.2 | 网络隔离（默认关） | 无网络权限控制。所有 LLM 调用走用户网络 | 2-3d |
| 2.3 | Read-only 模式 | 无 `--sandbox read-only`。Approval 模式靠用户自觉 | 0.5d |
| 2.4 | Permission profiles | 无命名、可继承的权限配置文件 | 1d |
| 2.5 | Docker sandbox 开箱即用 | 有 `sandbox` feature + `Sandbox` clap command，但无默认沙箱 | 2d |

### 3. CI/CD + 自动化

| # | 问题 | 差距 | 工作量 |
|---|------|------|--------|
| 3.1 | 三平台发布验证 | CI 能 build 但从未实际发布 release | 2d |
| 3.2 | GitHub Action | 无 `neotrix-action`，无法在 CI 中 `neotrix exec` | 1d |
| 3.3 | `neotrix exec` 互操作性 | 有 `Exec` subcommand 但无 JSONL streaming output | 1d |
| 3.4 | Stdin piping | CLI args 有 `--pipe` 标志，但 stdin 集成未验证 | 0.5d |
| 3.5 | CDN hosting for updates | Updater 配置了但无 hosting endpoint | 1d |
| 3.6 | Changelog 自动生成 | `cliff.toml` 存在但未接入 CI | 0.5d |

### 4. 测试置信度

| # | 问题 | 当前状态 | 工作量 |
|---|------|---------|--------|
| 4.1 | E2E smoke test | ❌ 无。`playwright.json.example` 存在但无测试 | 2d |
| 4.2 | Desktop E2E | ❌ 无 Tauri + Playwright 集成测试 | 3d |
| 4.3 | Flaky tests | 2 个已知 flaky（cost_tracker + td_jepa） | 0.5d |
| 4.4 | Test-compile errors | 2 个在 `crypto_agent/gas.rs` + `dex.rs` | 0.5d |
| 4.5 | neotrix-types clippy | 18 个 clippy 警告（`&Vec`→`&[T]`, `.max().min()`→`.clamp()`） | 0.5d |

### 5. 文档

| # | 问题 | 差距 | 工作量 |
|---|------|------|--------|
| 5.1 | 用户文档站 | ❌ 无。只有 README + 散落的 .md | 3-5d |
| 5.2 | 安装教程 | `install.sh` 有但缺少分平台详细说明 | 0.5d |
| 5.3 | API 参考 | HTTP API 无 `/openapi.json` 或 Swagger UI | 1d |
| 5.4 | 迁移/升级指南 | 无 breaking change 说明 | 0.5d |

---

## P1 — 重要能力差距

### CLI 用户体验

| # | 问题 | 差距 | 对标 | 工作量 |
|---|------|------|------|--------|
| 6.1 | `neotrix exec --json` | 无 JSONL streaming | Codex CLI `--output-schema` | 1d |
| 6.2 | `neotrix exec resume` | 无非交互式会话恢复 | Codex CLI `exec resume` | 0.5d |
| 6.3 | `--max-budget-usd` | 有 cost tracking 但无硬预算上限 | Claude Code `--max-budget-usd` | 0.5d |
| 6.4 | `/fork` 分支会话 | 无 | Codex CLI / Pi session trees | 1d |
| 6.5 | Session 持久化 | 内存中，退出后丢失 | Codex JSONL / OpenCode SQLite | 1d |
| 6.6 | `--ephemeral` 模式 | 无 disposable session | Codex CLI `--ephemeral` | 0.5d |
| 6.7 | 命令别名冲突 | `/search` alias `/s` 和 `/stats` alias `/s` 冲突 | 唯一匹配 | 0.25d |
| 6.8 | Tab 补全增强 | 仅默认补全，无上下文感知 | Codex CLI 会补全文件名/命令 | 1d |
| 6.9 | Vim 文本编辑 | 仅 TUI 导航 vim 模式，非文本编辑 | Codex CLI vim text-objects | 1d |
| 6.10 | `/model` 实际持久化 | 只打印提示，不写 config.toml | 运行时生效 + 写入 config | 0.5d |
| 6.11 | 中英文混用 | UI 英文/注释中文/提示中英混杂 | 全英文 | 1d |

### Desktop App

| # | 问题 | 差距 | 工作量 |
|---|------|------|--------|
| 7.1 | 前端 bundle 优化 | 未做 code splitting / lazy loading | 1d |
| 7.2 | 自动更新 UX | Updater configured but no frontend UI | 1d |
| 7.3 | 窗口状态持久化 | 位置/尺寸不保存 | 0.5d |
| 7.4 | 原生文件对话框 | Permission dialog 是文本路径输入而非 OS picker | 0.5d |
| 7.5 | 深色/浅色主题 | 仅有 CLI theme 切换，前端无 toggle | 1d |
| 7.6 | 跨会话搜索 | SearchOverlay 无 session content search | 2d |
| 7.7 | 键盘快捷键自定义 | 硬编码 shortcuts | 1d |
| 7.8 | 前端错误监控 | 无 Sentry/breadcrumb 集成 | 0.5d |
| 7.9 | 前台测试未接 CI | Vitest tests exist but not in workflow | 0.5d |
| 7.10 | 通知系统 | 前端无 desktop notification (仅 CLI 层有) | 1d |

### 生态系统

| # | 问题 | 差距 | 对标 | 工作量 |
|---|------|------|------|--------|
| 8.1 | AGENTS.md 跨工具兼容 | 现有 AGENTS.md 是内部格式，非 Linux Foundation 标准 | Codex/Aider/OpenCode 均支持 | 1d |
| 8.2 | MCP marketplace | `mcp_tools` 模块存在但无 registry | Codex CLI `mcp add/list` | 2d |
| 8.3 | Plugin 系统 | `plugin` 模块空壳 | OpenCode subagent system | 3-5d |
| 8.4 | Skills 分享机制 | `skills/` 目录存在但无仓库/安装 | Codex CLI `.agents/skills/` | 1d |
| 8.5 | LSP 集成 | 有 `lsp` 模块但未接入核心推理 | OpenCode LSP Diagnostic | 3-5d |
| 8.6 | HTTP API rate limiting | `/v1/completions` 无限流 | 常规 | 1d |
| 8.7 | Web search in reasoning | 有 `/search` 命令但未集成到 reasoning pipeline | Codex CLI `--search` | 1d |
| 8.8 | i18n 国际化 | 无多语言支持 | 常规 | 3-5d |

---

## P2 — 增强特性

### 代码质量

| # | 问题 | 工作量 |
|---|------|--------|
| 9.1 | `#![deny(warnings)]` 推广到所有 crate | 0.5d |
| 9.2 | 消除所有 `#[allow(dead_code)]` | 1d |
| 9.3 | 统一错误处理（消除 production `.unwrap()`） | 2d |
| 9.4 | MSRV 策略文档化 | 0.25d |
| 9.5 | OpenAPI spec for HTTP API | 1d |
| 9.6 | 模块级 doc comment 覆盖率 | 2d |
| 9.7 | CHANGELOG.md 维护 | 持续 |
| 9.8 | SECURITY.md 完善（含 responsible disclosure） | 0.5d |

### 性能

| # | 问题 | 工作量 |
|---|------|--------|
| 10.1 | Criterion benchmark 接入 CI | 1d |
| 10.2 | 内存泄漏检测（valgrind/heapprof） | 1d |
| 10.3 | KB 搜索延迟优化（大型 KB 场景） | 1d |
| 10.4 | TUI 渲染性能（大量消息时的帧率） | 1d |
| 10.5 | Desktop 前端首次加载时间优化 | 1d |

### 高级特性

| # | 问题 | 对标 | 工作量 |
|---|------|------|--------|
| 11.1 | Multi-agent 并行（`--agents N`） | Codex 8 concurrent + Guardian | 3-5d |
| 11.2 | Cloud 异步任务 | Codex `codex cloud` | 5d+ |
| 11.3 | Hooks 引擎（session start/stop 等） | Codex hooks engine | 2d |
| 11.4 | 自动 PR review via GitHub Action | Codex `openai/codex-action` | 1d |
| 11.5 | Goal tracking 集成 | Codex achievements/goal mode | 2d |
| 11.6 | 权限配置文件管理（命名继承） | Codex permission profiles | 2d |

---

## 关键数据对比

| 维度 | NeoTrix (当前) | Codex CLI | 差距 |
|------|---------------|-----------|------|
| LOC | 94K Rust + 2K TS | ~200K Rust | 规模相当 |
| Tests | 3451 | ~8000 | 距离较大 |
| E2E | ❌ 无 | Playwright + CI | 无 |
| Sandbox | ❌ 空壳 | Seatbelt + Landlock + seccomp | 严重缺失 |
| CI Release | ❌ 从未发布 | 709 releases | 严重缺失 |
| GitHub Action | ❌ 无 | `openai/codex-action` | 缺失 |
| JSONL streaming | ❌ 无 | `--output-schema` | 缺失 |
| Hard budget | ❌ 软限制 | Claude Code $上限 | 缺失 |
| Session tree | ❌ 无 | Pi session trees | 缺失 |
| LSP | 模块存在 | OpenCode已集成 | 未接入 |
| MCP server listing | ❌ 无 | Codex `mcp add/list` | 缺失 |
| macOS sign | ❌ 无 | 已签名 | 发布阻塞 |
| 文档站 | ❌ 无 | docs.codex.ai | 严重缺失 |
| Plugin system | 空壳 | Pi full extensions | 未实现 |

---

## 发布刻线（建议）

### Release v0.19.0 — "最小安全发布" (P0 only, 2-3 weeks)
- [ ] macOS 代码签名 + notarization
- [ ] Windows 代码签名
- [ ] Linux AppImage
- [ ] 3 平台 E2E smoke test
- [ ] Read-only sandbox 模式
- [ ] `--max-budget-usd` 硬上限
- [ ] 2 flaky tests 修复
- [ ] 2 crypto_agent 编译错误修复
- [ ] 18 neotrix-types clippy 修复
- [ ] 安装文档 + 快速开始

### v0.20.0 — "CI/CD 就绪" (P0+P1, 4-5 weeks)
- [ ] GitHub Action (`neotrix-action`)
- [ ] `neotrix exec --json` JSONL streaming
- [ ] Session 持久化
- [ ] `/fork` 分支会话
- [ ] `/model` 实际持久化
- [ ] Desktop 自动更新 UX
- [ ] 前端 bundle 优化
- [ ] 原生文件对话框
- [ ] Homebrew cask
- [ ] 文档站 MVP

### v0.21.0 — "生产可用" (P0+P1+P2, 6-8 weeks)
- [ ] Docker sandbox (开箱即用)
- [ ] OS 级沙箱 (macOS Seatbelt)
- [ ] Multi-agent `--agents N`
- [ ] Cloud 异步任务
- [ ] MCP marketplace
- [ ] LSP 集成
- [ ] AGENTS.md 跨工具标准
- [ ] Changelog + release automation
- [ ] 性能测试 baseline

---

## 快速修复项（可立即解决）

这些小问题不需要架构决策，可以随时 PR：

1. `/search` alias `/s` → `/stats` alias `/s` 冲突 — 改一个别名（0.25h）
2. `suggest` 字段 unused warning — `main.rs:237`（5min）
3. `#[allow(dead_code)]` 消除 — `engine_core.rs` / `monitor.rs` / `security.rs`（30min）
4. 2 crypto_agent 编译错误 — `gas.rs` borrow / `dex.rs` u128（30min）
5. neotrix-types 18 clippy — 已接近完成（30min）
6. `test_pipeline_stages_order` 已修复（已完成）
7. `notification.rs` + `doctor_cmds.rs` + `mention.rs` 已创建（已完成）
8. `/model` 命令已创建（已完成）
