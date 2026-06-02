use neotrix_types::skill::{Skill, SkillRegistry};

/// Action-sequence mining: discover reusable patterns from execution history
pub trait SkillMiner {
    fn mine_patterns(history: &[ExecutionStep]) -> Vec<ActionSequence>;
    fn extract_frequent_sequences(steps: &[ExecutionStep], min_support: usize) -> Vec<Vec<String>>;
}

/// Self-diagnosis: analyze skill failures and suggest repairs
pub trait SkillDiagnoser {
    fn diagnose(skill_output: &SkillOutput, expected: &str) -> DiagnosisResult;
    fn suggest_repair(diagnosis: &DiagnosisResult) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub action: String,
    pub result: String,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ActionSequence {
    pub actions: Vec<String>,
    pub frequency: usize,
    pub avg_success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SkillOutput {
    pub skill_name: String,
    pub steps_executed: Vec<String>,
    pub final_result: String,
    pub reward: f64,
}

#[derive(Debug, Clone)]
pub struct DiagnosisResult {
    pub needs_repair: bool,
    pub issue: String,
    pub severity: f64,
}

pub struct SkillCrystallizer {
    pub registry: SkillRegistry,
    min_reward_threshold: f64,
}

impl SkillCrystallizer {
    pub fn new() -> Self {
        Self {
            registry: SkillRegistry::new(),
            min_reward_threshold: 0.7,
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            registry: SkillRegistry::new(),
            min_reward_threshold: threshold,
        }
    }

    pub fn crystallize(
        &mut self,
        task_description: &str,
        task_type: &str,
        reward: f64,
    ) -> Option<String> {
        if reward < self.min_reward_threshold {
            return None;
        }

        let name = Self::generate_skill_name(task_description);
        let description = format!("Auto-crystallized skill from '{}' task (reward: {:.3})", task_type, reward);
        let tags = vec![task_type.to_string(), "auto-crystallized".to_string()];
        let steps = Self::abstract_steps(task_type, task_description);

        let skill = Skill::new(
            name,
            description,
            tags,
            steps,
            vec![reward, 0.0, 0.0, 0.0],
        );

        let id = skill.id.clone();
        self.registry.register(skill);
        Some(id)
    }

    fn generate_skill_name(description: &str) -> String {
        let words: Vec<&str> = description.split_whitespace().collect();
        let truncated: String = words.iter().take(5).cloned().collect::<Vec<&str>>().join("_");
        if truncated.is_empty() { "unnamed_skill".to_string() } else { truncated.to_lowercase() }
    }

    fn abstract_steps(task_type: &str, _description: &str) -> Vec<String> {
        match task_type {
            "code_generation" => vec![
                "Analyze requirements and existing code structure".to_string(),
                "Design solution approach with type signatures".to_string(),
                "Implement core logic with error handling".to_string(),
                "Add tests for edge cases".to_string(),
                "Verify compilation and run tests".to_string(),
            ],
            "debugging" => vec![
                "Reproduce the error and capture full trace".to_string(),
                "Isolate the root cause via binary search".to_string(),
                "Implement fix with regression test".to_string(),
                "Verify no new failures introduced".to_string(),
            ],
            "refactoring" => vec![
                "Map current code structure and dependencies".to_string(),
                "Define target architecture".to_string(),
                "Apply refactoring in small incremental steps".to_string(),
                "Verify behavior preservation with tests".to_string(),
            ],
            _ => vec![
                "Understand task requirements".to_string(),
                "Design solution approach".to_string(),
                "Implement solution".to_string(),
                "Verify correctness".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystallize_high_reward() {
        let mut crystallizer = SkillCrystallizer::new();
        let id = crystallizer.crystallize(
            "Implement a binary search tree",
            "code_generation",
            0.85,
        );
        assert!(id.is_some());
        assert_eq!(crystallizer.registry.len(), 1);
    }

    #[test]
    fn test_crystallize_low_reward_skip() {
        let mut crystallizer = SkillCrystallizer::new();
        let id = crystallizer.crystallize(
            "Failed attempt",
            "code_generation",
            0.3,
        );
        assert!(id.is_none());
        assert_eq!(crystallizer.registry.len(), 0);
    }

    #[test]
    fn test_skill_reuse_tracking() {
        let mut crystallizer = SkillCrystallizer::new();
        let id = crystallizer.crystallize("Test task", "debugging", 0.9).expect("value should be ok in test");
        let skill = crystallizer.registry.get_mut(&id).expect("value should be ok in test");
        assert_eq!(skill.reuse_count, 0);
        skill.record_reuse(0.8);
        assert_eq!(skill.reuse_count, 1);
        assert!((skill.average_reward() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_skill_search() {
        let mut crystallizer = SkillCrystallizer::new();
        crystallizer.crystallize("Binary search tree", "code_generation", 0.9);
        let results = crystallizer.registry.search("binary");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_threshold_configurable() {
        let mut crystallizer = SkillCrystallizer::with_threshold(0.5);
        assert!(crystallizer.crystallize("Medium task", "refactoring", 0.6).is_some());
        assert!(crystallizer.crystallize("Low task", "refactoring", 0.4).is_none());
    }

    #[test]
    fn test_abstract_steps_by_type() {
        let steps = SkillCrystallizer::abstract_steps("code_generation", "test");
        assert!(steps.len() >= 4);
        assert!(steps[0].contains("Analyze"));

        let debug_steps = SkillCrystallizer::abstract_steps("debugging", "test");
        assert!(debug_steps[0].contains("Reproduce"));
    }
}
