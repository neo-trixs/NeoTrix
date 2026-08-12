//! /redact — 隐私脱敏命令

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct RedactCmd;

impl CliCommand for RedactCmd {
    fn name(&self) -> &str { "/redact" }
    fn aliases(&self) -> Vec<&str> { vec!["/scrub", "/mask"] }
    fn description(&self) -> &str { "Privacy redaction: /redact <text> | /redact check <text> | /redact secrets-only <text>" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        use crate::neotrix::nt_shield::redaction::{Redactor, RiskLevel};
        let redactor = Redactor::new();
        if args.is_empty() {
            return CommandOutput::ok("用法:\n  /redact <text>                脱敏文本 (替换 secrets + PII 为 [REDACTED])\n  /redact check <text>          分析风险等级\n  /redact secrets-only <text>   仅脱敏 secrets (保留 email 等 PII)");
        }
        match args[0].as_str() {
            "check" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /redact check <text>");
                }
                let text = args[1..].join(" ");
                let (level, matched) = redactor.analyze(&text);
                let level_label = match level {
                    RiskLevel::Safe => "🟢 Safe",
                    RiskLevel::Suspicious => "🟡 Suspicious",
                    RiskLevel::Dangerous => "🔴 Dangerous",
                };
                let matches = if matched.is_empty() { "(无命中)".to_string() } else { matched.join(", ") };
                CommandOutput::ok(&format!("🔍 风险等级: {}\n  命中规则: {}", level_label, matches))
            }
            "secrets-only" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /redact secrets-only <text>");
                }
                let text = args[1..].join(" ");
                CommandOutput::ok(&format!("🛡️ 仅脱敏 secrets:\n{}", redactor.redact_secrets_only(&text)))
            }
            _ => {
                let text = args.join(" ");
                CommandOutput::ok(&format!("🛡️ 脱敏结果:\n{}", redactor.redact(&text)))
            }
        }
    }
}
