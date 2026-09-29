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
| `docs-architecture/` | ~~架构文档 + 重构方案（v0.21 快照）~~ ✅ **2026-09-29 删除**（已被 `docs/architecture/` 取代） | — |
| [external-absorption](external-absorption/) | 外部仓库功能吸收工作流 | `skills/external-absorption/` |
| [research-absorption](research-absorption/) | 长循环研究吸收流水线 | `skills/research-absorption/` |
| [self-iteration-agent](self-iteration-agent/) | 全栈自迭代协议 | `skills/self-iteration-agent/` |
| [self-health](self-health/) | 6 维自健康协议（命令已迁 `scripts/ops/*`，本文件留协议） | `skills/self-health/` |
| [productivity](productivity/) | 表达效率技能 | `skills/productivity/` |
| [root](root/) | 根级技能（如 `design-patrol`） | `skills/root/` |

### 2026-09-29 清理：删 5 个目录

| 已删 | 依据 |
|---|---|
| `code-expert/` `law-expert/` | 作者自标「⚠️ 占位，零实现」；实体仅 1 个 SKILL.md，只有触发词与示例，无脚本无实现 |
| `mcp-gateway/` | 自标「⚠️ 占位」。**幻觉文档**：其 `## Built-in Tools` 列的 `gateway_status` / `add_server` / `list_servers` 在全仓 `.rs` 里零命中（`list_servers` 的两处命中是别的东西）。留着会诱导模型调用不存在的 API |
| `trending/` | 5 张模式卡（139 行）全无 `## Usage`/`## Script` 段，纯元数据。其中 `memory-engine` 卡出处 `supermemory` 的引擎**闭源**（`ABSORPTION-AGENT-ARCH2-2026-09-29.md` §证伪前提已判定「只可吸收外围接线」） |
| `docs-architecture/` | v0.21 快照（1112 行），作者自标「已被 `docs/architecture/` 取代」 |

⛔ 未登记 ≠ 该删：`self-health/` `self-iteration-agent/` `external-absorption/`
**故意不进 `index.json`** —— `neotrix-core/src/skill_loader.rs:97-106` 专门
解释了：前两个**必须能被模型自动触发**（价值就在于被触发），而
`external-absorption` 会改 KB、不应被随手唤起。`SkillInvocationPolicy` 的
`model_invocable` / `user_invocable` 两个正交维度就是为此设计。

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
