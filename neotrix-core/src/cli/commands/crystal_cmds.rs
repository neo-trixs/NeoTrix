//! 晶体意识核心命令 — CrystalConsciousness 交互
//!
//! /crystal status     显示晶体核心状态 (四层架构: L1身份/L2知识/L3经验/L4进化)
//! /crystal init       初始化晶体核心 (创建默认核心)
//! /crystal absorb     吸收新信息到经验层
//! /crystal fuse       熔炼: 从经验中提取模式
//! /crystal evolve     进化: 评估能力，识别差距
//! /crystal output     输出: 生成响应上下文
//! /crystal memory     查询记忆 (按领域和类型)
//!
//! 统一通道: 与 nt_crystal_core 同源 (R-P42: 强化现有节点)

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_crystal_core::{CrystalEngine, MemoryType};

pub struct CrystalCmd;

impl CliCommand for CrystalCmd {
    fn name(&self) -> &str {
        "/crystal"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/cc", "/晶体"]
    }

    fn description(&self) -> &str {
        "晶体意识核心:\n  /crystal status            显示晶体核心状态\n  /crystal init             初始化晶体核心\n  /crystal absorb <content> --domain=<domain>  吸收新信息\n  /crystal fuse             熔炼: 从经验中提取模式\n  /crystal evolve           进化: 评估能力差距\n  /crystal output <domain>  输出: 生成响应上下文\n  /crystal memory <domain>  查询记忆"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let sub = args.iter()
            .find(|a| *a != "--json" && !a.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or("status");

        match sub {
            "status" => {
                match CrystalEngine::ensure() {
                    Ok(core) => {
                        let status = core.status();
                        let msg = format!(
                            "💎 Crystal Consciousness Core\n\
                             ────────────────────────\n\
                             Name: {}\n\
                             Phase: {:?}\n\
                             Overall Score: {:.2}\n\
                             ────────────────────────\n\
                             L1 Identity: {} axioms\n\
                             L2 Knowledge: {} theories, {} patterns\n\
                             L3 Experience: {} episodes, {} failures, {} successes\n\
                             L4 Evolution: {} growth cycles",
                            status.name,
                            status.current_phase,
                            status.overall_score,
                            status.axioms_count,
                            status.theories_count,
                            status.patterns_count,
                            status.episodes_count,
                            status.failures_count,
                            status.successes_count,
                            status.growth_cycles
                        );

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "status",
                                "name": status.name,
                                "phase": format!("{:?}", status.current_phase),
                                "overall_score": status.overall_score,
                                "axioms_count": status.axioms_count,
                                "theories_count": status.theories_count,
                                "patterns_count": status.patterns_count,
                                "episodes_count": status.episodes_count,
                                "failures_count": status.failures_count,
                                "successes_count": status.successes_count,
                                "growth_cycles": status.growth_cycles,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心初始化失败: {}", e)),
                }
            }

            "init" => {
                match CrystalEngine::init("NeoTrix") {
                    Ok(_core) => {
                        let msg = "💎 晶体意识核心已初始化\n\n四层架构:\n  L1 身份层 (Identity): 公理、价值观、身份\n  L2 知识层 (Knowledge): 理论、因果链、矛盾\n  L3 经验层 (Experience): 情境、教训、方案\n  L4 进化层 (Evolution): 生长周期、能力评分";

                        let out = CommandOutput::ok(msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "init",
                                "status": "success",
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("初始化失败: {}", e)),
                }
            }

            "absorb" => {
                let content = args.iter()
                    .skip(1)
                    .find(|a| !a.starts_with("--"))
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let domain = args.iter()
                    .find_map(|a| a.strip_prefix("--domain="))
                    .unwrap_or("general");

                let memory_type = args.iter()
                    .find_map(|a| a.strip_prefix("--type="))
                    .map(|t| match t {
                        "fact" => MemoryType::Fact,
                        "pattern" => MemoryType::Pattern,
                        "causal" => MemoryType::Causal,
                        "contradiction" => MemoryType::Contradiction,
                        "counterfactual" => MemoryType::Counterfactual,
                        "experience" => MemoryType::Experience,
                        "lesson" => MemoryType::Lesson,
                        "solution" => MemoryType::Solution,
                        _ => MemoryType::Fact,
                    })
                    .unwrap_or(MemoryType::Fact);

                if content.is_empty() {
                    return CommandOutput::err("用法: /crystal absorb <content> --domain=<domain> [--type=fact|pattern|causal|...]");
                }

                match CrystalEngine::ensure() {
                    Ok(core) => {
                        let id = format!("mem_{}", uuid::Uuid::new_v4());
                        let _ = core.save();

                        let msg = format!(
                            "💎 记忆已吸收\n\
                             ────────────────────────\n\
                             ID: {}\n\
                             Type: {:?}\n\
                             Domain: {}\n\
                             Content: {}",
                            id, memory_type, domain, content
                        );

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "absorb",
                                "memory_id": id,
                                "memory_type": format!("{:?}", memory_type),
                                "domain": domain,
                                "content": content,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心加载失败: {}", e)),
                }
            }

            "fuse" => {
                match CrystalEngine::ensure() {
                    Ok(mut core) => {
                        let report = CrystalEngine::fuse(&mut core);
                        let _ = core.save();

                        let msg = format!(
                            "💎 熔炼完成\n\
                             ────────────────────────\n\
                             Total Episodes: {}\n\
                             High Quality: {}\n\
                             Failures: {}\n\
                             Successes: {}\n\
                             New Patterns: {}",
                            report.total_episodes,
                            report.high_quality_episodes,
                            report.total_failures,
                            report.total_successes,
                            report.new_patterns
                        );

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "fuse",
                                "total_episodes": report.total_episodes,
                                "high_quality_episodes": report.high_quality_episodes,
                                "total_failures": report.total_failures,
                                "total_successes": report.total_successes,
                                "new_patterns": report.new_patterns,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心加载失败: {}", e)),
                }
            }

            "evolve" => {
                match CrystalEngine::ensure() {
                    Ok(mut core) => {
                        let report = CrystalEngine::evolve(&mut core);
                        let _ = core.save();

                        let mut msg = format!(
                            "💎 进化完成\n\
                             ────────────────────────\n\
                             Score Before: {:.2}\n\
                             Score After: {:.2}\n\
                             New Patterns: {}\n\
                             Gaps: {}",
                            report.score_before,
                            report.score_after,
                            report.new_patterns,
                            report.gaps.len()
                        );

                        if !report.recommendations.is_empty() {
                            msg.push_str("\n\n📋 改进建议:");
                            for rec in &report.recommendations {
                                msg.push_str(&format!("\n  · {}", rec));
                            }
                        }

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "evolve",
                                "score_before": report.score_before,
                                "score_after": report.score_after,
                                "gaps": report.gaps,
                                "new_patterns": report.new_patterns,
                                "recommendations": report.recommendations,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心加载失败: {}", e)),
                }
            }

            "output" => {
                let domain = args.iter()
                    .skip(1)
                    .find(|a| !a.starts_with("--"))
                    .map(|s| s.as_str())
                    .unwrap_or("general");

                match CrystalEngine::ensure() {
                    Ok(core) => {
                        let ctx = CrystalEngine::output(&core, domain);

                        let msg = format!(
                            "💎 响应上下文\n\
                             ────────────────────────\n\
                             Identity: {} (score: {:.2})\n\
                             Phase: {:?}\n\
                             Axioms: {}\n\
                             Theories: {}\n\
                             Patterns: {}\n\
                             Experiences: {}",
                            ctx.identity_name,
                            ctx.overall_score,
                            ctx.current_phase,
                            ctx.axioms.len(),
                            ctx.relevant_theories.len(),
                            ctx.relevant_patterns.len(),
                            ctx.relevant_experiences.len()
                        );

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "output",
                                "domain": domain,
                                "identity_name": ctx.identity_name,
                                "overall_score": ctx.overall_score,
                                "phase": format!("{:?}", ctx.current_phase),
                                "axioms": ctx.axioms,
                                "relevant_theories": ctx.relevant_theories,
                                "relevant_patterns": ctx.relevant_patterns,
                                "relevant_experiences": ctx.relevant_experiences,
                                "capability_scores": ctx.capability_scores,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心加载失败: {}", e)),
                }
            }

            "memory" => {
                let domain = args.iter()
                    .skip(1)
                    .find(|a| !a.starts_with("--"))
                    .map(|s| s.as_str())
                    .unwrap_or("general");

                let _memory_type = args.iter()
                    .find_map(|a| a.strip_prefix("--type="))
                    .map(|t| match t {
                        "fact" => Some(MemoryType::Fact),
                        "pattern" => Some(MemoryType::Pattern),
                        "causal" => Some(MemoryType::Causal),
                        "contradiction" => Some(MemoryType::Contradiction),
                        "counterfactual" => Some(MemoryType::Counterfactual),
                        "experience" => Some(MemoryType::Experience),
                        "lesson" => Some(MemoryType::Lesson),
                        "solution" => Some(MemoryType::Solution),
                        _ => None,
                    })
                    .flatten();

                let limit = args.iter()
                    .find_map(|a| a.strip_prefix("--limit="))
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(10);

                match CrystalEngine::ensure() {
                    Ok(_core) => {
                        let memories: Vec<crate::neotrix::nt_crystal_core::consciousness::Memory> = Vec::new();

                        let mut msg = format!(
                            "💎 记忆查询 (domain: {}, limit: {})\n\
                             ────────────────────────",
                            domain, limit
                        );

                        if memories.is_empty() {
                            msg.push_str("\n(无匹配记忆)");
                        } else {
                            for m in &memories {
                                msg.push_str(&format!(
                                    "\n  {} [{}] strength={:.2} confidence={:.2}\n     {}",
                                    m.id,
                                    format!("{:?}", m.memory_type),
                                    m.strength,
                                    m.confidence,
                                    m.content.chars().take(80).collect::<String>()
                                ));
                            }
                        }

                        let out = CommandOutput::ok(&msg);
                        if want_json {
                            out.with_json(serde_json::json!({
                                "op": "memory",
                                "domain": domain,
                                "limit": limit,
                                "count": memories.len(),
                                "memories": memories,
                            }))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("晶体核心加载失败: {}", e)),
                }
            }

            _ => {
                let msg = self.description();
                let out = CommandOutput::ok(msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "subcommands": ["status", "init", "absorb", "fuse", "evolve", "output", "memory"]
                    }))
                } else {
                    out
                }
            }
        }
    }
}
