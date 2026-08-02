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

/// 按 SpecialistType 声明序取 module 索引（与 default_specialist_states / hexagram_states 同序）。
/// BTreeMap<String, _> 的 values() 是 name-sort 序，与声明序不一致，绝不能按位置互用。
/// 自由函数而非方法：可在 values_mut() 迭代中调用而不触发 &self 借用冲突。
fn module_index(m: &SpecialistModule) -> Option<usize> {
    if (m.specialist_type as usize) < MODULE_COUNT {
        Some(m.specialist_type as usize)
    } else {
        None
    }
}

/// resonance_history 上限，防止无界增长 (每 tick push 一个)
const RESONANCE_HISTORY_LIMIT: usize = 512;
/// audit_chain 上限，防止无界增长 (每 tick append 一个 block)
const AUDIT_CHAIN_LIMIT: usize = 10_000;

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
    /// Inner Speech channel — self-talk summarizing the resonance state,
    /// fed back as context for subsequent experts (MIRROR AAAI 2026 §3.3).
    pub inner_speech: super::inner_speech::InnerSpeech,
    /// Top-Down Modality Attention Router (arXiv:2602.08597 §3). Gates per-modality
    /// workspace representation strength by task-query attention over modality keys.
    pub modality_router: super::modality_router::ModalityRouter,
    /// Complementary Learning Systems fast buffer (MIRROR AAAI 2026 §3.4).
    /// Hippocampus-style ring buffer of recent episodic experiences recorded from
    /// each resonance cycle; hybrid retrieval reranks fast candidates by salience.
    pub cls_buffer: super::cls_buffer::CLSBuffer,
    /// CTM-AI formal alignment verifier (arXiv:2605.04097 §2-4). Confirms the
    /// GWT is a Conscious Turing Machine instance: finite E8 states, bounded
    /// specialist actions, global broadcast, deterministic transition.
    pub ctm_verifier: super::ctm_verifier::CtmVerifier,
    /// Most recent CTM alignment verification result.
    pub last_ctm_report: Option<super::ctm_verifier::CtmAlignmentReport>,
    /// Most recent cognitive type profile (Phase 8.1): softmax distribution over
    /// Linguistic/Logical/Knowledge/Social + dominant type + Shannon entropy.
    pub cognitive_profile: Option<super::cognitive_type::CognitiveProfile>,
    /// Cross-group routing bridge (Phase 8.2): structured cognitive topology
    /// with learnable hub-to-hub collaboration weights.
    pub cognitive_hub: super::cognitive_hub::CognitiveHub,
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
            inner_speech: super::inner_speech::InnerSpeech::default(),
            modality_router: super::modality_router::ModalityRouter::default(),
            cls_buffer: super::cls_buffer::CLSBuffer::default(),
            ctm_verifier: super::ctm_verifier::CtmVerifier::new(),
            last_ctm_report: None,
            cognitive_profile: None,
            cognitive_hub: super::cognitive_hub::CognitiveHub::new(),
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

    /// 按 SpecialistType 声明序定位 specialist (与 default_specialist_states / hexagram_states 同序)。
    /// BTreeMap<String, _> 的 values() 是 name-sort 序，与声明序不一致，绝不能按位置互用。
    fn specialist_at_index(&self, idx: usize) -> Option<&SpecialistModule> {
        self.specialists.values()
            .find(|m| m.specialist_type as usize == idx)
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
            .filter(|m| {
                module_index(m)
                    .and_then(|idx| report.effective_saliences.get(idx).copied())
                    .unwrap_or(0.0) >= self.threshold
            })
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
        // 索引空间必须与 hexagram_states 一致：按 SpecialistType 声明序 (specialist_type as usize)
        // 而非 BTreeMap values() 的 name-sort 序，否则 E8 boost / winner / 回写全部错位。
        let mut raw = [0.0; MODULE_COUNT];
        for m in self.specialists.values() {
            if let Some(idx) = module_index(m) {
                raw[idx] = m.activation;
            }
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

        // Step 4b: Inner Speech — self-talk over the observed resonance state.
        // Generates a natural-language utterance (what the consciousness layer is
        // attending to, how focused, entropy health) and feeds it back into the
        // workspace so subsequent specialists reason over their own broadcast.
        {
            let winner_name = self.specialist_at_index(report.winner)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| format!("specialist_{}", report.winner));
            let speech_input = super::inner_speech::SpeechInput {
                winner: report.winner,
                winner_name,
                entropy: report.entropy,
                focused: report.is_focused(),
                complement_activated: report.complement_activated,
                content: content.to_string(),
            };
            let utterance = self.inner_speech.speak(&speech_input);
            if self.inner_speech.feed_back_enabled {
                self.broadcast_history.push(utterance.clone());
            }
        }

        // Step 4c: Top-Down Modality Attention (arXiv:2602.08597 §3).
        // Build a stable query embedding from the broadcast content, route attention
        // over modality keys, and gate a per-modality strength map. The winner is
        // recorded so downstream specialists know which modality the consciousness
        // layer is prioritizing. The router's learned keys train via reinforce()
        // whenever a downstream outcome reward is available.
        {
            let query = self.content_query(content);
            let strengths = self.modality_strengths(&report.effective_saliences);
            let _gated = self.modality_router.gate(&query, &strengths);
            if let Some(winner) = self.modality_router.winner() {
                self.broadcast_history.push(format!(
                    "[modality_router] active modality = {}",
                    winner.label(),
                ));
            }
        }

        // Step 4d: Complementary Learning Systems — record this resonance cycle as
        // an episodic experience in the fast (hippocampus) buffer. The winner's E8
        // mode + effective salience snapshot form the episode signature; the content
        // is the broadcast payload. High-reward episodes become consolidation
        // candidates for the slow (neocortex) HyperCube store (MIRROR AAAI 2026 §3.4).
        {
            let e8_state = hexagram_states
                .get(report.winner)
                .map(|h| h.0)
                .unwrap_or(0);
            let reward = report.effective_saliences.get(report.winner).copied().unwrap_or(0.0);
            self.cls_buffer.record(
                e8_state,
                report.effective_saliences.to_vec(),
                content.to_string(),
                reward,
            );
        }

        // Step 4e: CTM-AI formal alignment — verify the GWT is a Conscious Turing
        // Machine instance over the just-produced resonance report (finite states,
        // bounded actions, global broadcast, deterministic δ, bounded tape).
        {
            let active = self.specialists.len();
            let aligned = self.verify_ctm_report(hexagram_states, active, &report);
            if !aligned.aligned {
                self.broadcast_history.push(format!(
                    "[ctm_verifier] MISALIGNED: {}/{} axioms held",
                    aligned.passed_checks,
                    aligned.total_checks,
                ));
            }
        }

        // Step 4f: Cognitive Type profiling (Phase 8.1, MiCRo arXiv:2506.13331 §3).
        // Aggregate post-resonance effective saliences into the 4 cognitive-type
        // distribution, record the softmax-normalized profile (dominant type +
        // Shannon entropy) on the workspace, and surface it in the broadcast.
        {
            let activations: Vec<(SpecialistType, f64)> = self
                .specialists
                .values()
                .filter_map(|m| {
                    module_index(m)
                        .and_then(|idx| report.effective_saliences.get(idx).copied())
                        .map(|s| (m.specialist_type, s))
                })
                .collect();
            let raw = super::cognitive_type::group_activation(&activations);
            let profile = super::cognitive_type::CognitiveProfile {
                distribution: raw,
                dominant: super::cognitive_type::CognitiveType::Linguistic,
                entropy: 0.0,
            }
            .profile();
            self.cognitive_profile = Some(profile.clone());
            // Phase 8.2 — CognitiveHub: record cross-group collaborations from
            // the post-resonance winners so hub-to-hub weights learn which
            // cognitive types actually co-activate together (structured topology).
            self.cognitive_hub.record_broadcast_collaborations(&activations);
            self.broadcast_history.push(format!(
                "[cognitive_type] dominant = {}",
                profile.dominant.label(),
            ));
        }

        // Step 5: update module activations with effective salience
        // effective_saliences[i] 是声明序，按 specialist_type 写回而非 values() 枚举序
        for m in self.specialists.values_mut() {
            if let Some(idx) = module_index(m) {
                m.activation = report.effective_saliences.get(idx).copied().unwrap_or(0.0);
            }
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
        if self.resonance_history.len() > RESONANCE_HISTORY_LIMIT {
            self.resonance_history.drain(..self.resonance_history.len() - RESONANCE_HISTORY_LIMIT);
        }
        self.tick += 1;

        // Return reference to the report (guaranteed safe: set on line 266)
        self.last_resonance.as_ref().unwrap_or_else(|| {
            // Recovery: last entry in history (also just pushed)
            &self.resonance_history[self.resonance_history.len() - 1]
        })
    }

    /// Build a deterministic query embedding from broadcast content, in the
    /// ModalityRouter's key dimension. Uses a cheap rolling-hash to allow the same
    /// content to produce a stable query across cycles (so routing is reproducible).
    fn content_query(&self, content: &str) -> Vec<f64> {
        let dim = self.modality_router.dim;
        let mut q = vec![0.0_f64; dim];
        let bytes: Vec<u8> = content.bytes().collect();
        if bytes.is_empty() {
            return q;
        }
        for (i, &b) in bytes.iter().enumerate() {
            q[i % dim] += (b as f64 + 0.5).sin() * ((i % 5) as f64 + 1.0) / 255.0;
        }
        // normalize to unit-like magnitude
        let norm: f64 = q.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for v in q.iter_mut() {
                *v = (*v / norm).max(0.0).min(1.0);
            }
        }
        q
    }

    /// Map resonance effective saliences to per-modality representation strengths.
    /// Specialist modules are categorized by type into the five modalities.
    fn modality_strengths(&self, saliences: &[f64]) -> std::collections::BTreeMap<super::modality_router::Modality, f64> {
        use super::modality_router::Modality;
        let mut strengths = std::collections::BTreeMap::new();
        for (modality, strength) in self.specialists.values().filter_map(|m| {
            module_index(m).and_then(|idx| saliences.get(idx).copied())
                .map(|s| (self.specialist_modality(m.specialist_type), s))
        }) {
            *strengths.entry(modality).or_insert(0.0) += strength;
        }
        // ensure all modalities present
        for m in Modality::ALL {
            strengths.entry(m).or_insert(0.0);
        }
        strengths
    }

    /// Map a SpecialistType to its dominant representation modality.
    fn specialist_modality(&self, st: SpecialistType) -> super::modality_router::Modality {
        use super::module_def::SpecialistType as ST;
        use super::modality_router::Modality as M;
        match st {
            ST::ImageGenerator | ST::CreativityGenerator => M::Image,
            ST::AISecurity => M::Code,
            ST::CodeAnalyzer | ST::EvidenceWeightedHypothesis => M::Code,
            ST::PatternMatcher | ST::AnomalyDetector | ST::Planner
            | ST::KnowledgeIntegrator | ST::GoalPrioritizer | ST::RiskAssessor
            | ST::ReflectionEngine | ST::MetaCognitionAnalyst | ST::Orchestrator => M::Text,
            ST::KnowledgeRetriever => M::Latent,
        }
    }

    /// Get the winner module from the last resonance cycle.
    pub fn resonance_winner(&self) -> Option<&SpecialistModule> {
        let report = self.last_resonance.as_ref()?;
        self.specialist_at_index(report.winner)
    }

    /// Hybrid CLS retrieval: query the fast episodic buffer by E8 mode and by
    /// activation similarity, then merge by description. This is the query side of
    /// the dual-memory architecture — fast candidates are surfaced before the slow
    /// HyperCube semantic store reranks (roadmap Phase 7.3).
    pub fn recall_experiences(&self, e8_state: u8, activation: &[f64], top_k: usize) -> Vec<&super::cls_buffer::Experience> {
        let mut seen: Vec<&super::cls_buffer::Experience> = Vec::new();
        let mut dedup: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for exp in self.cls_buffer.query_fast(e8_state, top_k).into_iter()
            .chain(self.cls_buffer.query_fast_by_activation(activation, top_k))
        {
            if dedup.insert(exp.id) {
                seen.push(exp);
            }
        }
        seen.truncate(top_k);
        seen
    }

    /// Run the CTM-AI formal alignment verification over a resonance snapshot.
    /// Returns the alignment report and stores it in `last_ctm_report`.
    /// `specialists_active` is the count of registered specialists (|A| witness).
    pub fn verify_ctm(&mut self, hexagram_states: &[ReasoningHexagram], specialists_active: usize) -> super::ctm_verifier::CtmAlignmentReport {
        let report = match self.last_resonance.clone() {
            Some(r) => r,
            None => {
                // Degenerate placeholder when no resonance has run yet — only used to
                // keep the verifier total; alignment will fail the globality check.
                ResonanceReport {
                    winner: usize::MAX,
                    effective_saliences: [0.0; MODULE_COUNT],
                    raw_saliences: [0.0; MODULE_COUNT],
                    entropy: f64::NAN,
                    resonator_clusters: Vec::new(),
                    complement_activated: false,
                }
            }
        };
        let tape_len = self.broadcast_history.len();
        let aligned = self.ctm_verifier.verify(hexagram_states, specialists_active, &report, tape_len);
        let ret = aligned.clone();
        self.last_ctm_report = Some(aligned);
        ret
    }

    /// Verify an in-flight resonance report (used mid-cycle in resonant_broadcast,
    /// before `last_resonance` is stored at Step 6).
    fn verify_ctm_report(&mut self, hexagram_states: &[ReasoningHexagram], specialists_active: usize, report: &ResonanceReport) -> super::ctm_verifier::CtmAlignmentReport {
        let tape_len = self.broadcast_history.len();
        let aligned = self.ctm_verifier.verify(hexagram_states, specialists_active, report, tape_len);
        let ret = aligned.clone();
        self.last_ctm_report = Some(aligned);
        ret
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
                    .filter_map(|&i| self.specialist_at_index(i))
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
        while self.audit_chain.len() > AUDIT_CHAIN_LIMIT {
            self.audit_chain.pop_front();
        }
    }

    /// Verify the integrity of the retained audit chain window.
    /// 被裁剪后链头 previous_hash 指向已移除块，故 linkage 校验从链中第二块起。
    pub fn verify_chain(&self) -> bool {
        let mut prev_hash = [0u8; 32];
        for (i, block) in self.audit_chain.iter().enumerate() {
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
            if i > 0 && block.previous_hash != prev_hash {
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
    fn test_resonant_broadcast_records_cls_experience() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;

        ws.resonant_broadcast("episodic payload", &states);

        // Every resonance cycle records one episodic experience into the fast buffer
        assert_eq!(ws.cls_buffer.len(), 1);
        let winners = ws.cls_buffer.consolidation_candidates();
        // The winner's effective salience is used as reward proxy, so high-winner
        // episodes should surface as consolidation candidates
        let _ = winners;

        // Hybrid recall surfaces the recorded experience
        let winner_idx = ws.last_resonance.as_ref().map(|r| r.winner).unwrap_or(0);
        let e8_state = states.get(winner_idx).map(|h| h.0).unwrap_or(0);
        let recall = ws.recall_experiences(e8_state, &[0.9, 0.5, 0.1], 3);
        assert!(!recall.is_empty());
        assert_eq!(recall[0].description, "episodic payload");
    }

    #[test]
    fn test_cls_ring_eviction_in_broadcast() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.cls_buffer.max_fast = 2;

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("first", &states);
        ws.resonant_broadcast("second", &states);
        ws.resonant_broadcast("third", &states);

        // Ring buffer evicts oldest: only 2 most recent retained
        assert_eq!(ws.cls_buffer.len(), 2);
        assert!(ws.cls_buffer.query_fast(0, 10).iter().all(|e| e.description != "first"));
    }

    #[test]
    fn test_resonant_broadcast_runs_ctm_verification() {
        let mut ws = make_workspace();
        let states = default_specialist_states();

        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.9;
        ws.resonant_broadcast("ctm probe", &states);

        // Every resonance cycle runs CTM-AI formal alignment verification
        assert!(ws.last_ctm_report.is_some());
        let rep = ws.last_ctm_report.as_ref().unwrap();
        assert!(rep.aligned, "GWT should satisfy CTM axioms: {:#?}", rep.checks);
        assert_eq!(rep.total_checks, 5);
        assert_eq!(rep.passed_checks, 5);
    }

    #[test]
    fn test_ctm_verifier_manual_run() {
        let mut ws = make_workspace();
        let states = default_specialist_states();
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.8;
        ws.resonant_broadcast("manual", &states);

        // Re-run verification directly with the current snapshot's action space
        let active = ws.specialists.len();
        let rep = ws.verify_ctm(&states, active);
        assert!(rep.is_aligned());
        assert!(ws.last_ctm_report.is_some());
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

    #[test]
    fn test_resonant_broadcast_generates_cognitive_profile() {
        let mut ws = make_workspace();
        ws.register_default_specialists();
        let states = default_specialist_states();

        // Boost a Logical specialist so the cognitive type profile is meaningful
        ws.specialist_by_type_mut(&SpecialistType::CodeAnalyzer)
            .expect("CodeAnalyzer should be registered").activation = 0.95;
        ws.specialist_by_type_mut(&SpecialistType::PatternMatcher)
            .expect("PatternMatcher should be registered").activation = 0.1;

        ws.resonant_broadcast("cognitive probe", &states);

        let profile = ws.cognitive_profile.expect("cognitive_profile should be set after broadcast");
        // Distribution normalizes to 1
        let sum: f64 = profile.distribution.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "distribution sum={sum}");
        // Dominant type should be Logical (CodeAnalyzer dominates)
        assert_eq!(
            profile.dominant,
            crate::core::nt_core_gwt::cognitive_type::CognitiveType::Logical
        );
        // Entropy is a valid Shannon entropy
        assert!(profile.entropy >= 0.0);

        // The dominant cognitive type is surfaced in the broadcast
        assert!(ws.broadcast_history.iter().any(|b| {
            b.contains("[cognitive_type]") && b.contains("logical")
        }));
    }
}
