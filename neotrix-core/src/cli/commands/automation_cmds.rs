use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_act_autonomy::nt_mind_automation::{
    AutomationEngine, AutomationRule, AutomationTrigger, AutomationAction,
};

pub struct AutomationCmd;

impl CliCommand for AutomationCmd {
    fn name(&self) -> &str {
        "/automation"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/auto", "/trigger"]
    }

    fn description(&self) -> &str {
        "自动化规则管理: /automation [list|add|remove|enable|disable|check]"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");

        match sub {
            "list" | "ls" | "status" => {
                let engine = AutomationEngine::load_persisted();
                let rules = engine.list_rules();
                if rules.is_empty() {
                    return CommandOutput::ok("No automation rules defined. Use /automation add to create one.");
                }
                let mut lines = vec![format!("Automation rules ({}):", rules.len())];
                for rule in rules {
                    let status = if rule.enabled { "enabled" } else { "disabled" };
                    let last = rule.last_run
                        .map(|t| format!("last_run={:?}", t))
                        .unwrap_or_else(|| "never".into());
                    lines.push(format!(
                        "  {} [{}] trigger={} action={} runs={} {}",
                        rule.name, status, rule.trigger, rule.action, rule.run_count, last
                    ));
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            "add" => {
                if args.len() < 4 {
                    return CommandOutput::err("Usage: /automation add <name> <trigger> <action> [args...]\n  trigger: schedule|cron, file:<pattern>, git_push:<branch>, git_pr:<created|updated|merged|closed>, webhook:<path>\n  action: skill:<name>, pipeline, notify:<message>, cmd:<command>");
                }
                let name = &args[1];
                let trigger_str = &args[2];
                let action_str = &args[3];

                let trigger = parse_trigger(trigger_str);
                let trigger = match trigger {
                    Some(t) => t,
                    None => return CommandOutput::err(&format!("Unknown trigger: {}", trigger_str)),
                };

                let action = parse_action(action_str, args.get(4).map(|s| s.as_str()));
                let action = match action {
                    Some(a) => a,
                    None => return CommandOutput::err(&format!("Unknown action: {}", action_str)),
                };

                let rule = AutomationRule::new(name, trigger, action);
                let mut engine = AutomationEngine::load_persisted();
                match engine.add_rule(rule) {
                    Ok(()) => CommandOutput::ok(&format!("Rule '{}' added", name)),
                    Err(e) => CommandOutput::err(&e),
                }
            }

            "remove" | "rm" | "delete" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /automation remove <name>");
                }
                let mut engine = AutomationEngine::load_persisted();
                if engine.remove_rule(name) {
                    CommandOutput::ok(&format!("Rule '{}' removed", name))
                } else {
                    CommandOutput::not_found(&format!("Rule '{}' not found", name))
                }
            }

            "enable" | "on" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /automation enable <name>");
                }
                let mut engine = AutomationEngine::load_persisted();
                if engine.enable_rule(name) {
                    CommandOutput::ok(&format!("Rule '{}' enabled", name))
                } else {
                    CommandOutput::not_found(&format!("Rule '{}' not found", name))
                }
            }

            "disable" | "off" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /automation disable <name>");
                }
                let mut engine = AutomationEngine::load_persisted();
                if engine.disable_rule(name) {
                    CommandOutput::ok(&format!("Rule '{}' disabled", name))
                } else {
                    CommandOutput::not_found(&format!("Rule '{}' not found", name))
                }
            }

            "check" | "run" | "trigger" => {
                let mut engine = AutomationEngine::load_persisted();
                let matched: Vec<_> = engine.check_triggers().into_iter().collect();
                if matched.is_empty() {
                    return CommandOutput::ok("No triggers matched.");
                }
                let mut results = Vec::new();
                let count = matched.len();
                for rule in &matched {
                    match engine.run_rule(rule) {
                        Ok(msg) => results.push(format!("  {}: OK — {}", rule.name, msg)),
                        Err(e) => results.push(format!("  {}: FAIL — {}", rule.name, e)),
                    }
                }
                CommandOutput::ok(&format!("Triggered {} rules:\n{}", count, results.join("\n")))
            }

            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Try: list, add, remove, enable, disable, check", sub)),
        }
    }
}

fn parse_trigger(s: &str) -> Option<AutomationTrigger> {
    if s == "schedule" || s == "cron" || s == "timer" {
        return Some(AutomationTrigger::Schedule { cron: "* * * * *".into() });
    }
    if let Some(pattern) = s.strip_prefix("file:") {
        return Some(AutomationTrigger::FileChange { pattern: pattern.to_string() });
    }
    if let Some(branch) = s.strip_prefix("git_push:") {
        return Some(AutomationTrigger::GitPush { branch: branch.to_string() });
    }
    if let Some(action_str) = s.strip_prefix("git_pr:") {
        let action = match action_str {
            "created" => Some(crate::neotrix::nt_act_autonomy::PrAction::Created),
            "updated" => Some(crate::neotrix::nt_act_autonomy::PrAction::Updated),
            "merged" => Some(crate::neotrix::nt_act_autonomy::PrAction::Merged),
            "closed" => Some(crate::neotrix::nt_act_autonomy::PrAction::Closed),
            _ => None,
        };
        return action.map(|a| AutomationTrigger::GitPr { action: a });
    }
    if let Some(path) = s.strip_prefix("webhook:") {
        return Some(AutomationTrigger::Webhook { path: path.to_string() });
    }
    None
}

fn parse_action(s: &str, extra: Option<&str>) -> Option<AutomationAction> {
    if s == "pipeline" || s == "seal" {
        return Some(AutomationAction::RunSealPipeline);
    }
    if let Some(name) = s.strip_prefix("skill:") {
        return Some(AutomationAction::RunSkill { name: name.to_string() });
    }
    if let Some(message) = s.strip_prefix("notify:") {
        return Some(AutomationAction::Notify { message: message.to_string() });
    }
    if s == "cmd" || s == "command" {
        let cmd = extra.unwrap_or("");
        if cmd.is_empty() {
            return None;
        }
        return Some(AutomationAction::RunCommand { command: cmd.to_string() });
    }
    if let Some(cmd) = s.strip_prefix("cmd:") {
        return Some(AutomationAction::RunCommand { command: cmd.to_string() });
    }
    None
}
