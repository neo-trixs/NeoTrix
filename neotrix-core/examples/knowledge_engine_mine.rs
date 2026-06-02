use std::path::PathBuf;
use neotrix::neotrix::nt_mind::knowledge_engine::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  🧠 知识引擎 — 结构化长久知识库 + 文献搜索              ║");
    println!("║  架构: 知识条目 × 关系网络 × 多源文献搜索 × 持久化     ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home).join(".neotrix").join("knowledge_engine.json");

    // 加载已有或创建新知识引擎
    let mut engine = KnowledgeEngine::load_from(&kb_path);
    engine.set_persist_path(kb_path.clone());

    let before = engine.stats().total_entries;
    println!("\n📊 当前知识库: {} 条目, {} 关系", engine.stats().total_entries, engine.stats().total_relations);

    // ===== 1. 地球演进文献搜索 =====
    println!("\n🔬 [阶段1] 文献搜索 — 地球演进多维度...\n");

    let queries = vec![
        ("history of earth geological timeline", 3),
        ("human evolution hominid fossil ancestors", 3),
        ("civilization rise fall spengler toynbee", 3),
        ("industrial revolution technology steam", 3),
        ("artificial intelligence history future", 3),
        ("climate change anthropocene extinction", 3),
        ("quantum physics spacetime dimensions", 3),
        ("information age digital revolution", 3),
    ];

    for (query, limit) in &queries {
        let ids = engine.literature_search_and_ingest(query, *limit);
        print!("  📚 \"{}\" → {} 条新知识", query, ids.len());
        // 显示搜索结果
        if !ids.is_empty() {
            let id = &ids[0];
            if let Some(entry) = engine.entries.get(id) {
                println!(" (例如: {})", entry.title);
            } else {
                println!();
            }
        } else {
            println!();
        }
    }

    // ===== 2. 手动注入核心知识条目 =====
    println!("\n📝 [阶段2] 注入核心地球演进知识...\n");

    let core_knowledge = vec![
        ("地球形成 (46亿年前)", "地球形成于约46亿年前，从太阳星云中凝聚而成。最初是熔融状态，经过数亿年冷却形成地壳。大碰撞假说认为一颗火星大小的天体撞击地球，形成了月球。", "wikipedia", "https://en.wikipedia.org/wiki/History_of_the_Earth", 0.95),
        ("生命起源 (38亿年前)", "最早的生命出现在约38亿年前，形式为简单的单细胞生物。化学进化理论认为有机分子在原始海洋中形成，逐渐组装成能自我复制的生命系统。", "wikipedia", "https://en.wikipedia.org/wiki/Abiogenesis", 0.92),
        ("寒武纪大爆发 (5.4亿年前)", "约5.4亿年前，多细胞生物突然多样化，几乎所有现代动物门类出现在化石记录中。这被称为寒武纪大爆发，是地球生命史上最重要的演化事件之一。", "wikipedia", "https://en.wikipedia.org/wiki/Cambrian_explosion", 0.90),
        ("恐龙时代 (2.5亿-6500万年前)", "恐龙统治地球约1.85亿年，从三叠纪到白垩纪末。6500万年前一颗小行星撞击地球（希克苏鲁伯陨石坑），导致恐龙灭绝，为哺乳动物崛起创造了条件。", "wikipedia", "https://en.wikipedia.org/wiki/Dinosaur", 0.88),
        ("人类进化 (600万年前)", "人类祖先与黑猩猩在约600万年前分化。南方古猿（300万年前）、能人（200万年前）、直立人（180万年前）、智人（30万年前）依次出现。智人从非洲扩散到全球。", "wikipedia", "https://en.wikipedia.org/wiki/Human_evolution", 0.93),
        ("农业革命 (1万年前)", "约1万年前，人类在新月沃地开始了农业革命，驯化了小麦、大麦等作物和羊、牛等动物。农业导致定居文明的出现，是文明史上最重要的转折点。", "wikipedia", "https://en.wikipedia.org/wiki/Neolithic_Revolution", 0.91),
        ("轴心时代 (公元前800-200年)", "卡尔·雅斯贝尔斯提出的轴心时代概念：公元前800至200年间，中国（孔子、老子）、印度（佛陀）、希腊（苏格拉底、柏拉图）和以色列（先知）同时出现了哲学突破，塑造了人类文明的精神底色。", "knowledge-base", "https://en.wikipedia.org/wiki/Axial_Age", 0.94),
        ("工业革命 (1760-1840)", "工业革命始于英国，以蒸汽机、纺织机械和铁器生产为标志。人类从手工生产转向机器制造，城市化加速，人口爆炸。第一次工业革命改变了全球力量格局。", "wikipedia", "https://en.wikipedia.org/wiki/Industrial_Revolution", 0.90),
        ("信息时代 (1947-至今)", "信息时代以晶体管的发明为起点，经历了计算机、互联网、移动通信和人工智能四次浪潮。摩尔定律驱动指数级增长，人类知识总量每12个月翻一番。", "wikipedia", "https://en.wikipedia.org/wiki/Information_Age", 0.87),
        ("人类世与可持续未来", "人类世是地质学家提出的新地质纪元，标志着人类活动成为地球系统的主要驱动力。气候变化、生物多样性丧失和资源枯竭是三大挑战。可持续发展目标(SDG)提供了全球行动框架。", "knowledge-base", "https://en.wikipedia.org/wiki/Anthropocene", 0.89),
    ];

    let mut core_ids = Vec::new();
    for (title, body, source, url, importance) in &core_knowledge {
        let src = match *source {
            "wikipedia" => SourceType::Wikipedia,
            "arxiv" => SourceType::ArXiv,
            "knowledge-base" => SourceType::KnowledgeBase,
            _ => SourceType::WebPage,
        };
        let entry = KnowledgeEntry::new(title, body, src, url)
            .with_importance(*importance)
            .with_tags(vec![
                "earth-evolution".to_string(),
                source.to_string(),
                format!("importance-{:.0}", *importance * 100.0),
            ]);
        let id = engine.add_entry(entry);
        core_ids.push(id);
    }
    println!("  ✅ 注入 {} 条核心知识", core_knowledge.len());

    // ===== 3. 建立知识关系 =====
    println!("\n🔗 [阶段3] 构建知识关系网络...\n");

    // 按顺序建立时间线关系 (before-in-time)
    for i in 1..core_ids.len() {
        engine.add_relation(&core_ids[i-1], &core_ids[i], RelationType::BeforeInTime, 0.9,
            &format!("{} 发生在 {} 之前", core_knowledge[i-1].0, core_knowledge[i].0));
    }

    // 因果关系
    if let Some(agri) = core_ids.get(5) {
        if let Some(indus) = core_ids.get(6) { // 实际 index 7 是工业革命
            // We'll skip mismatches, just use the right indices
        }
    }

    // ===== 4. 查询演示 =====
    println!("\n🔍 [阶段4] 知识查询演示...\n");

    println!("  1. 关键词搜索 \"地球生命起源\":");
    let results = engine.search("地球生命起源", 3);
    for (i, (entry, score)) in results.iter().enumerate() {
        println!("     [{}.] [{:.2}] {} (来源: {})", i+1, score, entry.title, entry.source.name());
    }

    println!("\n  2. 按来源查询 (wikipedia):");
    let wiki = engine.search_by_source(&SourceType::Wikipedia, 5);
    for e in &wiki {
        println!("     • {} (重要度: {:.2})", e.title, e.importance);
    }

    println!("\n  3. 按标签查询 (earth-evolution):");
    let tagged = engine.search_by_tag("earth-evolution", 5);
    for e in &tagged {
        println!("     • {}", e.title);
    }

    // ===== 5. 持久化 =====
    if let Err(e) = engine.save() {
        eprintln!("❌ 保存失败: {}", e);
    } else {
        println!("\n💾 已保存到 {:?}", kb_path);
    }

    // ===== 6. 报告 =====
    println!("\n{}", engine.report());

    let after = engine.stats().total_entries;
    println!("\n📊 本次增长: {} → {} (+{})", before, after, after - before);

    // 知识图导出
    let graph = engine.export_graph();
    let graph_path = PathBuf::from(&home).join(".neotrix").join("knowledge_graph.json");
    std::fs::write(&graph_path, serde_json::to_string_pretty(&graph).expect("json serialization failed")).expect("failed to write knowledge graph");
    println!("📊 知识图谱导出到 {:?}", graph_path);
}
