use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;

pub struct WebScrapeTool {
    manifest: ToolManifest,
}

impl WebScrapeTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "web_scrape".into(),
                name: "Web Scraper".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "web_scrape".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Scrape web page content from a URL with stealth fingerprint rotation".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for WebScrapeTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for WebScrapeTool {
    fn id(&self) -> &str { &self.manifest.id }

    fn manifest(&self) -> &ToolManifest { &self.manifest }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let result = crate::neotrix::nt_agent_mcp_tools::exec_web_scrape(&args)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e })?;
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> { Ok(()) }
}
