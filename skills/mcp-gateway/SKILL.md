---
name: mcp-gateway
description: 占位描述（PLACEHOLDER，零实现）—— 见下方「状态」段
when_to_use: 暂勿使用
disable-model-invocation: true   # 占位技能，禁止模型自动触发
---

> **状态：PLACEHOLDER（2026-09-28 实测）**
>
> - 仓内**消费者 0 处**、目录内**只有本文件**（无 references/scripts/tests）、
>   最后提交 2026-07-06（84 天前，批量 checkpoint "Cycle 33 cleanup"）。
> - **不承担职责** —— 真实实现是 **`crates/neotrix-gateway`**（8 个 .rs，workspace member，
L5 定位：model routing / skill registry / search）。本文件是**早期占位描述**，
不应被当作该 crate 的文档。
> - 已 `disable-model-invocation: true`，模型不会自动加载它。
> - 处置：**保留不删**。它是「待建设」而非「错误资产」，且删除不可逆；
>   真要清理须同步改 `skills/index.json` + `skill_loader` + `check-skill-gate.sh`。
>   依据 AGENTS.md「导出 ≠ 调用」：0 消费者是删除的必要条件，不是充分条件。

# MCP Gateway Skill

## Description
Unified MCP server gateway with 1MCP runtime integration. Aggregates multiple MCP servers behind a single interface.

## Skill Type
automation

## Tags
- mcp
- gateway
- 1mcp
- aggregation
- tools

## Usage

This skill provides MCP Gateway capabilities:

1. **Server Aggregation**: Combine multiple MCP servers into one
2. **Tool Discovery**: List and query tools from all connected servers
3. **1MCP Integration**: Compatible with 1MCP runtime configuration

## Configuration

The gateway supports 1MCP-style configuration:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/workspace"]
    },
    "git": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"]
    }
  }
}
```

## Built-in Tools

- `gateway_status`: Get MCP Gateway status
- `list_servers`: List all configured MCP servers
- `brain_capability`: Query ReasoningBrain capability vector

## Activation Triggers

- User requests MCP server integration
- Task requires multiple external tools
- Need to aggregate MCP servers
- Working with 1MCP runtime

## Examples

### List Available Tools
```
User: What tools are available through the MCP gateway?
Assistant: [Uses gateway_status to list tools]
```

### Configure New Server
```
User: Add a filesystem MCP server for /tmp
Assistant: [Uses add_server to configure new MCP server]
```

## Notes

This skill integrates with the ReasoningBrain system. MCP servers can be dynamically added and removed based on task requirements.
