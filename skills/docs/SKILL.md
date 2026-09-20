# Documentation Skills

## Purpose
文档技能集合：项目文档的统一管理

## Available Skills

| Skill | Purpose | File |
|-------|---------|------|
| [project-overview](project-overview/) | 项目概述和入门指南 | `README.md` |
| [domain-language](domain-language/) | 领域术语和共享语言 | `CONTEXT.md` |
| [development-rules](development-rules/) | 开发规则和纪律 | `dev-rules.md` |
| [contribution](contribution/) | 贡献指南和开发流程 | `CONTRIBUTING.md` |
| [security](security/) | 安全策略和漏洞报告 | `SECURITY.md` |
| [changelog](changelog/) | 版本历史和发布记录 | `CHANGELOG.md` |
| [agent-instructions](agent-instructions/) | Agent 行为指令和路由 | `AGENTS.md` |
| [license-exceptions](license-exceptions/) | 专有模块许可例外 | `LICENSE-EXCEPTIONS.md` |
| [todo](todo/) | 任务列表和项目管理 | `TODO.md` |

## Architecture
- 每个文档对应一个 skill 目录
- SKILL.md 提供文档的元数据和使用说明
- 原始文档保留在根目录（标准位置）
- Skill 系统提供发现和路由能力

## Usage
Agent 可通过 skill 系统发现和加载文档，无需手动查找根目录文件。
