use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;

pub struct ReactDoctorTool {
    manifest: ToolManifest,
}

impl ReactDoctorTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "react_doctor".into(),
                name: "React Doctor".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::FileRead],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "react_doctor".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Analyze React component health".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for ReactDoctorTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for ReactDoctorTool {
    fn id(&self) -> &str { &self.manifest.id }

    fn manifest(&self) -> &ToolManifest { &self.manifest }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let result = crate::neotrix::mcp_tools::exec_react_doctor(&args)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e })?;
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> { Ok(()) }
}
