# Events

NeoTrix provides real-time events via WebSocket and Server-Sent Events (SSE). These allow clients to subscribe to system state changes, reasoning progress, and evolution cycles.

## WebSocket Connection

```
ws://localhost:3000/v1/ws
```

The WebSocket endpoint accepts the same Bearer token authentication as the REST API.

## Message Format

All events follow a uniform JSON structure:

```json
{
  "type": "event_type",
  "timestamp": "2026-06-10T12:00:00Z",
  "data": { }
}
```

## Event Types

### Reasoning Events

Fired during E8 reasoning state transitions.

```json
{
  "type": "reasoning.state_change",
  "timestamp": "2026-06-10T12:00:01Z",
  "data": {
    "from_state": "101101",
    "to_state": "101110",
    "axis": "Depth",
    "observer_classification": "Productive",
    "trajectory_length": 14
  }
}
```

| Event | Description |
|-------|-------------|
| `reasoning.state_change` | E8 hexagram state transition |
| `reasoning.dead_end` | Observer detected a dead-end state |
| `reasoning.oscillation` | Observer detected an oscillating loop |

### Evolution Events

Fired during SEAL pipeline execution.

```json
{
  "type": "evolution.stage_complete",
  "timestamp": "2026-06-10T12:00:05Z",
  "data": {
    "stage": 4,
    "stage_name": "SSM State Update",
    "duration_ms": 230,
    "result": "promoted"
  }
}
```

| Event | Description |
|-------|-------------|
| `evolution.started` | SEAL cycle began |
| `evolution.stage_complete` | Individual pipeline stage completed |
| `evolution.champion_promoted` | A new champion capability vector was set |
| `evolution.rollback` | Pipeline rolled back to previous champion |
| `evolution.complete` | Full SEAL cycle finished |

### Metrics Events

Periodic or threshold-triggered metric snapshots.

```json
{
  "type": "metrics.snapshot",
  "timestamp": "2026-06-10T12:01:00Z",
  "data": {
    "phi": 0.72,
    "fcs": 0.64,
    "usk": 0.81,
    "negentropy": 1.23,
    "curiosity": 0.45
  }
}
```

| Event | Description |
|-------|-------------|
| `metrics.snapshot` | Periodic metric snapshot (every 30s) |
| `metrics.threshold_breach` | A metric crossed a configured threshold |

### Session Events

```json
{
  "type": "session.updated",
  "timestamp": "2026-06-10T12:00:10Z",
  "data": {
    "session_id": "abc123",
    "turn_count": 7,
    "tokens_used": 12450,
    "cost_usd": 0.08
  }
}
```

| Event | Description |
|-------|-------------|
| `session.created` | New session started |
| `session.updated` | Session state changed (new turn) |
| `session.saved` | Session persisted to disk |

### Knowledge Events

```json
{
  "type": "knowledge.node_created",
  "timestamp": "2026-06-10T12:00:15Z",
  "data": {
    "node_id": "node_42",
    "label": "E8 Lie Algebra",
    "source": "user_ingest",
    "vector_proximity_to_existing": 0.31
  }
}
```

| Event | Description |
|-------|-------------|
| `knowledge.node_created` | New node added to HyperCube |
| `knowledge.edge_created` | New edge between existing nodes |
| `knowledge.ingestion_complete` | Batch ingestion finished |

## Server-Sent Events (SSE)

When using `POST /v1/chat/completions` with `stream: true`, the server returns SSE-formatted chunks following the OpenAI streaming specification:

```
data: {"choices":[{"delta":{"content":"Hello"},"index":0}]}

data: {"choices":[{"delta":{"content":" world"},"index":0}]}

data: [DONE]
```

## Subscribing to Specific Events

Clients can send a subscription message on the WebSocket to filter events:

```json
{
  "type": "subscribe",
  "events": ["reasoning.*", "metrics.snapshot"]
}
```

Use `"*"` to subscribe to all events (the default). Unsubscribe with:

```json
{
  "type": "unsubscribe",
  "events": ["metrics.*"]
}
```
