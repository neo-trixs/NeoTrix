//! nt_mcp_registry — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use std::collections::HashMap;
use super::nt_mcp_types::*;
use super::nt_mcp_scan_secrets::scan_secrets_handler;
use super::nt_mcp_audit::audit_code_security_handler;
use super::nt_mcp_deps::check_dependencies_handler;
use super::nt_mcp_injection::test_prompt_injection_handler;
use super::nt_mcp_threat::analyze_threat_handler;
use super::nt_mcp_health::security_health_check_handler;
pub struct SecurityMcpToolRegistry {
    tools: HashMap<String, _SecurityMcpTool>,
    pub(crate) scan_history: Vec<_SecurityScanRecord>,
    pub(crate) max_history: usize,
    pub(crate) rate_limiter: _RateLimitState,
}

impl SecurityMcpToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            scan_history: Vec::new(),
            max_history: DEFAULT_MAX_HISTORY,
            rate_limiter: _RateLimitState::new(),
        }
    }

    pub fn register_defaults(&mut self) {
        self.register_tool(_SecurityMcpTool::new(
            "scan_secrets",
            "Scan code or text for hardcoded secrets, API keys, tokens, and passwords. Returns findings with severity High for confirmed secrets.",
            SecurityToolCategory::SecretDetection,
            scan_secrets_handler,
        )).ok();

        self.register_tool(_SecurityMcpTool::new(
            "audit_code_security",
            "Static analysis for OWASP Top 10 security patterns including command injection, SQL injection, path traversal, and unsafe deserialization with CWE mapping.",
            SecurityToolCategory::CodeAudit,
            audit_code_security_handler,
        )).ok();

        self.register_tool(_SecurityMcpTool::new(
            "check_dependencies",
            "Check project dependencies for known vulnerable patterns in package.json, Cargo.toml, or requirements.txt files.",
            SecurityToolCategory::DependencyCheck,
            check_dependencies_handler,
        )).ok();

        self.register_tool(_SecurityMcpTool::new(
            "test_prompt_injection",
            "Test text for prompt injection patterns including jailbreaks, system prompt leaks, role-playing attacks, and delimiter poisoning.",
            SecurityToolCategory::PromptInjectionTest,
            test_prompt_injection_handler,
        )).ok();

        self.register_tool(_SecurityMcpTool::new(
            "analyze_threat",
            "Threat intelligence analysis of IOCs (IP addresses, domains, file hashes). Returns threat context, known associations, and risk assessment.",
            SecurityToolCategory::ThreatIntel,
            analyze_threat_handler,
        )).ok();

        self.register_tool(_SecurityMcpTool::new(
            "security_health_check",
            "Comprehensive security posture summary. Runs all available security tools on the target and returns an aggregated risk score with prioritized findings.",
            SecurityToolCategory::VulnerabilityScan,
            security_health_check_handler,
        )).ok();
    }

    pub fn register_tool(&mut self, tool: _SecurityMcpTool) -> Result<(), String> {
        if self.tools.contains_key(&tool.name) {
            return Err(format!("Tool '{}' is already registered", tool.name));
        }
        self.tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    pub fn execute_tool(
        &mut self,
        name: &str,
        context: &_SecurityMcpContext,
    ) -> Result<_SecurityMcpResponse, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;

        if !self.rate_limiter.is_allowed(name, now) {
            return Err(format!(
                "Rate limit exceeded for tool '{}': max {} calls per minute",
                name, self.rate_limiter.max_calls_per_minute
            ));
        }

        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("Unknown tool: '{}'", name))?;

        let start = std::time::Instant::now();
        let mut response = (tool.handler)(context)?;
        let duration_ms = start.elapsed().as_millis() as u64;
        response.duration_ms = duration_ms;
        response.tool_name = tool.name.clone();

        let critical_count = response
            .findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();

        let target_summary = if context.target.len() > 100 {
            format!("{}...", &context.target[..100])
        } else {
            context.target.clone()
        };

        let record = _SecurityScanRecord {
            timestamp: now,
            tool_name: tool.name.clone(),
            target_summary,
            finding_count: response.findings.len(),
            critical_count,
            risk_score: response.risk_score,
            duration_ms,
        };

        self.scan_history.push(record);
        if self.scan_history.len() > self.max_history {
            self.scan_history.remove(0);
        }

        Ok(response)
    }

    pub fn list_tools(
        &self,
        category_filter: Option<SecurityToolCategory>,
    ) -> Vec<&_SecurityMcpTool> {
        self.tools
            .values()
            .filter(|t| {
                if let Some(ref cat) = category_filter {
                    t.category == *cat
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn get_statistics(&self) -> _SecurityStats {
        let total_scans = self.scan_history.len();
        let total_findings: usize = self.scan_history.iter().map(|r| r.finding_count).sum();
        let critical_findings: usize = self.scan_history.iter().map(|r| r.critical_count).sum();
        let avg_risk = if total_scans > 0 {
            self.scan_history.iter().map(|r| r.risk_score).sum::<f64>() / total_scans as f64
        } else {
            0.0
        };

        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        for record in &self.scan_history {
            *tool_counts.entry(record.tool_name.clone()).or_default() += 1;
        }
        let mut top_tools: Vec<(String, usize)> = tool_counts.into_iter().collect();
        top_tools.sort_by(|a, b| b.1.cmp(&a.1));
        let top_tools = top_tools.into_iter().take(10).collect();

        let last_scan = self.scan_history.last().map(|r| r.timestamp);

        _SecurityStats {
            total_scans,
            total_findings,
            critical_findings,
            average_risk_score: avg_risk,
            top_tools,
            last_scan,
        }
    }

    pub fn _export_scan_history(&self) -> Vec<_SecurityScanRecord> {
        self.scan_history.clone()
    }

    pub fn check_rate_limit(&mut self, tool_name: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.rate_limiter.is_allowed(tool_name, now)
    }
}

impl Default for SecurityMcpToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn _now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

