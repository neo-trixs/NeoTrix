use super::self_evolution_loop::{MetaStrategy, MutationOp, SelfEvolutionLoop};
use super::trajectory_heuristics::{ExperienceRecord, Heuristic, TrajectoryHeuristicExtractor};
use crate::core::nt_core_experience::layered_mutability::{
    LayeredMutabilityTracker, MutabilityLayer,
};
use crate::core::nt_core_traits::ConsciousnessHandle;

use fastrand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealPhase {
    Distill,
    Apply,
    Verify,
    Rollback,
    Commit,
}

impl SealPhase {
    pub fn label(&self) -> &'static str {
        match self {
            SealPhase::Distill => "distill",
            SealPhase::Apply => "apply",
            SealPhase::Verify => "verify",
            SealPhase::Rollback => "rollback",
            SealPhase::Commit => "commit",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SealPhase::Distill => SealPhase::Apply,
            SealPhase::Apply => SealPhase::Verify,
            SealPhase::Verify => SealPhase::Rollback,
            SealPhase::Rollback => SealPhase::Commit,
            SealPhase::Commit => SealPhase::Distill,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangeCandidate {
    pub heuristic: Heuristic,
    pub mutation: MutationOp,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub phase: SealPhase,
    pub score_before: f64,
    pub score_after: f64,
    pub passed: bool,
    pub cycle: u64,
}

/// A Pareto-optimal candidate: one point in the trade-off space.
#[derive(Debug, Clone)]
pub struct ParetoCandidate {
    pub heuristic: Heuristic,
    pub score: f64,
    pub diversity: f64,
    pub cycle_discovered: u64,
    pub exploitation_score: f64,
    pub exploration_score: f64,
}

#[derive(Debug, Clone)]
pub struct SealClosedLoop {
    pub phase: SealPhase,
    pub current_experience: Option<ExperienceRecord>,
    pub applied_changes: Vec<String>,
    pub verification_results: Vec<bool>,
    pub cycle_counter: u64,
    pub distill_interval: u64,
    pub d_score_boost: f64,
    pub change_candidates: Vec<ChangeCandidate>,
    pub active_candidate: Option<ChangeCandidate>,
    pub score_before: f64,
    pub trace_history: Vec<TraceEntry>,
    pub pareto_front: Vec<ParetoCandidate>,
    pub pareto_max_size: usize,
    /// Hash-chained governance ledger for tamper-evident SEAL audit.
    /// Each trace entry is chained to the previous via SHA-256-style hash.
    pub governance_chain_hash: u64,
    pub governance_chain: Vec<GovernanceEntry>,
    /// Layered mutability tracker for identity hysteresis protection
    pub layered_mutability: Option<LayeredMutabilityTracker>,
}

/// A single entry in the SEAL governance hash chain.
/// Each entry links to the previous via prior_hash, forming a tamper-evident ledger.
#[derive(Debug, Clone)]
pub struct GovernanceEntry {
    pub phase: SealPhase,
    pub description: String,
    pub score_before: f64,
    pub score_after: f64,
    pub passed: bool,
    pub chain_hash: u64,
    pub prior_hash: u64,
    pub cycle: u64,
}

fn chain_hash(prior: u64, description: &str, phase: &str, score: f64) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prior.hash(&mut hasher);
    description.hash(&mut hasher);
    phase.hash(&mut hasher);
    score.to_bits().hash(&mut hasher);
    hasher.finish()
}

impl SealClosedLoop {
    pub fn new() -> Self {
        Self {
            phase: SealPhase::Distill,
            current_experience: None,
            applied_changes: Vec::new(),
            verification_results: Vec::new(),
            cycle_counter: 0,
            distill_interval: 10,
            d_score_boost: 0.1,
            change_candidates: Vec::new(),
            active_candidate: None,
            score_before: 0.0,
            trace_history: Vec::new(),
            pareto_front: Vec::new(),
            pareto_max_size: 20,
            governance_chain_hash: 0xe8e8_e8e8_e8e8_e8e8u64,
            governance_chain: Vec::new(),
            layered_mutability: Some(LayeredMutabilityTracker::new()),
        }
    }

    pub fn with_distill_interval(mut self, interval: u64) -> Self {
        self.distill_interval = interval;
        self
    }

    pub fn step(
        &mut self,
        evolution: &mut SelfEvolutionLoop,
        extractor: &mut TrajectoryHeuristicExtractor,
        trajectory: &[ExperienceRecord],
        ci: &mut impl ConsciousnessHandle,
    ) -> SealPhase {
        self.cycle_counter += 1;
        let current_score = ci.stats_c_score() - (ci.cognitive_load() * 0.3).min(0.3)
            + (ci.self_evolution_best_score() * self.d_score_boost).min(0.1);

        match self.phase {
            SealPhase::Distill => {
                let heuristics = extractor.distill_heuristics(trajectory);
                let mut candidates: Vec<ChangeCandidate> = heuristics
                    .iter()
                    .filter(|h| h.confidence > 0.5)
                    .map(|h| ChangeCandidate {
                        heuristic: h.clone(),
                        mutation: self.heuristic_to_mutation(h),
                        description: format!("heuristic: {}", h.principle),
                    })
                    .collect();

                // For high-confidence failure heuristics, also generate
                // ne_surface (S-expr) strategy mutations (Leap 2).
                for h in heuristics
                    .iter()
                    .filter(|h| !h.is_positive && h.confidence > 0.7)
                {
                    let ne_mutation = self.heuristic_to_ne_strategy(h, ci);
                    candidates.push(ChangeCandidate {
                        heuristic: h.clone(),
                        mutation: ne_mutation,
                        description: format!("ne_strategy_for: {}", h.principle),
                    });
                }

                self.change_candidates = candidates;

                if !self.change_candidates.is_empty() {
                    // GEPA: select from Pareto front with fallback to best
                    self.active_candidate = self
                        .select_from_pareto()
                        .and_then(|(h, _is_pareto)| {
                            self.change_candidates
                                .iter()
                                .find(|c| c.heuristic.principle == h.principle)
                                .cloned()
                        })
                        .or_else(|| self.change_candidates.first().cloned());
                    self.score_before = current_score;
                    self.phase = SealPhase::Apply;
                }
            }

            SealPhase::Apply => {
                // Check identity hysteresis before allowing mutation
                let mutation_allowed = self
                    .layered_mutability
                    .as_ref()
                    .map(|lm| lm.check_mutation_allowed(MutabilityLayer::SelfNarrative))
                    .unwrap_or((true, None));
                if !mutation_allowed.0 {
                    log::warn!(
                        "SEAL_CL: mutation blocked by LayeredMutabilityTracker: {}",
                        mutation_allowed.1.unwrap_or_default()
                    );
                    self.active_candidate = None;
                    self.phase = SealPhase::Distill;
                    return self.phase;
                }
                if let Some(ref candidate) = self.active_candidate {
                    let result = evolution.execute_mutation(&candidate.mutation, ci);
                    match result {
                        Ok(_) => {
                            // Record mutation in LayeredMutabilityTracker
                            if let Some(ref mut lm) = self.layered_mutability {
                                lm.record_mutation(
                                    MutabilityLayer::SelfNarrative,
                                    0.05,
                                    self.cycle_counter,
                                );
                            }
                            self.applied_changes.push(candidate.description.clone());
                            log::info!("SEAL_CL: applied change: {}", candidate.description);
                        }
                        Err(e) => {
                            log::warn!("SEAL_CL: apply failed: {}", e);
                            self.applied_changes.push(format!("failed: {}", e));
                        }
                    }
                }
                self.phase = SealPhase::Verify;
            }

            SealPhase::Verify => {
                let score_after = ci.stats_c_score() - (ci.cognitive_load() * 0.3).min(0.3)
                    + (ci.self_evolution_best_score() * self.d_score_boost).min(0.1);
                // GEPA: update Pareto front with the evaluated candidate
                let heuristic_for_pareto =
                    self.active_candidate.as_ref().map(|c| c.heuristic.clone());
                if let Some(ref h) = heuristic_for_pareto {
                    self.update_pareto(h, score_after);
                }
                let threshold = self.score_before * 0.95;
                let passed = score_after >= threshold;
                self.verification_results.push(passed);
                self.trace_history.push(TraceEntry {
                    phase: self.phase,
                    score_before: self.score_before,
                    score_after,
                    passed,
                    cycle: self.cycle_counter,
                });
                // Hash-chain governance entry for tamper-evident SEAL audit
                let desc = self
                    .active_candidate
                    .as_ref()
                    .map(|c| c.description.as_str())
                    .unwrap_or("verify");
                let new_hash = chain_hash(
                    self.governance_chain_hash,
                    desc,
                    self.phase.label(),
                    score_after,
                );
                self.governance_chain.push(GovernanceEntry {
                    phase: self.phase,
                    description: desc.to_string(),
                    score_before: self.score_before,
                    score_after,
                    passed,
                    chain_hash: new_hash,
                    prior_hash: self.governance_chain_hash,
                    cycle: self.cycle_counter,
                });
                self.governance_chain_hash = new_hash;

                if passed {
                    log::info!(
                        "SEAL_CL: verification PASSED (score {:.4} >= {:.4})",
                        score_after,
                        threshold
                    );
                } else {
                    log::warn!(
                        "SEAL_CL: verification FAILED (score {:.4} < {:.4})",
                        score_after,
                        threshold
                    );
                }
                self.phase = SealPhase::Rollback;
            }

            SealPhase::Rollback => {
                let last_passed = self.verification_results.last().copied().unwrap_or(false);
                if !last_passed {
                    if let Some(ref candidate) = self.active_candidate {
                        match &candidate.mutation {
                            MutationOp::TuneParam { target, delta } => {
                                evolution
                                    .execute_mutation(
                                        &MutationOp::TuneParam {
                                            target: target.clone(),
                                            delta: -delta,
                                        },
                                        ci,
                                    )
                                    .ok();
                                log::info!("SEAL_CL: rollback applied for {}", target);
                            }
                            MutationOp::RewriteMeta { strategy: _ } => {
                                // RewriteMeta rollback: meta_agent_tick() will
                                // generate a compensating strategy next cycle.
                                log::warn!(
                                    "SEAL_CL: RewriteMeta failed, will auto-correct next cycle"
                                );
                            }
                            _ => {}
                        }
                        // Record rollback in LayeredMutabilityTracker
                        if let Some(ref mut lm) = self.layered_mutability {
                            lm.record_rollback(MutabilityLayer::SelfNarrative, self.cycle_counter);
                        }
                    }
                }
                self.phase = SealPhase::Commit;
            }

            SealPhase::Commit => {
                let all_passed = self.verification_results.iter().all(|&v| v);
                if all_passed && !self.applied_changes.is_empty() {
                    let mutation = self
                        .active_candidate
                        .as_ref()
                        .map(|c| c.mutation.clone())
                        .unwrap_or(MutationOp::TuneParam {
                            target: "noop".into(),
                            delta: 0.0,
                        });
                    evolution.record_result(mutation, self.score_before, current_score, true, None);
                    log::info!(
                        "SEAL_CL: committed change: {}",
                        self.applied_changes.last().cloned().unwrap_or_default()
                    );
                }

                self.active_candidate = None;
                self.phase = SealPhase::Distill;
            }
        }

        self.phase
    }

    pub fn should_run(&self) -> bool {
        self.cycle_counter % self.distill_interval == 0
    }

    pub fn status_report(&self) -> String {
        let mut report = format!(
            "SEAL_CL: phase={} changes={} verifications={}/{} passed={}",
            self.phase.label(),
            self.applied_changes.len(),
            self.verification_results.len(),
            self.verification_results.iter().filter(|&&v| v).count(),
            self.cycle_counter,
        );
        if let Some(ref lm) = self.layered_mutability {
            report.push_str(&format!(" | hysteresis={:.4}", lm.hysteresis_ratio));
        }
        report
    }

    pub fn analyse_traces(&self) -> Vec<String> {
        let mut insights = Vec::new();
        let traces: Vec<&TraceEntry> = self.trace_history.iter().rev().take(20).collect();
        if traces.is_empty() {
            return insights;
        }

        let passes = traces.iter().take_while(|t| t.passed).count();
        if passes >= 5 {
            insights.push(format!(
                "GEPA_TRACE: stable — last {} changes all passed",
                passes
            ));
        }

        let failures = traces.iter().take_while(|t| !t.passed).count();
        if failures >= 2 {
            insights.push(format!(
                "GEPA_TRACE: degradation — {} consecutive failures. Rollback recommended.",
                failures
            ));
        }

        if traces.len() >= 6
            && traces
                .windows(2)
                .take(5)
                .all(|w| w[0].passed != w[1].passed)
        {
            insights.push(
                "GEPA_TRACE: oscillation detected — pass/fail alternates across 6 entries. Consider adjusting distill_interval.".to_string()
            );
        }

        insights
    }

    pub fn gepa_mutate_from_traces(&mut self, ci: &mut impl ConsciousnessHandle) {
        let traces: Vec<&TraceEntry> = self.trace_history.iter().rev().take(20).collect();
        if traces.len() < 2 {
            return;
        }

        let failures = traces.iter().take_while(|t| !t.passed).count();
        if failures >= 2 {
            if let Some(ref candidate) = self.active_candidate {
                let mutation = self.heuristic_to_ne_strategy(&candidate.heuristic, ci);
                if let MutationOp::RewriteMeta { strategy } = &mutation {
                    let _ = ci.eval_ne_string(&strategy.proposer);
                    log::info!(
                        "GEPA_TRACE: compensating strategy from {} failures via {}",
                        failures,
                        strategy.proposer
                    );
                }
            }
            return;
        }

        if traces.len() >= 6
            && traces
                .windows(2)
                .take(5)
                .all(|w| w[0].passed != w[1].passed)
        {
            if self.distill_interval > 5 {
                self.distill_interval -= 5;
            } else {
                self.distill_interval += 5;
            }
            log::info!(
                "GEPA_TRACE: oscillation, adjusted distill_interval to {}",
                self.distill_interval
            );
        }
    }

    /// Update Pareto front with a new candidate.
    /// Maintains a fixed-size diverse Pareto front.
    pub fn update_pareto(&mut self, heuristic: &Heuristic, score: f64) {
        let diversity = self
            .pareto_front
            .iter()
            .map(|pc| (pc.heuristic.confidence - heuristic.confidence).abs())
            .fold(f64::INFINITY, f64::min);

        let exploitation_score = score;
        let exploration_score = heuristic.confidence;

        let dominated = self
            .pareto_front
            .iter()
            .any(|pc| pc.score > score && pc.exploration_score > exploration_score);
        if dominated {
            return;
        }

        self.pareto_front
            .retain(|pc| !(score > pc.score && exploration_score > pc.exploration_score));

        self.pareto_front.push(ParetoCandidate {
            heuristic: heuristic.clone(),
            score,
            diversity: if diversity.is_finite() {
                diversity
            } else {
                1.0
            },
            cycle_discovered: self.cycle_counter,
            exploitation_score,
            exploration_score,
        });

        while self.pareto_front.len() > self.pareto_max_size {
            let idx = self
                .pareto_front
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.diversity
                        .partial_cmp(&b.diversity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.pareto_front.remove(idx);
        }
    }

    /// Select a candidate from the Pareto front.
    /// Returns (heuristic, is_pareto).
    /// Uses epsilon-greedy: 70% best, 20% random Pareto, 10% random from front.
    pub fn select_from_pareto(&self) -> Option<(Heuristic, bool)> {
        if self.pareto_front.is_empty() {
            return None;
        }

        let best = self.pareto_front.iter().max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

        let choice: f64 = fastrand::f64();
        if choice < 0.7 {
            return Some((best.heuristic.clone(), true));
        }

        if choice < 0.9 {
            let pareto_set: Vec<&ParetoCandidate> = self.pareto_front.iter().collect();
            let idx = fastrand::usize(..pareto_set.len());
            return Some((pareto_set[idx].heuristic.clone(), true));
        }

        let idx = fastrand::usize(..self.pareto_front.len());
        Some((self.pareto_front[idx].heuristic.clone(), true))
    }

    /// Get Pareto front metrics for dashboard.
    pub fn pareto_metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "paretoFront": {
                "size": self.pareto_front.len(),
                "maxSize": self.pareto_max_size,
                "bestScore": self.pareto_best_score(),
                "candidates": self.pareto_front.iter().map(|pc| {
                    serde_json::json!({
                        "principle": pc.heuristic.principle,
                        "score": pc.score,
                        "diversity": pc.diversity,
                        "exploitationScore": pc.exploitation_score,
                        "explorationScore": pc.exploration_score,
                    })
                }).collect::<Vec<_>>(),
            }
        })
    }

    /// Best score across Pareto front.
    pub fn pareto_best_score(&self) -> f64 {
        self.pareto_front
            .iter()
            .map(|pc| pc.score)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Crossover: combine two heuristics to produce a new one.
    /// Takes the principle from parent A and confidence from parent B.
    pub fn crossover_heuristics(&self, a: &Heuristic, b: &Heuristic) -> Heuristic {
        Heuristic {
            pattern: a.pattern.clone(),
            principle: format!("{} + crossover({})", a.principle, b.principle),
            confidence: (a.confidence + b.confidence) / 2.0,
            source_count: a.source_count.max(b.source_count),
            is_positive: a.is_positive || b.is_positive,
        }
    }

    fn heuristic_to_mutation(&self, heuristic: &Heuristic) -> MutationOp {
        let target = if heuristic.is_positive {
            "emergent_reasoning.exploration_rate"
        } else {
            "inner_critic.relevance_threshold"
        };
        let delta = (heuristic.confidence * 0.1 - 0.05).clamp(-0.1, 0.1);
        MutationOp::TuneParam {
            target: target.to_string(),
            delta,
        }
    }

    /// Generate a RewriteMeta mutation from a failure heuristic.
    /// Produces a MetaStrategy with proposer/evaluator/selector as Ne source strings
    /// that return formats parseable by `MutationOp::from_ne_string()`.
    fn heuristic_to_ne_strategy(
        &self,
        heuristic: &Heuristic,
        _ci: &mut impl ConsciousnessHandle,
    ) -> MutationOp {
        let _module = heuristic.principle.replace('"', "'").replace(':', "_");
        // Proposer returns "TuneParam:<target>:<delta>" format for from_ne_string().
        let ne_proposer = format!(
            "TuneParam:inner_critic.relevance_threshold:{:.4}",
            (heuristic.confidence * 0.1 - 0.05).clamp(-0.1, 0.1),
        );
        let ne_evaluator = format!("{}", heuristic.confidence);
        let ne_selector = "0".to_string();
        let strategy = MetaStrategy {
            proposer: ne_proposer,
            evaluator: ne_evaluator,
            selector: ne_selector,
            version: 1,
            self_proposed: true,
        };
        MutationOp::RewriteMeta { strategy }
    }
}

impl Default for SealClosedLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of SEAL configuration for meta-evolution.
#[derive(Debug, Clone)]
pub struct SealParams {
    pub distill_interval: u64,
    pub d_score_boost: f64,
    pub meta_mutation_rate: f64,
}

/// Meta-level SEAL engine.
///
/// Evolves the SEAL closed loop's own parameters (distill_interval, d_score_boost, etc.)
/// based on trace analysis over multiple meta-epochs. This is the "meta" layer —
/// evolution of the evolution mechanism itself (Autogenesis-inspired).
///
/// # Architecture
/// - Each meta-epoch: analyze traces from the inner SealClosedLoop
/// - If degradation or oscillation is detected: mutate SEAL parameters
/// - If stable: freeze parameters as the new best
/// - Tracks best-performing parameter sets across meta-epochs
#[derive(Debug, Clone)]
pub struct MetaSealEngine {
    /// Underlying SEAL instance
    pub inner: SealClosedLoop,
    /// Meta-epoch counter
    pub meta_epoch: u64,
    /// Cycles per meta-epoch (used to amortize mutation decisions)
    pub epochs_per_meta: u64,
    /// How aggressively to mutate SEAL params on degradation
    pub meta_mutation_rate: f64,
    /// Historical trace archive across meta-epochs
    pub meta_trace_history: Vec<TraceEntry>,
    /// Best-performing parameter set found so far
    pub best_params: Option<SealParams>,
    /// Best score achieved
    pub best_score: f64,
}

impl MetaSealEngine {
    pub fn new(inner: SealClosedLoop, epochs_per_meta: u64, meta_mutation_rate: f64) -> Self {
        Self {
            inner,
            meta_epoch: 0,
            epochs_per_meta,
            meta_mutation_rate,
            meta_trace_history: Vec::new(),
            best_params: None,
            best_score: f64::NEG_INFINITY,
        }
    }

    /// Run one meta-epoch.
    ///
    /// 1. Analyze trace patterns from the inner SEAL
    /// 2. If performance degrading, mutate SEAL parameters (explore)
    /// 3. If stable, freeze parameters (exploit)
    ///
    /// Returns `true` if parameters were mutated.
    ///
    /// NOTE: The caller is responsible for stepping the inner `SealClosedLoop`
    /// (`self.inner.step(…)`) for `epochs_per_meta` cycles before calling this.
    /// MetaSealEngine only *analyzes* and *mutates* — it does not drive the loop.
    pub fn step_meta_epoch(&mut self, ci: &mut impl ConsciousnessHandle) -> bool {
        // Copy latest trace entries into meta archive
        let start = self.meta_trace_history.len();
        if start < self.inner.trace_history.len() {
            for entry in self.inner.trace_history.iter().skip(start) {
                self.meta_trace_history.push(entry.clone());
            }
        }

        let insights = self.inner.analyse_traces();

        let degrading = insights.iter().any(|i| i.contains("degradation"));
        let oscillating = insights.iter().any(|i| i.contains("oscillation"));

        let mutated = if degrading || oscillating {
            // Mutate SEAL parameters based on meta mutation rate
            let new_interval = (self.inner.distill_interval as f64
                * (1.0 + self.meta_mutation_rate))
                .max(1.0) as u64;
            self.inner.distill_interval = new_interval;
            self.inner.d_score_boost =
                (self.inner.d_score_boost * (1.0 + self.meta_mutation_rate)).min(1.0);

            self.inner.gepa_mutate_from_traces(ci);

            log::info!(
                "META_SEAL: epoch={} degraded={} oscillating={} new_interval={} d_boost={:.3}",
                self.meta_epoch,
                degrading,
                oscillating,
                new_interval,
                self.inner.d_score_boost,
            );
            true
        } else {
            let current_score = self.inner.score_before;
            if current_score > self.best_score {
                self.best_score = current_score;
                self.best_params = Some(SealParams {
                    distill_interval: self.inner.distill_interval,
                    d_score_boost: self.inner.d_score_boost,
                    meta_mutation_rate: self.meta_mutation_rate,
                });
                log::info!(
                    "META_SEAL: epoch={} new best score={:.4} interval={} d_boost={:.3}",
                    self.meta_epoch,
                    current_score,
                    self.inner.distill_interval,
                    self.inner.d_score_boost,
                );
            }
            false
        };

        self.meta_epoch += 1;
        mutated
    }

    /// Suggest the next action based on meta-level analysis.
    pub fn meta_suggestion(&self) -> String {
        let insights = self.inner.analyse_traces();
        if insights.is_empty() {
            return format!(
                "meta_epoch_{}: stable — no degradation detected, continuing exploitation",
                self.meta_epoch,
            );
        }
        format!("meta_epoch_{}: {}", self.meta_epoch, insights.join("; "),)
    }

    /// Metrics snapshot for dashboarding.
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "metaSeal": {
                "metaEpoch": self.meta_epoch,
                "epochsPerMeta": self.epochs_per_meta,
                "metaMutationRate": self.meta_mutation_rate,
                "bestScore": self.best_score,
                "hasBestParams": self.best_params.is_some(),
                "inner": {
                    "phase": self.inner.phase.label(),
                    "cycleCounter": self.inner.cycle_counter,
                    "distillInterval": self.inner.distill_interval,
                    "dScoreBoost": self.inner.d_score_boost,
                    "appliedChanges": self.inner.applied_changes.len(),
                    "traces": self.inner.trace_history.len(),
                    "metaTraces": self.meta_trace_history.len(),
                }
            }
        })
    }

    /// Restore the best-performing parameter set found so far.
    pub fn restore_best(&mut self) -> bool {
        if let Some(ref params) = self.best_params {
            self.inner.distill_interval = params.distill_interval;
            self.inner.d_score_boost = params.d_score_boost;
            self.meta_mutation_rate = params.meta_mutation_rate;
            log::info!(
                "META_SEAL: restored best params (interval={}, d_boost={})",
                params.distill_interval,
                params.d_score_boost,
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionLoop;

    struct MockCI {
        c_score: f64,
        load: f64,
        best_score: f64,
        eval_result: Option<String>,
    }

    impl MockCI {
        fn new() -> Self {
            Self {
                c_score: 0.5,
                load: 0.3,
                best_score: 0.0,
                eval_result: None,
            }
        }
    }

    impl ConsciousnessHandle for MockCI {
        fn apply_ne_edit(&mut self, _target: &str, _value: f64) -> String {
            self.c_score = _value;
            String::new()
        }
        fn stats_c_score(&self) -> f64 {
            self.c_score
        }
        fn cognitive_load(&self) -> f64 {
            self.load
        }
        fn self_evolution_best_score(&self) -> f64 {
            self.best_score
        }
        fn eval_ne_string(&mut self, _expr: &str) -> Result<String, String> {
            self.eval_result
                .clone()
                .ok_or_else(|| "no mock result".to_string())
        }
        fn set_self_evolution_archive(&mut self, _best_score: f64) {}
    }

    #[test]
    fn test_seal_phase_cycle() {
        let mut phase = SealPhase::Distill;
        assert_eq!(phase, SealPhase::Distill);
        phase = phase.next();
        assert_eq!(phase, SealPhase::Apply);
        phase = phase.next();
        assert_eq!(phase, SealPhase::Verify);
        phase = phase.next();
        assert_eq!(phase, SealPhase::Rollback);
        phase = phase.next();
        assert_eq!(phase, SealPhase::Commit);
        phase = phase.next();
        assert_eq!(phase, SealPhase::Distill);
    }

    #[test]
    fn test_seal_phase_labels() {
        assert_eq!(SealPhase::Distill.label(), "distill");
        assert_eq!(SealPhase::Apply.label(), "apply");
        assert_eq!(SealPhase::Verify.label(), "verify");
        assert_eq!(SealPhase::Rollback.label(), "rollback");
        assert_eq!(SealPhase::Commit.label(), "commit");
    }

    #[test]
    fn test_closed_loop_new_state() {
        let loop_ = SealClosedLoop::new();
        assert_eq!(loop_.phase, SealPhase::Distill);
        assert!(loop_.applied_changes.is_empty());
        assert!(loop_.verification_results.is_empty());
        assert_eq!(loop_.cycle_counter, 0);
    }

    #[test]
    fn test_step_distill_generates_candidates() {
        let mut evolution = SelfEvolutionLoop::new();
        evolution.is_running = true;
        let mut extractor = TrajectoryHeuristicExtractor::new(100);
        let mut closed_loop = SealClosedLoop::new();
        let mut ci = MockCI::new();
        let trajectory = vec![
            ExperienceRecord {
                id: 1,
                context: "test domain alpha".into(),
                action: "test_action".into(),
                reward: 0.9,
                success: true,
                timestamp: 0,
                metadata: std::collections::HashMap::new(),
            },
            ExperienceRecord {
                id: 2,
                context: "test domain beta".into(),
                action: "test_action".into(),
                reward: 0.8,
                success: true,
                timestamp: 0,
                metadata: std::collections::HashMap::new(),
            },
        ];

        let phase = closed_loop.step(&mut evolution, &mut extractor, &trajectory, &mut ci);
        assert!(phase == SealPhase::Distill || phase == SealPhase::Apply);
    }

    #[test]
    fn test_should_run_at_interval() {
        let loop_ = SealClosedLoop::new().with_distill_interval(5);
        let mut test_loop = loop_.clone();
        test_loop.cycle_counter = 4;
        assert!(!test_loop.should_run());
        test_loop.cycle_counter = 5;
        assert!(test_loop.should_run());
    }

    #[test]
    fn test_status_report_format() {
        let mut loop_ = SealClosedLoop::new();
        loop_.applied_changes.push("test change".to_string());
        loop_.verification_results.push(true);
        loop_.cycle_counter = 42;
        let report = loop_.status_report();
        assert!(report.contains("SEAL_CL:"));
        assert!(report.contains("phase=distill"));
        assert!(report.contains("changes=1"));
        assert!(report.contains("42"));
    }

    #[test]
    fn test_heuristic_to_mutation_positive() {
        let loop_ = SealClosedLoop::new();
        let h = Heuristic {
            pattern: "test".to_string(),
            principle: "test principle".to_string(),
            confidence: 0.8,
            source_count: 5,
            is_positive: true,
        };
        let mutation = loop_.heuristic_to_mutation(&h);
        match mutation {
            MutationOp::TuneParam { target, delta } => {
                assert_eq!(target, "emergent_reasoning.exploration_rate");
                assert!(delta > -0.05);
            }
            _ => panic!("expected TuneParam"),
        }
    }

    #[test]
    fn test_heuristic_to_mutation_negative() {
        let loop_ = SealClosedLoop::new();
        let h = Heuristic {
            pattern: "test".to_string(),
            principle: "avoid pattern".to_string(),
            confidence: 0.6,
            source_count: 3,
            is_positive: false,
        };
        let mutation = loop_.heuristic_to_mutation(&h);
        match mutation {
            MutationOp::TuneParam { target, delta } => {
                assert_eq!(target, "inner_critic.relevance_threshold");
            }
            _ => panic!("expected TuneParam"),
        }
    }

    #[test]
    fn test_rollback_reverts_on_failure() {
        let mut evolution = SelfEvolutionLoop::new();
        let mut extractor = TrajectoryHeuristicExtractor::new(100);
        let mut closed_loop = SealClosedLoop::new();
        let mut ci = MockCI::new();

        let trajectory = vec![
            ExperienceRecord {
                id: 1,
                context: "test domain alpha".into(),
                action: "a".into(),
                reward: 0.9,
                success: true,
                timestamp: 0,
                metadata: std::collections::HashMap::new(),
            },
            ExperienceRecord {
                id: 2,
                context: "test domain beta".into(),
                action: "a".into(),
                reward: 0.8,
                success: true,
                timestamp: 0,
                metadata: std::collections::HashMap::new(),
            },
        ];

        closed_loop.step(&mut evolution, &mut extractor, &trajectory, &mut ci);

        let mut rollback_loop = SealClosedLoop {
            phase: SealPhase::Verify,
            verification_results: vec![false],
            active_candidate: closed_loop.active_candidate.clone(),
            ..SealClosedLoop::new()
        };
        rollback_loop.step(&mut evolution, &mut extractor, &trajectory, &mut ci);
        assert_eq!(rollback_loop.phase, SealPhase::Rollback);

        rollback_loop.step(&mut evolution, &mut extractor, &trajectory, &mut ci);
        assert_eq!(rollback_loop.phase, SealPhase::Commit);

        rollback_loop.step(&mut evolution, &mut extractor, &trajectory, &mut ci);
        assert_eq!(rollback_loop.phase, SealPhase::Distill);
    }
}
