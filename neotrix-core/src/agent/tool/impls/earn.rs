use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::neotrix::nt_act_earn::{EarnAgent, AgentState, default_registry};

pub struct EarnTool {
    manifest: ToolManifest,
}

impl EarnTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "earn".into(),
                name: "Earn Agent".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "earn".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
//                description: "Run EarnAgent publish cycle — generate & publish content across platforms".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for EarnTool {
    fn default() -> Self { Self::new() }
}

impl AgentTool for EarnTool {
    fn id(&self) -> &str { &self.manifest.id }

    fn manifest(&self) -> &ToolManifest { &self.manifest }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> { Ok(()) }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value = serde_json::from_str(&ctx.input)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e.to_string() })?;
        let result = exec_earn(&args)
            .map_err(|e| ToolError::Runtime { id: self.id().into(), message: e })?;
        Ok(ToolOutput { result, metadata: HashMap::new() })
    }

    fn stop(&mut self) -> Result<(), ToolError> { Ok(()) }
}

fn exec_earn(args: &serde_json::Value) -> Result<String, String> {
    let platform = args.get("platform").and_then(|v| v.as_str());
    let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let brand = args.get("brand_name").and_then(|v| v.as_str());

    let mut agent = if let Some(name) = brand {
        let mut cfg = crate::neotrix::nt_act_earn::StrategyConfig::default();
        cfg.brand_name = name.to_string();
        EarnAgent::with_config(cfg)
    } else {
        EarnAgent::new()
    };
    agent.set_publishers(default_registry());

    let mut results = Vec::new();
    for _ in 0..count {
        let r = agent.run_cycle(platform);
        results.push(r);
    }

    let stats = agent.earnings_stats();
    let json = serde_json::json!({
        "cycles": results.len(),
        "total_earnings": stats.total_earnings,
        "best_platform": stats.best_platform,
        "state": format!("{:?}", agent.state()),
        "platforms": agent.available_platforms(),
    });
    Ok(serde_json::to_string_pretty(&json).unwrap_or_default())
}
