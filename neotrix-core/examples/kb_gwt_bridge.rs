//! KB→GWT 桥接 — 将知识库 embeddings 映射到 GWT 注意力竞争
//!
//! 用法: cargo run --example kb_gwt_bridge -p neotrix --features full
#![forbid(unsafe_code)]

use neotrix::core::nt_core_gwt::cognitive_hub::CognitiveHub;
use neotrix::core::nt_core_gwt::cognitive_type::CognitiveType;
use std::collections::HashMap;
use neotrix::core::nt_core_traits::SpecialistType;
use neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = std::time::Instant::now();
    println!("🧠 KB→GWT Bridge");
    println!("{}", "=".repeat(50));

    // ── 打开 KB ──
    let kb = KnowledgeBase::open(None)?;
    let guard = kb.raw_conn()?;
    let conn = &*guard;

    // ── 加载 embeddings ──
    let mut stmt = conn.prepare("SELECT node_id, vector FROM embeddings")?;
    let rows: Vec<(String, Vec<u8>)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
    })?.filter_map(|r| r.ok()).collect();
    
    let dim = if rows.is_empty() { 256 } else { rows[0].1.len() / 4 };
    println!("📊 Embeddings: {} × {}d", rows.len(), dim);

    // ── 节点类型映射到 SpecialistType ──
    let mut stmt = conn.prepare("SELECT id, node_type, importance FROM nodes")?;
    let node_info: HashMap<String, (String, f64)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, f64>(2)?))
    })?
    .filter_map(|r| r.ok())
    .map(|(id, ty, imp)| (id, (ty, imp)))
    .collect();

    fn map_to_specialist(node_type: &str) -> SpecialistType {
        match node_type {
            "concept" => SpecialistType::KnowledgeRetriever,
            "article" => SpecialistType::KnowledgeIntegrator,
            "insight" => SpecialistType::CreativityGenerator,
            "synthesis" => SpecialistType::EvidenceWeightedHypothesis,
            _ => SpecialistType::PatternMatcher,
        }
    }

    // ── 计算每个节点的 activation score ──
    // activation = importance * ln(1 + degree)
    let mut activations: Vec<(SpecialistType, f64)> = Vec::new();
    
    for (nid, _blob) in &rows {
        if let Some((node_type, importance)) = node_info.get(nid) {
            // 计算 degree（简化：用 content 长度近似）
            let degree: i64 = conn.query_row(
                "SELECT count(*) FROM edges WHERE source_id=?1 OR target_id=?1",
                [nid.as_str()],
                |r| r.get(0),
            ).unwrap_or(0);
            
            let activation = (*importance as f64) * (1.0 + degree as f64).ln() as f64;
            let st = map_to_specialist(node_type);
            activations.push((st, activation));
        }
    }

    // ── GWT 注意力竞争 ──
    println!("\n🎯 GWT Attention Competition");
    let mut hub = CognitiveHub::new();
    
    // 分批广播
    for chunk in activations.chunks(1000) {
        hub.record_broadcast_collaborations(chunk);
    }

    let sparsity = hub.sparsity();
    let active_hubs = hub.active_hub_count(&activations[..activations.len().min(100)]);
    
    println!("  Sparsity: {:.3}", sparsity);
    println!("  Active hubs: {}", active_hubs);

    // ── 认知类型路由分析 ──
    println!("\n📡 Cognitive Type Distribution:");
    for ct in CognitiveType::ALL {
        let count = activations.iter()
            .filter(|(st, _)| matches!(
                (ct, st),
                (CognitiveType::Linguistic, SpecialistType::PatternMatcher)
                | (CognitiveType::Linguistic, SpecialistType::CreativityGenerator)
                | (CognitiveType::Linguistic, SpecialistType::ReflectionEngine)
                | (CognitiveType::Logical, SpecialistType::CodeAnalyzer)
                | (CognitiveType::Logical, SpecialistType::EvidenceWeightedHypothesis)
                | (CognitiveType::Logical, SpecialistType::MetaCognitionAnalyst)
                | (CognitiveType::Logical, SpecialistType::AnomalyDetector)
                | (CognitiveType::Knowledge, SpecialistType::KnowledgeRetriever)
                | (CognitiveType::Knowledge, SpecialistType::KnowledgeIntegrator)
                | (CognitiveType::Knowledge, SpecialistType::Planner)
                | (CognitiveType::Knowledge, SpecialistType::GoalPrioritizer)
                | (CognitiveType::Social, SpecialistType::RiskAssessor)
                | (CognitiveType::Social, SpecialistType::AISecurity)
                | (CognitiveType::Social, SpecialistType::ImageGenerator)
            ))
            .count();
        println!("  {}: {} nodes", ct.label(), count);
    }

    // ── 最终统计 ──
    let elapsed = t0.elapsed().as_secs_f64();
    println!("\n{}", "=".repeat(50));
    println!("📊 GWT-KB Bridge 完成 ({:.1}s)", elapsed);
    println!("  Activations processed: {}", activations.len());
    println!("  Sparsity: {:.3}", sparsity);
    println!("{}", "=".repeat(50));

    Ok(())
}
