pub mod trace;
pub mod skill;
pub mod harness;
pub mod co_evolution;

pub use trace::TraceRecorder;
pub use trace::TraceEpisode;
pub use trace::TraceStep;
pub use skill::SkillExtractor;
pub use skill::ExtractedSkill;
pub use harness::OutcomeDrivenHarness;
pub use harness::HarnessDecision;
pub use harness::ActionProfile;
pub use harness::OutcomeRecord;
pub use co_evolution::BridgeCoEvolution;
pub use co_evolution::EvolutionBeat;
pub use co_evolution::SharedInsight;
pub use co_evolution::CrossDomainLink;

/// Aggregate: all four evolution subsystems in one place
pub struct EvolutionCore {
    pub trace: TraceRecorder,
    pub skill: SkillExtractor,
    pub harness: OutcomeDrivenHarness,
    pub co_evolution: BridgeCoEvolution,
}

impl Default for EvolutionCore {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionCore {
    pub fn new() -> Self {
        Self {
            trace: TraceRecorder::new(200),
            skill: SkillExtractor::new(),
            harness: OutcomeDrivenHarness::new(),
            co_evolution: BridgeCoEvolution::new(),
        }
    }

    pub fn heartbeat_tick(&mut self) -> Vec<String> {
        let mut signals = Vec::new();

        // 1. Consolidate co-evolution insights
        let consolidated = self.co_evolution.consolidate();
        if !consolidated.is_empty() {
            signals.push(format!("consolidated {} cross-domain insights", consolidated.len()));
        }

        // 2. Evolve skills from trace data
        let new_skills = self.skill.evolve();
        if !new_skills.is_empty() {
            signals.push(format!("evolved {} skills (bank: {})", new_skills.len(), self.skill.skill_count()));
        }

        // 3. Check redirection signals
        let redirects = self.co_evolution.redirection_signals();
        for (domain, reason) in &redirects {
            signals.push(format!("redirect {}: {}", domain, reason));
        }

        // 4. Summarize harness health
        let harness_summary = self.harness.summary();
        for (domain, (total, _successes, rate)) in &harness_summary {
            if *total > 0 && *rate < 0.5 {
                signals.push(format!("low health {}: {:.0}% success", domain, *rate * 100.0));
            }
        }

        signals
    }

    pub fn stats_summary(&self) -> String {
        format!(
            "traces={} skills={} harness_intercepts={} recoveries={}/{} co_evolution_reflections={} consolidations={}",
            self.trace.total_recorded,
            self.skill.skill_count(),
            self.harness.total_interceptions,
            self.harness.recovery_successes,
            self.harness.total_recoveries,
            self.co_evolution.total_reflections,
            self.co_evolution.total_consolidations,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_core_construction() {
        let core = EvolutionCore::new();
        assert!(core.stats_summary().contains("traces=0"));
    }

    #[test]
    fn test_heartbeat_tick_no_signals_initial() {
        let mut core = EvolutionCore::new();
        let signals = core.heartbeat_tick();
        // Should have some evolved skills potentially
        assert!(signals.is_empty() || signals.iter().any(|s| s.contains("evolved")));
    }
}
