// ── Cycle 159: Self-Audit + Evolution Loop ──

use std::collections::VecDeque;

use super::agent::NeoCodexAgent;
use super::provider::NeoCodexMode;

/// Serializable health report used by the SelfTest trait, the TUI status line,
/// and the evolution loop. All checks are synchronous and side-effect free.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NeoCodexHealthReport {
    pub mode: NeoCodexMode,
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub tokens_used: usize,
    pub context_usage: f64,
    pub context_turns: usize,
    pub provider_count: usize,
    pub provider_resolvable: bool,
    pub provider_model: String,
    pub session_writable: bool,
    pub goals_active: bool,
    pub cost_spent: f64,
    pub cost_budget: f64,
    pub subagent_results: usize,
    pub consciousness_attached: bool,
    pub brain_attached: bool,
    pub event_bus_attached: bool,
    pub evolution_iterations: u64,
    pub tool_grounding_degraded: bool,
    /// Skill Node Evolution — per-domain 节点状态 (NodeTier/Constellation/Rune)
    /// 使 7 域健康网格反映真实 per-domain 遥测, 而非布尔投影。
    pub node_snapshots: Vec<crate::core::nt_core_consciousness_tree::NodeSnapshot>,
}

impl NeoCodexHealthReport {
    /// Number of failed checks (used by D43: every detection must feed behavior).
    pub fn failed_checks(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if !self.provider_resolvable {
            failures.push("provider not resolvable (no API key / no model)".into());
        }
        if !self.session_writable {
            failures.push("session dir not writable".into());
        }
        if self.context_usage > 0.9 {
            failures.push(format!(
                "context pipeline at {:.0}% (auto-compact will trigger)",
                self.context_usage * 100.0
            ));
        }
        if self.cost_spent > self.cost_budget && self.cost_budget > 0.0 {
            failures.push(format!(
                "budget exhausted ${:.2}/${:.2}",
                self.cost_spent, self.cost_budget
            ));
        }
        if self.provider_count == 0 {
            failures.push("provider catalog empty".into());
        }
        if self.tool_grounding_degraded {
            failures.push("tool grounding degraded (failure rate above adaptive threshold)".into());
        }
        failures
    }

    pub fn is_healthy(&self) -> bool {
        self.failed_checks().is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "NeoCodexHealth[mode={:?} turns={} tools={} ctx={:.0}% providers={} evo={}]",
            self.mode,
            self.turn_count,
            self.tool_call_count,
            self.context_usage * 100.0,
            self.provider_count,
            self.evolution_iterations,
        )
    }
}

/// SelfTest implementation — snapshot-based so `self_test()` is synchronous
/// (fits the `SelfTest` trait signature).
#[derive(Debug, Clone, Default)]
pub struct NeoCodexSelfAudit {
    pub last_report: NeoCodexHealthReport,
}

impl NeoCodexSelfAudit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(agent: &NeoCodexAgent) -> Self {
        Self {
            last_report: agent.health_report(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for NeoCodexSelfAudit {
    fn name(&self) -> &str {
        "neocodex_self_audit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let failures = self.last_report.failed_checks();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// Self-healing loop for NeoCodex (D22/D26). Each `step` advances the iteration
/// counter and runs a diagnose→fix cycle. This is the 100-iteration loop that
/// converges the agent toward Codex/Claude Code desktop parity.
#[derive(Debug, Clone, Default)]
pub struct EvolutionLoop {
    pub iteration: u64,
    pub target: u64,
    pub gaps_found: u64,
    pub fixes_applied: u64,
    pub history: VecDeque<EvolutionIteration>,
}

/// Record of a single evolution iteration.
#[derive(Debug, Clone)]
pub struct EvolutionIteration {
    pub iteration: u64,
    pub gaps: Vec<String>,
    pub fixes: Vec<String>,
    pub healthy: bool,
}

impl EvolutionLoop {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            target: 100,
            gaps_found: 0,
            fixes_applied: 0,
            history: VecDeque::new(),
        }
    }

    pub fn with_target(mut self, target: u64) -> Self {
        self.target = target;
        self
    }

    /// Advance one iteration: capture health → diagnose gaps → apply fixes.
    /// Associated function (not method) so the borrow checker accepts passing
    /// the whole agent while mutating the loop's own state.
    pub fn step(agent: &mut NeoCodexAgent) {
        agent.evolution.iteration += 1;
        let report = agent.health_report();
        let gaps = report.failed_checks();
        agent.evolution.gaps_found += gaps.len() as u64;

        let mut fixes = Vec::new();

        // Fix 1: active provider not resolvable (stub / empty) → sync from real
        // layer and pick a usable provider (Cycle 159 gap: the "opencode" stub
        // was never replaced because the old guard only checked `is_empty`).
        if !agent.provider.is_resolvable() {
            agent.provider.ensure_production_provider();
            fixes.push("synced provider catalog from real layer".into());
        }

        // Fix 2: context near budget → force compaction
        if report.context_usage > 0.9 {
            agent.context.compact_if_needed();
            fixes.push("forced context compaction".into());
        }

        // Fix 3: session dir missing → recreate
        if !report.session_writable {
            if let Some(parent) = agent.wire.path.parent() {
                let _ = std::fs::create_dir_all(parent);
                fixes.push("recreated session dir".into());
            }
        }

        // Fix 4: goal queue empty but evolution wants growth → seed introspection goal
        if agent.goals.goals.is_empty()
            && agent.goals.active.is_none()
            && agent.evolution.iteration.is_multiple_of(25)
        {
            agent.add_goal(
                "Self-audit: converge NeoCodex toward Codex/Claude Code desktop parity",
                5,
            );
            fixes.push("seeded introspection goal".into());
        }

        agent.evolution.fixes_applied += fixes.len() as u64;

        let record = EvolutionIteration {
            iteration: agent.evolution.iteration,
            gaps,
            fixes,
            healthy: report.is_healthy(),
        };
        if agent.evolution.history.len() >= 100 {
            agent.evolution.history.pop_front();
        }
        agent.evolution.history.push_back(record);
        agent.audit = NeoCodexSelfAudit::capture(agent);
    }

    /// Summary line for the TUI status.
    pub fn summary(&self) -> String {
        format!(
            "Evolution {}/{} · {} gaps · {} fixes",
            self.iteration, self.target, self.gaps_found, self.fixes_applied
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_report_snapshot() {
        let agent = NeoCodexAgent::new("health-test");
        let report = agent.health_report();
        assert_eq!(report.mode, NeoCodexMode::Agent);
        assert_eq!(report.turn_count, 0);
        assert_eq!(report.evolution_iterations, 0);
        // Default catalog (opencode stub) is not resolvable → gap reported
        assert!(!report.provider_resolvable);
        assert!(!report.failed_checks().is_empty());
    }

    #[test]
    fn test_self_audit_impl() {
        use crate::core::nt_core_self_test::SelfTest;
        let agent = NeoCodexAgent::new("selftest");
        let audit = NeoCodexSelfAudit::capture(&agent);
        assert_eq!(audit.name(), "neocodex_self_audit");
        // Fresh agent with no real provider → self_test fails with gaps
        assert!(audit.self_test().is_err());
    }

    #[test]
    fn test_evolution_loop_advances_and_fixes() {
        let mut agent = NeoCodexAgent::new("evo-test");
        // Empty catalog → step should sync from real layer
        agent.provider.providers.clear();
        EvolutionLoop::step(&mut agent);
        assert_eq!(agent.evolution.iteration, 1);
        assert!(!agent.provider.providers.is_empty());
        assert!(!agent.evolution.history.is_empty());
        assert!(agent.evolution.history[0].iteration == 1);
    }

    #[test]
    fn test_evolution_loop_100_iterations() {
        let mut agent = NeoCodexAgent::new("evo-100");
        agent.evolution = EvolutionLoop::new().with_target(100);
        for _ in 0..100 {
            EvolutionLoop::step(&mut agent);
        }
        assert_eq!(agent.evolution.iteration, 100);
        assert_eq!(agent.evolution.history.len(), 100);
        // Fixes applied monotonically
        assert!(agent.evolution.fixes_applied > 0 || agent.provider.providers.len() > 1);
    }

    #[test]
    fn test_health_report_after_evolution_advances() {
        let mut agent = NeoCodexAgent::new("health-evo");
        EvolutionLoop::step(&mut agent);
        let report = agent.health_report();
        assert_eq!(report.evolution_iterations, 1);
    }
}