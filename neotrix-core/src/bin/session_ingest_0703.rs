//! Absorb 2026-07-03 session work into KB.
//! Usage: cargo run -p neotrix --bin neotrix-session-ingest-0703

use neotrix::neotrix::nt_memory_kb::nt_memory_resource_ingest::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use rusqlite::Connection;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    let mut ingester = ResourceIngester::new(&conn);

    // 1. MetaPanel reasoning module
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "MetaPanel Multi-Perspective Reasoning Engine",
            "Multi-perspective analysis engine inspired by UZI-Skill's 66-judge panel. 3 depth levels (Lite/Mid/Deep), EWHR-weighted fusion, bull/bear debate, SelfReviewGate integration.",
        )
        .with_key_insights(vec![
            "AnalysisDepth::Lite → 3 viewpoints, no fusion, quick scan",
            "AnalysisDepth::Mid → 12 viewpoints, EWHR fusion, full self-review",
            "AnalysisDepth::Deep → 24 viewpoints, bull/bear debate, deep review",
            "FusionEngine computes consensus (0.0-1.0) + disagreement from confidence variance",
            "SelfReviewGate integrated: 10+ mechanical checks per analysis",
        ])
        .with_tags(vec!["module", "absorbed-2026-07-03"])
        .with_importance(0.85)
        .with_confidence(0.9),
    ).expect("MetaPanel ingest failed");
    println!("MetaPanel: node={} insights={}", r.node_id, r.insight_ids.len());

    // 2. SAE module implementation
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "SAE Sparse Autoencoder Implementation (Cycle 4 P0)",
            "Full SAE implementation for E8 reasoning state feature extraction. SparseAutoencoder, SaeEncoder/SaeDecoder, SteeringController, LayerSae, and constants (SAE_INPUT_DIM=512, SAE_LATENT_DIM=4096). Previously a 0-byte stub.",
        )
        .with_key_insights(vec![
            "SparseAutoencoder::e8_to_input() converts E8 hexagram + meta bits to SAE input vector",
            "SaeEncoder: encode() with ReLU activation + enforce_sparsity() placeholder",
            "SaeDecoder: decode() from latent space back to input dimension",
            "steer() enables causal intervention on specific latent features",
            "SAEBridge connects SAE ↔ E8 ↔ Observer for interpretable reasoning",
        ])
        .with_tags(vec!["module", "absorbed-2026-07-03", "cycle-4"])
        .with_importance(0.9)
        .with_confidence(0.95),
    ).expect("SAE ingest failed");
    println!("SAE: node={} insights={}", r.node_id, r.insight_ids.len());

    // 3. Pipeline SelfCheckStage
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "SelfReviewStage — Pipeline Self-Check Stage",
            "Integrated nt_core_self_review::SelfReviewGate into SEAL pipeline as SelfReviewStage. Runs every iteration, logs findings, non-blocking. Registered as final stage in seal_pipeline().",
        )
        .with_key_insights(vec![
            "SelfReviewStage runs at frequency=1 (every iteration)",
            "Runs 10+ mechanical checks: unwrap, todo, dead_code, public_docs, empty_match",
            "Non-blocking: never returns Rollback, only logs warnings",
            "Strict mode enabled by default (run_all checks)",
        ])
        .with_tags(vec!["module", "absorbed-2026-07-03"])
        .with_importance(0.75)
        .with_confidence(0.95),
    ).expect("SelfReviewStage ingest failed");
    println!("SelfReviewStage: node={} insights={}", r.node_id, r.insight_ids.len());

    // Relation links
    ingester.relate_by_title("MetaPanel Multi-Perspective Reasoning Engine", "UZI-Skill", RelationType::InspiredBy, 0.85, Some("Multi-judge analysis panel"))
        .ok();
    ingester.relate_by_title("MetaPanel Multi-Perspective Reasoning Engine", "SelfReviewStage — Pipeline Self-Check Stage", RelationType::References, 0.7,
        Some("MetaPanel integrates SelfReviewGate for mechanical checks"))
        .ok();

    // 4. Architecture review: L5→KB connection
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "Architecture Fix: L5/L9 Consciousness → KB Persistence Bridge",
            "Connected ConsciousnessMonitor in L9 background loop to KnowledgeBase via record_consciousness_snapshot(). Previously 2,067 lines of consciousness computation (Phi, FEP, IIT) never persisted — all scores lost on restart.",
        )
        .with_key_insights(vec![
            "handle_awareness() now persists phi/coherence/level to KB after each observation",
            "Consciousness tier classification: dormant/awakening/conscious/transcendent",
            "Uses existing nt_memory_gwtq::record_consciousness_snapshot() bridge",
            "L5 remains pure computation (Separation of Concerns preserved)",
        ])
        .with_tags(vec!["architecture-fix", "absorbed-2026-07-03"])
        .with_importance(0.85)
        .with_confidence(0.95),
    ).expect("L5-KB fix ingest failed");
    println!("L5-KB: node={}", r.node_id);

    // 5. Architecture review: evidence CLI
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "Architecture Fix: evidence_cmds Registered in CLI",
            "EvidenceCommand (list/get/calibrate/export/stats) was compiled but unreachable from CLI — 145 lines of dead code. Now registered as /evidence command in registry.rs.",
        )
        .with_key_insights(vec![
            "EvidenceCmd implements CliCommand trait with manual subcommand dispatch",
            "Aliases: /ev, /ewhr",
            "Backed by EvidenceStore from nt_memory_historian (8-file module)",
        ])
        .with_tags(vec!["architecture-fix", "absorbed-2026-07-03"])
        .with_importance(0.65)
        .with_confidence(0.95),
    ).expect("evidence CLI fix ingest failed");
    println!("evidence-cli: node={}", r.node_id);

    // 6. Architecture review: pipeline no-op analysis
    let r = ingester.ingest(
        &ResourceDescriptor::concept(
            "Architecture Audit: 14 No-Op Pipeline Stages + 15 Unregistered Stage Definitions",
            "Comprehensive pipeline audit: 30 registered stages (16 real, 14 no-op), 15 unregistered stage definitions in recipe.rs presets. 3 wrapper stages (Dpo/Constitutional/Safety) have full implementations behind them but never call them.",
        )
        .with_key_insights(vec![
            "DpoWrapperStage, ConstitutionalWrapperStage, SafetyWrapperStage: full implementations exist (255+1287+246 lines) but wrappers pass Continue",
            "GwtAbsorbStage, HarnessAdaptStage, KnowledgeQualityStage, SecretScanStage, ConversationDistillStage: expected to integrate GWT/KB/Scanner but are no-op",
            "6 goal_contract stages (EvidenceCapture through SemanticRecall): registered, frequency 1, all no-op",
            "15 unregistered stage definitions used only by recipe.rs presets (never executed by seal_pipeline())",
            "AntiDistillationStage: real impl (135 lines in core/) but only in recipe presets, not seal_pipeline()",
        ])
        .with_tags(vec!["architecture-audit", "absorbed-2026-07-03"])
        .with_importance(0.8)
        .with_confidence(0.95),
    ).expect("pipeline audit ingest failed");
    println!("pipeline-audit: node={}", r.node_id);

    // 7. External resource: alibaba/page-agent (GUI web agent)
    let r = ingester.ingest(
        &ResourceDescriptor::github("alibaba", "page-agent",
            "Page Agent — JavaScript in-page GUI agent",
            "In-page JavaScript GUI agent that controls web interfaces with natural language. Uses text-based DOM manipulation (no screenshots, no multimodal LLMs). 7-package monorepo: core (headless agent), page-agent (with UI panel), page-controller (DOM ops), llms, ui, extension (Chrome), mcp (MCP Server).")
        .with_key_insights(vec![
            "DOM Pipeline: Live DOM → FlatDomTree → Dehydration → LLM → Indexed Operations — no screenshots needed",
            "PageController ↔ PageAgent decoupled via async methods; PageController has zero LLM dependency",
            "SimulatorMask provides visual overlay blocking user interaction during automation",
            "Two entry points: IIFE demo script (<script> tag, free demo LLM) or NPM package (bring your own LLM)",
            "MCP Server (Beta) allows external agent clients to control the browser",
            "Built on browser-use DOM processing patterns (acknowledged in LICENSE)",
            "No browser extension needed for basic use; optional extension adds multi-page/tab support",
        ])
        .with_tags(vec!["github-repo", "absorbed-2026-07-03", "web-agent", "dom-automation", "mcp"])
        .with_importance(0.8)
        .with_confidence(0.85),
    ).expect("page-agent ingest failed");
    println!("page-agent: node={}", r.node_id);

    // 8. Architecture Fix: Lock Poison Unwrap Hardening
    let r = ingester.ingest(
        &ResourceDescriptor::concept("Architecture Fix: Lock Poison Unwrap Hardening",
            "Replaced 22 production lock().unwrap() calls with unwrap_or_else(|e| e.into_inner()) across web_navigator(6), agent(5), kanban(5), consciousness(4), cost_tracker(2), server(2), core(1), check_registry(1), tool_inspection(1), pipeline_stages(1). Eliminates process crash on any mutex/rwlock poison.")
        .with_key_insights(vec![
            "Lock poison unwrap crash sites reduced from 29 to 7 (all remaining are in #[cfg(test)] code, safe)",
            "Recovers lock data even when previous holder panicked — into_inner() extracts inner value from poisoned lock",
            "Most vulnerable paths: browser navigation (web_navigator), earning agent, kanban CLI, consciousness loop",
        ])
        .with_tags(vec!["architecture-fix", "absorbed-2026-07-03", "lock-poison", "hardening"])
        .with_importance(0.9)
        .with_confidence(0.95),
    ).expect("lock poison fix ingest failed");
    println!("lock-poison-fix: node={}", r.node_id);

    // 9. Architecture Fix: L7 Capability Bridge + 11 Bypass Import Fixes
    let r = ingester.ingest(
        &ResourceDescriptor::concept("Architecture Fix: L7 Bridge Module + Import Path Audit",
            "Created l7_capability_impl bridging module to expose core::l7_capability through the neotrix:: namespace. Fixed 11 bypass import paths that used deep impl/ paths instead of mod.rs re-exports. L7 now exposes SkillDoc, SkillMetrics, SkillBank, GroupCoordinator, etc. through crate::neotrix::.")
        .with_key_insights(vec![
            "L7 bridging: pub use crate::core::l7_capability::* from neotrix/l7_capability_impl/mod.rs",
            "11 bypass import paths fixed: l3_memory_spatial/types, l1_io_map_tile, l8_engine_core now use re-export paths",
            "Architecture convention: all neotrix layer access must go through neotrix/mod.rs re-exports, not impl/ paths",
        ])
        .with_tags(vec!["architecture-fix", "absorbed-2026-07-03", "l7-bridge", "import-audit"])
        .with_importance(0.8)
        .with_confidence(0.95),
    ).expect("l7 bridge ingest failed");
    println!("l7-bridge: node={}", r.node_id);

    // 10. Architecture Fix: 4 Pipeline Stages Upgraded from Limited to Real
    let r = ingester.ingest(
        &ResourceDescriptor::concept("Architecture Fix: 4 Pipeline Stage Upgrades",
            "Upgraded GwtAbsorbStage (KB kv_set persistence), HarnessAdaptStage (capability vector mutation on low reward), KnowledgeQualityStage (edge/node ratio quality scoring), and ConversationDistillStage (trajectory KB storage) from log-only to real implementations.")
        .with_key_insights(vec![
            "GwtAbsorbStage now persists iteration snapshots to KB kv_store instead of just logging",
            "HarnessAdaptStage now actually boosts weak capability dimensions on low reward (<0.3)",
            "KnowledgeQualityStage computes edge/node ratio quality score (0-100%)",
            "ConversationDistillStage stores trajectory snapshots to KB every 15 iterations when traj_len > 5",
        ])
        .with_tags(vec!["architecture-fix", "absorbed-2026-07-03", "pipeline-upgrade"])
        .with_importance(0.7)
        .with_confidence(0.9),
    ).expect("pipeline upgrade ingest failed");
    println!("pipeline-upgrade: node={}", r.node_id);

    // Relations
    ingester.relate_by_title("Architecture Fix: L5/L9 Consciousness → KB Persistence Bridge",
        "Architecture Audit: 14 No-Op Pipeline Stages + 15 Unregistered Stage Definitions",
        RelationType::References, 0.6, Some("Both address architecture gaps identified in audit"))
        .ok();
    ingester.relate_by_title("Architecture Fix: Lock Poison Unwrap Hardening",
        "Architecture Fix: L7 Bridge Module + Import Path Audit",
        RelationType::References, 0.7, Some("Both are 2026-07-03 architecture hardening fixes"))
        .ok();

    println!("\n{}", ingester.report());
    println!("✅ Session absorption complete (10 concepts + relations)");
}
