//! Cross-layer shared types — L1→L0 abstraction layer.
//!
//! Contains types that break circular dependencies between layers.
//! Observer traits, self-model traits, and review gate traits are defined here
//! so L5 can depend on L0 instead of L6.

use std::collections::HashMap;

use crate::l0_substrate::nt_core_hex::FullReasoningState;
pub use crate::l0_substrate::nt_core_substrate_types::E8TransitionMatrix;

// ─── Shared Constants ────────────────────────────────────────────────────────

/// Kernel dimension for reasoning vectors.
pub const KERNEL_DIM: usize = 64;

/// Shared vector type alias.
pub type Vector = Vec<f64>;

// ─── Hive Types ──────────────────────────────────────────────────────────────

/// Message type for inter-agent communication.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MessageType {
    Task,
    Query,
    Status,
    Escalation,
    Result,
    Broadcast,
}

/// A message between hive agents.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HiveMessage {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: u64,
    pub priority: u8,
    pub correlation_id: Option<String>,
    pub expires_at: Option<u64>,
}

/// Event type for the hive event log.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EventType {
    MessageDelivered,
    MessageSent,
    AgentRegistered,
    AgentUnregistered,
    BlackboardUpdated,
}

/// An event in the hive event log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HiveEvent {
    pub id: String,
    pub event_type: EventType,
    pub agent_id: Option<String>,
    pub message_id: Option<String>,
    pub detail: String,
    pub timestamp: u64,
}

// ─── Revertible Context Types ────────────────────────────────────────────────

/// A single reversible effect (forward + inverse closures).
pub struct ClosureEffect<S> {
    pub name: String,
    forward: Box<dyn FnOnce(&mut S)>,
    inverse: Box<dyn FnOnce(&mut S)>,
}

impl<S> ClosureEffect<S> {
    pub fn new(
        name: impl Into<String>,
        forward: impl FnOnce(&mut S) + 'static,
        inverse: impl FnOnce(&mut S) + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            forward: Box::new(forward),
            inverse: Box::new(inverse),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Helper to create a ClosureEffect with named forward/inverse closures.
pub fn add_effect<S>(
    name: impl Into<String>,
    forward: impl FnOnce(&mut S) + 'static,
    inverse: impl FnOnce(&mut S) + 'static,
) -> ClosureEffect<S> {
    ClosureEffect::new(name, forward, inverse)
}

/// A stored effect entry: name + inverse closure (forward already executed).
struct EffectEntry<S> {
    name: String,
    inverse: Box<dyn FnOnce(&mut S)>,
}

/// Stack-based revertible context — tracks effects and supports undo.
pub struct RevertibleContext<S> {
    state: S,
    effects: Vec<EffectEntry<S>>,
}

impl<S> RevertibleContext<S> {
    pub fn new(state: S) -> Self {
        Self { state, effects: Vec::new() }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn track(&mut self, effect: ClosureEffect<S>) {
        let ClosureEffect { name, forward, inverse } = effect;
        (forward)(&mut self.state);
        self.effects.push(EffectEntry { name, inverse });
    }

    pub fn depth(&self) -> usize {
        self.effects.len()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.effects.iter().map(|e| e.name.as_str()).collect()
    }

    pub fn is_clean(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn undo_last(&mut self) -> bool {
        if let Some(entry) = self.effects.pop() {
            (entry.inverse)(&mut self.state);
            true
        } else {
            false
        }
    }

    pub fn revert_key(&mut self, key: &str) -> bool {
        if let Some(pos) = self.effects.iter().position(|e| e.name == key) {
            let entry = self.effects.remove(pos);
            (entry.inverse)(&mut self.state);
            true
        } else {
            false
        }
    }

    pub fn revert_keys(&mut self, keys: &[&str]) -> usize {
        let mut count = 0;
        for key in keys {
            if self.revert_key(key) {
                count += 1;
            }
        }
        count
    }

    pub fn recover(&mut self) {
        while self.undo_last() {}
    }
}

/// A reversible effect: label + inverse operation.
#[derive(Clone)]
pub struct RevertibleEffect {
    pub label: String,
    inverse: std::sync::Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl RevertibleEffect {
    pub fn new(
        label: impl Into<String>,
        inverse: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            inverse: std::sync::Arc::new(inverse),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn run(&self) -> Result<(), String> {
        (self.inverse)()
    }
}

// ─── Evidence Chain ──────────────────────────────────────────────────────────

/// Cryptographic evidence chain for evolution results.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EvidenceChain {
    pub warc_path: Option<String>,
    pub sha256: Option<String>,
    pub run_id: Option<String>,
    pub timestamp: u64,
    pub tool_versions: Vec<String>,
}

// ─── Memory Index Types ─────────────────────────────────────────────────────

/// Walsh-Hadamard memory index for fast approximate similarity search.
pub struct WalshMemoryIndex {
    dim: usize,
    entries: HashMap<String, Vec<f64>>,
}

impl WalshMemoryIndex {
    pub fn new() -> Self {
        Self { dim: KERNEL_DIM, entries: HashMap::new() }
    }

    /// Store a text entry with its encoded vector.
    pub fn store(&mut self, id: &str, text: &str) {
        self.entries.insert(id.to_string(), self.encode(text));
    }

    /// Remove an entry by id. Returns true if the entry existed.
    pub fn remove(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index has no stored entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Walsh-Hadamard dot product of two vectors.
    pub fn wh_dot(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Apply Walsh-Hadamard-like transform (sign-flip spread across dimensions).
    pub fn wh_transform(&self, vec: &[f64]) -> Vec<f64> {
        let dim = vec.len();
        let mut result = vec![0.0f64; dim];
        for i in 0..dim {
            let sign = if (i & 1) == 0 { 1.0 } else { -1.0 };
            result[i] = vec[(i + 1) % dim] * sign;
        }
        result
    }

    /// Inverse Walsh-Hadamard-like transform.
    pub fn wh_inverse(&self, vec: &[f64]) -> Vec<f64> {
        let dim = vec.len();
        let mut result = vec![0.0f64; dim];
        for i in 0..dim {
            // Undo `wh_transform`: value at output j carries sign(j), so the
            // value read back from j = (i - 1) mod dim must be un-signed with
            // sign(j), not sign(i) (parities differ for i >= 1).
            let j = (dim + i - 1) % dim;
            let sign = if (j & 1) == 0 { 1.0 } else { -1.0 };
            result[i] = vec[j] * sign;
        }
        result
    }

    /// Denoise a vector by re-encoding through the Walsh-Hadamard domain.
    pub fn denoise(&self, vec: &[f64]) -> Vec<f64> {
        let transformed = self.wh_transform(vec);
        let recovered = self.wh_inverse(&transformed);
        // Normalize to preserve energy
        let orig_norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        let rec_norm: f64 = recovered.iter().map(|x| x * x).sum::<f64>().sqrt();
        if rec_norm > 1e-8 && orig_norm > 1e-8 {
            let scale = orig_norm / rec_norm;
            recovered.iter().map(|x| x * scale).collect()
        } else {
            recovered
        }
    }

    /// Ratio of original-to-recovered energy after denoising (1.0 = perfect).
    pub fn recovery_ratio(original: &[f64], recovered: &[f64]) -> f64 {
        let dot: f64 = original.iter().zip(recovered.iter()).map(|(a, b)| a * b).sum();
        let orig_norm: f64 = original.iter().map(|x| x * x).sum::<f64>().sqrt();
        let rec_norm: f64 = recovered.iter().map(|x| x * x).sum::<f64>().sqrt();
        if orig_norm > 1e-8 && rec_norm > 1e-8 {
            dot / (orig_norm * rec_norm)
        } else {
            0.0
        }
    }

    /// Search for similar entries by query text, returning top-k results.
    pub fn search(&self, query: &str, k: usize) -> Vec<(f64, String)> {
        let qv = self.encode(query);
        let nq: f64 = qv.iter().map(|x| x * x).sum::<f64>().sqrt();
        if nq < 1e-8 {
            return Vec::new();
        }
        let mut scored: Vec<(f64, String)> = self.entries
            .iter()
            .map(|(id, vec): (&String, &Vec<f64>)| {
                let nv: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
                let sim = if nv > 1e-8 {
                    qv.iter().zip(vec.iter()).map(|(a, b)| a * b).sum::<f64>() / (nq * nv)
                } else {
                    0.0
                };
                (sim, id.clone())
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Encode text into a Walsh-Hadamard vector representation.
    pub fn encode(&self, text: &str) -> Vec<f64> {
        let mut vec = vec![0.0f64; self.dim];
        for (i, byte) in text.bytes().enumerate() {
            vec[i % self.dim] += byte as f64;
        }
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-8 {
            for x in vec.iter_mut() {
                *x /= norm;
            }
        }
        vec
    }
}

impl Default for WalshMemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Kronecker-structured cleanup for approximate nearest-neighbor recall.
pub struct KroneckerCleanup {
    dim: usize,
}

impl KroneckerCleanup {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Rotate a vector using a deterministic Kronecker-based transform.
    pub fn rotate(&self, vec: &[f64]) -> Vec<f64> {
        assert_eq!(vec.len(), self.dim, "vector dimension must match");
        let mut result = vec![0.0f64; self.dim];
        for i in 0..self.dim {
            let idx = (i * 7 + 3) % self.dim; // deterministic permutation
            result[i] = vec[idx];
        }
        result
    }

    /// Cleanup: rank items by similarity to query after Kronecker rotation.
    pub fn cleanup(
        &self,
        query: &[f64],
        items: &[(String, Vec<f64>)],
        k: usize,
    ) -> Vec<(String, f64)> {
        if items.is_empty() || k == 0 {
            return Vec::new();
        }
        let rq = self.rotate(query);
        let mut scored: Vec<(String, f64)> = items
            .iter()
            .map(|(name, vec)| {
                let rv = self.rotate(vec);
                let dot: f64 = rq.iter().zip(rv.iter()).map(|(a, b)| a * b).sum();
                let nq: f64 = rq.iter().map(|x| x * x).sum::<f64>().sqrt();
                let nv: f64 = rv.iter().map(|x| x * x).sum::<f64>().sqrt();
                let sim = if nq > 1e-8 && nv > 1e-8 { dot / (nq * nv) } else { 0.0 };
                (name.clone(), sim)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }
}

// ─── Task Decomposition ─────────────────────────────────────────────────────

/// A suggested subtask from decomposition analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecomposeSuggestion {
    pub subtask: String,
    pub reasoning: String,
}

// ─── CRT Time Scale ─────────────────────────────────────────────────────────

/// Multi-scale cosmological time model (盖天/浑天/宣夜).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CrtTimeScale {
    Gaitian,
    Huntian,
    Xuanye,
}

/// Ordering: Gaitian < Huntian < Xuanye (smallest to largest cosmic scale).
impl Ord for CrtTimeScale {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for CrtTimeScale {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let ord = match (self, other) {
            (CrtTimeScale::Gaitian, CrtTimeScale::Gaitian) => std::cmp::Ordering::Equal,
            (CrtTimeScale::Gaitian, _) => std::cmp::Ordering::Less,
            (CrtTimeScale::Huntian, CrtTimeScale::Gaitian) => std::cmp::Ordering::Greater,
            (CrtTimeScale::Huntian, CrtTimeScale::Huntian) => std::cmp::Ordering::Equal,
            (CrtTimeScale::Huntian, CrtTimeScale::Xuanye) => std::cmp::Ordering::Less,
            (CrtTimeScale::Xuanye, _) => std::cmp::Ordering::Greater,
        };
        Some(ord)
    }
}

impl CrtTimeScale {
    pub fn to_ticks(&self, seconds: f64) -> f64 {
        match self {
            CrtTimeScale::Gaitian => seconds,
            CrtTimeScale::Huntian => seconds / 3600.0,
            CrtTimeScale::Xuanye => seconds / 86400.0 / 30.0,
        }
    }

    /// Inverse of to_ticks: convert ticks back to seconds.
    pub fn from_ticks(&self, ticks: f64) -> f64 {
        match self {
            CrtTimeScale::Gaitian => ticks,
            CrtTimeScale::Huntian => ticks * 3600.0,
            CrtTimeScale::Xuanye => ticks * 86400.0 * 30.0,
        }
    }

    /// Full cycle duration in seconds for this time scale.
    pub fn cycle_seconds(&self) -> f64 {
        match self {
            // 1 day
            CrtTimeScale::Gaitian => 86_400.0,
            // 1 year (365.25 days)
            CrtTimeScale::Huntian => 365.25 * 86_400.0,
            // 129600 years (Shao Yong cosmic cycle)
            CrtTimeScale::Xuanye => 129_600.0 * 365.25 * 86_400.0,
        }
    }

    /// Select the best time scale for a given duration in seconds.
    pub fn for_duration(seconds: f64) -> Self {
        if seconds < 6.0 * 3600.0 {
            // < 6 hours → Gaitian
            CrtTimeScale::Gaitian
        } else if seconds < 86_400.0 * 7.0 {
            // < 1 week → Huntian
            CrtTimeScale::Huntian
        } else {
            CrtTimeScale::Xuanye
        }
    }

    /// Whether this (larger) scale subsumes the given smaller scale.
    pub fn subsumes(&self, other: &CrtTimeScale) -> bool {
        self > other
    }

    /// Hexagram bias indices for this time scale.
    /// Gaitian: 0-23, Huntian: 24-47, Xuanye: 48-63.
    pub fn to_hexagram_bias(&self) -> &'static [usize] {
        match self {
            CrtTimeScale::Gaitian => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
            CrtTimeScale::Huntian => &[24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47],
            CrtTimeScale::Xuanye => &[48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63],
        }
    }

    /// Re-evaluation interval in seconds for this time scale.
    pub fn re_eval_interval(&self) -> f64 {
        match self {
            CrtTimeScale::Gaitian => 60.0,
            CrtTimeScale::Huntian => 3600.0,
            CrtTimeScale::Xuanye => 86400.0 * 7.0,
        }
    }

    /// Maximum iterations at this time scale.
    pub fn max_iterations(&self) -> u64 {
        match self {
            CrtTimeScale::Gaitian => 100,
            CrtTimeScale::Huntian => 50,
            CrtTimeScale::Xuanye => 12,
        }
    }

    /// English label for this time scale.
    pub fn label(&self) -> &'static str {
        match self {
            CrtTimeScale::Gaitian => "Gaitian",
            CrtTimeScale::Huntian => "Huntian",
            CrtTimeScale::Xuanye => "Xuanye",
        }
    }

    /// Chinese name for this time scale.
    pub fn chinese_name(&self) -> &'static str {
        match self {
            CrtTimeScale::Gaitian => "盖天",
            CrtTimeScale::Huntian => "浑天",
            CrtTimeScale::Xuanye => "宣夜",
        }
    }
}

// ─── Observer Types ──────────────────────────────────────────────────────────

/// Observer report — produced by `ObserverTrait::analyze()`.
/// Defined in L0 so L5 can consume it without importing L6.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObserverReport {
    pub trajectory_len: usize,
    pub quality_score: f64,
    pub distinct_states: usize,
    pub patterns: Vec<String>,
    pub has_actionable_insight: bool,
    pub trajectory_weighted_score: Option<f64>,
    pub convergence_score: Option<f64>,
}

/// Observer trait — captures the interface L5 needs from the +1 observer.
/// Implemented by `OneObserver` in L6.
pub trait ObserverTrait: Send + Sync {
    fn analyze(
        &mut self,
        trajectory: &[FullReasoningState],
        keywords: &[&str],
    ) -> ObserverReport;
    fn set_transition_matrix(&mut self, matrix: E8TransitionMatrix);
    fn transition_matrix(&self) -> Option<&E8TransitionMatrix>;
}

/// Error recovery trait — captures the interface L5 needs from observer error recovery.
/// Implemented by `ObserverErrorRecovery` in L6.
pub trait ErrorRecoveryTrait: Send + Sync {}

/// Silicon self-model trait — captures the interface L5 needs from the silicon self-model.
/// Implemented by `SiliconSelfModel` in L6.
pub trait SiliconSelfTrait: Send + Sync {}

/// Consciousness gold standard trait — captures the interface L5 needs.
/// Implemented by `ConsciousnessGoldStandard` in L6.
pub trait GoldStandardTrait: Send + Sync {}

// ─── Self-Review Gate Types ─────────────────────────────────────────────────

/// Review finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A single review finding — defined in L0 so L5 can consume it without importing L6.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReviewFinding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

/// Self-review configuration — defined in L0.
#[derive(Debug, Clone)]
pub struct SelfReviewConfig {
    pub min_test_line_count: usize,
    pub scan_safety_bound: usize,
}

impl Default for SelfReviewConfig {
    fn default() -> Self {
        Self {
            min_test_line_count: 50,
            scan_safety_bound: 300,
        }
    }
}

/// Self-review gate trait — captures the interface L5 needs.
/// Implemented by `SelfReviewGate` in L6.
pub trait SelfReviewGateTrait: Send + Sync {
    fn new(strict_mode: bool) -> Self
    where
        Self: Sized;
    fn run_review(&mut self, root: &std::path::Path) -> SelfReviewReport;
}

/// Self-review report — defined in L0.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelfReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
}

impl SelfReviewReport {
    pub fn is_pass(&self) -> bool {
        self.failed == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Self-review: {} passed, {} failed, {} warnings — overall {}",
            self.passed,
            self.failed,
            self.warnings,
            if self.is_pass() { "PASS" } else { "FAIL" }
        )
    }
}

// NOTE: L6 concrete type re-exports have been moved to l5_cognition::l1_facade.
// L0 should not depend on L6 — those re-exports belong in L5's facade.

// ─── Dependency Confidence (P1-05 / SIM-27) ────────────────────────────────────
// Provenance labels for cross-module edges (graphify confidence pattern).
// Critical paths MUST carry zero AMBIGUOUS edges (see ConfidenceLabelFitness).

/// How a cross-module dependency was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Confidence {
    /// Explicit in source: direct `use`, trait impl, direct call, event subscription.
    Extracted,
    /// Reasonable deduction: shared types, runtime patterns, inferred data flow.
    Inferred,
    /// Uncertain: potential cycle, unclear ownership, feature-gated path. Flag for review.
    Ambiguous,
}

/// The mechanism of a cross-module dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DependencyKind {
    Import,
    TraitImpl,
    Call,
    Event,
    SharedType,
}

/// Where a dependency was observed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
}

/// A single labeled edge between two modules.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LabeledDependency {
    pub source: String,
    pub target: String,
    pub kind: DependencyKind,
    pub confidence: Confidence,
    pub location: Option<SourceLocation>,
    pub rationale: Option<String>,
}

impl LabeledDependency {
    /// True when this edge needs human review before it may sit on a critical path.
    pub fn needs_review(&self) -> bool {
        self.confidence == Confidence::Ambiguous
    }
}

/// Collect the ambiguous edges out of a dependency set.
/// Used by critical-path gates: a non-empty return blocks the path.
pub fn ambiguous_edges(deps: &[LabeledDependency]) -> Vec<&LabeledDependency> {
    deps.iter().filter(|d| d.needs_review()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hive_message_serde_roundtrip() {
        let msg = HiveMessage {
            id: "m1".into(),
            from: "agent-a".into(),
            to: Some("agent-b".into()),
            content: "hello".into(),
            msg_type: MessageType::Task,
            timestamp: 1000,
            priority: 5,
            correlation_id: None,
            expires_at: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: HiveMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "m1");
        assert_eq!(back.msg_type, MessageType::Task);
    }

    #[test]
    fn hive_event_serde_roundtrip() {
        let ev = HiveEvent {
            id: "e1".into(),
            event_type: EventType::AgentRegistered,
            agent_id: Some("a1".into()),
            message_id: None,
            detail: "registered".into(),
            timestamp: 2000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: HiveEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, EventType::AgentRegistered);
    }

    #[test]
    fn closure_effect_name() {
        let eff = ClosureEffect::new("inc", |s: &mut i32| *s += 1, |s: &mut i32| *s -= 1);
        assert_eq!(eff.name(), "inc");
    }

    #[test]
    fn revertible_context_track_and_undo() {
        let mut ctx = RevertibleContext::new(10i32);
        assert!(ctx.is_clean());
        ctx.track(ClosureEffect::new("inc", |s| *s += 5, |s| *s -= 5));
        assert_eq!(*ctx.state(), 15);
        assert_eq!(ctx.depth(), 1);
        assert!(!ctx.is_clean());
        ctx.undo_last();
        assert_eq!(*ctx.state(), 10);
        assert!(ctx.is_clean());
    }

    #[test]
    fn revertible_context_revert_key() {
        let mut ctx = RevertibleContext::new(0i32);
        ctx.track(ClosureEffect::new("a", |s| *s += 10, |s| *s -= 10));
        ctx.track(ClosureEffect::new("b", |s| *s += 20, |s| *s -= 20));
        assert_eq!(*ctx.state(), 30);
        assert!(ctx.revert_key("a"));
        assert_eq!(*ctx.state(), 20);
        assert!(!ctx.revert_key("nonexistent"));
    }

    #[test]
    fn revertible_context_recover() {
        let mut ctx = RevertibleContext::new(0i32);
        ctx.track(ClosureEffect::new("a", |s| *s += 1, |s| *s -= 1));
        ctx.track(ClosureEffect::new("b", |s| *s += 2, |s| *s -= 2));
        ctx.track(ClosureEffect::new("c", |s| *s += 3, |s| *s -= 3));
        ctx.recover();
        assert_eq!(*ctx.state(), 0);
        assert!(ctx.is_clean());
    }

    #[test]
    fn revertible_context_keys() {
        let mut ctx = RevertibleContext::new(());
        ctx.track(ClosureEffect::new("x", |_| {}, |_| {}));
        ctx.track(ClosureEffect::new("y", |_| {}, |_| {}));
        assert_eq!(ctx.keys(), vec!["x", "y"]);
    }

    #[test]
    fn revertible_effect_label_and_run() {
        let eff = RevertibleEffect::new("undo_op", || Ok(()));
        assert_eq!(eff.label(), "undo_op");
        assert!(eff.run().is_ok());
    }

    #[test]
    fn evidence_chain_default() {
        let ec = EvidenceChain::default();
        assert_eq!(ec.timestamp, 0);
        assert!(ec.tool_versions.is_empty());
    }

    #[test]
    fn walsh_index_store_and_search() {
        let mut idx = WalshMemoryIndex::new();
        assert!(idx.is_empty());
        idx.store("a", "hello world");
        idx.store("b", "hello rust");
        assert_eq!(idx.len(), 2);
        assert!(!idx.is_empty());
        let results = idx.search("hello", 10);
        assert_eq!(results.len(), 2);
        assert!(results[0].0 > 0.0);
    }

    #[test]
    fn walsh_index_remove() {
        let mut idx = WalshMemoryIndex::new();
        idx.store("a", "test");
        assert!(idx.remove("a"));
        assert!(!idx.remove("nonexistent"));
        assert!(idx.is_empty());
    }

    #[test]
    fn walsh_wh_dot() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((WalshMemoryIndex::wh_dot(&a, &b) - 32.0).abs() < 1e-10);
    }

    #[test]
    fn walsh_transform_inverse_roundtrip() {
        let idx = WalshMemoryIndex::new();
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let t = idx.wh_transform(&v);
        let recovered = idx.wh_inverse(&t);
        for (orig, rec) in v.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 1e-10);
        }
    }

    #[test]
    fn walsh_denoise_preserves_energy() {
        let idx = WalshMemoryIndex::new();
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let denoised = idx.denoise(&v);
        let ratio = WalshMemoryIndex::recovery_ratio(&v, &denoised);
        assert!(ratio > 0.99, "recovery ratio {}", ratio);
    }

    #[test]
    fn kronecker_cleanup_dim() {
        let kc = KroneckerCleanup::new(8);
        assert_eq!(kc.dim(), 8);
    }

    #[test]
    fn kronecker_cleanup_rotate() {
        let kc = KroneckerCleanup::new(4);
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let r = kc.rotate(&v);
        assert_eq!(r.len(), 4);
        assert_ne!(r, v);
    }

    #[test]
    fn kronecker_cleanup_empty_items() {
        let kc = KroneckerCleanup::new(4);
        let q = vec![1.0, 2.0, 3.0, 4.0];
        let items: Vec<(String, Vec<f64>)> = vec![];
        assert!(kc.cleanup(&q, &items, 5).is_empty());
    }

    #[test]
    fn crt_time_scale_ordering() {
        assert!(CrtTimeScale::Gaitian < CrtTimeScale::Huntian);
        assert!(CrtTimeScale::Huntian < CrtTimeScale::Xuanye);
        assert!(CrtTimeScale::Gaitian < CrtTimeScale::Xuanye);
    }

    #[test]
    fn crt_time_scale_to_ticks_from_ticks_roundtrip() {
        let secs = 7200.0;
        let ticks = CrtTimeScale::Huntian.to_ticks(secs);
        let back = CrtTimeScale::Huntian.from_ticks(ticks);
        assert!((back - secs).abs() < 1e-6);
    }

    #[test]
    fn crt_time_scale_for_duration() {
        assert_eq!(CrtTimeScale::for_duration(100.0), CrtTimeScale::Gaitian);
        assert_eq!(CrtTimeScale::for_duration(25000.0), CrtTimeScale::Huntian);
        assert_eq!(CrtTimeScale::for_duration(1_000_000.0), CrtTimeScale::Xuanye);
    }

    #[test]
    fn crt_time_scale_labels() {
        assert_eq!(CrtTimeScale::Gaitian.label(), "Gaitian");
        assert_eq!(CrtTimeScale::Huntian.chinese_name(), "浑天");
        assert_eq!(CrtTimeScale::Xuanye.to_hexagram_bias().len(), 16);
    }

    #[test]
    fn crt_time_scale_subsumes() {
        assert!(CrtTimeScale::Xuanye.subsumes(&CrtTimeScale::Gaitian));
        assert!(!CrtTimeScale::Gaitian.subsumes(&CrtTimeScale::Huntian));
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn self_review_report_is_pass() {
        let r = SelfReviewReport {
            findings: vec![],
            passed: 10,
            failed: 0,
            warnings: 2,
        };
        assert!(r.is_pass());
        assert!(r.summary().contains("PASS"));
    }

    #[test]
    fn self_review_report_is_fail() {
        let r = SelfReviewReport {
            findings: vec![],
            passed: 10,
            failed: 3,
            warnings: 0,
        };
        assert!(!r.is_pass());
        assert!(r.summary().contains("FAIL"));
    }

    #[test]
    fn self_review_config_default() {
        let cfg = SelfReviewConfig::default();
        assert_eq!(cfg.min_test_line_count, 50);
        assert_eq!(cfg.scan_safety_bound, 300);
    }

    #[test]
    fn kernel_dim() {
        assert_eq!(KERNEL_DIM, 64);
    }

    #[test]
    fn decompose_suggestion_serde() {
        let s = DecomposeSuggestion {
            subtask: "step1".into(),
            reasoning: "because".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: DecomposeSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subtask, "step1");
    }

    #[test]
    fn confidence_ambiguous_needs_review() {
        let mk = |confidence| LabeledDependency {
            source: "l1_action::nt_io".into(),
            target: "l2_perception::nt_world".into(),
            kind: DependencyKind::Import,
            confidence,
            location: None,
            rationale: None,
        };
        assert!(!mk(Confidence::Extracted).needs_review());
        assert!(!mk(Confidence::Inferred).needs_review());
        assert!(mk(Confidence::Ambiguous).needs_review());
    }

    #[test]
    fn ambiguous_edges_filters_critical_path() {
        let deps = vec![
            LabeledDependency {
                source: "a".into(),
                target: "b".into(),
                kind: DependencyKind::Call,
                confidence: Confidence::Extracted,
                location: Some(SourceLocation {
                    file: "a.rs".into(),
                    line: 10,
                }),
                rationale: None,
            },
            LabeledDependency {
                source: "b".into(),
                target: "c".into(),
                kind: DependencyKind::SharedType,
                confidence: Confidence::Ambiguous,
                location: None,
                rationale: Some("ownership unclear".into()),
            },
        ];
        let bad = ambiguous_edges(&deps);
        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].target, "c");
    }

    #[test]
    fn labeled_dependency_serde_roundtrip() {
        let d = LabeledDependency {
            source: "l1".into(),
            target: "l0".into(),
            kind: DependencyKind::TraitImpl,
            confidence: Confidence::Inferred,
            location: Some(SourceLocation {
                file: "x.rs".into(),
                line: 1,
            }),
            rationale: Some("via shared trait".into()),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: LabeledDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }
}
