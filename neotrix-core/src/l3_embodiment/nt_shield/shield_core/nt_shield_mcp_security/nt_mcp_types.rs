//! nt_mcp_types — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use std::collections::HashMap;
pub(crate) const DEFAULT_MAX_HISTORY: usize = 1000;
pub(crate) const DEFAULT_MAX_CALLS_PER_MINUTE: usize = 30;

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityToolCategory {
    VulnerabilityScan,
    SecretDetection,
    CodeAudit,
    DependencyCheck,
    ThreatIntel,
    ComplianceCheck,
    NetworkScan,
    Forensics,
    MalwareAnalysis,
    PromptInjectionTest,
}

impl SecurityToolCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityToolCategory::VulnerabilityScan => "vulnerability-scan",
            SecurityToolCategory::SecretDetection => "secret-detection",
            SecurityToolCategory::CodeAudit => "code-audit",
            SecurityToolCategory::DependencyCheck => "dependency-check",
            SecurityToolCategory::ThreatIntel => "threat-intel",
            SecurityToolCategory::ComplianceCheck => "compliance-check",
            SecurityToolCategory::NetworkScan => "network-scan",
            SecurityToolCategory::Forensics => "forensics",
            SecurityToolCategory::MalwareAnalysis => "malware-analysis",
            SecurityToolCategory::PromptInjectionTest => "prompt-injection-test",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingSeverity::Critical => "critical",
            FindingSeverity::High => "high",
            FindingSeverity::Medium => "medium",
            FindingSeverity::Low => "low",
            FindingSeverity::Info => "info",
        }
    }

    pub fn numeric(&self) -> u8 {
        match self {
            FindingSeverity::Critical => 4,
            FindingSeverity::High => 3,
            FindingSeverity::Medium => 2,
            FindingSeverity::Low => 1,
            FindingSeverity::Info => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: FindingSeverity,
    pub category: String,
    pub description: String,
    pub location: Option<String>,
    pub remediation: Option<String>,
    pub cwe_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct _SecurityMcpContext {
    pub target: String,
    pub parameters: HashMap<String, String>,
    pub depth: String,
    pub user_id: String,
    pub session_id: String,
}

impl _SecurityMcpContext {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            parameters: HashMap::new(),
            depth: "normal".to_string(),
            user_id: String::new(),
            session_id: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct _SecurityMcpResponse {
    pub findings: Vec<SecurityFinding>,
    pub summary: String,
    pub risk_score: f64,
    pub duration_ms: u64,
    pub tool_name: String,
}

#[derive(Debug, Clone)]
pub struct _SecurityScanRecord {
    pub timestamp: u64,
    pub tool_name: String,
    pub target_summary: String,
    pub finding_count: usize,
    pub critical_count: usize,
    pub risk_score: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct _RateLimitState {
    pub calls_per_minute: HashMap<String, Vec<u64>>,
    pub max_calls_per_minute: usize,
}

impl _RateLimitState {
    pub fn new() -> Self {
        Self {
            calls_per_minute: HashMap::new(),
            max_calls_per_minute: DEFAULT_MAX_CALLS_PER_MINUTE,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            calls_per_minute: HashMap::new(),
            max_calls_per_minute: max,
        }
    }

    pub fn is_allowed(&mut self, tool_name: &str, now_ms: u64) -> bool {
        let window_start = now_ms.saturating_sub(60_000);
        let calls = self
            .calls_per_minute
            .entry(tool_name.to_string())
            .or_default();
        calls.retain(|ts| *ts > window_start);
        if calls.len() >= self.max_calls_per_minute {
            return false;
        }
        calls.push(now_ms);
        true
    }
}

impl Default for _RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

pub type _SecurityToolHandler = fn(&_SecurityMcpContext) -> Result<_SecurityMcpResponse, String>;

#[derive(Debug, Clone)]
pub struct _SecurityMcpTool {
    pub name: String,
    pub description: String,
    pub category: SecurityToolCategory,
    pub handler: fn(&_SecurityMcpContext) -> Result<_SecurityMcpResponse, String>,
    pub required_permissions: Vec<String>,
    pub timeout_seconds: u64,
}

impl _SecurityMcpTool {
    pub fn new(
        name: &str,
        description: &str,
        category: SecurityToolCategory,
        handler: fn(&_SecurityMcpContext) -> Result<_SecurityMcpResponse, String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category,
            handler,
            required_permissions: vec![],
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct _SecurityStats {
    pub total_scans: usize,
    pub total_findings: usize,
    pub critical_findings: usize,
    pub average_risk_score: f64,
    pub top_tools: Vec<(String, usize)>,
    pub last_scan: Option<u64>,
}

