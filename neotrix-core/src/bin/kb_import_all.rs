//! neotrix-kb-import-all — 导入所有外部数据到 KB
//!
//! Imports all 10 data sources: knowledge_data, review_findings, brain_state,
//! absorption_report, knowledge_engine, reasoning_memories, bandit, e8_state,
//! avatar_chain, proxy_pool

use std::path::Path;

fn import(kb: &neotrix::neotrix::nt_memory_kb::KnowledgeBase, label: &str,
    f: impl FnOnce() -> Result<neotrix::neotrix::nt_memory_kb::nt_memory_knowledge_assets::ImportReport, String>)
{
    match f() {
        Ok(r) => println!("[{}] {} nodes, {} edges, {} errors", label, r.imported, r.edges_created, r.errors.len()),
        Err(e) => eprintln!("[{}] FAILED: {}", label, e),
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = Path::new(&home).join(".neotrix");

    let kb = match neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => { eprintln!("KB open: {}", e); std::process::exit(1); }
    };

    import(&kb, "knowledge-assets", || kb.import_knowledge_assets(Path::new("assets/knowledge_data.json")));
    import(&kb, "review-findings", || kb.import_review_findings(Path::new("design/review-findings.json")));
    import(&kb, "brain-state", || kb.import_brain_state(&dir));
    import(&kb, "absorption-report", || kb.import_absorption_report(&dir.join("absorption_report.json")));
    import(&kb, "knowledge-engine", || kb.import_knowledge_engine(&dir.join("knowledge_engine.json")));
    import(&kb, "reasoning-memories", || kb.import_reasoning_memories(&dir.join("reasoning_bank.json")));
    import(&kb, "bandit", || kb.import_bandit_data(&dir.join("bandit.json")));
    import(&kb, "e8-state", || kb.import_e8_state(&dir.join("e8_state.json")));
    import(&kb, "avatar-chain", || kb.import_avatar_chain(&dir.join("avatar_chain.json")));
    import(&kb, "proxy-pool", || kb.import_proxy_pool(&dir.join("proxy-pool-state.json")));

    match kb.stats() {
        Ok(s) => println!("\nKB: {} nodes, {} edges", s.total_nodes, s.total_edges),
        Err(e) => eprintln!("Stats: {}", e),
    }
}
