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
| [trending](trending/) | 热门项目模式 | `skills/trending/` |
| [architecture-auditor](architecture-auditor/) | 架构审计 + 工程技能 | `skills/architecture-auditor/` |
| [docs-architecture](docs-architecture/) | 架构文档 + 重构方案 | `skills/docs-architecture/` |

## Core Code Directories (Not Skills)

| Directory | Purpose | Maintained By |
|-----------|---------|---------------|
| `neotrix-core/` | 核心库 | Rust 开发者 |
| `src-tauri/` | Tauri 桌面应用 | Rust 开发者 |
| `crates/` | Rust 包 | Rust 开发者 |

## Architecture Principles

1. **单一事实源**: 所有定义在 `skills/` 目录
2. **发现能力**: Agent 可通过 skill 系统自动发现
3. **向后兼容**: 原始目录保持可用
4. **文档完整**: 每个技能都有 SKILL.md
5. **易于扩展**: 新工具只需添加 skill 目录

## Usage

Agent 可通过 skill 系统发现和加载项目定义，无需手动查找目录。
