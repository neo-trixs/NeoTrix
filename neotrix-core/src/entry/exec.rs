//! exec — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::io::{self, Write};

use super::{build_brain, err, tokio_runtime};
use neotrix::l1_action::nt_io::nt_io_mention::resolve_mentions;
use neotrix::l1_action::nt_core_task_dispatcher::{DispatcherConfig, TaskDecomposerDispatcher};
use neotrix::l1_action::nt_io::nt_io_standalone::ReasoningKernel;
use neotrix::l5_cognition::nt_core_cot_generator::{CoTConfig, DefaultCoTGenerator};
use neotrix::l5_cognition::nt_core_policy::E8Policy;

/// Resolve the effective prompt from positional arg, file, or stdin.
pub fn resolve_prompt(prompt: Option<&str>, file: Option<&str>, pipe: bool) -> String {
    if let Some(p) = prompt {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    if let Some(f) = file {
        let path = std::path::Path::new(f);
        if path.exists() {
            return std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("{}: {}", err("Read file error"), e);
                String::new()
            });
        }
        eprintln!("{}: file not found: {}", err("Error"), f);
        return String::new();
    }
    if pipe {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::stdin().lock().read_to_string(&mut buf);
        return buf.trim().to_string();
    }
    String::new()
}

pub fn run_exec(prompt: &str, json_output: bool, stream: bool, timeout_secs: u64) {
    if prompt.is_empty() {
        if json_output {
            use neotrix::l0_substrate::nt_core_jsonl::JsonlWriter;
            let mut writer = JsonlWriter::new();
            writer.emit_error("Empty prompt", Some("EMPTY_PROMPT"), false);
            writer.emit_finish("", 0, 0, 1);
        } else {
            eprintln!("error: empty prompt");
        }
        return;
    }
    // src/cli/commands 已删除: slash 输入不再走命令注册表,
    // 与 desktop.rs 一致直接落到正常流程 (视为 LLM prompt)。
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() && !json_output {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let start = std::time::Instant::now();
    let rt = tokio_runtime();

    if json_output {
        use neotrix::l0_substrate::nt_core_jsonl::JsonlWriter;
        let mut writer = JsonlWriter::new();
        writer.emit_start(&prompt, None, None, None);

        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let task = async {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt)
                } else {
                    let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            };
            tokio::time::timeout(timeout, task).await
        });

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                let tokens_used = (response.len() / 4) as u32;
                writer.emit_message("assistant", &response, Some(tokens_used));
                writer.emit_finish(&response, tokens_used, elapsed, 0);
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                writer.emit_error(&msg, Some("LLM_ERROR"), true);
                writer.emit_finish("", 0, elapsed, 1);
            }
            Err(_timeout) => {
                let msg = format!("Execution timed out after {}s", timeout_secs);
                writer.emit_error(&msg, Some("TIMEOUT"), true);
                writer.emit_finish("", 0, elapsed, 124);
            }
        }
    } else if stream {
        // Streaming mode — print tokens as they arrive
        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            if let Some(ref mut engine) = agent.reasoning_engine {
                match engine.reason_stream(&prompt, None).await {
                    Ok((_full, mut rx)) => {
                        while let Some(token) = rx.recv().await {
                            print!("{}", token);
                            io::stdout().flush().ok();
                        }
                        println!();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                let r = agent.iterate(task_type);
                println!("Learned: {:.3} → {:.3}", r.score_before, r.score_after);
                Ok(())
            }
        });

        if let Err(e) = result {
            eprintln!("error: {}", e);
        }
    } else {
        // Plain text mode (original behavior)
        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let task = async {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt)
                } else {
                    let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            };
            tokio::time::timeout(timeout, task).await
        });

        let _elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                println!("{}", response);
            }
            Ok(Err(e)) => {
                eprintln!("error: {}", e);
            }
            Err(_timeout) => {
                eprintln!("error: execution timed out after {}s", timeout_secs);
            }
        }
    }
}

pub fn run_one_shot(prompt: &str, format: Option<&str>, profile: &str, stream: bool) {
    if prompt.is_empty() {
        eprintln!(
            "{}: usage: neotrix run <prompt> | neotrix reason <prompt>",
            err("Error")
        );
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let rt = tokio_runtime();

    // Check if task is complex and should use TaskDispatcher
    let use_dispatcher = is_complex_task(&prompt);

    if stream {
        // Streaming mode — print tokens as they arrive, no progress bar
        rt.block_on(async {
            let mut agent = build_brain(profile);

            let result = if let Some(ref mut engine) = agent.reasoning_engine {
                match engine.reason_stream(&prompt, None).await {
                    Ok((full_response, mut rx)) => {
                        while let Some(token) = rx.recv().await {
                            print!("{}", token);
                            io::stdout().flush().ok();
                        }
                        println!();
                        if format == Some("json") {
                            let json = serde_json::json!({
                                "success": true,
                                "response": full_response,
                                "prompt": prompt,
                            });
                            eprintln!(
                                "{}",
                                serde_json::to_string_pretty(&json).unwrap_or_default()
                            );
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                let r = agent.iterate(task_type);
                let msg = format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after);
                if format == Some("json") {
                    let json =
                        serde_json::json!({"success": true, "response": msg, "prompt": prompt});
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or(msg));
                } else {
                    println!("{}", msg);
                }
                Ok(())
            };
            if let Err(e) = result {
                if format == Some("json") {
                    let json = serde_json::json!({"success": false, "error": e.to_string()});
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    );
                } else {
                    eprintln!("{}: {}", err("Reasoning error"), e);
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    } else if use_dispatcher {
        // Use TaskDispatcher for complex tasks
        rt.block_on(async {
            let mut agent = build_brain(profile);

            // Extract components from the agent before moving it (single brain instance)
            let gateway = agent
                .reasoning_engine
                .as_ref()
                .and_then(|e| e.gateway.clone());
            let reasoning_engine = agent.reasoning_engine.take();
            let kernel = ReasoningKernel::new(3);
            let e8_policy = E8Policy::default();

            let mut dispatcher = match (gateway, reasoning_engine) {
                (Some(gw), Some(re)) => {
                    TaskDecomposerDispatcher::new(gw.clone(), DispatcherConfig::from_env())
                        // Phase 2 top-up (SIM-47)：CoT 显式注入，恢复 new() 自构前的行为。
                        .with_cot_generator(DefaultCoTGenerator::new(gw, CoTConfig::default()))
                        .with_reasoning_engine(Box::new(re))
                        .with_kernel(kernel)
                        .with_e8_policy(e8_policy)
                }
                _ => {
                    eprintln!(
                        "{}: missing gateway or reasoning engine",
                        err("Reasoning error")
                    );
                    return;
                }
            };

            let result = dispatcher.decompose_and_execute(&prompt).await;
            match result {
                Ok(response) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": true,
                            "response": response,
                            "prompt": prompt,
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or(response)
                        );
                    } else {
                        println!("\n{}", response);
                    }
                }
                Err(e) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": false,
                            "error": e.to_string(),
                        });
                        eprintln!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or_default()
                        );
                    } else {
                        eprintln!("{}: {}", err("Reasoning error"), e);
                    }
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    } else {
        // Non-streaming mode — original behavior with progress bar
        rt.block_on(async {
            let mut agent = build_brain(profile);

            let pb = indicatif::ProgressBar::new(100);
            match indicatif::ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {percent}% {msg}")
            {
                Ok(style) => pb.set_style(style.progress_chars("█▉▊▋▌▍▎▏ ")),
                Err(e) => eprintln!("{}: invalid progress bar template: {}", err("Error"), e),
            }
            pb.set_message("reasoning...");

            let result = if let Some(ref mut engine) = agent.reasoning_engine {
                pb.inc(30);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let r = engine.reason(&prompt);
                pb.finish_with_message("done");
                r
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                pb.inc(50);
                let r = agent.iterate(task_type);
                pb.finish_with_message("done");
                Ok(format!(
                    "Learned: {:.3} → {:.3}",
                    r.score_before, r.score_after
                ))
            };

            match result {
                Ok(response) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": true,
                            "response": response,
                            "prompt": prompt,
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or(response)
                        );
                    } else {
                        println!("\n{}", response);
                    }
                }
                Err(e) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": false,
                            "error": e.to_string(),
                        });
                        eprintln!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or_default()
                        );
                    } else {
                        eprintln!("{}: {}", err("Reasoning error"), e);
                    }
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    }
}

/// Check if a task is complex enough to use the TaskDispatcher
pub(crate) fn is_complex_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let complex_keywords = [
        "analyze",
        "design",
        "implement",
        "debug",
        "refactor",
        "optimize",
        "architecture",
        "plan",
        "research",
        "compare",
        "evaluate",
        "step by step",
        "think through",
        "break down",
        "decompose",
    ];
    complex_keywords.iter().any(|kw| lower.contains(kw)) || prompt.len() > 200
}
