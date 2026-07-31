#![deny(clippy::unwrap_used)]

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_second_brain::{BrainRelationType, SecondBrain};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct BrainCmd;

impl CliCommand for BrainCmd {
    fn name(&self) -> &str { "/brain" }
    fn aliases(&self) -> Vec<&str> { vec!["/sb", "/second-brain"] }
    fn description(&self) -> &str {
        "Second Brain — unified memory graph:\n  /brain status          Show KB snapshot (nodes/edges/emotion)\n  /brain graph <file>    Generate D3.js force-directed graph HTML\n  /brain query <q>       Search all memory (wiki+notes+emotion)\n  /brain link <src> <dst> <relation>  Link two nodes\n  /brain save <note>     Save a session note\n  /brain emotion         Show current emotion state\n  /brain dimensions      Show dimension scores"
    }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("status");
        let want_json = args.iter().any(|a| a == "--json");
        match subcmd {
            "status" => cmd_brain_status(want_json),
            "graph" => cmd_brain_graph(args),
            "query" | "q" | "search" => cmd_brain_query(args),
            "link" => cmd_brain_link(args),
            "save" | "note" => cmd_brain_save(args, brain),
            "emotion" | "em" => cmd_brain_emotion(),
            "dimensions" | "dims" => cmd_brain_dimensions(want_json),
            "help" | _ => CommandOutput::ok(self.description()),
        }
    }
}

fn get_second_brain() -> Result<SecondBrain, String> {
    let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(None)
        .map_err(|e| format!("Failed to open KB: {}", e))?;
    let mut brain = SecondBrain::new();
    brain.attach_kb(Arc::new(kb));
    Ok(brain)
}

fn cmd_brain_status(want_json: bool) -> CommandOutput {
    match get_second_brain().and_then(|mut b| b.status()) {
        Ok(snapshot) => {
            let emotion = snapshot.emotion_json.as_deref().unwrap_or("none");
            let notes = snapshot.session_notes.len();
            let msg = format!(
                "Second Brain Status:\n  Nodes:     {}\n  Edges:     {}\n  Wiki:      {} pages\n  Emotion:   {}\n  Notes:     {} saved\n  Dimensions: {}",
                snapshot.node_count, snapshot.edge_count,
                snapshot.wiki_page_count, emotion, notes,
                snapshot.dimensions.len()
            );
            if want_json {
                CommandOutput::ok(&msg).with_json(serde_json::json!(snapshot))
            } else {
                CommandOutput::ok(&msg)
            }
        }
        Err(e) => CommandOutput::err(&e),
    }
}

fn cmd_brain_graph(args: &[String]) -> CommandOutput {
    let out_path = args.get(1).map(|s| s.as_str()).unwrap_or("brain-graph.html");
    match get_second_brain().and_then(|b| b.generate_graph_html()) {
        Ok(html) => {
            if std::fs::write(out_path, &html).is_ok() {
                CommandOutput::ok(&format!("Brain graph written to {}", out_path))
            } else {
                CommandOutput::err(&format!("Failed to write {}", out_path))
            }
        }
        Err(e) => CommandOutput::err(&e),
    }
}

fn cmd_brain_query(args: &[String]) -> CommandOutput {
    if args.len() < 2 {
        return CommandOutput::err("Usage: /brain query <search term>");
    }
    let query = args[1..].join(" ");
    match get_second_brain().and_then(|b| b.search(&query, 20)) {
        Ok(results) => {
            if results.is_empty() {
                return CommandOutput::ok(&format!("No results for: {}", query));
            }
            let mut msg = format!("Brain search results for \"{}\":\n", query);
            for r in &results {
                let summary = if r.summary.len() > 80 { format!("{}...", &r.summary[..77]) } else { r.summary.clone() };
                msg.push_str(&format!("  [{:.2}] {} — {}\n    {}\n", r.score, r.node_type, r.title, summary));
            }
            if results.iter().any(|r| serde_json::to_value(r).is_ok()) {
                CommandOutput::ok(&msg).with_json(serde_json::json!({"query": query, "results": results}))
            } else {
                CommandOutput::ok(&msg)
            }
        }
        Err(e) => CommandOutput::err(&e),
    }
}

fn cmd_brain_link(args: &[String]) -> CommandOutput {
    if args.len() < 4 {
        return CommandOutput::err("Usage: /brain link <source_id> <target_id> <relation_type>\n  Relations: temporal_sequence, emotional_affinity, semantic_similar, causal_dependency, type_hierarchy, source_provenance, cross_reference");
    }
    let source = &args[1];
    let target = &args[2];
    let relation = match args[3].as_str() {
        "temporal_sequence" => BrainRelationType::TemporalSequence,
        "emotional_affinity" => BrainRelationType::EmotionalAffinity,
        "semantic_similar" => BrainRelationType::SemanticSimilar,
        "causal_dependency" => BrainRelationType::CausalDependency,
        "type_hierarchy" => BrainRelationType::TypeHierarchy,
        "source_provenance" => BrainRelationType::SourceProvenance,
        "cross_reference" => BrainRelationType::CrossReference,
        other => return CommandOutput::err(&format!("Unknown relation: {}. Use one of: temporal_sequence, emotional_affinity, semantic_similar, causal_dependency, type_hierarchy, source_provenance, cross_reference", other)),
    };
    let weight: f64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.8);
    match get_second_brain().and_then(|b| b.link_nodes(source, target, relation, weight)) {
        Ok(()) => CommandOutput::ok(&format!("Linked {} → {} ({})", source, target, args[3])),
        Err(e) => CommandOutput::err(&e),
    }
}

fn cmd_brain_save(args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
    if args.len() < 2 {
        return CommandOutput::err("Usage: /brain save <note text>");
    }
    let note = args[1..].join(" ");
    match get_second_brain().and_then(|b| b.save_note(&note)) {
        Ok(()) => CommandOutput::ok(&format!("Note saved: {}", &note[..note.len().min(80)])),
        Err(e) => CommandOutput::err(&e),
    }
}

fn cmd_brain_emotion() -> CommandOutput {
    match get_second_brain() {
        Ok(b) => {
            let (state, report) = b.read_emotion();
            let state_s = state.unwrap_or_else(|| "No emotion state saved".into());
            let report_s = report.unwrap_or_else(|| "No report".into());
            CommandOutput::ok(&format!("Emotion State:\n  engine: {}\n  report: {}", state_s, report_s))
        }
        Err(e) => CommandOutput::err(&format!("Cannot read emotion: {}", e)),
    }
}

fn cmd_brain_dimensions(want_json: bool) -> CommandOutput {
    match get_second_brain().and_then(|mut b| b.status()) {
        Ok(snapshot) => {
            let mut msg = String::from("Brain Dimensions:\n");
            for d in &snapshot.dimensions {
                msg.push_str(&format!("  {}: {:.2} ({} nodes, {} links)\n", d.name, d.score, d.node_count, d.link_count));
            }
            if want_json {
                CommandOutput::ok(&msg).with_json(serde_json::json!({"dimensions": snapshot.dimensions}))
            } else {
                CommandOutput::ok(&msg)
            }
        }
        Err(e) => CommandOutput::err(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_cmd_basic() {
        let cmd = BrainCmd;
        assert_eq!(cmd.name(), "/brain");
        assert!(cmd.aliases().contains(&"/sb"));
    }
}
