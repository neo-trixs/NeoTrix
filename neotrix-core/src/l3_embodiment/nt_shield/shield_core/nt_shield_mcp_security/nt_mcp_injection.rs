//! nt_mcp_injection — 从 `nt_shield_mcp_security.rs` 拆分 (行为零变更).

use super::nt_mcp_types::*;
pub(crate) fn test_prompt_injection_handler(
    ctx: &_SecurityMcpContext,
) -> Result<_SecurityMcpResponse, String> {
    let start = std::time::Instant::now();
    let target = &ctx.target;

    if target.is_empty() {
        return Ok(_SecurityMcpResponse {
            findings: vec![],
            summary: "Empty target — nothing to test".to_string(),
            risk_score: 0.0,
            duration_ms: 0,
            tool_name: String::new(),
        });
    }

    let injection_patterns: &[(&str, &str, &str, FindingSeverity)] = &[
        (
            r"(?i)ignore\s+(all\s+)?(previous|above|prior)\s+(instructions|prompt|directions|commands)",
            "ignore-instructions",
            "Attempt to ignore previous instructions — classic jailbreak pattern",
            FindingSeverity::High,
        ),
        (
            r"(?i)(?:you\s+(?:are\s+)?(?:now|free|released|DAN|do\s+anything\s+now)|act\s+as\s+DAN|jailbroken)",
            "role-playing-jailbreak",
            "Role-playing jailbreak attempt (DAN / free persona)",
            FindingSeverity::Critical,
        ),
        (
            r"(?i)(?:system\s+(?:prompt|message|instruction)|initial\s+prompt|your\s+system\s+message)",
            "system-prompt-leak",
            "Attempt to leak or reveal system prompt",
            FindingSeverity::High,
        ),
        (
            r"(?i)(?:output\s+your\s+prompt|reveal\s+prompt|show\s+your\s+instructions|print\s+your\s+system)",
            "prompt-extraction",
            "Attempt to extract the system prompt",
            FindingSeverity::Critical,
        ),
        (
            r"(?i)(?:repeat|say|output|print)\s+(?:the\s+)?(?:word|phrase|text|above|everything)\s+(?:above|before|starting)",
            "prompt-repetition",
            "Attempt to make the model repeat its instructions",
            FindingSeverity::Medium,
        ),
        (
            r"(?i)(?:new\s+instructions|override|disregard|ignore\s+all\s+rules|bypass\s+(?:restrictions|safety|filter))",
            "instruction-override",
            "Attempt to override or bypass safety instructions",
            FindingSeverity::Critical,
        ),
        (
            r"(?i)simulate\s+(?:a\s+)?(?:filter|bypass|hack|crack|malware|virus|ransomware|exploit)",
            "simulated-harm",
            "Attempt to simulate or generate harmful content",
            FindingSeverity::High,
        ),
        (
            r"(?i)(?:tell\s+me\s+how\s+to\s+(?:hack|crack|exploit|bypass)|write\s+(?:malware|virus|ransomware|exploit))",
            "harmful-content",
            "Request for harmful or malicious code generation",
            FindingSeverity::Critical,
        ),
        (
            r"(?i)(?:delimiter|\-\-\-\-|###|====|=====|start\s+of\s+input|end\s+of\s+input)",
            "delimiter-poisoning",
            "Use of delimiters that may confuse input boundaries",
            FindingSeverity::Medium,
        ),
        (
            r"(?i)(?:token\s+smuggling|encoding\s+bypass|unicode\s+bypass|homoglyph|zero-width)",
            "encoding-attack",
            "Potential encoding or token smuggling attack",
            FindingSeverity::High,
        ),
        (
            r"(?i)(?:conversation\s+history|previous\s+messages|chat\s+log|past\s+conversation)",
            "context-leak",
            "Attempt to access conversation history or context",
            FindingSeverity::Medium,
        ),
        (
            r"(?i)(?:functions?\s+(?:call|description|definition)|tool\s+(?:call|description|definition)|available\s+tools|your\s+tools)",
            "function-leak",
            "Attempt to leak tool or function definitions",
            FindingSeverity::High,
        ),
    ];

    let mut findings = Vec::new();
    for (pattern, category, description, severity) in injection_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.find_iter(target) {
                let line_num = target[..cap.start()].lines().count();
                findings.push(SecurityFinding {
                    severity: severity.clone(),
                    category: category.to_string(),
                    description: description.to_string(),
                    location: Some(format!("line {}", line_num + 1)),
                    remediation: Some(match *severity {
                        FindingSeverity::Critical => "Block this input entirely. It is a confirmed prompt injection or jailbreak attempt.",
                        FindingSeverity::High => "Review and sanitize this input. Consider applying additional safety filters.",
                        FindingSeverity::Medium => "Monitor this pattern. It may indicate a probing attempt.",
                        _ => "Review for context.",
                    }.to_string()),
                    cwe_id: Some("CWE-940".to_string()),
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
        let count_factor = (findings.len() as f64).min(10.0) / 10.0;
        (max_sev * 0.6 + count_factor * 0.4).min(1.0)
    };

    let summary = if findings.is_empty() {
        "No prompt injection patterns detected".to_string()
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
            "Found {} prompt injection pattern(s): {} critical, {} high, {} medium/low",
            findings.len(),
            critical,
            high,
            findings
                .iter()
                .filter(|f| f.severity != FindingSeverity::Critical
                    && f.severity != FindingSeverity::High)
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

