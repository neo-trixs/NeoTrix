#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Editable meta-strategy: the self-improvement mechanism itself.
///
/// In a DGM-H architecture, the meta-agent (what proposes/evaluates/selects
/// mutations) is merged with the task-agent into a single editable program.
/// `MetaStrategy` stores the three core meta-loop components as Ne source code,
/// enabling the evolution loop to modify its own improvement process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaStrategy {
    /// Ne source: proposes the next mutation given archive state and cycle.
    /// Signature: `fn propose(archive: &Archive, cycle: u64, rng: f64) -> MutationOp`
    pub proposer: String,
    /// Ne source: evaluates a candidate mutation's quality.
    /// Signature: `fn evaluate(mutation: &MutationOp, score_before: f64, score_after: f64) -> f64`
    pub evaluator: String,
    /// Ne source: selects parent from archive for branching.
    /// Signature: `fn select(archive: &Archive, rng: f64) -> usize`
    pub selector: String,
    /// Version incremented on each meta-mutation.
    pub version: u32,
    /// Whether this meta-strategy was proposed by the system itself.
    pub self_proposed: bool,
}

impl MetaStrategy {
    /// Default meta-strategy matching the original Rust hardcoded logic.
    pub fn default_v1() -> Self {
        Self {
            proposer: String::new(), // empty = fallback to Rust default
            evaluator: String::new(),
            selector: String::new(),
            version: 1,
            self_proposed: false,
        }
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let has_proposer = if self.proposer.is_empty() {
            "default"
        } else {
            "ne"
        };
        let has_evaluator = if self.evaluator.is_empty() {
            "default"
        } else {
            "ne"
        };
        let has_selector = if self.selector.is_empty() {
            "default"
        } else {
            "ne"
        };
        format!(
            "MetaStrategy v{}: proposer={}, evaluator={}, selector={}, self_proposed={}",
            self.version, has_proposer, has_evaluator, has_selector, self.self_proposed,
        )
    }

    /// Evaluate a mutation candidate via the Ne evaluator.
    ///
    /// The evaluator Ne code must return a float score string (e.g. `"0.75"`).
    /// If empty or evaluation fails, returns `None` (Rust fallback).
    pub fn evaluate_via_ne(
        &self,
        score_before: f64,
        score_after: f64,
        ci: &mut impl crate::core::nt_core_traits::ConsciousnessHandle,
    ) -> Option<f64> {
        if self.evaluator.is_empty() {
            return None;
        }
        let source = self.evaluator.clone();
        // Build evaluator expression with score values bound
        let expr = format!(
            r#"(let score_before {} (let score_after {} {}))"#,
            score_before, score_after, source
        );
        match ci.eval_ne_string(&expr) {
            Ok(val_str) => {
                let val_str = val_str.trim();
                let val_str =
                    if val_str.starts_with('"') && val_str.ends_with('"') && val_str.len() >= 2 {
                        &val_str[1..val_str.len() - 1]
                    } else {
                        val_str
                    };
                val_str.parse::<f64>().ok()
            }
            Err(e) => {
                log::warn!("META_AGENT: evaluator Ne execution failed: {}", e);
                None
            }
        }
    }

    /// Select a parent index via the Ne evaluator.
    ///
    /// The selector Ne code must return a usize string (e.g. `"3"`).
    /// If empty or evaluation fails, returns `None` (Rust fallback).
    pub fn select_via_ne(
        &self,
        archive_len: usize,
        rng_val: f64,
        ci: &mut impl crate::core::nt_core_traits::ConsciousnessHandle,
    ) -> Option<usize> {
        if self.selector.is_empty() {
            return None;
        }
        let source = self.selector.clone();
        let expr = format!(
            r#"(let archive_len {} (let rng {} {}))"#,
            archive_len, rng_val, source
        );
        match ci.eval_ne_string(&expr) {
            Ok(val_str) => {
                let val_str = val_str.trim();
                let val_str =
                    if val_str.starts_with('"') && val_str.ends_with('"') && val_str.len() >= 2 {
                        &val_str[1..val_str.len() - 1]
                    } else {
                        val_str
                    };
                val_str.parse::<usize>().ok().filter(|&i| i < archive_len)
            }
            Err(e) => {
                log::warn!("META_AGENT: selector Ne execution failed: {}", e);
                None
            }
        }
    }
}

/// Types of self-modification operations in the evolution loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MutationOp {
    /// Adjust a numeric parameter by a delta.
    TuneParam { target: String, delta: f64 },
    /// Add a new handler at a position with code body.
    AddHandler { position: String, code: String },
    /// Replace an existing handler's code.
    RewriteHandler { name: String, code: String },
    /// Change the PACE gate sequence.
    SwapPolicy { gates: Vec<String> },
    /// Rewrite a VSA primitive implementation.
    RewritePrimitive { name: String, impl_: String },
    /// Rewrite the meta-strategy itself — DGM-H style self-referential improvement.
    RewriteMeta { strategy: MetaStrategy },
    /// Gödel Agent self-modification: rewrite handler, parameter, primitive,
    /// pipeline stage, or safety gate with proposed source code.
    /// The source_code goes through SelfModifyGuard + SandboxValidator before execution.
    SelfModifyProposal {
        target: String,
        target_type: String,
        source_code: String,
    },
}

impl MutationOp {
    /// Parse a MutationOp from a Ne-runtime return value string.
    ///
    /// The Ne proposer must return a string in the format:
    /// - `"TuneParam:target_name:delta"`  → TuneParam { target: "target_name", delta }
    /// - `"RewriteHandler:handler_name"`  → RewriteHandler { name: "handler_name", code: "// Ne-proposed" }
    /// - `"AddHandler:position"`          → AddHandler { position, code: "// Ne-proposed" }
    /// - `"SwapPolicy:gate1,gate2"`       → SwapPolicy { gates: ["gate1", "gate2"] }
    /// - `"RewritePrimitive:name"`        → RewritePrimitive { name, impl_: "// Ne-proposed" }
    /// - `""` or invalid                  → None (fallback to Rust default)
    pub fn from_ne_string(s: &str) -> Option<MutationOp> {
        let s = s.trim();
        // Strip surrounding quotes if present (from NeValue::Str Display)
        let s = if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            &s[1..s.len() - 1]
        } else {
            s
        };
        if s.is_empty() {
            return None;
        }
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        let kind = parts.first()?;
        match *kind {
            "TuneParam" => {
                if parts.len() < 3 {
                    return None;
                }
                let target = parts[1].to_string();
                let delta: f64 = parts[2].parse().ok()?;
                Some(MutationOp::TuneParam { target, delta })
            }
            "RewriteHandler" => {
                let name = parts.get(1)?.to_string();
                Some(MutationOp::RewriteHandler {
                    name,
                    code: "// Ne-proposed".to_string(),
                })
            }
            "AddHandler" => {
                let position = parts.get(1)?.to_string();
                Some(MutationOp::AddHandler {
                    position,
                    code: "// Ne-proposed".to_string(),
                })
            }
            "SwapPolicy" => {
                let gates_str = parts.get(1)?;
                let gates: Vec<String> = gates_str
                    .split(',')
                    .map(|g| g.trim().to_string())
                    .filter(|g| !g.is_empty())
                    .collect();
                if gates.is_empty() {
                    return None;
                }
                Some(MutationOp::SwapPolicy { gates })
            }
            "RewritePrimitive" => {
                let name = parts.get(1)?.to_string();
                Some(MutationOp::RewritePrimitive {
                    name,
                    impl_: "// Ne-proposed".to_string(),
                })
            }
            "SelfModifyProposal" => {
                if parts.len() < 3 {
                    return None;
                }
                let target_type = parts[1].to_string();
                let target_and_code = parts[2].to_string();
                // format: "target_name:source_code"
                let mut sc_parts = target_and_code.splitn(2, ':');
                let target = sc_parts.next().unwrap_or("unknown").to_string();
                let source_code = sc_parts.next().unwrap_or("").to_string();
                Some(MutationOp::SelfModifyProposal {
                    target,
                    target_type,
                    source_code,
                })
            }
            _ => None,
        }
    }

    /// Human-readable label for the mutation type.
    pub fn label(&self) -> &'static str {
        match self {
            MutationOp::TuneParam { .. } => "TuneParam",
            MutationOp::AddHandler { .. } => "AddHandler",
            MutationOp::RewriteHandler { .. } => "RewriteHandler",
            MutationOp::SwapPolicy { .. } => "SwapPolicy",
            MutationOp::RewritePrimitive { .. } => "RewritePrimitive",
            MutationOp::RewriteMeta { .. } => "RewriteMeta",
            MutationOp::SelfModifyProposal { .. } => "SelfModifyProposal",
        }
    }

    /// Short summary string for reports.
    pub fn summary(&self) -> String {
        match self {
            MutationOp::TuneParam { target, delta } => {
                format!("TuneParam {} by {:.4}", target, delta)
            }
            MutationOp::AddHandler { position, code } => {
                let preview: String = code.chars().take(30).collect();
                format!("AddHandler at {}: {}", position, preview)
            }
            MutationOp::RewriteHandler { name, code } => {
                let preview: String = code.chars().take(30).collect();
                format!("RewriteHandler {}: {}", name, preview)
            }
            MutationOp::SwapPolicy { gates } => {
                format!("SwapPolicy [{}]", gates.join(", "))
            }
            MutationOp::RewritePrimitive { name, impl_ } => {
                let preview: String = impl_.chars().take(30).collect();
                format!("RewritePrimitive {}: {}", name, preview)
            }
            MutationOp::RewriteMeta { strategy } => {
                format!("RewriteMeta v{}", strategy.version)
            }
            MutationOp::SelfModifyProposal {
                target,
                target_type,
                source_code,
            } => {
                let preview: String = source_code.chars().take(40).collect();
                format!("SelfModifyProposal {}:{}: {}", target_type, target, preview)
            }
        }
    }
}

/// A pattern absorbed from GitHub or local source.
#[derive(Debug, Clone)]
pub struct AbsorbedPattern {
    pub id: u64,
    pub source_url: String,
    pub language: String,
    pub pattern_code: String,
    pub description: String,
    pub value_assessment: f64,
    pub absorbed_at: u64,
}

/// A single step recorded in the evolution archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEvolutionStep {
    pub id: u64,
    pub mutation: MutationOp,
    pub parent_id: u64,
    pub score_before: f64,
    pub score_after: Option<f64>,
    pub compiles: bool,
    pub accepted: bool,
    pub timestamp: u64,
    pub generation: u32,
    /// HGM CMP score — coherence-measured progress metric.
    /// Captured before/after each mutation when feature=hgm is enabled.
    /// None when HGM is not active or archive predates the feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmp_score: Option<f64>,
}

/// Fitness function type: takes (description, cycle, handler_stats) → score in [-1, 1]
pub type EvolutionFitnessFn = Box<dyn Fn(&str, u64, &EvolutionHandlerStats) -> f64 + Send + Sync>;

/// Handler statistics passed to the fitness function for evaluation.
#[derive(Debug, Clone)]
pub struct EvolutionHandlerStats {
    pub total_handlers: usize,
    pub cycle: u64,
    pub hot_count: usize,
    pub warm_count: usize,
    pub cold_count: usize,
    pub recent_acceptance_rate: f64,
    pub archive_size: usize,
}

/// Archive of all evolution steps with pruning and best-score tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEvolutionArchive {
    pub steps: Vec<SelfEvolutionStep>,
    pub archive_limit: usize,
    pub best_score: f64,
    pub best_step_id: u64,
    pub generation: u32,
}

impl SelfEvolutionArchive {
    pub fn new() -> Self {
        Self::new_with_limit(200)
    }

    pub fn new_with_limit(limit: usize) -> Self {
        Self {
            steps: Vec::with_capacity(limit),
            archive_limit: limit,
            best_score: 0.0,
            best_step_id: 0,
            generation: 0,
        }
    }

    /// Add a step, update best-score / generation, and prune if over capacity.
    pub fn add(&mut self, step: SelfEvolutionStep) {
        if let Some(after) = step.score_after {
            if after > self.best_score {
                self.best_score = after;
                self.best_step_id = step.id;
            }
        }
        if step.generation > self.generation {
            self.generation = step.generation;
        }
        self.steps.push(step);
        self.prune();
    }

    /// Prune steps when exceeding archive_limit.
    /// Keeps 3 best-scoring steps and fills remaining slots
    /// with a random sample of the rest to preserve diversity.
    pub fn prune(&mut self) {
        if self.steps.len() <= self.archive_limit {
            return;
        }
        let elite_count = 3.min(self.steps.len());
        self.steps.sort_by(|a, b| {
            let sa = a.score_after.unwrap_or(0.0);
            let sb = b.score_after.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        let elite: Vec<SelfEvolutionStep> = self.steps.drain(..elite_count).collect();
        let mut rest: Vec<SelfEvolutionStep> = self.steps.drain(..).collect();
        // Fisher-Yates shuffle of the rest
        let mut rng_seed: u64 = 42;
        for i in (1..rest.len()).rev() {
            rng_seed = rng_seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let x = (rng_seed >> 33) as u32;
            let r = (x as f64) / (u32::MAX as f64);
            let j = (r * (i + 1) as f64) as usize;
            rest.swap(i, j);
        }
        let fill = self.archive_limit.saturating_sub(elite.len());
        let selected: Vec<SelfEvolutionStep> = rest.into_iter().take(fill).collect();
        self.steps = elite.into_iter().chain(selected).collect();
    }
}

impl SelfEvolutionArchive {
    /// Serialize archive to JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(&self.steps)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Load archive from JSON bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let steps: Vec<SelfEvolutionStep> = serde_json::from_slice(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut archive = SelfEvolutionArchive::new_with_limit(1000);
        for step in steps {
            archive.add(step);
        }
        Ok(archive)
    }

    /// Save to file atomically (write to tmp, rename).
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let data = self.to_bytes()?;
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &data)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load from file, returns empty archive if file doesn't exist.
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        match std::fs::read(path) {
            Ok(data) => Self::from_bytes(&data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(SelfEvolutionArchive::new_with_limit(1000))
            }
            Err(e) => Err(e),
        }
    }
}

impl Default for SelfEvolutionArchive {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistable wrapper that bundles the evolution archive with the DGM-H meta-strategy.
///
/// Stored as a single JSON object (not a flat array), enabling single-file atomic save/restore
/// of both the archive history and the self-referential improvement mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionState {
    pub steps: Vec<SelfEvolutionStep>,
    pub meta_strategy: Option<MetaStrategy>,
    pub generation: u32,
    pub best_score: f64,
    pub best_step_id: u64,
}

impl EvolutionState {
    /// Serialize to JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Deserialize from JSON bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        serde_json::from_slice(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl SelfEvolutionArchive {
    /// Convert the archive into a persistable `EvolutionState` alongside the meta-strategy.
    pub fn to_evolution_state(&self, meta: Option<&MetaStrategy>) -> EvolutionState {
        EvolutionState {
            steps: self.steps.clone(),
            meta_strategy: meta.cloned(),
            generation: self.generation,
            best_score: self.best_score,
            best_step_id: self.best_step_id,
        }
    }

    /// Reconstruct `Self` and optional `MetaStrategy` from an `EvolutionState`.
    pub fn from_evolution_state(state: EvolutionState) -> (Self, Option<MetaStrategy>) {
        let mut archive = SelfEvolutionArchive::new_with_limit(1000);
        for step in state.steps {
            archive.add(step);
        }
        (archive, state.meta_strategy)
    }
}

/// An arm in the drive-controlled evolution bandit.
/// Tracks selection count and cumulative reward for Thompson sampling.
#[derive(Debug, Clone)]
pub struct BanditArm {
    pub drive_type: String,
    pub count: u32,
    pub sum_reward: f64,
    pub sum_sq_reward: f64,
}

/// Thompson-sampling bandit that selects evolution strategies
/// biased by the PAD-emotional dominant drive.
#[derive(Debug, Clone)]
pub struct DriveBanditState {
    pub arms: Vec<BanditArm>,
    pub total_trials: u32,
    pub alpha_prior: f64,
    pub beta_prior: f64,
    pub last_arm_selected: Option<String>,
}

impl DriveBanditState {
    pub fn new() -> Self {
        let drive_names = [
            "explore",
            "exploit",
            "repair",
            "innovate",
            "harden",
            "prune",
            "socialize",
            "rest",
        ];
        Self {
            arms: drive_names
                .iter()
                .map(|name| BanditArm {
                    drive_type: name.to_string(),
                    count: 0,
                    sum_reward: 0.0,
                    sum_sq_reward: 0.0,
                })
                .collect(),
            total_trials: 0,
            alpha_prior: 2.0,
            beta_prior: 2.0,
            last_arm_selected: None,
        }
    }

    /// Approximate Beta-distribution sample via mean + uncertainty noise.
    fn beta_sample(alpha: f64, beta: f64, rng_val: f64) -> f64 {
        let total = alpha + beta;
        if total <= 0.0 {
            return 0.0;
        }
        let mean = alpha / total;
        let std = (alpha * beta / (total * total * (total + 1.0)))
            .sqrt()
            .max(1e-6);
        let u = rng_val.clamp(1e-10, 1.0 - 1e-10);
        let z = (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * u).cos();
        (mean + z * std).clamp(0.001, 0.999)
    }

    /// Thompson sampling: pick the arm with the highest biased Beta sample.
    /// When all counts are zero, returns `dominant_drive` directly.
    pub fn select_arm(&self, dominant_drive: &str, rng: f64) -> String {
        if self.arms.iter().all(|a| a.count == 0) {
            return dominant_drive.to_string();
        }

        let mut best_score = -1.0f64;
        let mut best_arm = dominant_drive.to_string();

        for (i, arm) in self.arms.iter().enumerate() {
            let alpha = self.alpha_prior + arm.sum_reward;
            let beta_val = self.beta_prior + (arm.count as f64 - arm.sum_reward);
            let arm_rng = ((rng * 10000.0) as u64 + i as u64) as f64 * 0.0001;
            let sample = Self::beta_sample(alpha, beta_val, arm_rng);
            let bias = if arm.drive_type == dominant_drive {
                1.3
            } else {
                1.0
            };
            let score = sample * bias;
            if score > best_score {
                best_score = score;
                best_arm = arm.drive_type.clone();
            }
        }

        best_arm
    }

    /// Update an arm with a new reward observation.
    pub fn update_arm(&mut self, drive: &str, reward: f64) {
        if let Some(arm) = self.arms.iter_mut().find(|a| a.drive_type == drive) {
            arm.count += 1;
            arm.sum_reward += reward;
            arm.sum_sq_reward += reward * reward;
        }
        self.total_trials += 1;
        self.last_arm_selected = Some(drive.to_string());
    }

    /// Get the normalized weight (mean reward) for a given drive arm.
    /// Returns 0.0 if the arm has no trials.
    pub fn drive_weight(&self, drive: &str) -> f64 {
        self.arms
            .iter()
            .find(|a| a.drive_type == drive)
            .map(|arm| {
                if arm.count > 0 {
                    arm.sum_reward / arm.count as f64
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }
}

impl Default for DriveBanditState {
    fn default() -> Self {
        Self::new()
    }
}
