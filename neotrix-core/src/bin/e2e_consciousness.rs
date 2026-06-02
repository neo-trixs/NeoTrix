/// 验证意识进化回路: KB ↔ GWT ↔ E8 完整闭环
use neotrix::core::nt_core_gwt::module_def::SpecialistModule;
use neotrix::core::nt_core_gwt::workspace::GlobalWorkspace;
use neotrix::core::e8_reasoning::ReasoningHexagram;
use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
use neotrix::neotrix::nt_mind::consciousness_bridge::ConsciousnessBridge;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  意识进化回路验证 — KB ↔ GWT ↔ E8               ║");
    println!("╚═══════════════════════════════════════════════════╝");

    // 1. Open KB + seed
    let kb = KnowledgeBase::open(None).expect("KB open");
    println!("\n  KB 已打开: {} 节点, {} 边", kb.stats().unwrap().total_nodes, kb.stats().unwrap().total_edges);

    // 2. Create GWT + register specialists
    let mut gwt = GlobalWorkspace::new(0.35);
    gwt.register(SpecialistModule::new(neotrix::core::nt_core_gwt::module_def::SpecialistType::KnowledgeRetriever, "kb_query".into()));
    gwt.register(SpecialistModule::new(neotrix::core::nt_core_gwt::module_def::SpecialistType::PatternMatcher, "pattern".into()));
    gwt.register(SpecialistModule::new(neotrix::core::nt_core_gwt::module_def::SpecialistType::CreativityGenerator, "creative".into()));
    println!("  GWT 已创建 (3 个专家模块注册)");

    // 3. Create SelfIteratingBrain
    let mut brain = SelfIteratingBrain::new();
    println!("  Brain 已创建: capability={:?}", &brain.brain.capability.arr()[..3]);

    // 4. Create bridge with KB attached
    let mut bridge = ConsciousnessBridge::new();
    bridge.attach_kb(kb);
    bridge.poll_interval = 1;

    // 5. Verify E8 → KB query interface
    println!("\n━━━ E8 → KB 查询 ━━━");
    for mode in &[ReasoningHexagram::new(42), ReasoningHexagram::new(33), ReasoningHexagram::new(63)] {
        println!("  E8 模式 {} ({})", mode.0, mode.mode_name());
        let results = bridge.kb.as_ref().unwrap().query_by_e8_state(*mode, 3).unwrap();
        if results.is_empty() {
            println!("    (KB 暂无匹配)");
        }
        for r in &results {
            println!("    [{:?}] {} (score: {:.3})", r.node.node_type, r.node.title, r.score);
        }
    }

    // 6. Run bridge cycle (brain → GWT → KB → brain)
    println!("\n━━━ Consciousness Bridge 循环 ━━━");
    for i in 0..3 {
        gwt.broadcast(&format!("iteration {}: analyzing knowledge integration patterns", i));
        bridge.bridge_cycle(&mut brain, &mut gwt);
        println!("  [{}/3] GWT specialists: {} | brain cap[0..3]: {:?}",
            i+1, gwt.active_specialists().len(), &brain.brain.capability.arr()[..3]);
    }

    // 7. Verify snapshots were recorded to KB
    println!("\n━━━ KB 意识快照验证 ━━━");
    let results = bridge.kb.as_ref().unwrap().search("consciousness snapshot", 5).unwrap();
    println!("  找到 {} 条意识快照:", results.len());
    for r in &results {
        println!("    [{:?}] {} (score: {:.3}) — {}",
            r.node.node_type, r.node.title, r.score,
            r.node.summary.as_deref().unwrap_or(""));
    }

    // 8. Verify E8 recommendation
    println!("\n━━━ E8 推荐 ━━━");
    let recommend = bridge.kb.as_ref().unwrap().recommend_for_e8_mode("analytical method", 3).unwrap();
    for r in &recommend {
        println!("    [{:?}] {} (score: {:.3})", r.node.node_type, r.node.title, r.score);
    }

    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  意识进化回路验证完成                          ║");
    println!("╚═══════════════════════════════════════════════╝");
    let stats = bridge.kb.as_ref().unwrap().stats().unwrap();
    println!("  终态: {} 节点, {} 边", stats.total_nodes, stats.total_edges);
    println!("    其中 Insight={}", stats.by_type.iter().find(|(t,_)| t == "insight").map(|(_,c)| *c).unwrap_or(0));
}
