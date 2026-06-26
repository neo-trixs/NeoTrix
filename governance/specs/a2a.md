# Agent-to-Agent Protocol Subsystem
> Version 1.0 — Status: ✅ active

## Purpose
Implements Google A2A v1.0 (with v1.2 gRPC bridge) for inter-agent communication. Supports Agent Card discovery, task delegation, SSE streaming, and signed message verification.

## Exposed Operations

### Tools
- `send_task(target: AgentId, task: Task) -> Result<TaskId, A2aError>` — Dispatch a task to an external agent
- `register_agent_card(card: AgentCard) -> ()` — Register a local Agent Card for discovery
- `discover_agents(domain: String) -> Vec<AgentCard>` — Discover agents via Agent Card endpoint
- `forward_message(msg: RoutedMessage) -> Result<(), BusError>` — Route message through capability registry

### Resources
- `a2a://agent/{id}/card` — Agent Card metadata
- `a2a://tasks/active` — Currently active task list
- `a2a://tasks/{id}` — Single task status and result

## Configuration Schema

```rust
pub struct A2aConfig {
    pub server_port: u16,                // default 42071
    pub max_active_tasks: u32,           // default 100
    pub task_timeout_secs: u64,          // default 300
    pub enable_grpc: bool,               // v1.2 gRPC bridge, default false
    pub agent_card_ttl_secs: u64,        // cache TTL for agent cards, default 3600
}
```

## Dependencies
- **Agent Bus** (nt_core_agent) — Internal message routing target
- **Identity** (soul_identity) — Ed25519/ECDSA card signing
- **Discovery** (nt_core_discovery) — UDP beacon for LAN agents

## Error States

| State | Trigger | Recovery |
|-------|---------|----------|
| `CardExpired` | Agent Card TTL exceeded | Re-fetch from agent endpoint |
| `TaskTimeout` | Task exceeds timeout_secs | Cancel task, return timeout error |
| `ProtocolMismatch` | Remote agent A2A version incompatible | Fall back to v1.0 text mode |
| `SignatureInvalid` | Card signature verification failed | Reject card, log security event |
| `BusOverflow` | Internal bus channel full | Apply backpressure, drop lowest priority message |
