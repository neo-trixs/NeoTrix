//! nt_mcp_scan_secrets — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
pub(crate) fn scan_secrets_handler(ctx: &_SecurityMcpContext) -> Result<_SecurityMcpResponse, String> {
    let start = std::time::Instant::now();
    let target = &ctx.target;

    // 遗留3 (自审修复): SecretCollector 生产接线 — target 若为存在的目录/文件路径,
    // 走多层 Collector (env + dir 递归, file:line 定位, 复用 Redactor 正则库)。
    // 此前 SecretCollector 只有测试调用, 属剧场模块; 此处挂入 scan_secrets 工具生产路径。
    let path = std::path::Path::new(target);
    if !target.is_empty() && path.exists() {
        use crate::l3_embodiment::nt_shield::shield_core::nt_shield_secret_collector::SecretCollector;
        let collector = SecretCollector::new();
        let report = collector.collect(if path.is_dir() { Some(path) } else { None });
        let mut findings = Vec::new();
        for hit in &report.hits {
            findings.push(SecurityFinding {
                severity: if hit.exposed { FindingSeverity::High } else { FindingSeverity::Medium },
                category: "hardcoded-secret".to_string(),
                description: format!("Secret detected in {} ({})", hit.location, hit.rule),
                location: Some(match hit.line {
                    Some(l) => format!("{}:{}", hit.location, l),
                    None => hit.location.clone(),
                }),
                remediation: Some("Remove this credential from code. Store in environment variables or a secrets manager.".to_string()),
                cwe_id: Some("CWE-798".to_string()),
            });
        }
        let risk_score = if findings.is_empty() {
            0.0
        } else {
            (findings.len() as f64).min(20.0) / 20.0 * 0.8
        };
        let summary = format!(
            "Scanned {} ({} secrets found, {} hits)",
            target,
            report.total,
            findings.len(),
        );
        return Ok(_SecurityMcpResponse {
            findings,
            summary,
            risk_score,
            duration_ms: start.elapsed().as_millis() as u64,
            tool_name: String::new(),
        });
    }

    if target.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: "Empty target — nothing to scan".to_string(),
            risk_score: 0.0,
            duration_ms: 0,
            tool_name: String::new(),
        });
    }

    let secret_patterns: &[(&str, &str, &str, FindingSeverity, Option<&str>)] = &[
        (
            r"sk-[a-zA-Z0-9_-]{20,}",
            "openai-api-key",
            "OpenAI API key detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"sk-[a-fA-F0-9]{32,}",
            "stripe-api-key",
            "Stripe API key detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"ghp_[a-zA-Z0-9]{36}",
            "github-pat",
            "GitHub Personal Access Token detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"github_pat_[a-zA-Z0-9]{36}",
            "github-fine-grained-pat",
            "GitHub fine-grained PAT detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"gho_[a-zA-Z0-9]{36}",
            "github-oauth-token",
            "GitHub OAuth access token detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"AKIA[0-9A-Z]{16}",
            "aws-access-key",
            "AWS Access Key ID detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[a-zA-Z0-9/+]{40}"#,
            "aws-secret-key",
            "AWS Secret Access Key detected",
            FindingSeverity::Critical,
            Some("CWE-798"),
        ),
        (
            r"-----BEGIN (RSA |EC )?PRIVATE KEY-----",
            "private-key",
            "Private key block detected",
            FindingSeverity::Critical,
            Some("CWE-312"),
        ),
        (
            r"-----BEGIN CERTIFICATE-----",
            "certificate",
            "Certificate block detected",
            FindingSeverity::Low,
            Some("CWE-312"),
        ),
        (
            r"xox[abpors]-[a-zA-Z0-9]{10,}",
            "slack-token",
            "Slack token detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r#"(?i)api[_-]?key\s*[:=]\s*['"]?[a-zA-Z0-9_\-]{16,}"#,
            "generic-api-key",
            "Generic API key pattern detected",
            FindingSeverity::Medium,
            Some("CWE-798"),
        ),
        (
            r#"(?i)password\s*[:=]\s*['"]?[^'"\s]{8,}"#,
            "hardcoded-password",
            "Hardcoded password detected",
            FindingSeverity::High,
            Some("CWE-259"),
        ),
        (
            r#"(?i)secret\s*[:=]\s*['"]?[a-zA-Z0-9_\-]{16,}"#,
            "hardcoded-secret",
            "Hardcoded secret detected",
            FindingSeverity::Medium,
            Some("CWE-798"),
        ),
        (
            r"ghr_[a-zA-Z0-9]{36}",
            "github-refresh-token",
            "GitHub refresh token detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
        (
            r"glpat-[a-zA-Z0-9\-]{20,}",
            "gitlab-pat",
            "GitLab Personal Access Token detected",
            FindingSeverity::High,
            Some("CWE-798"),
        ),
    ];

    let mut findings = Vec::new();
    for (pattern, category, description, severity, cwe) in secret_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.find_iter(target) {
                let line_num = target[..cap.start()].lines().count();
                findings.push(SecurityFinding {
                    severity: severity.clone(),
                    category: category.to_string(),
                    description: description.to_string(),
                    location: Some(format!("line {}", line_num + 1)),
                    remediation: Some(match *severity {
                        FindingSeverity::Critical => "Immediately rotate this credential. Remove it from source control. Use a secrets manager or environment variables.",
                        FindingSeverity::High => "Remove this credential from code. Store in environment variables or a secrets manager.",
                        FindingSeverity::Medium => "Verify this is not a real secret. If it is, move to environment variables.",
                        _ => "Review this value and ensure it is not a sensitive credential.",
                    }.to_string()),
                    cwe_id: cwe.map(|s| s.to_string()),
                });
            }
        }
    }

    let risk_score = if findings.is_empty() {
        0.0
    } else {
        let max_sev = findings
            .iter()
            .map(|f| f.severity.numeric())
            .max()
            .unwrap_or(0) as f64
            / 4.0;
        let count_factor = (findings.len() as f64).min(20.0) / 20.0;
        (max_sev * 0.7 + count_factor * 0.3).min(1.0)
    };

    let summary = if findings.is_empty() {
        "No secrets detected in target".to_string()
    } else {
        let critical = findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();
        let high = findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::High)
            .count();
        format!(
            "Found {} secret(s): {} critical, {} high, {} medium/low",
            findings.len(),
            critical,
            high,
            findings
                .iter()
                .filter(|f| f.severity == FindingSeverity::Medium
                    || f.severity == FindingSeverity::Low
                    || f.severity == FindingSeverity::Info)
                .count()
        )
    };

    Ok(_SecurityMcpResponse {
        findings,
        summary,
        risk_score,
        duration_ms: start.elapsed().as_millis() as u64,
        tool_name: String::new(),
    })
}

