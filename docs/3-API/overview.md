# API Overview

NeoTrix provides both a CLI and a desktop application. The HTTP API is available when running the `neotrix-web` binary.

## Base URL

```
http://localhost:3456
```

## Endpoints

See the OpenAPI specification at `6-REFERENCE/openapi.yaml` for the complete API reference.

## Authentication

API endpoints require authentication via API key. Set the `Authorization: Bearer <key>` header.

## Rate Limiting

Rate limits apply per API key. See the `X-RateLimit-*` response headers.
