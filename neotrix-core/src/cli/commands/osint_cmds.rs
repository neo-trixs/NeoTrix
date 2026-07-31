use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_world_osint::{OsintConfig, OsintTarget, run_osint};

pub struct OsintCmd;

impl OsintCmd {
    fn run_and_persist(target: OsintTarget, config: OsintConfig) -> CommandOutput {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Runtime error: {}", e)),
        };
        let report = rt.block_on(run_osint(target, config));
        let output = format!("{}", report);

        if let Ok(kb) = KnowledgeBase::open(None) {
            let _ = report.write_to_kb(&kb);
        }

        CommandOutput::ok(&output)
    }
}

impl CliCommand for OsintCmd {
    fn name(&self) -> &str { "/osint" }
    fn aliases(&self) -> Vec<&str> { vec!["/recon", "/osint-scan"] }
    fn description(&self) -> &str {
        "OSINT reconnaissance: domain|email|username|url <target> [--active] [--concurrency N]"
    }
    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
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
                Self::run_and_persist(OsintTarget::from_domain(domain), config.clone())
            }
            "email" => {
                let email = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if email.is_empty() || !email.contains('@') { return CommandOutput::err("Usage: /osint email <email>"); }
                Self::run_and_persist(OsintTarget::from_email(email), config.clone())
            }
            "username" => {
                let username = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if username.is_empty() { return CommandOutput::err("Usage: /osint username <username>"); }
                Self::run_and_persist(OsintTarget::from_username(username), config.clone())
            }
            "url" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() { return CommandOutput::err("Usage: /osint url <url>"); }
                Self::run_and_persist(OsintTarget { url: Some(url.to_string()), ..Default::default() }, config.clone())
            }
            _ => CommandOutput::ok("Usage: /osint domain|email|username|url <target> [--active] [--concurrency N]"),
        }
    }
}
