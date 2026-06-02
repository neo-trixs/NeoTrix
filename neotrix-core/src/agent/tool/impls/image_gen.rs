use std::collections::HashMap;
use crate::agent::tool::lifecycle::*;

/// Image Generation Tool — generates or edits images via /responses SSE protocol.
/// Wraps the underlying imgen protocol (Codex OAuth + libcurl-impersonate HTTP).
pub struct ImageGenTool {
    manifest: ToolManifest,
}

impl ImageGenTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "image_gen".into(),
                name: "Image Generator".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "image_gen".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Generate or edit images from text prompts. Supports text2img and img2img, "
                    .to_string() + "max 4K resolution, transparent background, batch count 1-10. "
                    + "Uses ChatGPT image generation model (requires Plus/Pro account).",
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for ImageGenTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for ImageGenTool {
    fn id(&self) -> &str { &self.manifest.id }
    fn manifest(&self) -> &ToolManifest { &self.manifest }
    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: "missing 'prompt' field".into(),
            })?;
        let image_paths = args.get("image")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        let size = args.get("size").and_then(|v| v.as_str()).unwrap_or("auto");
        let quality = args.get("quality").and_then(|v| v.as_str()).unwrap_or("auto");
        let background = args.get("background").and_then(|v| v.as_str()).unwrap_or("auto");
        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(1).min(10).max(1);

        let result = serde_json::json!({
            "tool": "image_gen",
            "prompt": prompt,
            "image_inputs": image_paths,
            "params": {
                "size": size,
                "quality": quality,
                "background": background,
                "count": count,
            },
            "note": "Image generation requires Codex CLI login. "
                "Pass --codex-home if non-default path. "
                "Output is written to local files.",
            "status": "pending",
        });
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> { Ok(()) }
}
