use std::io::{self, Write};

/// Standalone 模式 — 纯 ReasoningKernel 推理，不依赖外部 LLM
pub(crate) async fn run_standalone(stage: usize) {
    use neotrix::neotrix::nt_io_standalone::StandaloneEngine;
    let mut engine = StandaloneEngine::new(stage.min(18));
    println!("╭─ NeoTrix Standalone Mode ──────────────────────────╮");
    println!("│                                                    │");
    println!("│  ReasoningKernel v3.0    No external LLM required   │");
    println!("│  {}                        │", engine.stats());
    println!("│                                                    │");
    println!("│  Type your questions. The kernel reasons internally │");
    println!("│  through {} stages of neural architecture.    │", stage.min(18) + 1);
    println!("│                                                    │");
    println!("│  Commands: /stats  /stage <N>  /help  /exit       │");
    println!("╰────────────────────────────────────────────────────╯");

    loop {
        print!("\n❯ ");
        io::stdout().flush().unwrap_or(());
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = input.trim();
                match trimmed {
                    "/exit" | "/q" => { println!("Exiting."); break; }
                    "/stats" | "/s" => println!("{}", engine.stats()),
                    cmd if cmd.starts_with("/stage") => {
                        let n = cmd.split_whitespace().nth(1)
                            .and_then(|s| match s.parse() {
                                Ok(n) => Some(n),
                                Err(e) => {
                                    log::warn!("[main] parse /stage arg: {}", e);
                                    None
                                }
                            }).unwrap_or(18).min(18);
                        engine.kernel = neotrix::neotrix::nt_core_kernel::ReasoningKernel::new(n);
                        println!("Switched to stage {}", n);
                    }
                    "/help" | "/h" => {
                        println!("Commands:");
                        println!("  /stats     - Kernel statistics");
                        println!("  /stage <N> - Switch evolution stage (0-18)");
                        println!("  /workflow  - Workflow engine (use --headless for full)");
                        println!("  /help      - This help");
                        println!("  /exit      - Exit");
                        println!("  <text>     - Reason with internal kernel");
                    }
                    cmd if cmd.starts_with("/workflow") => {
                        println!("WorkflowEngine available in headless/TUI mode (use --headless)");
                    }
                    _ if !trimmed.is_empty() => {
                        let response = engine.reason(trimmed);
                        println!("\n{}", response);
                    }
                    _ => {}
                }
            }
            Err(e) => { eprintln!("Error: {}", e); break; }
        }
    }
}
