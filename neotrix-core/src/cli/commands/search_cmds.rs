use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_world_search::WebSearchEngine;

pub struct SearchCmd;

impl CliCommand for SearchCmd {
    fn name(&self) -> &str {
        "/search"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/s"]
    }

    fn description(&self) -> &str {
        "Search the web: /search <query> [-n <count>]"
    }

    fn help_detail(&self) -> Option<String> {
        Some("Search the web for information. Use -n <N> to limit results. Results include title, URL, and snippet for each match.".into())
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("Usage:\n  /search <query>          Search the web\n  /search <query> -n <N>   Limit to N results");
        }

        let mut query_parts = Vec::new();
        let mut count: usize = 5;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-n" || args[i] == "--count" {
                if let Some(n) = args.get(i + 1).and_then(|s| s.parse::<usize>().ok()) {
                    count = n;
                }
                i += 2;
            } else {
                query_parts.push(args[i].clone());
                i += 1;
            }
        }

        let query = query_parts.join(" ");
        if query.is_empty() {
            return CommandOutput::err("Usage: /search <query> [-n <count>]");
        }

        let engine = WebSearchEngine::default();
        match engine.search(&query, count) {
            Ok(results) => {
                if results.is_empty() {
                    return CommandOutput::ok("No results found.");
                }
                let mut msg = format!("Search results for \"{}\":\n\n", query);
                for (i, r) in results.iter().enumerate() {
                    msg.push_str(&format!("{}. {}\n   {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
                }
                CommandOutput::ok(msg.trim())
            }
            Err(e) => CommandOutput::err(&format!("Search error: {}", e)),
        }
    }
}
