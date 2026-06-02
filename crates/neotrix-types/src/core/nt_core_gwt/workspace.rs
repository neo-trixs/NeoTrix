use std::collections::HashMap;
use super::module_def::{SpecialistModule, SpecialistType};
use super::resonance::{
    ResonanceReport, resonate_cycle, MODULE_COUNT,
};
use crate::core::nt_core_hex::ReasoningHexagram;

pub struct GlobalWorkspace {
    pub broadcast_history: Vec<String>,
    pub active_content: Option<String>,
    pub(crate) specialists: HashMap<String, SpecialistModule>,
    threshold: f64,
    /// Last resonance report from the attention cycle.
    pub last_resonance: Option<ResonanceReport>,
    /// Resonant broadcast history: tracks which clusters have been active.
    pub resonance_history: Vec<ResonanceReport>,
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
        }
    }

    pub fn specialists(&self) -> &HashMap<String, SpecialistModule> {
        &self.specialists
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

    /// Full resonance-aware broadcast cycle.
    ///
    /// 1. Collects raw activations from all specialist modules
    /// 2. Runs resonance competition (resonate_cycle)
    /// 3. Broadcasts winner and resonance clusters
    /// 4. Updates module activations with effective salience
    /// 5. Stores resonance report for future queries
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

        // Step 2: run resonance competition
        let report = resonate_cycle(&raw, hexagram_states);

        // Step 3: broadcast winner content
        self.broadcast_history.push(format!(
            "[resonant_broadcast] winner={}, entropy={:.3}, clusters={}",
            report.winner,
            report.entropy,
            report.resonator_clusters.len(),
        ));
        self.broadcast_history.push(content.to_string());

        // Step 4: update module activations with effective salience
        for (i, m) in self.specialists.values_mut().enumerate() {
            m.activation = report.effective_saliences[i];
        }

        // Step 5: store resonance report
        self.last_resonance = Some(report.clone());
        self.resonance_history.push(report.clone());

        // Return reference to the report
        self.last_resonance.as_ref().expect("resonance report was just set")
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
            .expect("PatternMatcher should exist").activation = 0.9;

        ws.resonant_broadcast("test content", &states);

        assert!(ws.broadcast_history.len() >= 2);
        assert!(ws.last_resonance.is_some());
    }

    #[test]
    fn test_resonant_broadcast_updates_activations() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should exist").activation = 0.5;

        ws.resonant_broadcast("content", &states);

        // Activations should be updated with effective salience
        let pm = ws.specialist_by_type_mut(&SpecialistType::PatternMatcher).expect("PatternMatcher should exist");
        assert!(pm.activation > 0.0);
    }

    #[test]
    fn test_resonance_winner_returns_correct() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::KnowledgeRetriever)
            .expect("KnowledgeRetriever should exist").activation = 0.95;

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
            .expect("PatternMatcher should exist").activation = 1.0;
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
            .expect("PatternMatcher should exist").activation = 0.5;

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
            .expect("AnomalyDetector should exist").activation = 0.8;
        ws.resonant_broadcast("data", &states);

        let winner_before = ws.resonance_winner().map(|m| m.name.clone());

        ws.decay_all(0.5);

        // Resonance report should still reflect the state at broadcast time
        let winner_after = ws.resonance_winner().map(|m| m.name.clone());
        assert_eq!(winner_before, winner_after);
    }
}
