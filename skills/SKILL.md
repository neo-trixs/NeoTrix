# NeoTrix Skills Architecture

## Purpose
统一技能架构：所有项目定义和实现的单一事实源

## Skill Categories

| Category | Purpose | Location |
|----------|---------|----------|
| [config](config/) | 配置文件管理 | `config/` |
| [dev-tools](dev-tools/) | 开发工具 | `skills/dev-tools/` |
| [docs](docs/) | 项目文档 | 根目录 MD 文件 |
| [assets](assets/) | 静态资源 | `assets/` |
| [deploy](deploy/) | 部署配置 | `deploy/` |
| [design](design/) | 前端设计 + UI 设计 | `design/` |
| [e2e](e2e/) | 端到端测试 | `e2e/` |
| [crates](crates/) | Rust 包管理 | `crates/` |
| [architecture-auditor](architecture-auditor/) | 架构审计 + 工程技能 | `skills/architecture-auditor/` |
| [docs-architecture](docs-architecture/) | 架构文档 + 重构方案（v0.21 快照，已被 `docs/architecture/` 取代） | `skills/docs-architecture/` |
| [external-absorption](external-absorption/) | 外部仓库功能吸收工作流 | `skills/external-absorption/` |
| [research-absorption](research-absorption/) | 长循环研究吸收流水线 | `skills/research-absorption/` |
| [self-iteration-agent](self-iteration-agent/) | 全栈自迭代协议 | `skills/self-iteration-agent/` |
| [self-health](self-health/) | 5 维自健康协议（命令已被 `scripts/ops/*` 取代） | `skills/self-health/` |
| [productivity](productivity/) | 表达效率技能 | `skills/productivity/` |
| [root](root/) | 根级技能（如 `design-patrol`） | `skills/root/` |
| [code-expert](code-expert/) | ⚠️ **占位**，零实现 | `skills/code-expert/` |
| [law-expert](law-expert/) | ⚠️ **占位**，零实现 | `skills/law-expert/` |
| [mcp-gateway](mcp-gateway/) | ⚠️ **占位**，真实实现见 `crates/neotrix-gateway` | `skills/mcp-gateway/` |
| [trending](trending/) | ⚠️ 5 个模式卡均已被 L1/L3/L5 的 Rust 模块取代 | `skills/trending/` |

## Core Code Directories (Not Skills)

| Directory | Purpose | Maintained By |
|-----------|---------|---------------|
| `neotrix-core/` | 核心库（L0–L6 分层） | Rust 开发者 |
| `crates/` | Rust 包（workspace member） | Rust 开发者 |
| `scripts/` | 门禁 + 运维脚本（**不是 skill**） | 见 `.neotrix/task-index.json` |
| `docs/` | 架构正典 | — |

> 2026-09-28 修正两处：
> - **`src-tauri/` 已删除**（桌面端随 5c02e738 归档 599 files，现由
>   `crates/neotrix-neobot` 承接）⇒ 移出本表。
> - **`crates/` 本在上表「skill 目录」里也列过**（`skills/crates/` 是指向
>   顶层 `crates/` 的**文档卡**，非第二套实现）⇒ 现只在此表列一次。

## Architecture Principles

1. **单一事实源**: 所有定义在 `skills/` 目录
2. **发现能力**: Agent 可通过 skill 系统自动发现
3. **向后兼容**: 原始目录保持可用
4. **文档完整**: 每个技能都有 SKILL.md
5. **易于扩展**: 新工具只需添加 skill 目录

## Usage

Agent 可通过 skill 系统发现和加载项目定义，无需手动查找目录。
