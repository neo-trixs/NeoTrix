use std::path::PathBuf;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

fn main() {
    println!("=== NeoTrix 知识挖掘启动 ===");

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;
    brain.quality_threshold = 0.7;
    brain.auto_absorb = true;

    // 创建工作目录
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("无法创建工作目录");

    println!("工作目录: {}", work_dir.display());
    println!("\n开始挖掘 12 个默认知识来源...");
    println!("  (AI/ML: transformers, langchain, openai-cookbook)");
    println!("  (后端: actix-web, spring-boot, express)");
    println!("  (移动端: flutter, react-native)");
    println!("  (DevOps: docker-compose, kubernetes)");
    println!("  (数据库: postgres, redis)");
    println!();

    match brain.run_knowledge_chain() {
        Ok(result) => {
            println!("\n✅ 知识挖掘完成!");
            println!("  发现: {}", result.discovered);
            println!("  挖掘成功: {}", result.mined);
            println!("  验证通过: {}", result.validated);
            println!("  吸收: {}", result.absorbed);
            println!("  存储: {}", result.stored);
            println!("  总奖励: {:.3}", result.total_reward);
            println!("\n详细报告:");
            for detail in &result.details {
                println!("  {}", detail);
            }
        }
        Err(e) => {
            eprintln!("❌ 知识挖掘失败: {}", e);
        }
    }

    let stats = brain.brain.get_statistics();
    println!("\n--- Brain 状态 ---");
    println!("吸收总数: {}", stats.total_absorbed);
    println!("能力向量和: {:.3}", stats.capability_sum);
    println!("Bank 记忆数: {}", brain.reasoning_bank.memories().len());

    // 显示已注册的自定义来源
    let sources = brain.brain.list_sources();
    println!("\n知识来源 ({}):", sources.len());
    for s in &sources {
        println!("  - {}", s);
    }

    // 保存状态
    if let Err(e) = brain.brain.save() {
        eprintln!("保存失败: {}", e);
    } else {
        println!("\n💾 已保存到 ~/.neotrix/brain.json");
    }
}
