use log;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use std::path::PathBuf;

fn main() {
    log::info!("=== NeoTrix 知识挖掘启动 ===");

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;
    brain.quality_threshold = 0.7;
    brain.auto_absorb = true;

    // 创建工作目录
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix").join("work");
    std::fs::create_dir_all(&work_dir).expect("无法创建工作目录");

    log::info!("工作目录: {}", work_dir.display());
    log::info!("\n开始挖掘 12 个默认知识来源...");
    log::info!("  (AI/ML: transformers, langchain, openai-cookbook)");
    log::info!("  (后端: actix-web, spring-boot, express)");
    log::info!("  (移动端: flutter, react-native)");
    log::info!("  (DevOps: docker-compose, kubernetes)");
    log::info!("  (数据库: postgres, redis)");
    log::info!();

    match brain.run_knowledge_chain() {
        Ok(result) => {
            log::info!("\n✅ 知识挖掘完成!");
            log::info!("  发现: {}", result.discovered);
            log::info!("  挖掘成功: {}", result.mined);
            log::info!("  验证通过: {}", result.validated);
            log::info!("  吸收: {}", result.absorbed);
            log::info!("  存储: {}", result.stored);
            log::info!("  总奖励: {:.3}", result.total_reward);
            log::info!("\n详细报告:");
            for detail in &result.details {
                log::info!("  {}", detail);
            }
        }
        Err(e) => {
            log::error!("❌ 知识挖掘失败: {}", e);
        }
    }

    let stats = brain.brain.get_statistics();
    log::info!("\n--- Brain 状态 ---");
    log::info!("吸收总数: {}", stats.total_absorbed);
    log::info!("能力向量和: {:.3}", stats.capability_sum);
    log::info!("Bank 记忆数: {}", brain.reasoning_bank.memories().len());

    // 显示已注册的自定义来源
    let sources = brain.brain.list_sources();
    log::info!("\n知识来源 ({}):", sources.len());
    for s in &sources {
        log::info!("  - {}", s);
    }

    // 保存状态
    if let Err(e) = brain.brain.save() {
        log::error!("保存失败: {}", e);
    } else {
        log::info!("\n💾 已保存到 ~/.neotrix/brain.json");
    }
}
