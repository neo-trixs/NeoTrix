//! nt_mcp_health — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
use super::nt_mcp_scan_secrets::scan_secrets_handler;
use super::nt_mcp_audit::audit_code_security_handler;
use super::nt_mcp_deps::check_dependencies_handler;
use super::nt_mcp_injection::test_prompt_injection_handler;
pub(crate) fn security_health_check_handler(
    ctx: &_SecurityMcpContext,
) -> Result<_SecurityMcpResponse, String> {
    let start = std::time::Instant::now();
    let target = &ctx.target;

    if target.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: "Empty target — nothing to check".to_string(),
            risk_score: 0.0,
            duration_ms: 0,
            tool_name: String::new(),
        });
    }

    let secrets_result = scan_secrets_handler(ctx)?;
    let audit_result = audit_code_security_handler(ctx)?;
    let deps_result = check_dependencies_handler(ctx)?;
    let injection_result = test_prompt_injection_handler(ctx)?;

    let mut all_findings = Vec::new();
    all_findings.extend(secrets_result.findings);
    all_findings.extend(audit_result.findings);
    all_findings.extend(deps_result.findings);
    all_findings.extend(injection_result.findings);

    if all_findings.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: format!(
                "Security health check passed — no issues found in '{}'",
                if target.len() > 50 {
                    format!("{}...", &target[..50])
                } else {
                    target.to_string()
                }
            ),
            risk_score: 0.0,
            duration_ms: start.elapsed().as_millis() as u64,
            tool_name: String::new(),
        });
    }

    let critical_count = all_findings
        .iter()
        .filter(|f| f.severity == FindingSeverity::Critical)
        .count();
    let high_count = all_findings
        .iter()
        .filter(|f| f.severity == FindingSeverity::High)
        .count();
    let medium_count = all_findings
        .iter()
        .filter(|f| f.severity == FindingSeverity::Medium)
        .count();

    let max_sev = all_findings
        .iter()
        .map(|f| f.severity.numeric())
        .max()
        .unwrap_or(0) as f64
        / 4.0;
    let density = (all_findings.len() as f64).min(50.0) / 50.0;
    let risk_score = (max_sev * 0.5 + density * 0.5).min(1.0);

    let summary = format!("Security health check for '{}': {} total findings ({} critical, {} high, {} medium, {} low). Risk score: {:.2}",
        if target.len() > 50 { format!("{}...", &target[..50]) } else { target.to_string() },
        all_findings.len(), critical_count, high_count, medium_count,
        all_findings.iter().filter(|f| f.severity == FindingSeverity::Low || f.severity == FindingSeverity::Info).count(),
        risk_score);

    Ok(_SecurityMcpResponse {
        findings: all_findings,
        summary,
        risk_score,
        duration_ms: start.elapsed().as_millis() as u64,
        tool_name: String::new(),
    })
}
