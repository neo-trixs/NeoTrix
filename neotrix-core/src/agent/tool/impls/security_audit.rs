use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;

pub struct SecurityAuditTool {
    manifest: ToolManifest,
}

impl SecurityAuditTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "security_audit".into(),
                name: "Security Auditor".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::FileRead, ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "security_audit".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Run security audit on code or URL".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for SecurityAuditTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for SecurityAuditTool {
    fn id(&self) -> &str { &self.manifest.id }

    fn manifest(&self) -> &ToolManifest { &self.manifest }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, _ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let result = "Security audit: not available in this build (MCP removed)".to_string();
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> { Ok(()) }
}
