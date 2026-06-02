use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditStep {
    pub step_index: usize,
    pub action: String,
    pub state: String,
    pub outcome: String,
    pub llm_attribution: f64,
    pub token_advantage: f64,
    pub composite_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAssignmentResult {
    pub trajectory_id: String,
    pub steps: Vec<CreditStep>,
    pub total_reward: f64,
    pub mean_attribution: f64,
    pub max_attribution: f64,
    pub min_attribution: f64,
    pub critical_steps: Vec<usize>,
    pub detrimental_steps: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditConfig {
    pub critical_threshold: f64,
    pub detrimental_threshold: f64,
    pub use_llm_attribution: bool,
    pub llm_temperature: f64,
    pub gamma: f64,
}

impl Default for CreditConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 0.7,
            detrimental_threshold: 0.2,
            use_llm_attribution: true,
            llm_temperature: 0.3,
            gamma: 0.95,
        }
    }
}

pub struct CreditAssigner {
    config: CreditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryData {
    pub id: String,
    pub actions: Vec<String>,
    pub states: Vec<String>,
    pub outcomes: Vec<String>,
    pub final_reward: f64,
    pub success: bool,
    pub token_counts: Vec<usize>,
}

impl CreditAssigner {
    pub fn new(config: CreditConfig) -> Self {
        Self { config }
    }

    pub fn assign_credit(&self, trajectory: &TrajectoryData) -> CreditAssignmentResult {
        let attributions = if self.config.use_llm_attribution {
            self.llm_attribution(trajectory)
        } else {
            self.heuristic_attribution(trajectory)
        };

        let total_steps = trajectory.actions.len();
        let outcome_reward = if trajectory.success { trajectory.final_reward } else { 0.0 };

        let mut steps = Vec::with_capacity(total_steps);
        for i in 0..total_steps {
            let action = trajectory.actions[i].clone();
            let state = trajectory.states.get(i).cloned().unwrap_or_default();
            let outcome = trajectory.outcomes.get(i).cloned().unwrap_or_default();
            let attribution = attributions[i];
            let composite = self.composite_reward_impl(attribution, i, total_steps, outcome_reward);

            steps.push(CreditStep {
                step_index: i,
                action,
                state,
                outcome,
                llm_attribution: attribution,
                token_advantage: attribution,
                composite_reward: composite,
            });
        }

        let total_reward: f64 = steps.iter().map(|s| s.composite_reward).sum();
        let mean_attribution = if attributions.is_empty() {
            0.0
        } else {
            attributions.iter().sum::<f64>() / attributions.len() as f64
        };
        let max_attribution = attributions.iter().copied().fold(0.0, f64::max);
        let min_attribution = if attributions.is_empty() {
            0.0
        } else {
            attributions.iter().copied().fold(f64::MAX, f64::min)
        };

        let (critical_steps, detrimental_steps) = self.classify_steps(&steps);

        CreditAssignmentResult {
            trajectory_id: trajectory.id.clone(),
            steps,
            total_reward,
            mean_attribution,
            max_attribution,
            min_attribution,
            critical_steps,
            detrimental_steps,
        }
    }

    pub fn llm_attribution(&self, trajectory: &TrajectoryData) -> Vec<f64> {
        self.heuristic_attribution(trajectory)
    }

    pub fn heuristic_attribution(&self, trajectory: &TrajectoryData) -> Vec<f64> {
        let n = trajectory.actions.len();
        if n == 0 {
            return vec![];
        }
        let mut attributions = Vec::with_capacity(n);
        for i in 0..n {
            let recency = (i + 1) as f64 / n as f64;
            let outcome = trajectory.outcomes.get(i).map(|s| s.as_str()).unwrap_or("");
            let outcome_boost = if outcome.to_lowercase().contains("success")
                || outcome.to_lowercase().contains("complete")
            {
                0.2
            } else if outcome.to_lowercase().contains("fail")
                || outcome.to_lowercase().contains("error")
            {
                -0.1
            } else {
                0.0
            };
            let val = (recency + outcome_boost).clamp(0.0, 1.0);
            attributions.push(val);
        }
        attributions
    }

    pub fn compute_token_advantage(
        &self,
        step_attributions: &[f64],
        token_counts: &[usize],
    ) -> Vec<f64> {
        let mut advantages = Vec::new();
        for (i, &count) in token_counts.iter().enumerate() {
            let attr = step_attributions.get(i).copied().unwrap_or(0.0);
            for _ in 0..count {
                advantages.push(attr);
            }
        }
        advantages
    }

    fn composite_reward_impl(
        &self,
        attribution: f64,
        step_index: usize,
        total_steps: usize,
        outcome_reward: f64,
    ) -> f64 {
        let discount = self
            .config
            .gamma
            .powi((total_steps - step_index) as i32);
        attribution * discount + outcome_reward
    }

    pub fn composite_reward(
        &self,
        step: &CreditStep,
        step_index: usize,
        total_steps: usize,
        outcome_reward: f64,
    ) -> f64 {
        self.composite_reward_impl(step.llm_attribution, step_index, total_steps, outcome_reward)
    }

    pub fn classify_steps(&self, steps: &[CreditStep]) -> (Vec<usize>, Vec<usize>) {
        let mut critical = Vec::new();
        let mut detrimental = Vec::new();
        for step in steps {
            if step.llm_attribution >= self.config.critical_threshold {
                critical.push(step.step_index);
            }
            if step.llm_attribution < self.config.detrimental_threshold {
                detrimental.push(step.step_index);
            }
        }
        (critical, detrimental)
    }

    pub fn report(&self, result: &CreditAssignmentResult) -> String {
        let mut s = String::new();
        s.push_str("=== ADCA Credit Assignment Report ===\n");
        s.push_str(&format!("Trajectory: {}\n", result.trajectory_id));
        s.push_str(&format!("Total Reward: {:.4}\n", result.total_reward));
        s.push_str(&format!("Mean Attribution: {:.4}\n", result.mean_attribution));
        s.push_str(&format!("Max Attribution: {:.4}\n", result.max_attribution));
        s.push_str(&format!("Min Attribution: {:.4}\n", result.min_attribution));
        s.push_str(&format!(
            "Critical Steps (>={:.1}): {:?}\n",
            self.config.critical_threshold, result.critical_steps
        ));
        s.push_str(&format!(
            "Detrimental Steps (<{:.1}): {:?}\n",
            self.config.detrimental_threshold, result.detrimental_steps
        ));
        s.push_str("\n--- Step Breakdown ---\n");
        for step in &result.steps {
            s.push_str(&format!(
                "  #{}: action={:.30} attribution={:.4} advantage={:.4} reward={:.4}\n",
                step.step_index,
                step.action,
                step.llm_attribution,
                step.token_advantage,
                step.composite_reward
            ));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(index: usize, attribution: f64) -> CreditStep {
        CreditStep {
            step_index: index,
            action: String::new(),
            state: String::new(),
            outcome: String::new(),
            llm_attribution: attribution,
            token_advantage: attribution,
            composite_reward: 0.0,
        }
    }

    #[test]
    fn test_heuristic_attribution() {
        let config = CreditConfig {
            use_llm_attribution: false,
            ..Default::default()
        };
        let assigner = CreditAssigner::new(config);
        let trajectory = TrajectoryData {
            id: "test".into(),
            actions: vec!["look".into(), "move".into(), "grab".into()],
            states: vec!["room".into(), "hall".into(), "table".into()],
            outcomes: vec![
                "found nothing".into(),
                "saw object".into(),
                "success".into(),
            ],
            final_reward: 10.0,
            success: true,
            token_counts: vec![5, 8, 6],
        };
        let result = assigner.assign_credit(&trajectory);
        assert_eq!(result.steps.len(), 3);
        assert!(result.steps[2].llm_attribution >= result.steps[0].llm_attribution);
        assert!(result.steps[2].llm_attribution > result.steps[1].llm_attribution);
    }

    #[test]
    fn test_compute_token_advantage() {
        let config = CreditConfig::default();
        let assigner = CreditAssigner::new(config);
        let attributions = vec![0.3, 0.8, 0.5];
        let token_counts = vec![2, 3, 1];
        let advantages = assigner.compute_token_advantage(&attributions, &token_counts);
        assert_eq!(advantages, vec![0.3, 0.3, 0.8, 0.8, 0.8, 0.5]);
        assert_eq!(advantages.len(), 6);
    }

    #[test]
    fn test_composite_reward_discount() {
        let config = CreditConfig {
            gamma: 0.5,
            ..Default::default()
        };
        let assigner = CreditAssigner::new(config);
        let step0 = make_step(0, 1.0);
        let r0 = assigner.composite_reward(&step0, 0, 3, 0.0);
        assert!((r0 - 0.125).abs() < 1e-10);
        let step2 = make_step(2, 1.0);
        let r2 = assigner.composite_reward(&step2, 2, 3, 0.0);
        assert!((r2 - 0.5).abs() < 1e-10);
        assert!(r2 > r0);
    }

    #[test]
    fn test_classify_critical_steps() {
        let config = CreditConfig {
            critical_threshold: 0.7,
            ..Default::default()
        };
        let assigner = CreditAssigner::new(config);
        let steps = vec![make_step(0, 0.5), make_step(1, 0.8), make_step(2, 0.9)];
        let (critical, _) = assigner.classify_steps(&steps);
        assert_eq!(critical, vec![1, 2]);
    }

    #[test]
    fn test_classify_detrimental_steps() {
        let config = CreditConfig {
            detrimental_threshold: 0.2,
            ..Default::default()
        };
        let assigner = CreditAssigner::new(config);
        let steps = vec![make_step(0, 0.1), make_step(1, 0.5), make_step(2, 0.15)];
        let (_, detrimental) = assigner.classify_steps(&steps);
        assert_eq!(detrimental, vec![0, 2]);
    }

    #[test]
    fn test_default_config() {
        let config = CreditConfig::default();
        assert!((config.critical_threshold - 0.7).abs() < 1e-10);
        assert!((config.detrimental_threshold - 0.2).abs() < 1e-10);
        assert!(config.use_llm_attribution);
        assert!((config.llm_temperature - 0.3).abs() < 1e-10);
        assert!((config.gamma - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_report_generation() {
        let config = CreditConfig::default();
        let assigner = CreditAssigner::new(config);
        let trajectory = TrajectoryData {
            id: "report-test".into(),
            actions: vec!["a1".into(), "a2".into()],
            states: vec!["s1".into(), "s2".into()],
            outcomes: vec!["o1".into(), "o2".into()],
            final_reward: 5.0,
            success: true,
            token_counts: vec![1, 1],
        };
        let result = assigner.assign_credit(&trajectory);
        let report = assigner.report(&result);
        assert!(report.contains("ADCA Credit Assignment Report"));
        assert!(report.contains("report-test"));
        assert!(report.contains("Step Breakdown"));
        assert!(report.contains("attribution"));
    }
}
