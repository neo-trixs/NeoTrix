# Agent Instructions

## Purpose
Agent 行为指令和路由规则

## Trigger Words
- agents
- agent instructions
- agent routing
- agent 行为
- agent 指令

## Content
- Agent 行为规范
- 技能路由规则
- 吸收协议
- 指针守恒规则
- 写入门禁机制

## File
`AGENTS.md`

## Key Rules
1. 统一吸收协议: 会话结束必须执行 experience-tree 五阶段吸收
2. 指针守恒: AGENTS.md 禁止追加 cycle 正文
3. 写入门禁: 经验存 KB，AGENTS.md 禁止内联
4. 外部文件惰性加载: 按需读取，不预加载

## Skill Routing
| 任务 | 加载 |
|------|------|
| review | rev-officer-agent.md |
| 吸收外部仓库 | external-absorption/SKILL.md |
| 会话收尾 | experience-tree/SKILL.md |
| 探索代码库 | codebase-exploration |
