use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::permission_profiles::{
    active_profile_name, create_profile, get_profile_info, list_profiles, remove_profile, set_rule,
    switch_profile,
};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct ProfileCmd;
impl CliCommand for ProfileCmd {
    fn name(&self) -> &str {
        "/profile"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/prof"]
    }
    fn description(&self) -> &str {
        "Permission profiles: /profile list | switch <name> | show [name] | create <name> [--parent <parent>] | rm <name> | set <action> <allow|deny|ask>"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain_args: Vec<&str> = args
            .iter()
            .map(|s| s.as_str())
            .filter(|a| *a != "--json")
            .collect();

        if plain_args.is_empty() {
            let active = active_profile_name();
            let names = list_profiles();
            let mut msg = format!("Active profile: {}\n\nAvailable profiles:\n", active);
            for name in names {
                let marker = if name == active { " *" } else { "  " };
                msg.push_str(&format!("  {}{}\n", name, marker));
            }
            let out = CommandOutput::ok(&msg);
            return if want_json {
                out.with_json(serde_json::json!({"active": active, "profiles": list_profiles()}))
            } else {
                out
            };
        }

        let cmd = plain_args[0];
        match cmd {
            "list" => {
                let active = active_profile_name();
                let names = list_profiles();
                let mut msg = format!("Active profile: {}\n\nProfiles:\n", active);
                for name in names {
                    let marker = if name == active { " *" } else { "  " };
                    msg.push_str(&format!("  {}{}\n", name, marker));
                }
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(
                        serde_json::json!({"active": active, "profiles": list_profiles()}),
                    )
                } else {
                    out
                }
            }
            "switch" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err("Usage: /profile switch <name>");
                }
                match switch_profile(plain_args[1]) {
                    Ok(msg) => CommandOutput::ok(&msg),
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "show" | "info" => {
                let name = if plain_args.len() > 1 {
                    plain_args[1]
                } else {
                    &active_profile_name()
                };
                match get_profile_info(name) {
                    Ok(info) => {
                        let active_flag = if name == active_profile_name() {
                            " (active)"
                        } else {
                            ""
                        };
                        let mut msg = format!(
                            "Profile: {}{}\n",
                            info["name"].as_str().unwrap_or(name),
                            active_flag
                        );
                        if let Some(parent) = info["parent"].as_str() {
                            msg.push_str(&format!("  Parent: {}\n", parent));
                        }
                        if let Some(mode) = info["effective_approval_mode"].as_str() {
                            msg.push_str(&format!("  Approval mode: {}\n", mode));
                        }
                        msg.push_str("  Rules:\n");
                        if let Some(rules) = info["effective_rules"].as_object() {
                            let mut keys: Vec<&String> = rules.keys().collect();
                            keys.sort();
                            for key in keys {
                                let val = rules[key].as_str().unwrap_or("deny");
                                msg.push_str(&format!("    {} → {}\n", key, val));
                            }
                        }
                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(info)
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "create" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err("Usage: /profile create <name> [--parent <parent>]");
                }
                let name = plain_args[1];
                let parent_idx = plain_args
                    .iter()
                    .position(|a| *a == "--parent" || *a == "-p");
                let parent = parent_idx.and_then(|i| plain_args.get(i + 1)).copied();
                match create_profile(name, parent) {
                    Ok(msg) => {
                        let _ = switch_profile(name);
                        CommandOutput::ok(&format!("{} (now active)", msg))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "rm" | "remove" | "delete" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err("Usage: /profile rm <name>");
                }
                match remove_profile(plain_args[1]) {
                    Ok(msg) => CommandOutput::ok(&msg),
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "set" => {
                if plain_args.len() < 4 {
                    return CommandOutput::err(
                        "Usage: /profile set <profile> <action> <allow|deny|ask>",
                    );
                }
                let profile_name = plain_args[1];
                let action = plain_args[2];
                let decision = plain_args[3];
                match set_rule(profile_name, action, decision) {
                    Ok(msg) => CommandOutput::ok(&msg),
                    Err(e) => CommandOutput::err(&e),
                }
            }
            _ => CommandOutput::err(&format!(
                "Unknown subcommand: {}. Available: list, switch, show, create, rm, set",
                cmd
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_cmd_no_args() {
        let cmd = ProfileCmd;
        let result = cmd.execute(&[], None);
        assert!(result.success);
        assert!(result.message.contains("Active profile"));
        assert!(result.message.contains("nt_shield"));
    }

    #[test]
    fn test_profile_cmd_list() {
        let cmd = ProfileCmd;
        let result = cmd.execute(&["list".into()], None);
        assert!(result.success);
        assert!(result.message.contains("nt_shield"));
        assert!(result.message.contains("developer"));
    }

    #[test]
    fn test_profile_cmd_show() {
        let cmd = ProfileCmd;
        let result = cmd.execute(&["show".into(), "developer".into()], None);
        assert!(result.success);
        assert!(result.message.contains("developer"));
        assert!(result.message.contains("Rules"));
    }
}
