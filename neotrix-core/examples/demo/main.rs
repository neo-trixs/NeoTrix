//! NeoTrix 演示应用 - 主入口
//!
//! 展示 ReasoningBrain + SEAL 循环的实际能力
//! 提供命令行界面选择不同演示

use clap::{Parser, Subcommand};
use colored::Colorize;

mod reasoning_brain_demo;
mod seal_loop_demo;
mod mcp_tools_demo;

use neotrix::reasoning_brain::self_evolver::SelfEvolver;
use neotrix::reasoning_brain::self_iterating::ReasoningBrain;
use neotrix::reasoning_brain::memory::ReasoningBank;

#[derive(Parser)]
#[command(
    name = "neotrix-demo",
    version = "0.1.0",
    about = "NeoTrix 演示应用 - 展示 ReasoningBrain + SEAL 循环",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行 ReasoningBrain 能力进化演示
    Brain {
        /// 运行特定子演示 (1-7, 或全部)
        demo: Option<u32>,
    },
    /// 运行 SEAL 自迭代循环演示
    Seal {
        /// 运行特定子演示 (1-6, 或全部)
        demo: Option<u32>,
    },
    /// 运行 MCP 工具调用演示
    Mcp {
        /// 运行特定子演示 (1-8, 或全部)
        demo: Option<u32>,
    },
    /// 从 URL 自我进化（S-06）
    Evolve {
        /// 要进化的 URL（GitHub 仓库或网页）
        url: String,
    },
    /// 运行所有演示
    All,
}

fn main() {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                    NeoTrix 演示应用                          ║".cyan());
    println!("{}", "║              ReasoningBrain + SEAL 循环展示                   ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    let cli = Cli::parse();
    
    match cli.command {
        Commands::Brain { demo } => {
            run_brain_demos(demo);
        }
        Commands::Seal { demo } => {
            run_seal_demos(demo);
        }
        Commands::Mcp { demo } => {
            run_mcp_demos(demo);
        }
        Commands::Evolve { url } => {
            run_evolve_url(&url);
        }
        Commands::All => {
            run_all_demos();
        }
    }
}

fn run_brain_demos(demo: Option<u32>) {
    match demo {
        Some(1) => {
            reasoning_brain_demo::demo_initial_state();
        }
        Some(2) => {
            reasoning_brain_demo::demo_absorb_single();
        }
        Some(3) => {
            reasoning_brain_demo::demo_absorb_batch();
        }
        Some(4) => {
            reasoning_brain_demo::demo_knowledge_comparison();
        }
        Some(5) => {
            reasoning_brain_demo::demo_task_affinity();
        }
        Some(6) => {
            reasoning_brain_demo::demo_normalization();
        }
        Some(7) => {
            reasoning_brain_demo::demo_generate_self_edit();
        }
        Some(n) => {
            eprintln!("错误: 未知的演示编号 {}，有效范围: 1-7", n);
        }
        None => {
            reasoning_brain_demo::run_all_demos();
        }
    }
}

fn run_seal_demos(demo: Option<u32>) {
    match demo {
        Some(1) => {
            seal_loop_demo::demo_single_seal_loop();
        }
        Some(2) => {
            seal_loop_demo::demo_multiple_seal_loops();
        }
        Some(3) => {
            seal_loop_demo::demo_reasoning_bank();
        }
        Some(4) => {
            seal_loop_demo::demo_batch_seal_loop();
        }
        Some(5) => {
            seal_loop_demo::demo_policy_update();
        }
        Some(6) => {
            seal_loop_demo::demo_rollback();
        }
        Some(n) => {
            eprintln!("错误: 未知的演示编号 {}，有效范围: 1-6", n);
        }
        None => {
            seal_loop_demo::run_all_demos();
        }
    }
}

fn run_mcp_demos(demo: Option<u32>) {
    match demo {
        Some(1) => {
            mcp_tools_demo::demo_mcp_tools_info();
        }
        Some(2) => {
            mcp_tools_demo::demo_create_tools();
        }
        Some(3) => {
            mcp_tools_demo::demo_playwright_verify_mock();
        }
        Some(4) => {
            mcp_tools_demo::demo_cua_check_mock();
        }
        Some(5) => {
            mcp_tools_demo::demo_reasoning_bank_stats();
        }
        Some(6) => {
            mcp_tools_demo::demo_brain_save_load();
        }
        Some(7) => {
            mcp_tools_demo::demo_brain_evaluate();
        }
        Some(8) => {
            mcp_tools_demo::demo_mcp_server();
        }
        Some(n) => {
            eprintln!("错误: 未知的演示编号 {}，有效范围: 1-8", n);
        }
        None => {
            mcp_tools_demo::run_all_demos();
        }
    }
}

fn run_all_demos() {
    println!("\n{}", "正在运行所有演示...".green().bold());
    println!("{}", "=".repeat(60));
    
    println!("\n{}", "第一部分：ReasoningBrain 能力进化".blue().bold());
    reasoning_brain_demo::run_all_demos();
    
    println!("\n{}", "第二部分：SEAL 自迭代循环".blue().bold());
    seal_loop_demo::run_all_demos();
    
    println!("\n{}", "第三部分：MCP 工具调用".blue().bold());
    mcp_tools_demo::run_all_demos();
    
    println!("{}", "\n╔══════════════════════════════════════════════════════════════╗".green());
    println!("{}", "║                    所有演示已完成!                            ║".green());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".green());
}

fn run_evolve_url(url: &str) {
    println!("\n{}", "启动 S-06 外部信息自我进化流程...".green().bold());
    println!("{} {}", "目标 URL:".blue(), url);
    
    // 检查是否为有效 URL
    if !SelfEvolver::is_url(url) {
        eprintln!("{} {}", "错误：".red(), "不是有效的 URL");
        return;
    }
    
    println!("{}", "正在初始化 SelfEvolver...".cyan());
    let brain = ReasoningBrain::new();
    let bank = ReasoningBank::new(100);
    let work_dir = std::env::temp_dir().join("neotrix_evolve");
    let _ = std::fs::create_dir_all(&work_dir);
    
    let mut evolver = SelfEvolver::new(brain, bank, work_dir);
    
    println!("{}", "开始进化...".cyan());
    match evolver.evolve_from_url(url) {
        Ok(reward) => {
            println!("{} {:.4}", "进化完成，奖励值：".green(), reward);
            println!("{}", "注意：此操作会修改 ~/.neotrix/brain.json".yellow());
        }
        Err(e) => {
            eprintln!("{} {}", "进化失败：".red(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parsing() {
        // 验证 CLI 定义正确
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}
