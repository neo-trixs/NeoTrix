# NeoTrix 深度对标分析报告

> 生成: 2026-06-01 | 版本: v0.18.0 | 测试: 3582 passed | 编译: `--lib` `--features full --lib` `--bin neotrix` 全 clean

## 市场定位

| 维度 | NeoTrix | Codex CLI v0.131 | OpenCode v1.15.6 | Aider v0.86 | Claude Code 2.1x |
|------|---------|-------------------|------------------|-------------|-------------------|
| 语言 | Rust | Rust | Go | Python | TypeScript |
| GitHub Stars | 私有 | 87K | 167K | 45.5K | 闭源 |
| 发布次数 | **0** | 805+ | 784+ | 13K+ commits | 296+ |
| 许可证 | Apache 2.0 | MIT | MIT | Apache 2.0 | 闭源 |
| 模型支持 | OpenAI 兼容 | OpenAI only | 75+ providers | 100+ LLMs | Claude only |
| 桌面端 | Tauri + React | 原生 macOS | 跨平台 beta | ❌ | 原生 macOS/Win |
| LOC | 236K Rust + 7.7K TS | ~200K Rust | ~150K Go/TS | ~100K Python | N/A |

**核心差异化**: E8 推理引擎、SEAL 自迭代、VSA HyperCube、GWT 意识架构 — 这些在 80+ 竞品中独一无二。

---

## 发现的可发布差距

### 1. 发布/分发 (Blocked by External Dependencies)

| # | 差距 | 影响 | 参考 (竞品) | 修复 |
|---|------|------|-------------|------|
| **1.1** | **0 个 GitHub Release** | 无法发布。tag v0.18.0 存在但无 artifacts | Codex: 805 releases; OpenCode: 784+ | 创建首个 release |
| **1.2** | macOS 代码签名 + notarization ❌ | 用户收到 GateKeeper 警告 | Codex: 已签名 | Apple Developer $99/yr |
| **1.3** | Windows EV 签名 ❌ | SmartScreen 拦截下载 | Codex: 已签名 | EV 证书 $300+/yr |
| **1.4** | Linux AppImage/Deb/RPM ❌ | Linux 无便捷安装 | Codex: deb + rpm | 构建配置 |
| **1.5** | Homebrew formula SHA256 占位符 | `scripts/neotrix.rb` 所有 SHA 为 `0000...` | Codex: 官方 tap | 发布时填充 |
| **1.6** | Desktop Homebrew cask ❌ | 无 `brew install --cask neotrix` | — | 新增 cask |
| **1.7** | CDN hosting for auto-updater ❌ | Updater 配置了但无 endpoint | Codex: S3/CloudFront | 部署基础设施 |
| **1.8** | 安装脚本单一入口 ❌ | 3 种不同安装方式散落，无统一入口 | OpenCode: `curl -fsSL https://opencode.ai/install \| bash` | 统一安装脚本 |

### 2. 安全/信任 (Need Engineering Work)

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **2.1** | OS 级内核沙箱 (Seatbelt/Landlock/seccomp) | **P0** | Codex: SSH + Windows + 网络代理 allowlist | `sandbox` feature 存在但空壳 |
| **2.2** | 网络隔离 + LLM 代理策略 | **P0** | Codex: 网络代理 allowlist | 无网络权限控制 |
| **2.3** | Read-only sandbox 不完全 | **P0** | — | 硬编码命令列表，非通用 |
| **2.4** | SECURITY.md PGP 密钥占位符 | P2 | 应有真实 PGP | `0xDEADBEEF` 占位符 |
| **2.5** | 无 SBOM 发布检查 | P2 | — | `generate-sbom.sh` 存在但未集成 |
| **2.6** | 无漏洞披露流程验证 | P2 | — | SECURITY.md 描述但无自动化 |

### 3. CLI 体验

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **3.1** | 命令描述全中文硬编码 | **P1** | 所有竞品英文 | `/help` 输出 30 条中文命令 |
| **3.2** | 别名冲突: `/s` = /stats 和 /search | **P1** | — | `RELEASE_CHECKLIST.md` 已记录，未修复 |
| **3.3** | 无分层帮助、无单命令 `--help` | **P1** | Codex: `/help <cmd>` | 只有硬编码一级帮助 |
| **3.4** | `/config` 是硬编码 stub | P2 | OpenCode: 真实 config 读写 | 输出 fake JSON |
| **3.5** | `/model` 是 stub | P2 | OpenCode: 75+ providers | 不持久化到 config |
| **3.6** | Shell 补全不完整 | P2 | Codex: 完整补全 | 仅 12 个命令 + `--json` |
| **3.7** | 无进度指示器 (长操作) | P2 | Codex: spinner + progress bar | 阻塞时无反馈 |
| **3.8** | 无颜色方案配置 | P2 | Codex: theme-aware status line | 硬编码 ANSI 颜色 |
| **3.9** | TUI tab 补全无上下文感知 | P2 | Codex: @ picker 补全文件/目录/插件 | 仅命令补全 |
| **3.10** | `/review` 无代码差异审查 | P2 | Claude Code: `/code-review --fix` | 仅结构输出 |

### 4. 桌面端 (Tauri)

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **4.1** | 无代码分割/懒加载 | **P1** | — | 单 JS bundle 包含全部 28 组件 |
| **4.2** | 自动更新无下载 UX | **P1** | Codex: 原生更新 UI | `check()` 调用存在但无进度条 |
| **4.3** | 无桌面端 E2E 测试 | **P1** | Codex: Playwright Tauri 测试 | 仅 4 个 CLI 冒烟测试 |
| **4.4** | 权限对话框是文本输入 | P2 | 原生 OS 文件选择器 | `PermissionDialog` 用文本路径 |
| **4.5** | 无原生 OS 通知 | P2 | Codex: macOS Notification Center | 仅应用内 toast |
| **4.6** | 无 Sentry/错误监控 | P2 | Codex: Sentry | 错误日志仅 stderr |
| **4.7** | 无深链接支持 | P2 | OpenCode: `opencode://` 协议 | 未使用 `@tauri-apps/plugin-deep-link` |
| **4.8** | 无性能优化 (memo/windowing) | P2 | — | 长会话列表无虚拟化 |

### 5. 测试/QA

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **5.1** | 无覆盖率测量工具 | **P1** | — | AGENTS.md 要求 80% 但无工具 |
| **5.2** | Benchmark 未集成 CI | P2 | Codex: 性能回归测试 | Criterion bench 存在但不运行 |
| **5.3** | 前端测试未接入 CI | **P1** | — | `vitest` 配置了但不执行 |
| **5.4** | 无模糊测试/属性测试 | P2 | — | 只有单元测试 |
| **5.5** | 2 个 flaky 测试未标记 | P2 | — | 偶发失败但无 retry 逻辑 |
| **5.6** | 无桌面端 E2E 测试 | **P1** | Codex: Playwright Tauri | 无 Tauri + Playwright 集成 |
| **5.7** | 无性能基准线 | P2 | Codex: CI 中运行 bench | 不知性能变化 |
| **5.8** | 回归测试集合不完整 | P2 | — | 无针对已修复 bug 的回归测试 |

### 6. 生态系统

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **6.1** | 插件系统空壳 | **P0** | Codex: 插件市场 + skill hub | `plugin/` 模块为空 |
| **6.2** | 无 MCP 市场/注册中心 | **P1** | Cursor: Team Marketplace; Codex: plugin hub | MCP server 有但无发现机制 |
| **6.3** | 无多 Provider 开箱 | **P1** | OpenCode: 75+ | 仅 OpenAI 兼容 |
| **6.4** | 无 GitHub Action | **P1** | Codex: `codex ci` | 无 CI 内调用入口 |
| **6.5** | 技能集市 | P2 | Codex: skill hub | 本地技能目录但无分享 |

### 7. API/集成

| # | 差距 | 严重度 | 参考 (竞品) | 当前状态 |
|---|------|--------|-------------|---------|
| **7.1** | 无 OpenAPI/Swagger 规范 | **P1** | OpenCode: v2 HTTP API + OpenAPI | HTTP API 存在但无文档 |
| **7.2** | JSONL streaming 不完整 | P2 | OpenCode: 结构化 JSONL | `exec` 无 streaming 输出 |
| **7.3** | WebSocket API 未版本化 | P2 | — | `ws://` 端点无版本前缀 |
| **7.4** | 无 REST API 认证 | **P1** | OpenCode: token-based auth | `--serve` 启动无验证 |

### 8. 代码质量

| # | 差距 | 严重度 | 当前状态 |
|---|------|--------|---------|
| **8.1** | neotrix-types 18 个 clippy 警告 | **P1** | 未解决 |
| **8.2** | 示例代码 4 个警告 | P2 | 未解决 |
| **8.3** | 2 个 test 编译错误 (crypto_agent) | **P1** | `gas.rs` + `dex.rs` |
| **8.4** | 19 个 binary target — 部分仅内部使用 | P2 | Cargo.toml 定义过多 |
| **8.5** | 中文混合英文: 注释中文，UI 中文 | P2 | 国际化一律用中文 |
| **8.6** | README 测试数量过期 (1715 vs 实际 3582) | P2 | 需同步 |
| **8.7** | README SEAL 阶段数不统一 (16 vs 26-27) | P2 | 需同步 |
| **8.8** | 无 ADR (架构决策记录) | P2 | 历史决策无记录 |

### 9. 文档

| # | 差距 | 严重度 | 当前状态 |
|---|------|--------|---------|
| **9.1** | 文档站未部署 | **P1** | `.vitepress/` 存在但无产出 |
| **9.2** | 无 API 引用文档 | **P1** | Rustdoc 存在但不部署 |
| **9.3** | 无 man page | P2 | 无 `neotrix.1` |
| **9.4** | 无 Quickstart 视频/GIF | P2 | README 纯文本 |
| **9.5** | SECURITY.md PGP 占位符 | P2 | `0xDEADBEEF` |
| **9.6** | 无贡献者指南网站 | P2 | `CONTRIBUTING.md` 文本 |

### 10. 高级特性 (vs 竞品)

| # | 差距 | 参考 (竞品) | 严重度 |
|---|------|-------------|--------|
| **10.1** | 无并行 Agent (`--agents N`) | Codex: 并行 threads; Claude: subagents | P2 |
| **10.2** | 无背景持久化 Agent | Claude Code: `/bg`; Codex: remote | P2 |
| **10.3** | 无云异步任务提交 | OpenCode: cloud submit; Codex: cloud sandbox | P2 |
| **10.4** | 无 Hook 系统完整接入 (PreToolUse 有但少用) | Codex: hooks 深度集成 | P2 |
| **10.5** | 无 Git worktree 自动管理 | Claude Code: `EnterWorktree` | P2 |
| **10.6** | 无 Agent teams/多角色协作 | OpenCode: plan + code 双 agent | P2 |
| **10.7** | 无 Computer use | Claude Code: computer use; Devin: sandbox VM | P2 |
| **10.8** | 无 VS Code/JetBrains 扩展 | 所有竞品均有 | P2 |

---

## 核心任务 TODO

### 第一优先级 (发布前完成)

```
P0-MUST (安全/发布/生态):
  □ release: 创建 GitHub Release v0.19.0-rc1 (无签名标记 pre-release)
  □ sandbox: 实现 OS 内核沙箱 (至少 macOS Seatbelt + Linux Landlock)
  □ plugin: 插件系统基础框架 (discovery + 简单生命周期)
  □ network: 网络隔离策略 (默认只允许 LLM API)
  □ read-only: 通用文件系统门控 (非硬编码列表)

P1-HIGH (体验/可信度/CI):
  □ cli: `/help` 英文化 + 分层帮助 (`/help <cmd>`)
  □ cli: 修复 `/s` 别名冲突 (/search → `/sr`)
  □ cli: `/config` 真实读写 config.toml
  □ cli: `/model` 真实持久化
  □ desktop: 代码分割 + 懒加载 (React.lazy + Suspense)
  □ desktop: 自动更新下载 UX (进度条 + 确认对话框)
  □ desktop: E2E 测试 (Playwright + Tauri driver)
  □ test: 接入覆盖率测量 (tarpaulin/llvm-cov)
  □ test: 前端测试接入 CI (vitest + c8)
  □ eco: 多 Provider 配置 (Claude + Gemini + DeepSeek)
  □ eco: GitHub Action (`neotrix-action`)
  □ api: OpenAPI 规范 (至少 HTTP API 文档)
  □ docs: 部署文档站 (VitePress → GitHub Pages/Cloudflare)

P2-NICE (锦上添花):
  □ cli: `--progress` spinner for long operations
  □ cli: Tab 补全上下文感知 (文件/会话)
  □ cli: 颜色方案可配置
  □ desktop: Sentry 错误监控
  □ desktop: 原生 OS 通知
  □ desktop: 深链接 `neotrix://` 协议
  □ test: Benchmark CI 集成
  □ test: 模糊测试 (fuzz targets)
  □ eco: MCP 注册中心 (社区共享)
  □ code: 清理 18 clippy warnings
  □ code: 修复 2 test 编译错误
  □ code: ADR 文档

Timeline: P0 = v0.19.0-rc1 (1-2周) | P1 = v0.20.0 (3-4周) | P2 = v0.21.0 (6-8周)
```

### 当前状态统计

```
ℹ️  编译:   --lib ✅ | --features full --lib ✅ | --bin neotrix ✅
ℹ️  测试:   3582 passed, 0 failed
ℹ️  已知:   18 clippy + 2 test-compile errors + 2 flaky tests (pre-existing)
ℹ️  发布:   0 releases, 0 installs outside of development
ℹ️  安全:   OS 沙箱 ❌ | 代码签名 ❌ | 插件隔离 ❌ | 网络隔离 ❌
ℹ️  文档:   文档站 ❌ | OpenAPI ❌ | man page ❌ | ADR ❌
ℹ️  桌面:   28 组件, 无代码分割 ❌ | 自动更新无 UX ❌ | 无 E2E ❌
