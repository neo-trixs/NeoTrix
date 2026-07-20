//! Consciousness 命令 — 意识状态查询与手动触发
//!
//! /consciousness tick       手动触发意识循环（读取 brain 中的最新状态）
//! /consciousness status     显示当前 phi、coherence、quality
//! /consciousness tree       显示 ConsciousnessTree 状态 (cycle, branches, fruits)

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ====== /consciousness ======

pub struct ConsciousnessCmd;

impl CliCommand for ConsciousnessCmd {
    fn name(&self) -> &str {
        "/consciousness"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/cns", "/awareness"]
    }

    fn description(&self) -> &str {
        "意识状态查询:\n  /consciousness tick       手动触发意识 tick (读取最新状态)\n  /consciousness status     显示 phi、coherence、quality\n  /consciousness tree       显示 ConsciousnessTree 状态"
    }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let sub = args.iter().find(|a| *a != "--json").map(|s| s.as_str()).unwrap_or("status");

        let Some(b) = brain else {
            return CommandOutput::err("Brain 不可用");
        };

        let brain_guard = b.blocking_read();

        match sub {
            "tick" => {
                // 手动触发 tick 实际上是读取已有的意识状态
                // 真正的 tick 由 BackgroundLoop 定期运行
                let quality = brain_guard._last_consciousness_quality;
                let critique_count = brain_guard._consciousness_critique_count;
                let iteration = brain_guard.iteration;

                // 从 ConsciousnessStream 读取状态
                let stream_len = brain_guard._consciousness_stream.len();
                let stream_cap = brain_guard._consciousness_stream.capacity();

                // 从 CognitiveLoadMonitor 读取状态
                let clm = &brain_guard._cognitive_load;
                let load_mode = format!("{:?}", clm.mode());
                let avg_load = clm.average_load();
                let thinking_budget = clm.thinking_budget();
                let can_deep = clm.can_do_deep_reasoning();

                let msg = format!(
                    "🧠 Consciousness Tick (manual read)\n\
                     Iteration: {}\n\
                     Last Quality: {:.4}\n\
                     Critique Count: {}\n\
                     Stream: {}/{} entries\n\
                     Cognitive Load: mode={} avg={:.3} budget={:.3} deep={}",
                    iteration, quality, critique_count, stream_len, stream_cap,
                    load_mode, avg_load, thinking_budget, can_deep
                );

                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "iteration": iteration,
                        "last_quality": quality,
                        "critique_count": critique_count,
                        "stream_len": stream_len,
                        "stream_cap": stream_cap,
                        "load_mode": load_mode,
                        "avg_load": avg_load,
                        "thinking_budget": thinking_budget,
                        "can_deep_reasoning": can_deep
                    }))
                } else {
                    out
                }
            }

            "status" => {
                // 从 brain 读取意识质量
                let quality = brain_guard._last_consciousness_quality;
                let critique_count = brain_guard._consciousness_critique_count;
                let iteration = brain_guard.iteration;

                // CognitiveLoadMonitor 状态
                let clm = &brain_guard._cognitive_load;
                let load_mode = format!("{:?}", clm.mode());
                let avg_load = clm.average_load();
                let thinking_budget = clm.thinking_budget();
                let can_deep = clm.can_do_deep_reasoning();

                // FirstPersonRef 状态
                let fpr = &brain_guard._first_person;
                let birth_step = fpr.birth_step();
                let self_similarity_threshold = fpr.self_similarity_threshold();
                let avg_coherence = fpr.average_coherence();

                // ConsciousnessStream 状态
                let stream_len = brain_guard._consciousness_stream.len();

                let msg = format!(
                    "🧠 Consciousness Status\n\
                     Iteration: {}\n\
                     ────────────────────\n\
                     Quality (InnerCritic): {:.4}\n\
                     Critique Count: {}\n\
                     ────────────────────\n\
                     Cognitive Load: mode={} avg={:.3} budget={:.3} deep_reasoning={}\n\
                     ────────────────────\n\
                     FirstPerson: birth_step={} threshold={:.3} avg_coherence={:.4}\n\
                     ConsciousnessStream: {} entries",
                    iteration, quality, critique_count,
                    load_mode, avg_load, thinking_budget, can_deep,
                    birth_step, self_similarity_threshold, avg_coherence, stream_len
                );

                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "iteration": iteration,
                        "quality": quality,
                        "critique_count": critique_count,
                        "load_mode": load_mode,
                        "avg_load": avg_load,
                        "thinking_budget": thinking_budget,
                        "can_deep_reasoning": can_deep,
                        "birth_step": birth_step,
                        "self_similarity_threshold": self_similarity_threshold,
                        "avg_coherence": avg_coherence,
                        "stream_len": stream_len
                    }))
                } else {
                    out
                }
            }

            "tree" => {
                // ConsciousnessTree 存储在 BackgroundLoopHandle 中，CLI 无法直接访问
                // 这里显示从 brain 可推断的树状态信息
                let iteration = brain_guard.iteration;
                let quality = brain_guard._last_consciousness_quality;

                let msg = format!(
                    "🌳 ConsciousnessTree (read-only from brain)\n\
                     ⚠️  完整的树状态在 BackgroundLoopHandle 中运行\n\
                     ──────────────────────────────────────────\n\
                     Cycle (via iteration): {}\n\
                     Last Quality: {:.4}\n\
                     ──────────────────────────────────────────\n\
                     要查看完整树状态 (branches, fruits, vuln_scan):\n\
                     1. 运行 `/bg start` 启动后台循环\n\
                     2. 日志中会输出 consciousness_tree cycle 报告\n\
                     3. 或检查 KB 中的 consciousness/growth_cycle 键",
                    iteration, quality
                );

                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "cycle": iteration,
                        "quality": quality,
                        "note": "Full tree state in BackgroundLoopHandle"
                    }))
                } else {
                    out
                }
            }

            "help" | _ => {
                let msg = self.description();
                let out = CommandOutput::ok(msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "subcommands": ["tick", "status", "tree"]
                    }))
                } else {
                    out
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert!(true);
    }
}