use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║     NeoTrix KnowledgeBase 知识种子填充             ║");
    println!("║     初始化 SQLite DB + 文明基础种子                 ║");
    println!("╚═══════════════════════════════════════════════════╝");

    let start = Instant::now();

    let kb = KnowledgeBase::open(None).expect("Failed to open KnowledgeBase");
    println!("  DB 路径: {}", kb.db_path.display());

    // 1. seed foundational knowledge
    println!("\n━━━ 1. 种子填充 ━━━");
    let seeded = kb.seed_foundational().expect("Seed failed");
    println!("  已创建 {} 个知识节点+关系", seeded);

    // 2. stats
    println!("\n━━━ 2. 数据库统计 ━━━");
    let stats = kb.stats().expect("Stats failed");
    println!("  节点: {} | 边: {} | 爬取待处理: {} | 域数: {}",
        stats.total_nodes, stats.total_edges, stats.crawl_pending, stats.by_domain.len());

    // 3. search test
    println!("\n━━━ 3. 搜索测试 ━━━");
    let queries = ["consciousness", "mathematics", "vector symbolic", "neural network"];
    for q in &queries {
        let results = kb.search(q, 5).expect("Search failed");
        println!("  \"{}\": {} 结果", q, results.len());
        for r in results.iter().take(3) {
            println!("    [{:?}] {} (score: {:.2})", r.node.node_type, r.node.title, r.score);
        }
    }

    // 4. graph test
    println!("\n━━━ 4. 图查询测试 ━━━");
    let consciousness = kb.search("consciousness", 1).expect("Search failed");
    if let Some(result) = consciousness.first() {
        let related = kb.get_related(&result.node.id, None, 10).expect("Related failed");
        println!("  '{}' 相关节点 ({} 个):", result.node.title, related.len());
        for r in &related {
            println!("    {:?} ←[{}]→ score={:.2}",
                r.node.title, r.matched_on.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>().join(","), r.score);
        }
    }

    // 5. Wikipedia ingestion test
    println!("\n━━━ 5. Wikipedia 爬取测试 (consciousness) ━━━");
    match kb.ingest_wikipedia("consciousness") {
        Ok(n) => println!("  已吸收 {} 个新节点", n),
        Err(e) => println!("  ⚠ Wikipedia 爬取: {}", e),
    }

    // 6. Integration: persist mined knowledge
    println!("\n━━━ 6. 集成测试: persist_mined ━━━");
    let edits = vec![("inference_depth".into(), 0.12), ("analysis".into(), 0.08)];
    let insights = vec!["意识研究: 增强推理深度".into(), "跨学科融合: 哲学+神经科学".into()];
    match kb.persist_mined("Consciousness Studies", "Research on conscious experience and neural correlates", "https://en.wikipedia.org/wiki/Consciousness", "Wikipedia", 0.85, &edits, &insights) {
        Ok(id) => println!("  持久化节点 ID: {}", id),
        Err(e) => println!("  ⚠ persist_mined: {}", e),
    }

    // 7. Consciousness interface test
    println!("\n━━━ 7. 意识层接口测试 ━━━");
    use neotrix::core::e8_reasoning::ReasoningHexagram;
    let state = ReasoningHexagram::new(42);
    println!("  E8 状态 {}: {}", state.0, state.mode_name());
    match kb.query_by_e8_state(state, 3) {
        Ok(results) => {
            println!("  E8 查询结果 ({} 个):", results.len());
            for r in &results {
                println!("    [{:?}] {} (score: {:.2})", r.node.node_type, r.node.title, r.score);
            }
        },
        Err(e) => println!("  ⚠ E8 查询: {}", e),
    }
    match kb.record_consciousness_snapshot(0.45, 0.72, true, "ConsciousLike", "GoldStandard eval cycle #1") {
        Ok(id) => println!("  意识快照 ID: {}", id),
        Err(e) => println!("  ⚠ 意识快照: {}", e),
    }

    // 8. Dedup
    println!("\n━━━ 8. 去重 ━━━");
    match kb.dedup_nodes() {
        Ok(n) => println!("  合并 {} 个重复节点", n),
        Err(e) => println!("  ⚠ dedup: {}", e),
    }
    let stats2 = kb.stats().expect("Stats failed");
    println!("  去重后: {} 节点, {} 边", stats2.total_nodes, stats2.total_edges);

    let elapsed = start.elapsed();
    let final_stats = kb.stats().expect("Final stats failed");
    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  填充完成                                     ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("  耗时:       {:.2}s", elapsed.as_secs_f64());
    println!("  最终节点:   {}", final_stats.total_nodes);
    println!("  最终边:     {}", final_stats.total_edges);
    println!("  爬取待处理: {}", final_stats.crawl_pending);
    println!("  域数:       {}", final_stats.by_domain.len());
}
