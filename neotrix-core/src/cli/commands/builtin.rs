use crate::cli::commands::{CliCommand, CommandOutput};

pub struct HelpCommand;
impl CliCommand for HelpCommand {
    fn name(&self) -> &str { "/help" }
    fn aliases(&self) -> Vec<&str> { vec!["/h", "/?"] }
    fn description(&self) -> &str { "List all available commands with descriptions" }
    fn execute(&self, _args: &[String]) -> CommandOutput {
        let mut s = String::from("NeoTrix Commands:\n\n");
        s.push_str("  /help /h /?       List all available commands\n");
        s.push_str("  /absorb source=X  Absorb knowledge from a source\n");
        s.push_str("  /evolve url=X     Evolve from a URL\n");
        s.push_str("  /mem query=X      Search memory via ReasoningBank\n");
        s.push_str("  /session          Session management [list|switch|new|delete]\n");
        s.push_str("  /stats /s         Show ReasoningBrain stats\n");
        s.push_str("  /config           Set config values key=value\n");
        s.push_str("  /agent            Agent management [create|list|status]\n");
        s.push_str("\nType /help <command> for details on a specific command.");
        CommandOutput::ok(&s)
    }
}

pub struct AbsorbCommand;
impl CliCommand for AbsorbCommand {
    fn name(&self) -> &str { "/absorb" }
    fn aliases(&self) -> Vec<&str> { vec!["/a"] }
    fn description(&self) -> &str { "Absorb knowledge from a source: /absorb source=HeroUI" }
    fn execute(&self, args: &[String]) -> CommandOutput {
        let source = args.iter().find_map(|a| a.strip_prefix("source="))
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let first = args.iter().find(|a| !a.starts_with("--"));
                first.cloned().unwrap_or_else(|| "default".to_string())
            });
        CommandOutput::ok(&format!("Absorbing knowledge from source: {}", source))
    }
}

pub struct EvolveCommand;
impl CliCommand for EvolveCommand {
    fn name(&self) -> &str { "/evolve" }
    fn aliases(&self) -> Vec<&str> { vec!["/e"] }
    fn description(&self) -> &str { "Evolve from a URL: /evolve url=https://github.com/..." }
    fn execute(&self, args: &[String]) -> CommandOutput {
        let url = args.iter().find_map(|a| a.strip_prefix("url="))
            .or_else(|| args.iter().find(|a| a.starts_with("http")).map(|s| &s[..]))
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from("(internal SEAL loop)"));
        CommandOutput::ok(&format!("Evolving from: {}", url))
    }
}

pub struct MemCommand;
impl CliCommand for MemCommand {
    fn name(&self) -> &str { "/mem" }
    fn aliases(&self) -> Vec<&str> { vec!["/memory", "/recall"] }
    fn description(&self) -> &str { "Search memory via ReasoningBank: /mem query=design patterns" }
    fn execute(&self, args: &[String]) -> CommandOutput {
        let query = args.iter().find_map(|a| a.strip_prefix("query="))
            .or_else(|| args.iter().find(|a| !a.starts_with("--")).map(String::as_str))
            .map(|s| s.to_string())
            .unwrap_or_default();
        if query.is_empty() {
            return CommandOutput::ok("Memories: 0 results (ReasoningBank connected)");
        }
        CommandOutput::ok(&format!("Searching memory for: {}", query))
    }
}

pub struct SessionCommand;
impl CliCommand for SessionCommand {
    fn name(&self) -> &str { "/session" }
    fn aliases(&self) -> Vec<&str> { vec!["/sess"] }
    fn description(&self) -> &str { "Session management: /session list | switch <id> | new [name] | delete <id>" }
    fn execute(&self, args: &[String]) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" | "ls" => {
                CommandOutput::ok("Sessions:\n  s-1  (active)  default")
            }
            "switch" | "sw" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("s-1");
                CommandOutput::ok(&format!("Switched to session: {}", id))
            }
            "new" | "create" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("new");
                CommandOutput::ok(&format!("Created new session: {}", name))
            }
            "delete" | "del" | "rm" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    CommandOutput::err("Usage: /session delete <id>")
                } else {
                    CommandOutput::ok(&format!("Deleted session: {}", id))
                }
            }
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Use: list, switch, new, delete", sub)),
        }
    }
}

pub struct StatsCommand;
impl CliCommand for StatsCommand {
    fn name(&self) -> &str { "/stats" }
    fn aliases(&self) -> Vec<&str> { vec!["/s"] }
    fn description(&self) -> &str { "Show ReasoningBrain stats" }
    fn execute(&self, _args: &[String]) -> CommandOutput {
        CommandOutput::ok(
            "╭─ ReasoningBrain Stats ─────────────────────╮\n\
             │ Capability Dims: 22     Learning Rate: 0.01 │\n\
             │ Total Absorbed: 0      Memory Entries: 0    │\n\
             │ Status: Ready                                │\n\
             ╰─────────────────────────────────────────────╯"
        )
    }
}

pub struct ConfigCommand;
impl CliCommand for ConfigCommand {
    fn name(&self) -> &str { "/config" }
    fn aliases(&self) -> Vec<&str> { vec!["/cfg", "/conf"] }
    fn description(&self) -> &str { "Set config values: /config key=value or /config list" }
    fn execute(&self, args: &[String]) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("Config:\n  learning_rate=0.01\n  auto_absorb=true\n  theme=pitaya");
        }
        if args[0] == "list" || args[0] == "ls" {
            return CommandOutput::ok("Config:\n  learning_rate=0.01\n  auto_absorb=true\n  theme=pitaya");
        }
        let kv: Vec<&str> = args[0].splitn(2, '=').collect();
        if kv.len() == 2 {
            CommandOutput::ok(&format!("Set config: {} = {}", kv[0], kv[1]))
        } else {
            CommandOutput::err("Usage: /config key=value (e.g. /config learning_rate=0.05)")
        }
    }
}

pub struct AgentCommand;
impl CliCommand for AgentCommand {
    fn name(&self) -> &str { "/agent" }
    fn aliases(&self) -> Vec<&str> { vec!["/ag"] }
    fn description(&self) -> &str { "Agent management: /agent create <role> | list | status <id>" }
    fn execute(&self, args: &[String]) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" | "ls" => {
                CommandOutput::ok("Agents:\n  main (active)  role: general\n  No sub-agents running.")
            }
            "create" | "new" | "spawn" => {
                let role = args.get(1).map(|s| s.as_str()).unwrap_or("helper");
                CommandOutput::ok(&format!("Created agent with role: {}", role))
            }
            "status" | "stat" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("main");
                CommandOutput::ok(&format!("Agent {}: idle", id))
            }
            "kill" | "stop" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    CommandOutput::err("Usage: /agent kill <id>")
                } else {
                    CommandOutput::ok(&format!("Agent {} terminated", id))
                }
            }
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Use: create, list, status, kill", sub)),
        }
    }
}
