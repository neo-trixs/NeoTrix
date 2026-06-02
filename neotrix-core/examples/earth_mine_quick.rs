use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::reasoning_brain::web_miner::WebKnowledgeMiner;

fn main() {
    println!("=== 🌍 地球演进知识库 — 快速挖掘（跳过git clone） ===");

    let mut brain = if neotrix::neotrix::reasoning_brain::ReasoningBrain::has_saved_state() {
        match neotrix::neotrix::reasoning_brain::ReasoningBrain::load() {
            Ok(b) => {
                println!("✅ 加载已有 brain.json");
                let mut agent = SelfIteratingBrain::new();
                agent.brain = b;
                agent
            }
            Err(_) => SelfIteratingBrain::new()
        }
    } else {
        println!("🆕 创建新 brain");
        SelfIteratingBrain::new()
    };
    brain.brain.learning_rate = 0.05;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("创建工作目录失败");

    // Wikipedia — 地球科学完整覆盖
    let mut miner = WebKnowledgeMiner::new(work_dir);
    let urls: Vec<&str> = vec![
        // 地球历史与地质
        "https://en.wikipedia.org/wiki/History_of_the_Earth",
        "https://en.wikipedia.org/wiki/Geologic_time_scale",
        "https://en.wikipedia.org/wiki/Plate_tectonics",
        "https://en.wikipedia.org/wiki/Climate_change_(general_concept)",
        "https://en.wikipedia.org/wiki/Mass_extinction",
        "https://en.wikipedia.org/wiki/Anthropocene",
        "https://en.wikipedia.org/wiki/Future_of_the_Earth",
        // 生命进化
        "https://en.wikipedia.org/wiki/Timeline_of_the_evolutionary_history_of_life",
        "https://en.wikipedia.org/wiki/History_of_life",
        "https://en.wikipedia.org/wiki/Abiogenesis",
        "https://en.wikipedia.org/wiki/Evolution",
        "https://en.wikipedia.org/wiki/Natural_selection",
        "https://en.wikipedia.org/wiki/Common_descent",
        // 人类进化与文明
        "https://en.wikipedia.org/wiki/Human_evolution",
        "https://en.wikipedia.org/wiki/Timeline_of_human_prehistory",
        "https://en.wikipedia.org/wiki/Neolithic_Revolution",
        "https://en.wikipedia.org/wiki/Industrial_Revolution",
        "https://en.wikipedia.org/wiki/Information_Age",
        "https://en.wikipedia.org/wiki/Space_exploration",
        "https://en.wikipedia.org/wiki/Sustainability",
        // 多维度时间概念
        "https://en.wikipedia.org/wiki/Spacetime",
        "https://en.wikipedia.org/wiki/Multiverse",
        "https://en.wikipedia.org/wiki/Dimension",
        "https://en.wikipedia.org/wiki/Philosophy_of_time",
        // 文明理论
        "https://en.wikipedia.org/wiki/Civilization",
        "https://en.wikipedia.org/wiki/Axial_Age",
        "https://en.wikipedia.org/wiki/Clash_of_Civilizations",
        // 地球科学
        "https://en.wikipedia.org/wiki/Earth_science",
        "https://en.wikipedia.org/wiki/Geography",
        "https://en.wikipedia.org/wiki/Portal:Earth_sciences",
    ];

    let result = miner.mine_all(&urls, &mut brain.brain, &mut brain.reasoning_bank);

    brain.brain.capability.normalize();

    // 立即保存
    match brain.brain.save() {
        Ok(_) => println!("\n💾 已保存到 ~/.neotrix/brain.json"),
        Err(e) => eprintln!("❌ 保存失败: {}", e),
    }

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║   📊 地球演进知识库 — 挖掘报告                    ║");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║  来源总数:  {:>3} / {}                            ║", result.success_count, urls.len());
    println!("║  总编辑数:  {:>3}                                  ║", result.total_edits);
    println!("║  总奖励:    {:.3}                                 ║", result.total_reward);
    println!("║  Brain来源: {:>3}                                  ║", brain.brain.list_sources().len());
    println!("║  Bank记忆:  {:>3}                                  ║", brain.reasoning_bank.memories().len());
    println!("╚════════════════════════════════════════════════════╝");

    println!("\n能力向量:");
    let cap = &brain.brain.capability;
    let tracked = ["synthesis", "inference_depth", "domain_specificity",
                    "analysis", "creativity", "experimental", "verification",
                    "compound_composition", "quality_gates"];
    for name in &tracked {
        if let Some(idx) = neotrix::neotrix::reasoning_brain::CapabilityVector::index_from_name(name) {
            let val = cap.arr()[idx];
            let bar = "█".repeat((val * 30.0) as usize);
            let empty = "░".repeat(30 - (val * 30.0) as usize);
            println!("  {:25} {:5.3} |{}{}|", name, val, bar, empty);
        }
    }

    println!("\n注册知识来源 ({}):", brain.brain.list_sources().len());
    for s in brain.brain.list_sources() {
        println!("  • {}", s);
    }

    println!("\n详细报告:");
    for d in &result.details {
        println!("  {}", d);
    }
}
