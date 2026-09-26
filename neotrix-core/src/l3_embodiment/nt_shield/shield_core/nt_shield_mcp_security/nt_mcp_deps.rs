//! nt_mcp_deps — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
pub(crate) fn check_dependencies_handler(ctx: &_SecurityMcpContext) -> Result<_SecurityMcpResponse, String> {
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

    let dep_patterns: &[(&str, &str, &str, FindingSeverity)] = &[
        (r#""lodash"\s*:\s*"[<>=~]*\s*4\.17\.[0-9]""#, "lodash-vulnerable",
         "lodash < 4.17.21 has prototype pollution vulnerabilities (CVE-2019-10744, CVE-2020-8203)", FindingSeverity::High),
        (r#""minimist"\s*:\s*"[<>=~]*\s*1\.2\.[0-5]""#, "minimist-vulnerable",
         "minimist < 1.2.6 has prototype pollution (CVE-2021-44906)", FindingSeverity::High),
        (r#""node-fetch"\s*:\s*"[<>=~]*\s*2\.[0-6]\.""#, "node-fetch-vulnerable",
         "node-fetch < 2.6.7 has exposure of sensitive information (CVE-2022-0235)", FindingSeverity::Medium),
        (r#""follow-redirects"\s*:\s*"[<>=~]*\s*1\.14\.[0-7]""#, "follow-redirects-vulnerable",
         "follow-redirects < 1.14.8 has credential leakage (CVE-2022-0536)", FindingSeverity::High),
        (r#"name\s*=\s*['"]?log4j['"]?\s*\n.*version\s*=\s*['"]?2\.[0-9]\."#, "log4j-vulnerable",
         "Apache Log4j 2.x series — potential Log4Shell (CVE-2021-44228) in older versions", FindingSeverity::Critical),
        (r#"name\s*=\s*['"]?spring-core['"]?\s*\n.*version\s*=\s*['"]?5\.3\.(1[0-7]|[0-9])"#, "spring4shell-vulnerable",
         "Spring Framework < 5.3.18 may be vulnerable to Spring4Shell (CVE-2022-22965)", FindingSeverity::Critical),
        (r#""axios"\s*:\s*"[<>=~]*\s*0\.[0-9]+\.[0-9]+""#, "axios-old-version",
         "axios 0.x has known vulnerabilities — upgrade to 1.x", FindingSeverity::Medium),
        (r#""tar"\s*:\s*"[<>=~]*\s*[0-4]\.""#, "tar-vulnerable",
         "tar < 5.x has arbitrary file overwrite vulnerabilities", FindingSeverity::High),
        (r#"name\s*=\s*['"]?openssl['"]?\s*\n.*version\s*=\s*['"]?1\."#, "openssl-old",
         "OpenSSL 1.x should be upgraded to 3.x for latest security patches", FindingSeverity::Medium),
    ];

    let mut findings = Vec::new();
    for (pattern, category, description, severity) in dep_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(target) {
                findings.push(SecurityFinding {
                    severity: severity.clone(),
                    category: category.to_string(),
                    description: description.to_string(),
                    location: Some("dependency file".to_string()),
                    remediation: Some(
                        match *severity {
                            FindingSeverity::Critical => {
                                "Upgrade immediately to the latest patched version."
                            }
                            FindingSeverity::High => "Update dependency to the latest version.",
                            _ => "Consider upgrading for security best practices.",
                        }
                        .to_string(),
                    ),
                    cwe_id: Some("CWE-1104".to_string()),
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
        (max_sev * 0.8 + 0.2).min(1.0)
    };

    let summary = if findings.is_empty() {
        "No known vulnerable dependency patterns detected".to_string()
    } else {
        let critical = findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();
        format!(
            "Found {} potentially vulnerable dependenc(ies): {} critical, {} high/medium",
            findings.len(),
            critical,
            findings
                .iter()
                .filter(|f| f.severity == FindingSeverity::High
                    || f.severity == FindingSeverity::Medium)
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

