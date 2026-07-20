use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_world_osint::{OsintConfig, OsintTarget, run_osint};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct OsintCmd;

impl CliCommand for OsintCmd {
    fn name(&self) -> &str { "/osint" }
    fn aliases(&self) -> Vec<&str> { vec!["/recon", "/osint-scan"] }
    fn description(&self) -> &str {
        "OSINT reconnaissance: domain <domain>|email <email>|username <user>|url <url> [--active] [--concurrency N]"
    }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("help");
        let active = args.iter().any(|a| a == "--active");
        let concurrency = args.iter()
            .position(|a| a == "--concurrency")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10);

        let config = OsintConfig {
            concurrency,
            enable_active: active,
            ..Default::default()
        };

        match subcmd {
            "domain" => {
                let domain = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if domain.is_empty() { return CommandOutput::err("Usage: /osint domain <domain>"); }
                let target = OsintTarget::from_domain(domain);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let report = rt.block_on(run_osint(target, config.clone()));
                CommandOutput::ok(&format!("{}", report))
            }
            "email" => {
                let email = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if email.is_empty() || !email.contains('@') { return CommandOutput::err("Usage: /osint email <email>"); }
                let target = OsintTarget::from_email(email);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let report = rt.block_on(run_osint(target, config.clone()));
                CommandOutput::ok(&format!("{}", report))
            }
            "username" => {
                let username = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if username.is_empty() { return CommandOutput::err("Usage: /osint username <username>"); }
                let target = OsintTarget::from_username(username);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let report = rt.block_on(run_osint(target, config.clone()));
                CommandOutput::ok(&format!("{}", report))
            }
            "url" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() { return CommandOutput::err("Usage: /osint url <url>"); }
                let target = OsintTarget { url: Some(url.to_string()), ..Default::default() };
                let rt = tokio::runtime::Runtime::new().unwrap();
                let report = rt.block_on(run_osint(target, config.clone()));
                CommandOutput::ok(&format!("{}", report))
            }
            "help" | _ => CommandOutput::ok("Usage: /osint domain|email|username|url <target> [--active] [--concurrency N]"),
        }
    }
}
