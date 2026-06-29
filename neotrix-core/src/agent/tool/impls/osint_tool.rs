use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;
use crate::core::nt_core_knowledge::osint::{
    BinaryAnalysisProbe, DomainProbe, IPProbe, IntelligenceOrchestrator,
};

pub struct OsintInvestigatorTool {
    manifest: ToolManifest,
    orchestrator: Option<IntelligenceOrchestrator>,
}

impl OsintInvestigatorTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "osint_investigate".into(),
                name: "OSINT Investigator".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::Network],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec![
                        "tool".to_string(),
                        "--run".to_string(),
                        "osint_investigate".to_string(),
                    ],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Multi-source OSINT investigation: domain intelligence (DNS/WHOIS/SSL/typosquatting), IP intelligence (classification/ASN/geolocation), and planned username/company/threat probes"
                    .into(),
                author: Some("NeoTrix".into()),
            },
            orchestrator: None,
        }
    }
}

impl Default for OsintInvestigatorTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for OsintInvestigatorTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        let mut orchestrator = IntelligenceOrchestrator::new();
        orchestrator.register_probe(Box::new(DomainProbe::new()));
        orchestrator.register_probe(Box::new(IPProbe::new()));
        orchestrator.register_probe(Box::new(BinaryAnalysisProbe::new()));
        self.orchestrator = Some(orchestrator);
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;

        let orchestrator = self
            .orchestrator
            .as_ref()
            .ok_or_else(|| ToolError::Runtime {
                id: self.id().into(),
                message: "orchestrator not initialized. call start() first".into(),
            })?;

        let goal = args
            .get("goal")
            .and_then(|v| v.as_str())
            .unwrap_or("OSINT investigation");
        let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let probe = args.get("probe").and_then(|v| v.as_str());

        if target.is_empty() {
            return Err(ToolError::Runtime {
                id: self.id().into(),
                message: "missing required argument: target".into(),
            });
        }

        let serialized = if let Some(probe_name) = probe {
            match orchestrator.run_probe(probe_name, target, 30) {
                Some(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "probe": probe_name,
                    "target": target,
                    "success": result.success,
                    "findings": result.findings.iter().map(|f| {
                        serde_json::json!({
                            "key": f.key,
                            "value": f.value,
                            "source": f.source,
                            "confidence": f.confidence,
                            "severity": format!("{:?}", f.severity),
                            "metadata": f.metadata,
                        })
                    }).collect::<Vec<_>>(),
                    "duration_ms": result.duration_ms,
                }))
                .map_err(|e| ToolError::Runtime {
                    id: self.id().into(),
                    message: e.to_string(),
                })?,
                None => {
                    return Err(ToolError::Runtime {
                        id: self.id().into(),
                        message: format!(
                            "unknown probe: {}. available: {:?}",
                            probe_name,
                            orchestrator.list_probes()
                        ),
                    });
                }
            }
        } else {
            let report = orchestrator.investigate(goal, target, 30);
            serde_json::to_string_pretty(&serde_json::json!({
                "goal": report.goal,
                "target": target,
                "total_probes": report.total_probes,
                "successful_probes": report.successful_probes,
                "total_findings": report.total_findings,
                "critical_findings": report.critical_findings,
                "duration_ms": report.duration_ms,
                "findings": report.findings.iter().map(|f| {
                    serde_json::json!({
                        "key": f.key,
                        "value": f.value,
                        "source": f.source,
                        "confidence": f.confidence,
                        "severity": format!("{:?}", f.severity),
                        "metadata": f.metadata,
                    })
                }).collect::<Vec<_>>(),
            }))
            .map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?
        };

        Ok(ToolOutput {
            result: serialized,
            metadata: HashMap::new(),
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        self.orchestrator = None;
        Ok(())
    }
}
