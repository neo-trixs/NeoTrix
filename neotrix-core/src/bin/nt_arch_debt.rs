//! nt-arch-debt —— 架构债红蓝对抗评分报告（裁定 ③A）
//!
//! 读 `.neotrix/arch-known-violations.tsv`（已知违反项清单，由
//! `scripts/check-arch-rules.sh` 判据⑤保证非空），跑红蓝对抗，
//! 输出**按收敛分降序**的裁决建议。
//!
//! # ⛔ 本命令不做架构微调
//!
//! 跨层依赖是**跨窗口共享的架构决策**。2,465 个可分类文件 + 13 条已知分层债
//! 不是本工具能单方面改的 ⇒ 本命令只产出**信号**，每条都带 `⚖️ 裁决：` 提示。
//!
//! 用法：
//! ```text
//! cargo run -p neotrix --bin nt-arch-debt
//! cargo run -p neotrix --bin nt-arch-debt -- --violations <path> --baseline <path>
//! ```
//!
//! 退出码：0 = 报告已产出（**不代表架构健康**）；2 = 用法错误。

use neotrix::l6_meta::coordination::nt_arch_rules::{load_known_violations, report_from_file, rank_layer_debts};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = repo_root();
    let mut viol = format!("{root}/.neotrix/arch-known-violations.tsv");
    let mut base = format!("{root}/.neotrix/arch-rules-baseline.txt");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--violations" | "-v" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("用法: --violations <path>");
                    std::process::exit(2);
                }
                viol = args[i].clone();
            }
            "--baseline" | "-b" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("用法: --baseline <path>");
                    std::process::exit(2);
                }
                base = args[i].clone();
            }
            "--help" | "-h" => {
                println!("nt-arch-debt — 架构债红蓝对抗评分报告（裁定 ③A）");
                println!();
                println!("读已知违反项清单 → 红蓝对抗（JudgePanel::deliberate）→ 排序建议。");
                println!("⛔ 只产出信号，不改代码：跨层依赖是跨窗口共享的架构决策。");
                println!();
                println!("  --violations, -v <path>   已知违反项清单（默认 .neotrix/arch-known-violations.tsv）");
                println!("  --baseline,  -b <path>   豁免基线（默认 .neotrix/arch-rules-baseline.txt）");
                std::process::exit(0);
            }
            other => {
                eprintln!("未知参数: {other}（--help 看用法）");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    println!("{}", report_from_file(&viol, &base));

    // 附加一段：清单为空时明确区分「无债」与「没登记」
    let debts = load_known_violations(&viol);
    if debts.is_empty() {
        eprintln!(
            "⛔ 清单为空或不可读: {viol}\n\
             ⚠️ 这**不等于「架构无债」** —— 门 scripts/check-arch-rules.sh 判据⑤\n   \
             应当已判红；若门是绿的，说明清单刚被清空。"
        );
        std::process::exit(2);
    }
    // 明示排序已降序，便于人工扫读
    let base_text = std::fs::read_to_string(&base).unwrap_or_default();
    let reader = |p: &str| base_text.contains(p);
    let ranked = rank_layer_debts(&debts, &reader);
    if ranked.windows(2).any(|w| w[0].converged_score < w[1].converged_score) {
        eprintln!("⛔ 排序断言失效：报告不是按收敛分降序 ⇒ 排序逻辑被改坏");
        std::process::exit(2);
    }
    println!("---");
    println!("共 {} 条；排序已自校验为降序。", ranked.len());
}

/// 从可执行文件位置反推仓库根（`target/debug/` 往上两级）。
fn repo_root() -> String {
    std::env::var("NEOTRIX_REPO_ROOT").unwrap_or_else(|_| {
        let exe = std::env::current_exe().unwrap_or_default();
        // <root>/target/{profile}/nt-arch-debt  ⇒ 上溯三级
        exe.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string())
    })
}