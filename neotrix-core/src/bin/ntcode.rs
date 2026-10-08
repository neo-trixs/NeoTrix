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
use neotrix::l1_action::nt_io::nt_io_provider::catalog::cli_free_source::CliFreeSource;
use neotrix::l1_action::nt_io::nt_io_provider::catalog::model_pool::{ModelSource, UnifiedModelPool};
use neotrix::l1_action::nt_act::nt_act_dev_tools::{
    load_external_cli_into, cli_descriptors_from_registry, find_cli_agent_by_name,
};
use neotrix::l1_action::nt_act::nt_act_dev_tools::external_cli_plugins::ensure_agent_args_need_agent;
use neotrix::l1_action::nt_io::nt_io_plugin::registry::global_registry;
use neotrix::l1_action::nt_io::nt_io_plugin::capability_plugins::{
    builtin_capability_plugins, model_sources_from_registry,
};
use neotrix::l1_action::nt_model_cli::NtModelCliAsk;
use neotrix::l1_action::nt_stdin_human::NtStdinHuman;
use neotrix::l5_cognition::nt_crystal_core::{
    CrystalCore, NtInnerLoop, NtLlmAsk, NtLoopStatus, NtTaskLoopConfig,
};
use std::path::PathBuf;
use std::time::Duration;

/// 最后兜底的免费模型（已验证显式 -m 可通；发现与定点全空时用）。
/// 注意：这是资源标识字符串，不是类型名；随时可被 --model 覆盖。
///
/// 2026-10-08 修正：原值 `opencode/mimo-v2.5-free` **已被后端下架** ——
/// 实测 `opencode models | grep -c 'mimo-v2.5-free$'` = **0**，
/// 活的是 `opencode/mimo-v2.6-flash-free`。继续指着旧名 = 「池空时的最后手段」
/// 必然打空。⚠️ 这类**资源标识漂移**是硬编码常量，⛔ 现有门
/// （layer-deps / feature-gates / claims-numbers）全都管不到；
/// 复核方式：`opencode models` 跑一遍，对比本常量。
const FALLBACK_FREE_MODEL: &str = "opencode/mimo-v2.6-flash-free";

struct Args {
    goal: String,
    model: Option<String>,
    max_rounds: usize,
    timeout_secs: u64,
    workdir: Option<PathBuf>,
    max_subtasks: usize,
    tui: bool,
    line: bool,
    agent: Option<String>,
    /// 透传给外部 CLI 插件的启动参数（F1）：按出现顺序累积、重复可给多次，
    /// 追加在 descriptor 的 `args` 之后。⛔ 只对 `--agent` 路径生效。
    agent_args: Vec<String>,
}

fn usage() -> &'static str {
    "用法: ntcode \"<目标>\" [--model provider/model] [--max-rounds N] [--timeout-secs N] [--workdir PATH] [--max-subtasks N] [--tui|--line]\n\
     不指定 --model 则发现 opencode 免费档进池轮转调用；NEOTRIX_DIALOGUE_MODEL 可指定默认模型。\n\
     TTY 下默认全屏 TUI；--line 强制行式；管道/CI 自动回退行式。\n\
     交互式外部 agent 直接以 ntcode --agent freebuff [--workdir PATH] 启动（不进模型池）。\n\
     --agent-arg ARG 追加传给该插件的启动参数（可重复，按出现顺序累积），\
     例：--agent-arg --continue --agent-arg <会话id>。缺 --agent 时给 --agent-arg 会报错。\n\
     注意 freebuff 只认继承来的工作目录，且按**目录名**划分项目历史\
     （家目录/.config/manicode/projects/<目录名>）：想续接哪个项目的会话，\
     就必须让 cwd 落在那个目录上。descriptor 里的 cwd 已钉死到仓库根，\
     而 --workdir 优先级更高、会顶掉它 —— 用 --workdir 覆盖等于换了一个项目历史。"
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
    let mut agent: Option<String> = None;
    let mut agent_args: Vec<String> = Vec::new();

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
            "--agent" => {
                i += 1;
                agent = Some(
                    argv.get(i)
                        .ok_or_else(|| "--agent 缺参数".to_string())?
                        .clone(),
                );
            }
            // F1：调用方透传参数（可重复，按出现顺序累积）。
            // ⚠️ 与 `--model` 等既有 flag 的隔离靠两点，缺一不可：
            //   ① 本 arm 在下面那个「`starts_with('-')` ⇒ 未知参数」的兜底 arm
            //      **之前**，所以 `--agent-arg` 自己不会落进兜底报错；
            //   ② 取值走 `i += 1` 直接吃掉下一个 argv 元素并**原样**存下，
            //      值里即便长得像既有 flag（如 `--model`）也**只当字符串**，
            //      不会再被 match 解析一次 ⇒ 绝不会被既有 flag 抢走。
            "--agent-arg" => {
                i += 1;
                agent_args.push(
                    argv.get(i)
                        .ok_or_else(|| "--agent-arg 缺参数".to_string())?
                        .clone(),
                );
            }
            other if other.starts_with('-') => {
                return Err(format!("未知参数 {other}\n{usage}", usage = usage()));
            }
            positional => goal_parts.push(positional.to_string()),
        }
        i += 1;
    }

    if goal_parts.is_empty() && agent.is_none() {
        return Err(usage().to_string());
    }
    // F1：透传参数只对 `--agent` 路径生效 ⇒ 缺 `--agent` 直接报错（⛔ 不静默忽略：
    // 模型池/chat 那条线根本不 spawn 插件，静默忽略会让用户以为参数传进去了）。
    ensure_agent_args_need_agent(agent.as_deref(), &agent_args)?;
    Ok(Args {
        goal: goal_parts.join(" "),
        model,
        max_rounds,
        timeout_secs,
        workdir,
        max_subtasks,
        tui,
        line,
        agent,
        agent_args,
    })
}

/// 从共享 PluginRegistry 取回全部 `capability="model_source"` 的真实 source。
///
/// 这是统一接入口的取物口（registry 是唯一登记处）；取不到时返回 `None`
/// 交由调用方报错，而不是静默回退到「自己 new 一份」——后者会让 registry
/// 的登记失去意义（登记什么就跑什么）。
fn rt_sources() -> Option<Vec<std::sync::Arc<dyn ModelSource>>> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    let sources = rt.block_on(model_sources_from_registry(&global_registry()));
    if sources.is_empty() {
        None
    } else {
        Some(sources)
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", usage());
        std::process::exit(0);
    }
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    // 外部 CLI descriptor 插件并入统一共享注册表（在 `--agent` 之前，
    // 否则 `find_cli_agent_by_name` 查的是一个空表 —— 「登记什么就跑什么」
    // 的语义要求 registry 先于一切消费方调用而被填充）。
    {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => {
                std::process::exit(1);
            }
        };
        let _ = rt.block_on(load_external_cli_into(&global_registry()));
        let _ = rt
            .block_on(global_registry().load_batch(builtin_capability_plugins()));
    }

    // 交互式外部 agent 插件（descriptor 驱动，热插拔）：不进模型池，不伪装成 chat
    if let Some(name) = &args.agent {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => {
                std::process::exit(1);
            }
        };
        match rt.block_on(find_cli_agent_by_name(&global_registry(), name)) {
            Some(mut p) => {
                // F0：`--workdir` 覆盖 descriptor 自带的 cwd —— CLI 给了以 CLI 为准，
                // 没给才沿用 descriptor 的。⛔ 注意它**只影响 spawn，不影响探活**：
                // `probe_available → run_capture → capture_model_command →
                // run_with_timeout`（`l2_perception/nt_world/social_access/probe.rs`）
                // 全程只有 `Command::new(cmd).args(..)`，**从不调 `current_dir`**。
                // ⇒ 探活跑在 ntcode 自己的 cwd 上。
                // 2026-10-08 更正：此处原注释写「放在 probe 之前，使探活与真正 spawn
                // 跑在同一工作目录」，那是**假的**（R46 文档声称已做而实现从未入库）。
                // 对 `--version` 无害（版本号与 cwd 无关），但别把它当保证。
                p.merge_cli_workdir(args.workdir.as_deref());
                if !p.probe_available() {
                    eprintln!("插件 `{name}` 探活失败（--version 不可用）。");
                    std::process::exit(1);
                }
                // F1：透传参数只走 `launch_with`，**不**参与上面那步探活 ——
                // 探活只跑 descriptor 的 `probe_args`（如 `--version`），
                // 而 `--continue <会话id>` 是一次性会话参数，混进探活会把
                // 「CLI 装不上」误判成「插件不可用」。
                let mut child = match p.launch_with(&args.agent_args) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("启动插件 `{name}` 失败：{e}");
                        std::process::exit(1);
                    }
                };
                let status = child.wait().ok();
                println!("\n(`{name}` 已退出：{:?})", status);
                std::process::exit(0);
            }
            None => {
                let names: Vec<String> = rt
                    .block_on(cli_descriptors_from_registry(&global_registry()))
                    .into_iter()
                    .map(|p| p.name)
                    .collect();
                if names.is_empty() {
                    eprintln!("未找到任何 CLI 插件（registry 未登记 cli 能力）。");
                } else {
                    eprintln!("未知插件 `{name}`。已安装：{}", names.join(", "));
                }
                std::process::exit(1);
            }
        }
    }

    // 磁盘有记忆就载入，没有就新建（两条路都不 panic）
    let mut core = match CrystalCore::load() {
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
    // 返回（问答桥，池共享句柄，池展示行，选择器模型）：TUI 事件驱动与行式共用。
    let (ask, pool_arc, pool_lines, picker_models): (
        std::sync::Arc<dyn NtLlmAsk>,
        Option<std::sync::Arc<NtFreePoolAsk>>,
        Vec<String>,
        Vec<String>,
    ) = match &args.model {
        Some(m) => {
            // 若模型定点的主名是某个「交互 CLI 插件」的名字，说明它是
            // 会话型 agent，不能当 chat completion 路由——显式报错并指引到
            // `--agent`，不要维 fallback 装进 headless 执行器。
            let (probe,) = (m.split('/').next().unwrap_or(m),);
            // 守门查共享 registry（capability=cli_agent），不直读 plugins 目录：
            // 这条 CLI 能力也要服从「登记什么就拦什么」。
            let is_cli_agent = {
                let rt = tokio::runtime::Runtime::new().ok();
                match rt {
                    Some(rt) => rt
                        .block_on(cli_descriptors_from_registry(&global_registry()))
                        .iter()
                        .any(|p| p.name == probe && p.mode == "interactive"),
                    None => false,
                }
            };
            if is_cli_agent {
                eprintln!(
                    "「{m}」是交互 CLI agent 插件，不能用于 chat completion。\n\
                     请改用 `ntcode --agent {probe}` 启动它。"
                );
                std::process::exit(1);
            }
            let mut op = NtModelCliAsk::new()
                .with_model(m.clone())
                .with_timeout(Duration::from_secs(args.timeout_secs));
            if let Some(dir) = &args.workdir {
                op = op.with_workdir(dir.clone());
            }
            println!("问答模型（定点）：{m}");
            (
                std::sync::Arc::new(op),
                None,
                vec![format!("定点 {m}")],
                vec![m.clone()],
            )
        }
        None => {
            // 统一接入口：池的 source 全部来自 PluginRegistry（capability=
            // "model_source"）交回的共享实例，ntcode 不再自行 new —— 故此处
            // registry 登记了什么，池就跑什么（含 opencode 的 cli-free 源）。
            let pool = match rt_sources() {
                Some(sources) => UnifiedModelPool::from_registry_sources(sources),
                None => {
                    eprintln!("PluginRegistry 未提供任何 model_source，请检查注册步骤。");
                    std::process::exit(1);
                }
            };
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
                // 诊断：发现源不可用还是真的无免费模型（第一单曾踩空，原因未定位）
                let available = CliFreeSource::new().is_available();
                println!("池中无可直接调用的免费模型（发现源可用：{available}）。");
                // 兜底定点：默认路由曾报服务端错，显式 -m 已验证可通；
                // 仅为最后手段，可被 --model / NEOTRIX_DIALOGUE_MODEL 覆盖。
                let mut op = NtModelCliAsk::new()
                    .with_model(FALLBACK_FREE_MODEL)
                    .with_timeout(Duration::from_secs(args.timeout_secs));
                if let Some(dir) = &args.workdir {
                    op = op.with_workdir(dir.clone());
                }
                println!("回退定点免费模型：{FALLBACK_FREE_MODEL}");
                (
                    std::sync::Arc::new(op),
                    None,
                    vec![format!("回退 {FALLBACK_FREE_MODEL}")],
                    vec![FALLBACK_FREE_MODEL.to_string()],
                )
            } else {
                println!("池免费模型 {} 个轮转调用：{}",
                    cli_ids.len(),
                    cli_ids.join(", "));
                let mut fp = NtFreePoolAsk::new(cli_ids.clone())
                    .with_timeout(Duration::from_secs(args.timeout_secs));
                if let Some(dir) = &args.workdir {
                    fp = fp.with_workdir(dir.clone());
                }
                let pool_line = format!("cli-free×{}", cli_ids.len());
                let msgs: Vec<String> = cli_ids
                    .iter()
                    .map(|id| format!("池 {id}"))
                    .collect();
                let arc = std::sync::Arc::new(fp);
                (
                    arc.clone() as std::sync::Arc<dyn NtLlmAsk>,
                    Some(arc),
                    std::iter::once(pool_line).chain(msgs).collect(),
                    cli_ids,
                )
            }
        }
    };

    let config = NtTaskLoopConfig {
        max_subtasks: args.max_subtasks,
        ..NtTaskLoopConfig::default()
    };
    // 人：TTY 默认全屏 TUI（--line 强制行式）。
    // v2 事件驱动（工作线程 + 实时渲染 + Esc 取消）；终端建失败自动回退 v1 行式。
    use std::io::IsTerminal;
    let use_tui = if args.line {
        false
    } else if args.tui {
        true
    } else {
        std::io::stdout().is_terminal()
    };
    let outcome = if use_tui {
        let history = Vec::new();
        match neotrix::l1_action::nt_tui_app::run_tui_session(
            args.goal.clone(),
            core,
            ask.clone(),
            pool_arc.unwrap_or_else(|| {
                std::sync::Arc::new(NtFreePoolAsk::new(picker_models.clone()))
            }),
            pool_lines.clone(),
            history,
            config,
            args.max_rounds,
        ) {
            Ok((outcome, core_back)) => {
                core = core_back;
                outcome
            }
            Err((e, core_back)) => {
                core = core_back;
                eprintln!("TUI 启动失败（{e}），回退行式。");
                let tui_human =
                    NtTuiHuman::new(pool_lines.first().cloned().unwrap_or_default());
                NtInnerLoop::new(config, args.max_rounds).drive(
                    &args.goal,
                    &mut core,
                    ask.as_ref(),
                    &tui_human,
                )
            }
        }
    } else {
        let human = NtStdinHuman::new();
        NtInnerLoop::new(config, args.max_rounds).drive(&args.goal, &mut core, ask.as_ref(), &human)
    };

    // 记忆落盘：循环已写回内存，这里显式持久化（失败如实打印，不改退出码）
    match NtInnerLoop::persist(&core) {
        Ok(()) => println!("晶体记忆已落盘。"),
        Err(e) => eprintln!("晶体记忆落盘失败：{e}"),
    }

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

/// F1 的命令行层测试（`cargo test -p neotrix --bin ntcode`）。
///
/// 为什么需要单独一层：`external_cli_plugins.rs` 里那几条钉住的是**透传语义**
/// （顺序累积 / 不污染 probe / 只对 agent 生效），而**这一层**钉的是 argv 解析
/// 本身 —— 缺参数要报错、值不能被既有 flag 二次解析、按出现顺序累积。
/// `parse_args` 在 bin 里，`--lib` 测试**永远编译不到它** ⇒ 不补这一层，
/// 参数解析就是零覆盖。
#[cfg(test)]
mod tests {
    use super::*;

    /// 构造 argv（补上 argv[0]，parse_args 从 i=1 开始）。
    fn argv_of(items: &[&str]) -> Vec<String> {
        std::iter::once("ntcode")
            .chain(items.iter().copied())
            .map(|s| s.to_string())
            .collect()
    }

    fn parse_ok(items: &[&str]) -> Args {
        match parse_args(&argv_of(items)) {
            Ok(a) => a,
            Err(e) => panic!("parse_args({items:?}) 应当成功，却报错：{e}"),
        }
    }

    fn parse_err(items: &[&str]) -> String {
        match parse_args(&argv_of(items)) {
            Ok(a) => panic!(
                "parse_args({items:?}) 应当报错，却成功（agent_args={:?} model={:?}）",
                a.agent_args, a.model
            ),
            Err(e) => e,
        }
    }

    /// F1 ③：多个 `--agent-arg` **按出现顺序**累积，重复的不去重、不排序。
    ///
    /// 为什么可能失败：任何一次「覆盖而非追加」「排序」「去重」「只保留最后一个」
    /// 都会让下面这条变红。
    #[test]
    fn agent_args_accumulate_in_occurrence_order() {
        let a = parse_ok(&[
            "--agent",
            "freebuff",
            "--agent-arg",
            "--continue",
            "--agent-arg",
            "conv-42",
            "--agent-arg",
            "--continue",
            "--agent-arg",
            "conv-7",
        ]);
        assert_eq!(
            a.agent_args,
            vec!["--continue", "conv-42", "--continue", "conv-7"],
            "透传参数必须按出现顺序累积、保留重复项"
        );
        assert_eq!(a.agent.as_deref(), Some("freebuff"), "--agent 不该被透传参数影响");
    }

    /// F1 ①：缺参数必须报错，且错误信息指名 `--agent-arg`（不是笼统的「未知参数」）。
    ///
    /// 为什么可能失败：漏掉 `ok_or_else` 会让 `argv.get(i)` 变成 `None` 解引用
    /// ⇒ panic 而非返回 Err；或者错误信息退化成兜底 arm 的「未知参数 --agent-arg」
    /// ⇒ 下面 `contains("--agent-arg 缺参数")` 变红。
    #[test]
    fn agent_arg_missing_value_is_an_error() {
        let e = parse_err(&["--agent", "freebuff", "--agent-arg"]);
        assert!(
            e.contains("--agent-arg 缺参数"),
            "缺参数必须指名 --agent-arg，实际：{e}"
        );
    }

    /// F1 ②：透传参数的值**不会被既有 flag 二次解析**。
    ///
    /// 手法：把一个**长得像既有 flag 的串**当 `--agent-arg` 的值，再跟一个真正的
    /// 模型名。若 `--agent-arg` 正确吃掉下一个 argv 元素 ⇒ `--model` 落到
    /// `agent_args` 里、`model` 不变；若它没吃掉（或被兜底 arm 当未知参数）⇒
    /// `--model` 会被 match 再解析一次，把 `x/y` 收进 `model`。
    /// ⛔ 不断言 `model == None`：`parse_args` 会拿 `NEOTRIX_DIALOGUE_MODEL` 当默认值，
    /// 断言「等于 None」会依赖环境变量 ⇒ 变成一条**可能因环境而红的假断言**。
    ///
    /// ⚠️ 2026-10-08 更正：本测试原先断言 `agent_args == ["--model", "x/y"]`，
    /// 那是**测试自己的错**：`--agent-arg` 是**单值** flag（usage 里写的就是
    /// `--agent-arg ARG`，且示例 `--agent-arg --continue --agent-arg <会话id>`
    /// 是两次出现两次传参），它只吃**下一个** argv 元素。原断言要求它贪心吃两个
    /// ⇒ 与声明的契约矛盾。实现是对的，断言是错的 —— 现按真实契约改写，
    /// 并补上「剩下的裸词去了哪」，防止有人日后用「改成贪心」来迎合旧断言。
    #[test]
    fn agent_arg_value_is_not_reparsed_as_an_ntcode_flag() {
        let a = parse_ok(&["--agent", "freebuff", "--agent-arg", "--model", "x/y"]);
        // 单值语义：`--agent-arg` 只收下紧跟其后的那一个元素。
        assert_eq!(
            a.agent_args,
            vec!["--model"],
            "--agent-arg 是单值 flag，只应吃掉紧跟其后的一个 argv 元素"
        );
        assert_ne!(
            a.model.as_deref(),
            Some("--model"),
            "`--model` 被当成了 flag 触发（应只作为 --agent-arg 的值）"
        );
        assert_ne!(
            a.model.as_deref(),
            Some("x/y"),
            "x/y 是透传参数的值，绝不能被解析成定点模型"
        );
        // 剩下的裸词合法落回 goal —— 它不是「丢失」，是被单值语义明确界定的去处。
        assert_eq!(
            a.goal, "x/y",
            "未被 --agent-arg 收下的裸词应落回 goal，而不是被静默吞掉"
        );
    }

    /// F1 ②-补：**两次** `--agent-arg` 各自带一个形似 flag 的值，且不留裸词。
    ///
    /// 为什么要有这条（2026-10-08 补）：上一条测试喂的是
    /// `[--agent-arg --model x/y]` —— `--model` 被正确收作值后，**`x/y` 变成裸词**，
    /// 于是「`x/y` 到底该落进 `agent_args` 还是落回 `goal`」变成一个
    /// **与本测试目的无关的歧义**（一度导致同一测试里出现两条互相矛盾的断言：
    /// 一条要 `goal == "x/y"`，一条要 `goal` 为空）。
    ///
    /// 正确解法不是二选一，而是**让输入不含歧义**：两个值都用 `--agent-arg` 显式传，
    /// 一个裸词都不留。这样 `goal` 必须为空、`agent_args` 必须按序收齐两个值 ——
    /// 两条断言同时成立，且各自都能在实现被改坏时真的红。
    ///
    /// 另：这条同时钉住 usage 里承诺的**多次出现按序累积**形态
    /// （`--agent-arg --continue --agent-arg <会话id>`）。
    #[test]
    fn agent_arg_takes_one_value_each_and_leaves_no_bare_word() {
        let a = parse_ok(&[
            "--agent",
            "freebuff",
            "--agent-arg",
            "--continue",
            "--agent-arg",
            "2026-10-07T11-53-48.313Z",
        ]);
        assert_eq!(
            a.agent_args,
            vec!["--continue", "2026-10-07T11-53-48.313Z"],
            "两次 --agent-arg 必须各收一个值并按出现顺序累积"
        );
        assert!(
            a.goal.is_empty(),
            "每个值都被 --agent-arg 显式收下了，不该有裸词落进 goal（实测 goal={:?}）",
            a.goal
        );
    }

    /// F1 ③-负例：给了 `--agent-arg` 却没给 `--agent` ⇒ **必须报错**。
    ///
    /// 为什么可能失败：一旦这条被放行（静默忽略），模型池/chat 那条线上
    /// `--agent-arg` 会被丢掉，而用户以为 `--continue` 传进去了。
    #[test]
    fn agent_args_without_agent_is_an_error() {
        let e = parse_err(&["做点事", "--agent-arg", "--continue"]);
        assert!(
            e.contains("--agent"),
            "错误信息要指回 --agent（让用户知道怎么改），实际：{e}"
        );
    }

    /// 反向回归：普通 goal 路径（既没 agent 也没透传）必须照旧解析成功。
    ///
    /// 为什么可能失败：把「无 agent 且无透传」也判错，会让**所有**普通
    /// `ntcode "目标"` 调用直接报错 —— 这是最容易被 F1 顺手打破的一条。
    #[test]
    fn plain_goal_path_is_unaffected() {
        let a = parse_ok(&["做点事", "--max-rounds", "2"]);
        assert_eq!(a.goal, "做点事", "goal 解析不能被 F1 影响");
        assert_eq!(a.max_rounds, 2, "既有 flag 的解析不能被 F1 影响");
        assert!(a.agent.is_none() && a.agent_args.is_empty(), "普通路径不该有 agent/透传参数");
    }
}
