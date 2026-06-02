use crate::agent::tools::{McpToolDef, McpTransport, McpRegistry};
use crate::neotrix::nt_world_scrape::{RequestScraper, ScraperConfig};
use crate::neotrix::nt_shield::SecurityManager;
use crate::neotrix::nt_shield_audit::{SecurityAuditor, Severity as AuditSeverity};
use crate::neotrix::nt_mind::react_doctor::{
    ReactDoctorEngine, ReactDiagnostic, ReactRuleCategory, RuleSeverity,
};

pub fn neotrix_mcp_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "web_scrape".to_string(),
            description: "Scrape a URL with stealth fingerprint rotation".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to scrape"
                    },
                    "proxy": {
                        "type": "string",
                        "description": "Optional proxy URL"
                    }
                },
                "required": ["url"]
            }),
        },
        McpToolDef {
            name: "nt_shield_audit".to_string(),
            description: "Run nt_shield audit on code or URL".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "File path or URL to audit"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Minimum severity level"
                    }
                },
                "required": ["target"]
            }),
        },
        McpToolDef {
            name: "react_doctor".to_string(),
            description: "Analyze React component health".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to React component file"
                    },
                    "check_types": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Checks to run"
                    }
                },
                "required": ["file_path"]
            }),
        },
        McpToolDef {
            name: "earn".to_string(),
            description: "EarnAgent — generate & publish content across social platforms to earn revenue".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "enum": ["twitter", "wechat", "bilibili", "youtube", "douyin", "github"],
                        "description": "Target platform"
                    },
                    "count": {
                        "type": "integer",
                        "description": "Number of publish cycles",
                        "default": 1
                    },
                    "brand_name": {
                        "type": "string",
                        "description": "Override brand name"
                    }
                },
                "required": []
            }),
        },
        McpToolDef {
            name: "architect".to_string(),
            description: "ArchitectAgent — scan codebase for weaknesses, design & implement architecture improvements, verify compilation — fully autonomous, no external API dependency".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Project root path (default: current dir)"
                    },
                    "cycles": {
                        "type": "integer",
                        "description": "Number of design cycles to run",
                        "default": 1
                    }
                },
                "required": []
            }),
        },
    ]
}

// ========== 内置工具处理器 ==========

fn handle_web_scrape(args: &serde_json::Value) -> Result<String, String> {
    let url = args
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: url".to_string())?;

    let proxy = args.get("proxy").and_then(|v| v.as_str());

    // Auto-prepend https:// if no scheme
    let url = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    let mut config = ScraperConfig::default();
    if let Some(p) = proxy {
        config.proxy = Some(p.to_string());
    }

    let nt_world_scrape = RequestScraper::new(config);
    let result = nt_world_scrape.get(&url);

    let output = if let Some(ref err) = result.error {
        serde_json::json!({
            "result": null,
            "success": false,
            "error": err,
            "url": result.url,
            "status_code": result.status_code,
        })
    } else {
        let text_preview = result
            .text
            .as_deref()
            .map(|t| {
                let max_len = 20000.min(t.len());
                let preview: String = t.chars().take(max_len).collect();
                if t.len() > max_len {
                    format!("{}...(truncated, {} total chars)", preview, t.len())
                } else {
                    preview
                }
            })
            .unwrap_or_default();

        serde_json::json!({
            "result": text_preview,
            "success": true,
            "error": null,
            "url": result.url,
            "status_code": result.status_code,
        })
    };

    serde_json::to_string_pretty(&output).map_err(|e| format!("Serialization error: {}", e))
}

fn handle_nt_shield_audit(args: &serde_json::Value) -> Result<String, String> {
    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: target".to_string())?;

    let min_severity = args
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("low");

    // Run the comprehensive OWASP checklist audit
    let report = SecurityAuditor::run_static(target, target);

    // Filter results by minimum severity
    let severity_filter: u8 = match min_severity {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    };

    fn severity_weight(s: &AuditSeverity) -> u8 {
        match s {
            AuditSeverity::Critical => 4,
            AuditSeverity::High => 3,
            AuditSeverity::Medium => 2,
            AuditSeverity::Low => 1,
            AuditSeverity::Info => 0,
        }
    }

    let checks = SecurityAuditor::checklist();
    let filtered: Vec<serde_json::Value> = checks
        .iter()
        .filter(|c| severity_weight(&c.severity) >= severity_filter)
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "title": c.title,
                "domain": format!("{:?}", c.domain),
                "severity": format!("{:?}", c.severity),
                "description": c.description,
                "remediation": c.remediation,
            })
        })
        .collect();

    // If the target is a local path, run file-level scan too
    let local_findings: Vec<serde_json::Value> = if std::path::Path::new(target).exists() {
        let sec_mgr = SecurityManager::new();
        let findings = if std::path::Path::new(target).is_dir() {
            sec_mgr.audit_project(target)
        } else {
            // Single file: wrap in parent dir scan or scan file directly
            if let Ok(content) = std::fs::read_to_string(target) {
                sec_mgr
                    .audit
                    .scan_file(std::path::Path::new(target), &content)
            } else {
                Vec::new()
            }
        };
        findings
            .into_iter()
            .map(|f| {
                serde_json::json!({
                    "file": f.file.to_string_lossy(),
                    "line": f.line,
                    "severity": f.severity,
                    "rule": f.rule,
                    "description": f.description,
                    "fix": f.fix,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let output = serde_json::json!({
        "result": {
            "project": report.project,
            "total_checks": report.total_checks,
            "score": report.score,
            "filtered_checks": filtered,
            "local_findings": local_findings,
            "note": if !std::path::Path::new(target).exists() {
//                format!("Target '{}' is not a local path — only running OWASP checklist audit. Use a local file path for file-level code scan.", target)
            } else {
//                format!("Audited {} — found {} file-level issues", target, local_findings.len())
            },
        },
        "success": true,
        "error": null,
    });

    serde_json::to_string_pretty(&output).map_err(|e| format!("Serialization error: {}", e))
}

fn handle_react_doctor(args: &serde_json::Value) -> Result<String, String> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: file_path".to_string())?;

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut diagnostics = Vec::new();

    // Pattern-based React diagnostics
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_no = i + 1;

        // Security: dangerouslySetInnerHTML
        if trimmed.contains("dangerouslySetInnerHTML") {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-dangerous-html".to_string(),
                category: ReactRuleCategory::Security,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message: "dangerouslySetInnerHTML without sanitization opens XSS vulnerabilities"
                    .to_string(),
            });
        }

        // Performance: array index as key
        if trimmed.contains("key={i}")
            || trimmed.contains("key={index}")
            || trimmed.contains("key={idx}")
        {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-array-index-as-key".to_string(),
                category: ReactRuleCategory::Performance,
                severity: RuleSeverity::Warning,
                file: file_path.to_string(),
                line: line_no,
                message: "Using array index as key can cause rendering bugs and poor reconciliation"
                    .to_string(),
            });
        }

        // Accessibility: <img without alt
        if trimmed.contains("<img") && !trimmed.contains("alt=") {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-missing-alt".to_string(),
                category: ReactRuleCategory::Accessibility,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message: "Image elements must have alt text for screen readers".to_string(),
            });
        }

        // Architecture: hooks in conditions
        if (line.contains("if ")
            || line.contains("while ")
            || line.contains("for ")
            || line.contains("switch "))
            && (line.contains("useState")
                || line.contains("useEffect")
                || line.contains("useCallback")
                || line.contains("useMemo")
                || line.contains("useRef"))
        {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-hooks-in-conditions".to_string(),
                category: ReactRuleCategory::Architecture,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message:
                    "React hooks must not be called inside conditions, loops, or nested functions"
                        .to_string(),
            });
        }

        // Architecture: direct DOM access
        if trimmed.contains("document.querySelector")
            || trimmed.contains("document.getElementById")
            || trimmed.contains("document.getElementsBy")
        {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-direct-dom-access".to_string(),
                category: ReactRuleCategory::Architecture,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message:
                    "Direct DOM access (document.querySelector) breaks SSR and React abstraction"
                        .to_string(),
            });
        }

        // Security: hardcoded secrets
        if (trimmed.contains("api_key")
            || trimmed.contains("API_KEY")
            || trimmed.contains("GITHUB_TOKEN")
            || trimmed.contains("sk-"))
            && (trimmed.contains('"') || trimmed.contains('\''))
        {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-hardcoded-secrets".to_string(),
                category: ReactRuleCategory::Security,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message: "Hardcoded API keys, tokens, or passwords detected in source code"
                    .to_string(),
            });
        }

        // Architecture: missing useEffect deps (heuristic: useEffect without dep array)
        if trimmed.contains("useEffect(") && !trimmed.contains("],") && !trimmed.contains("])") {
            diagnostics.push(ReactDiagnostic {
                rule_id: "no-missing-deps".to_string(),
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Error,
                file: file_path.to_string(),
                line: line_no,
                message: "useEffect is missing required dependency array".to_string(),
            });
        }
    }

    let report = ReactDoctorEngine::calculate_score(&diagnostics);

    let output = serde_json::json!({
        "result": {
            "score": report.score,
            "label": report.label,
            "total_diagnostics": report.total_diagnostics,
            "error_rules": report.unique_error_rules,
            "warning_rules": report.unique_warning_rules,
            "category_breakdown": report.category_breakdown.iter().map(|cb| serde_json::json!({
                "category": cb.category.name(),
                "count": cb.count,
            })).collect::<Vec<_>>(),
            "diagnostics": report.diagnostics.iter().map(|d| serde_json::json!({
                "rule_id": d.rule_id,
                "category": d.category.name(),
                "severity": format!("{:?}", d.severity),
                "file": d.file,
                "line": d.line,
                "message": d.message,
            })).collect::<Vec<_>>(),
            "react_project": ReactDoctorEngine::detect_react_project(
                std::path::Path::new(file_path).parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or(".")
            ),
        },
        "success": true,
        "error": null,
    });

    serde_json::to_string_pretty(&output).map_err(|e| format!("Serialization error: {}", e))
}

/// Register all built-in tool handlers into an McpRegistry
fn handle_architect(args: &serde_json::Value) -> Result<String, String> {
    use crate::core::nt_core_arch::ArchitectAgent;

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

pub fn register_neotrix_tools(registry: &mut McpRegistry) {
    registry.register_builtin("web_scrape", handle_web_scrape);
    registry.register_builtin("nt_shield_audit", handle_nt_shield_audit);
    registry.register_builtin("react_doctor", handle_react_doctor);
    registry.register_builtin("architect", handle_architect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_count() {
        let tools = neotrix_mcp_tools();
        assert!(tools.len() >= 5);
    }

    #[test]
    fn test_tool_names_and_descriptions() {
        let tools = neotrix_mcp_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"web_scrape"));
        assert!(names.contains(&"nt_shield_audit"));
        assert!(names.contains(&"react_doctor"));
        assert!(names.contains(&"earn"));
        assert!(names.contains(&"architect"));
        for tool in &tools {
            assert!(!tool.description.is_empty());
        }
    }

    #[test]
    fn test_tool_schema_has_required_fields() {
        let tools = neotrix_mcp_tools();
        for tool in &tools {
            let schema = &tool.input_schema;
            assert_eq!(schema["type"], "object");
            assert!(schema["required"].is_array(), "Tool {} missing required array", tool.name);
            assert!(schema["properties"].is_object());
        }
    }

    #[test]
    fn test_tool_transport_type() {
        let tools = neotrix_mcp_tools();
        for tool in &tools {
            assert_eq!(tool.server_name, "built-in");
            match &tool.transport {
                McpTransport::Stdio { command, .. } => {
                    assert_eq!(command, "echo");
                }
                _ => panic!("Expected Stdio transport"),
            }
        }
    }

    #[test]
    fn test_tool_schema_properties() {
        let tools = neotrix_mcp_tools();
        let scrape = tools.iter().find(|t| t.name == "web_scrape").expect("value should be ok in test");
        assert!(scrape.input_schema["properties"]["url"].is_object());
        let audit = tools.iter().find(|t| t.name == "nt_shield_audit").expect("value should be ok in test");
        assert!(audit.input_schema["properties"]["severity"]["enum"].is_array());
    }
}
