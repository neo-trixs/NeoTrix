use super::brain_core::ReasoningBrain;
use crate::neotrix::nt_mind::self_edit::MicroEdit;
use rand::Rng;

/// Context carrying editing parameters through the diffusion process.
pub struct EditContext<'a> {
    pub task: &'a str,
    pub brain: &'a ReasoningBrain,
    pub noise_level: f64,
}

/// Heuristic critic for scoring edit proposals.
#[derive(Debug, Clone)]
pub struct EditCritic {
    pub ideal_min_edits: usize,
    pub ideal_max_edits: usize,
    pub min_magnitude: f64,
    pub max_magnitude: f64,
}

impl Default for EditCritic {
    fn default() -> Self {
        Self {
            ideal_min_edits: 3,
            ideal_max_edits: 8,
            min_magnitude: 0.01,
            max_magnitude: 0.25,
        }
    }
}

impl EditCritic {
    /// Score a list of MicroEdits from 0.0 (bad) to 1.0 (perfect).
    pub fn score_edits(&self, edits: &[MicroEdit], _task: &str) -> f64 {
        if edits.is_empty() {
            return 0.0;
        }

        let mut score = 0.5;

        if edits.len() >= self.ideal_min_edits && edits.len() <= self.ideal_max_edits {
            score += 0.2;
        } else if edits.len() < self.ideal_min_edits {
            score -= 0.1 * (self.ideal_min_edits - edits.len()) as f64;
        } else {
            score -= 0.05 * (edits.len() - self.ideal_max_edits) as f64;
        }

        for edit in edits {
            if let MicroEdit::AdjustDimension(_, amount) = edit {
                let mag = amount.abs();
                if mag < self.min_magnitude || mag > self.max_magnitude {
                    score -= 0.1;
                }
            }
        }

        if edits.iter().any(|e| matches!(e, MicroEdit::NormalizeVector)) {
            score += 0.1;
        }

        if edits.iter().any(|e| matches!(e, MicroEdit::UpdateLearningRate(_))) {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    pub fn is_valid(&self, edits: &[MicroEdit]) -> bool {
        !edits.is_empty() && edits.iter().any(|e| matches!(e, MicroEdit::AdjustDimension(_, _)))
    }
}

/// DGM-Hyperagent self-edit strategy: wraps `generate_self_edit()` with
/// diffusion-like iterative refinement through multiple denoising steps.
#[derive(Debug, Clone)]
pub struct DgmSelfEditStrategy {
    pub num_diffusion_steps: usize,
    pub noise_schedule: Vec<f64>,
    pub critic: EditCritic,
}

impl DgmSelfEditStrategy {
    pub fn new(num_steps: usize) -> Self {
        Self {
            num_diffusion_steps: num_steps,
            noise_schedule: vec![0.3, 0.1, 0.01],
            critic: EditCritic::default(),
        }
    }

    pub fn with_schedule(num_steps: usize, noise_schedule: Vec<f64>) -> Self {
        Self {
            num_diffusion_steps: num_steps,
            noise_schedule,
            critic: EditCritic::default(),
        }
    }

    /// Generate a self-edit via the full diffusion process.
    pub fn generate_via_diffusion(&self, context: &EditContext) -> Vec<MicroEdit> {
        let task = context.task;
        if task.is_empty() {
            return Vec::new();
        }

        let initial_edits = context.brain.generate_self_edit(task);

        if self.num_diffusion_steps == 0 {
            return initial_edits;
        }

        let noise_0 = self.noise_schedule.first().copied().unwrap_or(0.3);
        let mut current = self.add_noise(&initial_edits, noise_0);
        let mut best_edits = current.clone();
        let mut best_score = self.critic.score_edits(&best_edits, task);

        for step in 0..self.num_diffusion_steps {
            let noise_level = self
                .noise_schedule
                .get(step)
                .copied()
                .unwrap_or(0.01);

            let ctx = EditContext {
                task,
                brain: context.brain,
                noise_level,
            };
            let refined = self.denoise_step(&current, step, &ctx);

            if !refined.is_empty() {
                let score = self.critic.score_edits(&refined, task);
                if score > best_score {
                    best_edits = refined.clone();
                    best_score = score;
                }
                current = refined;
            }
        }

        best_edits
    }

    /// Single refinement step: apply denoising + critic-guided adjustment.
    pub fn denoise_step(
        &self,
        edits: &[MicroEdit],
        _step: usize,
        context: &EditContext,
    ) -> Vec<MicroEdit> {
        let noise_level = context.noise_level;
        let mut refined: Vec<MicroEdit> = Vec::new();

        for edit in edits {
            match edit {
                MicroEdit::AdjustDimension(dim, amount) => {
                    let denoised = amount * (1.0 - noise_level * 0.3);
                    let clamped = denoised.clamp(-0.5, 0.5);
                    refined.push(MicroEdit::AdjustDimension(dim.clone(), clamped));
                }
                other => refined.push(other.clone()),
            }
        }

        if !refined.iter().any(|e| matches!(e, MicroEdit::NormalizeVector)) {
            refined.push(MicroEdit::NormalizeVector);
        }

        refined
    }

    fn add_noise(&self, edits: &[MicroEdit], noise_level: f64) -> Vec<MicroEdit> {
        let mut rng = rand::thread_rng();
        edits
            .iter()
            .map(|edit| match edit {
                MicroEdit::AdjustDimension(dim, amount) => {
                    let noise = (rng.gen::<f64>() - 0.5) * 2.0 * noise_level;
                    MicroEdit::AdjustDimension(dim.clone(), amount + noise)
                }
                other => other.clone(),
            })
            .collect()
    }

    /// Convert a SelfEdit to MicroEdits (bridge if needed).
    pub fn edit_to_micro(
        &self,
        target_dims: &[String],
        magnitude: f64,
        learning_rate: f64,
    ) -> Vec<MicroEdit> {
        let mut edits: Vec<MicroEdit> = target_dims
            .iter()
            .map(|dim| MicroEdit::AdjustDimension(dim.clone(), magnitude))
            .collect();
        edits.push(MicroEdit::UpdateLearningRate(learning_rate));
        edits.push(MicroEdit::NormalizeVector);
        edits
    }
}

/// Orchestrator that coordinates DGM diffusion across multiple candidate edits.
pub struct DgmEditOrchestrator {
    pub num_candidates: usize,
    pub dgm: DgmSelfEditStrategy,
}

impl DgmEditOrchestrator {
    pub fn new(num_candidates: usize, dgm: DgmSelfEditStrategy) -> Self {
        Self {
            num_candidates,
            dgm,
        }
    }

    /// Generate the single best edit across multiple parallel diffusion runs.
    pub fn generate_best_edit(&self, brain: &ReasoningBrain, task: &str) -> Vec<MicroEdit> {
        if task.is_empty() {
            return Vec::new();
        }

        let mut best_edits: Vec<MicroEdit> = Vec::new();
        let mut best_score = -1.0f64;

        let initial_edits = brain.generate_self_edit(task);

        for _ in 0..self.num_candidates {
            let noise_0 = self.dgm.noise_schedule.first().copied().unwrap_or(0.3);
            let mut current = self.dgm.add_noise(&initial_edits, noise_0);
            let mut local_best = current.clone();
            let mut local_best_score = self.dgm.critic.score_edits(&local_best, task);

            for step in 0..self.dgm.num_diffusion_steps {
                let noise_level = self
                    .dgm
                    .noise_schedule
                    .get(step)
                    .copied()
                    .unwrap_or(0.01);

                let ctx = EditContext {
                    task,
                    brain,
                    noise_level,
                };
                let refined = self.dgm.denoise_step(&current, step, &ctx);

                if !refined.is_empty() {
                    let score = self.dgm.critic.score_edits(&refined, task);
                    if score > local_best_score {
                        local_best = refined.clone();
                        local_best_score = score;
                    }
                    current = refined;
                }
            }

            if local_best_score > best_score {
                best_edits = local_best;
                best_score = local_best_score;
            }
        }

        best_edits
    }

    /// Return all candidate diffusion trajectories (for analysis).
    pub fn orchestrate_diffusion(
        &self,
        brain: &ReasoningBrain,
        task: &str,
    ) -> Vec<Vec<MicroEdit>> {
        if task.is_empty() || self.num_candidates == 0 {
            return Vec::new();
        }

        let initial_edits = brain.generate_self_edit(task);
        let mut trajectories: Vec<Vec<MicroEdit>> = Vec::new();

        for _ in 0..self.num_candidates {
            let noise_0 = self.dgm.noise_schedule.first().copied().unwrap_or(0.3);
            let mut current = self.dgm.add_noise(&initial_edits, noise_0);

            for step in 0..self.dgm.num_diffusion_steps {
                let noise_level = self
                    .dgm
                    .noise_schedule
                    .get(step)
                    .copied()
                    .unwrap_or(0.01);

                let ctx = EditContext {
                    task,
                    brain,
                    noise_level,
                };
                let refined = self.dgm.denoise_step(&current, step, &ctx);
                if !refined.is_empty() {
                    if step == self.dgm.num_diffusion_steps - 1 {
                        trajectories.push(refined.clone());
                    }
                    current = refined;
                }
            }
        }

        trajectories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

    #[test]
    fn test_zero_steps_is_identity() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(0);
        let ctx = EditContext {
            task: "design a UI",
            brain: &brain,
            noise_level: 0.0,
        };
        let result = dgm.generate_via_diffusion(&ctx);
        let direct = brain.generate_self_edit("design a UI");
        assert_eq!(result.len(), direct.len());
        for (r, d) in result.iter().zip(direct.iter()) {
            match (r, d) {
                (MicroEdit::AdjustDimension(r_dim, r_val), MicroEdit::AdjustDimension(d_dim, d_val)) => {
                    assert_eq!(r_dim, d_dim);
                    assert!((r_val - d_val).abs() < 1e-10);
                }
                _ => assert_eq!(format!("{:?}", r), format!("{:?}", d)),
            }
        }
    }

    #[test]
    fn test_three_step_diffusion_produces_valid_edits() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(3);
        let ctx = EditContext {
            task: "design a responsive UI with Tailwind",
            brain: &brain,
            noise_level: 0.0,
        };
        let result = dgm.generate_via_diffusion(&ctx);
        assert!(!result.is_empty());
        assert!(dgm.critic.is_valid(&result));
    }

    #[test]
    fn test_empty_task_returns_empty() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(3);
        let ctx = EditContext {
            task: "",
            brain: &brain,
            noise_level: 0.0,
        };
        let result = dgm.generate_via_diffusion(&ctx);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_step_produces_valid_edit() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(1);
        let ctx = EditContext {
            task: "analyze code performance",
            brain: &brain,
            noise_level: 0.0,
        };
        let result = dgm.generate_via_diffusion(&ctx);
        assert!(!result.is_empty());
        assert!(dgm.critic.is_valid(&result));
        assert!(result.iter().any(|e| matches!(e, MicroEdit::NormalizeVector)));
    }

    #[test]
    fn test_critic_scores_valid_edits() {
        let edits = vec![
            MicroEdit::AdjustDimension("typography".into(), 0.1),
            MicroEdit::AdjustDimension("grid".into(), 0.15),
            MicroEdit::UpdateLearningRate(0.05),
            MicroEdit::NormalizeVector,
        ];
        let critic = EditCritic::default();
        let score = critic.score_edits(&edits, "design");
        assert!(score > 0.0);
        assert!(score <= 1.0);
        assert!(critic.is_valid(&edits));
    }

    #[test]
    fn test_critic_scores_empty_as_zero() {
        let critic = EditCritic::default();
        let score = critic.score_edits(&[], "");
        assert!((score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_orchestrator_generates_best_edit() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(2);
        let orch = DgmEditOrchestrator::new(3, dgm);
        let result = orch.generate_best_edit(&brain, "build a React dashboard");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_orchestrator_empty_task() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(2);
        let orch = DgmEditOrchestrator::new(3, dgm);
        let result = orch.generate_best_edit(&brain, "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_orchestrator_trajectories() {
        let brain = ReasoningBrain::new();
        let dgm = DgmSelfEditStrategy::new(2);
        let orch = DgmEditOrchestrator::new(2, dgm);
        let trajectories = orch.orchestrate_diffusion(&brain, "optimize SQL queries");
        assert_eq!(trajectories.len(), 2);
        for t in &trajectories {
            assert!(!t.is_empty());
        }
    }

    #[test]
    fn test_edit_to_micro() {
        let dgm = DgmSelfEditStrategy::new(3);
        let dims = vec!["analysis".to_string(), "synthesis".to_string()];
        let edits = dgm.edit_to_micro(&dims, 0.1, 0.05);
        assert_eq!(edits.len(), 4);
        assert!(edits.iter().any(|e| matches!(e, MicroEdit::NormalizeVector)));
        assert!(edits.iter().any(|e| matches!(e, MicroEdit::UpdateLearningRate(0.05))));
    }

    #[test]
    fn test_denoise_reduces_extreme_values() {
        let dgm = DgmSelfEditStrategy::new(3);
        let edits = vec![MicroEdit::AdjustDimension("test".into(), 0.9)];
        let ctx = EditContext {
            task: "test",
            brain: &ReasoningBrain::new(),
            noise_level: 0.3,
        };
        let result = dgm.denoise_step(&edits, 0, &ctx);
        if let MicroEdit::AdjustDimension(_, val) = &result[0] {
            assert!(*val < 0.9);
        }
    }
}
