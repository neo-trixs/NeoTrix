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
