//! Consciousness 命令 — 意识状态查询与手动触发
//!
//! /consciousness tick       手动触发意识循环 (经持久化意识核心单例, 写回 KB)
//! /consciousness status     显示当前 phi、coherence、fog、MARS (转调 nt_core_consciousness_core)
//! /consciousness tree       显示 ConsciousnessTree 状态 (cycle, branches, fruits)
//!
//! 统一通道: 与 `entry::run_consciousness_core` 同源 (nt_core_consciousness_core),
//! R-P42 反对平行路径 — 不再读 brain 旧字段 (不暴露真实 phi/fog)。

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
        "意识状态查询:\n  /consciousness tick       手动触发意识 tick (写回 KB)\n  /consciousness status     显示 phi、coherence、fog、MARS\n  /consciousness tree       显示 ConsciousnessTree 状态 (branches/fruits)"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let sub = args.iter().find(|a| *a != "--json").map(|s| s.as_str()).unwrap_or("status");
        let cycles = args
            .iter()
            .find_map(|a| a.strip_prefix("--cycles="))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1);

        match sub {
            "tick" => {
                let snap = crate::core::nt_core_consciousness_core::tick(cycles);
                let msg = format!(
                    "🌳 Consciousness Tick\n\
                     ────────────────────────\n\
                     Cycle: {}\n\
                     Φ (IIT): {:.4}\n\
                     Coherence: {:.4}\n\
                     Resonance: {}\n\
                     Fruits: {}\n\
                     Weighted Fog: {:.3}\n\
                     Governance: compliance={:.4} constitution={} fractal_depth={}",
                    snap.cycle, snap.phi, snap.coherence, snap.resonance_cycle,
                    snap.fruits.len(), snap.weighted_fog_sum,
                    snap.governance_compliance,
                    snap.governance_constitution_count,
                    snap.governance_fractal_depth
                );

                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "op": "tick",
                        "cycles_run": cycles,
                        "cycle": snap.cycle,
                        "phi": snap.phi,
                        "coherence": snap.coherence,
                        "resonance_cycle": snap.resonance_cycle,
                        "fruits": snap.fruits.len(),
                        "weighted_fog_sum": snap.weighted_fog_sum,
                        "governance_compliance": snap.governance_compliance,
                        "governance_constitution_count": snap.governance_constitution_count,
                        "governance_fractal_depth": snap.governance_fractal_depth,
                        "attention_source": snap.attention_source,
                    }))
                } else {
                    out
                }
            }

            "status" => {
                let snap = crate::core::nt_core_consciousness_core::status();
                let fog_live = crate::core::nt_core_consciousness_core::current_fog_sum();
                let msg = format!(
                    "🌳 Consciousness Status\n\
                     ────────────────────────\n\
                     Cycle: {}\n\
                     Φ (IIT): {:.4}\n\
                     Coherence: {:.4}\n\
                     Resonance: {}\n\
                     MARS: S1={} S2={} bridge={}\n\
                     Fruits: {}\n\
                     Weighted Fog: {:.3} (live {:.3})\n\
                     Governance: compliance={:.4} constitution={} fractal_depth={}",
                    snap.cycle, snap.phi, snap.coherence, snap.resonance_cycle,
                    snap.mars_system1_activations, snap.mars_system2_iterations,
                    snap.mars_bridge_hits, snap.fruits.len(), snap.weighted_fog_sum, fog_live,
                    snap.governance_compliance,
                    snap.governance_constitution_count,
                    snap.governance_fractal_depth
                );

                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "op": "status",
                        "cycle": snap.cycle,
                        "phi": snap.phi,
                        "coherence": snap.coherence,
                        "resonance_cycle": snap.resonance_cycle,
                        "mars_system1_activations": snap.mars_system1_activations,
                        "mars_system2_iterations": snap.mars_system2_iterations,
                        "mars_bridge_hits": snap.mars_bridge_hits,
                        "fruits": snap.fruits.len(),
                        "weighted_fog_sum": snap.weighted_fog_sum,
                        "weighted_fog_sum_live": fog_live,
                        "governance_compliance": snap.governance_compliance,
                        "governance_constitution_count": snap.governance_constitution_count,
                        "governance_fractal_depth": snap.governance_fractal_depth,
                        "attention_source": snap.attention_source,
                    }))
                } else {
                    out
                }
            }

            "tree" => {
                let snap = crate::core::nt_core_consciousness_core::status();
                let branches = crate::core::nt_core_consciousness_core::branches();
                let mut body = format!(
                    "🌳 ConsciousnessTree\n\
                     ────────────────────────\n\
                     Cycle: {}\n\
                     Φ (IIT): {:.4}\n\
                     Coherence: {:.4}\n\
                     Branches ({}):\n",
                    snap.cycle, snap.phi, snap.coherence, branches.len()
                );
                for b in &branches {
                    body.push_str(&format!(
                        "  · {}  health={} constellation={} tier={} fog={}\n",
                        b.get("label").cloned().unwrap_or_default(),
                        b.get("health").cloned().unwrap_or_default(),
                        b.get("constellation").cloned().unwrap_or_default(),
                        b.get("node_tier").cloned().unwrap_or_default(),
                        b.get("fog").cloned().unwrap_or_default()
                    ));
                }
                let out = CommandOutput::ok(body.trim_end());
                if want_json {
                    out.with_json(serde_json::json!({
                        "cycle": snap.cycle,
                        "phi": snap.phi,
                        "coherence": snap.coherence,
                        "branches": branches,
                    }))
                } else {
                    out
                }
            }

            _ => {
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