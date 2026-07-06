# ACP (Agent Client Protocol) Implementation Plan

## Findings Summary

### ACP Protocol Overview

| Aspect | Detail |
|--------|--------|
| **Spec** | JSON-RPC 2.0, stdio transport, newline-delimited messages |
| **Role** | NeoTrix acts as **Agent** (not client/editor) |
| **Version** | Protocol v1 (integer, currently `1`) |
| **Rust SDK** | [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) crate — official, maintained by Zed |
| **Status** | [agentclientprotocol.com](https://agentclientprotocol.com/) — active spec |

### Existing NeoTrix ACP Code (Stub Only)

| File | Lines | Current State |
|------|-------|---------------|
| `nt_io_server/mod.rs` | 25 | `NeoTrixACPServer` with `server_info()` — registered in `l1_body_impl/mod.rs` |
| `nt_io_server/protocol.rs` | 25 | `ACPMessage`(Ping/Shutdown), `ACPResponse`(Pong/Error), `ToolCall`, `ToolResult` |
| `nt_io_server/handler.rs` | 19 | `ACPHandler::handle()` — returns Pong for everything |

The existing code is **not** real ACP — it's a minimal placeholder with no JSON-RPC framing, no method dispatch, no session management, no capability negotiation.

### Existing MCP Infrastructure (Reusable)

| Capability | Location | Status |
|------------|----------|--------|
| `McpRegistry` | `neotrix-core/src/agent/tool/mcp/` | Fully operational |
| Stdio transport | Entries in McpRegistry | ✅ Implemented |
| HTTP transport | Entries in McpRegistry | ✅ Implemented |
| WS transport | Entries in McpRegistry | ✅ Implemented |
| SSE transport | Entries in McpRegistry | ✅ Implemented |
| CLI `/mcp` commands | `agent_cmds.rs` | list/status/discover/search/publish |

### Key ACP Methods (Agent Side — What NeoTrix Must Implement)

#### Baseline (Required)

| Method | Type | Purpose |
|--------|------|---------|
| `initialize` | Request→Response | Version + capability negotiation |
| `session/new` | Request→Response | Create conversation context |
| `session/prompt` | Request→Response (async) | Process user message, stream updates |
| `session/cancel` | Notification | Cancel ongoing prompt turn |

#### Optional (Advertised via capabilities)

| Method | Type | Capability Flag |
|--------|------|-----------------|
| `session/load` | Request→Response | `loadSession: true` |
| `session/resume` | Request→Response | `sessionCapabilities.resume` |
| `session/close` | Request→Response | `sessionCapabilities.close` |
| `session/list` | Request→Response | `sessionCapabilities.list` |
| `session/delete` | Request→Response | `sessionCapabilities.delete` |
| `session/set_mode` | Request→Response | `mcpCapabilities` |
| `session/set_config_option` | Request→Response | Config options support |
| `logout` | Request→Response | `auth.logout` |
| `authenticate` | Request→Response | If auth required |

#### Notifications Sent by Agent

| Notification | Purpose |
|-------------|---------|
| `session/update` (agent_message_chunk) | Stream text response |
| `session/update` (tool_call) | Report tool invocation |
| `session/update` (tool_call_update) | Tool progress/result |
| `session/update` (plan) | Report execution plan |
| `session/update` (usage_update) | Token/cost reporting |
| `session/update` (current_mode_update) | Mode change |
| `session/update` (available_commands) | Slash commands update |

#### Client Methods Called by Agent

| Method | Purpose |
|--------|---------|
| `session/request_permission` | Ask user to authorize tool |
| `fs/read_text_file` | Read file from client FS |
| `fs/write_text_file` | Write file to client FS |
| `terminal/create` | Execute command on client |
| `terminal/output` | Get terminal output |
| `terminal/kill` | Kill terminal command |
| `terminal/release` | Release terminal |
| `terminal/wait_for_exit` | Wait and get exit code |

---

## Architecture Overview

```
┌────────────────────────────────────────────────────┐
│                   Client (Editor/IDE)               │
│  ┌──────────────────────────────────────────────┐   │
│  │  ACP stdio JSON-RPC (newline-delimited)       │   │
│  └──────────┬───────────────────────────────────┘   │
└─────────────┼────────────────────────────────────────┘
              │ stdin/stdout
┌─────────────┼────────────────────────────────────────┐
│  NeoTrix-ACP Agent (Subprocess)                      │
│  ┌──────────┴──────────────────────────────────┐     │
│  │  nt_io_acp/                                 │     │
│  │  ├── transport.rs    stdio JSON-RPC I/O     │     │
│  │  ├── protocol.rs     ACP types (serde)      │     │
│  │  ├── handler.rs      Method dispatcher      │     │
│  │  ├── router.rs       Session routing        │     │
│  │  ├── session.rs      Session state machine  │     │
│  │  └── mod.rs          ACP agent entry point  │     │
│  └──────────────────────────────────────────────┘     │
│              │ reuses                               │
│  ┌───────────┴──────────────────────────────────┐    │
│  │  Existing Infrastructure                      │    │
│  │  ├── McpRegistry (tool registry)              │    │
│  │  ├── E8 ReasoningEngine (core logic)          │    │
│  │  ├── SL AgentSessionManager (session store)   │    │
│  │  ├── GatewayV2 (LLM provider)                 │    │
│  │  └── CLI/REPL (interactive mode)              │    │
│  └──────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

---

## File Structure Proposal

Replace the existing stub module with a full ACP implementation:

```
neotrix-core/src/neotrix/l1_body_impl/nt_io_acp/      # NEW (replaces nt_io_server)
├── mod.rs              # ACP Agent entry, re-exports
├── transport.rs        # Stdio JSON-RPC read/write
├── protocol.rs         # All ACP types (serde)
├── handler.rs          # Method dispatcher + session/prompt
├── router.rs           # Session→handler routing
└── session.rs          # Session state + lifecycle
```

**Migration**: `nt_io_server/` → `nt_io_acp/` (rename and rewrite).

---

## Dependencies

Add to `neotrix-core/Cargo.toml`:

```toml
# Official ACP Rust SDK — optional, can use instead of handrolling protocol types
agent-client-protocol = { version = "0.1", optional = true }
```

**Decision**: Start with handrolled types for full control, switch to SDK once v1 stabilizes.

---

## Priority Order

### P0 — Baseline Agent (Week 1)

| Step | File | Description |
|------|------|-------------|
| 1 | `protocol.rs` | ACP v1 types: `InitializeRequest/Response`, `NewSessionRequest/Response`, `PromptRequest/Response`, `CancelNotification`, `SessionNotification`, `ContentBlock`, `ToolCallUpdate`, `StopReason`, `AgentCapabilities`, `ClientCapabilities`, JSON-RPC envelope |
| 2 | `transport.rs` | `StdioTransport` — read/write newline-delimited JSON-RPC from stdin/stdout, `listen()` loop that parses `method` + `params` + `id`, dispatches to handler, sends responses |
| 3 | `handler.rs` | `handle_initialize()`, `handle_session_new()`, `handle_session_prompt()`, `handle_session_cancel()`, `handle_ping()` — dispatcher routing |
| 4 | `session.rs` | `AcpSession` — holds `session_id`, `cwd`, `mcp_servers`, conversation context (Vec<ContentBlock>), active status |
| 5 | `mod.rs` | `AcpAgent` — owns `StdioTransport`, `HashMap<String, AcpSession>`, `Arc<McpRegistry>`, starts `listen()` on init, graceful shutdown |
| 6 | Wiring | Register `nt_io_acp` in `l1_body_impl/mod.rs`, replace `nt_io_server` re-export, add `--acp` flag to main entry to run as ACP agent subprocess |

### P1 — Tool Call + File System Integration (Week 2)

| Step | Description |
|------|-------------|
| 7 | `session/prompt` forwards tool call requests to `McpRegistry::call_tool()` |
| 8 | Implement `session/update` notifications (agent_message_chunk streaming) |
| 9 | Implement `session/request_permission` — delegate to NeoTrix shield/perm system |
| 10 | Implement `fs/read_text_file` + `fs/write_text_file` — use existing file sandbox |
| 11 | Wire E8 ReasoningEngine into `session/prompt` for LLM processing |

### P2 — Session Lifecycle + Resume (Week 3)

| Step | Description |
|-------------|------|
| 12 | `session/load` — restore session from `AgentSessionManager` (SQLite), replay history |
| 13 | `session/close` — cleanup resources, cancel active work |
| 14 | `session/list` + `session/delete` — list/manage persisted sessions |
| 15 | `session/set_mode` — map to E8 hexagram modes |
| 16 | `authenticate` + `logout` — integrate with NeoTrix auth/shield |

### P3 — Terminal + Advanced Features (Week 4+)

| Step | Description |
|-------------|------|
| 17 | `terminal/create/output/kill/release/wait_for_exit` — reuse sandbox infrastructure |
| 18 | Agent plan reporting (`session/update` plan) — map to `nt_core_plan` |
| 19 | Usage/cost tracking (`session/update` usage_update) — reuse cost tracker |
| 20 | MCP-over-ACP support (RFD) — MCP transport via ACP channels |
| 21 | Streamable HTTP transport support |

---

## Protocol Integration Points

| ACP Concept | NeoTrix Equivalent |
|-------------|-------------------|
| Session | `AgentSessionManager` (SQLite-backed) |
| Tool calls | `McpRegistry` + ToolRegistry |
| LLM processing | `E8 ReasoningEngine` + `GatewayV2` |
| File system | `SandboxEnforcer` + `nt_shield` permissions |
| Permission | `nt_shield_perm` permission system |
| Cancel | Background loop shutdown (watch channel pattern) |
| Terminal | `CloudSandbox` (LocalDockerProvider) |
| Capability negotiation | Advertise subset matching E8/GWT/SAE features |

---

## CLI Integration

New ACP-specific CLI command:

| Command | Purpose |
|---------|---------|
| `neotrix acp` | Run as ACP agent (stdio transport) — subprocess mode |
| `neotrix acp --http` | Run as ACP agent via HTTP (future) |

Register in `cli/commands/` module, add to command registry with `/acp` prefix for interactive mode.

The existing REPL `/mcp` commands remain unchanged — they manage the *internal* tool registry that ACP will also consume.

---

## Testing Strategy

| Level | Scope | Method |
|-------|-------|--------|
| Unit | Protocol types serde roundtrip | `assert_eq!(serde_json::from_str::<T>(&serde_json::to_string(&t)?)?, t)` |
| Unit | Handler dispatch | Mock transport, verify response JSON |
| Integration | Full prompt turn | Spawn agent as subprocess, send JSON-RPC over stdio, verify notifications |
| Integration | Session persistence | Create → list → load → prompt → close cycle |
| E2E | With real client | Point Zed/editor to neotrix ACP agent |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| ACP spec in flux (v2 proposal active) | Implement stable v1 baseline, use extensibility `_meta` fields for v2 prep |
| Stdio JSON-RPC line-length limits | Ensure no response exceeds practical limits, chunk large responses |
| Async prompt processing | Use existing background task pattern (spawn handler + watch channel for cancel) |
| Session memory growth | Reuse `nt_core_gwt` 5-layer compression pipeline for context budgeting |
| SDK version conflicts | Pin `agent-client-protocol` version, or skip SDK entirely for P0 |
