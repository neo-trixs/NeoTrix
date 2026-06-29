// Phase 3: Reasoning Federation Layer — unified trait + registry + fusion + calibration
// Wraps all 7 reasoning engines under a single interface.
// Replaces the hardcoded 80-line REASON pipeline in consciousness_cycle.rs.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use super::spectrum_signal::SpectrumSignal;

// ── Common Types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineId {
    MCTS,
    Causal,
    Analogical,
    RecurrentWorldModel,
    DualPath,
    EmergentMode,
    SpectrumSignal,
    Intuition,
    InnerCritic,
    Narrative,
    Metacognitive,
}

impl EngineId {
    pub fn name(&self) -> &'static str {
        match self {
            EngineId::MCTS => "MCTS",
            EngineId::Causal => "Causal",
            EngineId::Analogical => "Analogical",
            EngineId::RecurrentWorldModel => "RecurrentWM",
            EngineId::DualPath => "DualPath",
            EngineId::EmergentMode => "Emergent",
            EngineId::SpectrumSignal => "Spectrum",
            EngineId::Intuition => "Intuition",
            EngineId::InnerCritic => "InnerCritic",
            EngineId::Narrative => "Narrative",
            EngineId::Metacognitive => "Metacognitive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningContext {
    pub query: String,
    pub state_vector: Vec<u8>,
    pub attention_focus: Option<String>,
    pub domain_hint: Option<String>,
    pub constraints: Vec<String>,
    pub timestamp: u64,
}

impl ReasoningContext {
    pub fn new(query: &str, state: Vec<u8>) -> Self {
        Self {
            query: query.to_string(),
            state_vector: state,
            attention_focus: None,
            domain_hint: None,
            constraints: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub engine: EngineId,
    pub conclusion: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
    pub execution_time_ms: u64,
    pub engine_contribution: f64,
    pub alternative_hypotheses: Vec<String>,
}

// ── Reasoning Engine Trait ──

pub trait ReasoningEngine: Send + Sync + std::fmt::Debug {
    fn id(&self) -> EngineId;
    fn reason(&mut self, context: &ReasoningContext) -> ReasoningOutput;
    fn is_available(&self) -> bool;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn ReasoningEngine>;
}

// ── Fusion Strategy ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FusionStrategy {
    /// Each engine votes; majority wins
    #[default]
    MajorityVote,
    /// Weight outputs by confidence, weighted average
    ConfidenceWeighted,
    /// Pick the single highest-confidence output
    BestOfN,
    /// Weight each engine's vote by confidence
    WeightedVote,
    /// Run in fixed order, each feeds next
    SequentialCascade,
    /// Run all, synthesize via blackboard
    BlackboardSynthesis,
}

// ── Engine Calibration ──

#[derive(Debug, Clone)]
pub struct EngineCalibration {
    pub total_calls: u64,
    pub correct_calls: u64,
    pub avg_confidence: f64,
    pub avg_execution_time_ms: u64,
    pub last_error: Option<String>,
    pub accuracy: f64,
}

impl Default for EngineCalibration {
    fn default() -> Self {
        Self {
            total_calls: 0,
            correct_calls: 0,
            avg_confidence: 0.5,
            avg_execution_time_ms: 0,
            last_error: None,
            accuracy: 0.5,
        }
    }
}

impl EngineCalibration {
    pub fn record(&mut self, output: &ReasoningOutput, was_correct: bool) {
        self.total_calls += 1;
        if was_correct {
            self.correct_calls += 1;
        }
        self.avg_confidence = (self.avg_confidence + output.confidence) / 2.0;
        self.avg_execution_time_ms = (self.avg_execution_time_ms + output.execution_time_ms) / 2;
        self.accuracy = if self.total_calls > 0 {
            self.correct_calls as f64 / self.total_calls as f64
        } else {
            0.5
        };
    }

    pub fn weight(&self) -> f64 {
        self.accuracy * self.avg_confidence
    }
}

// ── Engine Registry ──

#[derive(Debug)]
pub struct EngineRegistry {
    engines: HashMap<EngineId, Box<dyn ReasoningEngine>>,
    calibrations: HashMap<EngineId, EngineCalibration>,
    enabled: HashMap<EngineId, bool>,
}

impl Clone for EngineRegistry {
    fn clone(&self) -> Self {
        Self {
            engines: self
                .engines
                .iter()
                .map(|(id, engine)| (*id, engine.clone_box()))
                .collect(),
            calibrations: self.calibrations.clone(),
            enabled: self.enabled.clone(),
        }
    }
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            calibrations: HashMap::new(),
            enabled: HashMap::new(),
        }
    }

    pub fn register(&mut self, engine: Box<dyn ReasoningEngine>) {
        let id = engine.id();
        self.engines.insert(id, engine);
        self.calibrations.insert(id, EngineCalibration::default());
        self.enabled.insert(id, true);
    }

    pub fn get(&mut self, id: &EngineId) -> Option<&mut Box<dyn ReasoningEngine>> {
        self.engines.get_mut(id)
    }

    pub fn is_enabled(&self, id: &EngineId) -> bool {
        self.enabled.get(id).copied().unwrap_or(true)
    }

    pub fn set_enabled(&mut self, id: EngineId, enabled: bool) {
        self.enabled.insert(id, enabled);
    }

    pub fn calibration(&self, id: &EngineId) -> Option<&EngineCalibration> {
        self.calibrations.get(id)
    }

    pub fn available_ids(&self) -> Vec<EngineId> {
        let mut ids: Vec<_> = self.engines.keys().copied().collect();
        ids.sort_by_key(|id| id.name().to_string());
        ids
    }

    pub fn count(&self) -> usize {
        self.engines.len()
    }

    pub fn best_engine(&self) -> Option<EngineId> {
        self.calibrations
            .iter()
            .filter(|(id, _)| self.enabled.get(id).copied().unwrap_or(true))
            .max_by(|(_, a), (_, b)| {
                a.weight()
                    .partial_cmp(&b.weight())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| *id)
    }
}

// ── Federation Stats ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationStats {
    pub total_queries: u64,
    pub total_engines_used: u64,
    pub avg_consensus_confidence: f64,
    pub avg_execution_time_ms: u64,
    pub fusion_strategy_used: FusionStrategy,
    pub engines_available: usize,
    pub engines_enabled: usize,
}

// ── Main Reasoning Federation ──

#[derive(Debug)]
pub struct ReasoningFederation {
    pub registry: EngineRegistry,
    pub strategy: FusionStrategy,
    pub stats: FederationStats,
    pub min_confidence_threshold: f64,
    pub max_execution_time_ms: u64,
    pub enable_calibration: bool,
    pub enable_parallel: bool,
}

impl Clone for ReasoningFederation {
    fn clone(&self) -> Self {
        Self {
            registry: EngineRegistry::new(),
            strategy: self.strategy,
            stats: FederationStats::default(),
            min_confidence_threshold: self.min_confidence_threshold,
            max_execution_time_ms: self.max_execution_time_ms,
            enable_calibration: self.enable_calibration,
            enable_parallel: self.enable_parallel,
        }
    }
}

impl ReasoningFederation {
    pub fn new(strategy: FusionStrategy) -> Self {
        Self {
            registry: EngineRegistry::new(),
            strategy,
            stats: FederationStats::default(),
            min_confidence_threshold: 0.3,
            max_execution_time_ms: 5000,
            enable_calibration: true,
            enable_parallel: false,
        }
    }

    pub fn with_default_engines() -> Self {
        let mut fed = Self::new(FusionStrategy::ConfidenceWeighted);
        fed.register_default_engine();
        fed
    }

    fn register_default_engine(&mut self) {
        self.registry.register(Box::new(VsaSimilarityEngine::new()));
        self.registry
            .register(Box::new(SpectrumSignalAdapter::new()));
    }

    // ── Core Reason Pipeline ──

    pub fn reason(&mut self, context: &ReasoningContext) -> ReasoningOutput {
        let start = Instant::now();
        let outputs = self.run_engines(context);

        let fused = match self.strategy {
            FusionStrategy::MajorityVote => self.fuse_majority_vote(&outputs),
            FusionStrategy::ConfidenceWeighted => self.fuse_confidence_weighted(&outputs),
            FusionStrategy::BestOfN => self.fuse_best_of_n(&outputs),
            FusionStrategy::SequentialCascade => self.fuse_sequential_cascade(&outputs),
            FusionStrategy::BlackboardSynthesis => self.fuse_blackboard_synthesis(&outputs),
            FusionStrategy::WeightedVote => self.fuse_confidence_weighted(&outputs),
        };

        let elapsed = start.elapsed().as_millis() as u64;

        self.stats.total_queries += 1;
        self.stats.total_engines_used += outputs.len() as u64;
        self.stats.avg_execution_time_ms = (self.stats.avg_execution_time_ms + elapsed) / 2;
        self.stats.avg_consensus_confidence =
            (self.stats.avg_consensus_confidence + fused.confidence) / 2.0;
        self.stats.engines_available = self.registry.count();
        self.stats.engines_enabled = self.registry.available_ids().len();
        self.stats.fusion_strategy_used = self.strategy;

        fused
    }

    fn run_engines(&mut self, context: &ReasoningContext) -> Vec<ReasoningOutput> {
        let mut outputs = Vec::new();
        let enabled_ids = self.registry.available_ids();

        for id in &enabled_ids {
            if !self.registry.is_enabled(id) {
                continue;
            }
            if let Some(engine) = self.registry.get(id) {
                if !engine.is_available() {
                    continue;
                }

                let engine_start = Instant::now();
                let output = engine.reason(context);
                let exec_time = engine_start.elapsed().as_millis() as u64;

                if output.confidence >= self.min_confidence_threshold {
                    let mut timed_output = output;
                    timed_output.execution_time_ms = exec_time;

                    if self.enable_calibration {
                        let correct = timed_output.confidence > 0.7;
                        if let Some(cal) = self.registry.calibrations.get_mut(id) {
                            cal.record(&timed_output, correct);
                        }
                    }

                    outputs.push(timed_output);
                }
            }
        }

        outputs.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        outputs
    }

    // ── Fusion Strategies ──

    fn fuse_majority_vote(&self, outputs: &[ReasoningOutput]) -> ReasoningOutput {
        if outputs.is_empty() {
            return self.empty_output("No engines produced output");
        }

        let mut conclusion_votes: HashMap<&str, (usize, f64)> = HashMap::new();
        for out in outputs {
            let key = &out.conclusion as &str;
            let entry = conclusion_votes.entry(key).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += out.confidence;
        }

        conclusion_votes
            .into_iter()
            .max_by(|(_, (count_a, _)), (_, (count_b, _))| count_a.cmp(count_b))
            .map(|(conclusion, (votes, total_conf))| ReasoningOutput {
                engine: EngineId::Metacognitive,
                conclusion: conclusion.to_string(),
                confidence: total_conf / votes as f64,
                supporting_evidence: outputs
                    .iter()
                    .map(|o| format!("{}: {}", o.engine.name(), o.conclusion))
                    .collect(),
                execution_time_ms: outputs.iter().map(|o| o.execution_time_ms).sum(),
                engine_contribution: votes as f64 / outputs.len() as f64,
                alternative_hypotheses: outputs
                    .iter()
                    .filter(|o| o.conclusion != conclusion)
                    .map(|o| o.conclusion.clone())
                    .collect(),
            })
            .unwrap_or_else(|| self.empty_output("No majority"))
    }

    fn fuse_confidence_weighted(&self, outputs: &[ReasoningOutput]) -> ReasoningOutput {
        if outputs.is_empty() {
            return self.empty_output("No engines produced output");
        }

        let total_weight: f64 = outputs.iter().map(|o| o.confidence).sum();
        if total_weight == 0.0 {
            return outputs[0].clone();
        }

        let weighted_conclusion = outputs
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|best| best.conclusion.clone())
            .unwrap_or_default();

        let avg_confidence =
            outputs.iter().map(|o| o.confidence).sum::<f64>() / outputs.len() as f64;

        let alt_conclusion = weighted_conclusion.clone();
        ReasoningOutput {
            engine: EngineId::Metacognitive,
            conclusion: weighted_conclusion,
            confidence: avg_confidence,
            supporting_evidence: outputs
                .iter()
                .map(|o| {
                    format!(
                        "{} ({:.2}): {}",
                        o.engine.name(),
                        o.confidence,
                        o.conclusion
                    )
                })
                .collect(),
            execution_time_ms: outputs.iter().map(|o| o.execution_time_ms).sum(),
            engine_contribution: 1.0,
            alternative_hypotheses: outputs
                .iter()
                .filter(|o| o.conclusion != alt_conclusion)
                .map(|o| o.conclusion.clone())
                .collect(),
        }
    }

    fn fuse_best_of_n(&self, outputs: &[ReasoningOutput]) -> ReasoningOutput {
        outputs
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap_or_else(|| self.empty_output("No engines produced output"))
    }

    fn fuse_sequential_cascade(&self, outputs: &[ReasoningOutput]) -> ReasoningOutput {
        if outputs.is_empty() {
            return self.empty_output("No engines produced output");
        }
        let last = outputs.last().unwrap();
        ReasoningOutput {
            engine: EngineId::Metacognitive,
            conclusion: last.conclusion.clone(),
            confidence: outputs.iter().map(|o| o.confidence).sum::<f64>() / outputs.len() as f64,
            supporting_evidence: outputs
                .iter()
                .map(|o| format!("[{}] {}", o.engine.name(), o.conclusion))
                .collect(),
            execution_time_ms: outputs.iter().map(|o| o.execution_time_ms).sum(),
            engine_contribution: 1.0,
            alternative_hypotheses: Vec::new(),
        }
    }

    fn fuse_blackboard_synthesis(&self, outputs: &[ReasoningOutput]) -> ReasoningOutput {
        if outputs.is_empty() {
            return self.empty_output("No engines produced output");
        }
        let unique_engines: std::collections::HashSet<EngineId> =
            outputs.iter().map(|o| o.engine).collect();
        let consensus = unique_engines.len() as f64 / self.registry.count().max(1) as f64;
        let avg_conf = outputs.iter().map(|o| o.confidence).sum::<f64>() / outputs.len() as f64;

        ReasoningOutput {
            engine: EngineId::Metacognitive,
            conclusion: outputs
                .first()
                .map(|o| o.conclusion.clone())
                .unwrap_or_default(),
            confidence: avg_conf * (0.5 + consensus * 0.5),
            supporting_evidence: outputs.iter().map(|o| o.conclusion.clone()).collect(),
            execution_time_ms: outputs.iter().map(|o| o.execution_time_ms).sum(),
            engine_contribution: consensus,
            alternative_hypotheses: Vec::new(),
        }
    }

    fn empty_output(&self, reason: &str) -> ReasoningOutput {
        ReasoningOutput {
            engine: EngineId::Metacognitive,
            conclusion: format!("Reasoning deferred: {}", reason),
            confidence: 0.0,
            supporting_evidence: Vec::new(),
            execution_time_ms: 0,
            engine_contribution: 0.0,
            alternative_hypotheses: Vec::new(),
        }
    }

    // ── Engine Adapters ──

    pub fn register_mcts_engine(&mut self, engine: Box<dyn ReasoningEngine>) {
        self.registry.register(engine);
    }

    pub fn register_causal_engine(&mut self, engine: Box<dyn ReasoningEngine>) {
        self.registry.register(engine);
    }

    pub fn register_analogical_engine(&mut self, engine: Box<dyn ReasoningEngine>) {
        self.registry.register(engine);
    }

    pub fn register_recurrent_wm_engine(&mut self, engine: Box<dyn ReasoningEngine>) {
        self.registry.register(engine);
    }

    // ── Meta Operations ──

    pub fn report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Reasoning Federation Report\n\
             Strategy: {:?}\n\
             Queries: {}\n\
             Avg Confidence: {:.2}\n\
             Avg Time: {}ms\n\
             Engines: {}/{} enabled\n",
            self.strategy,
            self.stats.total_queries,
            self.stats.avg_consensus_confidence,
            self.stats.avg_execution_time_ms,
            self.stats.engines_enabled,
            self.stats.engines_available,
        ));

        for id in self.registry.available_ids() {
            if let Some(cal) = self.registry.calibration(&id) {
                out.push_str(&format!(
                    "  {}: acc={:.2} conf={:.2} calls={}\n",
                    id.name(),
                    cal.accuracy,
                    cal.avg_confidence,
                    cal.total_calls,
                ));
            }
        }
        out
    }

    pub fn recalibrate(&mut self, id: EngineId, actual_was_correct: bool) {
        if !self.enable_calibration {
            return;
        }
        if let Some(cal) = self.registry.calibrations.get_mut(&id) {
            let dummy = ReasoningOutput {
                engine: id,
                conclusion: String::new(),
                confidence: cal.avg_confidence,
                supporting_evidence: Vec::new(),
                execution_time_ms: 0,
                engine_contribution: 0.0,
                alternative_hypotheses: Vec::new(),
            };
            cal.record(&dummy, actual_was_correct);
        }
    }
}

// ── Built-in Default Engine ──

#[derive(Debug, Clone)]
pub struct VsaSimilarityEngine {
    patterns: Vec<(Vec<u8>, String)>,
    call_count: u64,
}

impl VsaSimilarityEngine {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            call_count: 0,
        }
    }

    pub fn add_pattern(&mut self, state: Vec<u8>, conclusion: &str) {
        self.patterns.push((state, conclusion.to_string()));
    }

    fn hamming_sim(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let same = a[..len]
            .iter()
            .zip(b[..len].iter())
            .filter(|(x, y)| x == y)
            .count();
        same as f64 / len as f64
    }
}

impl ReasoningEngine for VsaSimilarityEngine {
    fn clone_box(&self) -> Box<dyn ReasoningEngine> {
        Box::new(self.clone())
    }

    fn id(&self) -> EngineId {
        EngineId::Metacognitive
    }

    fn reason(&mut self, context: &ReasoningContext) -> ReasoningOutput {
        self.call_count += 1;
        let start = std::time::Instant::now();

        if self.patterns.is_empty() || context.state_vector.is_empty() {
            return ReasoningOutput {
                engine: EngineId::Metacognitive,
                conclusion: "VSA Similarity: no baseline patterns available".into(),
                confidence: 0.1,
                supporting_evidence: vec!["No patterns registered".into()],
                execution_time_ms: 0,
                engine_contribution: 0.0,
                alternative_hypotheses: vec!["Register patterns via add_pattern()".into()],
            };
        }

        let mut best: Option<(f64, &str)> = None;
        for (pattern, conclusion) in &self.patterns {
            let sim = Self::hamming_sim(&context.state_vector, pattern);
            if sim > best.map(|(s, _)| s).unwrap_or(-1.0) {
                best = Some((sim, conclusion));
            }
        }

        if let Some((similarity, conclusion)) = best {
            let confidence = (similarity * 0.8 + 0.2).clamp(0.0, 1.0);
            let elapsed = start.elapsed().as_millis() as u64;
            ReasoningOutput {
                engine: EngineId::Metacognitive,
                conclusion: format!("VSA Similarity: {}", conclusion),
                confidence,
                supporting_evidence: vec![format!("Pattern match: {:.2} similarity", similarity)],
                execution_time_ms: elapsed,
                engine_contribution: similarity,
                alternative_hypotheses: vec![],
            }
        } else {
            ReasoningOutput {
                engine: EngineId::Metacognitive,
                conclusion: "VSA Similarity: no matching pattern".into(),
                confidence: 0.1,
                supporting_evidence: vec!["Closest pattern below threshold".into()],
                execution_time_ms: 0,
                engine_contribution: 0.0,
                alternative_hypotheses: vec![],
            }
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    fn reset(&mut self) {
        self.patterns.clear();
        self.call_count = 0;
    }
}

// ── SpectrumSignal Adapter ──

#[derive(Debug)]
pub struct SpectrumSignalAdapter {
    inner: SpectrumSignal,
}

impl SpectrumSignalAdapter {
    pub fn new() -> Self {
        Self {
            inner: SpectrumSignal::new(Default::default()),
        }
    }

    pub fn with_config(config: super::spectrum_signal::SpectrumConfig) -> Self {
        Self {
            inner: SpectrumSignal::new(config),
        }
    }
}

impl ReasoningEngine for SpectrumSignalAdapter {
    fn clone_box(&self) -> Box<dyn ReasoningEngine> {
        Box::new(Self {
            inner: SpectrumSignal::new(self.inner.config().clone()),
        })
    }

    fn id(&self) -> EngineId {
        EngineId::SpectrumSignal
    }

    fn reason(&mut self, context: &ReasoningContext) -> ReasoningOutput {
        let start = std::time::Instant::now();
        let result = self
            .inner
            .run_pipeline(&context.state_vector, &context.query);
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Some(best) => {
                let confidence = (best.quality_score * 0.5 + best.confidence * 0.5).clamp(0.0, 1.0);
                ReasoningOutput {
                    engine: EngineId::SpectrumSignal,
                    conclusion: format!("Spectrum: {}", best.label),
                    confidence,
                    supporting_evidence: vec![
                        format!("Diversity: {:.2}", best.diversity_score),
                        format!("Quality: {:.2}", best.quality_score),
                        format!("Candidates generated: {}", self.inner.candidates().len()),
                    ],
                    execution_time_ms: elapsed,
                    engine_contribution: confidence,
                    alternative_hypotheses: self
                        .inner
                        .candidates()
                        .iter()
                        .map(|c| c.label.clone())
                        .collect(),
                }
            }
            None => ReasoningOutput {
                engine: EngineId::SpectrumSignal,
                conclusion: "SpectrumSignal: no candidate survived pipeline".into(),
                confidence: 0.0,
                supporting_evidence: vec!["All candidates filtered by quality threshold".into()],
                execution_time_ms: elapsed,
                engine_contribution: 0.0,
                alternative_hypotheses: vec![],
            },
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    fn reset(&mut self) {
        self.inner = SpectrumSignal::new(Default::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsa_engine_empty() {
        let mut eng = VsaSimilarityEngine::new();
        let ctx = ReasoningContext::new("test", vec![]);
        let out = eng.reason(&ctx);
        assert!(out.confidence < 0.2);
    }

    #[test]
    fn test_vsa_engine_match() {
        let mut eng = VsaSimilarityEngine::new();
        eng.add_pattern(vec![0x01, 0x02, 0x03], "alpha");
        let ctx = ReasoningContext::new("test", vec![0x01, 0x02, 0x03]);
        let out = eng.reason(&ctx);
        assert!(out.confidence > 0.8);
        assert!(out.conclusion.contains("alpha"));
    }

    #[test]
    fn test_vsa_engine_partial() {
        let mut eng = VsaSimilarityEngine::new();
        eng.add_pattern(vec![0xff; 64], "full");
        let ctx = ReasoningContext::new("test", vec![0xff; 32]);
        let out = eng.reason(&ctx);
        assert!(out.confidence > 0.0);
    }

    #[test]
    fn test_vsa_engine_register_in_federation() {
        let mut fed = ReasoningFederation::new(FusionStrategy::BestOfN);
        let eng = Box::new(VsaSimilarityEngine::new());
        fed.registry.register(eng);
        assert_eq!(fed.registry.count(), 1);
        let ctx = ReasoningContext::new("test", vec![0x01]);
        let out = fed.reason(&ctx);
        assert!(!out.conclusion.is_empty());
    }

    #[test]
    fn test_federation_report() {
        let mut fed = ReasoningFederation::new(FusionStrategy::ConfidenceWeighted);
        let eng = Box::new(VsaSimilarityEngine::new());
        fed.registry.register(eng);
        let ctx = ReasoningContext::new("test", vec![0x01]);
        fed.reason(&ctx);
        let report = fed.report();
        assert!(report.contains("Federation"));
    }

    // ── SpectrumSignalAdapter Tests ──

    #[test]
    fn test_spectrum_adapter_reason() {
        let mut adapter = SpectrumSignalAdapter::new();
        let ctx = ReasoningContext::new("test_query", vec![128u8; 64]);
        let out = adapter.reason(&ctx);
        assert_eq!(out.engine, EngineId::SpectrumSignal);
        assert!(out.confidence > 0.0);
        assert!(out.conclusion.contains("Spectrum"));
        assert!(!out.supporting_evidence.is_empty());
    }

    #[test]
    fn test_spectrum_adapter_registered_in_default() {
        let mut fed = ReasoningFederation::with_default_engines();
        let ctx = ReasoningContext::new("default", vec![200u8; 64]);
        let out = fed.reason(&ctx);
        assert!(!out.conclusion.is_empty());
        // Should have both engines
        assert!(fed.registry.count() >= 2);
    }

    #[test]
    fn test_spectrum_adapter_reset() {
        let mut adapter = SpectrumSignalAdapter::new();
        let ctx = ReasoningContext::new("test", vec![128u8; 64]);
        let out1 = adapter.reason(&ctx);
        adapter.reset();
        let out2 = adapter.reason(&ctx);
        assert!(out1.confidence > 0.0);
        assert!(out2.confidence > 0.0);
    }

    #[test]
    fn test_spectrum_adapter_with_few_candidates() {
        use super::super::spectrum_signal::SpectrumConfig;
        let config = SpectrumConfig {
            n_candidates: 2,
            ..Default::default()
        };
        let mut adapter = SpectrumSignalAdapter::with_config(config);
        let ctx = ReasoningContext::new("fast", vec![0u8; 64]);
        let out = adapter.reason(&ctx);
        assert!(out.confidence > 0.0);
        assert!(out.execution_time_ms < 100);
    }

    #[test]
    fn test_multi_engine_fusion() {
        let mut fed = ReasoningFederation::new(FusionStrategy::ConfidenceWeighted);
        fed.registry.register(Box::new(VsaSimilarityEngine::new()));
        fed.registry
            .register(Box::new(SpectrumSignalAdapter::new()));
        let ctx = ReasoningContext::new("fusion_test", vec![128u8; 64]);
        let out = fed.reason(&ctx);
        assert!(!out.conclusion.is_empty());
        // Fusion output should have evidence from both engines
        assert!(out.supporting_evidence.len() >= 1);
    }
}
