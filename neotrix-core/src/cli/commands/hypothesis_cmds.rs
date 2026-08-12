use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;

pub struct HypothesisCmd;

impl CliCommand for HypothesisCmd {
    fn name(&self) -> &str { "/hypothesis" }
    fn aliases(&self) -> Vec<&str> { vec!["/hyp", "/hnet"] }
    fn description(&self) -> &str { "Manage EWHR hypothesis network: list, propose <id> <title> <prior>, status <id>, strongest" }
    fn is_primary(&self) -> bool { false }


    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("Usage: /hypothesis (list|propose <id> <title> <prior>|status <id>|update <id> <conf> <supports> <strength>|strongest)");
        }

        // Try to access the shared HypothesisNetwork from the reasoning engine
        let shared_net: Option<Arc<Mutex<HypothesisNetwork>>> = brain.and_then(|b| {
            b.try_read().ok().and_then(|brain_guard| {
                brain_guard.reasoning_engine.as_ref()
                    .and_then(|engine| engine.hypothesis_network.clone())
            })
        });

        let store: Arc<Mutex<HypothesisNetwork>> = shared_net.unwrap_or_else(|| {
            Arc::new(Mutex::new(HypothesisNetwork::new()))
        });

        match args[0].as_str() {
            "list" | "ls" | "all" => {
                match store.lock() {
                    Ok(net) => {
                        if net.hypotheses.is_empty() {
                            return CommandOutput::ok("No hypotheses proposed.");
                        }
                        let mut lines = vec![format!("EWHR Hypothesis Network (total: {})", net.hypotheses.len())];
                        for h in &net.hypotheses {
                            lines.push(format!("  [{:>3}] {:<36} posterior={:.3}  status={}",
                                h.id.chars().take(20).collect::<String>(),
                                h.title.chars().take(36).collect::<String>(),
                                h.posterior_probability,
                                h.status.label(),
                            ));
                        }
                        CommandOutput::ok(&lines.join("\n"))
                    }
                    Err(e) => CommandOutput::err(&format!("lock: {}", e)),
                }
            }
            "propose" | "add" | "new" => {
                if args.len() < 4 {
                    return CommandOutput::err("Usage: /hypothesis propose <id> <title> <prior>");
                }
                let id = &args[1];
                let title = &args[2];
                let prior_val: f64 = match args[3].parse() {
                    Ok(v) => v,
                    Err(_) => return CommandOutput::err("prior must be a float (0-1)"),
                };
                let prior = prior_val.max(0.01_f64).min(0.99_f64);
                match store.lock() {
                    Ok(mut net) => {
                        if net.get_hypothesis(id).is_some() {
                            return CommandOutput::err(&format!("Hypothesis '{}' already exists", id));
                        }
                        let h = net.propose_hypothesis(id, title, &format!("Proposed via CLI with prior={}", prior), prior);
                        CommandOutput::ok(&format!("Proposed '{}' (prior={:.3}, posterior={:.3})", h.id, h.prior_probability, h.posterior_probability))
                    }
                    Err(e) => CommandOutput::err(&format!("lock: {}", e)),
                }
            }
            "status" | "get" | "show" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /hypothesis status <id>");
                }
                let id = &args[1];
                match store.lock() {
                    Ok(net) => {
                        match net.get_hypothesis(id) {
                            Some(h) => CommandOutput::ok(&format!(
                                "ID: {}\nTitle: {}\nStatus: {}\nPrior: {:.3}\nPosterior: {:.3}\nSupporting: {:.3}\nRefuting: {:.3}\nEvidence: {}\nSummary: {}",
                                h.id, h.title, h.status.label(), h.prior_probability, h.posterior_probability,
                                h.supporting_weight, h.refuting_weight, h.evidence_ids.len(), h.bayes_factor_summary())),
                            None => CommandOutput::err(&format!("Hypothesis '{}' not found", id)),
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("lock: {}", e)),
                }
            }
            "update" | "evidence" => {
                if args.len() < 5 {
                    return CommandOutput::err("Usage: /hypothesis update <id> <confidence> <supports(0|1)> <strength>");
                }
                let id = &args[1];
                let conf_raw: f64 = match args[2].parse() { Ok(v) => v, Err(_) => return CommandOutput::err("confidence must be 0-1") };
                let conf = conf_raw.max(0.0_f64).min(1.0_f64);
                let supports = args[3] == "1" || args[3].to_lowercase() == "true";
                let strength_raw: f64 = match args[4].parse() { Ok(v) => v, Err(_) => return CommandOutput::err("strength must be 0-1") };
                let strength = strength_raw.max(0.0_f64).min(1.0_f64);
                match store.lock() {
                    Ok(mut net) => {
                        match net.get_hypothesis_mut(id) {
                            Some(h) => {
                                h.update_with_evidence(conf, supports, strength);
                                CommandOutput::ok(&format!("Updated '{}': posterior={:.3} support={:.3} refute={:.3} status={}",
                                    id, h.posterior_probability, h.supporting_weight, h.refuting_weight, h.status.label()))
                            }
                            None => CommandOutput::err(&format!("Hypothesis '{}' not found", id)),
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("lock: {}", e)),
                }
            }
            "strongest" | "best" | "worst" => {
                match store.lock() {
                    Ok(net) => {
                        let supported = net.find_strongest_supported();
                        let refuted = net.find_strongest_refuted();
                        let mut lines = Vec::new();
                        if let Some(h) = supported {
                            lines.push(format!("Strongest supported: '{}' (p={:.3})", h.title, h.posterior_probability));
                        } else {
                            lines.push("No supported hypotheses.".into());
                        }
                        if let Some(h) = refuted {
                            lines.push(format!("Strongest refuted: '{}' (p={:.3})", h.title, h.posterior_probability));
                        } else {
                            lines.push("No refuted hypotheses.".into());
                        }
                        CommandOutput::ok(&lines.join("\n"))
                    }
                    Err(e) => CommandOutput::err(&format!("lock: {}", e)),
                }
            }
            _ => CommandOutput::err(&format!("Unknown subcommand '{}'. Usage: /hypothesis (list|propose|status|update|strongest)", args[0])),
        }
    }
}
