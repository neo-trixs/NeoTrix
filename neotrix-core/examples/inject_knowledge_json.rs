use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::knowledge_engine::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  🔱 知识注入 — 从 JSON 批量加载 250+ 条知识                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home).join(".neotrix").join("knowledge_engine.json");
    let data_path = PathBuf::from(&home).join("Downloads").join("code").join("neotrix").join("assets").join("knowledge_data.json");

    // Load existing engine
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    // Read JSON data
    let json_str = std::fs::read_to_string(&data_path).expect("Cannot read knowledge_data.json");
    let entries: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("Invalid JSON");
    println!("📦 读取到 {} 条知识条目", entries.len());

    // Inject each entry
    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for entry in &entries {
        let title = entry["title"].as_str().unwrap_or("Untitled");
        let body = entry["body"].as_str().unwrap_or("");
        let importance = entry["importance"].as_f64().unwrap_or(0.7);
        let tags: Vec<&str> = entry["tags"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        // Skip if already exists (by title)
        if eng.entries.values().any(|e| e.title == title) {
            skipped += 1;
            continue;
        }

        let entry_obj = KnowledgeEntry::new(title, body, SourceType::KnowledgeBase, "kb:god-level")
            .with_importance(importance)
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        eng.add_entry(entry_obj);
        inserted += 1;
    }

    println!("\n📊 注入结果:");
    println!("  已存在(跳过): {}", skipped);
    println!("  新插入: {}", inserted);
    println!("  累计: {} 条目, {} 关系", eng.stats().total_entries, eng.stats().total_relations);

    // Build cross-references
    println!("\n🔗 建立交叉关系...");
    let mut rel_count = 0;
    let all: Vec<String> = eng.entries.keys().cloned().collect();
    for i in 0..all.len().min(200) {
        for j in (i+1)..all.len().min(i+5) { // limit to 5 links per entry
            if let (Some(a), Some(b)) = (eng.entries.get(&all[i]), eng.entries.get(&all[j])) {
                // Check if they share any tag
                if a.tags.iter().any(|t| b.tags.contains(t)) {
                    eng.add_relation(&all[i], &all[j], RelationType::Related, 0.4,
                        &format!("{} ↔ {}", a.title, b.title));
                    rel_count += 1;
                }
            }
        }
    }
    println!("  ✅ {} 条交叉关系", rel_count);

    // Save
    if let Err(e) = eng.save() {
        eprintln!("❌ 保存失败: {}", e);
    } else {
        println!("\n💾 已保存到 {:?}", kb_path);
    }

    // Summary
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  📊 知识引擎最终状态                                        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  条目总数: {:>3}                                          ║", eng.stats().total_entries);
    println!("║  关系总数: {:>3}                                          ║", eng.stats().total_relations);
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Domain breakdown
    let domains = ["天界","地界","人界","数学","物理","化学","天文","历史","哲学","生物","地理","科学","文学","艺术"];
    println!("\n  标签统计:");
    for d in &domains {
        let c = eng.search_by_tag(d, 10000).len();
        if c > 0 { println!("    {}: {}", d, c); }
    }
}
