//! MCP 2026-07-28 Stateless Protocol Adapter
//! Removes initialize handshake and session IDs per SEP-2575 and SEP-2567.
//! Adds mandatory Mcp-Method and Mcp-Name headers per SEP-2590.
//! Extensions use reverse-DNS identifiers per SEP-2612.

/// MCP 2026-07-28 stateless request headers
#[derive(Clone, Debug)]
pub struct McpRequestHeaders {
    /// Mandatory: method name (reverse-DNS format, e.g. "tools.call")
    pub method: String,
    /// Mandatory: extension name (reverse-DNS format, e.g. "com.example.my-ext")
    pub name: String,
    /// Optional: request ID for response correlation
    pub request_id: Option<String>,
}

/// MCP 2026-07-28 stateless response
#[derive(Clone, Debug)]
pub struct McpResponse {
    pub request_id: Option<String>,
    pub content: serde_json::Value,
    pub error: Option<String>,
}

/// Stateless MCP client
pub struct McpStatelessClient {
    /// Server URL endpoint
    server_url: String,
    /// HTTP client (reused)
    client: reqwest::Client,
    /// Optional Bearer token for Authorization header (MCP 2026 OAuth 2.0)
    auth_token: Option<String>,
}

impl McpStatelessClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            client: reqwest::Client::new(),
            auth_token: None,
        }
    }

    /// Set Bearer token for Authorization header (OAuth 2.0 / API key)
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Send a stateless MCP request
    pub async fn send_request(&self, headers: McpRequestHeaders, body: serde_json::Value) -> Result<McpResponse, String> {
        let mut req_headers = reqwest::header::HeaderMap::new();
        req_headers.insert(
            "Mcp-Method",
            reqwest::header::HeaderValue::from_str(&headers.method).map_err(|e| format!("Invalid Mcp-Method: {}", e))?,
        );
        req_headers.insert(
            "Mcp-Name",
            reqwest::header::HeaderValue::from_str(&headers.name).map_err(|e| format!("Invalid Mcp-Name: {}", e))?,
        );
        if let Some(ref rid) = headers.request_id {
            req_headers.insert(
                "Mcp-Request-Id",
                reqwest::header::HeaderValue::from_str(rid).map_err(|e| format!("Invalid request ID: {}", e))?,
            );
        }
        req_headers.insert(
            "content-type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        if let Some(ref token) = self.auth_token {
            if let Ok(hv) = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)) {
                req_headers.insert(reqwest::header::AUTHORIZATION, hv);
            }
        }

        let resp = self.client
            .post(&self.server_url)
            .headers(req_headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = resp.status();
        let resp_headers = resp.headers();
        let rid = resp_headers
            .get("Mcp-Request-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Ok(McpResponse {
                request_id: rid,
                content: serde_json::json!({}),
                error: Some(format!("HTTP {}: {}", status, text)),
            });
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("JSON parse: {}", e))?;
        let err = json.get("error").and_then(|e| e.as_str()).map(|s| s.to_string());

        Ok(McpResponse {
            request_id: rid,
            content: json,
            error: err,
        })
    }

    /// Call a tool via stateless MCP
    pub async fn call_tool(&self, tool_name: &str, args: serde_json::Value) -> Result<McpResponse, String> {
        self.send_request(
            McpRequestHeaders {
                method: "tools.call".to_string(),
                name: format!("tools.{}", tool_name),
                request_id: Some(format!("req_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())),
            },
            serde_json::json!({ "arguments": args }),
        ).await
    }

    /// List available tools via stateless MCP
    pub async fn list_tools(&self) -> Result<Vec<String>, String> {
        let resp = self.send_request(
            McpRequestHeaders {
                method: "tools.list".to_string(),
                name: "tools.list".to_string(),
                request_id: None,
            },
            serde_json::json!({}),
        ).await?;
        let tools = resp.content.get("tools")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        Ok(tools)
    }

    /// Health check (replaces old initialize)
    pub async fn ping(&self) -> Result<bool, String> {
        let resp = self.send_request(
            McpRequestHeaders {
                method: "ping".to_string(),
                name: "ping".to_string(),
                request_id: None,
            },
            serde_json::json!({}),
        ).await?;
        Ok(resp.error.is_none())
    }
}

/// Convert old McpClient initialize result to stateless ping
pub fn migrate_initialize_to_ping() -> Vec<String> {
    vec![
        "MCP 2026-07-28: initialize handshake removed per SEP-2575".into(),
        "Use ping() instead for connection check".into(),
        "Session IDs removed per SEP-2567".into(),
        "Use Mcp-Method and Mcp-Name headers instead".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_defaults() {
        let h = McpRequestHeaders {
            method: "tools.call".into(),
            name: "tools.test".into(),
            request_id: None,
        };
        assert_eq!(h.method, "tools.call");
        assert_eq!(h.name, "tools.test");
    }

    #[test]
    fn test_response_no_error() {
        let r = McpResponse {
            request_id: None,
            content: serde_json::json!({"result": "ok"}),
            error: None,
        };
        assert!(r.error.is_none());
        assert_eq!(r.content["result"], "ok");
    }

    #[test]
    fn test_response_with_error() {
        let r = McpResponse {
            request_id: Some("req_1".into()),
            content: serde_json::json!({}),
            error: Some("something went wrong".into()),
        };
        assert!(r.error.is_some());
    }

    #[test]
    fn test_client_creation() {
        let client = McpStatelessClient::new("http://localhost:8080/mcp");
        assert_eq!(client.server_url, "http://localhost:8080/mcp");
    }

    #[test]
    fn test_migration_notes() {
        let notes = migrate_initialize_to_ping();
        assert_eq!(notes.len(), 4);
        assert!(notes[0].contains("SEP-2575"));
        assert!(notes[2].contains("SEP-2567"));
        assert!(notes[3].contains("Mcp-Method"));
    }
}
