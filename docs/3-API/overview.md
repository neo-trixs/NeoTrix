# API Overview

NeoTrix provides both a CLI and a desktop application. The desktop (Tauri) application exposes the HTTP API internally.

## Base URL

```
http://localhost:3456
```

## Endpoints

See the OpenAPI specification at `6-REFERENCE/openapi.yaml` for the complete API reference.

## Authentication

API endpoints require authentication via API key. Set the `Authorization: Bearer &lt;key&gt;` header.

## Rate Limiting

Rate limits apply per API key. See the `X-RateLimit-*` response headers.
