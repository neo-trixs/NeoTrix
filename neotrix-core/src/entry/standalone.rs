use std::io::{self, Write};

/// Standalone 模式 — 纯 ReasoningKernel 推理，不依赖外部 LLM
pub(crate) async fn run_standalone(stage: usize) {
    use log;
    use neotrix::neotrix::nt_io_standalone::StandaloneEngine;
    let mut engine = StandaloneEngine::new(stage.min(18));
    log::info!("╭─ NeoTrix Standalone Mode ──────────────────────────╮");
    log::info!("│                                                    │");
    log::info!("│  ReasoningKernel v3.0    No external LLM required   │");
    log::info!("│  {}                        │", engine.stats());
    log::info!("│                                                    │");
    log::info!("│  Type your questions. The kernel reasons internally │");
    log::info!(
        "│  through {} stages of neural architecture.    │",
        stage.min(18) + 1
    );
    log::info!("│                                                    │");
    log::info!("│  Commands: /stats  /stage <N>  /help  /exit       │");
    log::info!("╰────────────────────────────────────────────────────╯");

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
                        log::info!("Exiting.");
                        break;
                    }
                    "/stats" | "/s" => log::info!("{}", engine.stats()),
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
                        engine.kernel = neotrix::neotrix::nt_core_kernel::ReasoningKernel::new(n);
                        log::info!("Switched to stage {}", n);
                    }
                    "/help" | "/h" => {
                        log::info!("Commands:");
                        log::info!("  /stats     - Kernel statistics");
                        log::info!("  /stage <N> - Switch evolution stage (0-18)");
                        log::info!("  /workflow  - Workflow engine (use --headless for full)");
                        log::info!("  /help      - This help");
                        log::info!("  /exit      - Exit");
                        log::info!("  <text>     - Reason with internal kernel");
                    }
                    cmd if cmd.starts_with("/workflow") => {
                        log::info!(
                            "WorkflowEngine available in headless/TUI mode (use --headless)"
                        );
                    }
                    _ if !trimmed.is_empty() => {
                        let response = engine.reason(trimmed);
                        log::info!("\n{}", response);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                log::error!("Error: {}", e);
                break;
            }
        }
    }
}
