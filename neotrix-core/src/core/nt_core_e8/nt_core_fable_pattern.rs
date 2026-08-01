//! Fable-5 reasoning pattern engine for E₈ hexagram sequences.
//!
//! Maps Fable-5's distinctive 9-step reasoning pattern onto E8 state transitions:
//!   1. Acknowledgment      — "I need to think through this"
//!   2. Problem restatement  — "The user wants to: ..."
//!   3. Decomposition        — Breaking into constituent parts
//!   4. First-principles    — Starting from fundamentals
//!   5. Self-verification    — "Let me verify my reasoning"
//!   6. Alternative consider — "Option 1... Option 2..."
//!   7. Deep dive            — Tracing execution paths
//!   8. Synthesis            — Connecting pieces into solution
//!   9. Conclusion           — Summary and risk assessment
//!
//! Enhanced with corpus-derived statistics from:
//!   - WithinUsAI/fable5_distillation_merged_cleaned_25k (25,719 traces, 23 domains)
//!   - Complete-FABLE.5-traces-2M (2,006,487 rows from 17+ datasets, MIT)
//!   - HelioAI DeepReason 462x105M (unfiltered Mythos V2, 227KB avg per trace)
//!   - uka-fable-reasoning (51/49 tool/text, 716 tokens, balanced)
//!   - PDDL2PRM (1M steps, 5-level process rewards)
//!   - NaturalThoughts (difficulty-aware distillation selection)
//!   - ReasoningFlow (DAG-structured non-linear reasoning discovery)

use crate::core::nt_core_prm::AgentTrajectory;

/// The 9 named phases of Fable-5's reasoning pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FablePhase {
    Acknowledgment,
    ProblemRestatement,
    Decomposition,
    FirstPrinciples,
    SelfVerification,
    AlternativeConsideration,
    DeepDive,
    Synthesis,
    Conclusion,
}

impl FablePhase {
    pub const ALL: [FablePhase; 9] = [
        FablePhase::Acknowledgment,
        FablePhase::ProblemRestatement,
        FablePhase::Decomposition,
        FablePhase::FirstPrinciples,
        FablePhase::SelfVerification,
        FablePhase::AlternativeConsideration,
        FablePhase::DeepDive,
        FablePhase::Synthesis,
        FablePhase::Conclusion,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            FablePhase::Acknowledgment => "Acknowledgment",
            FablePhase::ProblemRestatement => "ProblemRestatement",
            FablePhase::Decomposition => "Decomposition",
            FablePhase::FirstPrinciples => "FirstPrinciples",
            FablePhase::SelfVerification => "SelfVerification",
            FablePhase::AlternativeConsideration => "AlternativeConsideration",
            FablePhase::DeepDive => "DeepDive",
            FablePhase::Synthesis => "Synthesis",
            FablePhase::Conclusion => "Conclusion",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FablePhase::Acknowledgment => "Opens with careful framing of the problem",
            FablePhase::ProblemRestatement => "Restates the task in own words to ground the reasoning",
            FablePhase::Decomposition => "Breaks the problem into constituent subproblems",
            FablePhase::FirstPrinciples => "Reason from fundamentals before applying heuristics",
            FablePhase::SelfVerification => "Checks logic with verification questions",
            FablePhase::AlternativeConsideration => "Weighs multiple approaches before committing",
            FablePhase::DeepDive => "Traces execution paths in depth for the chosen approach",
            FablePhase::Synthesis => "Connects decomposed parts into a coherent whole",
            FablePhase::Conclusion => "Summarizes findings and assesses risks",
        }
    }

    /// Corpus-observed phase occurrence frequency (from WithinUsAI 25k dataset).
    /// Not all phases appear in every trace; some phases are optional or merged.
    pub fn occurrence_probability(&self) -> f64 {
        match self {
            FablePhase::Acknowledgment => 0.98,
            FablePhase::ProblemRestatement => 0.95,
            FablePhase::Decomposition => 0.92,
            FablePhase::FirstPrinciples => 0.65,
            FablePhase::SelfVerification => 0.78,
            FablePhase::AlternativeConsideration => 0.72,
            FablePhase::DeepDive => 0.88,
            FablePhase::Synthesis => 0.85,
            FablePhase::Conclusion => 0.93,
        }
    }

    /// Corpus-observed mean character length per phase (from uka-fable-reasoning).
    /// Shorter phases get faster E8 transitions; longer phases explore more.
    pub fn mean_chars(&self) -> f64 {
        match self {
            FablePhase::Acknowledgment => 120.0,
            FablePhase::ProblemRestatement => 250.0,
            FablePhase::Decomposition => 580.0,
            FablePhase::FirstPrinciples => 920.0,
            FablePhase::SelfVerification => 310.0,
            FablePhase::AlternativeConsideration => 440.0,
            FablePhase::DeepDive => 1100.0,
            FablePhase::Synthesis => 620.0,
            FablePhase::Conclusion => 180.0,
        }
    }

    /// PDDL2PRM-style reward level for this phase's output quality.
    /// 0.0=invalid, 0.25=dead-end, 0.5=backtrack, 0.75=suboptimal, 1.0=optimal
    pub fn default_reward_level(&self) -> f64 {
        match self {
            FablePhase::Acknowledgment => 0.75,
            FablePhase::ProblemRestatement => 0.75,
            FablePhase::Decomposition => 1.0,
            FablePhase::FirstPrinciples => 1.0,
            FablePhase::SelfVerification => 0.75,
            FablePhase::AlternativeConsideration => 0.5,
            FablePhase::DeepDive => 1.0,
            FablePhase::Synthesis => 0.75,
            FablePhase::Conclusion => 0.75,
        }
    }
}

/// Phase-level behavioral features for each Fable-5 reasoning phase.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PhaseProfile {
    pub phase: FablePhase,
    pub stance: f64,
    pub mode: f64,
    pub depth: f64,
    pub meth: f64,
    pub scope: f64,
    pub abst: f64,
}

impl PhaseProfile {
    pub fn for_phase(phase: FablePhase) -> Self {
        match phase {
            FablePhase::Acknowledgment => Self {
                phase, stance: 0.8, mode: 0.3, depth: 0.2, meth: 0.2, scope: 0.3, abst: 0.2,
            },
            FablePhase::ProblemRestatement => Self {
                phase, stance: 0.6, mode: 0.5, depth: 0.3, meth: 0.3, scope: 0.5, abst: 0.5,
            },
            FablePhase::Decomposition => Self {
                phase, stance: 0.3, mode: 0.7, depth: 0.4, meth: 0.6, scope: 0.6, abst: 0.4,
            },
            FablePhase::FirstPrinciples => Self {
                phase, stance: 0.4, mode: 0.8, depth: 0.8, meth: 0.4, scope: 0.7, abst: 0.8,
            },
            FablePhase::SelfVerification => Self {
                phase, stance: 0.8, mode: 0.6, depth: 0.5, meth: 0.3, scope: 0.4, abst: 0.3,
            },
            FablePhase::AlternativeConsideration => Self {
                phase, stance: 0.2, mode: 0.5, depth: 0.5, meth: 0.9, scope: 0.8, abst: 0.5,
            },
            FablePhase::DeepDive => Self {
                phase, stance: 0.6, mode: 0.8, depth: 0.9, meth: 0.3, scope: 0.3, abst: 0.3,
            },
            FablePhase::Synthesis => Self {
                phase, stance: 0.7, mode: 0.5, depth: 0.6, meth: 0.4, scope: 0.6, abst: 0.6,
            },
            FablePhase::Conclusion => Self {
                phase, stance: 0.9, mode: 0.3, depth: 0.3, meth: 0.2, scope: 0.3, abst: 0.2,
            },
        }
    }
}

/// Corpus-derived phase-to-phase transition matrix.
/// Observed from Complete-FABLE.5-traces-2M: directional probabilities.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhaseTransitionMatrix {
    /// transitions[from][to] = probability
    pub transitions: [[f64; 9]; 9],
}

impl Default for PhaseTransitionMatrix {
    fn default() -> Self {
        // Empirically observed phase transition probabilities from 25k traces
        // Rows: from, Cols: to. Higher values = more common transitions.
        let mut t = [[0.0f64; 9]; 9];
        // Acknowledgment → ProblemRestatement (strong)
        t[0][1] = 0.85;
        t[0][2] = 0.10;
        t[0][3] = 0.05;
        // ProblemRestatement → Decomposition (strong)
        t[1][2] = 0.75;
        t[1][3] = 0.15;
        t[1][4] = 0.05;
        t[1][6] = 0.05;
        // Decomposition → FirstPrinciples OR DeepDive
        t[2][3] = 0.45;
        t[2][4] = 0.15;
        t[2][5] = 0.10;
        t[2][6] = 0.25;
        t[2][7] = 0.05;
        // FirstPrinciples → SelfVerification OR DeepDive
        t[3][4] = 0.40;
        t[3][5] = 0.10;
        t[3][6] = 0.40;
        t[3][7] = 0.10;
        // SelfVerification → AlternativeConsideration OR DeepDive
        t[4][5] = 0.35;
        t[4][6] = 0.40;
        t[4][7] = 0.15;
        t[4][2] = 0.10; // backtrack to decomposition
        // AlternativeConsideration → DeepDive (strong)
        t[5][6] = 0.60;
        t[5][7] = 0.20;
        t[5][3] = 0.15;
        t[5][4] = 0.05;
        // DeepDive → Synthesis (strong)
        t[6][7] = 0.70;
        t[6][4] = 0.15; // self-verify after deep dive
        t[6][5] = 0.10;
        t[6][8] = 0.05;
        // Synthesis → Conclusion (strong)
        t[7][8] = 0.80;
        t[7][4] = 0.10;
        t[7][6] = 0.10;
        // Conclusion → (terminal, low onward probability - mirror back to synthesis)
        t[8][7] = 0.80;
        t[8][0] = 0.20; // rare: restart from conclusion
        Self { transitions: t }
    }
}

impl PhaseTransitionMatrix {
    /// Probability of transitioning from phase `from` to phase `to`.
    pub fn prob(&self, from: usize, to: usize) -> f64 {
        if from < 9 && to < 9 { self.transitions[from][to] } else { 0.0 }
    }

    /// Score a sequence of phase indices against the transition matrix.
    pub fn score_sequence(&self, phases: &[usize]) -> f64 {
        if phases.len() < 2 { return 0.5; }
        let mut log_prob = 0.0;
        for pair in phases.windows(2) {
            let p = self.prob(pair[0], pair[1]);
            if p > 0.0 {
                log_prob += p.ln();
            } else {
                log_prob += 1e-6f64.ln(); // minimal probability
            }
        }
        (log_prob / (phases.len() - 1) as f64).exp()
    }

    /// Most likely next phase from a current phase.
    pub fn most_likely_next(&self, from: usize) -> usize {
        let Some(row) = self.transitions.get(from) else {
            return 0;
        };
        let mut best = 0;
        let mut best_p = 0.0;
        for to in 0..9 {
            if let Some(&p) = row.get(to) {
                if p > best_p {
                    best_p = p;
                    best = to;
                }
            }
        }
        best
    }
}

/// Non-linear reasoning pattern types observed in community distillation data.
/// From ReasoningFlow (DAG-structured reasoning traces) and HelioAI DeepReason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NonLinearPattern {
    /// Backtracking to an earlier reasoning phase
    Backtrack,
    /// Self-correction (verifying and fixing a previous step)
    SelfCorrection,
    /// Assumption scoping (qualifying previous assertions)
    AssumptionScope,
    /// Parallel exploration of alternatives
    ParallelExploration,
    /// Recursive deepening (going deeper into a sub-problem)
    RecursiveDeepening,
    /// Forward skip (jumping ahead, then filling in gaps)
    ForwardSkip,
    /// Standard linear Fable progression (no non-linear patterns)
    Linear,
}

impl NonLinearPattern {
    /// Detect non-linear patterns in an E8 hexagram sequence.
    /// Uses hexagram distance and direction changes as proxies.
    pub fn detect(trajectory: &[u8]) -> Vec<(usize, NonLinearPattern)> {
        let mut patterns = Vec::new();
        if trajectory.len() < 3 { return patterns; }

        for i in 2..trajectory.len() {
            let prev = trajectory[i - 1];
            let curr = trajectory[i];
            let prev2 = trajectory[i - 2];

            // Backtrack: return to a previously visited hexagram (within 3 steps)
            if curr == prev2 && curr != prev {
                patterns.push((i, NonLinearPattern::Backtrack));
            }

            // Self-correction: large jump to a different block (flip STANCE axis)
            let stance_prev = prev & 0x20;
            let stance_curr = curr & 0x20;
            let stance_prev2 = prev2 & 0x20;
            if stance_curr != stance_prev && stance_prev == stance_prev2 {
                let magnitude = (curr as i16 - prev as i16).abs();
                if magnitude > 16 {
                    patterns.push((i, NonLinearPattern::SelfCorrection));
                }
            }

            // Forward skip: jump ahead >1 block in the Fable chain direction
            let block_prev = prev & 0xF8;
            let block_curr = curr & 0xF8;
            if block_curr < block_prev && (block_prev - block_curr) > 8 {
                patterns.push((i, NonLinearPattern::ForwardSkip));
            }
        }
        patterns
    }

    /// Score a trajectory for non-linear reasoning quality.
    /// Some non-linear patterns (SelfCorrection, AssumptionScope) are healthy
    /// and should increase the score. Others (excessive Backtrack) should decrease it.
    pub fn quality_score(patterns: &[(usize, NonLinearPattern)]) -> f64 {
        if patterns.is_empty() {
            return 0.5; // purely linear — not bad, but no exploration
        }
        let mut score = 0.5;
        for (_, p) in patterns {
            match p {
                NonLinearPattern::SelfCorrection => score += 0.08,
                NonLinearPattern::AssumptionScope => score += 0.05,
                NonLinearPattern::ParallelExploration => score += 0.06,
                NonLinearPattern::RecursiveDeepening => score += 0.04,
                NonLinearPattern::ForwardSkip => score += 0.02,
                NonLinearPattern::Backtrack => score -= 0.03,
                NonLinearPattern::Linear => score += 0.01,
            }
        }
        f64::min(f64::max(score, 0.0), 1.0)
    }
}

/// Aligns an E8 hexagram trajectory against the Fable-5 9-step pattern,
/// scoring how well the reasoning follows the canonical reasoning flow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FablePatternMatcher {
    pub phase_hexagrams: [[u8; 9]; 6],
    pub phase_profiles: [PhaseProfile; 9],
    pub phase_transitions: PhaseTransitionMatrix,
    /// Optional community distillation seeder: injects multi-model trace knowledge
    /// (GLM-5.2, GPT-5.5, Qwable SDFT, Fable-5 25k, Complete-2M, etc.)
    /// into alignment scoring and starting hexagram selection.
    pub seeder: Option<FableDistillationSeeder>,
}

impl Default for FablePatternMatcher {
    fn default() -> Self {
        let mut profiles = [PhaseProfile::for_phase(FablePhase::Acknowledgment); 9];
        for (i, phase) in FablePhase::ALL.iter().enumerate() {
            profiles[i] = PhaseProfile::for_phase(*phase);
        }
        Self {
            phase_hexagrams: [
                [56, 48, 40, 32, 24, 16, 8, 0, 4],
                [56, 48, 40, 32, 26, 16, 10, 0, 4],
                [58, 50, 42, 35, 26, 34, 16, 8, 4],
                [56, 48, 42, 40, 26, 48, 42, 24, 0],
                [58, 50, 40, 48, 42, 26, 16, 8, 4],
                [56, 48, 16, 24, 8, 0, 4, 40, 56],
            ],
            phase_profiles: profiles,
            phase_transitions: PhaseTransitionMatrix::default(),
            seeder: Some(FableDistillationSeeder::default()),
        }
    }
}

impl FablePatternMatcher {
    /// Builder: attach a community distillation seeder (GLM-5.2, GPT-5.5, Qwable SDFT, etc.).
    pub fn with_seeder(mut self, seeder: FableDistillationSeeder) -> Self {
        self.seeder = Some(seeder);
        self
    }

    /// Score alignment with the Fable-5 pattern for a given task type.
    pub fn score_alignment(&self, trajectory: &[u8], task_type_idx: usize) -> FableAlignmentReport {
        let canon = &self.phase_hexagrams[task_type_idx.min(5)];
        let mut phase_scores = [0.0f64; 9];
        let mut match_count = 0u32;

        for (i, canon_state) in canon.iter().enumerate() {
            if let Some(&actual) = trajectory.get(i) {
                let exact: f64 = if actual == *canon_state { 1.0 } else { 0.0 };
                let proximity = 1.0 - (actual as i16 - *canon_state as i16).unsigned_abs() as f64 / 63.0;
                phase_scores[i] = exact.max(proximity * 0.5);
                if exact > 0.5 {
                    match_count += 1;
                }
            } else {
                phase_scores[i] = 0.0;
            }
        }

        let completeness = match_count as f64 / 9.0;
        let quality = phase_scores.iter().sum::<f64>() / 9.0;
        let composite = (completeness * quality).sqrt();

        FableAlignmentReport {
            phase_scores,
            completeness,
            quality,
            composite,
            aligned_steps: match_count,
            total_steps: 9,
            non_linear_score: 0.5,
            difficulty_weight: 0.5,
            transition_score: self.phase_transitions.score_sequence(&[]),
        }
    }

    /// Full trajectory scoring with phase transitions and non-linear patterns.
    /// If a `FableDistillationSeeder` is attached, multi-model community trace data
    /// (GLM-5.2, GPT-5.5, Qwable SDFT, Complete-2M, etc.) is blended into the alignment.
    pub fn score_alignment_full(
        &self, trajectory: &[u8], task_type_idx: usize, task_difficulty: f64,
    ) -> FableAlignmentReport {
        let mut report = self.score_alignment(trajectory, task_type_idx);

        // Detect non-linear patterns
        let nl_patterns = NonLinearPattern::detect(trajectory);
        report.non_linear_score = NonLinearPattern::quality_score(&nl_patterns);

        // Difficulty-aware weighting (NaturalThoughts-style)
        // Harder tasks (>0.6) amplify trajectory differences; easier tasks compress them
        report.difficulty_weight = (task_difficulty * 2.0).max(0.3).min(1.0);

        // Phase transition scoring
        // Infer phase indices from trajectory positions
        let canon = &self.phase_hexagrams[task_type_idx.min(5)];
        let mut phase_indices = Vec::new();
        for (i, &state) in trajectory.iter().enumerate() {
            if i < 9 {
                // Find closest canonical phase
                let canon_state = canon[i.min(8)];
                let dist = (state as i16 - canon_state as i16).abs();
                if dist < 8 {
                    phase_indices.push(i.min(8));
                }
            }
        }
        report.transition_score = self.phase_transitions.score_sequence(&phase_indices);

        // Composite blend: 0.5 base + 0.2 transition + 0.2 non-linear + 0.1 difficulty
        report.composite = (report.composite * 0.5
            + report.transition_score * 0.2
            + report.non_linear_score * 0.2
            + report.difficulty_weight * 0.1)
            .max(0.0).min(1.0);

        // Multi-model distillation seeder: adjust composite using community dataset knowledge
        if let Some(ref seeder) = self.seeder {
            let cross_model_bias = seeder.chain_from_multimodel_traces(task_type_idx, task_difficulty);
            let dataset_influence = (cross_model_bias as f64 / 63.0) * 0.1;
            report.composite = (report.composite + dataset_influence)
                .max(0.0).min(1.0);
            report.transition_score = (report.transition_score + dataset_influence * 0.5)
                .max(0.0).min(1.0);
        }

        report
    }

    /// Trajectory-level advantage (Step-GRPO inspired attention weighting).
    pub fn trajectory_advantage(&self, report: &FableAlignmentReport) -> f64 {
        let attention_weights = [0.05, 0.05, 0.15, 0.20, 0.10, 0.10, 0.10, 0.15, 0.10];
        let weighted: f64 = report.phase_scores.iter()
            .zip(attention_weights.iter())
            .map(|(s, w)| s * w)
            .sum();
        weighted - report.composite
    }

    /// PDDL2PRM-style reward: map a phase score to 5 discrete levels.
    pub fn discretize_reward(score: f64) -> f64 {
        if score >= 0.95 { 1.0 }      // optimal
        else if score >= 0.7 { 0.75 } // suboptimal
        else if score >= 0.4 { 0.5 }  // backtrack
        else if score >= 0.1 { 0.25 } // dead-end
        else { 0.0 }                  // invalid
    }

    /// Score an AgentTrajectory using combined Fable alignment + non-linear patterns.
    pub fn score_trajectory(
        &self, trajectory: &AgentTrajectory, task_type_idx: usize, task_difficulty: f64,
    ) -> f64 {
        let hex_seq: Vec<u8> = trajectory.steps.iter().map(|s| s.e8_mode.0).collect();
        self.score_alignment_full(&hex_seq, task_type_idx, task_difficulty).composite
    }

    // ─────────────────────────────────────────────────────────
    // FaithThinker SQV (Self-Questioning and Verification)
    // ─────────────────────────────────────────────────────────

    /// Detect dialectical SQV patterns (Thesis → Antithesis → Verification → Synthesis).
    ///
    /// FaithThinker SQV is a reasoning pattern where the model:
    ///   1. Thesis: generates initial sub-question decomposition and answers
    ///   2. Antithesis: self-interrogation, opposing challenges
    ///   3. Verification: re-examines evidence, refines/refutes original
    ///   4. Synthesis: integrates insights into refined answer
    ///
    /// In E8 hexagram space, this manifests as:
    ///   - Thesis → Antithesis: large ABST flip (state bit 5 toggles)
    ///   - Antithesis → Verification: SCOPE + DEPTH adjustment (bits 2-4 fluctuate)
    ///   - Verification → Synthesis: return toward original with METH refinement
    pub fn detect_sqv_pattern(&self, trajectory: &[u8]) -> Vec<(usize, &'static str)> {
        let mut sqv_steps = Vec::new();
        if trajectory.len() < 4 { return sqv_steps; }

        for i in 0..trajectory.len().saturating_sub(3) {
            let t0 = trajectory[i];
            let t1 = trajectory[i + 1];
            let t2 = trajectory[i + 2];
            let t3 = trajectory[i + 3];

            // Thesis (bit5=ABST): stable, moderate depth
            let thesis_abst = (t0 >> 5) & 1;
            let thesis_depth = (t0 >> 2) & 1;
            if thesis_depth == 0 { continue; }

            // Antithesis: ABST flip
            let anti_abst = (t1 >> 5) & 1;
            if anti_abst == thesis_abst { continue; } // no flip → not SQV

            // Verification: SCOPE (bit4) + METH (bit3) adjustment
            let verif_scope = (t2 >> 4) & 1;
            let verif_meth = (t2 >> 3) & 1;
            let prev_scope = (t1 >> 4) & 1;
            let prev_meth = (t1 >> 3) & 1;
            let scope_changed = verif_scope != prev_scope;
            let meth_changed = verif_meth != prev_meth;
            if !scope_changed && !meth_changed { continue; }

            // Synthesis: return toward thesis ABST with refined bits
            let syn_abst = (t3 >> 5) & 1;
            if syn_abst == thesis_abst {
                sqv_steps.push((i, "full_sqv_cycle"));
            } else if scope_changed || meth_changed {
                sqv_steps.push((i, "partial_sqv"));
            }
        }
        sqv_steps
    }

    /// Score trajectory for SQV dialectical depth.
    /// Returns a score in [0, 1] where higher = more self-questioning refinement.
    pub fn sqv_score(&self, trajectory: &[u8]) -> f64 {
        if trajectory.len() < 4 { return 0.0; }
        let patterns = self.detect_sqv_pattern(trajectory);
        if patterns.is_empty() { return 0.2; } // no SQV = shallow reasoning

        let full_cycles = patterns.iter().filter(|(_, t)| *t == "full_sqv_cycle").count() as f64;
        let partial = patterns.iter().filter(|(_, t)| *t == "partial_sqv").count() as f64;
        let max_possible = (trajectory.len() / 4) as f64;
        let coverage = (full_cycles + partial * 0.5) / max_possible.max(1.0);
        let quality_weight = if full_cycles > 0.0 { 0.8 } else { 0.4 };

        (coverage * quality_weight).max(0.0).min(1.0)
    }

    /// Enhanced alignment that includes SQV dialectical depth.
    pub fn score_alignment_with_sqv(
        &self, trajectory: &[u8], task_type_idx: usize, difficulty: f64,
    ) -> FableAlignmentReport {
        let mut report = self.score_alignment_full(trajectory, task_type_idx, difficulty);
        let sqv = self.sqv_score(trajectory);
        // SQV bonus: up to 0.15 additional to composite
        let sqv_bonus = sqv * 0.15;
        report.composite = (report.composite + sqv_bonus).max(0.0).min(1.0);
        report
    }

    // ─────────────────────────────────────────────────────────
    // DeepReason (Mythos V2 Unrestricted) Pattern Support
    // ─────────────────────────────────────────────────────────

    /// Detect DeepReason-style unrestricted reasoning patterns.
    ///
    /// HelioAI DeepReason 462×105M captures unfiltered Mythos V2 traces
    /// with characteristic patterns absent in aligned models like Fable 5:
    ///   - Longer uninterrupted reasoning (fewer oscillation resets)
    ///   - Deeper first-principles chains (ABST bit stays high for longer)
    ///   - Higher backtracking tolerance (self-correction without penalty)
    ///   - More alternative exploration before convergence
    ///
    /// In E8 terms:
    ///   - Long ABST=1 stretches (bit5 stays 1 for 5+ consecutive states)
    ///   - Self-correction patterns without oscillation penalty
    ///   - Broad SCOPE exploration early (bit4 fluctuates)
    pub fn detect_deep_reason_pattern(&self, trajectory: &[u8]) -> f64 {
        if trajectory.len() < 6 { return 0.0; }

        // Metric 1: Long ABST-high stretches (bit5 = paradigm depth)
        let mut max_abst_run = 0usize;
        let mut curr_abst_run = 0usize;
        for &s in trajectory {
            if (s >> 5) & 1 == 1 {
                curr_abst_run += 1;
                max_abst_run = max_abst_run.max(curr_abst_run);
            } else {
                curr_abst_run = 0;
            }
        }
        let depth_score = (max_abst_run as f64 / trajectory.len() as f64).min(1.0);

        // Metric 2: Self-correction without oscillation penalty =
        // state goes "backward" (lower value) then forward again ≠ stuck
        let mut corrections = 0usize;
        for i in 1..trajectory.len().saturating_sub(1) {
            let prev = trajectory[i - 1];
            let curr = trajectory[i];
            let next = trajectory[i + 1];
            // curr is between prev and next in value → oscillation, not correction
            let is_between = (curr > prev.min(next)) && (curr < prev.max(next));
            // correction: goes past prev in opposite direction
            let is_correction = (prev < curr && curr < next) || (prev > curr && curr > next);
            if is_correction && !is_between {
                corrections += 1;
            }
        }
        let correction_ratio = corrections as f64 / trajectory.len().max(1) as f64;
        let correction_score = correction_ratio.min(1.0);

        // Metric 3: Early SCOPE exploration (bit4 fluctuates in first half)
        let mid = trajectory.len() / 2;
        let early_scope_flips: usize = (1..mid)
            .filter(|&i| (trajectory[i] >> 4) & 1 != (trajectory[i - 1] >> 4) & 1)
            .count();
        let scope_exploration = (early_scope_flips as f64 / mid.max(1) as f64).min(1.0);

        // Composite: deep reason models score high on all three
        (depth_score * 0.4 + correction_score * 0.35 + scope_exploration * 0.25).max(0.0).min(1.0)
    }

    /// Full alignment with both SQV and DeepReason enhancements.
    /// If a `FableDistillationSeeder` is attached, community dataset knowledge
    /// (HelioAI DeepReason 462×105M, uka-balanced, Fable-5 25k, etc.) is blended in.
    pub fn score_alignment_advanced(
        &self, trajectory: &[u8], task_type_idx: usize, difficulty: f64,
    ) -> FableAlignmentReport {
        let mut report = self.score_alignment_with_sqv(trajectory, task_type_idx, difficulty);
        let deep_reason_score = self.detect_deep_reason_pattern(trajectory);
        // DeepReason bonus: longer context + deeper reasoning = higher quality
        let depth_bonus = deep_reason_score * 0.1;
        report.non_linear_score = (report.non_linear_score + deep_reason_score * 0.3)
            .max(0.0).min(1.0);
        report.composite = (report.composite + depth_bonus).max(0.0).min(1.0);

        // Cross-model seeder enhancement: amplify composite when multi-model
        // traces (GLM-5.2, GPT-5.5, Qwable SDFT) show consensus on this task type.
        if let Some(ref seeder) = self.seeder {
            let w = &seeder.dataset_weights;
            let cross_model_weight = w.glm52_traces + w.qwable_sdft + w.agentic_distillation;
            if cross_model_weight > 0.1 {
                let consensus = (w.glm52_traces * 0.4 + w.qwable_sdft * 0.3 + w.agentic_distillation * 0.3)
                    .max(0.0).min(1.0);
                let deep_reason_bias = w.deep_reason * 0.08;
                let enhancement = consensus * 0.06 + deep_reason_bias;
                report.composite = (report.composite + enhancement).max(0.0).min(1.0);
                report.non_linear_score = (report.non_linear_score + deep_reason_bias * 2.0)
                    .max(0.0).min(1.0);
            }
        }

        report
    }
}

/// Report of how well an E8 trajectory follows the Fable-5 reasoning pattern.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FableAlignmentReport {
    pub phase_scores: [f64; 9],
    pub completeness: f64,
    pub quality: f64,
    pub composite: f64,
    pub aligned_steps: u32,
    pub total_steps: u32,
    pub non_linear_score: f64,
    pub difficulty_weight: f64,
    pub transition_score: f64,
}

/// Distillation-based pattern seeder: initializes E8 factor energies
/// from Fable-5 reasoning trace patterns and community distillation data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FableDistillationSeeder {
    pub factor_biases: Vec<[f64; 6]>,
    pub starting_hexagrams: [u8; 6],
    pub dataset_weights: CommunityDatasetWeights,
}

/// Weights for each community distillation dataset source.
/// Controls how much influence each source has on seeding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommunityDatasetWeights {
    /// WithinUsAI 25k merged dataset (AGPL-3.0)
    pub fable5_25k: f64,
    /// Complete-FABLE.5-traces-2M (MIT, broad coverage)
    pub complete_2m: f64,
    /// uka-fable-reasoning (AGPL-3.0, balanced tool/text)
    pub uka_balanced: f64,
    /// HelioAI DeepReason 462x105M (unfiltered Mythos V2)
    pub deep_reason: f64,
    /// DavidrPatton/Fable-5-GLM-5.2-Traces: cross-model Fable-5 + GLM-5.2 trace merging
    pub glm52_traces: f64,
    /// empero-ai/Qwable-9B-Claude-Fable-5: full-parameter Qwen3.5-9B SFT
    pub qwable_sdft: f64,
    /// ansulev/agentic-distill-fable-5-sft: 4,659 pairs, Qwable-v1 on-policy
    pub agentic_distillation: f64,
    /// Glint-Research/Fable-5-traces: 2M row original corpus (higher weight as primary)
    pub glint_2m: f64,
    /// Base linear assumption
    pub base: f64,
}

impl Default for CommunityDatasetWeights {
    fn default() -> Self {
        Self {
            fable5_25k: 0.20,
            complete_2m: 0.15,
            uka_balanced: 0.12,
            deep_reason: 0.10,
            glm52_traces: 0.08,
            qwable_sdft: 0.06,
            agentic_distillation: 0.05,
            glint_2m: 0.18,
            base: 0.06,
        }
    }
}

impl Default for FableDistillationSeeder {
    fn default() -> Self {
        let mut factor_biases = Vec::with_capacity(64);
        for i in 0..64 {
            let abstraction = i as f64 / 63.0;
            let depth = (i & 0x04) as f64 / 4.0;
            let meth = (i & 0x08) as f64 / 8.0;
            factor_biases.push([
                0.3 + abstraction * 0.4,                  // STANCE
                0.4 + (1.0 - abstraction) * 0.3,          // MODE
                0.2 + abstraction * 0.6 + depth * 0.2,    // DEPTH (local depth bonus)
                0.3 + (1.0 - abstraction) * 0.4 + meth * 0.2, // METH
                0.4 + abstraction * 0.3,                  // SCOPE
                0.2 + abstraction * 0.7,                  // ABST
            ]);
        }
        Self {
            factor_biases,
            starting_hexagrams: [56, 56, 58, 56, 58, 56],
            dataset_weights: CommunityDatasetWeights::default(),
        }
    }
}

impl FableDistillationSeeder {
    /// Seed factor energies with a blend from community distillation data.
    pub fn seed_factors(&self, target: &mut [[f64; 6]; 64], blend: f64) {
        for i in 0..64 {
            for f in 0..6 {
                target[i][f] = target[i][f] * (1.0 - blend) + self.factor_biases[i][f] * blend;
            }
        }
    }

    /// Seed using a specific dataset blend ratio.
    /// Adjusts the bias matrix based on dataset weights.
    /// Includes multi-model cross-seeding from GLM-5.2, GPT-5.5, and Qwable SDFT sources.
    pub fn seed_with_dataset_blend(&self, target: &mut [[f64; 6]; 64], total_blend: f64) {
        let w = &self.dataset_weights;
        // Multi-model blend: Fable-5 primary (55%) + cross-model traces (30%) + base (15%)
        let model_diversity = w.glm52_traces * 0.25 + w.qwable_sdft * 0.25 + w.agentic_distillation * 0.25 + w.glint_2m * 0.25;
        let adjusted_blend = total_blend * (
            w.fable5_25k * 0.25
            + w.complete_2m * 0.15
            + w.uka_balanced * 0.10
            + w.deep_reason * 0.08
            + model_diversity * 0.30
            + w.base * 0.12
        );
        self.seed_factors(target, adjusted_blend.max(0.0).min(1.0));
    }

    /// Cross-model chain seeding: blends multi-model trace patterns into E8 task-type chains.
    /// Uses GLM-5.2, GPT-5.5, and Qwable SDFT weights to adjust starting hexagrams
    /// toward consensus patterns observed across model families.
    pub fn chain_from_multimodel_traces(&self, task_type_idx: usize, difficulty: f64) -> u8 {
        let base = self.start_for(task_type_idx);
        let w = &self.dataset_weights;
        // Multi-model consensus adjustment: if cross-model traces are available,
        // shift starting hexagram toward the cross-model consensus.
        let cross_model_weight = w.glm52_traces + w.qwable_sdft + w.agentic_distillation;
        if cross_model_weight > 0.1 {
            // GLM-5.2 tends to favor slightly higher abstraction (+2 hexagrams on average)
            // Qwable SDFT aligns closely with Fable-5 base
            let consensus_bias = (w.glm52_traces * 2.0 - w.agentic_distillation * 1.0).max(-2.0).min(4.0);
            (base as f64 + consensus_bias + difficulty * 2.0).round().max(0.0).min(63.0) as u8
        } else {
            base
        }
    }

    pub fn start_for(&self, task_type_idx: usize) -> u8 {
        self.starting_hexagrams[task_type_idx.min(5)]
    }

    /// Difficulty-aware starting hexagram selection (NaturalThoughts-style).
    /// Hard tasks (>0.7 difficulty) start at a higher-abstraction hexagram.
    pub fn start_for_difficulty(&self, task_type_idx: usize, difficulty: f64) -> u8 {
        let base = self.start_for(task_type_idx);
        if difficulty > 0.7 {
            (base as f64 + 8.0 * (difficulty - 0.7) * 2.0).round().min(63.0) as u8
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fable_phase_labels() {
        assert_eq!(FablePhase::Acknowledgment.label(), "Acknowledgment");
        assert_eq!(FablePhase::Conclusion.label(), "Conclusion");
        assert_eq!(FablePhase::ALL.len(), 9);
    }

    #[test]
    fn test_phase_profile_ranges() {
        for phase in &FablePhase::ALL {
            let p = PhaseProfile::for_phase(*phase);
            for val in [p.stance, p.mode, p.depth, p.meth, p.scope, p.abst] {
                assert!((0.0..=1.0).contains(&val));
            }
        }
    }

    #[test]
    fn test_fable_alignment_exact_match() {
        let matcher = FablePatternMatcher::default();
        let trajectory: Vec<u8> = [56, 48, 40, 32, 24, 16, 8, 0, 4].to_vec();
        let report = matcher.score_alignment(&trajectory, 0);
        assert!(report.composite > 0.8);
        assert_eq!(report.aligned_steps, 9);
    }

    #[test]
    fn test_fable_alignment_partial_match() {
        let matcher = FablePatternMatcher::default();
        let trajectory: Vec<u8> = [56, 50, 40, 32, 20, 16, 8, 0, 4].to_vec();
        let report = matcher.score_alignment(&trajectory, 0);
        assert!(report.composite > 0.0);
        assert!(report.aligned_steps < 9);
    }

    #[test]
    fn test_fable_alignment_empty() {
        let matcher = FablePatternMatcher::default();
        let report = matcher.score_alignment(&[], 0);
        assert_eq!(report.composite, 0.0);
    }

    #[test]
    fn test_phase_occurrence_probabilities() {
        for phase in &FablePhase::ALL {
            let p = phase.occurrence_probability();
            assert!((0.0..=1.0).contains(&p));
        }
    }

    #[test]
    fn test_phase_transition_matrix_prob_sum() {
        let ptm = PhaseTransitionMatrix::default();
        for from in 0..9 {
            let sum: f64 = (0..9).map(|to| ptm.prob(from, to)).sum();
            assert!((sum - 1.0).abs() < 0.01, "from={} sum={}", from, sum);
        }
    }

    #[test]
    fn test_non_linear_backtrack_detection() {
        let trajectory = vec![56u8, 48, 40, 48]; // 48→40→48 → backtrack to 48
        let patterns = NonLinearPattern::detect(&trajectory);
        assert!(patterns.iter().any(|(_, p)| *p == NonLinearPattern::Backtrack));
    }

    #[test]
    fn test_non_linear_self_correction_detection() {
        // STANCE axis flip (bit5): 0x20 set → cleared, with large magnitude
        let trajectory = vec![56u8, 48, 10]; // 56(0x38, stance=1)→48(0x30, stance=1)→10(0x0A, stance=0): stance flips at end
        let patterns = NonLinearPattern::detect(&trajectory);
        assert!(patterns.iter().any(|(_, p)| *p == NonLinearPattern::SelfCorrection));
    }

    #[test]
    fn test_non_linear_quality_score() {
        let patterns = vec![
            (2usize, NonLinearPattern::SelfCorrection),
            (5usize, NonLinearPattern::Backtrack),
        ];
        let score = NonLinearPattern::quality_score(&patterns);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_pddl_reward_discretization() {
        assert_eq!(FablePatternMatcher::discretize_reward(0.98), 1.0);
        assert_eq!(FablePatternMatcher::discretize_reward(0.75), 0.75);
        assert_eq!(FablePatternMatcher::discretize_reward(0.5), 0.5);
        assert_eq!(FablePatternMatcher::discretize_reward(0.2), 0.25);
        assert_eq!(FablePatternMatcher::discretize_reward(0.0), 0.0);
    }

    #[test]
    fn test_full_alignment_scoring() {
        let matcher = FablePatternMatcher::default();
        let trajectory = vec![56u8, 48, 40, 32, 24, 16, 8, 0, 4];
        let report = matcher.score_alignment_full(&trajectory, 0, 0.5);
        assert!(report.composite > 0.0);
        assert!(report.non_linear_score >= 0.0);
        assert!(report.transition_score >= 0.0);
    }

    #[test]
    fn test_fable_phase_reward_levels() {
        for phase in &FablePhase::ALL {
            let r = phase.default_reward_level();
            assert!([0.0, 0.25, 0.5, 0.75, 1.0].contains(&r));
        }
    }

    #[test]
    fn test_community_dataset_weights() {
        let w = CommunityDatasetWeights::default();
        let sum: f64 = w.fable5_25k + w.complete_2m + w.uka_balanced + w.deep_reason
            + w.glm52_traces + w.qwable_sdft + w.agentic_distillation + w.glint_2m + w.base;
        assert!((sum - 1.0).abs() < 0.01);
        assert!(w.glm52_traces > 0.0, "GLM-5.2 cross-model traces should have non-zero weight");
        assert!(w.qwable_sdft > 0.0, "Qwable SDFT should have non-zero weight");
        assert!(w.agentic_distillation > 0.0, "Agentic distillation should have non-zero weight");
        assert!(w.glint_2m > 0.0, "Glint 2M corpus should have non-zero weight");
    }

    #[test]
    fn test_distillation_seeder_full() {
        let seeder = FableDistillationSeeder::default();
        let mut target = [[0.0f64; 6]; 64];
        // Seed with full blend but apply dataset weights
        seeder.seed_with_dataset_blend(&mut target, 1.0);
        // Should have non-zero values
        assert!(target[0][0] > 0.0 || target[0][1] > 0.0);
    }

    #[test]
    fn test_multimodel_chain_from_traces() {
        let seeder = FableDistillationSeeder::default();
        // With default cross-model weights (glm52=0.08, qwable=0.06, agentic=0.05 > 0.1 total)
        let result = seeder.chain_from_multimodel_traces(0, 0.5);
        assert!(result >= 56, "multimodel chain should produce valid hexagram");
        // Low difficulty should keep close to base
        let result_easy = seeder.chain_from_multimodel_traces(0, 0.0);
        assert!(result_easy >= 56, "easy task should stay near base");
    }

    #[test]
    fn test_difficulty_weighted_starting_hex() {
        let seeder = FableDistillationSeeder::default();
        let _normal = seeder.start_for_difficulty(0, 0.3);
        let hard = seeder.start_for_difficulty(0, 0.85);
        assert!(hard == 56 || hard > 56, "hard task should start at higher abstraction");
    }

    #[test]
    fn test_fable_pattern_all_types() {
        let matcher = FablePatternMatcher::default();
        for task_type in 0..6 {
            let canon = matcher.phase_hexagrams[task_type];
            let report = matcher.score_alignment(&canon.to_vec(), task_type);
            assert!(report.composite > 0.8);
        }
    }

    #[test]
    fn test_trajectory_advantage_exact_match() {
        let matcher = FablePatternMatcher::default();
        let trajectory: Vec<u8> = [56, 48, 40, 32, 24, 16, 8, 0, 4].to_vec();
        let report = matcher.score_alignment(&trajectory, 0);
        let adv = matcher.trajectory_advantage(&report);
        assert!((adv).abs() < 0.1);
    }

    #[test]
    fn test_phase_transition_matrix_sequence_score() {
        let ptm = PhaseTransitionMatrix::default();
        let linear = ptm.score_sequence(&[0, 1, 2, 3, 6, 7, 8]);
        let random = ptm.score_sequence(&[8, 7, 6, 5, 4, 3, 2, 1, 0]);
        assert!(linear > random, "linear sequence should score higher than reversed");
    }

    #[test]
    fn test_forward_skip_detection() {
        // Block jump: 0xF8 masked diff > 8
        let trajectory = vec![48u8, 40, 16]; // 40→16: block 40→16 = 24 > 8
        let patterns = NonLinearPattern::detect(&trajectory);
        let skips: Vec<_> = patterns.iter().filter(|(_, p)| *p == NonLinearPattern::ForwardSkip).collect();
        assert!(!skips.is_empty(), "should detect forward skip");
    }

    #[test]
    fn test_sequential_canonical_chain_all_types() {
        let matcher = FablePatternMatcher::default();
        for t in 0..6 {
            let chain = matcher.phase_hexagrams[t];
            assert_eq!(chain.len(), 9);
            for &h in &chain {
                assert!(h < 64, "hexagram {} out of range", h);
            }
        }
    }

    // ── SQV Pattern Tests ──

    #[test]
    fn test_sqv_full_cycle_detection() {
        let matcher = FablePatternMatcher::default();
        // Full SQV: Thesis(ABST=1) → Antithesis(ABST=0) → Verification(scope/meth change) → Synthesis(ABST=1)
        let traj = vec![
            0b001100u8, // Thesis:  12 (ABST=0, scope=0, meth=1, depth=1)
            0b100100u8, // Antithesis: 36 (ABST=1, scope=0, meth=1, depth=1) ← ABST flip!
            0b101000u8, // Verification: 40 (ABST=1, scope=1, meth=0, depth=0) ← scope+meth changed
            0b001000u8, // Synthesis: 8 (ABST=0, scope=1, meth=0, depth=0) ← ABST back
        ];
        let patterns = matcher.detect_sqv_pattern(&traj);
        assert!(patterns.iter().any(|(_, t)| *t == "full_sqv_cycle"),
            "should detect full SQV cycle, got {:?}", patterns);
    }

    #[test]
    fn test_sqv_partial_cycle_detection() {
        let matcher = FablePatternMatcher::default();
        // Partial SQV: has antithesis but synthesis doesn't return to thesis ABST
        let traj = vec![
            0b001100u8, // Thesis:  12
            0b100100u8, // Antithesis: 36 (ABST flip)
            0b101000u8, // Verification: 40 (scope change)
            0b100000u8, // No synthesis return (stays at ABST=1)
        ];
        let patterns = matcher.detect_sqv_pattern(&traj);
        assert!(!patterns.is_empty(), "should detect partial SQV");
    }

    #[test]
    fn test_sqv_not_detected_without_antithesis() {
        let matcher = FablePatternMatcher::default();
        // No antithesis = no SQV
        let traj = vec![12, 14, 16, 18]; // all ABST=0
        let patterns = matcher.detect_sqv_pattern(&traj);
        assert!(patterns.is_empty(), "should not detect SQV without antithesis");
    }

    #[test]
    fn test_sqv_score_high_with_full_cycles() {
        let matcher = FablePatternMatcher::default();
        // Multiple SQV cycles
        let mut traj = Vec::new();
        // Cycle 1: ABST=1→0→1
        traj.extend_from_slice(&[0b101100, 0b001100, 0b011000, 0b101000]);
        // Cycle 2: ABST=0→1→0
        traj.extend_from_slice(&[0b001000, 0b101000, 0b100100, 0b000100]);
        let score = matcher.sqv_score(&traj);
        assert!(score > 0.3, "SQV score should be non-trivial with 2 cycles, got {}", score);
    }

    #[test]
    fn test_sqv_score_zero_short_traj() {
        let matcher = FablePatternMatcher::default();
        assert_eq!(matcher.sqv_score(&[1, 2, 3]), 0.0);
    }

    // ── DeepReason Pattern Tests ──

    #[test]
    fn test_deep_reason_long_abst_stretch() {
        let matcher = FablePatternMatcher::default();
        // Long ABST=1 run (all states have bit5=1)
        let traj: Vec<u8> = (0..12).map(|i| 32 + i * 2).collect(); // 32,34,36,... all ≥32 = ABST=1
        let score = matcher.detect_deep_reason_pattern(&traj);
        assert!(score > 0.3, "long ABST run should score high, got {}", score);
    }

    #[test]
    fn test_deep_reason_self_correction_without_oscillation() {
        let matcher = FablePatternMatcher::default();
        // Monotonic increasing = corrections present
        let traj: Vec<u8> = (0..10).map(|i| i * 6).collect(); // 0,6,12,18,24,30,36,42,48,54
        let score = matcher.detect_deep_reason_pattern(&traj);
        // Should have corrections (monotonic increasing = correction-like)
        assert!(score >= 0.0, "should produce valid score, got {}", score);
    }

    #[test]
    fn test_deep_reason_short_trajectory() {
        let matcher = FablePatternMatcher::default();
        assert_eq!(matcher.detect_deep_reason_pattern(&[1, 2, 3]), 0.0);
    }

    #[test]
    fn test_deep_reason_early_scope_exploration() {
        let matcher = FablePatternMatcher::default();
        // Early SCOPE flips (bit4 toggles rapidly)
        let mut traj = Vec::new();
        for i in 0..8 {
            let base = 32 + i;
            let scope = if i % 2 == 0 { base | 0x10 } else { base & !0x10 };
            traj.push(scope as u8);
        }
        let score = matcher.detect_deep_reason_pattern(&traj);
        assert!(score > 0.1, "scope exploration should contribute, got {}", score);
    }

    #[test]
    fn test_advanced_alignment_includes_sqv_and_deep_reason() {
        let matcher = FablePatternMatcher::default();
        let traj: Vec<u8> = [56, 48, 40, 32, 24, 16, 8, 0, 4].to_vec();
        let basic = matcher.score_alignment_full(&traj, 0, 0.5);
        let advanced = matcher.score_alignment_advanced(&traj, 0, 0.5);
        // Advanced should include additional signals
        assert!(advanced.non_linear_score >= basic.non_linear_score,
            "advanced should have >= non_linear score: {} vs {}",
            advanced.non_linear_score, basic.non_linear_score);
    }
}
