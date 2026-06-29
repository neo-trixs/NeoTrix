use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::neotrix::nt_mind::lsp_client::LspManager;

pub struct LspTool {
    manifest: ToolManifest,
    manager: std::sync::Mutex<Option<LspManager>>,
}

impl LspTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "lsp_diagnostics".into(),
                name: "LSP Diagnostics".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::ProcessSpawn, ToolPermission::FileRead],
                mcp: None,
                min_runtime: "0.1.0".into(),
                description: "Run LSP diagnostics on a file path, returning JSON array of diagnostics. "
                    .to_string() + "Supports rust-analyzer (Rust), typescript-language-server (TS/JS), pyright (Python). "
                    + "Parameters: path (file to analyze), method (diagnostics|hover|completion|definition).",
                author: Some("NeoTrix".into()),
            },
            manager: std::sync::Mutex::new(None),
        }
    }
}

impl Default for LspTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for LspTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        // Initialize LSP manager with default configs
        let mgr = LspManager::new();
        // Warm-up: detect available LSP servers without starting them
        let _ = mgr.has_server("rust-analyzer");
        *self.manager.lock().map_err(|e| ToolError::Runtime {
            id: self.id().into(),
            message: e.to_string(),
        })? = Some(mgr);
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: "missing 'path' argument".into(),
            })?;

        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("diagnostics");

        let mut guard = self.manager.lock().map_err(|e| ToolError::Runtime {
            id: self.id().into(),
            message: e.to_string(),
        })?;

        let manager = guard.as_mut().ok_or_else(|| ToolError::Runtime {
            id: self.id().into(),
            message: "LSP manager not initialized. call start() first".into(),
        })?;

        let server_name = manager
            .detect_and_start(path)
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: format!("no LSP server found for path: {}", path),
            })?;

        let uri = format!("file://{}", path);

        let _ = manager.send_request(
            &server_name,
            "textDocument/didOpen",
            serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "plaintext",
                    "version": 1,
                    "text": "",
                }
            }),
        );

        let result = match method {
            "hover" => {
                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0);
                let resp = manager.send_request(
                    &server_name,
                    "textDocument/hover",
                    serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": character },
                    }),
                );
                match resp {
                    Some(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".into()),
                    None => "{}".into(),
                }
            }
            "definition" => {
                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0);
                let resp = manager.send_request(
                    &server_name,
                    "textDocument/definition",
                    serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": character },
                    }),
                );
                match resp {
                    Some(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "[]".into()),
                    None => "[]".into(),
                }
            }
            "completion" => {
                let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0);
                let resp = manager.send_request(
                    &server_name,
                    "textDocument/completion",
                    serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": character },
                    }),
                );
                match resp {
                    Some(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "[]".into()),
                    None => "[]".into(),
                }
            }
            _ => {
                let resp = manager.send_request(
                    &server_name,
                    "textDocument/diagnostic",
                    serde_json::json!({
                        "textDocument": { "uri": uri },
                    }),
                );
                match resp {
                    Some(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "[]".into()),
                    None => "[]".into(),
                }
            }
        };

        Ok(ToolOutput {
            result,
            metadata: HashMap::new(),
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        if let Ok(mut guard) = self.manager.lock() {
            *guard = None;
        }
        Ok(())
    }
}
