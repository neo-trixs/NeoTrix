//! 处理指定URL的进化分析
use log;
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use neotrix::neotrix::nt_mind::self_evolver::SelfEvolver;
use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
use std::path::PathBuf;

fn main() {
    let url = "https://github.com/h4ckf0r0day/obscura";

    log::info!("🚀 开始处理 URL: {}", url);

    // 初始化
    let brain = ReasoningBrain::new();
    let bank = ReasoningBank::new(1000);
    let work_dir = PathBuf::from("/tmp/neotrix_evolve");
    let _ = std::fs::create_dir_all(&work_dir);

    let mut evolver = SelfEvolver::new(brain, bank, work_dir);

    // 直接处理URL（已知是有效URL）
    log::info!("✅ 开始进化分析...");

    match evolver.evolve_from_url(url) {
        Ok(reward) => {
            log::info!("✅ 进化完成！奖励值: {:.4}", reward);
            log::info!("💾 结果已保存到 ~/.neotrix/brain.json");
        }
        Err(e) => {
            log::error!("❌ 进化失败: {}", e);
        }
    }
}
