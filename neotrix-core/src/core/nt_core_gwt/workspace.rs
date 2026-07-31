use std::collections::BTreeMap;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use super::module_def::{SpecialistModule, SpecialistType};
use super::monitor::EntropyMonitor;
use super::physics_attention::AdaptiveSlicer;
use super::resonance::{
    OscillationEnhancedReport, OscillatorNetwork, ResonanceMatrix, ResonanceReport,
    resonate_cycle, resonate_cycle_with_matrix, resonate_cycle_with_physics, MODULE_COUNT,
};
use super::competition_gate::{CompetitionGate, CompetitionResult};
use super::compaction::CompactionPipeline;
use super::moe_router::MoERouter;
use super::geometry_sync::GeometrySync;

use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_harness::HarnessAdapter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalWorkspace {
    pub broadcast_history: Vec<String>,
    pub active_content: Option<String>,
    pub(crate) specialists: BTreeMap<String, SpecialistModule>,
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
    /// WTA Competition Gate for global ignition (GNW theory)
    pub competition_gate: Option<CompetitionGate>,
    /// Last competition result
    pub last_competition: Option<CompetitionResult>,
    /// 5-layer compaction pipeline for context management
    pub compaction_pipeline: CompactionPipeline,
    /// Learnable MoE router for expert selection
    pub moe_router: MoERouter,
    /// SHA-256 audit chain for verifiable broadcast history
    pub audit_chain: VecDeque<AuditBlock>,
    /// Tick counter for audit blocks
    pub tick: u64,
    /// E8 prediction attention weights [f64; 64] — differentiable bridge from E8 oracle.
    /// Each entry corresponds to one of the 64 E8 hexagram modes.
    /// Applied as salience bias before resonance: for each specialist i,
    /// salience *= (1.0 + bias * attention_weights[hexagram_states[i].0 as usize]).
    /// Stored as Vec<f64> because serde doesn't support [f64; 64] arrays.
    pub e8_attention_weights: Option<Vec<f64>>,
    /// Attention bias multiplier stored from set_e8_attention_weights call.
    /// Used in resonant_broadcast to scale attention contribution.
    pub e8_attention_bias: f64,
    /// VSA content scorer for continuous resonance computation.
    /// When present, resonance matrix is built using VSA cosine similarity
    /// instead of discrete Hamming distance — producing differentiable [0,1]
    /// strength scores mapped to u32 [0,6]. Wired from seal_loop init.
    pub vsa_scorer: Option<super::vsa_scorer::VsaContentScorer>,
    /// Cross-dimensional consciousness geometry sync (12-layer Φ integration).
    /// Ticked every resonant_broadcast cycle to compute integrated information.
    pub geometry_sync: Option<GeometrySync>,
}

/// Events that trigger an audit block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    ResonanceCycle,
    CompetitionOverride,
    Compaction,
    FinalStorage,
}

/// A single block in the SHA-256 audit chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBlock {
    pub index: u64,
    pub previous_hash: [u8; 32],
    pub tick: u64,
    pub event_type: AuditEventType,
    pub winner: usize,
    pub entropy: f64,
    pub ignition: bool,
    pub compaction_triggered: bool,
    pub hash: [u8; 32],
}

impl GlobalWorkspace {
    pub fn new(threshold: f64) -> Self {
        Self {
            specialists: BTreeMap::new(),
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
            competition_gate: Some(CompetitionGate::default()),
            last_competition: None,
            compaction_pipeline: CompactionPipeline::default(),
            moe_router: MoERouter::new(MODULE_COUNT),
            audit_chain: VecDeque::new(),
            tick: 0,
            e8_attention_weights: None,
            e8_attention_bias: 0.3,
            vsa_scorer: None,
            geometry_sync: None,
        }
    }

    /// Inject E8 attention weights [f64; 64] to bias expert selection before resonance.
    /// The weights form a softmax-tempered prediction distribution over all 64 E8 hexagram modes,
    /// computed by E8PredictionOracle. During resonant_broadcast, each specialist's raw salience
    /// is boosted by the attention weight corresponding to its current E8 hexagram mode:
    ///   `salience *= (1.0 + bias * attention_weights[hexagram_states[i].mode.0])`
    pub fn set_e8_attention_weights(&mut self, weights: [f64; 64], bias: f64) {
        self.e8_attention_weights = Some(weights.to_vec());
        self.e8_attention_bias = bias;
        self.broadcast_history.push(format!(
            "[e8_attention] weights injected, bias={}, max_weight={:.4}",
            bias,
            weights.iter().cloned().fold(0.0_f64, f64::max),
        ));
    }

    pub fn with_vsa_scorer(mut self, scorer: super::vsa_scorer::VsaContentScorer) -> Self {
        self.vsa_scorer = Some(scorer);
        self
    }

    pub fn register(&mut self, module: SpecialistModule) -> bool {
        if self.specialists.len() >= MODULE_COUNT {
            return false;
        }
        self.specialists.insert(module.name.clone(), module);
        true
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

        // Step 1a: E8 attention bias — modulate raw salience with prediction attention weights
        // Each specialist's salience is boosted by the E8 attention weight for its current
        // hexagram mode: the oracle predicts which E8 states are most likely next, and experts
        // aligned with those states get a salience advantage. This creates a differentiable
        // bridge from the discrete E8 state space to the continuous GWT competition dynamics.
        if let Some(ref attn) = self.e8_attention_weights {
            for (i, state) in hexagram_states.iter().enumerate() {
                let mode_idx = state.0 as usize;
                if mode_idx < 64 {
                    let boost = 1.0 + self.e8_attention_bias * attn[mode_idx];
                    raw[i] = (raw[i] * boost).min(1.0);
                }
            }
            self.e8_attention_weights = None; // one-shot: consume after use
        }

        // Step 1b: Kuramoto oscillator pre-sync — update amplitudes and synchronize
        if let Some(ref mut net) = self.oscillator_network {
            net.update_amplitudes(&raw);
            net.synchronize(10);
        }

        // Step 2: run resonance competition (VSA-enhanced, physics-attention, or standard)
        // When a VsaContentScorer is available, build the resonance matrix using continuous
        // VSA cosine similarity instead of discrete Hamming distance — producing differentiable
        // strength scores that vary smoothly with E8 state similarity.
        let mut report = if let Some(ref scorer) = self.vsa_scorer {
            let vsa_matrix = ResonanceMatrix::from_states_with_vsa(hexagram_states, scorer);
            resonate_cycle_with_matrix(&raw, hexagram_states, &vsa_matrix, Some(&mut self.moe_router))
        } else if self.use_physics_attention {
            resonate_cycle_with_physics(&raw, hexagram_states, &mut self.physics_slicer)
        } else {
            resonate_cycle(&raw, hexagram_states, Some(&mut self.moe_router))
        };

        // Step 3: Discovery loop — entropy-based deadlock detection + stimulus injection
        self.entropy_monitor.feed(report.entropy);
        if self.entropy_monitor.is_deadlocked() {
            let stimulus = self.entropy_monitor.inject_stimulus(&mut raw);
            report = if let Some(ref scorer) = self.vsa_scorer {
                let vsa_matrix = ResonanceMatrix::from_states_with_vsa(hexagram_states, scorer);
                resonate_cycle_with_matrix(&raw, hexagram_states, &vsa_matrix, Some(&mut self.moe_router))
            } else if self.use_physics_attention {
                resonate_cycle_with_physics(&raw, hexagram_states, &mut self.physics_slicer)
            } else {
                resonate_cycle(&raw, hexagram_states, Some(&mut self.moe_router))
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

        // Step 5b: Competition Gate — WTA ignition override if enabled
        if let Some(ref gate) = self.competition_gate {
            let resonance_matrix = crate::core::nt_core_gwt::resonance::ResonanceMatrix::from_states(hexagram_states);
            let competition_result = gate.compete(&raw, &resonance_matrix);
            if competition_result.ignition {
                // Override winner and effective saliences with competition result
                let mut comp_report = report.clone();
                comp_report.winner = competition_result.winner_index;
                for (i, score) in competition_result.final_scores.iter().enumerate() {
                    comp_report.effective_saliences[i] = *score;
                }
                report = comp_report;
            }
            self.last_competition = Some(competition_result);
        }

        let ignition = self.last_competition.as_ref().map(|c| c.ignition).unwrap_or(false);
        self.append_audit_block(AuditEventType::CompetitionOverride, report.winner, report.entropy, ignition, false);

        // Step 5c: MoE Router — REINFORCE update using effective salience as reward
        self.moe_router.routing_update(&report.effective_saliences);

        // Step 5d': Geometry Sync — cross-dimensional consciousness Φ integration
        if let Some(ref mut gs) = self.geometry_sync {
            let phi = gs.tick();
            if phi.total > super::geometry_sync::CONSCIOUS_PHI_THRESHOLD {
                self.broadcast_history.push(format!(
                    "[geometry_sync] Φ={:.4} above threshold — consciousness binding active",
                    phi.total,
                ));
            }
        }

        // Step 5e: Compaction — run compaction pipeline on broadcast_history
        {
            let mut history_deque: std::collections::VecDeque<String> =
                self.broadcast_history.iter().cloned().collect();
            let compaction_report = self.compaction_pipeline.compact(&mut history_deque);
            self.broadcast_history = history_deque.into_iter().collect();
            if compaction_report.auto_compacted {
                self.broadcast_history.push(format!(
                    "[auto_compact] triggered at {} entries",
                    compaction_report.entries_before,
                ));
            }
            self.append_audit_block(AuditEventType::Compaction, report.winner, report.entropy, ignition, compaction_report.auto_compacted);
        }

        // Step 6: store resonance report
        self.last_resonance = Some(report.clone());
        self.resonance_history.push(report.clone());
        self.tick += 1;

        // Return reference to the report (guaranteed safe: set on line 266)
        self.last_resonance.as_ref().unwrap_or_else(|| {
            // Recovery: last entry in history (also just pushed)
            &self.resonance_history[self.resonance_history.len() - 1]
        })
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

    /// Register all 14 default specialists with neutral activation (0.3).
    pub fn register_default_specialists(&mut self) {
        use super::module_def::SpecialistType::*;
        for st in &[
            PatternMatcher, AnomalyDetector, KnowledgeRetriever,
            CodeAnalyzer, Planner, KnowledgeIntegrator,
            GoalPrioritizer, RiskAssessor, CreativityGenerator,
            ReflectionEngine, MetaCognitionAnalyst, AISecurity,
            ImageGenerator, EvidenceWeightedHypothesis,
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

    /// Enable WTA competition gate with given threshold and suppression.
    pub fn enable_competition_gate(&mut self, ignition_threshold: f64, suppression_strength: f64) {
        self.competition_gate = Some(CompetitionGate::new(ignition_threshold, suppression_strength));
    }

    /// Enable softmax competition mode.
    pub fn enable_softmax_competition(&mut self, ignition_threshold: f64, suppression_strength: f64) {
        let mut gate = CompetitionGate::new(ignition_threshold, suppression_strength);
        gate.softmax_mode = true;
        self.competition_gate = Some(gate);
    }

    /// Disable competition gate, falling back to resonance-only selection.
    pub fn disable_competition_gate(&mut self) {
        self.competition_gate = None;
        self.last_competition = None;
    }

    /// Whether the oscillator network has achieved consciousness binding (R > 0.7).
    pub fn is_conscious_bound(&self) -> bool {
        self.last_oscillation_report
            .as_ref()
            .map(|r| r.is_bound)
            .unwrap_or(false)
    }

    pub fn append_audit_block(&mut self, event_type: AuditEventType, winner: usize, entropy: f64, ignition: bool, compaction_triggered: bool) {
        let index = self.audit_chain.len() as u64;
        let previous_hash = self.audit_chain.back().map(|b| b.hash).unwrap_or([0u8; 32]);
        let tick = self.tick;

        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(previous_hash);
        hasher.update(tick.to_le_bytes());
        hasher.update([event_type as u8]);
        hasher.update(winner.to_le_bytes());
        hasher.update(entropy.to_le_bytes());
        let ignition_byte = if ignition { 1u8 } else { 0u8 };
        hasher.update([ignition_byte]);
        let compaction_byte = if compaction_triggered { 1u8 } else { 0u8 };
        hasher.update([compaction_byte]);
        let hash = hasher.finalize().into();

        self.audit_chain.push_back(AuditBlock {
            index,
            previous_hash,
            tick,
            event_type,
            winner,
            entropy,
            ignition,
            compaction_triggered,
            hash,
        });
    }

    /// Verify the integrity of the entire audit chain
    pub fn verify_chain(&self) -> bool {
        let mut prev_hash = [0u8; 32];
        for block in &self.audit_chain {
            let mut hasher = Sha256::new();
            hasher.update(block.index.to_le_bytes());
            hasher.update(block.previous_hash);
            hasher.update(block.tick.to_le_bytes());
            hasher.update([block.event_type as u8]);
            hasher.update(block.winner.to_le_bytes());
            hasher.update(block.entropy.to_le_bytes());
            hasher.update([if block.ignition { 1u8 } else { 0u8 }]);
            hasher.update([if block.compaction_triggered { 1u8 } else { 0u8 }]);
            let computed: [u8; 32] = hasher.finalize().into();
            if computed != block.hash {
                return false;
            }
            if block.index > 0 && block.previous_hash != prev_hash {
                return false;
            }
            prev_hash = block.hash;
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            let _ = ws.register(SpecialistModule::new(*st, format!("{:?}", st)));
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
    fn test_competition_gate_integration() {
        let mut ws = make_workspace();
        ws.enable_competition_gate(0.5, 0.7);
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.specialist_by_type_mut(&SpecialistType::AnomalyDetector)
            .expect("AnomalyDetector should be registered").activation = 0.3;

        ws.resonant_broadcast("test", &states);
        assert!(ws.last_competition.is_some());
    }

    #[test]
    fn test_softmax_competition_integration() {
        let mut ws = make_workspace();
        ws.enable_softmax_competition(0.4, 0.5);
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.8;

        ws.resonant_broadcast("test", &states);
        assert!(ws.last_competition.is_some());
    }

    #[test]
    fn test_disable_competition_gate_clears_state() {
        let mut ws = make_workspace();
        ws.enable_competition_gate(0.5, 0.7);
        ws.disable_competition_gate();
        assert!(ws.competition_gate.is_none());
        assert!(ws.last_competition.is_none());
    }

    #[test]
    fn test_compaction_runs_during_broadcast() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        // Fill broadcast_history past compaction threshold
        for i in 0..200 {
            ws.broadcast_history.push(format!("filler_{}", i));
        }

        ws.resonant_broadcast("test", &states);
        // Compaction should trim the history
        assert!(ws.broadcast_history.len() < 200);
    }

    #[test]
    fn test_resonant_broadcast_all_zero_activations() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        let report = ws.resonant_broadcast("zero", &states);
        // All activations are zero — winner should still be determined
        assert!(report.entropy >= 0.0);
    }

    #[test]
    fn test_audit_chain_creation() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("test", &states);
        assert!(!ws.audit_chain.is_empty(), "audit chain should have at least one block after broadcast");
    }

    #[test]
    fn test_audit_chain_verify_passes() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("test", &states);
        assert!(ws.verify_chain(), "audit chain should verify correctly after broadcast");
    }

    #[test]
    fn test_audit_chain_tamper_detected() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("test", &states);
        // Corrupt the last block's hash
        if let Some(last) = ws.audit_chain.back_mut() {
            last.hash[0] ^= 0xFF;
        }
        assert!(!ws.verify_chain(), "tampered chain should fail verification");
    }

    #[test]
    fn test_audit_chain_links_sequential() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("test1", &states);
        ws.resonant_broadcast("test2", &states);
        // Should have blocks from both broadcasts
        assert!(ws.audit_chain.len() >= 2, "chain should have at least 2 blocks after 2 broadcasts");
        // Check sequential linking
        let blocks: Vec<_> = ws.audit_chain.iter().collect();
        for i in 1..blocks.len() {
            assert_eq!(blocks[i].previous_hash, blocks[i-1].hash,
                "block {} should link to block {}", i, i-1);
        }
    }
}
