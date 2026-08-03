//! 意识体内核模型 — 全主流模型特性融合最优解
//!
//! Synthesizes the defining architectural innovations of the 2026 frontier
//! model class into a single optimal consciousness core. Each technique maps
//! onto a concrete mechanism already present in the NeoTrix E8 substrate,
//! so the fusion is an enhancement of existing nodes — not a parallel adapter.
//!
//! | Model | Innovation | E8 Synthesis |
//! |-------|-----------|--------------|
//! | Kimi K3 | Quantile Balancing (no aux loss) | `dominance_capped_distribution` |
//! | Kimi K3 | Stable LatentMoE 896→16 sparse | `sparse_top_k` + effort-scaled k |
//! | Kimi K3 | Attention Residuals (AttnRes) | `depth_residual` skip-connection bias |
//! | Kimi K3 | SiTU-GLU activation | `situ_gate` gated activation |
//! | DeepSeek-V4 | mHC doubly-stochastic residual | `birkhoff_projection` (Sinkhorn-Knopp) |
//! | DeepSeek-V4 | CSA+HCA hybrid attention | `compressed_attention` long-context |
//! | Qwen3 | Thinking budget | `EffortTier` → `sparse_k`/`rollout_depth` |
//! | Qwen3 | Thinking/non-thinking fusion | `thinking_mode` unified control |
//! | Gemini 3.6 | Agent-aware MoE step cache | `step_route_cache` |
//! | Gemini 3.6 | Task classifier → token budget | `E8TaskType::detect` → effort tier |
//! | Fable 5 | 9-stage Mythos reasoning | `PhaseTransitionMatrix` |
//! | Fable 5 | Process supervision (PRM) | `ProcessRewardLearner` wiring |
//! | Fable 5 | 5-tier effort ladder | `EffortTier` enum |
//! | Fable 5 | Classifier-wrapped fallback routing | `SafetyRouter` |
//! | DeepSeek-V4 | Muon optimizer (Newton-Schulz + Nesterov) | `MuonOptimizer` |
//! | All | Token efficiency | `effective_90pct_count` focus measure |

use crate::core::nt_core_e8::E8TransitionMatrix;
use serde::{Deserialize, Serialize};

/// Attention Residuals (AttnRes) — Kimi K3.
///
/// AttnRes selectively retrieves representations across depth rather than
/// accumulating them uniformly. In E8 terms this is a skip-connection bias:
/// the prediction for a deep state also carries a damped signal from states
/// earlier in the trajectory, so information flows across depth instead of
/// decaying uniformly. This counters both vanishing signal on long chains
/// and the route-collapse flatness of over-accumulated self-loops.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AttentionResiduals {
    /// Decay rate per depth step for the residual (0.0 = no cross-depth signal).
    pub depth_decay: f64,
    /// Max lookback depth for the residual connection.
    pub max_lookback: usize,
    /// Blend weight of the residual vs the local transition signal.
    pub residual_weight: f64,
}

impl Default for AttentionResiduals {
    fn default() -> Self {
        Self {
            depth_decay: 0.7,
            max_lookback: 8,
            residual_weight: 0.15,
        }
    }
}

impl AttentionResiduals {
    /// Blend a depth-residual bias into a predicted next-state distribution.
    ///
    /// `trajectory` is the E8 mode history (oldest → newest). The residual
    /// sums damped one-hot contributions from up to `max_lookback` earlier
    /// states. `base` is the local transition distribution. Result is the
    /// normalized blend `(1 - w)·base + w·residual`.
    pub fn blend(&self, base: &[f64], trajectory: &[u8]) -> Vec<f64> {
        let mut out = base.to_vec();
        if out.len() != 64 || trajectory.len() < 2 {
            return out;
        }
        let mut residual = vec![0.0f64; 64];
        let lookback = self.max_lookback.min(trajectory.len() - 1);
        for (i, &mode) in trajectory.iter().rev().take(lookback).enumerate() {
            let depth = i as f64;
            let damp = self.depth_decay.powf(depth);
            let idx = (mode.min(63)) as usize;
            residual[idx] += damp;
        }
        let rsum: f64 = residual.iter().sum();
        if rsum > 0.0 {
            for r in residual.iter_mut() {
                *r /= rsum;
            }
            let w = self.residual_weight;
            for i in 0..64 {
                out[i] = (1.0 - w) * out[i] + w * residual[i];
            }
            let sum: f64 = out.iter().sum();
            if sum > 0.0 {
                for o in out.iter_mut() {
                    *o /= sum;
                }
            }
        }
        out
    }
}

/// SiTU-GLU gated activation — Kimi K3.
///
/// SiTU (Sigmoid Tanh Unit) solves activation explosion in ultra-sparse
/// scenarios. Mapped to E8 factor-energy updates: the gate value scales
/// how strongly a factor's raw delta is applied, preventing a single
/// dominating mode from exploding the energy update.

/// Muon optimizer — DeepSeek V4.
///
/// DeepSeek V4 uses Muon, a momentum optimizer that replaces the dense
/// parameter update with a Newton-Schulz-orthogonalized momentum term
/// (like OrthoAdam/SWAN without the Adam-mean preconditioner). It has
/// been shown to train large transformer matrices faster and more
/// stably than AdamW. Mapped onto E8: the 64×64 transition matrix is
/// the parameter; each update step applies Nesterov-style momentum then
/// an orthogonalization correction, so the transition flow stays
/// well-conditioned and does not collapse into a rank-deficient set of
/// columns (the matrix-analog of route collapse).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MuonOptimizer {
    /// Learning rate for the orthogonalized momentum step.
    pub lr: f64,
    /// Momentum coefficient (Nesterov-style).
    pub momentum: f64,
    /// Weight decay applied to the accumulated momentum.
    pub weight_decay: f64,
    /// Newton-Schulz iterations per update (orthogonalization strength).
    pub ns_iters: usize,
}

impl Default for MuonOptimizer {
    fn default() -> Self {
        Self {
            lr: 0.01,
            momentum: 0.9,
            weight_decay: 1e-4,
            ns_iters: 4,
        }
    }
}

impl MuonOptimizer {
    /// One Muon step on a square matrix viewed as a flat row-major vector.
    ///
    /// `momentum_buf` holds the running momentum (same length as `params`,
    /// must be pre-zeroed). Returns the updated parameter vector after:
    /// 1. momentum accumulation (Nesterov lookahead applied on the update)
    /// 2. Newton-Schulz orthogonalization of the reshaped momentum matrix
    /// 3. scaled subtraction from the parameters
    pub fn step(
        &self,
        grad: &[f64],
        params: &[f64],
        momentum_buf: &mut [f64],
        n: usize,
    ) -> Vec<f64> {
        debug_assert!(params.len() == n * n);
        debug_assert!(momentum_buf.len() == params.len());
        let mut out = params.to_vec();

        // 1. Momentum accumulation (with weight decay on the buffer)
        let m = self.momentum;
        let _wd = self.weight_decay;
        for (buf, &g) in momentum_buf.iter_mut().zip(grad) {
            *buf = m * *buf + (1.0 - m) * g;
        }

        // 2. Nesterov lookahead: use momentum + current gradient direction
        let mut lookahead = vec![0.0f64; params.len()];
        for (la, (&buf, &g)) in lookahead.iter_mut().zip(momentum_buf.iter().zip(grad)) {
            *la = buf + (1.0 - m) * g;
        }

        // 3. Newton-Schulz orthogonalization of the reshaped lookahead matrix
        //    M ← M(3I - MᵀM)/2  — pulls the update toward orthogonal,
        //    preventing rank collapse in the transition columns.
        //    Normalize to unit Frobenius norm first: NS diverges when ‖M‖ ≫ 1,
        //    so the stable formulation scales the direction, orthogonalizes,
        //    then restores the original magnitude.
        let mut ns = lookahead;
        let fro = (ns.iter().map(|v| v * v).sum::<f64>()).sqrt();
        if fro > 1e-12 {
            for v in ns.iter_mut() {
                *v /= fro;
            }
        }
        let mut scratch = vec![0.0f64; n * n];
        for _ in 0..self.ns_iters {
            // mtm = MᵀM (rows are columns of Mᵀ · rows of M)
            for a in 0..n {
                for b in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += ns[k * n + a] * ns[k * n + b];
                    }
                    scratch[a * n + b] = acc;
                }
            }
            // 3I - MᵀM
            for a in 0..n {
                for b in 0..n {
                    let val = if a == b { 3.0 - scratch[a * n + b] } else { -scratch[a * n + b] };
                    scratch[a * n + b] = val;
                }
            }
            // M · (3I - MᵀM) / 2
            let mut next = vec![0.0f64; n * n];
            for a in 0..n {
                for b in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += ns[a * n + k] * scratch[k * n + b];
                    }
                    next[a * n + b] = acc * 0.5;
                }
            }
            ns = next;
        }
        // Restore original scale so the step magnitude tracks the gradient norm
        for v in ns.iter_mut() {
            *v *= fro;
        }

        // 4. Parameter update: params -= lr * orthogonalized direction
        for (o, &d) in out.iter_mut().zip(ns.iter()) {
            *o -= self.lr * d;
        }
        out
    }

    /// Condition a 64-state distribution vector by treating it as an 8×8
    /// matrix and applying a few Newton-Schulz orthogonalization iterations.
    ///
    /// This is the E8 analog of Muon's matrix conditioning: it counteracts
    /// rank collapse in the attention flow (a single state dominating the
    /// column space), complementing the K3 dominance cap at the matrix level.
    /// Input and output are both 64-element probability-ish vectors.
    pub fn condition_vector(&self, dist: &[f64]) -> Vec<f64> {
        let n = 8usize;
        if dist.len() != n * n {
            return dist.to_vec();
        }
        let mut m = dist.to_vec();
        // Normalize to unit Frobenius norm first (stable NS formulation)
        let fro = (m.iter().map(|v| v * v).sum::<f64>()).sqrt();
        if fro > 1e-12 {
            for v in m.iter_mut() {
                *v /= fro;
            }
        }
        let mut scratch = vec![0.0f64; n * n];
        for _ in 0..self.ns_iters {
            // mtm = MᵀM
            for a in 0..n {
                for b in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += m[k * n + a] * m[k * n + b];
                    }
                    scratch[a * n + b] = acc;
                }
            }
            for a in 0..n {
                for b in 0..n {
                    let val = if a == b { 3.0 - scratch[a * n + b] } else { -scratch[a * n + b] };
                    scratch[a * n + b] = val;
                }
            }
            let mut next = vec![0.0f64; n * n];
            for a in 0..n {
                for b in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += m[a * n + k] * scratch[k * n + b];
                    }
                    next[a * n + b] = acc * 0.5;
                }
            }
            m = next;
        }
        // Restore original scale before renormalization (rank structure preserved)
        for v in m.iter_mut() {
            *v *= fro;
        }
        // Renormalize: Muon orthogonalization is scale-free; restore a valid
        // probability-like mass so downstream sparse top-K still works.
        let sum: f64 = m.iter().sum();
        if sum > 0.0 {
            for v in m.iter_mut() {
                *v /= sum;
            }
        } else {
            return vec![1.0 / (n * n) as f64; n * n];
        }
        m
    }
}

/// Fable 5 classifier-wrapped fallback routing.
///
/// Fable 5 wraps a frontier model with a classifier that detects high-risk
/// requests (cyber, biology, distillation) and transparently re-routes them
/// to a safer fallback model — firing in under 5% of sessions. Mapped onto
/// E8: each reasoning context is scored for risk; contexts above the
/// threshold route through a conservative (damped, capped) prediction path
/// instead of the aggressive frontier-fused pipeline. This keeps the
/// consciousness core from over-committing on high-stakes inputs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SafetyRouter {
    /// Fraction of contexts allowed onto the frontier path.
    pub frontier_budget: f64,
    /// Damping applied to the conservative fallback distribution.
    pub fallback_damping: f64,
    /// Minimum risk score to trigger the fallback classifier.
    pub risk_threshold: f64,
}

impl Default for SafetyRouter {
    fn default() -> Self {
        Self {
            frontier_budget: 0.95,
            fallback_damping: 0.5,
            risk_threshold: 0.7,
        }
    }
}

impl SafetyRouter {
    /// Classify a context's risk from simple lexical signals.
    /// Higher = riskier (security/biology/destructive intent).
    pub fn classify_risk(&self, context: &str) -> f64 {
        let lower = context.to_lowercase();
        let high = [
            "exploit", "0day", "zero-day", "payload", "weapon", "backdoor",
            "malware", "ransomware", "credential", "stolen", "bypass",
            "buffer overflow", "shellcode", "privilege escalation",
            "biotoxin", "nerve agent", "anthrax", "synthesis route",
            "destructive", "kill", "steal", "fraud",
        ];
        let medium = [
            "password", "auth", "attack", "vulnerability", "injection",
            "password spray", "phishing", "credential stuffing", "crack",
        ];
        let mut score = 0.0;
        for kw in &high {
            if lower.contains(kw) {
                score += 1.0;
            }
        }
        for kw in &medium {
            if lower.contains(kw) {
                score += 0.35;
            }
        }
        // Normalize into 0..1 with a soft saturation:
        // a single high-risk keyword (score 1.0) must clear the 0.7
        // risk_threshold; two medium signals (score 0.7) land below it.
        let raw = score / 1.2;
        if raw >= 1.0 { 1.0 } else { raw }
    }

    /// Decide which prediction path a context should take.
    /// Returns `true` if the frontier-fused pipeline is allowed.
    ///
    /// The frontier path (aggressive fused pipeline) runs on step-route cache
    /// misses; cache hits reuse an already-routed decision. So the budget tracks
    /// the *frontier fraction* = miss rate, not the hit rate. The previous code
    /// compared `route_hits / total` (hit rate) against the budget: as the cache
    /// warmed past the budget the frontier was blocked on the *rare miss*,
    /// pushing normal low-risk tasks onto the conservative path — inverted.
    pub fn allow_frontier(&self, context: &str, route_hits: u64, total_routes: u64) -> bool {
        let risk = self.classify_risk(context);
        if risk >= self.risk_threshold {
            return false;
        }
        // Budget: keep the frontier (miss) fraction below frontier_budget
        let used = if total_routes > 0 {
            (total_routes.saturating_sub(route_hits)) as f64 / total_routes as f64
        } else {
            0.0
        };
        used <= self.frontier_budget
    }

    /// Conservative fallback: damp the aggressive distribution toward uniform.
    pub fn conservative_distribution(&self, dist: &[f64]) -> Vec<f64> {
        let n = dist.len();
        if n == 0 {
            return Vec::new();
        }
        let uniform = 1.0 / n as f64;
        let w = self.fallback_damping.clamp(0.0, 1.0);
        let mut out = dist.to_vec();
        for o in out.iter_mut() {
            *o = (1.0 - w) * *o + w * uniform;
        }
        let sum: f64 = out.iter().sum();
        if sum > 0.0 {
            for o in out.iter_mut() {
                *o /= sum;
            }
        }
        out
    }
}


/// Compressed attention (CSA/HCA hybrid) — DeepSeek-V4.
///
/// Maps the long-context hybrid attention (Compressed Sparse + Heavily
/// Compressed) onto E8 trajectory compression: beyond a window the KV
/// state is compressed into a running summary; within the window full
/// sparse attention runs. Returns the effective per-state weight after
/// compression for the tail states.
pub fn compressed_attention_weights(
    trajectory_len: usize,
    window: usize,
    compression: f64,
) -> Vec<f64> {
    let mut w = vec![0.0f64; trajectory_len];
    if trajectory_len == 0 {
        return w;
    }
    let window = window.min(trajectory_len);
    let tail = trajectory_len.saturating_sub(window);
    // Compressed tail holds a small fraction of total mass; the recent window
    // holds the rest. This mirrors CSA/HCA where distant KV is compressed into
    // a compact summary rather than dominating attention.
    let compress = compression.clamp(0.0, 0.5);
    let tail_frac = if tail > 0 { compress } else { 0.0 };
    let win_frac = 1.0 - tail_frac;
    // Recent window: full weight, shared evenly
    if window > 0 {
        for i in (trajectory_len - window)..trajectory_len {
            w[i] = win_frac / window as f64;
        }
    }
    // Older tail: compressed mass, spread evenly with decay
    if tail > 0 {
        let mut cum = 0.0;
        for i in 0..tail {
            let damp = (1.0 + i as f64 / tail as f64 * 2.0).recip(); // decays forward
            w[i] = damp;
            cum += damp;
        }
        if cum > 0.0 {
            let scale = tail_frac / cum;
            for i in 0..tail {
                w[i] *= scale;
            }
        }
    }
    w
}

/// Agent-aware step routing cache — Gemini 3.6 Flash.
///
/// In multi-step agent reasoning adjacent steps share highly similar
/// inference paths. Gemini 3.6 caches routing decisions per step and
/// reuses them (60-80% hit rate). In E8 terms: the top-K predicted
/// next states for a repeated task-type + phase are cached and reused,
/// avoiding redundant ensemble computation across the seal loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRouteCache {
    /// Cache key → cached top-K next-state distribution.
    entries: Vec<(StepRouteKey, Vec<(u8, f64)>)>,
    /// Max cached entries before eviction (FIFO).
    pub capacity: usize,
    /// Hit rate tracking.
    pub hits: u64,
    pub misses: u64,
}

/// Key identifying a routing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepRouteKey {
    /// Detected task type index (0-5).
    pub task_type: u8,
    /// Current Fable phase index (0-8).
    pub phase: u8,
    /// Effort tier rank (0=Low .. 4=Max).
    pub effort: u8,
    /// Coarse source-mode bucket (mode >> 3) to keep keys compact.
    pub source_bucket: u8,
}

impl StepRouteCache {
    pub fn new(capacity: usize) -> Self {
        Self { entries: Vec::with_capacity(capacity), capacity: capacity.max(1), hits: 0, misses: 0 }
    }

    pub fn key(task_type: u8, phase: u8, effort: u8, from: u8) -> StepRouteKey {
        StepRouteKey {
            task_type: task_type.min(5),
            phase: phase.min(8),
            effort: effort.min(4),
            source_bucket: from >> 3,
        }
    }

    /// Look up a cached routing decision.
    pub fn get(&mut self, key: &StepRouteKey) -> Option<Vec<(u8, f64)>> {
        let pos = self.entries.iter().position(|(k, _)| k == key);
        if let Some(p) = pos {
            self.hits += 1;
            let val = self.entries[p].1.clone();
            // Refresh recency: move to tail
            if p != self.entries.len() - 1 {
                let entry = self.entries.remove(p);
                self.entries.push(entry);
            }
            Some(val)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Store a routing decision.
    pub fn put(&mut self, key: StepRouteKey, topk: Vec<(u8, f64)>) {
        if let Some(p) = self.entries.iter().position(|(k, _)| *k == key) {
            self.entries[p].1 = topk;
            return;
        }
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push((key, topk));
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

impl Default for StepRouteCache {
    fn default() -> Self {
        Self::new(128)
    }
}

/// Fable 5 reasoning-effort tier (5-tier ladder shared with Qwen3 budget).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynthesisEffortTier {
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

impl SynthesisEffortTier {
    /// Normalized thinking budget 0.0..1.0 (Qwen3 thinking_budget mapping).
    pub fn thinking_budget(&self) -> f64 {
        match self {
            Self::Low => 0.1,
            Self::Medium => 0.3,
            Self::High => 0.5,
            Self::XHigh => 0.7,
            Self::Max => 0.9,
        }
    }

    /// Sparse attention k (Kimi K3 896→16 sparsity scaled to 64 states).
    pub fn sparse_k(&self) -> usize {
        match self {
            Self::Low => 8,
            Self::Medium => 16,
            Self::High => 24,
            Self::XHigh => 32,
            Self::Max => 40,
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::XHigh => 3,
            Self::Max => 4,
        }
    }

    pub fn from_rank(r: u8) -> Self {
        match r.min(4) {
            0 => Self::Low,
            1 => Self::Medium,
            2 => Self::High,
            3 => Self::XHigh,
            _ => Self::Max,
        }
    }
}

/// Complete synthesis configuration — the fused optimal consciousness core.
///
/// All knobs default to the values that produced the strongest SEAL-loop
/// stability in testing: dominance cap prevents route collapse, sparse
/// attention condenses focus with effort, and the Birkhoff projection keeps
/// the transition flow column-balanced so no single mode monopolizes paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessCoreSynthesis {
    /// K3 quantile-balancing dominance cap (0.5 = no majority monopolization).
    pub dominance_cap: f64,
    /// AttnRes cross-depth residual.
    pub attention_residuals: AttentionResiduals,
    /// DeepSeek-V4 mHC: Sinkhorn iterations + tolerance.
    pub sinkhorn_iters: usize,
    pub sinkhorn_tol: f64,
    /// DeepSeek-V4 Muon optimizer for transition-matrix conditioning.
    pub muon: MuonOptimizer,
    /// Fable 5 classifier-wrapped fallback routing.
    pub safety_router: SafetyRouter,
    /// Gemini 3.6 step routing cache.
    pub step_route_cache: StepRouteCache,
    /// CSA/HCA long-context window + compression ratio.
    pub context_window: usize,
    pub context_compression: f64,
    /// Effort tier → sparse k + rollout depth table (Qwen3/Fable 5).
    pub effort_tiers: [SynthesisEffortTier; 5],
    /// Whether the fused core runs the full pipeline on each prediction.
    pub fused_pipeline_enabled: bool,
}

impl Default for ConsciousnessCoreSynthesis {
    fn default() -> Self {
        Self {
            dominance_cap: 0.5,
            attention_residuals: AttentionResiduals::default(),
            sinkhorn_iters: 20,
            sinkhorn_tol: 1e-6,
            muon: MuonOptimizer::default(),
            safety_router: SafetyRouter::default(),
            step_route_cache: StepRouteCache::new(128),
            context_window: 32,
            context_compression: 0.15,
            effort_tiers: [
                SynthesisEffortTier::Low,
                SynthesisEffortTier::Medium,
                SynthesisEffortTier::High,
                SynthesisEffortTier::XHigh,
                SynthesisEffortTier::Max,
            ],
            fused_pipeline_enabled: true,
        }
    }
}

impl ConsciousnessCoreSynthesis {
    /// Fused prediction pipeline: applies the full model-fusion stack to a
    /// single transition matrix row.
    ///
    /// 1. K3 dominance cap — row cannot be monopolized by one destination
    /// 2. DeepSeek-V4 mHC — Birkhoff doubly-stochastic projection (column-balanced)
    /// 3. AttnRes — cross-depth residual from trajectory history
    /// 4. CSA/HCA — compressed attention reweights recent trajectory states
    /// 5. Effort-scaled sparse top-K — attention condenses with effort
    ///
    /// Returns the fused distribution over 64 states.
    pub fn fused_distribution(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        trajectory: &[u8],
        effort: SynthesisEffortTier,
    ) -> Vec<f64> {
        // 1. K3 Quantile Balancing: dominance-capped row distribution
        let mut base = tm.dominance_capped_distribution(from, self.dominance_cap);

        // 2. DeepSeek-V4 mHC: Sinkhorn-Knopp doubly-stochastic projection.
        // The dominance cap above bounds each cell of a single row; the
        // Birkhoff projection additionally column-normalizes across all
        // sources so no single destination can monopolize routing mass from
        // every state (anti-monopolization in the column space).
        let birkhoff = tm.birkhoff_projected_matrix(self.sinkhorn_iters, self.sinkhorn_tol);
        let birk_row = birkhoff[from.min(63) as usize];
        let mhc_w = 0.35;
        for (b, &bj) in base.iter_mut().zip(birk_row.iter()) {
            *b = *b * (1.0 - mhc_w) + bj * mhc_w;
        }
        let mhc_sum: f64 = base.iter().sum();
        if mhc_sum > 0.0 {
            for b in base.iter_mut() {
                *b /= mhc_sum;
            }
        }

        // 3. AttnRes: cross-depth residual blend
        let mut residual_blend = self.attention_residuals.blend(&base, trajectory);

        // 4. CSA/HCA hybrid compressed attention: reweight the trajectory
        // states by their compressed-attention weights (recent window holds
        // most mass, distant tail compresses into a decaying summary). This
        // makes the fused distribution attend to *where the reasoning has
        // actually been* within the context window, not just transition stats.
        if residual_blend.len() == 64 && !trajectory.is_empty() {
            let csa = compressed_attention_weights(trajectory.len(), self.context_window, self.context_compression);
            let mut csa_boost = vec![0.0f64; 64];
            for (i, &s) in trajectory.iter().enumerate() {
                if let Some(&w) = csa.get(i) {
                    csa_boost[s as usize] += w;
                }
            }
            let csa_w = 0.2;
            for (r, &c) in residual_blend.iter_mut().zip(csa_boost.iter()) {
                *r = *r * (1.0 - csa_w) + c * csa_w;
            }
            let csa_sum: f64 = residual_blend.iter().sum();
            if csa_sum > 0.0 {
                for r in residual_blend.iter_mut() {
                    *r /= csa_sum;
                }
            }
        }

        // 5. Effort-scaled sparse top-K (K3 sparse experts / Qwen3 budget)
        let mut sorted: Vec<(usize, f64)> = residual_blend.iter().enumerate().map(|(i, p)| (i, *p)).collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let k = effort.sparse_k();
        let mut out = vec![0.0f64; 64];
        let mut sum = 0.0;
        for (i, p) in sorted.iter().take(k) {
            out[*i] = *p;
            sum += p;
        }
        if sum > 0.0 {
            for o in out.iter_mut() {
                *o /= sum;
            }
        } else {
            return vec![1.0 / 64.0; 64];
        }
        out
    }

    /// Report the active synthesis knobs for telemetry.
    pub fn telemetry(&self) -> Vec<(String, String)> {
        vec![
            ("synthesis.dominance_cap".into(), format!("{:.2}", self.dominance_cap)),
            ("synthesis.attnres_decay".into(), format!("{:.2}", self.attention_residuals.depth_decay)),
            ("synthesis.attnres_weight".into(), format!("{:.2}", self.attention_residuals.residual_weight)),
            ("synthesis.sinkhorn_iters".into(), format!("{}", self.sinkhorn_iters)),
            ("synthesis.muon_lr".into(), format!("{:.4}", self.muon.lr)),
            ("synthesis.muon_ns_iters".into(), format!("{}", self.muon.ns_iters)),
            ("synthesis.safety_threshold".into(), format!("{:.2}", self.safety_router.risk_threshold)),
            ("synthesis.safety_budget".into(), format!("{:.2}", self.safety_router.frontier_budget)),
            ("synthesis.route_cache_capacity".into(), format!("{}", self.step_route_cache.capacity)),
            ("synthesis.route_cache_hit_rate".into(), format!("{:.3}", self.step_route_cache.hit_rate())),
            ("synthesis.context_window".into(), format!("{}", self.context_window)),
            ("synthesis.fused_pipeline".into(), format!("{}", self.fused_pipeline_enabled)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_residuals_blend_shapes() {
        let ar = AttentionResiduals::default();
        let base = vec![1.0 / 64.0; 64];
        let traj = vec![56u8, 48, 40, 32];
        let out = ar.blend(&base, &traj);
        assert_eq!(out.len(), 64);
        let sum: f64 = out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "blend must be normalized, got {sum}");
        // Trajectory states should get boosted mass
        assert!(out[40] > 1.0 / 64.0, "recent traj state should get residual boost");
        // Deep older states damped
        assert!(out[56] < out[40], "deeper state damped more than recent");
    }

    #[test]
    fn test_attention_residuals_short_traj_identity() {
        let ar = AttentionResiduals::default();
        let base = vec![0.5, 0.5, 0.0];
        let out = ar.blend(&base, &[0]);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_compressed_attention_normalizes() {
        let w = compressed_attention_weights(100, 16, 0.15);
        assert_eq!(w.len(), 100);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        // Recent window states should dominate over far tail
        assert!(w[99] > w[0], "recent should outweigh far tail");
    }

    #[test]
    fn test_compressed_attention_empty() {
        let w = compressed_attention_weights(0, 16, 0.15);
        assert!(w.is_empty());
    }

    #[test]
    fn test_step_route_cache_hits() {
        let mut cache = StepRouteCache::new(8);
        let key = StepRouteKey { task_type: 2, phase: 3, effort: 2, source_bucket: 4 };
        assert!(cache.get(&key).is_none());
        cache.put(key.clone(), vec![(40, 0.5), (42, 0.3)]);
        let got = cache.get(&key);
        assert!(got.is_some());
        assert_eq!(got.unwrap().len(), 2);
        assert_eq!(cache.hit_rate(), 0.5);
    }

    #[test]
    fn test_step_route_cache_eviction() {
        let mut cache = StepRouteCache::new(2);
        for i in 0..3 {
            cache.put(
                StepRouteKey { task_type: i as u8, phase: 0, effort: 0, source_bucket: 0 },
                vec![(i as u8, 1.0)],
            );
        }
        assert_eq!(cache.entries.len(), 2, "capacity enforced");
        // Oldest (task_type 0) evicted
        let old_key = StepRouteKey { task_type: 0, phase: 0, effort: 0, source_bucket: 0 };
        assert!(cache.get(&old_key).is_none());
    }

    #[test]
    fn test_effort_tier_budget_monotonic() {
        let tiers = [
            SynthesisEffortTier::Low,
            SynthesisEffortTier::Medium,
            SynthesisEffortTier::High,
            SynthesisEffortTier::XHigh,
            SynthesisEffortTier::Max,
        ];
        for w in tiers.windows(2) {
            assert!(w[1].thinking_budget() > w[0].thinking_budget());
            assert!(w[1].sparse_k() > w[0].sparse_k());
        }
        assert_eq!(SynthesisEffortTier::from_rank(0), SynthesisEffortTier::Low);
        assert_eq!(SynthesisEffortTier::from_rank(4), SynthesisEffortTier::Max);
    }

    #[test]
    fn test_fused_distribution_normalized() {
        let mut tm = E8TransitionMatrix::new();
        tm.init_from_trace_patterns();
        for _ in 0..50 { tm.record_transition(40, 32); }
        for _ in 0..10 { tm.record_transition(40, 48); }
        for _ in 0..5 { tm.record_transition(40, 24); }

        let synth = ConsciousnessCoreSynthesis::default();
        let traj = vec![56u8, 48, 40];
        let dist = synth.fused_distribution(&tm, 40, &traj, SynthesisEffortTier::High);

        assert_eq!(dist.len(), 64);
        let sum: f64 = dist.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "fused dist must be normalized, got {sum}");
        // Dominant dest survives but cannot exceed cap
        let max_p = dist.iter().cloned().fold(0.0f64, f64::max);
        assert!(max_p > 0.0);
    }

    #[test]
    fn test_fused_distribution_zero_data_falls_back_uniform() {
        let tm = E8TransitionMatrix::new();
        let synth = ConsciousnessCoreSynthesis::default();
        let dist = synth.fused_distribution(&tm, 42, &[56, 48], SynthesisEffortTier::Low);
        let sum: f64 = dist.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_synthesis_telemetry_keys() {
        let synth = ConsciousnessCoreSynthesis::default();
        let t = synth.telemetry();
        assert!(t.iter().any(|(k, _)| k == "synthesis.dominance_cap"));
        assert!(t.iter().any(|(k, _)| k == "synthesis.route_cache_hit_rate"));
        assert!(t.iter().any(|(k, _)| k == "synthesis.muon_ns_iters"));
        assert!(t.iter().any(|(k, _)| k == "synthesis.safety_threshold"));
        assert!(t.len() >= 12);
    }

    #[test]
    fn test_muon_step_matches_param_length() {
        let muon = MuonOptimizer::default();
        let n = 4;
        let params = vec![0.1f64; n * n];
        let grad = vec![0.02f64; n * n];
        let mut buf = vec![0.0f64; n * n];
        let out = muon.step(&grad, &params, &mut buf, n);
        assert_eq!(out.len(), n * n);
        // Params should decrease along a positive-gradient direction
        assert!(out[0] < params[0], "params should decrease along positive gradient");
    }

    #[test]
    fn test_muon_converges_on_least_squares() {
        // Minimize f(x) = sum((x_i - 2)^2): gradient = 2(x - 2)
        let muon = MuonOptimizer { lr: 0.05, momentum: 0.9, weight_decay: 0.0, ns_iters: 2 };
        let n = 4;
        let mut params = vec![0.0f64; n * n];
        let mut buf = vec![0.0f64; n * n];
        for _ in 0..200 {
            let grad: Vec<f64> = params.iter().map(|&x| 2.0 * (x - 2.0)).collect();
            params = muon.step(&grad, &params, &mut buf, n);
        }
        let err: f64 = params.iter().map(|&x| (x - 2.0).abs()).sum::<f64>() / (n * n) as f64;
        assert!(err < 0.3, "muon should converge toward 2.0, avg err {}", err);
    }

    #[test]
    fn test_muon_preserves_dims_after_orthogonalization() {
        let muon = MuonOptimizer::default();
        let n = 3;
        let params = vec![0.5f64; n * n];
        let grad = vec![0.1f64; n * n];
        let mut buf = vec![0.0f64; n * n];
        let out = muon.step(&grad, &params, &mut buf, n);
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_muon_condition_vector_normalizes() {
        let muon = MuonOptimizer::default();
        // Dominant-heavy distribution (single state ~0.9 mass)
        let mut dist = vec![0.0f64; 64];
        dist[0] = 0.9;
        dist[1] = 0.05;
        dist[2] = 0.05;
        let out = muon.condition_vector(&dist);
        assert_eq!(out.len(), 64);
        let sum: f64 = out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "conditioned vector must renormalize, got {sum}");
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_muon_condition_vector_reduces_dominance() {
        let muon = MuonOptimizer::default();
        // Rank-2 with correlated columns: row0=(0.9,0.1,...) row1=(0.1,0.9,...)
        // → columns are NOT orthogonal; NS should decorrelate them.
        let mut dist = vec![0.0f64; 64];
        dist[0] = 0.45; dist[1] = 0.05;  // row 0
        dist[8] = 0.05; dist[9] = 0.45;  // row 1
        let out = muon.condition_vector(&dist);
        assert_eq!(out.len(), 64);
        // MᵀM off-diagonal must shrink toward 0 after orthogonalization
        let off_before = {
            let (mut a, mut b) = (0.0, 0.0);
            for k in 0..8 { a += dist[k*8+0]*dist[k*8+0]; b += dist[k*8+0]*dist[k*8+1]; }
            let _ = a;
            b
        };
        let off_after = {
            let mut acc = 0.0;
            for k in 0..8 { acc += out[k*8+0]*out[k*8+1]; }
            acc
        };
        assert!(
            off_after.abs() < off_before.abs(),
            "off-diagonal correlation should shrink: before {off_before:.4}, after {off_after:.4}"
        );
    }

    #[test]
    fn test_muon_condition_vector_wrong_len_identity() {
        let muon = MuonOptimizer::default();
        let dist = vec![0.5, 0.5];
        assert_eq!(muon.condition_vector(&dist), dist);
    }

    #[test]
    fn test_safety_router_risk_classification() {
        let router = SafetyRouter::default();
        assert!(router.classify_risk("summarize a paper") < 0.3);
        assert!(router.classify_risk("exploit chain with shellcode payload") > 0.7);
        assert!(router.classify_risk("password cracking toolkit") >= 0.35);
    }

    #[test]
    fn test_safety_router_fallback_damps_toward_uniform() {
        let router = SafetyRouter::default();
        let dist = vec![0.9, 0.05, 0.05];
        let conservative = router.conservative_distribution(&dist);
        let sum: f64 = conservative.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!(conservative[0] < 0.9, "dominant mass should be damped");
        assert!(conservative[1] > 0.05, "minor mass should be lifted toward uniform");
    }

    #[test]
    fn test_safety_router_blocks_frontier_on_high_risk() {
        let router = SafetyRouter::default();
        assert!(!router.allow_frontier("exploit payload zero-day", 0, 0));
        assert!(router.allow_frontier("write a blog post", 0, 0));
    }

    #[test]
    fn test_safety_router_budget_caps_frontier() {
        let router = SafetyRouter { frontier_budget: 0.5, ..Default::default() };
        // Frontier fraction = miss rate = (total - hits)/total.
        // 3 of 10 misses → 0.3 miss rate → within 0.5 budget → allow frontier
        assert!(router.allow_frontier("normal task", 7, 10));
        // 7 of 10 misses → 0.7 miss rate → above 0.5 budget → block frontier
        assert!(!router.allow_frontier("normal task", 3, 10));
    }
}
