use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_shield::network_enforcer::{global_enforcer, persist_policy};
use crate::neotrix::nt_shield::tool_permissions::NetworkPolicy;

/// `/network` — view and manage network isolation policy.
pub struct NetworkCmd;

impl CliCommand for NetworkCmd {
    fn name(&self) -> &str {
        "/network"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/net"]
    }
    fn description(&self) -> &str {
        "Network policy: /network policy show|set <mode> | allowlist add|remove|list"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            return self.show_policy(want_json);
        }

        let sub = args[0].as_str();
        match sub {
            "policy" => {
                if args.len() < 2 {
                    return CommandOutput::err(
                        "Usage: /network policy show | set <allow-all|deny-all|default-deny>",
                    );
                }
                match args[1].as_str() {
                    "show" | "status" => self.show_policy(want_json),
                    "set" => {
                        if args.len() < 3 {
                            return CommandOutput::err(
                                "Usage: /network policy set <allow-all|deny-all|default-deny>",
                            );
                        }
                        self.set_policy(&args[2], want_json)
                    }
                    other => CommandOutput::err(&format!(
                        "Unknown policy subcommand: {}. Available: show, set",
                        other
                    )),
                }
            }
            "allowlist" | "al" => {
                if args.len() < 2 {
                    return CommandOutput::err(
                        "Usage: /network allowlist add|remove|list [domain]",
                    );
                }
                match args[1].as_str() {
                    "add" => {
                        if args.len() < 3 {
                            return CommandOutput::err("Usage: /network allowlist add <domain>");
                        }
                        self.allowlist_add(&args[2], want_json)
                    }
                    "remove" | "rm" => {
                        if args.len() < 3 {
                            return CommandOutput::err("Usage: /network allowlist remove <domain>");
                        }
                        self.allowlist_remove(&args[2], want_json)
                    }
                    "list" | "ls" => self.allowlist_list(want_json),
                    other => CommandOutput::err(&format!(
                        "Unknown allowlist subcommand: {}. Available: add, remove, list",
                        other
                    )),
                }
            }
            other => CommandOutput::err(&format!(
                "Unknown subcommand: {}. Available: policy, allowlist",
                other
            )),
        }
    }
}

impl NetworkCmd {
    fn show_policy(&self, want_json: bool) -> CommandOutput {
        let enforcer = global_enforcer();
        let policy = enforcer.policy();
        let label = match &policy {
            NetworkPolicy::AllowAll => "allow-all",
            NetworkPolicy::DenyAll => "deny-all",
            NetworkPolicy::DefaultDeny => "default-deny",
            NetworkPolicy::AllowList(_) => "allow-list",
        };
        let domains = match &policy {
            NetworkPolicy::AllowList(d) => d.clone(),
            _ => Vec::new(),
        };
        let mut msg = format!("Network policy: {}", label);
        if !domains.is_empty() {
            msg.push_str(&format!("\n  Allowed domains: {}", domains.join(", ")));
        }
        let out = CommandOutput::ok(&msg);
        if want_json {
            out.with_json(serde_json::json!({
                "policy": label,
                "allowed_domains": domains,
            }))
        } else {
            out
        }
    }

    fn set_policy(&self, mode: &str, want_json: bool) -> CommandOutput {
        let policy = match mode {
            "allow-all" | "allow_all" => NetworkPolicy::AllowAll,
            "deny-all" | "deny_all" => NetworkPolicy::DenyAll,
            "default-deny" | "default_deny" | "default" => NetworkPolicy::DefaultDeny,
            other => {
                return CommandOutput::err(&format!(
                    "Unknown policy mode: {}. Options: allow-all, deny-all, default-deny",
                    other
                ))
            }
        };
        let enforcer = global_enforcer();
        enforcer.set_policy(policy);
        persist_policy();
        let msg = format!("Network policy set to: {}", mode);
        let out = CommandOutput::ok(&msg);
        if want_json {
            out.with_json(serde_json::json!({"policy": mode}))
        } else {
            out
        }
    }

    fn allowlist_add(&self, domain: &str, want_json: bool) -> CommandOutput {
        let enforcer = global_enforcer();
        let mut policy = enforcer.policy();
        match &mut policy {
            NetworkPolicy::AllowList(ref mut list) => {
                if list.contains(&domain.to_string()) {
                    let msg = format!("Domain '{}' already in allowlist", domain);
                    let out = CommandOutput::ok(&msg);
                    return if want_json {
                        out.with_json(serde_json::json!({"domain": domain, "added": false, "reason": "already present"}))
                    } else {
                        out
                    };
                }
                list.push(domain.to_string());
            }
            _ => {
                policy = NetworkPolicy::AllowList(vec![domain.to_string()]);
            }
        }
        enforcer.set_policy(policy);
        persist_policy();
        let msg = format!("Added '{}' to network allowlist", domain);
        let out = CommandOutput::ok(&msg);
        if want_json {
            out.with_json(serde_json::json!({"domain": domain, "added": true}))
        } else {
            out
        }
    }

    fn allowlist_remove(&self, domain: &str, want_json: bool) -> CommandOutput {
        let enforcer = global_enforcer();
        let mut policy = enforcer.policy();
        let removed = match &mut policy {
            NetworkPolicy::AllowList(ref mut list) => {
                let len = list.len();
                list.retain(|d| d != domain);
                list.len() < len
            }
            _ => false,
        };
        if removed {
            enforcer.set_policy(policy);
            persist_policy();
            let msg = format!("Removed '{}' from network allowlist", domain);
            let out = CommandOutput::ok(&msg);
            if want_json {
                out.with_json(serde_json::json!({"domain": domain, "removed": true}))
            } else {
                out
            }
        } else {
            let msg = format!("Domain '{}' not found in allowlist", domain);
            let out = CommandOutput::ok(&msg);
            if want_json {
                out.with_json(
                    serde_json::json!({"domain": domain, "removed": false, "reason": "not found"}),
                )
            } else {
                out
            }
        }
    }

    fn allowlist_list(&self, want_json: bool) -> CommandOutput {
        let enforcer = global_enforcer();
        let policy = enforcer.policy();
        let domains = match &policy {
            NetworkPolicy::AllowList(d) => d.clone(),
            _ => Vec::new(),
        };
        let msg = if domains.is_empty() {
            "Allowlist is empty (no custom domains allowed)".to_string()
        } else {
            format!("Network allowlist:\n  {}", domains.join("\n  "))
        };
        let out = CommandOutput::ok(&msg);
        if want_json {
            out.with_json(serde_json::json!({"allowed_domains": domains}))
        } else {
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_policy_show() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["policy".into(), "show".into()], None);
        assert!(out.success);
        assert!(out.message.contains("Network policy"));
    }

    #[test]
    fn test_network_policy_set_default() {
        let cmd = NetworkCmd;
        let out = cmd.execute(
            &["policy".into(), "set".into(), "default-deny".into()],
            None,
        );
        assert!(out.success);
        assert!(out.message.contains("default-deny"));
        // Reset to default-deny for test isolation
        let enforcer = global_enforcer();
        enforcer.set_policy(NetworkPolicy::DefaultDeny);
    }

    #[test]
    fn test_network_policy_set_deny_all() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["policy".into(), "set".into(), "deny-all".into()], None);
        assert!(out.success);
        assert!(out.message.contains("deny-all"));
        let enforcer = global_enforcer();
        enforcer.set_policy(NetworkPolicy::DefaultDeny);
    }

    #[test]
    fn test_network_policy_set_allow_all() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["policy".into(), "set".into(), "allow-all".into()], None);
        assert!(out.success);
        assert!(out.message.contains("allow-all"));
        let enforcer = global_enforcer();
        enforcer.set_policy(NetworkPolicy::DefaultDeny);
    }

    #[test]
    fn test_network_allowlist_add_and_list() {
        let cmd = NetworkCmd;
        let out = cmd.execute(
            &["allowlist".into(), "add".into(), "test.example.com".into()],
            None,
        );
        assert!(out.success);
        let list = cmd.execute(&["allowlist".into(), "list".into()], None);
        assert!(list.message.contains("test.example.com"));
        let _ = cmd.execute(
            &[
                "allowlist".into(),
                "remove".into(),
                "test.example.com".into(),
            ],
            None,
        );
    }

    #[test]
    fn test_network_allowlist_remove() {
        let cmd = NetworkCmd;
        let _ = cmd.execute(
            &[
                "allowlist".into(),
                "add".into(),
                "remove-test.example.com".into(),
            ],
            None,
        );
        let out = cmd.execute(
            &[
                "allowlist".into(),
                "remove".into(),
                "remove-test.example.com".into(),
            ],
            None,
        );
        assert!(out.success);
        assert!(out.message.contains("Removed"));
    }

    #[test]
    fn test_network_allowlist_list_empty() {
        let cmd = NetworkCmd;
        let enforcer = global_enforcer();
        enforcer.set_policy(NetworkPolicy::DefaultDeny);
        let out = cmd.execute(&["allowlist".into(), "list".into()], None);
        assert!(out.success);
        assert!(out.message.contains("empty") || out.message.contains("Allowlist"));
    }

    #[test]
    fn test_network_policy_unknown_subcommand() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["policy".into(), "unknown".into()], None);
        assert!(!out.success);
    }

    #[test]
    fn test_network_allowlist_unknown_subcommand() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["allowlist".into(), "unknown".into()], None);
        assert!(!out.success);
    }

    #[test]
    fn test_network_policy_set_invalid_mode() {
        let cmd = NetworkCmd;
        let out = cmd.execute(&["policy".into(), "set".into(), "invalid".into()], None);
        assert!(!out.success);
    }
}
