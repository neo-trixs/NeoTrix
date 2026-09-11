use super::trait_defs::*;
use crate::agents::SimAgent;
use crate::consciousness::ConsciousnessState;
use crate::foundation::SimEvent;
use crate::world_sim::WorldSnapshot;

/// Bidirectional bridge between WorldSim and NeoTrix core.
///
/// Data flow:
/// - Sim → Core: each tick, `build_report()` collects state into `SimReport`
/// - Core → Sim: core injects `CoreDirective`s, sim polls via `active_directives()`
/// - Accumulation: reports buffer for batch SEAL absorption
/// - Phi tracking: phi trend enables consciousness evolution monitoring
pub struct HostBridge {
    directives: Vec<CoreDirective>,
    report_buffer: Vec<SimReport>,
    buffer_limit: usize,
    phi_trend: Vec<f64>,
    phi_trend_window: usize,
    total_ticks: u64,
    adjustment_queue: Vec<CoreAdjustment>,
}

impl HostBridge {
    pub fn new(buffer_limit: usize, phi_trend_window: usize) -> Self {
        Self {
            directives: Vec::new(),
            report_buffer: Vec::new(),
            buffer_limit,
            phi_trend: Vec::new(),
            phi_trend_window,
            total_ticks: 0,
            adjustment_queue: Vec::new(),
        }
    }

    /// Inject a directive from the core.
    pub fn inject_directive(&mut self, directive: CoreDirective) {
        self.directives.push(directive);
    }

    /// Queue an adjustment for the sim to apply.
    pub fn queue_adjustment(&mut self, adjustment: CoreAdjustment) {
        self.adjustment_queue.push(adjustment);
    }

    /// Drain pending adjustments (sim calls this each tick).
    pub fn drain_adjustments(&mut self) -> Vec<CoreAdjustment> {
        std::mem::take(&mut self.adjustment_queue)
    }

    /// Get active (non-expired) directives.
    pub fn active_directives(&self, current_tick: u64) -> Vec<&CoreDirective> {
        self.directives
            .iter()
            .filter(|d| !d.is_expired(current_tick))
            .collect()
    }

    /// Remove expired directives.
    pub fn gc_directives(&mut self, current_tick: u64) {
        self.directives.retain(|d| !d.is_expired(current_tick));
    }

    /// Build a SimReport from current world state.
    pub fn build_report(
        &mut self,
        snapshot: &WorldSnapshot,
        consciousness: ConsciousnessState,
        phi: f64,
        events: Vec<SimEvent>,
        agents: &[SimAgent],
    ) -> SimReport {
        self.total_ticks += 1;

        self.phi_trend.push(phi);
        if self.phi_trend.len() > self.phi_trend_window {
            self.phi_trend.remove(0);
        }

        let mut agent_highlights: Vec<AgentHighlight> = agents
            .iter()
            .filter(|a| a.is_alive())
            .map(|a| AgentHighlight {
                agent_id: a.id,
                position: [a.core.position.x, a.core.position.y],
                fitness: a.fitness(),
                phi: a.phi as f64,
                dominant_trait: a.personality.dominant_trait(),
                notable_action: format!("{:?}", a.last_action),
            })
            .collect();
        agent_highlights.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        agent_highlights.truncate(5);

        let report = SimReport {
            snapshot: snapshot.clone(),
            consciousness,
            phi_trend: self.phi_trend.clone(),
            evolution_records: Vec::new(),
            notable_events: events,
            agent_highlights,
            recommended_core_actions: self.recommend_actions(&snapshot),
        };

        self.report_buffer.push(report.clone());
        report
    }

    /// Flush buffered reports for SEAL absorption.
    pub fn flush_reports(&mut self) -> Vec<SimReport> {
        std::mem::take(&mut self.report_buffer)
    }

    /// Buffer full and ready for absorption?
    pub fn should_absorb(&self) -> bool {
        self.report_buffer.len() >= self.buffer_limit
    }

    /// Value-weighted action scoring using core directives.
    pub fn value_score(&self, action: &str, current_tick: u64) -> f64 {
        let active = self.active_directives(current_tick);
        if active.is_empty() {
            return 0.5;
        }

        let mut score = 0.0;
        let mut total_weight = 0.0;
        for d in &active {
            let alignment = if action.contains(&d.goal) { 1.0 } else { 0.3 };
            score += alignment * d.weight;
            total_weight += d.weight;
        }

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.5
        }
    }

    fn recommend_actions(&self, snapshot: &WorldSnapshot) -> Vec<String> {
        let mut recs = Vec::new();
        if snapshot.mean_phi < 0.2 {
            recs.push("increase_phi_focus".to_string());
        }
        if snapshot.species_count < 2 {
            recs.push("reduce_selection_pressure".to_string());
        }
        if snapshot.population < 5 {
            recs.push("inject_agents".to_string());
        }
        if snapshot.mean_coherence < 0.3 {
            recs.push("boost_social_bonds".to_string());
        }
        recs
    }

    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub fn phi_trend(&self) -> &[f64] {
        &self.phi_trend
    }

    pub fn directive_count(&self) -> usize {
        self.directives.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::ConsciousnessState;

    fn mock_consciousness() -> ConsciousnessState {
        ConsciousnessState::new("test_agent")
    }

    fn mock_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            tick: 0,
            time: "00:00".to_string(),
            population: 10,
            mean_phi: 0.5,
            mean_coherence: 0.6,
            species_count: 3,
            evolution_generations: 5,
            resources_total: 50,
            resources_depleted: 5,
            total_relationships: 12,
            total_trades: 3,
            emotion_dominant: "Neutral".into(),
            emotion_valence: 0.0,
            emotion_arousal: 0.0,
            emotion_dominance: 0.0,
            safety_alerts: 0,
            audit_entries: 0,
            active_conflicts: 0,
            norms_compliance: 1.0,
            faction_count: 0,
        }
    }

    #[test]
    fn test_directive_expiry() {
        let d = CoreDirective::new("explore", 0.8, 10, 0);
        assert!(!d.is_expired(5));
        assert!(d.is_expired(10));
        assert!(d.is_expired(100));
    }

    #[test]
    fn test_directive_gc() {
        let mut bridge = HostBridge::new(10, 20);
        bridge.inject_directive(CoreDirective::new("a", 0.5, 5, 0));
        bridge.inject_directive(CoreDirective::new("b", 0.5, 20, 0));

        bridge.gc_directives(6);
        assert_eq!(bridge.active_directives(6).len(), 1);
        assert_eq!(bridge.active_directives(6)[0].goal, "b");
    }

    #[test]
    fn test_value_score_no_directives() {
        let bridge = HostBridge::new(10, 20);
        assert_eq!(bridge.value_score("eat", 0), 0.5);
    }

    #[test]
    fn test_value_score_with_directives() {
        let mut bridge = HostBridge::new(10, 20);
        bridge.inject_directive(CoreDirective::new("explore", 1.0, 100, 0));
        let score = bridge.value_score("explore_north", 0);
        assert!(score > 0.5);
    }

    #[test]
    fn test_report_buffering() {
        let mut bridge = HostBridge::new(3, 20);
        let snap = mock_snapshot();
        for _ in 0..2 {
            bridge.build_report(
                &snap,
                mock_consciousness(),
                0.5,
                Vec::new(),
                &[],
            );
        }
        assert!(!bridge.should_absorb());
        bridge.build_report(
            &snap,
            mock_consciousness(),
            0.5,
            Vec::new(),
            &[],
        );
        assert!(bridge.should_absorb());
        let reports = bridge.flush_reports();
        assert_eq!(reports.len(), 3);
        assert_eq!(bridge.report_buffer.len(), 0);
    }

    #[test]
    fn test_adjustment_queue() {
        let mut bridge = HostBridge::new(10, 20);
        bridge.queue_adjustment(CoreAdjustment::SelectionPressure(0.8));
        bridge.queue_adjustment(CoreAdjustment::MutationRate(1.5));
        let adj = bridge.drain_adjustments();
        assert_eq!(adj.len(), 2);
        assert!(bridge.drain_adjustments().is_empty());
    }
}
