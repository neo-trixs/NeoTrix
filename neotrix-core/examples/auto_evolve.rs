//! 自动进化示例 - 自动检测并执行 URL 分析
//! 从 stdin 读取输入，自动检测 URL 并执行进化
use neotrix::neotrix::nt_mind::self_evolver::SelfEvolver;
use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use std::io::{self, BufRead};
use std::path::PathBuf;

fn main() {
    println!("🚀 Neotrix 自动进化模式");
    println!("{}", "=".repeat(60));
    println!("输入 URL 将自动执行进化分析");
    println!("输入 'quit' 或 'exit' 退出");
    println!("{}", "=".repeat(60));

    // 初始化
    let brain = ReasoningBrain::new();
    let bank = ReasoningBank::new(1000);
    let work_dir = PathBuf::from("/tmp/neotrix_evolve");
    let _ = std::fs::create_dir_all(&work_dir);

    let evolver = SelfEvolver::new(brain, bank, work_dir);

    // 输入循环
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                let trimmed = input.trim();
                
                if trimmed.is_empty() {
                    continue;
                }
                
                // 检查退出命令
                if trimmed == "quit" || trimmed == "exit" {
                    println!("👋 再见！");
                    break;
                }
                
                // 检测是否为 URL
                if SelfEvolver::is_url(trimmed) {
                    println!("\n🔗 检测到 URL: {}", trimmed);
                    println!("⚡ 自动执行进化分析...");
                    
                    match evolver.evolve_from_url(trimmed) {
                        Ok(reward) => {
                            println!("✅ 进化完成！奖励值: {:.4}", reward);
                            println!("💾 结果已保存到 ~/.neotrix/brain.json");
                        }
                        Err(e) => {
                            eprintln!("❌ 进化失败: {}", e);
                        }
                    }
                    println!();
                } else {
                    println!("ℹ️  非 URL 输入: {}", trimmed);
                    println!("提示: 请输入有效的 http(s) URL");
                }
            }
            Err(e) => {
                eprintln!("读取输入失败: {}", e);
                break;
            }
        }
    }
}
