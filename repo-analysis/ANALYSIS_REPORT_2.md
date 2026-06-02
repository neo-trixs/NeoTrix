# 三项目分析与对比（第二批）

## 1. AdStrike — AD 红队框架
- **12 stars** | Python | 101 commits
- 56 个模块菜单、9 阶段 kill-chain、Kerberos/NTLM 工作流
- AI Agent（Ollama/Claude 后端）辅助规划
- 模块化插件架构 + Session Manager 跨模块共享状态
- **与 NeoTrix 关联度**: 低（AD 安全 vs Agent 推理，Python vs Rust）
- **可吸收**: 模块注册模式（但 NeoTrix 已有 plugin system）

## 2. mangofetch — Rust 下载引擎
- **31 stars** | Rust（2581 commits）| GPL-3.0
- 工作空间拆分：`mangofetch-core`(SDK) / `mangofetch-cli`(TUI) / `mangofetch-plugin-sdk`
- ratatui TUI + 11 主题 + 鼠标支持 + vim 快捷键
- 核心引擎使用 Tokio async + MPSC channel 进度报告
- 有 `AGENTS.md` 驱动开发
- **与 NeoTrix 关联度**: 中（同 Rust + ratatui + 异步引擎）
- **可吸收**: `core/cli/sdk` 工作空间拆分模式、AGENTS.md 驱动、TUI 主题系统

## 3. witr — 进程因果链分析（重点 ⭐）
- **15.3k stars** | Go | Apache-2.0 | 462 commits | 19 releases
- 核心回答："Why is this running?" → 构建 PID 因果链
- 跨平台（Linux/macOS/Windows/FreeBSD）单静态二进制
- 输出模式：标准/短链/树状/JSON/详细/警告
- Shell 补全：`witr completion bash|zsh|fish|powershell`
- TUI 模式：实时代仪表盘 + 鼠标支持
- 退出码约定：0=clean / 1=warnings / 2=not found / 3=permission / 4=invalid
- 8 种包管理器分发
- **与 NeoTrix 关联度**: 中高（"causal chain" 映射到 ReasoningTrace）
- **可吸收**: 退出码约定、--json 输出、shell 补全、单二进制分发、TUI 仪表盘

## 与 NeoTrix 对比矩阵

| 功能 | AdStrike | mangofetch | witr | NeoTrix |
|------|----------|------------|------|---------|
| 生态位 | AD 安全 | 媒体下载 | 进程诊断 | Agent 推理 |
| 因果链 | ❌ | ❌ | ✅ 核心 | ⚠️ ReasoningTrace |
| TUI | ❌(text menu) | ✅ ratatui | ✅ ratatui | ✅ ratatui |
| Shell 补全 | ❌ | ❌ | ✅ 4 shell | ❌ |
| `--json` | ❌ | ❌ | ✅ | ❌ |
| 退出码约定 | ❌ | ❌ | ✅ (5级) | ❌ |
| 单二进制 | ❌ | ⚠️ | ✅ | ⚠️ Tauri |
| 跨平台 | ✅ Linux | ✅ 3平台 | ✅ 4+平台 | ⚠️ mac/linux |
| 包管理器 | ❌ | ❌ crates.io | ✅ 8个 | ❌ |
| AGENTS.md | ❌ | ✅ | ❌ | ✅ |

## 可吸收建议

### P1: 吸收 witr 的退出码约定
```rust
// 当前 NeoTrix CLI：返回 Result<String, String>
// 改为：统一退出码
pub enum ExitCode {
    Success = 0,
    Warning = 1,
    NotFound = 2,
    PermissionDenied = 3,
    InvalidInput = 4,
    InternalError = 5,
}
```

### P1: CLI 增加 --json 输出模式
所有命令（stats / absorb / status / list）增加 `--json` 标志，对标 OpenSpec 双模式。

### P2: Shell 补全生成
对标 witr 的 `witr completion bash|zsh|fish|powershell`。NeoTrix 使用自定义 `CliCommand` trait，可以加 `generate_completions()` 方法。

### P2: 单二进制分发
NeoTrix 有 Tauri 桌面端但缺 CLI-only 静态二进制。对标 witr 的 goreleaser.yml 每周构建 + 8个包管理器分发。
