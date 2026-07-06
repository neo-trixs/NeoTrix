# MCP (Model Context Protocol) 2026 Ecosystem Overview

## Search Results Summary

| Name | URL | Description | Type | Relevance |
|------|-----|-------------|------|-----------|
| **2026-07-28 Spec Release Candidate** | https://blog.modelcontextprotocol.io/posts/2026-07-28-release-candidate/ | Largest protocol revision since launch — stateless core, removes initialize handshake & session ID, Streamable HTTP becomes standard, MCP Apps (SEP-1865) for server-rendered UIs, Tasks extension for long-running work, OAuth-aligned auth | Specification | ★★★★★ Spec v2 era |
| **MCP 2026 Roadmap** | https://modelcontextprotocol.io/development/roadmap | Four priority areas: Transport Evolution/Streamable HTTP stateless, Agent Communication, Governance Maturation (Linux Foundation AAIF), Enterprise Readiness (audit trails, SSO, gateway patterns) | Specification | ★★★★★ |
| **Official Rust SDK (rmcp)** | https://github.com/modelcontextprotocol/rust-sdk | Official MCP Rust SDK with tokio async runtime, 3.4k★, 78 releases (rmcp v1.7.0). Streamable HTTP, OAuth 2.1, transport-agnostic, `#[tool]` macros | SDK | ★★★★★ |
| **rmcp crate (crates.io)** | https://crates.io/crates/rmcp | 10.5M+ total downloads, 6.6M in 90d. Server/client features, transport-io, transport-streamable-http, auth, elicitation. Mature production Rust SDK | SDK | ★★★★★ |
| **rust-mcp-sdk** | https://github.com/rust-mcp-stack/rust-mcp-sdk | High-performance async Rust toolkit, 165★. Supports latest MCP 2025-11-25 protocol, Streamable HTTP + SSE, multi-client concurrency, OAuth, health checks, telemetry | SDK | ★★★★☆ |
| **FastMCP Rust** | https://github.com/Dicklesworthstone/fastmcp_rust | Rust port of Python fastmcp, cancel-correct async, zero-copy serialization, `#[tool]` attribute macros, structured concurrency, budget-based timeouts | SDK | ★★★☆☆ Niche alternative |
| **mcpkit** | https://docs.rs/mcpkit | Typestate-pattern MCP SDK with unified `#[mcp_server]` macro, runtime-agnostic, miette diagnostics, full 2025-11-25 coverage | SDK | ★★★☆☆ |
| **codebase-mcp** | https://github.com/ndhkaeru/codebase-mcp | Local-first MCP server with 37 tools — AST-aware code intelligence (Rust/Python/TS/JS), git inspect, SQLite inspection, safe write with undo/redo. Works with Cursor, Claude Desktop, VS Code | Server/Tool | ★★★★★ Code intelligence |
| **Pathfinder (Headless IDE)** | https://github.com/irahardianto/pathfinder | Rust MCP server with Tree-sitter AST parsing + LSP validation, 18 tools for symbol-level code navigation/editing, 7 languages, sandbox security | Server/Tool | ★★★★★ Rust-native IDE |
| **agent-tool** | https://github.com/knewstimek/agent-tool | MCP tool server for AI coding agents — encoding-aware Edit/Read/Write, SSH, SFTP, process management, FindTools. Works with Claude Code, Codex CLI, Cursor, Windsurf | Server/Tool | ★★★★☆ Remote ops |
| **mcp-devtools** | https://github.com/marin1321/mcp-devtools | 14 tools + 3 resources + 4 prompts for filesystem, databases, processes, OpenAPI. Two transports (stdio + HTTP w/ auth), audit log, plugin API | Server/Tool | ★★★★☆ |
| **Krusch Context MCP** | https://github.com/kruschdev/krusch-context-mcp | Zero-Trust MCP server with semantic codebase search, episodic project memory (persistent across sessions), framework RAG, 26 tools, temporal decay scoring | Server/Tool | ★★★★☆ Memory + Search |
| **coding-mcp** | https://github.com/kieutrongthien/coding-mcp | Multi-project remote coding MCP server — agents work across repos without cloning locally. File ops, git, allowlist-based command runner, OTel telemetry, RBAC | Server/Tool | ★★★★☆ Remote coding |
| **H1 2026 Ecosystem Retrospective** | https://www.digitalapplied.com/blog/mcp-ecosystem-h1-2026-retrospective-adoption-data-points | 9,400+ servers by Apr 2026, 5 canon host surfaces (Claude Desktop/Code, Codex CLI, Cursor, Windsurf + VS Code Copilot), enterprise OAuth/governance became table stakes | Analysis | ★★★★★ Ecosystem stats |
| **56 Server Ecosystem Tracker** | https://www.digitalapplied.com/blog/mcp-server-ecosystem-tracker-50-servers-cataloged-2026 | 15,930+ indexed servers (PulseMCP), MCP tunnels + sandboxes (Cloudflare/Daytona/Modal/Vercel) announced May 2026, production deployment shift | Analysis | ★★★★★ Deployment patterns |

## Key Takeaways

- **Protocol**: The 2026-07-28 spec is the biggest change since launch — stateless core, no session handshake, Streamable HTTP standard, SSE deprecated for new remote servers
- **Rust SDK**: `rmcp` (official, 10.5M downloads) is the dominant Rust implementation; alternatives include `rust-mcp-sdk`, `FastMCP Rust`, `mcpkit`
- **Code Tools**: `codebase-mcp` (37 tools) and `Pathfinder` (Rust, AST+LSP) are the most capable coding-focused MCP servers
- **Ecosystem**: 15,930+ servers across registries, 97M monthly SDK downloads, governance under Linux Foundation AAIF (Anthropic, OpenAI, Google, Microsoft, AWS)
