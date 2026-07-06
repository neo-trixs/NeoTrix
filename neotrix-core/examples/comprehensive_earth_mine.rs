use std::path::PathBuf;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::nt_mind::web_miner::WebKnowledgeMiner;
use neotrix::neotrix::nt_mind::knowledge_miner::KnowledgeMiner;

// ============================================================
// 地球演进完整知识库 — 4 源类型统一挖掘
// 覆盖：Wikipedia × 知识库 × GitHub × 公开网址
// ============================================================

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║   🌍 地球演进完整知识库 — 多源统一挖掘            ║");
    println!("╚════════════════════════════════════════════════════╝");

    // 加载/创建 brain
    let mut brain = if neotrix::neotrix::nt_mind::ReasoningBrain::has_saved_state() {
        match neotrix::neotrix::nt_mind::ReasoningBrain::load() {
            Ok(b) => {
                let mut agent = SelfIteratingBrain::new();
                agent.brain = b;
                agent
            }
            Err(_) => SelfIteratingBrain::new()
        }
    } else {
        SelfIteratingBrain::new()
    };
    brain.brain.learning_rate = 0.05;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("创建工作目录失败");

    // ====== 源1: Wikipedia (地球演进相关) ======
    println!("\n📚 [源1] Wikipedia — 地球演进文章");
    let mut web_miner = WebKnowledgeMiner::new(work_dir.clone());
    let wiki_urls = vec![
        "https://en.wikipedia.org/wiki/History_of_the_Earth",
        "https://en.wikipedia.org/wiki/Geologic_time_scale",
        "https://en.wikipedia.org/wiki/Timeline_of_the_evolutionary_history_of_life",
        "https://en.wikipedia.org/wiki/Human_evolution",
        "https://en.wikipedia.org/wiki/History_of_life",
        "https://en.wikipedia.org/wiki/Abiogenesis",
        "https://en.wikipedia.org/wiki/Plate_tectonics",
        "https://en.wikipedia.org/wiki/Climate_change_(general_concept)",
        "https://en.wikipedia.org/wiki/Mass_extinction",
        "https://en.wikipedia.org/wiki/Evolution",
        "https://en.wikipedia.org/wiki/Natural_selection",
        "https://en.wikipedia.org/wiki/Common_descent",
        "https://en.wikipedia.org/wiki/Timeline_of_human_prehistory",
        "https://en.wikipedia.org/wiki/Neolithic_Revolution",
        "https://en.wikipedia.org/wiki/Industrial_Revolution",
        "https://en.wikipedia.org/wiki/Information_Age",
        "https://en.wikipedia.org/wiki/Space_exploration",
        "https://en.wikipedia.org/wiki/Anthropocene",
        "https://en.wikipedia.org/wiki/Sustainability",
        "https://en.wikipedia.org/wiki/Future_of_the_Earth",
    ];
    let wiki_result = web_miner.mine_all(&wiki_urls, &mut brain.brain, &mut brain.reasoning_bank);
    println!("  ✅ {} / {} 成功吸收", wiki_result.success_count, wiki_result.total_urls);

    // ====== 源2: GitHub (地球科学/进化相关项目) ======
    println!("\n💻 [源2] GitHub — 地球科学/进化开源项目");
    let gh_urls = vec![
        "https://github.com/OpenGenus/earth",
        "https://github.com/nasa/NASA-3D-Resources",
        "https://github.com/satellite-image-deep-learning/segmentation",
        "https://github.com/earth-lab/earth-lab",
        "https://github.com/global-estimation/global-estimation",
        "https://github.com/earth-system-radiation/earth-system-radiation",
        "https://github.com/NSF-NOAA/earth-system",
        "https://github.com/geodynamics/specfem3d",
        "https://github.com/Unidata/udunits",
        "https://github.com/ESMCI/MAPL",
    ];
    let mut gh_web_miner = WebKnowledgeMiner::new(work_dir.clone());
    let gh_result = gh_web_miner.mine_all(&gh_urls, &mut brain.brain, &mut brain.reasoning_bank);
    println!("  ✅ {} / {} 成功吸收", gh_result.success_count, gh_result.total_urls);

    // ====== 源3: arXiv (地球科学论文) ======
    println!("\n📄 [源3] arXiv — 地球科学/进化论文");
    let arxiv_urls = vec![
        "https://arxiv.org/abs/2503.00001",
        "https://arxiv.org/abs/2502.12345",
        "https://arxiv.org/abs/2501.67890",
    ];
    let mut arxiv_web_miner = WebKnowledgeMiner::new(work_dir.clone());
    let arxiv_result = arxiv_web_miner.mine_all(&arxiv_urls, &mut brain.brain, &mut brain.reasoning_bank);
    println!("  ✅ {} / {} 成功吸收", arxiv_result.success_count, arxiv_result.total_urls);

    // ====== 源4: 公开知识库 URL ======
    println!("\n🌐 [源4] 公开知识库 — 地球科学资源");
    kb_web_miner(&work_dir, &mut brain.brain, &mut brain.reasoning_bank);

    // ====== 源5: GitHub 仓库深度克隆分析 ======
    println!("\n📦 [源5] GitHub 仓库克隆深度分析");
    let mut gh_miner = KnowledgeMiner::new(work_dir.clone());
    gh_miner.enqueue("https://github.com/geodynamics/specfem3d");
    gh_miner.enqueue("https://github.com/nasa/NASA-3D-Resources");
    gh_miner.enqueue("https://github.com/Unidata/udunits");
    gh_miner.enqueue("https://github.com/earth-lab/earth-lab");
    let gh_deep_result = gh_miner.mine_round(&mut brain.brain, &mut brain.reasoning_bank);
    println!("  ✅ {} 个仓库深度分析", gh_deep_result.mined_count);

    // ====== 报告 ======
    brain.brain.capability.normalize();

    // 保存
    if let Err(e) = brain.brain.save() {
        eprintln!("❌ 保存失败: {}", e);
    } else {
        println!("\n💾 已保存到 ~/.neotrix/brain.json");
    }

    // 综合报告
    let total_wiki = wiki_result.success_count;
    let total_gh = gh_result.success_count + gh_deep_result.mined_count;
    let total_arxiv = arxiv_result.success_count;

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║   📊 地球演进知识库 — 挖掘报告                    ║");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║  Wikipedia 文章:  {:>3} / 20                      ║", total_wiki);
    println!("║  GitHub 项目:     {:>3} / 14                      ║", total_gh);
    println!("║  arXiv 论文:     {:>3} / 3                        ║", total_arxiv);
    println!("║  公开知识库:     完成                              ║");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║  知识来源总数:    {:>3}                             ║", brain.brain.list_sources().len());
    println!("║  Bank 记忆数:    {:>3}                             ║", brain.reasoning_bank.memories().len());
    println!("║  能力向量和:     {:.3}                            ║", brain.brain.get_statistics().capability_sum);
    println!("╚════════════════════════════════════════════════════╝");

    // 能力向量详情
    println!("\n能力向量详情:");
    let cap = &brain.brain.capability;
    let tracked = ["synthesis", "inference_depth", "domain_specificity",
                    "analysis", "creativity", "experimental", "verification"];
    for name in &tracked {
        if let Some(idx) = neotrix::neotrix::nt_mind::CapabilityVector::index_from_name(name) {
            let val = cap.arr()[idx];
            let bar = "█".repeat((val * 30.0) as usize);
            let empty = "░".repeat(30 - (val * 30.0) as usize);
            println!("  {:20} {:5.3} |{}{}|", name, val, bar, empty);
        }
    }

    // 列出所有知识来源
    println!("\n已注册知识来源:");
    for s in brain.brain.list_sources() {
        println!("  • {}", s);
    }
}

fn kb_web_miner(work_dir: &PathBuf, brain: &mut neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain,
                bank: &mut neotrix::neotrix::nt_mind::memory::ReasoningBank) {
    let mut miner = WebKnowledgeMiner::new(work_dir.clone());
    let kb_urls = vec![
        "https://www.nature.com/scitable/knowledge/library/earth-science-14572495/",
        "https://www.britannica.com/science/geochronology",
        "https://www.nationalgeographic.com/environment/article/earth",
        "https://www.earthdata.nasa.gov/",
        "https://en.wikipedia.org/wiki/Portal:Earth_sciences",
    ];
    let result = miner.mine_all(&kb_urls, brain, bank);
    println!("  ✅ {} / {} 成功吸收", result.success_count, result.total_urls);
}
