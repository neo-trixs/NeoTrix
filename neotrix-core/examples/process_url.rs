//! 处理指定URL的进化分析
use neotrix::neotrix::nt_mind::self_evolver::SelfEvolver;
use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use std::path::PathBuf;

fn main() {
    let url = "https://github.com/h4ckf0r0day/obscura";
    
    println!("🚀 开始处理 URL: {}", url);
    
    // 初始化
    let mut brain = ReasoningBrain::new();
    let bank = ReasoningBank::new(1000);
    let work_dir = PathBuf::from("/tmp/neotrix_evolve");
    let _ = std::fs::create_dir_all(&work_dir);
    
    let mut evolver = SelfEvolver::new(brain, bank, work_dir);
    
    // 直接处理URL（已知是有效URL）
    println!("✅ 开始进化分析...");
    
    match evolver.evolve_from_url(url) {
        Ok(reward) => {
            println!("✅ 进化完成！奖励值: {:.4}", reward);
            println!("💾 结果已保存到 ~/.neotrix/brain.json");
        }
        Err(e) => {
            eprintln!("❌ 进化失败: {}", e);
        }
    }
}
