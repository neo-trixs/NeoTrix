use std::collections::HashMap;
use super::module_def::{SpecialistModule, SpecialistType};
use super::monitor::EntropyMonitor;
use super::physics_attention::AdaptiveSlicer;
use super::resonance::{
    OscillationEnhancedReport, OscillatorNetwork, ResonanceReport, resonate_cycle,
    resonate_cycle_with_physics, MODULE_COUNT,
};
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::neotrix::nt_mind::self_iterating::harness_adapter::HarnessAdapter;

pub struct GlobalWorkspace {
    pub broadcast_history: Vec<String>,
    pub active_content: Option<String>,
    pub(crate) specialists: HashMap<String, SpecialistModule>,
    threshold: f64,
    /// Last resonance report from the attention cycle.
    pub last_resonance: Option<ResonanceReport>,
    /// Resonant broadcast history: tracks which clusters have been active.
    pub resonance_history: Vec<ResonanceReport>,
    /// Kuramoto oscillator network for consciousness binding.
    pub oscillator_network: Option<OscillatorNetwork>,
    /// Last oscillation-enhanced report.
    pub last_oscillation_report: Option<OscillationEnhancedReport>,
    /// Life-Harness inspired runtime adapter for cross-environment transfer.
    pub harness_adapter: HarnessAdapter,
    /// Current environment context for harness adaptation.
    pub current_environment: Option<String>,
    /// Entropy-based deadlock monitor for runtime evaluation.
    pub entropy_monitor: EntropyMonitor,
    /// Physics-Attention adaptive slicer (Transolver-inspired).
    pub physics_slicer: AdaptiveSlicer,
    /// Whether to use Physics-Attention instead of fixed Hamming-distance resonance.
    pub use_physics_attention: bool,
}

impl GlobalWorkspace {
    pub fn new(threshold: f64) -> Self {
        Self {
            specialists: HashMap::new(),
            broadcast_history: Vec::new(),
            active_content: None,
            threshold,
            last_resonance: None,
            resonance_history: Vec::new(),
            oscillator_network: None,
            last_oscillation_report: None,
            harness_adapter: HarnessAdapter::default(),
            current_environment: None,
            entropy_monitor: EntropyMonitor::default(),
            physics_slicer: AdaptiveSlicer::default(),
            use_physics_attention: false,
        }
    }

    pub fn register(&mut self, module: SpecialistModule) {
        self.specialists.insert(module.name.clone(), module);
    }

    pub fn broadcast(&mut self, content: &str) {
        self.broadcast_history.push(content.to_string());
    }

    pub fn specialist_by_type_mut(&mut self, st: &SpecialistType) -> Option<&mut SpecialistModule> {
        self.specialists.values_mut().find(|m| m.specialist_type == *st)
    }

    /// Pre-resonance: returns specialists with raw activation above threshold.
    pub fn active_specialists(&self) -> Vec<&SpecialistModule> {
        self.specialists.values().filter(|m| m.activation >= self.threshold).collect()
    }

    /// Resonance-aware: returns specialists whose effective salience exceeds threshold.
    pub fn resonant_specialists(&self) -> Vec<&SpecialistModule> {
        let report = match self.last_resonance {
            Some(ref r) => r,
            None => return self.active_specialists(),
        };
        self.specialists.values()
            .enumerate()
            .filter(|(i, _)| report.effective_saliences.get(*i).copied().unwrap_or(0.0) >= self.threshold)
            .map(|(_, m)| m)
            .collect()
    }

    pub fn decay_all(&mut self, _rate: f64) {
        for m in self.specialists.values_mut() {
            m.activation *= 1.0 - _rate;
        }
    }

    /// Initialize Kuramoto oscillator network with the given number of specialists.
    pub fn init_oscillators(&mut self, num_specialists: usize) {
        self.oscillator_network = Some(OscillatorNetwork::new(num_specialists));
    }

    /// Full resonance-aware broadcast cycle with Discovery loop integration.
    ///
    /// 1. Collects raw activations from all specialist modules
    /// 2. Runs resonance competition (resonate_cycle)
    /// 3. Entropy-based deadlock detection — if stuck, injects stimulus (Variation)
    /// 4. Broadcasts winner and resonance clusters
    /// 5. Updates module activations with effective salience
    /// 6. Stores resonance report for future queries
    pub fn resonant_broadcast(
        &mut self,
        content: &str,
        hexagram_states: &[ReasoningHexagram; MODULE_COUNT],
    ) -> &ResonanceReport {
        // Step 1: collect raw activations
        let mut raw = [0.0; MODULE_COUNT];
        for (i, m) in self.specialists.values().enumerate() {
            raw[i] = m.activation;
        }

        // Step 1b: Kuramoto oscillator pre-sync — update amplitudes and synchronize
        if let Some(ref mut net) = self.oscillator_network {
            net.update_amplitudes(&raw);
            net.synchronize(10);
        }

        // Step 2: run resonance competition (standard or physics-attention)
        let mut report = if self.use_physics_attention {
            resonate_cycle_with_physics(&raw, hexagram_states, &mut self.physics_slicer)
        } else {
            resonate_cycle(&raw, hexagram_states)
        };

        // Step 3: Discovery loop — entropy-based deadlock detection + stimulus injection
        self.entropy_monitor.feed(report.entropy);
        if self.entropy_monitor.is_deadlocked() {
            let stimulus = self.entropy_monitor.inject_stimulus(&mut raw);
            report = if self.use_physics_attention {
                resonate_cycle_with_physics(&raw, hexagram_states, &mut self.physics_slicer)
            } else {
                resonate_cycle(&raw, hexagram_states)
            };
            self.broadcast_history.push(format!(
                "[entropy_monitor] deadlock detected! stimulus={:.3}, new_entropy={:.3}",
                stimulus, report.entropy,
            ));
        }

        // Step 3b: compute oscillation-enhanced report from synchronized network
        {
            let oscillation_enhanced = self.oscillator_network
                .as_ref()
                .map(|net| report.with_oscillation(net));
            self.last_oscillation_report = oscillation_enhanced;
        }

        // Step 3c: Life-Harness adaptation — apply proven environmental boost
        {
            let env = self.current_environment.clone().unwrap_or_default();
            if !env.is_empty() {
                if let Some(profile) = self.harness_adapter.active_profile() {
                    for (_, m) in self.specialists.iter_mut() {
                        m.apply_harness_boost(&env, 0.05);
                        if let Some(adaptations) = profile.specialist_adaptations.get(&m.specialist_type) {
                            m.activation *= 1.0 + (adaptations.len() as f64 * 0.02).min(0.2);
                        }
                    }
                }
            }
        }

        // Step 4: broadcast winner content
        self.broadcast_history.push(format!(
            "[resonant_broadcast] winner={}, entropy={:.3}, clusters={}",
            report.winner,
            report.entropy,
            report.resonator_clusters.len(),
        ));
        self.broadcast_history.push(content.to_string());

        // Step 5: update module activations with effective salience
        for (i, m) in self.specialists.values_mut().enumerate() {
            m.activation = report.effective_saliences[i];
        }

        // Step 6: store resonance report
        self.last_resonance = Some(report.clone());
        self.resonance_history.push(report.clone());

        // Return reference to the report
        self.last_resonance.as_ref().expect("last_resonance was just set")
    }

    /// Get the winner module from the last resonance cycle.
    pub fn resonance_winner(&self) -> Option<&SpecialistModule> {
        let report = self.last_resonance.as_ref()?;
        self.specialists.values().nth(report.winner)
    }

    /// Get resonance cluster members as module references.
    pub fn resonance_clusters(&self) -> Vec<Vec<&SpecialistModule>> {
        let report = match self.last_resonance {
            Some(ref r) => r,
            None => return vec![],
        };
        report.resonator_clusters.iter()
            .map(|cluster| {
                cluster.iter()
                    .filter_map(|&i| self.specialists.values().nth(i))
                    .collect()
            })
            .collect()
    }

    /// Whether the attention is focused or distributed (from last resonance).
    pub fn attention_state(&self) -> AttentionState {
        match self.last_resonance {
            Some(ref r) if r.is_focused() => AttentionState::Focused,
            Some(ref r) if r.is_distributed() => AttentionState::Distributed,
            Some(_) => AttentionState::Balanced,
            None => AttentionState::Idle,
        }
    }

    /// Register all 13 default specialists with neutral activation (0.3).
    pub fn register_default_specialists(&mut self) {
        use super::module_def::SpecialistType::*;
        for st in &[
            PatternMatcher, AnomalyDetector, KnowledgeRetriever,
            CodeAnalyzer, Planner, KnowledgeIntegrator,
            GoalPrioritizer, RiskAssessor, CreativityGenerator,
            ReflectionEngine, MetaCognitionAnalyst, AISecurity,
            ImageGenerator,
        ] {
            let name = format!("{:?}", st);
            if !self.specialists.contains_key(&name) {
                let mut module = SpecialistModule::new(*st, name);
                module.activation = 0.3;
                self.register(module);
            }
        }
        if self.oscillator_network.is_none() && self.specialists.len() >= 3 {
            self.init_oscillators(self.specialists.len());
        }
    }

    /// Whether the oscillator network has achieved consciousness binding (R > 0.7).
    pub fn is_conscious_bound(&self) -> bool {
        self.last_oscillation_report
            .as_ref()
            .map(|r| r.is_bound)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionState {
    Idle,
    Focused,
    Balanced,
    Distributed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::default_specialist_states;

    fn make_workspace() -> GlobalWorkspace {
        let mut ws = GlobalWorkspace::new(0.3);
        for st in &[
            SpecialistType::PatternMatcher,
            SpecialistType::AnomalyDetector,
            SpecialistType::KnowledgeRetriever,
        ] {
            ws.register(SpecialistModule::new(*st, format!("{:?}", st)));
        }
        ws
    }

    #[test]
    fn test_resonant_broadcast_basic() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        // Set one module high activation
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;

        ws.resonant_broadcast("test content", &states);

        assert!(ws.broadcast_history.len() >= 2);
        assert!(ws.last_resonance.is_some());
    }

    #[test]
    fn test_resonant_broadcast_updates_activations() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.5;

        ws.resonant_broadcast("content", &states);

        // Activations should be updated with effective salience
        let pm = ws.specialist_by_type_mut(&SpecialistType::PatternMatcher).expect("PatternMatcher should be registered for activation check");
        assert!(pm.activation > 0.0);
    }

    #[test]
    fn test_resonance_winner_returns_correct() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::KnowledgeRetriever)
            .expect("KnowledgeRetriever should be registered").activation = 0.95;

        ws.resonant_broadcast("query", &states);

        let winner = ws.resonance_winner();
        assert!(winner.is_some());
    }

    #[test]
    fn test_attention_state_transitions() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        // Initially idle
        assert_eq!(ws.attention_state(), AttentionState::Idle);

        // After broadcast, should be focused (one module dominates)
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered for attention test").activation = 1.0;
        ws.resonant_broadcast("test", &states);
        assert_ne!(ws.attention_state(), AttentionState::Idle);
    }

    #[test]
    fn test_resonant_specialists_differs_from_active() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        // Without resonance, only active_specialists works
        let active_before = ws.resonant_specialists().len();
        assert_eq!(active_before, 0); // all zero activation

        // Activate one module at threshold level
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered for resonant test").activation = 0.5;

        // Before resonance cycle, resonant_specialists falls back to active
        let before = ws.resonant_specialists().len();
        assert_eq!(before, 1); // just PatternMatcher

        // After resonance cycle, resonance boost may pull in more
        ws.resonant_broadcast("test", &states);
        // Effective saliences are set, so resonant_specialists works from stored report
        let after = ws.resonant_specialists().len();
        assert!(after >= 1);
    }

    #[test]
    fn test_decay_does_not_affect_resonance_cache() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::AnomalyDetector)
            .expect("AnomalyDetector should be registered").activation = 0.8;
        ws.resonant_broadcast("data", &states);

        let winner_before = ws.resonance_winner().map(|m| m.name.clone());

        ws.decay_all(0.5);

        // Resonance report should still reflect the state at broadcast time
        let winner_after = ws.resonance_winner().map(|m| m.name.clone());
        assert_eq!(winner_before, winner_after);
    }

    #[test]
    fn test_oscillator_init() {
        let mut gw = GlobalWorkspace::new(0.5);
        gw.init_oscillators(5);
        assert!(gw.oscillator_network.is_some());
    }

    #[test]
    fn test_oscillation_report_after_broadcast() {
        let mut gw = make_workspace();
        gw.init_oscillators(3);
        let states = default_specialist_states();
        let report = gw.resonant_broadcast("test", &states).clone();
        assert!(gw.last_oscillation_report.is_some() || report.entropy >= 0.0);
    }

    #[test]
    fn test_conscious_bound_false_without_oscillators() {
        let gw = GlobalWorkspace::new(0.3);
        assert!(!gw.is_conscious_bound());
    }

    #[test]
    fn test_is_conscious_bound_after_broadcast() {
        let mut gw = make_workspace();
        gw.init_oscillators(3);
        let states = default_specialist_states();
        // All modules same activation → likely synchronous after 10 steps
        for (_, m) in gw.specialists.iter_mut() {
            m.activation = 0.9;
        }
        let _ = gw.resonant_broadcast("sync test", &states).clone();
        // is_conscious_bound should be callable without panic
        let _bound = gw.is_conscious_bound();
        assert!(gw.last_oscillation_report.is_some() || gw.last_resonance.is_some());
    }

    #[test]
    fn test_conscious_bound_varies_with_synchrony() {
        let mut gw_high = make_workspace();
        gw_high.init_oscillators(3);
        let states = default_specialist_states();
        for (_, m) in gw_high.specialists.iter_mut() {
            m.activation = 0.95;
        }
        gw_high.resonant_broadcast("high sync", &states);
        let _high_bound = gw_high.is_conscious_bound();

        let mut gw_low = make_workspace();
        gw_low.init_oscillators(3);
        for (_, m) in gw_low.specialists.iter_mut() {
            m.activation = 0.1;
        }
        gw_low.resonant_broadcast("low sync", &states);
        let low_bound = gw_low.is_conscious_bound();

        // Both calls return a boolean without panic
        let _ = (_high_bound, low_bound);
    }

    #[test]
    fn test_oscillator_init_zero_specialists() {
        let mut gw = GlobalWorkspace::new(0.5);
        gw.init_oscillators(0);
        assert!(gw.oscillator_network.is_some());
    }

    #[test]
    fn test_resonant_broadcast_empty_specialists() {
        let mut gw = GlobalWorkspace::new(0.3);
        let states = default_specialist_states();
        let report = gw.resonant_broadcast("empty", &states);
        assert!(report.entropy >= 0.0);
        assert!(gw.broadcast_history.len() >= 2);
    }

    #[test]
    fn test_resonant_broadcast_all_zero_activations() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        let report = ws.resonant_broadcast("zero", &states);
        // All activations are zero — winner should still be determined
        assert!(report.entropy >= 0.0);
    }
}
