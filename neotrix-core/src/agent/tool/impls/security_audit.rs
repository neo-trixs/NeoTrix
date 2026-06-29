use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::neotrix::nt_shield::audit::SecurityAudit;

pub struct SecurityAuditTool {
    manifest: ToolManifest,
    engine: Option<SecurityAudit>,
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
                    args: vec![
                        "tool".to_string(),
                        "--run".to_string(),
                        "security_audit".to_string(),
                    ],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description:
                    "Run security audit on code or URL — OWASP Top 10:2025 pattern scanning".into(),
                author: Some("NeoTrix".into()),
            },
            engine: None,
        }
    }
}

impl Default for SecurityAuditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for SecurityAuditTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        self.engine = Some(SecurityAudit::new());
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;

        let engine = self.engine.as_ref().ok_or_else(|| ToolError::Runtime {
            id: self.id().into(),
            message: "audit engine not initialized. call start() first".into(),
        })?;

        let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");

        if code.is_empty() {
            let mut checks = Vec::new();
            for rule in &engine.rules {
                checks.push(serde_json::json!({
                    "rule": rule.name,
                    "severity": rule.severity,
                    "description": rule.description,
                    "fix": rule.fix,
                }));
            }
            let result = serde_json::json!({
                "tool": "security_audit",
                "mode": "checklist",
                "rules_count": checks.len(),
                "rules": checks,
            });
            return Ok(ToolOutput {
                result: serde_json::to_string_pretty(&result).unwrap_or_default(),
                metadata: HashMap::new(),
            });
        }

        let findings = engine.scan_file(std::path::Path::new("stdin"), code);

        let by_severity = {
            let mut m: HashMap<&str, usize> = HashMap::new();
            for f in &findings {
                *m.entry(&f.severity).or_default() += 1;
            }
            m
        };

        let result = serde_json::json!({
            "tool": "security_audit",
            "mode": "scan",
            "findings_count": findings.len(),
            "by_severity": by_severity,
            "findings": findings.iter().map(|f| serde_json::json!({
                "line": f.line,
                "severity": f.severity,
                "rule": f.rule,
                "description": f.description,
                "fix": f.fix,
            })).collect::<Vec<_>>(),
        });
        Ok(ToolOutput {
            result: serde_json::to_string_pretty(&result).unwrap_or_default(),
            metadata: HashMap::new(),
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        self.engine = None;
        Ok(())
    }
}
