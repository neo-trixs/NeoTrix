use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::cortex_memory::*;
use neotrix::neotrix::reasoning_brain::web_miner::WebKnowledgeMiner;

fn mine_to_cortex(miner: &mut WebKnowledgeMiner, urls: &[&str], cortex: &mut CortexMemory, label: &str) -> usize {
    println!("\n📌 {} ({} 个来源)", label, urls.len());
    let mut count = 0;
    for (i, url) in urls.iter().enumerate() {
        match miner.mine_url(url) {
            Ok(k) => {
                inject_from_web_miner(cortex, &k.source_url, &k.source_name,
                    k.source_type.name(), &k.title, &k.summary, &k.edits);
                count += 1;
                print!("    [{}/{}] ✅ {} {:?}\n", i+1, urls.len(), k.title, k.source_type);
            }
            Err(e) => {
                print!("    [{}/{}] ❌ {} — {}\n", i+1, urls.len(), url, e);
            }
        }
    }
    count
}

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║  🧠 CortexMemory — 类人脑多维知识挖掘              ║");
    println!("║  存储架构: 时间线 × 7维度 × 多模态 × 联想关联     ║");
    println!("╚════════════════════════════════════════════════════╝");

    let mut cortex = CortexMemory::new(50, 500);
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("failed to create work directory");
    let mut miner = WebKnowledgeMiner::new(work_dir);

    // 7 大维度链 — 44 个来源
    let _ = mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/History_of_the_Earth",
        "https://en.wikipedia.org/wiki/Geologic_time_scale",
        "https://en.wikipedia.org/wiki/Abiogenesis",
        "https://en.wikipedia.org/wiki/Timeline_of_the_evolutionary_history_of_life",
        "https://en.wikipedia.org/wiki/Human_evolution",
        "https://en.wikipedia.org/wiki/Neolithic_Revolution",
        "https://en.wikipedia.org/wiki/Industrial_Revolution",
        "https://en.wikipedia.org/wiki/Information_Age",
        "https://en.wikipedia.org/wiki/Future_of_the_Earth",
    ], &mut cortex, "⏳ 时间链 — 46亿年地球史");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/Civilization",
        "https://en.wikipedia.org/wiki/Axial_Age",
        "https://en.wikipedia.org/wiki/Clash_of_Civilizations",
        "https://en.wikipedia.org/wiki/Oswald_Spengler",
        "https://en.wikipedia.org/wiki/The_Rise_and_Fall_of_the_Great_Powers",
    ], &mut cortex, "🏛️ 文明链 — 文明兴衰理论");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/History_of_technology",
        "https://en.wikipedia.org/wiki/Agricultural_revolution",
        "https://en.wikipedia.org/wiki/Digital_Revolution",
        "https://en.wikipedia.org/wiki/Artificial_intelligence",
        "https://en.wikipedia.org/wiki/Space_exploration",
    ], &mut cortex, "⚙️ 科技链 — 技术革命");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/Evolution",
        "https://en.wikipedia.org/wiki/Natural_selection",
        "https://en.wikipedia.org/wiki/Mass_extinction",
        "https://en.wikipedia.org/wiki/Biodiversity",
        "https://en.wikipedia.org/wiki/Human_impact_on_the_environment",
    ], &mut cortex, "🧬 物种链 — 生命网络");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/Plate_tectonics",
        "https://en.wikipedia.org/wiki/Climate_change_(general_concept)",
        "https://en.wikipedia.org/wiki/Anthropocene",
        "https://en.wikipedia.org/wiki/Sustainability",
        "https://en.wikipedia.org/wiki/Earth_science",
    ], &mut cortex, "🌍 地理链 — 地球系统");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/Spacetime",
        "https://en.wikipedia.org/wiki/Multiverse",
        "https://en.wikipedia.org/wiki/Dimension",
        "https://en.wikipedia.org/wiki/String_theory",
        "https://en.wikipedia.org/wiki/Philosophy_of_time",
    ], &mut cortex, "🌌 宇宙链 — 时空维度");

    total += mine_to_cortex(&mut miner, &[
        "https://en.wikipedia.org/wiki/History_of_science",
        "https://en.wikipedia.org/wiki/Philosophy_of_history",
        "https://en.wikipedia.org/wiki/Collective_intelligence",
        "https://en.wikipedia.org/wiki/Knowledge_representation_and_reasoning",
        "https://en.wikipedia.org/wiki/Systems_thinking",
    ], &mut cortex, "📚 知识链 — 人类思想");

    // 巩固
    let consolidated = cortex.consolidate_all();
    println!("\n🧠 巩固: {} → 长期, 总数: {} 条", consolidated, cortex.stats().total_traces);

    // 报告
    println!("{}", cortex.report());

    // 多维度联想检索
    println!("🔍 联想检索:");
    let queries = [
        "dinosaur extinction asteroid climate",
        "industrial revolution machine steam power",
        "future human civilization technology AI",
        "spacetime dimension quantum multiverse",
        "human evolution ancient ancestors",
        "sustainability climate change earth future",
    ];
    for query in &queries {
        let results = cortex.recall(query, 3);
        println!("  \"{}\" → {} 条", query, results.len());
        for (t, s) in &results {
            println!("    [{:.2}] {} [{}]", s, t.title, t.source_type);
        }
    }

    // 按维度链导出
    println!("\n📊 各维度链记忆数:");
    let chains = ["时间链", "文明链", "科技链", "物种链", "地理链", "宇宙链", "知识链"];
    for cat in &chains {
        let traces = cortex.dimension_chain(cat, 100);
        let bar = "█".repeat(traces.len().min(30));
        println!("  {:10} |{:<30}| {}", cat, bar, traces.len());
    }

    // 持久化 JSON
    let json = cortex.export_json();
    let json_path = PathBuf::from(&home).join(".neotrix").join("cortex_memory.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(&json).expect("json serialization failed")).expect("failed to write cortex memory");
    let cortex_json = PathBuf::from(&home).join(".neotrix").join("cortex.json");
    std::fs::write(&cortex_json, serde_json::to_string_pretty(&json).expect("json serialization failed")).expect("failed to write cortex json");
    println!("\n💾 已保存到 {:?} 和 {:?}", json_path, cortex_json);
}
