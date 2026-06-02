use std::collections::HashMap;
use crate::agent::tool::lifecycle::*;
use crate::core::nt_core_arch::ArchitectAgent;

pub struct ArchitectTool {
    manifest: ToolManifest,
}

impl ArchitectTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "architect".into(),
                name: "Architect Agent".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::FileRead, ToolPermission::FileWrite, ToolPermission::ProcessSpawn, ToolPermission::ShellExec],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "architect".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
//                description: "ArchitectAgent — scan codebase, detect weaknesses, design & implement architecture improvements, verify compilation".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for ArchitectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for ArchitectTool {
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
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let result = exec_architect(&args)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e })?;
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        Ok(())
    }
}

pub fn exec_architect(args: &serde_json::Value) -> Result<String, String> {
    let project_root = args.get("project_root")
        .and_then(|v| v.as_str())
        .unwrap_or(".");
    let cycles = args.get("cycles")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;

    let mut agent = ArchitectAgent::new(project_root);
    let results = agent.run_batch(cycles);

    let mut output = String::new();
    for result in &results {
        output.push_str(&format!("[Cycle {}] {}\n", result.cycle, result.summary));
        for change in &result.file_changes {
            output.push_str(&format!("  → {} ({:?})\n", change.path, change.action));
        }
    }
    output.push_str(&format!("\nTotal cycles: {}\n", results.len()));
    output.push_str(&format!("Status: {}\n", agent.status_summary()));
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architect_tool_manifest() {
        let tool = ArchitectTool::new();
        assert_eq!(tool.id(), "architect");
        assert!(tool.manifest().description.contains("ArchitectAgent"));
    }

    #[test]
    fn test_exec_architect_empty_args() {
        let args = serde_json::json!({});
        let result = exec_architect(&args);
        assert!(result.is_ok());
        assert!(result.expect("result should be ok in test").contains("Total cycles: 1"));
    }
}
