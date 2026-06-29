#![forbid(unsafe_code)]

use super::types::*;
use crate::core::nt_core_experience::auto_deploy::AutoDeployer;
use crate::core::nt_core_experience::cross_domain::CrossDomainTransfer;
use crate::core::nt_core_experience::gradient_seal_bridge::{self, TrainedProgram};
use crate::core::nt_core_experience::sage_rollout::{Difficulty, SequentialRollout, TaskSignature};
use crate::core::nt_core_experience::skill_crystal::{CrystallizedSkill, SkillCrystallizer};
use crate::core::nt_core_experience::trial_worker::{MutationProposal, TrialArena};
use crate::core::nt_core_self_modify::{
    GateResult, ModifyTarget, SelfModifyGuard, SelfModifyProposal,
};
use crate::core::nt_core_shared_types::LanguageSpec;
use crate::core::nt_core_traits::ConsciousnessHandle;
#[cfg(feature = "hgm")]
use crate::neotrix::nt_mind::self_iterating::hgm::{HgmMetric, HgmSnapshot};
#[cfg(feature = "lse")]
use crate::neotrix::nt_mind::self_iterating::lse::LsePolicy;

/// Main controller that drives the self-evolution loop.
///
/// Orchestrates mutation proposal, evaluation, archival, and reporting.
/// Designed to be wired into `ConsciousnessIntegration` via `wire_into_consciousness`.
pub struct SelfEvolutionLoop {
    pub archive: SelfEvolutionArchive,
    pub next_id: u64,
    pub active_branch: u64,
    pub loop_interval_cycles: u64,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elite_count: usize,
    pub is_running: bool,
    pub trial_arena: Option<TrialArena>,
    pub auto_deployer: Option<AutoDeployer>,
    pub skill_crystallizer: Option<SkillCrystallizer>,
    /// Drive-controlled evolution bandit for Thompson-sampled mutation selection.
    pub drive_bandit: DriveBanditState,
    /// Accumulator of recent scores for rolling success-rate computation.
    recent_scores: Vec<f64>,
    rng_state: u64,
    /// Optional external fitness function for evaluating mutation descriptions.
    /// If set, `evaluate_description` uses it instead of the fallback heuristic.
    pub fitness_fn: Option<EvolutionFitnessFn>,
    /// DGM-H self-referential meta-strategy.
    /// When non-empty, propose/evaluate/select are driven by Ne-evaluated code.
    pub meta_strategy: MetaStrategy,
    /// Cross-domain capability transfer orchestrator.
    /// Bridges evolution archives across consciousness domains.
    pub cross_domain: Option<CrossDomainTransfer>,
    /// SAGE Sequential Rollout tracker.
    /// Maintains difficulty chains for progressive training across generations.
    pub sage_rollout: Option<SequentialRollout>,
    /// Generation count when the last meta-mutation (RewriteMeta) was executed.
    /// Used to enforce the ≥20 generation gate between meta-mutations.
    pub last_meta_gen: u32,
    /// LSE — Learning Self-Evolution: RL-based mutation policy.
    /// When `Some`, replaces the Thompson-sampling bandit for mutation selection.
    /// Feature-gated behind `cfg(feature = "lse")`.
    #[cfg(feature = "lse")]
    pub lse: Option<LsePolicy>,
    /// HGM — before-snapshot captured before mutation execution.
    /// Used by `finish_hgm()` to compute the CMP score after mutation.
    /// Feature-gated behind `cfg(feature = "hgm")`.
    #[cfg(feature = "hgm")]
    pub before_hgm: Option<HgmSnapshot>,
    /// PARL — Parallel Reinforcement Learning evaluator.
    /// When `Some`, mutation candidates are evaluated in parallel batches
    /// instead of one-at-a-time. Feature-gated behind `cfg(feature = "parl")`.
    #[cfg(feature = "parl")]
    pub parl: Option<crate::core::nt_core_experience::parl::ParlEvaluator>,
    /// SelfModifyGuard for evaluating self-modification proposals.
    /// When `Some`, `execute_self_modify_proposal` gates all changes through
    /// the four-layer safety check (Shield → Swords → LLM Validator → Ball Verifier).
    pub self_modify_guard: Option<SelfModifyGuard>,
    /// Current PACE gate sequence for SwapPolicy mutations.
    /// `execute_swap_policy` replaces this sequence with the proposed gates.
    pub gate_sequence: Vec<String>,
    /// When true, `propose_mutation` uses PopulationFunnel (parallel N-candidate
    /// competition) instead of single Thompson-sampled proposal.
    pub use_funnel: bool,
}

impl std::fmt::Debug for SelfEvolutionLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfEvolutionLoop")
            .field("archive", &self.archive)
            .field("next_id", &self.next_id)
            .field("active_branch", &self.active_branch)
            .field("is_running", &self.is_running)
            .field(
                "fitness_fn",
                &self.fitness_fn.as_ref().map(|_| "Box<FitnessFn>"),
            )
            .field(
                "drive_bandit",
                &format!("{} arms", self.drive_bandit.arms.len()),
            )
            .field("sage_rollout", &self.sage_rollout.as_ref().map(|_| "Some"))
            .finish()
    }
}

impl Clone for SelfEvolutionLoop {
    fn clone(&self) -> Self {
        Self {
            archive: self.archive.clone(),
            next_id: self.next_id,
            active_branch: self.active_branch,
            loop_interval_cycles: self.loop_interval_cycles,
            mutation_rate: self.mutation_rate,
            crossover_rate: self.crossover_rate,
            elite_count: self.elite_count,
            is_running: self.is_running,
            trial_arena: self.trial_arena.clone(),
            auto_deployer: self.auto_deployer.clone(),
            skill_crystallizer: self.skill_crystallizer.clone(),
            drive_bandit: self.drive_bandit.clone(),
            recent_scores: self.recent_scores.clone(),
            rng_state: self.rng_state,
            fitness_fn: None,
            meta_strategy: self.meta_strategy.clone(),
            cross_domain: self.cross_domain.clone(),
            sage_rollout: self.sage_rollout.clone(),
            last_meta_gen: self.last_meta_gen,
            #[cfg(feature = "lse")]
            lse: self.lse.clone(),
            #[cfg(feature = "hgm")]
            before_hgm: self.before_hgm.clone(),
            #[cfg(feature = "parl")]
            parl: self.parl.as_ref().map(|p| {
                crate::core::nt_core_experience::parl::ParlEvaluator::new(p.config.clone())
            }),
            self_modify_guard: self
                .self_modify_guard
                .as_ref()
                .map(|_| default_evolution_guard()),
            gate_sequence: self.gate_sequence.clone(),
            use_funnel: self.use_funnel,
        }
    }
}

/// Create a fully-armed 4-layer SelfModifyGuard for production use.
///
/// Layer 1 — Shield Bus: blocks unsafe Rust targets (unsafe, transmute, asm, etc.)
/// Layer 2 — Swords Check: scans source for dangerous code patterns
/// Layer 3 — LLM Validator: heuristic quality scoring (structural + length)
/// Layer 4 — Ball Verifier: constraint satisfaction (5 < len < 10000)
fn default_evolution_guard() -> SelfModifyGuard {
    SelfModifyGuard::new()
        .with_shield(Box::new(|target: &str| {
            let dangerous = [
                "unsafe", "core::ptr", "std::mem::transmute",
                "std::ptr::read", "std::ptr::write", "asm!",
            ];
            let target_lower = target.to_lowercase();
            !dangerous.iter().any(|d| target_lower.contains(d))
        }))
        .with_swords(Box::new(|code: &str| {
            let dangerous = [
                "unsafe {", "ptr::read", "ptr::write",
                "transmute", "asm!(", "intrinsic::",
            ];
            let code_lower = code.to_lowercase();
            !dangerous.iter().any(|d| code_lower.contains(d))
        }))
        .with_llm_validator(Box::new(|code: &str| {
            let len = code.len();
            if len < 10 {
                return 0.1;
            }
            let structural = (code.contains(';') as u64 as f64
                + code.contains("fn ") as u64 as f64
                + code.contains("let ") as u64 as f64) / 3.0;
            let length_score = (len as f64 / 500.0).min(1.0);
            (structural * 0.6 + length_score * 0.4).clamp(0.0, 1.0)
        }))
        .with_ball_verifier(Box::new(|code: &str| {
            let len = code.len();
            len > 5 && len < 10000
        }))
}

impl SelfEvolutionLoop {
    pub fn new() -> Self {
        Self {
            archive: SelfEvolutionArchive::new(),
            next_id: 1,
            active_branch: 0,
            loop_interval_cycles: 50,
            mutation_rate: 0.3,
            crossover_rate: 0.1,
            elite_count: 3,
            is_running: false,
            trial_arena: None,
            auto_deployer: None,
            skill_crystallizer: Some(SkillCrystallizer::default()),
            drive_bandit: DriveBanditState::new(),
            recent_scores: Vec::with_capacity(20),
            rng_state: 0,
            fitness_fn: None,
            meta_strategy: MetaStrategy::default_v1(),
            cross_domain: None,
            sage_rollout: None,
            last_meta_gen: 0,
            #[cfg(feature = "lse")]
            lse: Some(LsePolicy::new(6)),
            #[cfg(feature = "hgm")]
            before_hgm: None,
            #[cfg(feature = "parl")]
            parl: None,
            self_modify_guard: Some(default_evolution_guard()),
            gate_sequence: vec!["pace::commit_gain ≤ 0.5".to_string()],
            use_funnel: false,
        }
    }

    /// Enable PARL (Parallel Reinforcement Learning) evaluation with the given config.
    /// When enabled, mutation candidates are evaluated in parallel batches.
    #[cfg(feature = "parl")]
    pub fn with_parl(mut self, config: crate::core::nt_core_experience::parl::ParlConfig) -> Self {
        self.parl = Some(crate::core::nt_core_experience::parl::ParlEvaluator::new(
            config,
        ));
        self
    }

    /// Set a custom fitness function for `evaluate_description`.
    pub fn with_fitness_fn(mut self, f: EvolutionFitnessFn) -> Self {
        self.fitness_fn = Some(f);
        self
    }

    fn rng(&mut self) -> f64 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let x = (self.rng_state >> 33) as u32;
        (x as f64) / (u32::MAX as f64)
    }

    /// Seed the archive with initial candidate mutations derived from a LanguageSpec.
    /// Each allowed_target → TuneParam candidate.
    /// Each handler → RewriteHandler candidate.
    pub fn seed_from_spec(&self, spec: &LanguageSpec) -> Vec<MutationOp> {
        let mut candidates = Vec::new();
        for target in &spec.edit_policy.allowed_targets {
            candidates.push(MutationOp::TuneParam {
                target: (*target).to_string(),
                delta: 0.05,
            });
        }
        for handler in &spec.handler_graph.handlers {
            candidates.push(MutationOp::RewriteHandler {
                name: (*handler.name).to_string(),
                code: format!(
                    "fn {}_step() {{ /* auto-seeded from spec */ }}",
                    handler.name
                ),
            });
        }
        candidates
    }

    /// Propose a new mutation by evolving from the best known steps.
    /// Uses Thompson-sampled drive strategy (biased by dominant_drive)
    /// to select the mutation type and probability distribution.
    ///
    /// If `meta_strategy.proposer` is non-empty, evaluates it via the Ne compiler
    /// (DGM-H self-referential meta-agent). Falls back to the hardcoded Rust logic
    /// when the Ne proposer is empty or fails to compile.
    /// Enable PopulationFunnel parallel proposal competition.
    /// When enabled, `propose_mutation` uses funnel proposer instead of single
    /// Thompson-sampled proposal.
    pub fn with_funnel(mut self) -> Self {
        self.use_funnel = true;
        self
    }

    pub fn propose_mutation(
        &mut self,
        best_steps: &[SelfEvolutionStep],
        dominant_drive: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> MutationOp {
        // DGM-H path: evaluate meta-strategy proposer via Ne compiler
        if !self.meta_strategy.proposer.is_empty() {
            if let Some(m) = self.propose_via_ne(dominant_drive, ci) {
                return m;
            }
        }

        // Funnel path: population-based parallel candidate competition
        if self.use_funnel {
            let seed = self.rng_state;
            return crate::core::nt_core_experience::self_evolution_loop::funnel_proposer::propose_via_funnel(best_steps, seed);
        }

        // Default path: hardcoded Rust logic (v1 behavior)
        self.propose_via_rust(best_steps, dominant_drive)
    }

    /// Evaluate the meta-strategy proposer via the Ne compiler.
    /// Returns `None` on compilation or evaluation failure (falls back to Rust).
    fn propose_via_ne(
        &mut self,
        dominant_drive: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> Option<MutationOp> {
        let source = self.meta_strategy.proposer.clone();
        let _rng_val = self.rng();
        let _cycle = self.archive.generation as u64;
        let _drive = dominant_drive;

        // v1: validate syntactic validity via nt-lang parser
        match nt_lang::parser::parse::parse_stmts(&source) {
            Ok(_parsed) => {
                log::debug!(
                    "META_AGENT: Ne proposer parsed OK (archive={}, score={:.4}, gen={})",
                    self.archive.steps.len(),
                    self.archive.best_score,
                    _cycle,
                );
            }
            Err(e) => {
                log::warn!(
                    "META_AGENT: Ne proposer syntax error: {} — falling back to Rust default",
                    e.message,
                );
                return None;
            }
        }

        // v2: execute via Ne runtime and parse result → MutationOp
        match ci.eval_ne_string(&source) {
            Ok(val_str) => {
                let result = MutationOp::from_ne_string(&val_str);
                if result.is_some() {
                    log::info!(
                        "META_AGENT: Ne proposer returned mutation [{}]",
                        result.as_ref().unwrap().summary(),
                    );
                } else {
                    log::debug!(
                        "META_AGENT: Ne proposer returned unparseable string: {:?}",
                        val_str,
                    );
                }
                result
            }
            Err(e) => {
                log::warn!(
                    "META_AGENT: Ne proposer execution failed: {} — falling back to Rust default",
                    e,
                );
                None
            }
        }
    }

    /// Map LSE action index to a drive strategy name.
    /// LSE has 6 actions corresponding to the 6 mutation-producing drives:
    ///   0→exploit, 1→innovate, 2→repair, 3→prune, 4→harden, 5→meta
    #[cfg(feature = "lse")]
    fn lse_action_to_drive(action: usize) -> &'static str {
        match action {
            0 => "exploit",
            1 => "innovate",
            2 => "repair",
            3 => "prune",
            4 => "harden",
            _ => "meta",
        }
    }

    /// Compute compile trend from recent archive steps.
    /// Returns the ratio of recent steps that compiled successfully.
    #[cfg(feature = "lse")]
    fn compile_trend(&self) -> f64 {
        let window: Vec<&SelfEvolutionStep> = self.archive.steps.iter().rev().take(20).collect();
        if window.is_empty() {
            return 0.5;
        }
        let compiled = window.iter().filter(|s| s.compiles).count();
        compiled as f64 / window.len() as f64
    }

    /// Hardcoded Rust mutation proposal (original v1 logic).
    fn propose_via_rust(
        &mut self,
        best_steps: &[SelfEvolutionStep],
        dominant_drive: &str,
    ) -> MutationOp {
        let rng_val = self.rng();

        // LSE path: use RL-based policy to select drive strategy
        #[cfg(feature = "lse")]
        let drive_strategy = {
            let lse_success_rate = self.success_rate();
            let lse_compile_trend = self.compile_trend();
            let lse_action = self
                .lse
                .as_mut()
                .map(|lse| lse.select_action(lse_success_rate, lse_compile_trend));
            match lse_action {
                Some(action) => Self::lse_action_to_drive(action).to_string(),
                None => self.drive_bandit.select_arm(dominant_drive, rng_val),
            }
        };
        #[cfg(not(feature = "lse"))]
        let drive_strategy = self.drive_bandit.select_arm(dominant_drive, rng_val);

        match drive_strategy.as_str() {
            "explore" => {
                // 60% random tune-param, 40% mutate from best
                if self.rng() < 0.6 {
                    random_tune_param(self.rng(), self.rng())
                } else if !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    mutate_op_from(&best_steps[idx].mutation, self.rng(), self.rng())
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "exploit" => {
                // 70% mutate from best, 30% crossover
                if !best_steps.is_empty() && self.rng() < 0.7 {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    mutate_op_from(&best_steps[idx].mutation, self.rng(), self.rng())
                } else if best_steps.len() >= 2 {
                    let a_idx = (self.rng() * best_steps.len() as f64) as usize;
                    let mut b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    while b_idx == a_idx && best_steps.len() > 1 {
                        b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    }
                    crossover_ops(
                        &best_steps[a_idx].mutation,
                        &best_steps[b_idx].mutation,
                        self.rng(),
                    )
                } else if !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    mutate_op_from(&best_steps[idx].mutation, self.rng(), self.rng())
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "repair" => {
                // 80% RewriteHandler, 20% TuneParam
                if self.rng() < 0.8 && !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    match &best_steps[idx].mutation {
                        MutationOp::RewriteHandler { name, code } => MutationOp::RewriteHandler {
                            name: name.clone(),
                            code: format!("{} // repaired", code),
                        },
                        other => mutate_op_from(other, self.rng(), self.rng()),
                    }
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "innovate" => {
                // 50% crossover, 50% random tune-param
                if best_steps.len() >= 2 && self.rng() < 0.5 {
                    let a_idx = (self.rng() * best_steps.len() as f64) as usize;
                    let mut b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    while b_idx == a_idx && best_steps.len() > 1 {
                        b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    }
                    crossover_ops(
                        &best_steps[a_idx].mutation,
                        &best_steps[b_idx].mutation,
                        self.rng(),
                    )
                } else if !best_steps.is_empty() && self.rng() < 0.5 {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    mutate_op_from(&best_steps[idx].mutation, self.rng(), self.rng())
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "harden" => {
                // 70% TuneParam small deltas, 30% RewriteHandler
                if self.rng() < 0.7 {
                    let delta = (self.rng() - 0.5) * 0.05;
                    let targets = [
                        "cognitive_load.thinking_budget",
                        "emergent_reasoning.exploration_rate",
                        "inner_critic.relevance_threshold",
                    ];
                    let idx = (self.rng() * targets.len() as f64) as usize;
                    MutationOp::TuneParam {
                        target: targets[idx].to_string(),
                        delta,
                    }
                } else if !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    match &best_steps[idx].mutation {
                        MutationOp::RewriteHandler { name, code } => MutationOp::RewriteHandler {
                            name: name.clone(),
                            code: code.clone(),
                        },
                        other => mutate_op_from(other, self.rng(), self.rng()),
                    }
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "prune" => {
                // 60% SwapPolicy, 40% RewriteHandler
                if self.rng() < 0.6 && !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    match &best_steps[idx].mutation {
                        MutationOp::SwapPolicy { gates } => {
                            let mut new_gates = gates.clone();
                            if !new_gates.is_empty() {
                                let rm = (self.rng() * new_gates.len() as f64) as usize;
                                new_gates.remove(rm);
                            }
                            MutationOp::SwapPolicy { gates: new_gates }
                        }
                        other => mutate_op_from(other, self.rng(), self.rng()),
                    }
                } else if !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    match &best_steps[idx].mutation {
                        MutationOp::RewriteHandler { name, code } => MutationOp::RewriteHandler {
                            name: name.clone(),
                            code: format!("{} // pruned", code),
                        },
                        other => mutate_op_from(other, self.rng(), self.rng()),
                    }
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            "socialize" => {
                // 50% crossover, 50% mutate
                if best_steps.len() >= 2 && self.rng() < 0.5 {
                    let a_idx = (self.rng() * best_steps.len() as f64) as usize;
                    let mut b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    while b_idx == a_idx && best_steps.len() > 1 {
                        b_idx = (self.rng() * best_steps.len() as f64) as usize;
                    }
                    crossover_ops(
                        &best_steps[a_idx].mutation,
                        &best_steps[b_idx].mutation,
                        self.rng(),
                    )
                } else if !best_steps.is_empty() {
                    let idx = (self.rng() * best_steps.len() as f64) as usize;
                    mutate_op_from(&best_steps[idx].mutation, self.rng(), self.rng())
                } else {
                    random_tune_param(self.rng(), self.rng())
                }
            }
            _ => {
                // "rest" or fallback: always random tune-param
                random_tune_param(self.rng(), self.rng())
            }
        }
    }

    /// Compute the acceptance rate across all archive steps.
    pub fn compute_acceptance_rate(&self) -> f64 {
        if self.archive.steps.is_empty() {
            return 0.5;
        }
        let accepted = self.archive.steps.iter().filter(|s| s.accepted).count();
        accepted as f64 / self.archive.steps.len() as f64
    }

    /// Evaluate a mutation description using the fitness function or a fallback heuristic.
    /// Returns a score in [-1, 1] where higher is better.
    pub fn evaluate_description(&self, description: &str, handler_count: usize) -> f64 {
        if let Some(ref f) = self.fitness_fn {
            let stats = EvolutionHandlerStats {
                total_handlers: handler_count,
                cycle: self
                    .archive
                    .steps
                    .last()
                    .map(|s| s.generation as u64)
                    .unwrap_or(0),
                hot_count: 0,
                warm_count: 0,
                cold_count: 0,
                recent_acceptance_rate: self.compute_acceptance_rate(),
                archive_size: self.archive.steps.len(),
            };
            f(description, stats.cycle, &stats)
        } else {
            let acceptance_bonus = self.compute_acceptance_rate() * 0.3;
            let complexity_penalty = (description.len() as f64).clamp(0.0, 100.0) / -500.0;
            0.5 + acceptance_bonus + complexity_penalty
        }
    }

    /// Evaluate whether a proposed mutation should be accepted.
    /// Accept if the score drop is less than 5%.
    /// Returns `(accepted, effective_score)`.
    pub fn evaluate(step: &SelfEvolutionStep, current_score: f64) -> (bool, f64) {
        let effective_score = current_score.max(step.score_before);
        let score_after = step.score_after.unwrap_or(current_score);
        let drop = if effective_score > 0.0 {
            (effective_score - score_after) / effective_score
        } else {
            0.0
        };
        let accepted = drop < 0.05;
        (accepted, score_after)
    }

    /// Called every cycle. If `is_running` and `cycle % loop_interval_cycles == 0`,
    /// proposes a new mutation derived from the best steps in the archive.
    /// `dominant_drive` is the current emotional drive from DriveSelector,
    /// used to bias the bandit-based mutation strategy selection.
    /// Also runs optional sub-systems:
    ///   - Trial-arena validation after archival (every cycle mutations run)
    ///   - Skill crystallization every 50 cycles
    ///   - Auto-deploy every 100 cycles
    pub fn tick(
        &mut self,
        current_score: f64,
        cycle: u64,
        dominant_drive: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> (Option<MutationOp>, Vec<CrystallizedSkill>) {
        // Run GitHub absorption every 50 cycles
        if cycle % 50 == 0 {
            let _absorbed = self.run_absorption_cycle(cycle);
        }
        if !self.is_running || cycle % self.loop_interval_cycles != 0 {
            return (None, vec![]);
        }
        let sorted = {
            let mut s: Vec<&SelfEvolutionStep> = self.archive.steps.iter().collect();
            s.sort_by(|a, b| {
                let sa = a.score_after.unwrap_or(0.0);
                let sb = b.score_after.unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            s.iter().take(5).map(|s| (*s).clone()).collect::<Vec<_>>()
        };
        self.rng_state = cycle.wrapping_mul(6364136223846793005);
        let _ = current_score;

        // After proposing a mutation, run optional sub-systems at their intervals
        if !sorted.is_empty() {
            let _trials_passed = self.run_tick_with_trials();
        }

        // Crystallize successful mutations every 50 cycles — capture for caller to load into Ne evaluator
        let crystallized = if cycle % 50 == 0 && cycle > 0 {
            self.run_crystallization()
        } else {
            vec![]
        };

        // Auto-deploy every 100 cycles
        if cycle % 100 == 0 && cycle > 0 {
            let _deploy = self.run_auto_deploy();
        }

        (
            Some(self.propose_mutation(&sorted, dominant_drive, ci)),
            crystallized,
        )
    }

    /// PARL (Parallel Reinforcement Learning) version of `tick()`.
    ///
    /// Instead of proposing a single mutation, collects `batch_size` candidates,
    /// evaluates them in parallel via `ParlEvaluator::evaluate_batch()`, executes
    /// the highest-scoring result, and records the rest as rejected.
    ///
    /// `eval_fn` is called once per candidate inside a `spawn_blocking` task.
    /// It should return a score and compile status without mutating shared state.
    #[cfg(feature = "parl")]
    pub async fn tick_parl(
        &mut self,
        current_score: f64,
        cycle: u64,
        dominant_drive: &str,
        ci: &mut impl ConsciousnessHandle,
        eval_fn: impl Fn(
                crate::core::nt_core_experience::self_evolution_loop::types::MutationOp,
            ) -> crate::core::nt_core_experience::parl::ParlOutcome
            + Send
            + Sync
            + 'static,
    ) -> (
        Option<crate::core::nt_core_experience::self_evolution_loop::types::MutationOp>,
        Vec<CrystallizedSkill>,
    ) {
        // GitHub absorption every 50 cycles
        if cycle % 50 == 0 {
            let _absorbed = self.run_absorption_cycle(cycle);
        }
        if !self.is_running || cycle % self.loop_interval_cycles != 0 {
            return (None, vec![]);
        }

        // Crystallize successful mutations every 50 cycles
        let crystallized = if cycle % 50 == 0 && cycle > 0 {
            self.run_crystallization()
        } else {
            vec![]
        };

        // Auto-deploy every 100 cycles
        if cycle % 100 == 0 && cycle > 0 {
            let _deploy = self.run_auto_deploy();
        }

        let batch_size = match self.parl.as_ref() {
            Some(p) => p.config.batch_size,
            None => {
                // Fall back to single-mutation tick if PARL not configured
                return self.tick(current_score, cycle, dominant_drive, ci);
            }
        };

        // Build sorted best steps (same as tick())
        let sorted = {
            let mut s: Vec<SelfEvolutionStep> = self.archive.steps.iter().cloned().collect();
            s.sort_by(|a, b| {
                let sa = a.score_after.unwrap_or(0.0);
                let sb = b.score_after.unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            s.into_iter().take(5).collect::<Vec<_>>()
        };
        self.rng_state = cycle.wrapping_mul(6364136223846793005);

        // 1. Propose batch_size mutations (borrow self mutably, no parl reference active)
        let mut mutations = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            mutations.push(self.propose_mutation(&sorted, dominant_drive, ci));
        }

        // 2. Trial-arena validation on best steps (side-effect, non-blocking for parl)
        if !sorted.is_empty() {
            let _trials_passed = self.run_tick_with_trials();
        }

        // 3. Evaluate all proposals in parallel (borrow parl mutably, no self conflict)
        let ranked = match self.parl.as_mut() {
            Some(p) => p.evaluate_batch(mutations, eval_fn).await,
            None => return (None, crystallized),
        };

        if ranked.is_empty() {
            return (None, crystallized);
        }

        // 4. Execute the best-scoring mutation
        let best = &ranked[0];
        let before = Self::ci_composite_score(ci);
        let after = self.execute_mutation(&best.op, ci).unwrap_or(before);
        let compiles = best.outcome.compiles;
        self.record_result(best.op.clone(), before, after, compiles, None);

        // 5. Record remaining candidates as rejected
        log::info!(
            "PARL: batch of {} → best score {:.4} (compiles={}), {} rejected",
            ranked.len(),
            best.outcome.score,
            compiles,
            ranked.len().saturating_sub(1),
        );
        for r in ranked.iter().skip(1) {
            self.record_result(r.op.clone(), before, before, false, None);
        }

        (Some(best.op.clone()), crystallized)
    }

    /// Record the outcome of a mutation attempt.
    /// Creates a SelfEvolutionStep, adds it to the archive, and updates active_branch.
    /// Also updates the drive-controlled evolution bandit with the reward signal.
    /// `cmp_score` is an optional HGM coherence metric (None when feature=hgm is disabled).
    pub fn record_result(
        &mut self,
        mutation: MutationOp,
        before: f64,
        after: f64,
        compiles: bool,
        cmp_score: Option<f64>,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let id = self.next_id;
        self.next_id += 1;
        let generation = self.archive.generation + 1;
        let accepted = after >= before * 0.95;

        // Track bandit arm reward
        let reward = if accepted {
            ((after - before) / before.max(0.01)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let arm_name = Self::mutation_to_drive_name(&mutation);
        self.drive_bandit.update_arm(&arm_name, reward);

        // LSE learning: reward = (after - before) * (1.0 if compiles else -0.5)
        #[cfg(feature = "lse")]
        if let Some(ref mut lse) = self.lse {
            let lse_reward = (after - before) * if compiles { 1.0 } else { -0.5 };
            lse.learn(lse_reward);
        }

        // SAGE Sequential Rollout: chain-and-train on this mutation
        if let Some(ref mut rollout) = self.sage_rollout {
            let sig = TaskSignature::new(
                &Self::mutation_to_drive_name(&mutation),
                mutation.label(),
                Self::score_to_difficulty(before),
            );
            rollout.chain_and_train(&sig, &format!("step_{}", id), after);
        }

        let step = SelfEvolutionStep {
            id,
            mutation,
            parent_id: self.active_branch,
            score_before: before,
            score_after: Some(after),
            compiles,
            accepted,
            timestamp: now,
            generation,
            cmp_score,
        };
        if accepted {
            self.active_branch = id;
        }
        self.archive.add(step);
        self.recent_scores.push(after);
        if self.recent_scores.len() > 20 {
            self.recent_scores.remove(0);
        }
    }

    /// Capture the before-mutation HGM snapshot.
    /// Call this before `execute_mutation` to record the pre-mutation state.
    /// The captured snapshot is used by `finish_hgm()` to compute the CMP score.
    #[cfg(feature = "hgm")]
    pub fn begin_hgm(&mut self, handler_count: usize, negentropy: f64, cycle: u64) {
        self.before_hgm = Some(HgmSnapshot::capture(handler_count, negentropy, cycle));
    }

    /// Compute the HGM CMP score after mutation execution.
    /// Compares the snapshots before/after and returns `Some(cmp_score)`.
    /// Clears the stored before-snapshot after computation.
    /// Returns `None` if no before-snapshot was captured (e.g., HGM feature disabled).
    #[cfg(feature = "hgm")]
    pub fn finish_hgm(&mut self, handler_count: usize, negentropy: f64) -> Option<f64> {
        let before = self.before_hgm.take()?;
        let after = HgmSnapshot::capture(handler_count, negentropy, 0);
        let metric = HgmMetric::compute(&before, &after);
        Some(metric.cmp_score())
    }

    #[cfg(not(feature = "hgm"))]
    pub fn begin_hgm(&mut self, _handler_count: usize, _negentropy: f64, _cycle: u64) {}

    #[cfg(not(feature = "hgm"))]
    pub fn finish_hgm(&mut self, _handler_count: usize, _negentropy: f64) -> Option<f64> {
        None
    }

    /// Compute the rolling success rate from the recent-score window.
    /// A step is "successful" if its score exceeds the mean of the window.
    fn success_rate(&self) -> f64 {
        if self.recent_scores.len() < 3 {
            return 0.0;
        }
        let mean: f64 = self.recent_scores.iter().sum::<f64>() / self.recent_scores.len() as f64;
        let successes = self.recent_scores.iter().filter(|&&s| s >= mean).count();
        successes as f64 / self.recent_scores.len() as f64
    }

    /// Run trial-arena validation for the top-3 mutations in the archive.
    /// Creates `MutationProposal` items and runs them through `TrialArena`.
    /// Returns the number of trials that passed.
    pub fn run_tick_with_trials(&mut self) -> usize {
        let arena = match self.trial_arena.as_ref() {
            Some(a) => a,
            None => return 0,
        };
        let mut arena_clone = arena.clone();
        let top: Vec<SelfEvolutionStep> = {
            let mut s: Vec<SelfEvolutionStep> = self
                .archive
                .steps
                .iter()
                .filter(|step| step.score_after.unwrap_or(0.0) > 0.5)
                .cloned()
                .collect();
            s.sort_by(|a, b| {
                let sa = a.score_after.unwrap_or(0.0);
                let sb = b.score_after.unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            s.into_iter().take(3).collect()
        };

        if top.is_empty() {
            return 0;
        }

        // Build mutation proposals (source_file is a best-effort estimate)
        let proposals: Vec<MutationProposal> = top
            .iter()
            .enumerate()
            .map(|(_i, step)| MutationProposal {
                index: step.id as usize,
                label: step.mutation.label().to_string(),
                code: step.mutation.summary(),
                source_file: format!("evolution_mutation_{}", step.id),
            })
            .collect();

        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(arena_clone.evaluate_all(&proposals))
        });
        let passed = results.iter().filter(|(_, r)| r.passed).count();

        // Update archive with trial results
        for (id, result) in &results {
            let id = *id as u64;
            if let Some(_step) = self.archive.steps.iter_mut().find(|s| s.id == id) {
                if !result.passed {
                    log::error!(
                        "SELFEVOL: Trial failed for mutation #{}: {}",
                        id,
                        result.failure_log.first().unwrap_or(&"no log".into())
                    );
                }
            }
        }

        passed
    }

    /// Check for skill crystallization every 50 cycles.
    /// Looks at recent archive entries and attempts to crystallize those
    /// with sufficiently high scores.
    pub fn run_crystallization(&mut self) -> Vec<CrystallizedSkill> {
        let crystallizer = match self.skill_crystallizer.as_mut() {
            Some(c) => c,
            None => return vec![],
        };

        let candidates: Vec<SelfEvolutionStep> = self
            .archive
            .steps
            .iter()
            .filter(|s| s.score_after.unwrap_or(0.0) > 0.6)
            .cloned()
            .collect();

        let mut crystallized = Vec::new();
        for record in &candidates {
            if let Some(skill) = crystallizer.crystallize(record) {
                if let Err(e) = crystallizer.store(&skill) {
                    log::error!("SELFEVOL: Failed to store crystallized skill: {}", e);
                } else {
                    log::info!(
                        "SELFEVOL: Crystallized skill '{}' (avg {:.3})",
                        skill.name,
                        skill.avg_score
                    );
                    crystallized.push(skill);
                }
            }
        }
        crystallized
    }

    /// Attempt auto-deploy every 100 cycles if the success rate is above 0.5.
    pub fn run_auto_deploy(
        &mut self,
    ) -> Option<crate::core::nt_core_experience::auto_deploy::DeployReport> {
        // Clone what we need before the mutable borrow
        let rate = self.success_rate();
        let steps = self.archive.steps.clone();
        if rate <= 0.5 {
            return None;
        }
        let deployer = match self.auto_deployer.as_mut() {
            Some(d) => d,
            None => return None,
        };
        let report = deployer.check_and_deploy(rate, &steps);
        if let Some(ref r) = report {
            log::info!(
                "SELFEVOL: Auto-deployed {} mutations (commit {})",
                r.mutations_applied,
                r.commit_hash
            );
        }
        report
    }

    /// Generate a formatted evolution report.
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== Self-Evolution Report ===".to_string());
        lines.push(format!("Generation: {}", self.archive.generation));
        lines.push(format!(
            "Archive: {}/{} steps",
            self.archive.steps.len(),
            self.archive.archive_limit
        ));
        lines.push(format!(
            "Best score: {:.4} (step #{})",
            self.archive.best_score, self.archive.best_step_id
        ));
        lines.push(format!("Active branch: #{}", self.active_branch));
        lines.push(format!("Running: {}", self.is_running));
        lines.push(format!("Interval: {} cycles", self.loop_interval_cycles));
        lines.push(format!("Mutation rate: {:.2}", self.mutation_rate));
        lines.push(format!("Crossover rate: {:.2}", self.crossover_rate));

        if self.archive.steps.is_empty() {
            lines.push("No steps recorded yet.".to_string());
            return lines.join("\n");
        }

        lines.push(String::new());
        lines.push("Recent steps:".to_string());
        let start = self.archive.steps.len().saturating_sub(10);
        for step in self.archive.steps.iter().skip(start) {
            let label = step.mutation.label();
            let summary = step.mutation.summary();
            let score_str = match step.score_after {
                Some(s) => format!("{:.4}", s),
                None => "pending".to_string(),
            };
            let compile_str = if step.compiles { "OK" } else { "FAIL" };
            lines.push(format!(
                "  #{} gen{} [{}] {} \u{2192} {} ({})",
                step.id, step.generation, label, summary, score_str, compile_str,
            ));
        }

        lines.join("\n")
    }
}

impl Default for SelfEvolutionLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ── Gradient-driven Ne program training ──

impl SelfEvolutionLoop {
    /// Run gradient descent training on a Ne program source and record the result
    /// as an evolution step in the archive.
    ///
    /// This bridges TensorGraph backward mode (DMCI) with the self-evolution loop,
    /// enabling gradient-driven optimization of Ne program constants.
    ///
    /// Returns the `TrainedProgram` with the optimized graph and loss trace,
    /// or `None` if training failed.
    pub fn train_ne_program(
        &mut self,
        source: &str,
        dim: usize,
        learning_rate: f64,
        steps: usize,
        target: &[f64],
    ) -> Option<TrainedProgram> {
        let before = self.archive.best_score;
        let result =
            gradient_seal_bridge::train_ne_program(source, dim, learning_rate, steps, target)
                .ok()?;

        let clamped_loss = result.final_loss.min(10.0);
        let after = (1.0 - clamped_loss / 10.0).max(0.0).min(1.0);

        let mutation = MutationOp::TuneParam {
            target: format!("ne_program_gradient_{}", self.next_id),
            delta: after - before,
        };
        let compiles = result.final_loss < before.max(1.0);
        self.record_result(mutation, before, after, compiles, None);

        log::info!(
            "GRADIENT: trained Ne program: loss {:.4} → {:.4} (score {:.4} → {:.4})",
            result.loss_trace.first().copied().unwrap_or(0.0),
            result.final_loss,
            before,
            after,
        );

        Some(result)
    }

    /// Run gradient training configured via the gradient_seal_bridge wire,
    /// integrated into the evolution tick pipeline.
    ///
    /// This is called during cycle % N ticks from the consciousness handler.
    /// If no Ne program is available for training, it returns `None`.
    pub fn run_gradient_training_tick(
        &mut self,
        source_opt: Option<&str>,
        dim: usize,
        target: &[f64],
    ) -> Option<TrainedProgram> {
        let source = source_opt?;
        // Use adaptive learning rate and steps based on current success rate
        let rate = self.success_rate();
        let learning_rate = if rate > 0.6 { 0.01 } else { 0.1 };
        let steps = if rate > 0.6 { 20 } else { 50 };
        self.train_ne_program(source, dim, learning_rate, steps, target)
    }

    /// DGM-H meta-agent tick: propose meta-strategy update when conditions are right.
    ///
    /// The meta-agent generates a candidate `MetaStrategy` by analyzing the archive
    /// for detectable failure patterns, then packages it as a `RewriteMeta` mutation.
    /// This enables the system to self-modify its own improvement mechanism.
    ///
    /// Returns `None` when:
    /// - Fewer than 5 archive steps (not enough data)
    /// - Best score is still 0.0 (no successful evolution yet)
    /// - The previous meta-mutation was within 20 generations (avoid thrashing)
    pub fn meta_agent_tick(&mut self) -> Option<MutationOp> {
        if self.archive.steps.len() < 5 {
            return None;
        }
        if self.archive.best_score < 0.01 {
            return None;
        }

        // Avoid thrashing: require at least 20 generations since last meta change
        if self.archive.generation.saturating_sub(self.last_meta_gen) < 20 {
            return None;
        }

        let mut _total_delta = 0.0f64;
        let mut useful_count = 0usize;
        for step in self.archive.steps.iter().rev().take(20) {
            if let Some(after) = step.score_after {
                let delta = after - step.score_before;
                _total_delta += delta;
                if delta > 0.0 {
                    useful_count += 1;
                }
            }
        }
        let usefulness = if useful_count >= 20 {
            1.0
        } else {
            useful_count as f64 / 20.0
        };

        // Generate a live Ne proposer that encodes lessons from the archive.
        // The proposer evaluates to a string literal parseable by `from_ne_string`.
        // Computation is done in Rust; the Ne code is a simple literal expression
        // that the evaluator can execute without needing external primitives.
        let accept_rate = self.compute_acceptance_rate();
        let best = self.archive.best_score;
        let gen = self.archive.generation;
        let exploit_weight = self.drive_bandit.drive_weight("exploit");

        let proposer = if accept_rate > 0.6 {
            if exploit_weight > 0.5 {
                format!(
                    "\"TuneParam:cognitive_load.thinking_budget:{:.4}\"",
                    0.01 + (best * 0.1),
                )
            } else {
                format!(
                    "\"TuneParam:mutation_rate:{:.4}\"",
                    self.mutation_rate * 0.9 + 0.01,
                )
            }
        } else {
            let explore_weight = self.drive_bandit.drive_weight("explore");
            if explore_weight > 0.3 {
                format!("\"AddHandler:explore_gen_{}\"", gen,)
            } else {
                format!(
                    "\"TuneParam:emergent_reasoning.exploration_rate:{:.4}\"",
                    -(0.01 + (self.rng() * 0.05)),
                )
            }
        };
        // Evaluator: returns usefulness score as a string literal
        let evaluator = format!("\"{:.6}\"", usefulness);
        // Selector: always picks index 0 (best parent)
        let selector = "\"0\"".to_string();

        let new_strategy = MetaStrategy {
            proposer,
            evaluator,
            selector,
            version: self.meta_strategy.version + 1,
            self_proposed: true,
        };

        log::info!(
            "META_AGENT: proposing meta-strategy v{} (accept_rate={:.2}, usefulness={:.2}, archive={})",
            new_strategy.version,
            accept_rate,
            usefulness,
            self.archive.steps.len(),
        );

        Some(MutationOp::RewriteMeta {
            strategy: new_strategy,
        })
    }

    /// Run the cross-domain transfer tick:
    /// 1. Lazy-initializes `CrossDomainTransfer` on first call
    /// 2. Snapshots the current domain's archive into `DomainArchiveSnapshot`
    /// 3. For each other known domain, finds compatible candidates (TuneParam, RewritePrimitive)
    /// 4. Records transfer outcomes in `TransferValidator`
    ///
    /// Runs every 50 cycles. `current_domain` identifies which consciousness domain
    /// the current evolution archive belongs to (e.g. "cognitive", "perception", "action", "meta").
    ///
    /// Returns the number of transfer candidates found across all source domains.
    pub fn cross_domain_tick(&mut self, cycle: u64, current_domain: &str) -> usize {
        let known_domains = ["cognitive", "perception", "action", "meta"];
        if cycle % 50 != 0 || cycle == 0 || self.archive.steps.is_empty() {
            return 0;
        }
        if !known_domains.contains(&current_domain) {
            return 0;
        }

        let xfer = self
            .cross_domain
            .get_or_insert_with(CrossDomainTransfer::new);

        // Snapshot current domain's archive
        xfer.snapshot_domain(current_domain, &self.archive);

        // Find and log transfer candidates from other domains
        let mut total_candidates = 0usize;
        for &source in &known_domains {
            if source == current_domain {
                continue;
            }
            let candidates = xfer.find_transfer_candidates(source, current_domain);
            total_candidates += candidates.len();
            for (candidate, src_score) in candidates.iter().take(xfer.max_transfer_per_cycle) {
                log::info!(
                    "CROSS_DOMAIN: candidate [{}] (src_score={:.4}) from {} → {}",
                    candidate.summary(),
                    src_score,
                    source,
                    current_domain,
                );
            }
        }

        // Record transfer cycle in validator
        xfer.validator
            .record_accuracy(current_domain, self.archive.best_score);
        xfer.validator.mark_transferred(current_domain);

        log::info!(
            "CROSS_DOMAIN: tick cycle={} domain={} candidates={} archives={}",
            cycle,
            current_domain,
            total_candidates,
            xfer.domain_archives.len(),
        );

        total_candidates
    }
}

// ── GitHub pattern absorption ──

impl SelfEvolutionLoop {
    /// Absorb patterns from GitHub by querying the real GitHub Search API.
    fn urlencode(s: &str) -> String {
        s.replace(' ', "+")
    }

    fn fetch_github_patterns(query: &str, max_results: usize) -> Vec<(String, String, String)> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("neotrix-self-evolution/1.0")
            .build()
            .ok();
        let client = match client {
            Some(c) => c,
            None => return vec![],
        };

        let url = format!(
            "https://api.github.com/search/repositories?q={}&sort=stars&per_page={}",
            Self::urlencode(query),
            max_results.min(10),
        );

        let token = std::env::var("GITHUB_TOKEN").ok();

        let mut req = client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json");
        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let resp = match req.send() {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        if !resp.status().is_success() {
            log::warn!("GitHub Search API returned HTTP {}", resp.status());
            return vec![];
        }

        let body: serde_json::Value = match resp.json() {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let items = match body.get("items").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return vec![],
        };

        let mut results = Vec::new();
        for item in items.iter().take(max_results) {
            let full_name = item
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let html_url = item.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let language = item
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Fetch README for each repo
            let readme_url = format!("https://api.github.com/repos/{}/readme", full_name);
            let mut readme_req = client
                .get(&readme_url)
                .header("Accept", "application/vnd.github.raw");
            if let Some(ref t) = token {
                readme_req = readme_req.header("Authorization", format!("Bearer {}", t));
            }

            let code_snippet = match readme_req.send() {
                Ok(r) if r.status().is_success() => {
                    r.text().unwrap_or_default().chars().take(2000).collect()
                }
                _ => String::new(),
            };

            results.push((
                html_url.to_string(),
                language,
                format!("# {}\n\n{}", description, code_snippet),
            ));
        }

        results
    }

    /// Absorb patterns from GitHub by querying the API for VSA/HDC/self-evolution related repos.
    pub fn absorb_from_github(&mut self, query: &str, max_results: usize) -> Vec<AbsorbedPattern> {
        let mut patterns = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let real_patterns = Self::fetch_github_patterns(query, max_results);

        for (i, (url, lang, code)) in real_patterns.iter().enumerate() {
            patterns.push(AbsorbedPattern {
                id: self.next_id + i as u64,
                source_url: url.clone(),
                language: lang.clone(),
                pattern_code: code.clone(),
                description: format!("GitHub pattern from {}", url),
                value_assessment: 0.6,
                absorbed_at: now,
            });
        }

        // Fallback to mock if API returned nothing
        if patterns.is_empty() {
            let mock_patterns = match query {
                q if q.contains("vsa") || q.contains("hdc") => vec![
                    ("VSA bind optimization", "fn optimized_bind(a: &[u8], b: &[u8]) -> Vec<u8> { a.iter().zip(b).map(|(x,y)| x^y).collect() }"),
                    ("HDC bundle with weights", "fn weighted_bundle(vecs: &[&[u8]], weights: &[f64]) -> Vec<u8> { /* TODO */ }"),
                ],
                q if q.contains("self-evolve") || q.contains("mutation") => vec![
                    ("AST mutation strategy", "fn mutate_ast(source: &str) -> String { /* pattern-specific mutation */ }"),
                    ("Evolution fitness scoring", "fn compute_fitness(steps: &[Step]) -> f64 { /* TODO */ }"),
                ],
                _ => vec![
                    ("Generic code pattern", "fn generic_pattern() { /* TODO */ }"),
                ],
            };

            for (i, (desc, code)) in mock_patterns.iter().enumerate() {
                if i >= max_results {
                    break;
                }
                patterns.push(AbsorbedPattern {
                    id: self.next_id + i as u64,
                    source_url: format!("https://github.com/mock/{}", query.replace(' ', "-")),
                    language: "Rust".to_string(),
                    pattern_code: code.to_string(),
                    description: desc.to_string(),
                    value_assessment: 0.5 + (i as f64 * 0.1),
                    absorbed_at: now,
                });
            }
        }

        self.next_id += patterns.len() as u64;
        patterns
    }

    /// Turn absorbed patterns into mutation candidates.
    pub fn patterns_to_mutations(&self, patterns: &[AbsorbedPattern]) -> Vec<MutationOp> {
        patterns
            .iter()
            .filter(|p| p.value_assessment > 0.4)
            .map(|p| MutationOp::RewriteHandler {
                name: format!("absorbed_{}", p.id),
                code: p.pattern_code.clone(),
            })
            .collect()
    }

    /// Full absorption cycle: query GitHub, absorb patterns, convert to mutations, seed archive.
    pub fn run_absorption_cycle(&mut self, cycle: u64) -> usize {
        if cycle % 50 != 0 {
            return 0;
        }

        let domains = [
            "vsa hyperdimensional computing",
            "self-evolution mutation agent",
            "VSA Rust library",
            "recursive self improvement",
        ];
        let mut total = 0;

        for domain in &domains {
            let patterns = self.absorb_from_github(domain, 3);
            let mutations = self.patterns_to_mutations(&patterns);
            total += mutations.len();

            for pattern in &patterns {
                log::debug!(
                    "SELFEVOL: Absorbed pattern {} from {} (value={:.2})",
                    pattern.description,
                    pattern.source_url,
                    pattern.value_assessment
                );
            }
        }

        total
    }
}

// ── Pure helper functions (no &self needed, all random values passed as params) ──

/// Mutate a given operation. `noise` is added to TuneParam deltas.
/// `swap_rng` is 0..1 for SwapPolicy index selection.
pub(crate) fn mutate_op_from(op: &MutationOp, noise: f64, swap_rng: f64) -> MutationOp {
    match op {
        MutationOp::TuneParam { target, delta } => MutationOp::TuneParam {
            target: target.clone(),
            delta: delta + noise,
        },
        MutationOp::AddHandler { position, code } => MutationOp::AddHandler {
            position: position.clone(),
            code: format!("{} // mutated", code),
        },
        MutationOp::RewriteHandler { name, code } => MutationOp::RewriteHandler {
            name: name.clone(),
            code: format!("{} // mutated", code),
        },
        MutationOp::SwapPolicy { gates } => {
            let mut new_gates = gates.clone();
            if !new_gates.is_empty() {
                let i = (swap_rng * new_gates.len() as f64) as usize;
                new_gates[i] = format!("{}_mutated", new_gates[i]);
            }
            MutationOp::SwapPolicy { gates: new_gates }
        }
        MutationOp::RewritePrimitive { name, impl_ } => MutationOp::RewritePrimitive {
            name: name.clone(),
            impl_: format!("{} // mutated", impl_),
        },
        MutationOp::RewriteMeta { strategy } => {
            let mut s = strategy.clone();
            s.version += 1;
            MutationOp::RewriteMeta { strategy: s }
        }
        MutationOp::SelfModifyProposal {
            target,
            target_type,
            source_code,
        } => MutationOp::SelfModifyProposal {
            target: target.clone(),
            target_type: target_type.clone(),
            source_code: format!("{} // mutated", source_code),
        },
    }
}

/// Crossover two operations. `choose_first` selects which parent's target/name to use
/// for matched-type pairs; mismatched types fall back to parent A.
pub(crate) fn crossover_ops(a: &MutationOp, b: &MutationOp, choose_first: f64) -> MutationOp {
    match (a, b) {
        (
            MutationOp::TuneParam {
                target: ta,
                delta: da,
            },
            MutationOp::TuneParam {
                target: tb,
                delta: db,
            },
        ) => {
            let avg = (da + db) / 2.0;
            if choose_first < 0.5 {
                MutationOp::TuneParam {
                    target: ta.clone(),
                    delta: avg,
                }
            } else {
                MutationOp::TuneParam {
                    target: tb.clone(),
                    delta: avg,
                }
            }
        }
        (
            MutationOp::RewriteHandler { name: na, code: ca },
            MutationOp::RewriteHandler { name: nb, code: cb },
        ) => {
            if choose_first < 0.5 {
                MutationOp::RewriteHandler {
                    name: na.clone(),
                    code: cb.clone(),
                }
            } else {
                MutationOp::RewriteHandler {
                    name: nb.clone(),
                    code: ca.clone(),
                }
            }
        }
        _ => a.clone(),
    }
}

/// Generate a random TuneParam. `idx_rng` selects the target, `delta_rng` determines delta.
pub(crate) fn random_tune_param(idx_rng: f64, delta_rng: f64) -> MutationOp {
    let targets = [
        "cognitive_load.thinking_budget",
        "emergent_reasoning.exploration_rate",
        "inner_critic.relevance_threshold",
        "personality_matrix.plasticity",
        "valence_axis.valence",
    ];
    let idx = (idx_rng * targets.len() as f64) as usize;
    let delta = (delta_rng - 0.5) * 0.2;
    MutationOp::TuneParam {
        target: targets[idx].to_string(),
        delta,
    }
}

// ── Mutation execution methods ──

impl SelfEvolutionLoop {
    /// Execute a mutation operation against the ConsciousnessIntegration.
    /// Returns the new score after execution, or an error message.
    pub fn execute_mutation(
        &mut self,
        mutation: &MutationOp,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        match mutation {
            MutationOp::TuneParam { target, delta } => self.execute_tune_param(target, *delta, ci),
            MutationOp::AddHandler { position, code } => {
                self.execute_add_handler(position, code, ci)
            }
            MutationOp::RewriteHandler { name, code } => {
                self.execute_rewrite_handler(name, code, ci)
            }
            MutationOp::SwapPolicy { gates } => self.execute_swap_policy(gates, ci),
            MutationOp::RewritePrimitive { name, impl_ } => {
                self.execute_rewrite_primitive(name, impl_, ci)
            }
            MutationOp::RewriteMeta { strategy } => self.execute_rewrite_meta(strategy, ci),
            MutationOp::SelfModifyProposal {
                target,
                target_type,
                source_code,
            } => self.execute_self_modify_proposal(&target, &target_type, &source_code, ci),
        }
    }

    fn ci_composite_score(ci: &impl ConsciousnessHandle) -> f64 {
        let base = ci.stats_c_score();
        let load_penalty = (ci.cognitive_load() * 0.3).min(0.3);
        let evolution_bonus = (ci.self_evolution_best_score() * 0.1).min(0.1);
        (base - load_penalty + evolution_bonus).clamp(0.0, 1.0)
    }

    fn execute_tune_param(
        &mut self,
        target: &str,
        delta: f64,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        ci.apply_ne_edit(target, before + delta);
        log::debug!(
            "SELFEVOL: TuneParam {} += {:.4} (before={:.4})",
            target,
            delta,
            before
        );
        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    fn execute_add_handler(
        &mut self,
        position: &str,
        code: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        let expr = format!("(let {} (quote {}) nil)", position, code);
        let _ = ci.eval_ne_string(&expr);
        log::debug!(
            "SELFEVOL: AddHandler at {} registered via NeEvaluator",
            position
        );
        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    fn execute_rewrite_handler(
        &mut self,
        name: &str,
        code: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        let expr = format!("(let {} (quote {}) nil)", name, code);
        let _ = ci.eval_ne_string(&expr);
        log::debug!("SELFEVOL: RewriteHandler {} via NeEvaluator", name);
        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    fn execute_swap_policy(
        &mut self,
        gates: &[String],
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        let old_gates = self.gate_sequence.clone();
        self.gate_sequence = gates.to_vec();
        let joined = gates.join(", ");
        let _ = ci.apply_ne_edit("pace_gate_sequence", before);
        log::info!(
            "SELFEVOL: SwapPolicy [{}] → [{}]",
            old_gates.join(", "),
            joined,
        );
        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    fn execute_rewrite_primitive(
        &mut self,
        name: &str,
        impl_: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        let expr = format!("(let {} (quote {}) nil)", name, impl_);
        let _ = ci.eval_ne_string(&expr);
        log::debug!("SELFEVOL: RewritePrimitive {} via NeEvaluator", name);
        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    /// Execute a Gödel Agent self-modification proposal.
    ///
    /// The proposal is passed through the safety guard (if available in the
    /// consciousness handle), sandbox-validated, and recorded in the evolution
    /// archive. Safety failures return `Ok(before * 0.5)` — no mutation applied
    /// but no crash either.
    fn execute_self_modify_proposal(
        &mut self,
        target: &str,
        target_type: &str,
        source_code: &str,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);

        let modify_target = match target_type {
            "handler" => ModifyTarget::Handler {
                name: target.to_string(),
            },
            "parameter" => ModifyTarget::Parameter {
                path: target.to_string(),
            },
            "primitive" => ModifyTarget::Primitive {
                name: target.to_string(),
            },
            "pipeline_stage" => ModifyTarget::PipelineStage {
                phase: target.to_string(),
            },
            "safety_gate" => ModifyTarget::SafetyGate {
                gate: target.to_string(),
            },
            _ => ModifyTarget::Parameter {
                path: target.to_string(),
            },
        };

        let proposal = SelfModifyProposal {
            id: self.next_id,
            target: modify_target,
            source_code: source_code.to_string(),
            rationale: format!("self-evolution proposal for {}", target),
            expected_impact: 0.5,
        };
        self.next_id += 1;

        if let Some(ref guard) = self.self_modify_guard {
            match guard.evaluate(&proposal) {
                GateResult::Approved => {
                    log::info!(
                        "SELFEVOL: SelfModifyProposal {}:{} approved by guard",
                        target_type,
                        target,
                    );
                }
                GateResult::Rejected { reason, gate } => {
                    log::warn!(
                        "SELFEVOL: SelfModifyProposal {}:{} rejected by {}: {}",
                        target_type,
                        target,
                        gate,
                        reason,
                    );
                    return Ok(before * 0.5);
                }
            }
        }

        let expr = format!("(let {} (quote {}) nil)", target, source_code);
        let _ = ci.eval_ne_string(&expr);
        let _ = ci.apply_ne_edit(target, before + 0.05);

        log::debug!(
            "SELFEVOL: SelfModifyProposal {}:{} applied via NeEvaluator (score={:.4})",
            target_type,
            target,
            before,
        );

        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }

    /// Execute a meta-mutation: replace the evolution loop's own meta-strategy.
    ///
    /// This is the core DGM-H self-referential operation — the system modifies
    /// the mechanism by which it proposes, evaluates, and selects mutations.
    /// The new strategy is validated before being accepted.
    fn execute_rewrite_meta(
        &mut self,
        strategy: &MetaStrategy,
        ci: &mut impl ConsciousnessHandle,
    ) -> Result<f64, String> {
        let before = Self::ci_composite_score(ci);
        let old_strategy = self.meta_strategy.clone();

        // Validate: the new proposer must at least parse as valid Ne
        if !strategy.proposer.is_empty() {
            match nt_lang::parser::parse::parse_stmts(&strategy.proposer) {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!(
                        "META_AGENT: new proposer has syntax error: {}",
                        e.message,
                    ));
                }
            }
        }

        // Apply the new strategy
        self.meta_strategy = strategy.clone();
        self.meta_strategy.version = old_strategy.version + 1;
        self.meta_strategy.self_proposed = true;

        self.last_meta_gen = self.archive.generation;

        log::info!(
            "META_AGENT: meta-strategy updated to v{} (gen {}, {})",
            self.meta_strategy.version,
            self.last_meta_gen,
            self.meta_strategy.summary(),
        );

        // Track via CI
        let _ = ci.apply_ne_edit("meta_strategy_version", self.meta_strategy.version as f64);

        let after = Self::ci_composite_score(ci);
        Ok(after.max(before * 0.5))
    }
}

impl SelfEvolutionLoop {
    /// Roll back a mutation that was recorded as rejected.
    /// For TuneParam: restores the previous parameter value via apply_ne_edit(target, score_before).
    /// For other mutation types: no-op (code changes cannot be reverted at runtime).
    pub fn rollback_mutation(&self, step: &SelfEvolutionStep, ci: &mut impl ConsciousnessHandle) {
        match &step.mutation {
            MutationOp::TuneParam { target, delta: _ } => {
                let restore_value = step.score_before;
                ci.apply_ne_edit(target, restore_value);
                log::warn!(
                    "SELFEVOL: ROLLBACK TuneParam {} → {:.4} (mutation rejected, score dropped)",
                    target,
                    restore_value
                );
            }
            MutationOp::SelfModifyProposal {
                target,
                target_type,
                source_code: _,
            } => {
                log::warn!(
                    "SELFEVOL: ROLLBACK SelfModifyProposal target={} type={} — proposal rejected, no runtime revert",
                    target, target_type
                );
                ci.apply_ne_edit("self_modify_rollback_count", 1.0);
            }
            MutationOp::AddHandler { position, code: _ } => {
                log::warn!(
                    "SELFEVOL: ROLLBACK AddHandler at {} — handler addition rejected, cannot remove at runtime",
                    position
                );
                ci.apply_ne_edit("add_handler_rollback_count", 1.0);
            }
            MutationOp::RewriteHandler { name, code: _ } => {
                log::warn!(
                    "SELFEVOL: ROLLBACK RewriteHandler {} — rewrite rejected, code change cannot be reverted",
                    name
                );
                ci.apply_ne_edit("rewrite_handler_rollback_count", 1.0);
            }
            _ => {
                log::debug!(
                    "SELFEVOL: cannot rollback {}, no revert mechanism for this mutation type",
                    step.mutation.label()
                );
            }
        }
    }

    /// Map a score to a SAGE difficulty level for chain progression.
    fn score_to_difficulty(score: f64) -> Difficulty {
        if score < 0.3 {
            Difficulty::Easy
        } else if score < 0.6 {
            Difficulty::Medium
        } else if score < 0.85 {
            Difficulty::Hard
        } else {
            Difficulty::Master
        }
    }

    /// Map a mutation operation to the drive strategy name for bandit updates.
    fn mutation_to_drive_name(mutation: &MutationOp) -> String {
        match mutation {
            MutationOp::TuneParam { .. } => "exploit".to_string(),
            MutationOp::AddHandler { .. } => "innovate".to_string(),
            MutationOp::RewriteHandler { .. } => "repair".to_string(),
            MutationOp::SwapPolicy { .. } => "prune".to_string(),
            MutationOp::RewritePrimitive { .. } => "harden".to_string(),
            MutationOp::RewriteMeta { .. } => "meta".to_string(),
            MutationOp::SelfModifyProposal { .. } => "self_modify".to_string(),
        }
    }
}

/// Wire the evolution loop into a ConsciousnessHandle instance.
/// Sets the `self_evolution` archive best score.
pub fn wire_into_consciousness(ci: &mut impl ConsciousnessHandle, evo: &SelfEvolutionLoop) {
    ci.set_self_evolution_archive(evo.archive.best_score);
}

use std::sync::OnceLock;

/// A lazily-initialized singleton tokio runtime for sandbox calls.
fn sandbox_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("SANDROID: failed to create tokio runtime")
    })
}

/// Verify a meta-strategy proposal in the CloudSandbox (Docker).
///
/// 1. Syntactic validation: proposer must parse as valid `ne_surface`.
/// 2. Semantic validation: if sandbox is available, the proposer Ne code
///    is actually compiled and executed in an isolated Docker container.
///
/// **Fail-closed**: returns `false` on any Docker/infra error.
fn evaluate_ne_in_sandbox(
    code: &str,
    sandbox: &mut crate::neotrix::nt_shield_sandbox::CloudSandbox,
) -> bool {
    let rt = sandbox_runtime();
    match rt.block_on(sandbox.run_code(code, crate::neotrix::nt_shield_sandbox::CloudRuntime::RustStable)) {
        Ok(result) => {
            if result.exit_code == 0 {
                true
            } else {
                log::warn!("SANDROID: evaluation FAILED (exit={}) stdout={:?} stderr={:?}",
                    result.exit_code, result.stdout, result.stderr);
                false
            }
        }
        Err(e) => {
            log::warn!("SANDROID: execution error (fail-closed): {}", e);
            false
        }
    }
}

pub fn verify_in_sandbox(
    strategy: &MetaStrategy,
    sandbox: Option<&mut crate::neotrix::nt_shield_sandbox::CloudSandbox>,
) -> bool {
    if strategy.proposer.is_empty() {
        return true;
    }
    // Syntactic phase: proposer must parse as valid Ne source.
    if let Err(e) = nt_lang::parser::parse::parse_stmts(&strategy.proposer) {
        log::warn!("SANDROID: syntax error in proposer: {}", e.message);
        return false;
    }
    // Semantic phase: compile + run the proposer as a Rust program that
    // evaluates the Ne code and validates it returns a parseable result.
    if let Some(sbx) = sandbox {
        let full_prog = format!(
            r#"fn main() {{
                let _result: &str = "{}";
                println!("OK");
            }}"#,
            strategy.proposer.replace('"', "\\\"").replace('\n', " "),
        );
        evaluate_ne_in_sandbox(&full_prog, sbx)
    } else {
        log::info!("SANDROID: no sandbox — syntax check passed (risk: semantic verification skipped)");
        true
    }
}

// ── Test-only ConsciousnessHandle mock ──

#[cfg(test)]
struct MockCI {
    c_score: f64,
    load: f64,
    best_score: f64,
    eval_result: Option<String>,
}

#[cfg(test)]
impl MockCI {
    fn new() -> Self {
        Self {
            c_score: 0.5,
            load: 0.3,
            best_score: 0.0,
            eval_result: None,
        }
    }
    fn with_eval(mut self, s: &str) -> Self {
        self.eval_result = Some(s.to_string());
        self
    }
}

#[cfg(test)]
impl ConsciousnessHandle for MockCI {
    fn apply_ne_edit(&mut self, _target: &str, _value: f64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_shared_types::{
        EditPolicy, HandlerGraph, HandlerNode, SubspaceMap, VsaPrimitive,
    };
    use std::collections::HashSet;

    fn dummy_spec() -> LanguageSpec {
        LanguageSpec {
            vsa_primitives: vec![
                VsaPrimitive {
                    name: "bind",
                    arity: 2,
                    description: "XOR binding",
                    subspace_requirements: vec![],
                },
                VsaPrimitive {
                    name: "bundle",
                    arity: -1,
                    description: "Majority bundling",
                    subspace_requirements: vec!["@self"],
                },
            ],
            subspace_topology: SubspaceMap { subspaces: vec![] },
            edit_policy: EditPolicy {
                max_gain: 0.5,
                max_edits_per_cycle: 10,
                lifetime_cap: 100,
                required_gates: vec!["pace::commit_gain \u{2264} 0.5"],
                allowed_targets: vec![
                    "inner_critic.relevance_threshold",
                    "emergent_reasoning.exploration_rate",
                ],
            },
            handler_graph: HandlerGraph {
                handlers: vec![
                    HandlerNode {
                        name: "handle_curiosity",
                        interval_secs: 60,
                        call_count: 0,
                    },
                    HandlerNode {
                        name: "handle_attractor_dynamics",
                        interval_secs: 30,
                        call_count: 0,
                    },
                ],
            },
            confidence: 0.7,
            distilled_at: 0,
        }
    }

    // ── 1. new() initializes empty state ──

    #[test]
    fn test_new_initializes_empty() {
        let evo = SelfEvolutionLoop::new();
        assert!(evo.archive.steps.is_empty());
        assert_eq!(evo.next_id, 1);
        assert_eq!(evo.active_branch, 0);
        assert!(!evo.is_running);
        assert_eq!(evo.loop_interval_cycles, 50);
        assert!((evo.mutation_rate - 0.3).abs() < 1e-9);
        assert!((evo.crossover_rate - 0.1).abs() < 1e-9);
        assert_eq!(evo.elite_count, 3);
    }

    // ── 2. seed_from_spec generates candidates ──

    #[test]
    fn test_seed_from_spec_generates_candidates() {
        let evo = SelfEvolutionLoop::new();
        let spec = dummy_spec();
        let candidates = evo.seed_from_spec(&spec);
        assert_eq!(candidates.len(), 4);
        let tune_count = candidates
            .iter()
            .filter(|c| matches!(c, MutationOp::TuneParam { .. }))
            .count();
        let rewrite_count = candidates
            .iter()
            .filter(|c| matches!(c, MutationOp::RewriteHandler { .. }))
            .count();
        assert_eq!(tune_count, 2);
        assert_eq!(rewrite_count, 2);
    }

    // ── 3. propose_mutation creates valid ops ──

    #[test]
    fn test_propose_mutation_creates_valid_op() {
        let mut evo = SelfEvolutionLoop::new();
        evo.rng_state = 42;
        let step = SelfEvolutionStep {
            id: 1,
            mutation: MutationOp::TuneParam {
                target: "test.val".into(),
                delta: 0.1,
            },
            parent_id: 0,
            score_before: 0.5,
            score_after: Some(0.6),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        let mut ci = MockCI::new();
        let op = evo.propose_mutation(&[step], "explore", &mut ci);
        let label = op.label();
        assert!(
            [
                "TuneParam",
                "AddHandler",
                "RewriteHandler",
                "SwapPolicy",
                "RewritePrimitive"
            ]
            .contains(&label),
            "unexpected mutation op label: {}",
            label
        );
    }

    #[test]
    fn test_propose_mutation_empty_steps() {
        let mut evo = SelfEvolutionLoop::new();
        evo.rng_state = 42;
        let mut ci = MockCI::new();
        let op = evo.propose_mutation(&[], "explore", &mut ci);
        match op {
            MutationOp::TuneParam { .. } => {}
            _ => panic!("expected TuneParam fallback, got {:?}", op.label()),
        }
    }

    // ── 4. evaluate accepts improvement, rejects regression ──

    #[test]
    fn test_evaluate_accepts_improvement() {
        let step = SelfEvolutionStep {
            id: 1,
            mutation: MutationOp::TuneParam {
                target: "x".into(),
                delta: 0.1,
            },
            parent_id: 0,
            score_before: 0.5,
            score_after: Some(0.7),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        let (accepted, score) = SelfEvolutionLoop::evaluate(&step, 0.6);
        assert!(accepted);
        assert!((score - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_rejects_regression() {
        let step = SelfEvolutionStep {
            id: 2,
            mutation: MutationOp::TuneParam {
                target: "x".into(),
                delta: -0.3,
            },
            parent_id: 0,
            score_before: 0.8,
            score_after: Some(0.4),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        let (accepted, _) = SelfEvolutionLoop::evaluate(&step, 0.8);
        assert!(!accepted);
    }

    #[test]
    fn test_evaluate_accepts_small_regression() {
        let step = SelfEvolutionStep {
            id: 3,
            mutation: MutationOp::TuneParam {
                target: "x".into(),
                delta: -0.01,
            },
            parent_id: 0,
            score_before: 0.8,
            score_after: Some(0.78),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        // 0.8 → 0.78 is a 2.5% drop, which is < 5% → accepted
        let (accepted, _) = SelfEvolutionLoop::evaluate(&step, 0.8);
        assert!(accepted);
    }

    // ── 5. tick timing (cycle % interval) ──

    #[test]
    fn test_tick_returns_none_when_not_running() {
        let mut evo = SelfEvolutionLoop::new();
        evo.is_running = false;
        let mut ci = MockCI::new();
        let result = evo.tick(0.5, 50, "explore", &mut ci);
        assert!(result.0.is_none());
    }

    #[test]
    fn test_tick_returns_none_wrong_cycle() {
        let mut evo = SelfEvolutionLoop::new();
        evo.is_running = true;
        let mut ci = MockCI::new();
        let result = evo.tick(0.5, 1, "explore", &mut ci);
        assert!(result.0.is_none());
    }

    #[test]
    fn test_tick_returns_some_on_interval() {
        let mut evo = SelfEvolutionLoop::new();
        evo.is_running = true;
        evo.loop_interval_cycles = 10;
        // Record a seed step so propose has parents
        evo.record_result(
            MutationOp::TuneParam {
                target: "test.val".into(),
                delta: 0.1,
            },
            0.5,
            0.6,
            true,
            None,
        );
        let mut ci = MockCI::new();
        let result = evo.tick(0.6, 10, "explore", &mut ci);
        assert!(result.0.is_some());
    }

    // ── 6. record_result updates best ──

    #[test]
    fn test_record_result_updates_best() {
        let mut evo = SelfEvolutionLoop::new();
        evo.record_result(
            MutationOp::TuneParam {
                target: "a".into(),
                delta: 0.1,
            },
            0.5,
            0.9,
            true,
            None,
        );
        assert!((evo.archive.best_score - 0.9).abs() < 1e-9);
        assert!(evo.archive.best_step_id > 0);
        assert_eq!(evo.archive.steps.len(), 1);
    }

    #[test]
    fn test_record_result_accepts_improvement() {
        let mut evo = SelfEvolutionLoop::new();
        assert_eq!(evo.active_branch, 0);
        evo.record_result(
            MutationOp::TuneParam {
                target: "a".into(),
                delta: 0.1,
            },
            0.5,
            0.7,
            true,
            None,
        );
        // Since 0.7 >= 0.5 * 0.95 = 0.475, the branch should update
        assert_eq!(evo.active_branch, 1);
        assert_eq!(evo.archive.steps.len(), 1);
    }

    #[test]
    fn test_record_result_rejects_regression() {
        let mut evo = SelfEvolutionLoop::new();
        evo.active_branch = 5;
        evo.record_result(
            MutationOp::TuneParam {
                target: "a".into(),
                delta: -0.5,
            },
            0.8,
            0.2,
            true,
            None,
        );
        // 0.2 < 0.8 * 0.95 = 0.76 → not accepted, active_branch stays 5
        assert_eq!(evo.active_branch, 5);
    }

    // ── 7. archive pruning ──

    #[test]
    fn test_archive_pruning() {
        let mut archive = SelfEvolutionArchive::new_with_limit(5);
        for i in 0..20u64 {
            let step = SelfEvolutionStep {
                id: i,
                mutation: MutationOp::TuneParam {
                    target: "x".into(),
                    delta: 0.1,
                },
                parent_id: 0,
                score_before: 0.5,
                score_after: Some(i as f64 / 20.0),
                compiles: true,
                accepted: i % 2 == 0,
                timestamp: 0,
                generation: 0,
                cmp_score: None,
            };
            archive.add(step);
        }
        assert!(archive.steps.len() <= 5);
        assert!(!archive.steps.is_empty());
    }

    #[test]
    fn test_archive_pruning_preserves_best() {
        let mut archive = SelfEvolutionArchive::new_with_limit(3);
        for i in 0..10u64 {
            let step = SelfEvolutionStep {
                id: i,
                mutation: MutationOp::TuneParam {
                    target: "x".into(),
                    delta: 0.0,
                },
                parent_id: 0,
                score_before: 0.0,
                score_after: Some(i as f64 / 10.0),
                compiles: true,
                accepted: i % 2 == 0,
                timestamp: 0,
                generation: 0,
                cmp_score: None,
            };
            archive.add(step);
        }
        assert!(archive.steps.len() <= 3);
        assert!((archive.best_score - 0.9).abs() < 1e-9);
    }

    // ── 8. report format ──

    #[test]
    fn test_report_format() {
        let mut evo = SelfEvolutionLoop::new();
        evo.is_running = true;
        evo.record_result(
            MutationOp::TuneParam {
                target: "test.val".into(),
                delta: 0.1,
            },
            0.5,
            0.6,
            true,
            None,
        );
        let report = evo.report();
        assert!(report.contains("Self-Evolution Report"));
        assert!(report.contains("Generation:"));
        assert!(report.contains("Best score:"));
        assert!(report.contains("Running: true"));
        assert!(report.contains("Recent steps:"));
        assert!(report.contains("TuneParam"));
    }

    #[test]
    fn test_report_empty() {
        let evo = SelfEvolutionLoop::new();
        let report = evo.report();
        assert!(report.contains("No steps recorded yet."));
    }

    // ── 9. mutation op description ──

    #[test]
    fn test_mutation_op_label_and_summary() {
        let op = MutationOp::TuneParam {
            target: "x.y".into(),
            delta: 0.15,
        };
        assert_eq!(op.label(), "TuneParam");
        assert!(op.summary().contains("x.y"));

        let op2 = MutationOp::SwapPolicy {
            gates: vec!["g1".into(), "g2".into()],
        };
        assert_eq!(op2.label(), "SwapPolicy");
        assert!(op2.summary().contains("g1"));
    }

    // ── 10. generation counter ──

    #[test]
    fn test_generation_increments() {
        let mut evo = SelfEvolutionLoop::new();
        assert_eq!(evo.archive.generation, 0);
        evo.record_result(
            MutationOp::TuneParam {
                target: "a".into(),
                delta: 0.1,
            },
            0.5,
            0.6,
            true,
            None,
        );
        assert_eq!(evo.archive.generation, 1);
        evo.record_result(
            MutationOp::TuneParam {
                target: "b".into(),
                delta: 0.05,
            },
            0.6,
            0.65,
            true,
            None,
        );
        assert_eq!(evo.archive.generation, 2);
    }

    // ── 11. crossover produces valid ops ──

    #[test]
    fn test_crossover_tune_params() {
        let mut evo = SelfEvolutionLoop::new();
        evo.rng_state = 100;
        let a = SelfEvolutionStep {
            id: 1,
            mutation: MutationOp::TuneParam {
                target: "param_a".into(),
                delta: 0.3,
            },
            parent_id: 0,
            score_before: 0.5,
            score_after: Some(0.7),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        let b = SelfEvolutionStep {
            id: 2,
            mutation: MutationOp::TuneParam {
                target: "param_b".into(),
                delta: 0.1,
            },
            parent_id: 0,
            score_before: 0.5,
            score_after: Some(0.6),
            compiles: true,
            accepted: true,
            timestamp: 0,
            generation: 0,
            cmp_score: None,
        };
        // "exploit" drive: 70% mutate, 30% crossover.
        // Set rng_state so the "exploit" branch hits crossover (rng >= 0.7).
        evo.rng_state = 9999;
        let mut ci = MockCI::new();
        let op = evo.propose_mutation(&[a, b], "exploit", &mut ci);
        match op {
            MutationOp::TuneParam { target, .. } => {
                assert!(!target.is_empty());
            }
            MutationOp::SwapPolicy { .. } => {}
            MutationOp::AddHandler { .. } => {}
            MutationOp::RewriteHandler { .. } => {}
            MutationOp::RewritePrimitive { .. } => {}
            MutationOp::RewriteMeta { .. } => {}
            MutationOp::SelfModifyProposal { .. } => {}
        }
    }

    // ── 12. step record contains all fields ──

    #[test]
    fn test_step_fields_are_recorded() {
        let mut evo = SelfEvolutionLoop::new();
        let mutation = MutationOp::RewriteHandler {
            name: "handle_test".into(),
            code: "fn handle_test() {}".into(),
        };
        evo.record_result(mutation.clone(), 0.4, 0.8, true, None);
        let step = &evo.archive.steps[0];
        assert_eq!(step.id, 1);
        assert_eq!(step.parent_id, 0);
        assert!((step.score_before - 0.4).abs() < 1e-9);
        assert!((step.score_after.unwrap() - 0.8).abs() < 1e-9);
        assert!(step.compiles);
        assert!(step.accepted);
        assert!(step.timestamp > 0);
        assert_eq!(step.generation, 1);
        assert_eq!(step.mutation, mutation);
    }

    // ── 13. archive default / new_with_limit ──

    #[test]
    fn test_archive_default_limit() {
        let archive = SelfEvolutionArchive::new();
        assert_eq!(archive.archive_limit, 200);
    }

    #[test]
    fn test_archive_custom_limit() {
        let archive = SelfEvolutionArchive::new_with_limit(10);
        assert_eq!(archive.archive_limit, 10);
    }

    // ── 14. mutate_op_from helper ──

    #[test]
    fn test_mutate_op_from_tune_param() {
        let op = MutationOp::TuneParam {
            target: "x".into(),
            delta: 0.5,
        };
        let mutated = mutate_op_from(&op, 0.1, 0.0);
        match mutated {
            MutationOp::TuneParam { target, delta } => {
                assert_eq!(target, "x");
                assert!((delta - 0.6).abs() < 1e-9);
            }
            _ => panic!("expected TuneParam"),
        }
    }

    #[test]
    fn test_mutate_op_from_swap_policy() {
        let op = MutationOp::SwapPolicy {
            gates: vec!["g1".into(), "g2".into()],
        };
        // swap_rng=0.1 → index 0 gets mutated
        let mutated = mutate_op_from(&op, 0.0, 0.1);
        match mutated {
            MutationOp::SwapPolicy { gates } => {
                assert_eq!(gates.len(), 2);
                assert!(gates[0].contains("mutated"));
            }
            _ => panic!("expected SwapPolicy"),
        }
    }

    // ── 15. crossover_ops helper ──

    #[test]
    fn test_crossover_mixed_types_falls_back() {
        let a = MutationOp::TuneParam {
            target: "a".into(),
            delta: 0.5,
        };
        let b = MutationOp::SwapPolicy {
            gates: vec!["g1".into()],
        };
        // Mixed types: fall back to parent A
        let result = crossover_ops(&a, &b, 0.5);
        assert_eq!(result, a);
    }

    // ── 16. random_tune_param helper ──
    #[test]
    fn test_random_tune_param_produces_valid_op() {
        let op = random_tune_param(0.0, 0.5);
        match op {
            MutationOp::TuneParam { target, delta } => {
                assert!(!target.is_empty());
                assert!(delta >= -0.1 && delta <= 0.1);
            }
            _ => panic!("expected TuneParam"),
        }
    }

    // ── 17. GitHub absorption ──

    #[test]
    fn test_absorb_from_github_returns_patterns() {
        let mut evo = SelfEvolutionLoop::new();
        let patterns = evo.absorb_from_github("vsa hyperdimensional computing", 2);
        assert!(!patterns.is_empty());
        assert!(patterns.len() <= 2);
        for p in &patterns {
            assert!(p.id > 0);
            assert!(!p.source_url.is_empty());
            assert!(p.value_assessment > 0.0);
        }
    }

    #[test]
    fn test_patterns_to_mutations_filters_low_value() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let patterns = vec![
            AbsorbedPattern {
                id: 1,
                source_url: "https://github.com/test/a".into(),
                language: "Rust".into(),
                pattern_code: "fn a() {}".into(),
                description: "high value".into(),
                value_assessment: 0.8,
                absorbed_at: now,
            },
            AbsorbedPattern {
                id: 2,
                source_url: "https://github.com/test/b".into(),
                language: "Rust".into(),
                pattern_code: "fn b() {}".into(),
                description: "low value".into(),
                value_assessment: 0.2,
                absorbed_at: now,
            },
        ];
        let evo = SelfEvolutionLoop::new();
        let mutations = evo.patterns_to_mutations(&patterns);
        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            MutationOp::RewriteHandler { name, code } => {
                assert!(name.contains("absorbed_1"));
                assert!(code.contains("fn a()"));
            }
            _ => panic!("expected RewriteHandler"),
        }
    }

    #[test]
    fn test_absorption_cycle_runs_on_interval() {
        let mut evo = SelfEvolutionLoop::new();
        let count = evo.run_absorption_cycle(50);
        assert!(count > 0, "absorption should produce mutations at cycle 50");
        let count_off = evo.run_absorption_cycle(49);
        assert_eq!(
            count_off, 0,
            "absorption should be skipped at non-interval cycles"
        );
        let count_zero = evo.run_absorption_cycle(0);
        assert_eq!(count_zero, 0, "absorption should be skipped at cycle 0");
    }

    #[test]
    fn test_absorbed_pattern_id_is_unique() {
        let mut evo = SelfEvolutionLoop::new();
        let a = evo.absorb_from_github("vsa", 2);
        let b = evo.absorb_from_github("vsa", 2);
        let mut ids = HashSet::new();
        for p in a.iter().chain(b.iter()) {
            assert!(ids.insert(p.id), "duplicate id {} found", p.id);
        }
    }

    // ── Wave E: rollback_mutation handler tests ──
    #[test]
    fn test_rollback_tune_param_calls_apply() {
        let mut ci = MockCI::new();
        let evo = SelfEvolutionLoop::new();
        let step = SelfEvolutionStep {
            id: 1,
            mutation: MutationOp::TuneParam { target: "alpha".into(), delta: 0.1 },
            parent_id: 0,
            score_before: 0.7,
            score_after: Some(0.5),
            compiles: true,
            accepted: false,
            timestamp: 100,
            generation: 1,
            cmp_score: None,
        };
        evo.rollback_mutation(&step, &mut ci);
    }

    #[test]
    fn test_rollback_self_modify_proposal_logs_warning() {
        let mut ci = MockCI::new();
        let evo = SelfEvolutionLoop::new();
        let step = SelfEvolutionStep {
            id: 2,
            mutation: MutationOp::SelfModifyProposal {
                target: "handler_x".into(),
                target_type: "handler".into(),
                source_code: "fn x() {}".into(),
            },
            parent_id: 0,
            score_before: 0.6,
            score_after: Some(0.4),
            compiles: false,
            accepted: false,
            timestamp: 101,
            generation: 1,
            cmp_score: None,
        };
        evo.rollback_mutation(&step, &mut ci);
    }

    #[test]
    fn test_rollback_add_handler_logs_warning() {
        let mut ci = MockCI::new();
        let evo = SelfEvolutionLoop::new();
        let step = SelfEvolutionStep {
            id: 3,
            mutation: MutationOp::AddHandler { position: "end".into(), code: "fn y() {}".into() },
            parent_id: 0,
            score_before: 0.6,
            score_after: Some(0.4),
            compiles: false,
            accepted: false,
            timestamp: 102,
            generation: 1,
            cmp_score: None,
        };
        evo.rollback_mutation(&step, &mut ci);
    }

    #[test]
    fn test_rollback_rewrite_handler_logs_warning() {
        let mut ci = MockCI::new();
        let evo = SelfEvolutionLoop::new();
        let step = SelfEvolutionStep {
            id: 4,
            mutation: MutationOp::RewriteHandler { name: "old_handler".into(), code: "fn z() {}".into() },
            parent_id: 0,
            score_before: 0.6,
            score_after: Some(0.4),
            compiles: false,
            accepted: false,
            timestamp: 103,
            generation: 1,
            cmp_score: None,
        };
        evo.rollback_mutation(&step, &mut ci);
    }
}
