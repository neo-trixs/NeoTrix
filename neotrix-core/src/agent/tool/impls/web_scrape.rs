use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::neotrix::nt_io_http_factory::{
    BlockingHttpClientAdapter, HttpClientBackend, HttpClientConfig, TlsFingerprint,
};

pub struct WebScrapeTool {
    manifest: ToolManifest,
    client: BlockingHttpClientAdapter,
}

impl WebScrapeTool {
    pub fn new() -> Self {
        let config = HttpClientConfig {
            backend: HttpClientBackend::Simple,
            tls_fingerprint: TlsFingerprint::Chrome116,
            proxy_url: None,
            timeout_secs: 30,
            max_retries: 3,
            extra_headers: vec![],
        };
        let client = crate::neotrix::nt_io_http_factory::create_blocking_http_client(config);
        Self {
            manifest: ToolManifest {
                id: "web_scrape".into(),
                name: "Web Scraper".into(),
                version: "0.2.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec![
                        "tool".to_string(),
                        "--run".to_string(),
                        "web_scrape".to_string(),
                    ],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Scrape web page content from a URL with stealth fingerprint rotation"
                    .into(),
                author: Some("NeoTrix".into()),
            },
            client,
        }
    }

    /// Create with stealth HTTP backend and optional proxy URL.
    pub fn with_stealth(proxy_url: Option<String>) -> Self {
        let config = HttpClientConfig {
            backend: HttpClientBackend::Stealth,
            tls_fingerprint: TlsFingerprint::Chrome116,
            proxy_url,
            timeout_secs: 30,
            max_retries: 3,
            extra_headers: vec![],
        };
        let client = crate::neotrix::nt_io_http_factory::create_blocking_http_client(config);
        Self {
            manifest: ToolManifest {
                id: "web_scrape".into(),
                name: "Web Scraper".into(),
                version: "0.2.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec![
                        "tool".to_string(),
                        "--run".to_string(),
                        "web_scrape".to_string(),
                    ],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Scrape web page content from a URL with stealth fingerprint rotation"
                    .into(),
                author: Some("NeoTrix".into()),
            },
            client,
        }
    }
}

impl Default for WebScrapeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for WebScrapeTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;
        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let method = args.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
        let result = match method {
            "HEAD" => match self.client.get_blocking(url) {
                Ok(resp) => format!("Status: {}\nHeaders:\n{:?}", resp.status_code, resp.headers),
                Err(e) => format!("request error: {}", e),
            },
            _ => match self.client.get_blocking(url) {
                Ok(resp) => {
                    let status = resp.status_code;
                    let body = String::from_utf8(resp.body)
                        .unwrap_or_else(|e| format!("utf8 error: {}", e));
                    format!("Status: {}\n\n--- Response ---\n{}", status, body)
                }
                Err(e) => format!("request error: {}", e),
            },
        };
        Ok(ToolOutput {
            result,
            metadata: HashMap::new(),
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        Ok(())
    }
}
