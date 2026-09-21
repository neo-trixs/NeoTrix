//! # ntcode — NeoTrix 对话终端（产品名）
//!
//! ```text
//! ntcode "接入支付功能" [--model provider/model] [--max-rounds 3] [--line]
//! ```
//!
//! goal → 晶体智能拆解 → 池免费模型智能调用 → JEV 融合 →
//! 内需摆上本终端 → 你回车回复 → 回灌重熔 → 收敛/挂起/超轮。
//! 模型选择回到池子：`--model` 定点，否则发现 opencode 免费档进池轮转调用。
//! 形态：TTY 下默认全屏 TUI（借鉴 Claude Code / opencode），管道/CI 或 `--line` 走行式。
//! 退出码：0 收敛 / 2 人沉默挂起 / 3 打满轮次 / 1 参数错误。

use neotrix::l1_action::nt_dialogue_tui::NtTuiHuman;
use neotrix::l1_action::nt_free_pool::NtFreePoolAsk;
use neotrix::l1_action::nt_io::nt_io_provider::catalog::model_pool::UnifiedModelPool;
use neotrix::l1_action::nt_io::nt_io_provider::catalog::cli_free_source::CliFreeSource;
use neotrix::l1_action::nt_model_cli::NtModelCliAsk;
use neotrix::l1_action::nt_stdin_human::NtStdinHuman;
use neotrix::neotrix::nt_crystal_core::{
    CrystalCore, NtInnerLoop, NtLlmAsk, NtLoopStatus, NtTaskLoopConfig,
};
use std::path::PathBuf;
use std::time::Duration;

struct Args {
    goal: String,
    model: Option<String>,
    max_rounds: usize,
    timeout_secs: u64,
    workdir: Option<PathBuf>,
    max_subtasks: usize,
    tui: bool,
    line: bool,
}

fn usage() -> &'static str {
    "用法: ntcode \"<目标>\" [--model provider/model] [--max-rounds N] [--timeout-secs N] [--workdir PATH] [--max-subtasks N] [--tui|--line]\n\
     不指定 --model 则发现 opencode 免费档进池轮转调用；NEOTRIX_DIALOGUE_MODEL 可指定默认模型。\n\
     TTY 下默认全屏 TUI；--line 强制行式；管道/CI 自动回退行式。"
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut goal_parts = Vec::new();
    let mut model: Option<String> = std::env::var("NEOTRIX_DIALOGUE_MODEL").ok();
    let mut max_rounds = 3usize;
    let mut timeout_secs = 300u64;
    let mut workdir: Option<PathBuf> = None;
    let mut max_subtasks = 5usize;
    let mut tui = false;
    let mut line = false;

    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--help" | "-h" => return Err(usage().to_string()),
            "--tui" => {
                tui = true;
            }
            "--line" => {
                line = true;
            }
            "--model" => {
                i += 1;
                model = Some(
                    argv.get(i)
                        .ok_or_else(|| "--model 缺参数".to_string())?
                        .clone(),
                );
            }
            "--max-rounds" => {
                i += 1;
                max_rounds = argv
                    .get(i)
                    .ok_or_else(|| "--max-rounds 缺参数".to_string())?
                    .parse::<usize>()
                    .map_err(|_| "--max-rounds 不是数字".to_string())?
                    .max(1);
            }
            "--timeout-secs" => {
                i += 1;
                timeout_secs = argv
                    .get(i)
                    .ok_or_else(|| "--timeout-secs 缺参数".to_string())?
                    .parse::<u64>()
                    .map_err(|_| "--timeout-secs 不是数字".to_string())?
                    .max(10);
            }
            "--workdir" => {
                i += 1;
                workdir = Some(PathBuf::from(
                    argv.get(i)
                        .ok_or_else(|| "--workdir 缺参数".to_string())?,
                ));
            }
            "--max-subtasks" => {
                i += 1;
                max_subtasks = argv
                    .get(i)
                    .ok_or_else(|| "--max-subtasks 缺参数".to_string())?
                    .parse::<usize>()
                    .map_err(|_| "--max-subtasks 不是数字".to_string())?
                    .max(1);
            }
            other if other.starts_with('-') => {
                return Err(format!("未知参数 {other}\n{usage}", usage = usage()));
            }
            positional => goal_parts.push(positional.to_string()),
        }
        i += 1;
    }

    if goal_parts.is_empty() {
        return Err(usage().to_string());
    }
    Ok(Args {
        goal: goal_parts.join(" "),
        model,
        max_rounds,
        timeout_secs,
        workdir,
        max_subtasks,
        tui,
        line,
    })
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    // 晶体：磁盘有记忆就载入，没有就新建（两条路都不 panic）
    let core = match CrystalCore::load() {
        Ok(c) => {
            println!("已载入晶体记忆。");
            c
        }
        Err(_) => {
            println!("无晶体记忆文件，从空晶体启动。");
            CrystalCore::new("nt-dialogue")
        }
    };

    // 模型选择回到池子：定点 or 发现免费档进池轮转，兜底默认。
    // 返回（问答桥，池摘要行）：摘要行喂给 TUI 侧栏/底栏。
    let (ask, pool_line): (Box<dyn NtLlmAsk>, String) = match &args.model {
        Some(m) => {
            let mut op = NtModelCliAsk::new()
                .with_model(m.clone())
                .with_timeout(Duration::from_secs(args.timeout_secs));
            if let Some(dir) = &args.workdir {
                op = op.with_workdir(dir.clone());
            }
            println!("问答模型（定点）：{m}");
            (Box::new(op), format!("定点 {m}"))
        }
        None => {
            // 全源统一管理：内置源（本地GGUF/免费云/本地端点）+ opencode实时发现，
            // 一张表展示；CLI 可直接调用的只有 cli-free 源，其余需 key/端点。
            let mut pool = UnifiedModelPool::default_pool();
            pool.add_source(Box::new(CliFreeSource::new()));
            let entries = pool.refresh();
            println!("模型池统一清单（{} 个）：", entries.len());
            let mut order: Vec<&str> = Vec::new();
            let mut by_source: std::collections::HashMap<&str, Vec<&str>> =
                std::collections::HashMap::new();
            for e in &entries {
                by_source
                    .entry(e.source.as_str())
                    .or_default()
                    .push(e.id.as_str());
                if !order.contains(&e.source.as_str()) {
                    order.push(e.source.as_str());
                }
            }
            for s in order {
                if let Some(ids) = by_source.get(s) {
                    let show: Vec<&str> = ids.iter().take(8).copied().collect();
                    let more = if ids.len() > 8 {
                        format!(" 等{}个", ids.len() - 8)
                    } else {
                        String::new()
                    };
                    println!("  [{s}] {}个：{}{more}", ids.len(), show.join(", "));
                }
            }
            let cli_ids: Vec<String> = entries
                .iter()
                .filter(|e| e.is_free && e.source == "cli-free")
                .map(|e| e.id.clone())
                .collect();
            if cli_ids.is_empty() {
                println!("池中无可直接调用的免费模型，回退 opencode 默认模型。");
                let mut op = NtModelCliAsk::new()
                    .with_timeout(Duration::from_secs(args.timeout_secs));
                if let Some(dir) = &args.workdir {
                    op = op.with_workdir(dir.clone());
                }
                (Box::new(op), "回退 opencode 默认".to_string())
            } else {
                println!("池免费模型 {} 个轮转调用：{}",
                    cli_ids.len(),
                    cli_ids.join(", "));
                let mut fp = NtFreePoolAsk::new(cli_ids.clone())
                    .with_timeout(Duration::from_secs(args.timeout_secs));
                if let Some(dir) = &args.workdir {
                    fp = fp.with_workdir(dir.clone());
                }
                (
                    Box::new(fp),
                    format!("cli-free×{}", cli_ids.len()),
                )
            }
        }
    };

    let config = NtTaskLoopConfig {
        max_subtasks: args.max_subtasks,
        ..NtTaskLoopConfig::default()
    };
    // 人：TTY 默认全屏 TUI（--line 强制行式；NtTuiHuman 建不起终端也会回退）。
    // 选中逻辑集中在一处，行式与全屏同协议（NtStdinHuman::parse_lines）。
    use std::io::IsTerminal;
    let use_tui = if args.line {
        false
    } else if args.tui {
        true
    } else {
        std::io::stdout().is_terminal()
    };
    let outcome = if use_tui {
        let tui_human = NtTuiHuman::new(pool_line);
        NtInnerLoop::new(config, args.max_rounds).drive(&args.goal, &core, ask.as_ref(), &tui_human)
    } else {
        let human = NtStdinHuman::new();
        NtInnerLoop::new(config, args.max_rounds).drive(&args.goal, &core, ask.as_ref(), &human)
    };

    println!("\n══ 对话实录 ══");
    for line in &outcome.transcript {
        println!("{line}");
    }
    println!("终态：{:?}（{} 轮）", outcome.status, outcome.rounds);
    if !outcome.report.fused.text.is_empty() {
        println!(
            "融合结论（置信 {:.2}）：{}",
            outcome.report.fused.confidence, outcome.report.fused.text
        );
    }

    std::process::exit(match outcome.status {
        NtLoopStatus::Converged => 0,
        NtLoopStatus::Stalled => 2,
        NtLoopStatus::MaxRounds => 3,
    });
}
