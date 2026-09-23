//! `nt_audit` — append-only 审计 + 脱敏.
//!
//! 移植 openbot `server/src/audit.ts`: refused 必带 rule 名;
//! secret/`tool_result` 永不落明文 transcript. 本地落 SQLite (见 `nt_store`).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 审计裁决.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditDecision {
    Allow,
    Deny,
}

impl AuditDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// 审计事件 (只增不改).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub at: String,
    pub actor: String,
    pub tool: String,
    pub decision: AuditDecision,
    pub rule: Option<String>,
    /// 已脱敏明细.
    pub detail: String,
}

impl AuditEvent {
    pub fn new(
        actor: &str,
        tool: &str,
        decision: AuditDecision,
        rule: Option<String>,
        raw_detail: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            at: Utc::now().to_rfc3339(),
            actor: actor.to_owned(),
            tool: tool.to_owned(),
            decision,
            rule,
            detail: redact_detail(raw_detail),
        }
    }
}

/// 脱敏: 命中敏感 key 名的行整行替换为 `[redacted]`.
/// (openbot `sensitiveKeys` 集合的本地子集.)
pub fn redact_detail(raw: &str) -> String {
    const KEYS: [&str; 7] = [
        "api_key",
        "apikey",
        "token",
        "secret",
        "password",
        "authorization",
        "key_encryption_key",
    ];
    let mut out = Vec::new();
    for line in raw.lines() {
        let lowered = line.to_ascii_lowercase();
        if KEYS.iter().any(|key| lowered.contains(key)) {
            out.push("[redacted]".to_owned());
        } else {
            out.push(truncate_line(line));
        }
    }
    if out.is_empty() {
        return String::new();
    }
    out.join("\n")
}

fn truncate_line(line: &str) -> String {
    const LIMIT: usize = 500;
    if line.len() <= LIMIT {
        return line.to_owned();
    }
    let mut cut = LIMIT;
    while cut > 0 && !line.is_char_boundary(cut) {
        cut -= 1;
    }
    if let Some(safe) = line.get(..cut) {
        format!("{safe}…[truncated]")
    } else {
        "…[truncated]".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditEvent, redact_detail};

    #[test]
    fn secrets_are_redacted() {
        let event = AuditEvent::new("bot", "bash", super::AuditDecision::Allow, None, "OPENAI_API_KEY=sk-x\nok=1");
        assert!(event.detail.contains("[redacted]"));
        assert!(!event.detail.contains("sk-x"));
    }

    #[test]
    fn long_lines_truncated() {
        let raw = "x".repeat(600);
        assert!(redact_detail(&raw).contains("[truncated]"));
    }
}
