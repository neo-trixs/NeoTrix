use clap::Subcommand;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_memory_historian::EvidenceRecord;
use crate::neotrix::nt_mind::SelfIteratingBrain;

#[derive(Debug, Subcommand)]
pub enum EvidenceCommand {
    /// List all evidence records with tiers
    List {
        /// Filter by tier (1-5)
        #[arg(short, long)]
        tier: Option<u8>,
    },
    /// Show evidence details
    Get {
        /// Evidence ID
        id: String,
    },
    /// Run Bayesian calibration
    Calibrate,
    /// Export all evidence as JSON
    Export {
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show evidence statistics
    Stats,
}

pub struct EvidenceCmd;

impl CliCommand for EvidenceCmd {
    fn name(&self) -> &str {
        "/evidence"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/ev", "/ewhr"]
    }
    fn description(&self) -> &str {
        "EWHR evidence management: list|get|calibrate|export|stats"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("Usage: /evidence <list|get|calibrate|export|stats> [options]");
        }
        let sub = match args[0].as_str() {
            "list" => EvidenceCommand::List { tier: None },
            "get" if args.len() >= 2 => EvidenceCommand::Get { id: args[1].clone() },
            "calibrate" => EvidenceCommand::Calibrate,
            "export" => EvidenceCommand::Export { output: args.get(1).cloned() },
            "stats" => EvidenceCommand::Stats,
            other => return CommandOutput::err(&format!("Unknown subcommand '{}'. Use: list, get <id>, calibrate, export [path], stats", other)),
        };
        match handle_evidence_command(&sub) {
            Ok(()) => CommandOutput::ok("done"),
            Err(e) => CommandOutput::err(&e),
        }
    }
}

pub fn handle_evidence_command(cmd: &EvidenceCommand) -> Result<(), String> {
    match cmd {
        EvidenceCommand::List { tier } => cmd_list(*tier),
        EvidenceCommand::Get { id } => cmd_get(id),
        EvidenceCommand::Calibrate => cmd_calibrate(),
        EvidenceCommand::Export { output } => cmd_export(output.as_deref()),
        EvidenceCommand::Stats => cmd_stats(),
    }
}

fn get_store() -> Result<crate::neotrix::nt_memory_historian::EvidenceStore, String> {
    crate::neotrix::nt_memory_historian::EvidenceStore::try_open_default()
        .ok_or_else(|| "Failed to open KB for EWHR".into())
}

fn cmd_list(tier_filter: Option<u8>) -> Result<(), String> {
    let store = get_store()?;
    let records = store.list_evidence()?;
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  EWHR Evidence Records  (total: {})               ║", records.len());
    println!("╚══════════════════════════════════════════════════════════╝");

    let filtered: Vec<_> = match tier_filter {
        Some(t) => records.into_iter().filter(|r: &EvidenceRecord| r.tier() as u8 + 1 == t).collect(),
        None => records,
    };

    if filtered.is_empty() {
        println!("  No records found.");
        return Ok(());
    }

    for r in &filtered {
        let tier = r.tier();
        let label = tier.label();
        let conf = r.effective_confidence();
        let risk = r.forgery_risk().total();
        println!("  [{:>3}] {:<30} {:8}  conf={:.0}% risk={:.0}%",
            r.id.chars().take(20).collect::<String>(),
            r.name.chars().take(28).collect::<String>(),
            label, conf * 100.0, risk * 100.0);
    }
    Ok(())
}

fn cmd_get(id: &str) -> Result<(), String> {
    let store = get_store()?;
    match store.get_evidence(id)? {
        None => println!("Evidence '{}' not found.", id),
        Some(r) => {
            let tier = r.tier();
            println!("╔═══════════════════════════════════════════╗");
            println!("║  {}  ", r.name);
            println!("║  {}  (conf={:.0}%)", tier.label(), r.effective_confidence() * 100.0);
            println!("╚═══════════════════════════════════════════╝");
            println!("  ID:       {}", r.id);
            println!("  Era:      {}", r.era);
            println!("  Category: {}", r.category);
            println!("  Location: {:.2}°, {:.2}°", r.latitude, r.longitude);
            println!("  Methods:  {}", r.dating_methods.join(", "));
            println!("  Replications: {}", r.independent_replications);
            println!("  Forge Risk: {:.1}%", r.forgery_risk().total() * 100.0);
            println!("  Desc:     {}", r.description.chars().take(200).collect::<String>());
            println!("  Ref:      {}", r.references.chars().take(200).collect::<String>());
        }
    }
    Ok(())
}

fn cmd_calibrate() -> Result<(), String> {
    let store = get_store()?;
    println!("Running EWHR calibration...");
    let result = store.calibrate()?;
    println!("╔═══════════════════════════════════════════╗");
    println!("║  Calibration Complete                     ║");
    println!("╚═══════════════════════════════════════════╝");
    println!("  Evidence:      {}", result.evidence_count);
    println!("  Links found:   {}", result.links_found);
    println!("  Clusters:      {}", result.clusters_found);
    if result.tier_changes.is_empty() {
        println!("  Tier changes:  none");
    } else {
        println!("  Tier changes:");
        for change in &result.tier_changes {
            println!("    - {}", change);
        }
    }
    Ok(())
}

fn cmd_export(output: Option<&str>) -> Result<(), String> {
    let store = get_store()?;
    let records = store.list_evidence()?;
    let json = serde_json::to_string_pretty(&records)
        .map_err(|e| format!("serialize: {}", e))?;
    match output {
        Some(path) => std::fs::write(path, &json).map_err(|e| format!("write: {}", e)),
        None => { println!("{}", json); Ok(()) },
    }
}

fn cmd_stats() -> Result<(), String> {
    let store = get_store()?;
    let stats = store.stats()?;
    println!("╔═══════════════════════════════════════════╗");
    println!("║  EWHR Statistics                         ║");
    println!("╚═══════════════════════════════════════════╝");
    println!("  Total:       {}", stats.total);
    println!("  T1 (double): {}", stats.t1_count);
    println!("  T2 (single): {}", stats.t2_count);
    println!("  T3 (likely): {}", stats.t3_count);
    println!("  T4 (suspect): {}", stats.t4_count);
    println!("  T5 (forgery): {}", stats.t5_count);
    println!("  Links:       {}", stats.links);
    println!("  Clusters:    {}", stats.clusters);
    Ok(())
}
