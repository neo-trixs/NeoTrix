# NeoTrix — 底层规则 (Infrastructure)

> 这些规则由项目架构决定，与人格无关。几乎不变。
> 工具链、CI、构建系统自动读取。

## 项目结构

- **Workspace**: `/Volumes/neotrix/neotrix`
- **语言**: Rust edition 2021
- **安全**: `#![forbid(unsafe_code)]` in core crates
- **警告**: `#![deny(warnings)]` in lib.rs — test-only unused code triggers compile error

## 测试

- **命令**: `cargo test -p neotrix --lib`
- **测试总数**: 4235 passed, 0 failed (2026-06-08)

## 命名规范

- 格式: `nt_{domain}_{subsystem}`. No generic names.
- 架构: 7 domains → CORE / MIND / MEMORY / WORLD / ACT / SHIELD / IO

## 构建

- `[profile.release]`: LTO fat, codegen-units=1, opt-level=z, strip=symbols, panic=abort

## 频率调度

| Stage | Frequency |
|-------|-----------|
| Ingestion | 3 |
| VsaFingerprint | 1 |
| CanonicalSort | 5 |
| StreamHygiene | 3 |
| StormBreaker | 2 |
| Compaction | 3 |
| MetaImprovement | 10 |
| DMN | 10 |
| IntrinsicReward | 3 |
| Forgetting | 7 |
| CapabilitiesLog | 20 |
| SelfPreservation | 7 |

## VSA 维度

- 4096, 8-bit 量化 (目标)

## 拒绝设计
- 不向用户暴露 CLI 命令来控制意识子系统
- 依赖 MCP/LangChain/OpenAI Functions 等外部协议框架 — 所有功能用原生能力构建

## 设计原则

- **自身原生优先**: 不依赖外部协议框架。内部通信唯一协议 = VSA 向量。
- **吸收核心架构而非产品**: 外部信息只吸收核心架构逻辑, 不复刻功能/界面。
- **VSA 是唯一协议**: 子系统间调用、Agent通信全部通过 4096-bit VSA 向量。

## KB 状态

- ~59,725 节点 / ~240,710 边 / 20 域 (2026-06-08)
