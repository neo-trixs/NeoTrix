use std::io::{self, Write};

/// Standalone 模式 — 纯 ReasoningKernel 推理，不依赖外部 LLM
pub(crate) async fn run_standalone(stage: usize) {
    use neotrix::l1_action::nt_io::nt_io_standalone::StandaloneEngine;
    let mut engine = StandaloneEngine::new(stage.min(18));
    // ⚠️ 2026-10-05 迁移到 nt_term_viz::panel。
    //
    // 首版缺陷（实测可见宽，ANSI 已剥）：
    //   │                       │  空行      w=3   ← 手写了 52 个空格？
    //   │  ReasoningKernel …   │            w=55
    //   │  {stats}              │            w=35  ← 且**被截断**
    //   │  Commands: …          │            w=53
    //   顶边 w=54 / 底边 w=54（另外两串手数横线，与内容行无契约）
    // ⇒ 同一个框里行宽在 3~55 之间乱跳；且 `engine.stats()` 是**变长**串，
    //   塞进 `{:35}` 会被静默截断 ⇒ 信息丢失。
    //
    // 现全部交给 render_panel：内容宽 = 最长内容的可见列宽，
    // 空行交给 ""，`stats()` 不截断（超出即撑宽）。
    let rows = nt_term_viz::panel::render_panel(
        "NeoTrix Standalone Mode",
        &[
            "",
            "  ReasoningKernel v3.0    No external LLM required",
            &format!("  {}", engine.stats()),
            "",
            "  Type your questions. The kernel reasons internally",
            &format!(
                "  through {} stages of neural architecture.",
                stage.min(18) + 1
            ),
            "",
            "  Commands: /stats  /stage <N>  /help  /exit",
        ],
    );
    for r in rows {
        println!("{}", r);
    }

    loop {
        print!("\n❯ ");
        io::stdout().flush().unwrap_or(());
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = input.trim();
                match trimmed {
                    "/exit" | "/q" => {
                        println!("Exiting.");
                        break;
                    }
                    "/stats" | "/s" => println!("{}", engine.stats()),
                    cmd if cmd.starts_with("/stage") => {
                        let n = cmd
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| match s.parse() {
                                Ok(n) => Some(n),
                                Err(e) => {
                                    log::warn!("[main] parse /stage arg: {}", e);
                                    None
                                }
                            })
                            .unwrap_or(18)
                            .min(18);
                        engine.kernel = neotrix::nt_io_standalone::ReasoningKernel::new(n);
                        println!("Switched to stage {}", n);
                    }
                    "/help" | "/h" => {
                        println!("Commands:");
                        println!("  /stats     - Kernel statistics");
                        println!("  /stage <N> - Switch evolution stage (0-18)");
                        println!("  /workflow  - Workflow engine (use --headless for full)");
                        println!("  /mcp       - MCP tool registry");
                        println!("  /help      - This help");
                        println!("  /exit      - Exit");
                        println!("  <text>     - Reason with internal kernel");
                    }
                    cmd if cmd.starts_with("/workflow") => {
                        println!("WorkflowEngine available in headless/TUI mode (use --headless)");
                    }
                    cmd if cmd.starts_with("/mcp") => {
                        let parts: Vec<&str> = cmd.split_whitespace().collect();
                        match parts.get(1).copied() {
                            Some("list") | None => {
                                println!("╭─ MCP Registry (standalone) ───────────────╮");
                                println!("│  Limited MCP support in standalone mode.  │");
                                println!("│  Use --headless for full MCP features.     │");
                                println!("╰────────────────────────────────────────────╯");
                            }
                            Some("status") => {
                                println!("╭─ MCP Server Status ─────────────────────╮");
                                println!("│  Mode: standalone (limited)              │");
                                println!("╰──────────────────────────────────────────╯");
                            }
                            Some(other) => {
                                println!("Unknown mcp subcommand: {}. Try: list, status", other)
                            }
                        }
                    }
                    _ if !trimmed.is_empty() => {
                        let response = engine.reason(trimmed);
                        println!("\n{}", response);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}
