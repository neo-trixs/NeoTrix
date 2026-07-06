//! Absorb 2026-07-03 round 2 session work into KB.
//! Usage: cargo run -p neotrix --bin neotrix-session-ingest-round2
use neotrix::neotrix::nt_memory_kb::nt_memory_resource_ingest::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use rusqlite::Connection;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);
    let conn = Connection::open(db_path.as_str()).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    let mut ingester = ResourceIngester::new(&conn);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "L6 Self Intra-Reflection Engine Connected",
            "L6 Self layer intra-reflection analyzer connected to background loop awareness handler for KB persistence.",
        ).with_key_insights(vec![
            "IntraReflection::analyze computes keyword-overlap coherence from reasoning trace windows",
            "Efficiency = MIN_EXPECTED_STEPS / trace_len, capped at 1.0 with success gates",
            "Mode stability = 1 - (mode_switches / max_possible) from E8 hexagram history",
            "Bottleneck detection: slow steps (>2x mean), high error density clusters, oscillation loops",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "architecture"])
    ).expect("ingest L6 Self");
    println!("Ingested: L6 Self connection (id={})", r.node_id);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "MetaEvolveStage / DGMMetaEvolveStage Pipeline Registration",
            "Two previously commented-out hyperstage pipeline stages now registered in seal_pipeline().",
        ).with_key_insights(vec![
            "MetaEvolveStage: HyperMetaAgent::new(10, true) + HyperAgentArchive with safety gating",
            "DGMMetaEvolveStage: DGMMetaAgent::new(512, 5, 0.1) + archive + GenerativeReplay + SelfReferentialCheck",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "architecture"])
    ).expect("ingest MetaEvolveStage");
    println!("Ingested: MetaEvolveStage registration (id={})", r.node_id);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "Lock Poison 100% — 126/126 Production Lock.expect Hardened",
            "All production lock poison paths hardened to unwrap_or_else. 64 server/shield/Web API + 62 CLI.",
        ).with_key_insights(vec![
            "Stage 1: 22 lock.expect in web_navigator/agent/kanban/consciousness/cost_tracker/server/shield/pipeline",
            "Stage 2: 42 lock.expect in api/connection/perm_chain/browse/tcp_server/shield_enforcer/sandbox",
            "Stage 3: 62 lock.expect in ui_cmds/cost_cmds/budget_cmds/brain_cmds/connector_cmds/schedule_cmds",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "security"])
    ).expect("ingest Lock poison");
    println!("Ingested: Lock poison 100% (id={})", r.node_id);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "L7 Capability Bridge Module",
            "Bridging module l7_capability_impl/mod.rs re-exports core::l7_capability types into neotrix:: namespace.",
        ).with_key_insights(vec![
            "Exposes SkillDoc, SkillMetrics, SkillBank, GroupCoordinator, SkillRegistry, RewardModel",
            "Part of 11 bypass import path fix — eliminates impl/ path leakage",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "architecture"])
    ).expect("ingest L7 Bridge");
    println!("Ingested: L7 Bridge module (id={})", r.node_id);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "4 Pipeline Stage Upgrades",
            "Four pipeline stages upgraded from no-op/limited to full real implementations with KB persistence.",
        ).with_key_insights(vec![
            "GwtAbsorbStage: persists global workspace via kb.kv_set('gwt', 'absorbed', json)",
            "HarnessAdaptStage: mutates capability vector with dimension-specific reward deltas",
            "KnowledgeQualityStage: node/edge ratio, confidence, FTS5 freshness from KB",
            "ConversationDistillStage: trajectory storage in session_logs with structured metadata",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "architecture"])
    ).expect("ingest Pipeline upgrades");
    println!("Ingested: Pipeline upgrades (id={})", r.node_id);

    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "11 Bypass Import Paths Fixed",
            "Deep non-re-export import paths through impl/ directories fixed to use proper re-export paths.",
        ).with_key_insights(vec![
            "l3_spatial/types → through KnowledgeBase::search_within_bounds re-export",
            "l1_map_tile → through ModalityToken re-exports",
            "l8_engine_core → through l8_autonomic_impl re-exports",
        ]).with_tags(vec!["fix", "absorbed-2026-07-03", "architecture"])
    ).expect("ingest Import paths");
    println!("Ingested: Bypass import paths fix (id={})", r.node_id);

    // Relate all as siblings
    let titles = [
        "L6 Self Intra-Reflection Engine Connected",
        "MetaEvolveStage / DGMMetaEvolveStage Pipeline Registration",
        "Lock Poison 100% — 126/126 Production Lock.expect Hardened",
        "L7 Capability Bridge Module",
        "4 Pipeline Stage Upgrades",
        "11 Bypass Import Paths Fixed",
    ];
    for i in 1..titles.len() {
        let _ = ingester.relate_by_title(titles[0], titles[i], RelationType::Supports, 0.7, Some("Sibling fix in same 2026-07-03 round 2 session"));
    }
    println!("Done!");
}
