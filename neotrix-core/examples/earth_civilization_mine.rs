use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::self_iterating::SelfIteratingBrain;

fn main() {
    println!("=== 地球文明多维度时间链路 - 知识挖掘 ===");

    let mut brain = if neotrix::neotrix::reasoning_brain::ReasoningBrain::has_saved_state() {
        match neotrix::neotrix::reasoning_brain::ReasoningBrain::load() {
            Ok(b) => {
                println!("✅ 加载已有 brain.json");
                let mut agent = SelfIteratingBrain::new();
                agent.brain = b;
                agent
            }
            Err(e) => {
                eprintln!("加载失败 ({}), 创建新 brain", e);
                SelfIteratingBrain::new()
            }
        }
    } else {
        println!("🆕 创建新 brain");
        SelfIteratingBrain::new()
    };
    brain.brain.learning_rate = 0.05;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("无法创建工作目录");

    println!("\n🏛️  开始挖掘地球文明多维度时间链路...\n");

    // 使用 SelfEvolver 逐个进化 Wikipedia URL
    let urls = vec![
        // 地球历史与文明发展
        ("https://en.wikipedia.org/wiki/History_of_Earth", "地球历史"),
        ("https://en.wikipedia.org/wiki/Timeline_of_human_evolution", "人类进化时间线"),
        ("https://en.wikipedia.org/wiki/Human_history", "人类文明史"),
        ("https://en.wikipedia.org/wiki/Civilization", "文明定义与发展"),
        ("https://en.wikipedia.org/wiki/Timeline_of_historical_events", "历史事件时间线"),
        // 多维度时间概念
        ("https://en.wikipedia.org/wiki/Multiverse", "多元宇宙"),
        ("https://en.wikipedia.org/wiki/Spacetime", "时空"),
        ("https://en.wikipedia.org/wiki/String_theory", "弦理论"),
        ("https://en.wikipedia.org/wiki/Dimension", "维度"),
        ("https://en.wikipedia.org/wiki/Philosophy_of_time", "时间哲学"),
        // 文明周期理论
        ("https://en.wikipedia.org/wiki/Oswald_Spengler", "斯宾格勒文明周期"),
        ("https://en.wikipedia.org/wiki/Axial_Age", "轴心时代"),
        ("https://en.wikipedia.org/wiki/Clash_of_Civilizations", "文明冲突"),
        ("https://en.wikipedia.org/wiki/The_Rise_and_Fall_of_the_Great_Powers", "大国兴衰"),
    ];

    let mut success_count = 0;
    let mut total_reward = 0.0;

    for (i, (url, label)) in urls.iter().enumerate() {
        println!("[{}/{}] 🌍 {} <{}>", i+1, urls.len(), label, url);

        // 使用 KnowledgeMiner 的 fetch + analyze 逻辑
        let parsed = url::Url::parse(url).expect("invalid url");
        let file_name = parsed.path()
            .trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("page");
        let target_file = work_dir.join(format!("{}.html", file_name));

        // 获取网页内容
        if !target_file.exists() {
            let agent = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("Mozilla/5.0 (compatible; NeoTrix/1.0)")
                .build()
                .expect("failed to build http client");
            match agent.get(*url).send() {
                Ok(resp) => {
                    if let Ok(text) = resp.text() {
                        std::fs::write(&target_file, &text).expect("failed to write downloaded file");
                        let size_kb = text.len() / 1024;
                        println!("   📥 已获取 ({} KB)", size_kb);
                    }
                }
                Err(e) => {
                    eprintln!("   ❌ 网络错误: {}", e);
                    continue;
                }
            }
        } else {
            println!("   📂 使用缓存文件");
        }

        // 分析内容并生成 MicroEdits
        if let Ok(content) = std::fs::read_to_string(&target_file) {
            let lower = content.to_lowercase();

            // 提取洞察
            let mut insights = Vec::new();
            let mut edits = Vec::new();

            // 文明/历史相关维度
            if lower.contains("civilization") || lower.contains("empire") || lower.contains("dynasty") {
                edits.push(("synthesis".to_string(), 0.08));
                insights.push("文明史: 增强综合分析能力".to_string());
            }
            if lower.contains("evolution") || lower.contains("timeline") || lower.contains("chronology") {
                edits.push(("inference_depth".to_string(), 0.06));
                insights.push("时间线: 增强推理深度".to_string());
            }
            if lower.contains("dimension") || lower.contains("multiverse") || lower.contains("string theory") {
                edits.push(("inference_depth".to_string(), 0.10));
                edits.push(("domain_specificity".to_string(), 0.08));
                insights.push("维度理论: 增强领域专精度和推理深度".to_string());
            }
            if lower.contains("spacetime") || lower.contains("relativity") || lower.contains("quantum") {
                edits.push(("experimental".to_string(), 0.07));
                edits.push(("domain_specificity".to_string(), 0.10));
                insights.push("时空/量子: 增强实验性和领域专精度".to_string());
            }
            if lower.contains("culture") || lower.contains("philosophy") || lower.contains("religion") {
                edits.push(("creativity".to_string(), 0.06));
                insights.push("文化哲学: 增强创造力".to_string());
            }
            if lower.contains("technology") || lower.contains("industrial") || lower.contains("digital") {
                edits.push(("analysis".to_string(), 0.07));
                insights.push("科技史: 增强分析能力".to_string());
            }
            if lower.contains("war") || lower.contains("conflict") || lower.contains("revolution") {
                edits.push(("analysis".to_string(), 0.05));
                insights.push("冲突史: 增强分析能力".to_string());
            }

            if !edits.is_empty() {
                // 注册为知识来源
                let source_name = format!("wiki_{}", file_name);
                let mut cv = neotrix::neotrix::reasoning_brain::CapabilityVector::default();
                for (dim, delta) in &edits {
                    if let Some(idx) = neotrix::neotrix::reasoning_brain::CapabilityVector::index_from_name(dim) {
                        cv.arr_mut()[idx] = *delta;
                    }
                }
                cv.normalize();
                cv.set_provenance(url.to_string());

                brain.brain.register_knowledge_source(&source_name, cv);

                // 应用 MicroEdits
                for (dim, delta) in &edits {
                    if let Some(idx) = neotrix::neotrix::reasoning_brain::CapabilityVector::index_from_name(dim) {
                        let val = &mut brain.brain.capability.arr_mut()[idx];
                        *val = (*val + delta).clamp(0.0, 1.0);
                    }
                }

                let reward = edits.len() as f64 * 0.02;
                total_reward += reward;
                success_count += 1;

                println!("   ✅ 吸收 {} 条知识 (reward: {:.3})", edits.len(), reward);
                for insight in &insights {
                    println!("      💡 {}", insight);
                }
            } else {
                println!("   ⚠️  未检测到相关知识");
            }
        }
    }

    brain.brain.capability.normalize();

    // 存储到 ReasoningBank
    let memory = neotrix::neotrix::reasoning_brain::ReasoningMemory::new(
        &format!("地球文明多维度时间链路: {} 个来源吸收", success_count),
        neotrix::neotrix::world_model::TaskType::CodeAnalysis,
        &[],
        total_reward / urls.len() as f64,
    );
    brain.reasoning_bank.store(memory);

    // 保存
    if let Err(e) = brain.brain.save() {
        eprintln!("❌ 保存失败: {}", e);
    } else {
        println!("\n💾 已保存到 ~/.neotrix/brain.json");
    }

    // 显示结果
    println!("\n=== 挖掘报告 ===");
    println!("来源总数: {}", urls.len());
    println!("成功吸收: {}", success_count);
    println!("总奖励: {:.3}", total_reward);
    println!("能力向量和: {:.3}", brain.brain.get_statistics().capability_sum);
    println!("Bank 记忆数: {}", brain.reasoning_bank.memories().len());

    let sources = brain.brain.list_sources();
    println!("\n知识来源 ({}):", sources.len());
    for s in &sources {
        println!("  - {}", s);
    }

    println!("\n能力向量变化:");
    let cap = &brain.brain.capability;
    let names = ["inference_depth", "domain_specificity", "synthesis", "analysis",
                  "creativity", "experimental"];
    for name in &names {
        if let Some(idx) = neotrix::neotrix::reasoning_brain::CapabilityVector::index_from_name(name) {
            if cap.arr()[idx] > 0.01 {
                println!("  {}: {:.3}", name, cap.arr()[idx]);
            }
        }
    }
}
