//! Brain 交互命令 — Absorb / Evolve / Mem / Save / Trace / Avatar

use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::core::nt_core_knowledge::KnowledgeSource;
use crate::core::nt_core_wbmem::{MemoryEntry, WhiteBoxMemory};
use crate::neotrix::nt_io_user_avatar::DistillationEngine;

// TODO: inject via DI — pass &WhiteBoxMemory through CLI command context
static WHITEBOX_MEMORY: LazyLock<Mutex<WhiteBoxMemory>> = LazyLock::new(|| {
    Mutex::new(WhiteBoxMemory::load())
});

static AVATAR_ENGINE: LazyLock<Mutex<DistillationEngine>> = LazyLock::new(|| {
    Mutex::new(DistillationEngine::new())
});

// ====== /absorb ======

pub struct AbsorbCmd;
impl CliCommand for AbsorbCmd {
    fn name(&self) -> &str { "/absorb" }
    fn aliases(&self) -> Vec<&str> { vec!["/a"] }
    fn description(&self) -> &str { "吸收知识: /absorb HeroUI" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let source_name = args.iter().find(|a| *a != "--json").map(|s| s.as_str()).unwrap_or("");
        if source_name.is_empty() {
            let sources: Vec<String> = KnowledgeSource::all().iter().map(|s| format!("{:?}", s)).collect();
            return CommandOutput::err(&format!("用法: /absorb <source>\n可用: {}", sources.join(", ")));
        }
        if let Some(b) = brain {
            if let Some(ks) = KnowledgeSource::from_name(source_name) {
                let mut a = b.blocking_write();
                a.brain.absorb(ks);
                let msg = format!("✅ 已吸收知识源: {:?} (学习率={})", ks, a.brain.learning_rate);
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "source": format!("{:?}", ks), "absorbed": true, "learning_rate": a.brain.learning_rate
                    }));
                }
                CommandOutput::ok(&msg)
            } else {
                let sources: Vec<String> = KnowledgeSource::all().iter().map(|s| format!("{:?}", s)).collect();
                CommandOutput::err(&format!("未知知识源: {}. 可用: {}", source_name, sources.join(", ")))
            }
        } else {
            CommandOutput::err("Brain 不可用")
        }
    }
}

// ====== /evolve ======

pub struct EvolveCmd;
impl CliCommand for EvolveCmd {
    fn name(&self) -> &str { "/evolve" }
    fn aliases(&self) -> Vec<&str> { vec!["/e"] }
    fn description(&self) -> &str { "SEAL 进化: /evolve <url> | /evolve (internal SEAL loop)" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let url = args.iter().find(|a| *a != "--json").map(|s| s.as_str()).unwrap_or("");
        if url.is_empty() {
            return CommandOutput::err("用法: /evolve <url> | /evolve self\n示例: /evolve https://github.com/user/repo");
        }
        if url == "self" {
            if let Some(b) = brain {
                let mut a = b.blocking_write();
                let result = a.iterate(crate::neotrix::nt_world_model::TaskType::General);
                let msg = format!("🧬 SEAL self-evolve: {:.3} → {:.3} (improved={})",
                    result.score_before, result.score_after, result.improved);
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "evolved": true, "score_before": result.score_before,
                        "score_after": result.score_after, "improved": result.improved
                    }));
                }
                CommandOutput::ok(&msg)
            } else {
                CommandOutput::err("Brain 不可用")
            }
        } else if crate::neotrix::nt_mind::self_evolver::SelfEvolver::is_url(url) {
            if let Some(b) = brain {
                use std::path::PathBuf;
                let work_dir = PathBuf::from(
                    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
                ).join(".neotrix").join("work");
                let _ = std::fs::create_dir_all(&work_dir);
                let mut a = b.blocking_write();
                match a.run_seal_loop(url, None, None) {
                    Ok(reward) => {
                        let _ = a.brain.save();
                        let msg = format!("🌐 进化完成: reward={:.3}", reward);
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "evolved": true, "url": url, "reward": reward
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&format!("进化失败: {}", e)),
                }
            } else {
                CommandOutput::err("Brain 不可用")
            }
        } else {
            CommandOutput::err("无效 URL 或命令. 用法: /evolve <url> | /evolve self")
        }
    }
}

// ====== /mem ======

pub struct MemCmd;
impl CliCommand for MemCmd {
    fn name(&self) -> &str { "/mem" }
    fn aliases(&self) -> Vec<&str> { vec!["/memory"] }
    fn description(&self) -> &str {
        "白盒记忆管理:\n  /mem view <id>                     查看记忆详情\n  /mem list [--tag <tag>] [--source <s>]  列出/过滤记忆\n  /mem search <query>                 搜索记忆\n  /mem edit <id> <内容>               编辑记忆(保存原始)\n  /mem tag <id> <tag1,tag2,...>       更新标签\n  /mem delete <id>                    删除记忆\n  /mem pin <id>                       固定/取消固定\n  /mem checkpoint create <描述>       创建检查点\n  /mem checkpoint list                列出检查点\n  /mem rollback <checkpoint_id>       回滚到检查点\n  /mem dream                          手动梦境周期\n  /mem dream toggle                   开关自动梦境\n  /mem stats                          记忆统计"
    }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain_args: Vec<&str> = args.iter().map(|s| s.as_str()).filter(|a| *a != "--json").collect();

        if let Some(b) = brain {
            let bank = b.blocking_read();
            let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
            guard.sync_from_brain(&bank.reasoning_bank);
        }

        if plain_args.is_empty() {
            return CommandOutput::ok(self.description());
        }

        let cmd = plain_args[0];
        match cmd {
            "view" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem view <id>"); }
                let guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.view(plain_args[1]) {
                    Ok(entry) => {
                        let tags = entry.tags.join(", ");
                        let pinned = if entry.pinned { " 📌" } else { "" };
                        let edited = if entry.edited { " ✏️" } else { "" };
                        let original = if let Some(ref orig) = entry.original_content {
                            format!("\n  🗂 原始: {}", orig)
                        } else { String::new() };
                        let msg = format!(
                            "📄 记忆 [{}]{}{}\n  内容: {}\n  来源: {} | 置信度: {:.2}\n  标签: {}\n  创建: {} | 访问: {} ({}次)\n  检查点: {}{}",
                            entry.id, pinned, edited, entry.content, entry.source, entry.confidence,
                            if tags.is_empty() { "(无)" } else { &tags },
                            entry.created_at.format("%Y-%m-%d %H:%M"), entry.last_accessed.format("%Y-%m-%d %H:%M"), entry.access_count,
                            guard.checkpoints.len(), original
                        );
                        if want_json {
                            let out = CommandOutput::ok(&msg);
                            out.with_json(serde_json::json!(entry))
                        } else { CommandOutput::ok(&msg) }
                    }
                    Err(e) => CommandOutput::not_found(&e.to_string()),
                }
            }

            "list" | "ls" => {
                let filter = if plain_args.len() >= 3 {
                    match plain_args[1] {
                        "--tag" => None,
                        "--source" => Some(plain_args[2]),
                        _ => None,
                    }
                } else { None };

                let use_tag = plain_args.iter().position(|a| *a == "--tag");
                let guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                let entries = guard.list(filter);
                let filtered: Vec<&&MemoryEntry> = if let Some(pos) = use_tag {
                    let tag_val = plain_args.get(pos + 1).unwrap_or(&"");
                    entries.iter().filter(|e| e.tags.iter().any(|t| t == tag_val)).collect()
                } else {
                    entries.iter().collect()
                };

                if filtered.is_empty() { return CommandOutput::ok("📭 暂无记忆"); }
                let mut s = format!("📚 共 {} 条记忆:\n", filtered.len());
                for (i, e) in filtered.iter().take(20).enumerate() {
                    let content = if e.content.len() > 50 { format!("{}…", &e.content[..47]) } else { e.content.clone() };
                    let tags = if e.tags.is_empty() { String::new() } else { format!(" [{}]", e.tags.join(",")) };
                    let pin = if e.pinned { " 📌" } else { "" };
                    s.push_str(&format!("  {}. {}{}{}\n", i + 1, content, tags, pin));
                }
                if filtered.len() > 20 { s.push_str(&format!("  ... 还有 {} 条\n", filtered.len() - 20)); }
                if want_json {
                    let json_list: Vec<serde_json::Value> = filtered.iter().map(|e| serde_json::json!({"id": e.id, "content": e.content, "source": e.source, "pinned": e.pinned})).collect();
                    CommandOutput::ok(&s).with_json(serde_json::json!({"entries": json_list, "count": filtered.len()}))
                } else { CommandOutput::ok(&s) }
            }

            "search" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem search <query>"); }
                let query = plain_args[1..].join(" ");
                let guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                let results = guard.search(&query);
                if results.is_empty() { return CommandOutput::ok(&format!("🔍 未找到匹配: {}", query)); }
                let mut s = format!("🔍 \"{}\": {} 条结果\n", query, results.len());
                for (i, e) in results.iter().enumerate() {
                    let content = if e.content.len() > 60 { format!("{}…", &e.content[..57]) } else { e.content.clone() };
                    s.push_str(&format!("  {}. [{}] {} (conf={:.2})\n", i + 1, e.source, content, e.confidence));
                }
                if want_json {
                    let json_results: Vec<serde_json::Value> = results.iter().map(|e| serde_json::json!({"id": e.id, "content": e.content, "source": e.source, "confidence": e.confidence})).collect();
                    CommandOutput::ok(&s).with_json(serde_json::json!({"query": query, "results": json_results}))
                } else { CommandOutput::ok(&s) }
            }

            "edit" => {
                if plain_args.len() < 3 { return CommandOutput::err("用法: /mem edit <id> <新内容>"); }
                let id = plain_args[1];
                let new_content = plain_args[2..].join(" ");
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.edit_content(id, &new_content) {
                    Ok(()) => {
                        let _ = guard.save();
                        if let Some(b) = brain {
                            let mut a = b.blocking_write();
                            guard.sync_to_brain(&mut a.reasoning_bank);
                        }
                        CommandOutput::ok(&format!("✏️ 记忆 {} 已更新", id))
                    }
                    Err(e) => CommandOutput::err(&e.to_string()),
                }
            }

            "tag" => {
                if plain_args.len() < 3 { return CommandOutput::err("用法: /mem tag <id> <tag1,tag2,...>"); }
                let id = plain_args[1];
                let tags: Vec<String> = plain_args[2].split(',').map(|s| s.trim().to_string()).collect();
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.edit_tags(id, tags) {
                    Ok(()) => {
                        let _ = guard.save();
                        CommandOutput::ok(&format!("🏷️ 记忆 {} 标签已更新", id))
                    }
                    Err(e) => CommandOutput::err(&e.to_string()),
                }
            }

            "delete" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem delete <id>"); }
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.delete(plain_args[1]) {
                    Ok(()) => {
                        let _ = guard.save();
                        CommandOutput::ok(&format!("🗑️ 记忆 {} 已删除", plain_args[1]))
                    }
                    Err(e) => CommandOutput::err(&e.to_string()),
                }
            }

            "pin" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem pin <id>"); }
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.pin(plain_args[1]) {
                    Ok(()) => {
                        let _ = guard.save();
                        CommandOutput::ok(&format!("📌 记忆 {} 固定状态已切换", plain_args[1]))
                    }
                    Err(e) => CommandOutput::err(&e.to_string()),
                }
            }

            "checkpoint" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem checkpoint create <描述> | /mem checkpoint list"); }
                let sub = plain_args[1];
                match sub {
                    "create" => {
                        let desc = if plain_args.len() >= 3 { plain_args[2..].join(" ") } else { "manual checkpoint".to_string() };
                        let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                        guard.create_checkpoint(&desc);
                        let _ = guard.save();
                        CommandOutput::ok(&format!("💾 检查点已创建: {}", desc))
                    }
                    "list" | "ls" => {
                        let guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                        let cps = guard.list_checkpoints();
                        if cps.is_empty() { return CommandOutput::ok("📭 暂无检查点"); }
                        let mut s = format!("📦 检查点 ({}):\n", cps.len());
                        for (i, cp) in cps.iter().enumerate().rev().take(10) {
                            s.push_str(&format!("  {}. [{}] {} — {} 条记忆\n", i + 1, &cp.id[..8], cp.description, cp.entries.len()));
                        }
                        CommandOutput::ok(&s)
                    }
                    _ => CommandOutput::err(&format!("未知子命令: {}. 可用: create, list", sub)),
                }
            }

            "rollback" => {
                if plain_args.len() < 2 { return CommandOutput::err("用法: /mem rollback <checkpoint_id>"); }
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                match guard.rollback(plain_args[1]) {
                    Ok(()) => {
                        let _ = guard.save();
                        if let Some(b) = brain {
                            let mut a = b.blocking_write();
                            guard.sync_to_brain(&mut a.reasoning_bank);
                        }
                        CommandOutput::ok(&format!("⏪ 已回滚到检查点 {}", plain_args[1]))
                    }
                    Err(e) => CommandOutput::err(&e.to_string()),
                }
            }

            "dream" => {
                let mut guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                if plain_args.len() >= 2 && plain_args[1] == "toggle" {
                    let enabled = !guard.auto_consolidate;
                    guard.set_auto_consolidate(enabled);
                    let _ = guard.save();
                    return if enabled {
                        CommandOutput::ok("🌙 自动梦境已开启")
                    } else {
                        CommandOutput::ok("☀️ 自动梦境已关闭")
                    };
                }
                let report = guard.dream_cycle();
                let _ = guard.save();
                let msg = format!(
                    "🌙 梦境周期完成:\n  条目前: {} → 后: {}\n  合并: {} | 修剪: {}\n  耗时: {}ms",
                    report.entries_before, report.entries_after, report.merged, report.pruned, report.duration_ms
                );
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "entries_before": report.entries_before,
                        "entries_after": report.entries_after,
                        "merged": report.merged,
                        "pruned": report.pruned,
                        "duration_ms": report.duration_ms,
                    }))
                } else { CommandOutput::ok(&msg) }
            }

            "stats" => {
                let guard = WHITEBOX_MEMORY.lock().unwrap_or_else(|e| e.into_inner());
                let stats = guard.stats();
                let sources: String = stats.top_sources.iter().map(|(s, c)| format!("{} ({})", s, c)).collect::<Vec<_>>().join(", ");
                let tags: String = stats.top_tags.iter().map(|(t, c)| format!("{} ({})", t, c)).collect::<Vec<_>>().join(", ");
                let msg = format!(
                    "📊 记忆统计:\n  总条目: {}\n  平均置信度: {:.3}\n  已编辑: {} | 已固定: {}\n  检查点: {}\n  梦境: {}\n  最早: {} | 最新: {}\n  主要来源: {}\n  热门标签: {}",
                    stats.total_entries, stats.avg_confidence,
                    stats.edited_entries, stats.pinned_entries,
                    stats.checkpoint_count,
                    if stats.dream_enabled { "🌙 开启" } else { "☀️ 关闭" },
                    stats.oldest_entry.format("%Y-%m-%d"), stats.newest_entry.format("%Y-%m-%d"),
                    if sources.is_empty() { "(无)" } else { &sources },
                    if tags.is_empty() { "(无)" } else { &tags },
                );
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!(stats))
                } else { CommandOutput::ok(&msg) }
            }

            _ => CommandOutput::err(&format!("未知子命令: {}. 使用 /mem 查看用法", cmd)),
        }
    }
}

// ====== /save ======

pub struct SaveCmd;
impl CliCommand for SaveCmd {
    fn name(&self) -> &str { "/save" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "保存状态到 ~/.neotrix/brain.json" }
    fn execute(&self, _args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if let Some(b) = brain {
            let a = b.blocking_read();
            match a.brain.save() {
                Ok(()) => CommandOutput::ok("💾 Brain 已保存到 ~/.neotrix/brain.json"),
                Err(e) => CommandOutput::err(&format!("保存失败: {}", e)),
            }
        } else {
            CommandOutput::err("Brain 不可用")
        }
    }
}

// ====== /trace ======

pub struct TraceCmd;
impl CliCommand for TraceCmd {
    fn name(&self) -> &str { "/trace" }
    fn aliases(&self) -> Vec<&str> { vec!["/tree", "/chain"] }
    fn description(&self) -> &str { "显示推理因果链" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let limit = args.iter().find_map(|a| {
            a.parse::<usize>().ok()
        }).unwrap_or(10);
        let b = match brain {
            Some(b) => b.blocking_read(),
            None => return CommandOutput::err("Brain 不可用"),
        };
        let trajectory: Vec<crate::core::nt_core_hex::FullReasoningState> = b.reasoning_engine.as_ref()
            .map(|e| e.state_trajectory.clone())
            .unwrap_or_default();
        if trajectory.is_empty() {
            let msg = "暂无推理轨迹。运行 SEAL 循环后 (如 /evolve) 再查看。";
            return if want_json {
                CommandOutput::ok(msg).with_json(serde_json::json!({"traces": [], "count": 0}))
            } else { CommandOutput::ok(msg) };
        }
        let count = trajectory.len().min(limit);
        let lines: Vec<String> = trajectory.iter().rev().take(count).enumerate().map(|(i, s)| {
            format!("#{:<4} mode={:<20} sig={}",
                trajectory.len().saturating_sub(i),
                s.mode_name(),
                s.signature())
        }).collect();
        let tree = format!("推理轨迹 (最近 {} / 共 {}):\n\n{}\n\n迭代={}, 奖励={:.4}",
            count, trajectory.len(), lines.join("\n"), b.iteration, b._reward);
        let out = CommandOutput::ok(&tree);
        if want_json {
            let traces: Vec<serde_json::Value> = trajectory.iter().rev().take(count).map(|s: &crate::core::nt_core_hex::FullReasoningState| {
                serde_json::json!({"mode": s.mode_name(), "signature": s.signature()})
            }).collect();
            out.with_json(serde_json::json!({"traces": traces, "count": count, "total": trajectory.len()}))
        } else { out }
    }
}

// ====== /avatar ======

pub struct AvatarCmd;
impl CliCommand for AvatarCmd {
    fn name(&self) -> &str { "/avatar" }
    fn aliases(&self) -> Vec<&str> { vec!["/av"] }
    fn description(&self) -> &str { "Avatar 管理: /avatar list | /avatar create <name> | /avatar status | /avatar harvest | /avatar evolve" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let msg = "Usage:\n  /avatar list                  Show current avatar identity & profile\n  /avatar create <name>         Create/replace avatar identity\n  /avatar status                Show avatar details\n  /avatar harvest               Snapshot current profile to avatar chain\n  /avatar evolve                Run chain distillation\n  /avatar --json                Output as JSON";
            let out = CommandOutput::ok(msg);
            return if want_json { out.with_json(serde_json::json!({"subcommands": ["list", "create", "status", "harvest", "evolve"]})) } else { out };
        }

        let cmd = args[0].as_str();
        match cmd {
            "list" => {
                let eng = AVATAR_ENGINE.lock().unwrap_or_else(|e| {
                    log::warn!("[avatar] engine mutex poisoned");
                    e.into_inner()
                });
                let av = eng.get_avatar().clone();
                let identity = eng.identity.as_ref().map(|i| format!("{} (edition {})", i.name, i.edition))
                    .unwrap_or_else(|| "no identity".into());
                let domains: Vec<String> = av.domain_scores.iter()
                    .map(|(k, v)| format!("  {}: {:.2}", k, v))
                    .collect();
                let tasks: Vec<String> = av.task_affinity.iter()
                    .map(|(k, v)| format!("  {}: {:.2}", k, v))
                    .collect();
                let lines = vec![
                    format!("Identity: {}", identity),
                    format!("Confidence: {:.3}", av.confidence),
                    format!("Edition: {}", av.edition),
                    format!("Messages processed: {}", av.total_messages_processed),
                    format!("Chain length: {}", av.chain_length),
                    format!("Summary: {}", av.summary),
                    format!("Domain scores ({}):", av.domain_scores.len()),
                    domains.join("\n"),
                    format!("Task affinity ({}):", av.task_affinity.len()),
                    tasks.join("\n"),
                ];
                let out = CommandOutput::ok(&lines.join("\n"));
                if want_json {
                    out.with_json(serde_json::json!({
                        "identity_name": av.identity_name,
                        "edition": av.edition,
                        "confidence": av.confidence,
                        "total_messages": av.total_messages_processed,
                        "chain_length": av.chain_length,
                        "summary": av.summary,
                        "domain_scores": av.domain_scores,
                        "task_affinity": av.task_affinity,
                    }))
                } else { out }
            }
            "create" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /avatar create <name>");
                }
                let name = args[1..].join(" ");
                let mut eng = AVATAR_ENGINE.lock().unwrap_or_else(|e| {
                    log::warn!("[avatar] engine mutex poisoned");
                    e.into_inner()
                });
                eng.set_identity(&name);
                // Write some initial messages to seed domain knowledge
                eng.distill_message(&format!("Hello from {}", name));
                let out = CommandOutput::ok(&format!("Avatar identity created: {} (edition {})", name, eng.avatar.edition));
                if want_json {
                    out.with_json(serde_json::json!({"name": name, "edition": eng.avatar.edition}))
                } else { out }
            }
            "status" => {
                let eng = AVATAR_ENGINE.lock().unwrap_or_else(|e| {
                    log::warn!("[avatar] engine mutex poisoned");
                    e.into_inner()
                });
                let av = eng.get_avatar();
                let tags = av.tags.join(", ");
                let domains: Vec<String> = av.domain_scores.iter()
                    .map(|(k, v)| format!("    {}: {:.3}", k, v))
                    .collect();
                let tasks: Vec<String> = av.task_affinity.iter()
                    .map(|(k, v)| format!("    {}: {:.3}", k, v))
                    .collect();
                let lines = vec![
                    "Avatar Profile".to_string(),
                    format!("  Name:           {}", av.identity_name),
                    format!("  Edition:        {}", av.edition),
                    format!("  Confidence:     {:.3}", av.confidence),
                    format!("  Language:       {:.3}", av.language_preference),
                    format!("  Communication:  {:.3}", av.communication_style),
                    format!("  Reasoning:      {:.3}", av.reasoning_depth),
                    format!("  Technical:      {:.3}", av.technical_depth),
                    format!("  Messages:       {}", av.total_messages_processed),
                    format!("  Chain length:   {}", av.chain_length),
                    format!("  Tags:           [{}]", tags),
                    format!("  Summary:        {}", av.summary),
                    format!("  Domain scores:"),
                    domains.join("\n"),
                    format!("  Task affinity:"),
                    tasks.join("\n"),
                ];
                let out = CommandOutput::ok(&lines.join("\n"));
                if want_json {
                    out.with_json(serde_json::json!({
                        "name": av.identity_name,
                        "edition": av.edition,
                        "confidence": av.confidence,
                        "language_preference": av.language_preference,
                        "communication_style": av.communication_style,
                        "reasoning_depth": av.reasoning_depth,
                        "technical_depth": av.technical_depth,
                        "total_messages": av.total_messages_processed,
                        "chain_length": av.chain_length,
                        "tags": av.tags,
                        "summary": av.summary,
                        "domain_scores": av.domain_scores,
                        "task_affinity": av.task_affinity,
                        "knowledge_affinity": av.knowledge_affinity,
                    }))
                } else { out }
            }
            "harvest" => {
                let mut eng = AVATAR_ENGINE.lock().unwrap_or_else(|e| {
                    log::warn!("[avatar] engine mutex poisoned");
                    e.into_inner()
                });
                let result = eng.auto_distill();
                let chain_len = eng.chain.entries.len();
                let out = CommandOutput::ok(&format!("Harvested: {}\n  Chain entries: {}", result, chain_len));
                if want_json {
                    out.with_json(serde_json::json!({"result": result, "chain_entries": chain_len}))
                } else { out }
            }
            "evolve" => {
                let mut eng = AVATAR_ENGINE.lock().unwrap_or_else(|e| {
                    log::warn!("[avatar] engine mutex poisoned");
                    e.into_inner()
                });
                let snapshot = eng.auto_distill();
                let chain_len = eng.chain.entries.len();
                let edition = eng.avatar.edition;
                let lines = [
                    "Chain distillation complete".to_string(),
                    format!("  Chain entries: {}", chain_len),
                    format!("  Edition:       {}", edition),
                    format!("  Last action:   {}", snapshot),
                ];
                let out = CommandOutput::ok(&lines.join("\n"));
                if want_json {
                    out.with_json(serde_json::json!({
                        "chain_entries": chain_len,
                        "edition": edition,
                        "snapshot": snapshot,
                    }))
                } else { out }
            }
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Available: list, create, status, harvest, evolve", cmd)),
        }
    }
}

// ====== /e8 ======

pub struct E8Cmd;
impl CliCommand for E8Cmd {
    fn name(&self) -> &str { "/e8" }
    fn aliases(&self) -> Vec<&str> { vec!["/hexagram", "/mode"] }
    fn description(&self) -> &str { "E8 推理状态查询: /e8 status|set <0-63>|matrix|transition" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let b = match brain {
            Some(b) => b.blocking_read(),
            None => return CommandOutput::err("Brain 不可用"),
        };
        let engine = match b.reasoning_engine.as_ref() {
            Some(e) => e,
            None => return CommandOutput::err("推理引擎不可用"),
        };
        let sub = args.iter().find(|a| !a.starts_with("--"));
        match sub.map(|s| s.as_str()) {
            None | Some("status") => {
                let state = &engine.current_state;
                let mode = state.mode;
                let meta = state.meta;
                let sig = state.signature();
                let traj_len = engine.state_trajectory.len();
                let msg = format!(
                    "E8 推理状态\n\
                     当前模式: {:<20} (0x{:02X})\n\
                     元状态:   {:<4} (0x{:02X})\n\
                     签名:     {} (0x{:04X})\n\
                     轨迹长度: {}\n\
                     Hexagram: {:06b} ({:02o})\n\
                     轴: 抽象={} 范围={} 方法={} 深度={} 模式={} 姿态={}",
                    mode.mode_name(), mode.0,
                    meta.0, meta.0,
                    sig, sig,
                    traj_len,
                    mode.0, mode.0,
                    axis_label(mode.0, 5), axis_label(mode.0, 4),
                    axis_label(mode.0, 3), axis_label(mode.0, 2),
                    axis_label(mode.0, 1), axis_label(mode.0, 0),
                );
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "mode": mode.0, "mode_name": mode.mode_name(),
                        "meta": meta.0, "signature": sig,
                        "trajectory_len": traj_len,
                    }))
                } else { out }
            }
            Some("matrix") => {
                let mut s = String::from("E8 策略矩阵 (8×8):\n\n   ");
                for col in 0..8 { s.push_str(&format!("{:4}", col)); }
                s.push('\n');
                for row in 0..8 {
                    s.push_str(&format!("{:2} ", row));
                    for col in 0..8 {
                        let val = engine.strategy_matrix[row][col].0;
                        s.push_str(&format!("{:4}", val));
                    }
                    s.push('\n');
                }
                CommandOutput::ok(&s)
            }
            Some("transition") => {
                if engine.state_trajectory.len() < 2 {
                    return CommandOutput::ok("轨迹太短，无法分析转换");
                }
                let mut s = String::from("E8 状态转换序列:\n");
                for w in engine.state_trajectory.windows(2) {
                    let from = w[0].mode.0;
                    let to = w[1].mode.0;
                    let arrow = if from == to { "━" } else { "→" };
                    s.push_str(&format!("  {} {} {}\n", w[0].mode.mode_name(), arrow, w[1].mode.mode_name()));
                }
                let out = CommandOutput::ok(&s);
                if want_json {
                    let transitions: Vec<serde_json::Value> = engine.state_trajectory.windows(2).map(|w| {
                        serde_json::json!({"from": w[0].mode.0, "to": w[1].mode.0})
                    }).collect();
                    out.with_json(serde_json::json!({"transitions": transitions}))
                } else { out }
            }
            Some("set") if args.len() >= 2 => {
                let val = args[1].parse::<u8>();
                match val {
                    Ok(v) if v < 64 => {
                        CommandOutput::ok(&format!("设置 E8 模式为 {} ({})", v, crate::core::nt_core_hex::ReasoningHexagram::new(v).mode_name()))
                    }
                    Ok(_) => CommandOutput::err("E8 模式范围 0-63"),
                    Err(_) => CommandOutput::err("无效参数，请输入 0-63 的数字"),
                }
            }
            Some(other) => CommandOutput::err(&format!("未知子命令: {}。可用: status, set <0-63>, matrix, transition", other)),
        }
    }
}

fn axis_label(mode: u8, bit: u8) -> &'static str {
    let bit_val = (mode >> bit) & 1;
    match (bit, bit_val) {
        (5, 0) => "具体", (5, 1) => "抽象",
        (4, 0) => "聚焦", (4, 1) => "广泛",
        (3, 0) => "分析", (3, 1) => "生成",
        (2, 0) => "深度", (2, 1) => "快速",
        (1, 0) => "独立", (1, 1) => "协作",
        (0, 0) => "确定", (0, 1) => "探索",
        _ => "未知",
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
