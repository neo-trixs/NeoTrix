//! nt_mcp_threat — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
pub(crate) fn analyze_threat_handler(ctx: &_SecurityMcpContext) -> Result<_SecurityMcpResponse, String> {
    let start = std::time::Instant::now();
    let target = &ctx.target;

    if target.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: "Empty target — nothing to analyze".to_string(),
            risk_score: 0.0,
            duration_ms: 0,
            tool_name: String::new(),
        });
    }

    let mut findings = Vec::new();
    let trimmed = target.trim();

    let is_ip = regex::Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$")
        .ok()
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false);

    let is_domain =
        regex::Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$")
            .ok()
            .map(|re| re.is_match(trimmed))
            .unwrap_or(false);

    let is_hash = regex::Regex::new(r"^[a-fA-F0-9]{32,64}$")
        .ok()
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false);

    let is_url = regex::Regex::new(r"^https?://")
        .ok()
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false);

    if is_ip {
        let octets: Vec<u8> = trimmed
            .split('.')
            .filter_map(|o| o.parse::<u8>().ok())
            .collect();

        let is_private = octets.len() == 4
            && (octets[0] == 10
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 168)
                || octets[0] == 127);

        let mut description = if is_private {
            format!("Internal/private IP address: {}", trimmed)
        } else {
            format!("Public IP address: {}", trimmed)
        };

        if is_private {
            description.push_str(" — no external threat context available for private addresses");
        } else {
            description.push_str(" — check threat intelligence feeds for known malicious activity");
        }

        findings.push(SecurityFinding {
            severity: if is_private { FindingSeverity::Info } else { FindingSeverity::Medium },
            category: "ioc-ip".to_string(),
            description,
            location: Some(trimmed.to_string()),
            remediation: Some("Monitor this IP for suspicious activity. Cross-reference with threat intelligence feeds.".to_string()),
            cwe_id: Some("CWE-200".to_string()),
        });
    }

    if is_domain {
        findings.push(SecurityFinding {
            severity: FindingSeverity::Medium,
            category: "ioc-domain".to_string(),
            description: format!("Domain: {} — check reputation and DNS records for malicious indicators", trimmed),
            location: Some(trimmed.to_string()),
            remediation: Some("Verify domain reputation via threat intelligence platforms. Check for typosquatting or lookalike domains.".to_string()),
            cwe_id: Some("CWE-297".to_string()),
        });
    }

    if is_hash {
        let hash_type = match trimmed.len() {
            32 => "MD5",
            40 => "SHA-1",
            64 => "SHA-256",
            _ => "Unknown",
        };
        findings.push(SecurityFinding {
            severity: FindingSeverity::Medium,
            category: "ioc-hash".to_string(),
            description: format!("{} file hash: {} — check against known malware databases", hash_type, trimmed),
            location: Some(trimmed.to_string()),
            remediation: Some("Query VirusTotal or other malware databases for this hash. Check against known IOC feeds.".to_string()),
            cwe_id: Some("CWE-200".to_string()),
        });
    }

    if is_url {
        findings.push(SecurityFinding {
            severity: FindingSeverity::Medium,
            category: "ioc-url".to_string(),
            description: format!("URL: {} — check for phishing, malware distribution, or C2 infrastructure", trimmed),
            location: Some(trimmed.to_string()),
            remediation: Some("Verify URL safety via URL scanning services. Check against known phishing databases.".to_string()),
            cwe_id: Some("CWE-601".to_string()),
        });
    }

    if !is_ip && !is_domain && !is_hash && !is_url {
        findings.push(SecurityFinding {
            severity: FindingSeverity::Low,
            category: "ioc-unknown".to_string(),
            description: format!(
                "Unknown IOC type: '{}' — could not classify the indicator",
                trimmed.len().min(50)
            ),
            location: Some(trimmed.to_string()),
            remediation: Some(
                "Provide a valid IP address, domain, file hash, or URL for threat analysis."
                    .to_string(),
            ),
            cwe_id: Some("CWE-200".to_string()),
        });
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
        max_sev * 0.5
    };

    let ioc_types: Vec<&str> = {
        let mut types = Vec::new();
        if is_ip {
            types.push("IP");
        }
        if is_domain {
            types.push("domain");
        }
        if is_hash {
            types.push("hash");
        }
        if is_url {
            types.push("URL");
        }
        types
    };

    let summary = if ioc_types.is_empty() {
        "No recognized IOC patterns found in target".to_string()
    } else {
        format!(
            "Analyzed {} IOC(s): {}",
            ioc_types.len(),
            ioc_types.join(", ")
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

