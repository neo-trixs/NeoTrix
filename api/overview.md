# API Overview

NeoTrix exposes an OpenAI-compatible REST API when run in server mode. This allows any tool that supports a custom OpenAI endpoint to route through NeoTrix's reasoning engine.

## Starting the Server

```bash
neotrix --serve --addr 127.0.0.1:3000
```

## Base URL

```
http://localhost:3000/v1
```

## Authentication

All requests require a Bearer token in the `Authorization` header:

```
Authorization: Bearer <your-api-token>
```

The token is configured in `~/.neotrix/config.toml` or via the `NEOTRIX_API_TOKEN` environment variable.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/models` | List available models |
| `POST` | `/v1/chat/completions` | Chat completion (OpenAI-compatible) |
| `POST` | `/v1/completions` | Text completion |
| `GET` | `/v1/health` | Health check |
| `GET` | `/v1/metrics` | System metrics and brain state |

### Chat Completions

`POST /v1/chat/completions`

Standard OpenAI chat completions format:

```json
{
  "model": "claude-sonnet-4-20260514",
  "messages": [
    {"role": "system", "content": "You are NeoTrix."},
    {"role": "user", "content": "Explain E8 reasoning."}
  ],
  "temperature": 0.7,
  "max_tokens": 4096,
  "stream": true
}
```

### Health Check

`GET /v1/health`

```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "subsystems": {
    "e8": "healthy",
    "hcube": "healthy",
    "seal": "idle",
    "gwt": "active"
  }
}
```

### Metrics

`GET /v1/metrics`

```json
{
  "phi": 0.72,
  "fcs": 0.64,
  "usk": 0.81,
  "negentropy": 1.23,
  "curiosity": 0.45,
  "stagnation": 0.12,
  "sessions_active": 3,
  "knowledge_nodes": 56708,
  "knowledge_edges": 225503
}
```

## WebSocket

NeoTrix supports WebSocket connections for streaming completions and event subscriptions. Connect to:

```
ws://localhost:3000/v1/ws
```

For full WebSocket event documentation, see the [Events](/api/events) page.

## Rate Limiting

The server applies per-token rate limiting:
- **100 requests per minute** per IP (default)
- Configurable via `NEOTRIX_RATE_LIMIT` environment variable

## Error Responses

Standard HTTP error codes:

| Code | Meaning |
|------|---------|
| `400` | Bad request (invalid parameters) |
| `401` | Unauthorized (missing/invalid token) |
| `429` | Rate limit exceeded |
| `500` | Internal server error |

```json
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "Too many requests. Try again in 30 seconds."
  }
}
```
