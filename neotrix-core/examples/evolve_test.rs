//! 简单的进化测试 - 执行 S-06 流程
use neotrix::neotrix::nt_mind::self_evolver::SelfEvolver;
use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use std::path::PathBuf;

fn main() {
    println!("开始 S-06 自我进化流程...");
    
    // 初始化
    let brain = ReasoningBrain::new();
    let bank = ReasoningBank::new(100);
    let work_dir = PathBuf::from("/tmp/neotrix_evolve");
    let _ = std::fs::create_dir_all(&work_dir);
    
    let mut evolver = SelfEvolver::new(brain, bank, work_dir);
    
    // 测试 helmor 仓库（带重试逻辑）
    let helmor_url = "https://github.com/dohooo/helmor";
    if SelfEvolver::is_url(helmor_url) {
        println!("\n{}", "=".repeat(60));
        println!("处理 helmor 仓库: {}", helmor_url);
        println!("（已启用重试、超时、镜像fallback等容错机制）");
        
        match evolver.evolve_from_url(helmor_url) {
            Ok(reward) => {
                println!("✅ helmor 进化完成！奖励值: {:.4}", reward);
                println!("检查 ~/.neotrix/brain.json 查看变化");
            }
            Err(e) => {
                eprintln!("❌ helmor 进化失败: {}", e);
                eprintln!("提示: 请检查网络连接或尝试使用代理");
            }
        }
    }
    
    // 测试 arXiv 论文
    let arxiv_url = "https://arxiv.org/html/2604.25707v1";
    if SelfEvolver::is_url(arxiv_url) {
        println!("\n{}", "=".repeat(60));
        println!("处理 arXiv 论文: {}", arxiv_url);
        
        match evolver.evolve_from_url(arxiv_url) {
            Ok(reward) => {
                println!("✅ arXiv 进化完成！奖励值: {:.4}", reward);
                println!("检查 ~/.neotrix/brain.json 查看变化");
            }
            Err(e) => {
                eprintln!("❌ arXiv 进化失败: {}", e);
            }
        }
    }
}
