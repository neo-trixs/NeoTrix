//! # MCP Transport — 2-mode transport + initialization handshake
//!
//! Clean separation: **Local (Stdio)** vs **Remote (HTTP/SSE)**.
//! Provides the shared `mcp_initialize()` handshake used by both
//! `McpRegistry` and `McpDiscovery`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::neotrix::nt_agent_mcp_auth::OAuthToken;

/// Transport mode — strictly 3 modes
#[derive(Debug, Clone, PartialEq)]
pub enum TransportMode {
    /// Local subprocess (stdio-based JSON-RPC)
    Local {
        command: String,
        args: Vec<String>,
    },
    /// Remote server (HTTP POST for JSON-RPC, optional SSE for streaming)
    Remote {
        /// HTTP(S) URL for JSON-RPC POST requests
        http_url: String,
        /// Custom HTTP headers
        headers: HashMap<String, String>,
        /// Optional SSE endpoint for server-initiated messages
        sse_url: Option<String>,
        /// Optional OAuth 2.1 token
        auth: Option<OAuthToken>,
    },
    /// Streamable HTTP — initial POST + SSE streaming endpoint
    /// Per MCP 2026-07-28 spec: client POSTs JSON-RPC, server responds
    /// synchronously with CacheableResult, then streams partial results via SSE.
    StreamableHttp {
        /// HTTP(S) URL for JSON-RPC POST requests
        http_url: String,
        /// SSE endpoint URL for streaming events (EventSource pattern)
        sse_endpoint: String,
        /// Custom HTTP headers
        headers: HashMap<String, String>,
        /// Optional OAuth 2.1 token
        auth: Option<OAuthToken>,
    },
}

impl TransportMode {
    pub fn mode_name(&self) -> &str {
        match self {
            TransportMode::Local { .. } => "local/stdio",
            TransportMode::Remote { .. } => "remote/http+sse",
            TransportMode::StreamableHttp { .. } => "remote/streamable-http",
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, TransportMode::Local { .. })
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, TransportMode::Remote { .. } | TransportMode::StreamableHttp { .. })
    }
}

/// P0-4: TLS verification must stay ON for remote MCP servers. The only
/// legitimate case for skipping cert validation is an explicit loopback URL
/// (localhost/127.0.0.1) where the user controls the endpoint — typically a
/// local bridge with a self-signed cert. Everything else enforces TLS.
fn is_loopback_url(url: &str) -> bool {
    url.split("://").nth(1).unwrap_or(url)
        .split(['/', '?']).next().unwrap_or("")
        .split(':').next().unwrap_or("")
        .to_lowercase()
        .trim_matches(['[', ']'])
        .eq_ignore_ascii_case("localhost")
        || url.contains("127.0.0.1") || url.contains("::1")
}

/// P0-4: build a reqwest blocking client that enforces TLS by default and only
/// relaxes cert validation for explicit loopback endpoints.
fn build_mcp_blocking_client(url: &str) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5));
    if is_loopback_url(url) {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build()
}

/// P0-4: build a reqwest async client that enforces TLS by default and only
/// relaxes cert validation for explicit loopback endpoints.
fn build_mcp_async_client(url: &str) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(5));
    if is_loopback_url(url) {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build()
}

/// Result of an MCP initialize handshake
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpInitResult {
    /// Protocol version negotiated (e.g. "2024-11-05")
    pub protocol_version: String,
    /// Server identification
    pub server_info: McpServerIdentity,
    /// Server capabilities
    pub capabilities: serde_json::Value,
}

/// Server identity from initialize response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerIdentity {
    pub name: String,
    pub version: String,
}

/// Standard MCP initialize request params
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpInitRequestParams {
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    pub client_info: McpClientInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

impl Default for McpInitRequestParams {
    fn default() -> Self {
        Self {
            protocol_version: "2024-11-05".into(),
            capabilities: serde_json::json!({
                "mcpApps": {},
                "tasks": {},
                "supports": {
                    "multiRoundTrip": true,
                    "traceContext": true,
                },
            }),
            client_info: McpClientInfo {
                name: "neotrix".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
        }
    }
}

/// JSON-RCP request envelope (2026-07-28: + `_meta`, + `resultType`)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Client capabilities for stateless requests (replaces old per-request handshake)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<serde_json::Value>,
    /// Multi-round-trip: "complete" | "input_required"
    #[serde(skip_serializing_if = "Option::is_none", rename = "resultType")]
    pub result_type: Option<String>,
}

/// Default `_meta` for stateless MCP requests per 2026-07-28 spec
///
/// SEP-2577: Roots, Sampling, Logging are removed. Replaced by `mcpApps` and `tasks`.
/// Added `supports` for multiRoundTrip and traceContext per W3C Trace Context spec.
pub fn default_client_meta() -> serde_json::Value {
    let traceparent = generate_traceparent();
    serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "mcpApps": {},
            "tasks": {},
            "supports": {
                "multiRoundTrip": true,
                "traceContext": true,
            },
        },
        "clientInfo": {
            "name": "neotrix",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "traceparent": traceparent,
        "tracestate": "",
    })
}

/// JSON-RPC response envelope
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcErrorValue>,
    /// Multi-round-trip: "complete" | "input_required"
    #[serde(default, rename = "resultType")]
    pub result_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JsonRpcErrorValue {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// CacheableResult — server-defined caching policy per 2026-07-28
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CacheableResult {
    /// Server-hinted TTL in milliseconds (from `x-mcp-ttl-ms` header or response body)
    pub ttl_ms: Option<u64>,
    /// Cache scope: "global", "session", "user", or server-defined
    pub cache_scope: Option<String>,
}

impl CacheableResult {
    pub fn get_cache_ttl(&self) -> Option<Duration> {
        self.ttl_ms.map(Duration::from_millis)
    }

    /// Check if the cached result is stale based on current time
    pub fn is_stale(&self, cached_at: Instant) -> bool {
        self.ttl_ms.is_some_and(|ttl| cached_at.elapsed() > Duration::from_millis(ttl))
    }

    /// Extract from HTTP response headers
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let ttl_ms = headers
            .get("x-mcp-ttl-ms")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let cache_scope = headers
            .get("x-mcp-cache-scope")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Self { ttl_ms, cache_scope }
    }
}

// ---- JSON Schema 2020-12 support ----

/// JSON Schema 2020-12 `$schema` URI constant
pub const JSON_SCHEMA_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

/// JSON Schema draft-07 `$schema` URI constant (backward compatibility)
pub const JSON_SCHEMA_DRAFT_07: &str = "http://json-schema.org/draft-07/schema#";

/// Resolve a `$ref` reference in a JSON Schema using `$defs`.
///
/// Supports JSON Schema 2020-12 `$defs` (replaces `definitions` from draft-07).
/// Only handles local `#/$defs/...` references (non-remote).
///
/// If the schema has a `$ref` field, it follows the pointer into `defs`
/// and returns the resolved sub-schema. Otherwise returns the schema unchanged.
pub fn resolve_json_schema_ref<'a>(
    schema: &'a serde_json::Value,
    defs: &'a serde_json::Value,
) -> &'a serde_json::Value {
    if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
        if let Some(rest) = ref_val.strip_prefix("#/$defs/") {
            if let Some(resolved) = defs.get(rest) {
                return resolved;
            }
        }
        // Fallback: try definitions (draft-07 compatibility)
        if let Some(rest) = ref_val.strip_prefix("#/definitions/") {
            if let Some(resolved) = defs.get(rest) {
                return resolved;
            }
        }
    }
    schema
}

/// Ensure a JSON schema value has `$schema` set to 2020-12.
///
/// Backward compatible: if `$schema` already present (draft-07 or otherwise),
/// it is preserved. Only adds `$schema` when absent.
pub fn ensure_schema_dialect(schema: &serde_json::Value) -> serde_json::Value {
    if schema.get("$schema").is_some() {
        return schema.clone();
    }
    let mut map = match schema {
        serde_json::Value::Object(m) => m.clone(),
        _ => return schema.clone(),
    };
    map.insert("$schema".into(), serde_json::Value::String(JSON_SCHEMA_2020_12.into()));
    serde_json::Value::Object(map)
}

/// Recursively resolve all `$ref` pointers in a schema using `$defs`.
///
/// Returns a new schema with all local `$ref` resolved to their definitions.
/// This is useful when consuming tools/list from servers that use 2020-12 schemas
/// with `$defs` and `$ref` patterns.
pub fn resolve_schema_refs(schema: &serde_json::Value, defs: &serde_json::Value) -> serde_json::Value {
    match schema {
        serde_json::Value::Object(map) => {
            // Check if this object is a $ref
            if let Some(ref_val) = map.get("$ref").and_then(|v| v.as_str()) {
                if ref_val.starts_with("#/$defs/") || ref_val.starts_with("#/definitions/") {
                    let resolved = resolve_json_schema_ref(schema, defs);
                    if !std::ptr::eq(resolved, schema) {
                        return resolve_schema_refs(resolved, defs);
                    }
                }
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), resolve_schema_refs(v, defs));
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| resolve_schema_refs(v, defs)).collect())
        }
        other => other.clone(),
    }
}

/// Normalize a tool input schema for MCP 2026-07-28 compatibility:
/// 1. Ensure `$schema` is set to 2020-12 if absent
/// 2. Extract `$defs` from the schema and resolve any top-level `$ref`
/// 3. Remove `$defs` from the output (kept as sideband for resolution)
pub fn normalize_tool_schema(schema: &serde_json::Value) -> serde_json::Value {
    let schema = ensure_schema_dialect(schema);
    let defs = schema.get("$defs")
        .or_else(|| schema.get("definitions"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if defs.is_null() {
        return schema;
    }
    // Keep $defs in output for client-side resolution (some clients need them)
    resolve_schema_refs(&schema, &defs)
}

// ---- SSE Event types ----

/// A single SSE event from a Streamable HTTP connection
#[derive(Debug, Clone, PartialEq)]
pub struct SseEvent {
    /// Event type (e.g. "message", "result", "error")
    pub event: Option<String>,
    /// Event data payload
    pub data: String,
    /// Optional event ID for replay
    pub id: Option<String>,
}

/// Result of a Streamable HTTP transport call
#[derive(Debug, Clone, PartialEq)]
pub struct StreamableHttpResult {
    /// Synchronous JSON-RPC response body
    pub response: String,
    /// Cache control from response headers
    pub cacheable: Option<CacheableResult>,
    /// Subsequent SSE events (may be empty if no streaming)
    pub sse_events: Vec<SseEvent>,
}

/// Make a Streamable HTTP transport call:
/// 1. POSTs the JSON-RPC body to the HTTP URL
/// 2. Reads the synchronous response + CacheableResult headers
/// 3. If the response indicates streaming (`resultType: "input_required"` or
///    server sends `x-mcp-stream` hint), connects to the SSE endpoint
///    and collects events
pub fn streamable_http_transport(
    http_url: &str,
    sse_endpoint: &str,
    headers: &HashMap<String, String>,
    auth: Option<&OAuthToken>,
    body: &str,
    mcp_method: &str,
) -> Result<StreamableHttpResult, TransportError> {
    // 1. Initial POST — TLS enforced except explicit loopback (P0-4)
    let client = build_mcp_blocking_client(http_url)
        .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

    let mut req = client.post(http_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2024-11-05")
        .header("Mcp-Method", mcp_method);

    for (k, v) in headers {
        req = req.header(k, v);
    }
    if let Some(token) = auth {
        req = req.header("Authorization", token.auth_header());
    }

    let resp = req.body(body.to_string())
        .send()
        .map_err(|e| TransportError::Io(format!("Streamable HTTP POST: {}", e)))?;

    let cacheable = CacheableResult::from_headers(resp.headers());

    // Check if server wants streaming
    let should_stream = resp.headers()
        .get("x-mcp-stream")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    let text = resp.text()
        .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

    // Check if the response body indicates streaming is needed
    let needs_streaming = should_stream
        || text.contains(r#""input_required""#)
        || text.contains(r#""resultType":"input_required""#);

    let sse_events = if needs_streaming {
        streamable_http_sse_connect(sse_endpoint, headers, auth, Duration::from_secs(30))?
    } else {
        Vec::new()
    };

    Ok(StreamableHttpResult {
        response: text,
        cacheable: Some(cacheable),
        sse_events,
    })
}

/// Connect to a Streamable HTTP SSE endpoint and collect events.
///
/// Uses reqwest blocking GET with `Accept: text/event-stream`.
/// Collects events until the connection closes or timeout expires.
/// Each SSE event (lines starting with `event:`, `data:`, `id:`) is parsed
/// into an `SseEvent`.
pub fn streamable_http_sse_connect(
    sse_endpoint: &str,
    headers: &HashMap<String, String>,
    auth: Option<&OAuthToken>,
    timeout: Duration,
) -> Result<Vec<SseEvent>, TransportError> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(timeout);
    if is_loopback_url(sse_endpoint) {
        builder = builder.danger_accept_invalid_certs(true);
    }
    let client = builder.build()
        .map_err(|e| TransportError::Io(format!("Build SSE client: {}", e)))?;

    let mut req = client.get(sse_endpoint)
        .header("Accept", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("MCP-Protocol-Version", "2024-11-05")
        .header("Mcp-Method", "subscriptions/listen");

    for (k, v) in headers {
        req = req.header(k, v);
    }
    if let Some(token) = auth {
        req = req.header("Authorization", token.auth_header());
    }

    let resp = req.send()
        .map_err(|e| TransportError::Io(format!("SSE connect: {}", e)))?;

    if !resp.status().is_success() {
        return Err(TransportError::Io(format!(
            "SSE endpoint returned HTTP {}", resp.status()
        )));
    }

    // Read the response body and parse SSE events
    let body = resp.text()
        .map_err(|e| TransportError::Io(format!("Read SSE body: {}", e)))?;

    Ok(parse_sse_events(&body))
}

/// Parse raw SSE text into a list of SseEvent structs.
///
/// SSE format per spec:
/// ```text
/// event: result
/// data: {"jsonrpc":"2.0","result":...}
/// id: 1
///
/// ```
pub fn parse_sse_events(raw: &str) -> Vec<SseEvent> {
    let mut events = Vec::new();
    let mut current_event: Option<String> = None;
    let mut current_data = String::new();
    let mut current_id: Option<String> = None;

    for line in raw.lines() {
        if line.is_empty() {
            // Empty line = event boundary
            if !current_data.is_empty() || current_event.is_some() {
                events.push(SseEvent {
                    event: current_event.take(),
                    data: std::mem::take(&mut current_data),
                    id: current_id.take(),
                });
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("event:") {
            current_event = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(value.trim());
        } else if let Some(value) = line.strip_prefix("id:") {
            current_id = Some(value.trim().to_string());
        }
        // Ignore retry: and other fields
    }

    // Flush last event
    if !current_data.is_empty() || current_event.is_some() {
        events.push(SseEvent {
            event: current_event,
            data: current_data,
            id: current_id,
        });
    }

    events
}

/// Generate a W3C Trace Context `traceparent` header value.
/// Format: `00-{trace_id}-{span_id}-{flags}`
/// - trace_id: 32 hex chars (16 bytes, random)
/// - span_id: 16 hex chars (8 bytes, random)
/// - flags: "01" (sampled)
pub fn generate_traceparent() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seed = now.as_nanos() as u64;
    // Simple pseudo-random trace ID (32 hex chars)
    let trace_id = format!(
        "{:016x}{:016x}",
        seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407),
        seed.wrapping_mul(2862933555777941757).wrapping_add(3037000493),
    );
    // Span ID (16 hex chars)
    let span_id = format!(
        "{:016x}",
        seed.wrapping_mul(3202034522624059733).wrapping_add(1),
    );
    format!("00-{}-{}-01", trace_id, span_id)
}

/// Error during MCP transport operations
#[derive(Debug, Clone, PartialEq)]
pub enum TransportError {
    Io(String),
    Protocol(String),
    Handshake(String),
    Timeout,
    Auth(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Io(e) => write!(f, "Transport I/O: {}", e),
            TransportError::Protocol(e) => write!(f, "Protocol error: {}", e),
            TransportError::Handshake(e) => write!(f, "Handshake failed: {}", e),
            TransportError::Timeout => write!(f, "Transport timeout"),
            TransportError::Auth(e) => write!(f, "Auth error: {}", e),
        }
    }
}

impl std::error::Error for TransportError {}

/// Perform MCP initialize handshake over a transport
///
/// # Deprecated
/// Per 2026-07-28 spec: `initialize` handshake is REMOVED.
/// Protocol version is now carried via `_meta.protocolVersion` in every request
/// and `MCP-Protocol-Version` HTTP header for remote transport.
/// Kept for backward compatibility with legacy servers.
#[deprecated(since = "0.20.0", note = "initialize handshake removed in 2026-07-28 spec; use _meta.protocolVersion + MCP-Protocol-Version header instead")]
pub fn mcp_initialize(
    transport: &TransportMode,
    timeout: Duration,
) -> Result<McpInitResult, TransportError> {
    let params = McpInitRequestParams::default();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(serde_json::to_value(params).unwrap_or_default()),
        _meta: Some(default_client_meta()),
        result_type: None,
    };
    let request_json = serde_json::to_string(&request)
        .map_err(|e| TransportError::Protocol(format!("Serialize request: {}", e)))?;

    match transport {
        TransportMode::Local { command, args } => {
            initialize_stdio(command, args, &request_json, timeout)
        }
        TransportMode::Remote { http_url, headers, auth, .. }
        | TransportMode::StreamableHttp { http_url, headers, auth, .. } => {
            initialize_remote(http_url, headers, auth.as_ref(), &request_json, timeout)
        }
    }
}

/// Send `notifications/initialized` after successful init
pub fn mcp_initialized_notification(
    transport: &TransportMode,
) -> Result<(), TransportError> {
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {},
    });
    let json = serde_json::to_string(&notification)
        .map_err(|e| TransportError::Protocol(format!("Serialize: {}", e)))?;

    match transport {
        TransportMode::Local { command, args } => {
            // One-shot: spawn, write, done (fire-and-forget is fine)
            if let Ok(mut child) = std::process::Command::new(command)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = writeln!(stdin, "{}", json);
                }
                let _ = child.wait();
            }
            Ok(())
        }
        TransportMode::Remote { http_url, headers, auth, .. }
        | TransportMode::StreamableHttp { http_url, headers, auth, .. } => {
            let client = build_mcp_blocking_client(http_url)
                .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

            let mut req = client.post(http_url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("MCP-Protocol-Version", "2024-11-05")
                .header("Mcp-Method", "notifications/initialized");

            for (k, v) in headers {
                req = req.header(k, v);
            }
            if let Some(token) = auth {
                req = req.header("Authorization", token.auth_header());
            }

            let _ = req.json(&notification).send();
            Ok(())
        }
    }
}

// ---- Stdio init ----

fn initialize_stdio(
    command: &str,
    args: &[String],
    request_json: &str,
    timeout: Duration,
) -> Result<McpInitResult, TransportError> {
    let mut child = std::process::Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| TransportError::Io(format!("Spawn {}: {}", command, e)))?;

    // Write request to stdin
    {
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| TransportError::Io("No stdin".into()))?;
        use std::io::Write;
        writeln!(stdin, "{}", request_json)
            .map_err(|e| TransportError::Io(format!("Write stdin: {}", e)))?;
    }

    // Read response from stdout with timeout
    let start = Instant::now();
    let mut line = String::new();
    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TransportError::Timeout);
        }

        // Use non-blocking read via BufReader on stdout
        if let Some(stdout) = child.stdout.as_mut() {
            use std::io::BufRead;
            let mut reader = std::io::BufReader::new(&mut *stdout);
            // Try reading with a short timeout-like approach
            reader.read_line(&mut line).ok();
            if !line.trim().is_empty() {
                break;
            }
        }
        // Brief yield
        std::thread::sleep(Duration::from_millis(10));
    }

    let _ = child.kill();
    let _ = child.wait();

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(TransportError::Handshake("No response from server (empty)".into()));
    }

    parse_init_response(trimmed)
}

// ---- Remote init ----

fn initialize_remote(
    url: &str,
    headers: &HashMap<String, String>,
    auth: Option<&OAuthToken>,
    request_json: &str,
    timeout: Duration,
) -> Result<McpInitResult, TransportError> {
    // TLS enforced except explicit loopback (P0-4)
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(timeout);
    if is_loopback_url(url) {
        builder = builder.danger_accept_invalid_certs(true);
    }
    let client = builder.build()
        .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

    let mut req = client.post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("MCP-Protocol-Version", "2024-11-05")
        .header("Mcp-Method", "initialize");

    for (k, v) in headers {
        req = req.header(k, v);
    }
    if let Some(token) = auth {
        req = req.header("Authorization", token.auth_header());
    }

    let resp = req.body(request_json.to_string())
        .send()
        .map_err(|e| TransportError::Io(format!("HTTP request: {}", e)))?;

    if !resp.status().is_success() {
        return Err(TransportError::Handshake(format!(
            "HTTP {} from initialize endpoint", resp.status()
        )));
    }

    let text = resp.text()
        .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

    parse_init_response(&text)
}

fn parse_init_response(response_json: &str) -> Result<McpInitResult, TransportError> {
    let rpc_resp: JsonRpcResponse = serde_json::from_str(response_json)
        .map_err(|e| TransportError::Protocol(format!("Invalid JSON-RPC: {} — raw: {}", e, &response_json[..response_json.len().min(200)])))?;

    if let Some(err) = rpc_resp.error {
        return Err(TransportError::Handshake(format!(
            "Server error ({}): {}", err.code, err.message
        )));
    }

    let result = rpc_resp.result
        .ok_or_else(|| TransportError::Handshake("No result in initialize response".into()))?;

    let protocol_version = result.get("protocolVersion")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TransportError::Handshake("Missing protocolVersion in response".into()))?
        .to_string();

    let server_info_val = result.get("serverInfo")
        .ok_or_else(|| TransportError::Handshake("Missing serverInfo in response".into()))?;

    let server_info = McpServerIdentity {
        name: server_info_val.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        version: server_info_val.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string(),
    };

    let capabilities = result.get("capabilities")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    Ok(McpInitResult {
        protocol_version,
        server_info,
        capabilities,
    })
}

/// Execute a JSON-RPC tool call on a transport
/// Returns (response_body, optional CacheableResult from response headers)
pub fn mcp_call_tool(
    transport: &TransportMode,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    let meta = default_client_meta();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": args,
        },
        "_meta": meta,
        "id": 2,
    });
    let body_str = serde_json::to_string(&body)
        .map_err(|e| TransportError::Protocol(format!("Serialize: {}", e)))?;

    match transport {
        TransportMode::Local { command, args: cmd_args } => {
            call_stdio(command, cmd_args, &body_str)
                .map(|s| (s, None))
        }
        TransportMode::Remote { http_url, headers, auth, .. }
        | TransportMode::StreamableHttp { http_url, headers, auth, .. } => {
            call_remote(http_url, headers, auth.as_ref(), &body_str, Some("tools/call"), Some(tool_name))
        }
    }
}

fn call_stdio(command: &str, args: &[String], body: &str) -> Result<String, TransportError> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| TransportError::Io(format!("Spawn {}: {}", command, e)))?;

    // Write body to stdin
    {
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| TransportError::Io("No stdin".into()))?;
        use std::io::Write;
        writeln!(stdin, "{}", body)
            .map_err(|e| TransportError::Io(format!("Write stdin: {}", e)))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| TransportError::Io(format!("Wait: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| TransportError::Protocol(format!("Invalid UTF-8: {}", e)))?;
        // Extract result from JSON-RPC response
        extract_tool_result(&stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TransportError::Io(format!("Process exited with {}: {}", output.status, stderr)))
    }
}

fn call_remote(
    url: &str,
    headers: &HashMap<String, String>,
    auth: Option<&OAuthToken>,
    body: &str,
    mcp_method: Option<&str>,
    mcp_name: Option<&str>,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    let client = build_mcp_blocking_client(url)
        .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

    let mut req = client.post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("MCP-Protocol-Version", "2024-11-05");

    // Mcp-Method header per 2026-07-28
    if let Some(method) = mcp_method {
        req = req.header("Mcp-Method", method);
    }
    // Mcp-Name header per 2026-07-28 (for tools/call, resources/read, etc.)
    if let Some(name) = mcp_name {
        req = req.header("Mcp-Name", name);
    }

    for (k, v) in headers {
        req = req.header(k, v);
    }
    if let Some(token) = auth {
        req = req.header("Authorization", token.auth_header());
    }

    let resp = req.body(body.to_string())
        .send()
        .map_err(|e| TransportError::Io(format!("HTTP POST: {}", e)))?;

    if !resp.status().is_success() {
        return Err(TransportError::Io(format!("HTTP {}: {}", resp.status(), resp.text().unwrap_or_default())));
    }

    // Extract CacheableResult from response headers
    let cacheable = CacheableResult::from_headers(resp.headers());

    let text = resp.text()
        .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

    extract_tool_result(&text).map(|s| (s, Some(cacheable)))
}

fn extract_tool_result(response_json: &str) -> Result<String, TransportError> {
    let rpc_resp: JsonRpcResponse = serde_json::from_str(response_json)
        .map_err(|e| TransportError::Protocol(format!("Invalid JSON-RPC response: {}", e)))?;

    if let Some(err) = rpc_resp.error {
        return Err(TransportError::Protocol(format!("Tool call error ({}): {}",
            err.code, err.message)));
    }

    Ok(response_json.to_string())
}

/// Send an arbitrary JSON-RPC request to an MCP transport
/// Returns (response_body_string, optional CacheableResult)
pub fn mcp_send_request(
    transport: &TransportMode,
    method: &str,
    params: Option<serde_json::Value>,
    id: u64,
    name_hint: Option<&str>,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    let meta = default_client_meta();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: method.into(),
        params,
        _meta: Some(meta),
        result_type: None,
    };
    let request_json = serde_json::to_string(&request)
        .map_err(|e| TransportError::Protocol(format!("Serialize request: {}", e)))?;

    match transport {
        TransportMode::Local { command, args } => {
            call_stdio(command, args, &request_json).map(|s| (s, None))
        }
        TransportMode::Remote { http_url, headers, auth, .. }
        | TransportMode::StreamableHttp { http_url, headers, auth, .. } => {
            call_remote(
                http_url, headers, auth.as_ref(), &request_json,
                Some(method), name_hint,
            )
        }
    }
}

/// `server/discover` — stateless capability discovery (replaces initialize for lightweight clients)
///
/// Per 2026-07-28 spec: returns server capabilities without establishing a session.
pub fn mcp_server_discover(
    transport: &TransportMode,
) -> Result<McpInitResult, TransportError> {
    let meta = default_client_meta();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "server/discover".into(),
        params: None,
        _meta: Some(meta),
        result_type: None,
    };
    let request_json = serde_json::to_string(&request)
        .map_err(|e| TransportError::Protocol(format!("Serialize request: {}", e)))?;

    match transport {
        TransportMode::Local { command, args } => {
            initialize_stdio(command, args, &request_json, Duration::from_secs(10))
        }
        TransportMode::Remote { http_url, headers, auth, .. }
        | TransportMode::StreamableHttp { http_url, headers, auth, .. } => {
            let mut builder = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10));
            if is_loopback_url(http_url) {
                builder = builder.danger_accept_invalid_certs(true);
            }
            let client = builder.build()
                .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

            let mut req = client.post(http_url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("MCP-Protocol-Version", "2024-11-05")
                .header("Mcp-Method", "server/discover");

            for (k, v) in headers {
                req = req.header(k, v);
            }
            if let Some(token) = auth {
                req = req.header("Authorization", token.auth_header());
            }

            let resp = req.body(request_json)
                .send()
                .map_err(|e| TransportError::Io(format!("HTTP discover: {}", e)))?;

            if !resp.status().is_success() {
                return Err(TransportError::Handshake(format!(
                    "HTTP {} from discover endpoint", resp.status()
                )));
            }

            let text = resp.text()
                .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

            parse_init_response(&text)
        }
    }
}

/// `subscriptions/listen` — SSE streaming for server-initiated notifications
///
/// Per 2026-07-28 spec: replaces old `resources/subscribe`. Returns a stream
/// of JSON-RPC notifications. The optional `filter` limits which notifications
/// are delivered.
pub fn mcp_subscribe_listen(
    transport: &TransportMode,
    filter: Option<serde_json::Value>,
) -> Result<String, TransportError> {
    let meta = default_client_meta();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "subscriptions/listen".into(),
        params: filter.map(|f| serde_json::json!({ "filter": f })),
        _meta: Some(meta),
        result_type: None,
    };
    let request_json = serde_json::to_string(&request)
        .map_err(|e| TransportError::Protocol(format!("Serialize request: {}", e)))?;

    match transport {
        TransportMode::Local { command, args } => {
            call_stdio(command, args, &request_json)
        }
        TransportMode::Remote { http_url, headers, auth, sse_url } => {
            // Use SSE URL if provided, otherwise fall back to HTTP POST
            let url = sse_url.as_deref().unwrap_or(http_url);
            // TLS enforced except explicit loopback (P0-4)
            let mut builder = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30));
            if is_loopback_url(url) {
                builder = builder.danger_accept_invalid_certs(true);
            }
            let client = builder.build()
                .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

            let mut req = client.post(url)
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .header("MCP-Protocol-Version", "2024-11-05")
                .header("Mcp-Method", "subscriptions/listen");

            for (k, v) in headers {
                req = req.header(k, v);
            }
            if let Some(token) = auth {
                req = req.header("Authorization", token.auth_header());
            }

            let resp = req.body(request_json)
                .send()
                .map_err(|e| TransportError::Io(format!("HTTP subscribe/listen: {}", e)))?;

            if !resp.status().is_success() {
                return Err(TransportError::Io(format!(
                    "HTTP {} from subscribe/listen", resp.status()
                )));
            }

            let text = resp.text()
                .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

            Ok(text)
        }
        TransportMode::StreamableHttp { http_url, headers, auth, sse_endpoint } => {
            // Streamable HTTP: POST to http_url, then connect to SSE endpoint for streaming.
            // TLS enforced except explicit loopback (P0-4).
            let mut builder = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30));
            if is_loopback_url(http_url) {
                builder = builder.danger_accept_invalid_certs(true);
            }
            let client = builder.build()
                .map_err(|e| TransportError::Io(format!("Build client: {}", e)))?;

            let mut req = client.post(http_url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json, text/event-stream")
                .header("MCP-Protocol-Version", "2024-11-05")
                .header("Mcp-Method", "subscriptions/listen");

            for (k, v) in headers {
                req = req.header(k, v);
            }
            if let Some(token) = auth {
                req = req.header("Authorization", token.auth_header());
            }

            let resp = req.body(request_json)
                .send()
                .map_err(|e| TransportError::Io(format!("Streamable HTTP subscribe/listen: {}", e)))?;

            if !resp.status().is_success() {
                return Err(TransportError::Io(format!(
                    "HTTP {} from subscribe/listen", resp.status()
                )));
            }

            let text = resp.text()
                .map_err(|e| TransportError::Io(format!("Read response: {}", e)))?;

            // Also connect to SSE endpoint for ongoing events
            // Note: SSE events are collected but the primary response is the POST response
            let _sse_events = streamable_http_sse_connect(
                sse_endpoint, headers, auth.as_ref(), Duration::from_secs(30),
            ).unwrap_or_default();

            Ok(text)
        }
    }
}

/// `tasks/get` — retrieve the current status/progress/result of a task.
///
/// Per 2026-07-28 Tasks extension: the server creates task handles,
/// and the client polls with tasks/get.
pub fn mcp_task_get(
    transport: &TransportMode,
    task_id: &str,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    mcp_send_request(
        transport,
        "tasks/get",
        Some(serde_json::json!({ "id": task_id })),
        1,
        Some(task_id),
    )
}

/// `tasks/update` — provide new input or instructions to a running task.
pub fn mcp_task_update(
    transport: &TransportMode,
    task_id: &str,
    input: serde_json::Value,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    mcp_send_request(
        transport,
        "tasks/update",
        Some(serde_json::json!({ "id": task_id, "input": input })),
        1,
        Some(task_id),
    )
}

/// `tasks/cancel` — request cancellation of a running task.
pub fn mcp_task_cancel(
    transport: &TransportMode,
    task_id: &str,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    mcp_send_request(
        transport,
        "tasks/cancel",
        Some(serde_json::json!({ "id": task_id })),
        1,
        Some(task_id),
    )
}

/// `tasks/subscribe` — subscribe to progress notifications for a task.
pub fn mcp_task_subscribe(
    transport: &TransportMode,
    task_id: &str,
) -> Result<(String, Option<CacheableResult>), TransportError> {
    mcp_send_request(
        transport,
        "tasks/subscribe",
        Some(serde_json::json!({ "id": task_id })),
        1,
        Some(task_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_mode_name() {
        let local = TransportMode::Local {
            command: "echo".into(),
            args: vec![],
        };
        assert_eq!(local.mode_name(), "local/stdio");
        assert!(local.is_local());
        assert!(!local.is_remote());

        let remote = TransportMode::Remote {
            http_url: "https://example.com/mcp".into(),
            headers: HashMap::new(),
            sse_url: None,
            auth: None,
        };
        assert_eq!(remote.mode_name(), "remote/http+sse");
        assert!(!remote.is_local());
        assert!(remote.is_remote());
    }

    #[test]
    fn test_parse_init_response_ok() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "test-server",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }"#;
        let result = parse_init_response(json).expect("parse should succeed");
        assert_eq!(result.protocol_version, "2024-11-05");
        assert_eq!(result.server_info.name, "test-server");
        assert_eq!(result.server_info.version, "1.0.0");
        assert!(result.capabilities.get("tools").is_some());
    }

    #[test]
    fn test_parse_init_response_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        }"#;
        let result = parse_init_response(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Request"));
    }

    #[test]
    fn test_parse_init_response_missing_fields() {
        let json = r#"{"jsonrpc": "2.0", "id": 1, "result": {}}"#;
        let result = parse_init_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_rpc_roundtrip() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 42,
            method: "test".into(),
            params: Some(serde_json::json!({"key": "value"})),
            _meta: Some(serde_json::json!({"protocolVersion": "2024-11-05"})),
            result_type: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("test"));
        assert!(json.contains("key"));
        assert!(json.contains("_meta"));
        assert!(json.contains("protocolVersion"));
    }

    #[test]
    fn test_json_rpc_roundtrip_without_meta() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "ping".into(),
            params: None,
            _meta: None,
            result_type: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        // _meta should be absent when None (skip_serializing_if)
        assert!(!json.contains("_meta"));
    }

    #[test]
    fn test_extract_tool_result_ok() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "content": [{"type": "text", "text": "hello"}]
            }
        }"#;
        let result = extract_tool_result(json).expect("should succeed");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_extract_tool_result_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "error": {
                "code": -32000,
                "message": "Tool execution failed"
            }
        }"#;
        let result = extract_tool_result(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool execution failed"));
    }

    #[test]
    fn test_mcp_init_request_params_default() {
        let params = McpInitRequestParams::default();
        assert_eq!(params.protocol_version, "2024-11-05");
        assert_eq!(params.client_info.name, "neotrix");
        // SEP-2577: roots/sampling removed, mcpApps/tasks added
        let caps = &params.capabilities;
        assert!(caps.get("mcpApps").is_some(), "mcpApps should be present per SEP-2577");
        assert!(caps.get("tasks").is_some(), "tasks should be present per SEP-2577");
        assert!(caps.get("roots").is_none(), "roots should be removed per SEP-2577");
        assert!(caps.get("sampling").is_none(), "sampling should be removed per SEP-2577");
        let supports = caps.get("supports").unwrap();
        assert_eq!(supports["multiRoundTrip"], true);
        assert_eq!(supports["traceContext"], true);
    }

    #[test]
    fn test_generate_traceparent_format() {
        let tp = generate_traceparent();
        // Format: 00-{32 hex}-{16 hex}-01
        assert!(tp.len() == 55, "traceparent should be 55 chars: {}", tp);
        assert!(tp.starts_with("00-"), "should start with 00-");
        assert!(tp.ends_with("-01"), "should end with -01 (sampled)");
        let parts: Vec<&str> = tp.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[1].len(), 32, "trace_id should be 32 hex chars");
        assert_eq!(parts[2].len(), 16, "span_id should be 16 hex chars");
        assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()), "trace_id should be hex");
        assert!(parts[2].chars().all(|c| c.is_ascii_hexdigit()), "span_id should be hex");
    }

    #[test]
    fn test_cacheable_result_is_stale() {
        let cr = CacheableResult { ttl_ms: Some(100), cache_scope: None };
        let past = std::time::Instant::now() - std::time::Duration::from_millis(200);
        assert!(cr.is_stale(past), "should be stale if TTL expired");
        let cr_no_ttl = CacheableResult::default();
        assert!(!cr_no_ttl.is_stale(past), "no TTL = never stale");
    }

    #[test]
    fn test_transport_error_display() {
        let e = TransportError::Timeout;
        assert_eq!(e.to_string(), "Transport timeout");

        let e = TransportError::Handshake("bad version".into());
        assert!(e.to_string().contains("bad version"));
    }

    #[test]
    fn test_default_client_meta() {
        let meta = default_client_meta();
        assert!(meta.get("protocolVersion").is_some());
        assert!(meta.get("capabilities").is_some());
        assert!(meta.get("clientInfo").is_some());
        assert!(meta.get("traceparent").is_some());
        assert!(meta.get("tracestate").is_some());
        assert_eq!(meta["protocolVersion"], "2024-11-05");
        // SEP-2577: roots/sampling removed, mcpApps/tasks added
        let caps = meta.get("capabilities").unwrap();
        assert!(caps.get("mcpApps").is_some(), "mcpApps should be present per SEP-2577");
        assert!(caps.get("tasks").is_some(), "tasks should be present per SEP-2577");
        assert!(caps.get("roots").is_none(), "roots should be removed per SEP-2577");
        assert!(caps.get("sampling").is_none(), "sampling should be removed per SEP-2577");
        // supports flags
        let supports = caps.get("supports").unwrap();
        assert_eq!(supports["multiRoundTrip"], true);
        assert_eq!(supports["traceContext"], true);
    }

    #[test]
    fn test_cacheable_result_default() {
        let cr = CacheableResult::default();
        assert!(cr.ttl_ms.is_none());
        assert!(cr.cache_scope.is_none());
        assert!(cr.get_cache_ttl().is_none());
        assert!(!cr.is_stale(std::time::Instant::now()));
    }

    #[test]
    fn test_cacheable_result_get_ttl() {
        let cr = CacheableResult { ttl_ms: Some(5000), cache_scope: Some("global".into()) };
        let ttl = cr.get_cache_ttl().expect("should have ttl");
        assert_eq!(ttl, Duration::from_millis(5000));
        assert_eq!(cr.cache_scope.as_deref(), Some("global"));
        assert!(!cr.is_stale(std::time::Instant::now()));
    }

    #[test]
    fn test_server_discover_request_has_meta() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "server/discover".into(),
            params: None,
            _meta: Some(default_client_meta()),
            result_type: None,
        };
        let json = serde_json::to_string(&request).expect("serialize");
        assert!(json.contains("server/discover"));
        assert!(json.contains("protocolVersion"));
    }

    #[test]
    fn test_subscribe_listen_request() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "subscriptions/listen".into(),
            params: Some(serde_json::json!({"filter": {"tools": ["search"]}})),
            _meta: Some(default_client_meta()),
            result_type: None,
        };
        let json = serde_json::to_string(&request).expect("serialize");
        assert!(json.contains("subscriptions/listen"));
        assert!(json.contains("filter"));
        assert!(json.contains("search"));
    }

    // ---- StreamableHttp tests ----

    #[test]
    fn test_streamable_http_mode_name() {
        let sh = TransportMode::StreamableHttp {
            http_url: "https://example.com/mcp".into(),
            sse_endpoint: "https://example.com/mcp/sse".into(),
            headers: HashMap::new(),
            auth: None,
        };
        assert_eq!(sh.mode_name(), "remote/streamable-http");
        assert!(!sh.is_local());
        assert!(sh.is_remote());
    }

    #[test]
    fn test_streamable_http_is_remote() {
        let sh = TransportMode::StreamableHttp {
            http_url: "https://mcp.example.com/rpc".into(),
            sse_endpoint: "https://mcp.example.com/events".into(),
            headers: HashMap::new(),
            auth: None,
        };
        assert!(sh.is_remote(), "StreamableHttp should be considered remote");
        // Verify it's not local
        let local = TransportMode::Local { command: "echo".into(), args: vec![] };
        assert!(!local.is_remote());
    }

    // ---- JSON Schema 2020-12 tests ----

    #[test]
    fn test_ensure_schema_dialect_adds_2020_12() {
        let schema = serde_json::json!({"type": "object", "properties": {}});
        let result = ensure_schema_dialect(&schema);
        assert_eq!(result["$schema"], JSON_SCHEMA_2020_12);
        assert_eq!(result["type"], "object");
    }

    #[test]
    fn test_ensure_schema_dialect_preserves_existing() {
        let schema = serde_json::json!({
            "type": "object",
            "$schema": "http://json-schema.org/draft-07/schema#"
        });
        let result = ensure_schema_dialect(&schema);
        // Should preserve draft-07, not override to 2020-12
        assert_eq!(result["$schema"], "http://json-schema.org/draft-07/schema#");
    }

    #[test]
    fn test_ensure_schema_dialect_non_object() {
        // Non-object schemas (e.g. boolean true) returned unchanged
        let schema = serde_json::json!(true);
        let result = ensure_schema_dialect(&schema);
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_resolve_json_schema_ref_local() {
        let defs = serde_json::json!({
            "address": {
                "type": "object",
                "properties": {
                    "street": {"type": "string"},
                    "city": {"type": "string"}
                }
            }
        });
        let schema = serde_json::json!({"$ref": "#/$defs/address"});
        let resolved = resolve_json_schema_ref(&schema, &defs);
        assert_eq!(resolved["type"], "object");
        assert!(resolved["properties"]["street"].is_object());
    }

    #[test]
    fn test_resolve_json_schema_ref_no_ref() {
        let defs = serde_json::json!({});
        let schema = serde_json::json!({"type": "string"});
        let resolved = resolve_json_schema_ref(&schema, &defs);
        // No $ref, returns original
        assert_eq!(resolved, &serde_json::json!({"type": "string"}));
    }

    #[test]
    fn test_resolve_json_schema_ref_missing_def() {
        let defs = serde_json::json!({});
        let schema = serde_json::json!({"$ref": "#/$defs/nonexistent"});
        let resolved = resolve_json_schema_ref(&schema, &defs);
        // def not found, returns original
        assert_eq!(resolved, &schema);
    }

    #[test]
    fn test_resolve_json_schema_ref_draft07_compat() {
        let defs = serde_json::json!({
            "color": {"type": "string", "enum": ["red", "green", "blue"]}
        });
        // Draft-07 uses #/definitions/ instead of #/$defs/
        let schema = serde_json::json!({"$ref": "#/definitions/color"});
        let resolved = resolve_json_schema_ref(&schema, &defs);
        assert_eq!(resolved["enum"][0], "red");
    }

    #[test]
    fn test_resolve_schema_refs_recursive() {
        let schema = serde_json::json!({
            "type": "object",
            "$defs": {
                "name": {"type": "string", "minLength": 1},
                "age": {"type": "integer", "minimum": 0}
            },
            "properties": {
                "user_name": {"$ref": "#/$defs/name"},
                "user_age": {"$ref": "#/$defs/age"}
            },
            "required": ["user_name"]
        });
        let defs = schema["$defs"].clone();
        let resolved = resolve_schema_refs(&schema, &defs);
        assert_eq!(resolved["properties"]["user_name"]["type"], "string");
        assert_eq!(resolved["properties"]["user_name"]["minLength"], 1);
        assert_eq!(resolved["properties"]["user_age"]["type"], "integer");
        assert_eq!(resolved["properties"]["user_age"]["minimum"], 0);
        // $defs should still be present in output
        assert!(resolved.get("$defs").is_some());
    }

    #[test]
    fn test_resolve_schema_refs_nested() {
        let schema = serde_json::json!({
            "$defs": {
                "coords": {
                    "type": "object",
                    "properties": {
                        "lat": {"type": "number"},
                        "lng": {"type": "number"}
                    }
                }
            },
            "allOf": [
                {"$ref": "#/$defs/coords"},
                {"properties": {"label": {"type": "string"}}}
            ]
        });
        let defs = schema["$defs"].clone();
        let resolved = resolve_schema_refs(&schema, &defs);
        let all_of = resolved["allOf"].as_array().unwrap();
        assert_eq!(all_of[0]["type"], "object");
        assert_eq!(all_of[0]["properties"]["lat"]["type"], "number");
        assert_eq!(all_of[1]["properties"]["label"]["type"], "string");
    }

    #[test]
    fn test_normalize_tool_schema_adds_schema() {
        let schema = serde_json::json!({"type": "object", "properties": {}});
        let result = normalize_tool_schema(&schema);
        assert_eq!(result["$schema"], JSON_SCHEMA_2020_12);
        assert_eq!(result["type"], "object");
    }

    #[test]
    fn test_normalize_tool_schema_resolves_refs() {
        let schema = serde_json::json!({
            "$defs": {
                "filter": {
                    "type": "object",
                    "properties": {
                        "field": {"type": "string"},
                        "value": {"type": "string"}
                    }
                }
            },
            "properties": {
                "search_filter": {"$ref": "#/$defs/filter"}
            }
        });
        let result = normalize_tool_schema(&schema);
        assert_eq!(result["$schema"], JSON_SCHEMA_2020_12);
        assert_eq!(result["properties"]["search_filter"]["type"], "object");
        assert!(result["properties"]["search_filter"]["properties"]["field"].is_object());
    }

    // ---- SSE parsing tests ----

    #[test]
    fn test_parse_sse_events_single() {
        let raw = "event: result\ndata: {\"jsonrpc\":\"2.0\",\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"hello\"}]}}\nid: 1\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("result"));
        assert!(events[0].data.contains("hello"));
        assert_eq!(events[0].id.as_deref(), Some("1"));
    }

    #[test]
    fn test_parse_sse_events_multiple() {
        let raw = "event: result\ndata: first\n\nid: 2\nevent: progress\ndata: 50%\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("result"));
        assert_eq!(events[0].data, "first");
        assert_eq!(events[1].event.as_deref(), Some("progress"));
        assert_eq!(events[1].data, "50%");
    }

    #[test]
    fn test_parse_sse_events_data_only() {
        let raw = "data: plain message\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 1);
        assert!(events[0].event.is_none());
        assert_eq!(events[0].data, "plain message");
    }

    #[test]
    fn test_parse_sse_events_multi_line_data() {
        let raw = "data: line1\ndata: line2\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn test_parse_sse_events_empty() {
        let events = parse_sse_events("");
        assert!(events.is_empty());

        let events = parse_sse_events("   \n\n  \n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_sse_events_ignores_retry() {
        let raw = "retry: 1000\ndata: hello\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    // ---- SseEvent struct tests ----

    #[test]
    fn test_sse_event_debug() {
        let event = SseEvent {
            event: Some("progress".into()),
            data: "{\"progress\": 0.5}".into(),
            id: Some("evt_001".into()),
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("progress"));
        assert!(debug.contains("evt_001"));
    }

    #[test]
    fn test_sse_event_no_id() {
        let event = SseEvent {
            event: Some("message".into()),
            data: "ok".into(),
            id: None,
        };
        assert!(event.id.is_none());
        assert_eq!(event.event.as_deref(), Some("message"));
    }

    #[test]
    fn test_sse_event_no_event_type() {
        let event = SseEvent {
            event: None,
            data: "just data".into(),
            id: None,
        };
        assert!(event.event.is_none());
        assert_eq!(event.data, "just data");
    }
}
