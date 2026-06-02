use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::neotrix::nt_mind::lsp_client::LspManager;

pub struct LspTool {
    manifest: ToolManifest,
    manager: std::sync::Mutex<LspManager>,
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
                description: "Run LSP diagnostics on a file path, returning JSON array of diagnostics".into(),
                author: Some("NeoTrix".into()),
            },
            manager: std::sync::Mutex::new(LspManager::new()),
        }
    }
}

impl Default for LspTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for LspTool {
    fn id(&self) -> &str { &self.manifest.id }

    fn manifest(&self) -> &ToolManifest { &self.manifest }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: "missing 'path' argument".into(),
            })?;

        let mut manager = self.manager.lock().map_err(|e| ToolError::Runtime {
            id: self.id().into(),
            message: e.to_string(),
        })?;

        let server_name = manager.detect_and_start(path)
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: format!("no LSP server found for path: {}", path),
            })?;

        let uri = format!("file://{}", path);
        let _ = manager.send_request(&server_name, "textDocument/didOpen", serde_json::json!({
            "textDocument": {
                "uri": uri,
                "languageId": "rust",
                "version": 1,
                "text": "",
            }
        }));

        let resp = manager.send_request(&server_name, "textDocument/diagnostic", serde_json::json!({
            "textDocument": { "uri": uri },
        }));

        let result = match resp {
            Some(val) => serde_json::to_string(&val).unwrap_or_else(|_| "[]".into()),
            None => "[]".into(),
        };

        Ok(ToolOutput { result, metadata: HashMap::new() })
    }
}
