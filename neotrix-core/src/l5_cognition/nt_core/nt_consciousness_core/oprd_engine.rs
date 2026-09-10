//! OPRD Engine — On-Policy Reverse Distillation for Self-Evolution
//!
//! Based on: "Eliciting Weak-to-Strong Generalization with On-Policy Reverse Distillation"
//! (arXiv:2609.08798v1, Park et al., 2026)
//!
//! Key insight: Instead of matching a teacher's final policy, extract the teacher's
//! *policy shift* (change from reference to final) and use it to rescale the student's
//! own optimization gradient. This preserves stationary points while adding alignment gain.
//!
//! NeoTrix adaptation:
//! - "Teacher" = weaker/cheaper model (or previous evolution cycle)
//! - "Student" = stronger/current model
//! - "Policy shift" = knowledge delta from evolution cycle
//! - "Gradient" = evolution direction (knowledge gain vector)
//! - "Verifier" = convergence checker + quality gate

/// Configuration for OPRD-based evolution
#[derive(Debug, Clone)]
pub struct OprdConfig {
    /// Scaling strength (λ in paper). Higher = more teacher influence.
    pub scaling_strength: f64,
    /// Warm-up steps before negative alignment scaling activates
    pub warmup_steps: u32,
    /// Maximum number of evolution cycles
    pub max_cycles: u32,
    /// Convergence threshold (stop when improvement < threshold)
    pub convergence_threshold: f64,
    /// Number of consecutive no-improvement cycles before stopping
    pub convergence_patience: u32,
}

impl Default for OprdConfig {
    fn default() -> Self {
        Self {
            scaling_strength: 1.0,
            warmup_steps: 10,
            max_cycles: 100,
            convergence_threshold: 0.01,
            convergence_patience: 5,
        }
    }
}

/// A knowledge dimension in the evolution space
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnowledgeDimension {
    pub name: String,
    pub domain: String,
}

/// Policy shift: the change in knowledge between reference and current state
#[derive(Debug, Clone)]
pub struct PolicyShift {
    /// Direction of change in knowledge space
    pub direction: Vec<f64>,
    /// Magnitude of the shift
    pub magnitude: f64,
    /// Which dimensions were affected
    pub affected_dims: Vec<KnowledgeDimension>,
}

/// Evolution gradient: the student's own optimization direction
#[derive(Debug, Clone)]
pub struct EvolutionGradient {
    /// Raw gradient direction
    pub direction: Vec<f64>,
    /// Alignment coefficient with teacher shift (u_t in paper)
    pub alignment: f64,
    /// Projected component along teacher direction
    pub projected: Vec<f64>,
    /// Orthogonal component (unchanged by OPRD)
    pub orthogonal: Vec<f64>,
}

/// Result of a single OPRD evolution step
#[derive(Debug, Clone)]
pub struct OprdStepResult {
    /// Cycle number
    pub cycle: u32,
    /// Teacher's policy shift
    pub teacher_shift: PolicyShift,
    /// Student's original gradient
    pub student_gradient: EvolutionGradient,
    /// Rescaled gradient (after OPRD)
    pub rescaled_gradient: Vec<f64>,
    /// Scaling coefficient used (λ_t)
    pub lambda_t: f64,
    /// Alignment gain (λ_t * u_t^2)
    pub alignment_gain: f64,
    /// Whether positive or negative alignment was used
    pub alignment_type: AlignmentType,
}

/// Type of alignment scaling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentType {
    /// Teacher and student agree (u_t >= 0)
    Positive,
    /// Teacher and student disagree (u_t < 0)
    Negative,
}

/// OPRD Engine for self-evolution
pub struct OprdEngine {
    config: OprdConfig,
    /// Reference policy (initial state)
    #[allow(dead_code)]
    reference_state: Vec<f64>,
    /// Current policy (student)
    #[allow(dead_code)]
    current_state: Vec<f64>,
    /// History of evolution steps
    history: Vec<OprdStepResult>,
    /// Current cycle
    cycle: u32,
    /// Convergence tracking
    best_improvement: f64,
    no_improvement_count: u32,
}

impl OprdEngine {
    /// Create a new OPRD engine
    pub fn new(config: OprdConfig, initial_state: Vec<f64>) -> Self {
        Self {
            config,
            reference_state: initial_state.clone(),
            current_state: initial_state,
            history: Vec::new(),
            cycle: 0,
            best_improvement: f64::INFINITY,
            no_improvement_count: 0,
        }
    }

    /// Compute teacher's policy shift (Eq. 2.4 in paper)
    ///
    /// Δ_t = C(z_T(s_t) - z_T^ref(s_t))
    /// where C is mean-centering operation
    pub fn compute_teacher_shift(
        &self,
        teacher_policy: &[f64],
        reference_policy: &[f64],
    ) -> PolicyShift {
        assert_eq!(teacher_policy.len(), reference_policy.len());

        // Compute difference
        let diff: Vec<f64> = teacher_policy
            .iter()
            .zip(reference_policy.iter())
            .map(|(t, r)| t - r)
            .collect();

        // Mean-centering (C operation)
        let mean: f64 = diff.iter().sum::<f64>() / diff.len() as f64;
        let centered: Vec<f64> = diff.iter().map(|d| d - mean).collect();

        // Compute magnitude
        let magnitude: f64 = centered.iter().map(|d| d * d).sum::<f64>().sqrt();

        PolicyShift {
            direction: centered,
            magnitude,
            affected_dims: Vec::new(), // Would be populated with actual dimensions
        }
    }

    /// Compute student's evolution gradient
    ///
    /// g_t = A_t * ∇_{z_t} log π_θ(y_t | s_t)
    pub fn compute_student_gradient(
        &self,
        teacher_shift: &PolicyShift,
        student_improvement: &[f64],
    ) -> EvolutionGradient {
        assert_eq!(teacher_shift.direction.len(), student_improvement.len());

        // Normalize teacher shift to unit direction (d_t)
        let d: Vec<f64> = if teacher_shift.magnitude > 1e-10 {
            teacher_shift
                .direction
                .iter()
                .map(|d| d / teacher_shift.magnitude)
                .collect()
        } else {
            vec![0.0; teacher_shift.direction.len()]
        };

        // Compute alignment coefficient (u_t = d_t^T * g_t)
        let alignment: f64 = d
            .iter()
            .zip(student_improvement.iter())
            .map(|(di, gi)| di * gi)
            .sum();

        // Project onto teacher direction
        let projected: Vec<f64> = d.iter().map(|di| alignment * di).collect();

        // Orthogonal component
        let orthogonal: Vec<f64> = student_improvement
            .iter()
            .zip(projected.iter())
            .map(|(gi, pi)| gi - pi)
            .collect();

        EvolutionGradient {
            direction: student_improvement.to_vec(),
            alignment,
            projected,
            orthogonal,
        }
    }

    /// Apply OPRD gradient scaling (Eq. 2.5 in paper)
    ///
    /// g̃_t = g_t + λ_t * Proj_{d_t}(g_t)
    ///      = (1 + λ_t) * Proj_{d_t}(g_t) + g_t^⊥
    pub fn apply_oprd_scaling(
        &mut self,
        teacher_shift: PolicyShift,
        student_gradient: EvolutionGradient,
    ) -> OprdStepResult {
        // Compute λ_t with asymmetric scaling (Eq. 2.8)
        let lambda_t = if student_gradient.alignment >= 0.0 {
            // Positive alignment: full scaling from start
            self.config.scaling_strength
        } else {
            // Negative alignment: warm-up ramp
            let warmup_factor =
                (self.cycle as f64 / self.config.warmup_steps as f64).min(1.0);
            self.config.scaling_strength * warmup_factor
        };

        // Compute alignment type
        let alignment_type = if student_gradient.alignment >= 0.0 {
            AlignmentType::Positive
        } else {
            AlignmentType::Negative
        };

        // Apply scaling: g̃_t = g_t + λ_t * Proj_{d_t}(g_t)
        let rescaled_gradient: Vec<f64> = student_gradient
            .direction
            .iter()
            .zip(student_gradient.projected.iter())
            .map(|(gi, pi)| gi + lambda_t * pi)
            .collect();

        // Compute alignment gain (λ_t * u_t^2)
        let alignment_gain = lambda_t * student_gradient.alignment * student_gradient.alignment;

        let result = OprdStepResult {
            cycle: self.cycle,
            teacher_shift: teacher_shift.clone(),
            student_gradient: student_gradient.clone(),
            rescaled_gradient,
            lambda_t,
            alignment_gain,
            alignment_type,
        };

        self.history.push(result.clone());
        self.cycle += 1;

        result
    }

    /// Check if evolution has converged
    pub fn check_convergence(&mut self, current_improvement: f64) -> bool {
        if current_improvement < self.config.convergence_threshold {
            self.no_improvement_count += 1;
        } else {
            self.no_improvement_count = 0;
            self.best_improvement = current_improvement;
        }

        self.no_improvement_count >= self.config.convergence_patience
    }

    /// Convenience: run one full OPRD cycle.
    ///
    /// 1. Compute teacher shift from teacher vs reference
    /// 2. Compute student gradient from student vs reference
    /// 3. Apply OPRD scaling
    /// 4. Check convergence
    ///
    /// Returns `Some(OprdStepResult)` if the cycle ran, `None` if converged.
    pub fn run_cycle(
        &mut self,
        student_state: &[f64],
        teacher_state: &[f64],
    ) -> Option<OprdStepResult> {
        let reference = self.reference_state.clone();
        let teacher_shift = self.compute_teacher_shift(teacher_state, &reference);

        // Skip cycles with negligible teacher shift
        if teacher_shift.magnitude < 1e-10 {
            return None;
        }

        let student_improvement: Vec<f64> = student_state
            .iter()
            .zip(reference.iter())
            .map(|(s, r)| s - r)
            .collect();

        let student_gradient = self.compute_student_gradient(&teacher_shift, &student_improvement);
        let result = self.apply_oprd_scaling(teacher_shift, student_gradient);

        let _converged = self.check_convergence(result.alignment_gain);
        Some(result)
    }

    /// Get the reference policy state (initial state).
    pub fn reference_state(&self) -> &[f64] {
        &self.reference_state
    }

    /// Get the current policy state (student).
    pub fn current_state(&self) -> &[f64] {
        &self.current_state
    }

    /// Get the current cycle number.
    pub fn cycle(&self) -> u32 {
        self.cycle
    }

    /// Get evolution summary
    pub fn summary(&self) -> OprdSummary {
        let total_gain: f64 = self.history.iter().map(|h| h.alignment_gain).sum();
        let positive_count = self
            .history
            .iter()
            .filter(|h| h.alignment_type == AlignmentType::Positive)
            .count();
        let negative_count = self
            .history
            .iter()
            .filter(|h| h.alignment_type == AlignmentType::Negative)
            .count();

        OprdSummary {
            total_cycles: self.cycle,
            total_alignment_gain: total_gain,
            positive_alignments: positive_count as u32,
            negative_alignments: negative_count as u32,
            average_scaling: if self.cycle > 0 {
                self.history.iter().map(|h| h.lambda_t).sum::<f64>() / self.cycle as f64
            } else {
                0.0
            },
            converged: self.no_improvement_count >= self.config.convergence_patience,
        }
    }
}

/// Summary of OPRD evolution
#[derive(Debug, Clone)]
pub struct OprdSummary {
    pub total_cycles: u32,
    pub total_alignment_gain: f64,
    pub positive_alignments: u32,
    pub negative_alignments: u32,
    pub average_scaling: f64,
    pub converged: bool,
}

impl std::fmt::Display for OprdSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "  OPRD Evolution Summary")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "  Total Cycles: {}", self.total_cycles)?;
        writeln!(f, "  Total Alignment Gain: {:.4}", self.total_alignment_gain)?;
        writeln!(
            f,
            "  Positive Alignments: {} (teacher-student agree)",
            self.positive_alignments
        )?;
        writeln!(
            f,
            "  Negative Alignments: {} (student diverges from teacher)",
            self.negative_alignments
        )?;
        writeln!(f, "  Average Scaling: {:.4}", self.average_scaling)?;
        writeln!(
            f,
            "  Converged: {}",
            if self.converged { "Yes" } else { "No" }
        )?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teacher_shift_computation() {
        let engine = OprdEngine::new(OprdConfig::default(), vec![0.0; 10]);
        let teacher = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let reference = vec![0.0; 10];

        let shift = engine.compute_teacher_shift(&teacher, &reference);
        assert!(shift.magnitude > 0.0);
    }

    #[test]
    fn test_oprd_preserves_stationarity() {
        let config = OprdConfig::default();
        let mut engine = OprdEngine::new(config, vec![0.0; 10]);

        let teacher_shift = PolicyShift {
            direction: vec![1.0; 10],
            magnitude: 10.0_f64.sqrt(),
            affected_dims: Vec::new(),
        };

        // Zero gradient should remain zero after OPRD
        let zero_gradient = EvolutionGradient {
            direction: vec![0.0; 10],
            alignment: 0.0,
            projected: vec![0.0; 10],
            orthogonal: vec![0.0; 10],
        };

        let result = engine.apply_oprd_scaling(teacher_shift, zero_gradient);
        assert!(result.rescaled_gradient.iter().all(|x| x.is_finite() && (*x - 0.0).abs() < 1e-10));
    }

    #[test]
    fn test_alignment_gain_nonnegative() {
        let config = OprdConfig::default();
        let mut engine = OprdEngine::new(config, vec![0.0; 10]);

        let teacher_shift = PolicyShift {
            direction: vec![1.0; 10],
            magnitude: 10.0_f64.sqrt(),
            affected_dims: Vec::new(),
        };

        let student_gradient = EvolutionGradient {
            direction: vec![0.5; 10],
            alignment: 5.0,
            projected: vec![0.5; 10],
            orthogonal: vec![0.0; 10],
        };

        let result = engine.apply_oprd_scaling(teacher_shift, student_gradient);
        assert!(result.alignment_gain >= 0.0);
    }

    #[test]
    fn test_convergence_detection() {
        let config = OprdConfig {
            convergence_threshold: 0.1,
            convergence_patience: 3,
            ..Default::default()
        };
        let mut engine = OprdEngine::new(config, vec![0.0; 10]);

        // Simulate decreasing improvements
        assert!(!engine.check_convergence(0.5));
        assert!(!engine.check_convergence(0.05));
        assert!(!engine.check_convergence(0.05));
        assert!(engine.check_convergence(0.05)); // 3 consecutive below threshold
    }
}
