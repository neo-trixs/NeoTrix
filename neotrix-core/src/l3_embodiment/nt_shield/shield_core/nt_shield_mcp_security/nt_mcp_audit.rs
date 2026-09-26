//! nt_mcp_audit — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
pub(crate) fn audit_code_security_handler(ctx: &_SecurityMcpContext) -> Result<_SecurityMcpResponse, String> {
    let start = std::time::Instant::now();
    let target = &ctx.target;

    if target.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: "Empty target — nothing to audit".to_string(),
            risk_score: 0.0,
            duration_ms: 0,
            tool_name: String::new(),
        });
    }

    let vuln_patterns: &[(&str, &str, &str, FindingSeverity, &str)] = &[
        (
            r"(?i)(?:system|exec|shell_exec|popen|proc_open|subprocess\.run|subprocess\.Popen|cmd\.Run|exec\.Command)\s*\(",
            "command-injection",
            "Potential OS command injection — user-controlled input passed to command execution",
            FindingSeverity::Critical,
            "CWE-78",
        ),
        (
            r"(?i)(?:eval|assert|exec)\s*\(",
            "code-injection",
            "Potential code injection via eval/exec — allows arbitrary code execution",
            FindingSeverity::Critical,
            "CWE-94",
        ),
        (
            r#"(?i)(?:SELECT|INSERT|UPDATE|DELETE)\s+.*?\+\s*['"]"#,
            "sql-injection",
            "Potential SQL injection — string concatenation in SQL query",
            FindingSeverity::Critical,
            "CWE-89",
        ),
        (
            r#"\.format\(\s*['"]"#,
            "format-string-injection",
            "Potential format string injection — user input in format string",
            FindingSeverity::High,
            "CWE-134",
        ),
        (
            r#"(?i)path\.join\s*\(\s*['"].*?['"]"#,
            "path-traversal",
            "Potential path traversal — string concatenation in path construction",
            FindingSeverity::High,
            "CWE-22",
        ),
        (
            r#"(?i)(?:open|read|write|file_get_contents|fs\.readFile|fs\.writeFile)\s*\(\s*['"].*?\+"#,
            "path-traversal-file",
            "Potential path traversal — user input in file operations",
            FindingSeverity::High,
            "CWE-22",
        ),
        (
            r"(?i)(?:pickle\.loads?|yaml\.load\b|json\.loads?\b.*?\bobject_hook|marshal\.loads?|java.*?deserialize|ObjectInputStream|readObject)",
            "unsafe-deserialization",
            "Unsafe deserialization — may lead to remote code execution",
            FindingSeverity::Critical,
            "CWE-502",
        ),
        (
            r"(?i)(?:innerHTML\s*=|dangerouslySetInnerHTML|v-html\s*=)",
            "xss-inner-html",
            "Potential XSS — raw HTML injection into the DOM",
            FindingSeverity::High,
            "CWE-79",
        ),
        (
            r"(?i)(?:document\.write|document\.writeln)\s*\(",
            "xss-document-write",
            "Potential XSS — document.write with user-controlled data",
            FindingSeverity::High,
            "CWE-79",
        ),
        (
            r#"(?i)Authorization:\s*Bearer\s*['"]"#,
            "authorization-injection",
            "Potential authorization header injection",
            FindingSeverity::High,
            "CWE-20",
        ),
        (
            r#"(?i)(?:unsafe|noopener|noopener\s+noreferrer)\s*['"]?,\s*['"](?:blank|_self)"#,
            "unsafe-link-target",
            "Missing rel='noopener noreferrer' on target='_blank' links",
            FindingSeverity::Low,
            "CWE-1021",
        ),
        (
            r"\$\{.*?(?:request|params|query|body|input|user|data).*?\}",
            "template-injection",
            "Potential server-side template injection (SSTI)",
            FindingSeverity::Critical,
            "CWE-1336",
        ),
    ];

    let mut findings = Vec::new();
    for (pattern, category, description, severity, cwe) in vuln_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.find_iter(target) {
                let line_num = target[..cap.start()].lines().count();
                findings.push(SecurityFinding {
                    severity: severity.clone(),
                    category: category.to_string(),
                    description: description.to_string(),
                    location: Some(format!("line {}", line_num + 1)),
                    remediation: Some(match *severity {
                        FindingSeverity::Critical => "Immediately fix this vulnerability. Use parameterized queries, input validation, and avoid dynamic code execution.",
                        FindingSeverity::High => "Review and fix this issue. Sanitize user input and use safe APIs.",
                        _ => "Consider applying defense-in-depth measures.",
                    }.to_string()),
                    cwe_id: Some(cwe.to_string()),
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
        let count_factor = (findings.len() as f64).min(30.0) / 30.0;
        (max_sev * 0.6 + count_factor * 0.4).min(1.0)
    };

    let summary = if findings.is_empty() {
        "No security vulnerabilities detected".to_string()
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
            "Found {} security issue(s): {} critical, {} high, {} medium/low",
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

