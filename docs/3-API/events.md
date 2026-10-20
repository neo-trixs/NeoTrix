# Events

NeoTrix emits events via WebSocket connections for real-time monitoring.

## Event Types

| Event | Description |
|-------|-------------|
| `reasoning.tick` | Fired on each E8 reasoning tick |
| `knowledge.search` | Knowledge base search result |
| `system.status` | System health status update |
| `agent.action` | Agent action notification |

## WebSocket

Connect to `ws://localhost:3456/ws` for real-time event streaming.
