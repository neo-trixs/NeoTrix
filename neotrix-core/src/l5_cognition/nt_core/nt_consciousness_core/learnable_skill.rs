#![forbid(unsafe_code)]

//! Learnable Skill Agent (可学习技能代理)
//!
//! Based on the CoSkill paper: skills are learnable agents, not static templates.
//! Key principles:
//!   - Joint training with reasoning backbone
//!   - Hierarchical skill library (task skills → step skills)
//!   - Co-evolution with reasoning agent
//!
//! Skills learn from experience, adapt to new contexts, and co-evolve
//! with the reasoning system they serve.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learnable Skill Agent — a skill that learns and evolves
pub struct LearnableSkillAgent {
    /// Hierarchical skill library
    pub library: SkillLibrary,
    /// Co-evolution engine
    pub coevolution: CoEvolutionEngine,
    /// Agent configuration
    pub config: SkillAgentConfig,
    /// Execution history
    pub history: Vec<SkillExecutionRecord>,
}

/// Skill agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAgentConfig {
    /// Learning rate for skill adaptation
    pub learning_rate: f64,
    /// Maximum skill library size
    pub max_library_size: usize,
    /// Co-evolution sync interval (number of executions)
    pub coevolution_sync_interval: usize,
    /// Minimum fitness to retain a skill
    pub min_fitness_threshold: f64,
}

impl Default for SkillAgentConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            max_library_size: 500,
            coevolution_sync_interval: 10,
            min_fitness_threshold: 0.3,
        }
    }
}

/// Hierarchical skill library
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillLibrary {
    /// Task-level skills (high-level)
    pub task_skills: Vec<TaskSkill>,
    /// Step-level skills (low-level, composed by task skills)
    pub step_skills: Vec<StepSkill>,
    /// Skill mapping: task skill ID → list of step skill IDs
    pub composition_map: HashMap<String, Vec<String>>,
}

/// A task-level skill (high-level, goal-oriented)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Goal this skill achieves
    pub goal: String,
    /// Composed step skill IDs (in execution order)
    pub step_sequence: Vec<String>,
    /// Fitness score (learned from execution)
    pub fitness: f64,
    /// Learning history
    pub learning_history: Vec<LearningEvent>,
    /// Current version
    pub version: u32,
    /// Applicable contexts
    pub applicable_contexts: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// A step-level skill (low-level, atomic operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Input schema
    pub input_schema: Vec<String>,
    /// Output schema
    pub output_schema: Vec<String>,
    /// Fitness score
    pub fitness: f64,
    /// Parameters (learned weights)
    pub parameters: HashMap<String, f64>,
    /// Learning history
    pub learning_history: Vec<LearningEvent>,
    /// Version
    pub version: u32,
}

/// A learning event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Event ID
    pub id: String,
    /// Fitness before
    pub fitness_before: f64,
    /// Fitness after
    pub fitness_after: f64,
    /// Context
    pub context: String,
    /// What was learned
    pub insight: String,
    /// Timestamp
    pub timestamp: String,
}

/// Co-evolution engine — syncs skill evolution with reasoning
pub struct CoEvolutionEngine {
    /// Reasoning agent state
    pub reasoning_state: ReasoningState,
    /// Skill adaptation rules
    pub adaptation_rules: Vec<AdaptationRule>,
    /// Co-evolution history
    pub history: Vec<CoevolutionEvent>,
}

/// Reasoning agent state (simplified)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReasoningState {
    /// Current reasoning strategy
    pub strategy: String,
    /// Reasoning performance metrics
    pub metrics: HashMap<String, f64>,
    /// Known failure patterns
    pub failure_patterns: Vec<FailurePattern>,
    /// Last sync timestamp
    pub last_sync: Option<String>,
}

/// A failure pattern detected in reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Pattern ID
    pub id: String,
    /// Description
    pub description: String,
    /// Affected skill types
    pub affected_skills: Vec<String>,
    /// Suggested adaptation
    pub suggested_adaptation: String,
    /// Frequency
    pub frequency: u32,
}

/// Adaptation rule for co-evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRule {
    /// Rule ID
    pub id: String,
    /// Trigger condition
    pub trigger: String,
    /// Action to take
    pub action: String,
    /// Priority
    pub priority: u32,
    /// Whether rule is active
    pub active: bool,
}

/// Co-evolution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoevolutionEvent {
    /// Event ID
    pub id: String,
    /// Skills affected
    pub affected_skills: Vec<String>,
    /// Adaptation applied
    pub adaptation: String,
    /// Fitness delta
    pub fitness_delta: f64,
    /// Reasoning sync result
    pub sync_result: String,
    /// Timestamp
    pub timestamp: String,
}

/// Skill execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionRecord {
    /// Record ID
    pub id: String,
    /// Task skill executed
    pub task_skill_id: String,
    /// Steps executed
    pub step_skill_ids: Vec<String>,
    /// Input
    pub input: HashMap<String, String>,
    /// Output
    pub output: HashMap<String, String>,
    /// Success
    pub success: bool,
    /// Fitness delta
    pub fitness_delta: f64,
    /// Duration (ms)
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: String,
}

/// Error type for skill agent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillAgentError {
    /// Skill not found
    SkillNotFound(String),
    /// Invalid composition
    InvalidComposition(String),
    /// Library full
    LibraryFull,
    /// Learning failed
    LearningFailed(String),
    /// Co-evolution sync failed
    SyncFailed(String),
}

impl std::fmt::Display for SkillAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SkillNotFound(id) => write!(f, "Skill not found: {}", id),
            Self::InvalidComposition(msg) => write!(f, "Invalid composition: {}", msg),
            Self::LibraryFull => write!(f, "Skill library full"),
            Self::LearningFailed(msg) => write!(f, "Learning failed: {}", msg),
            Self::SyncFailed(msg) => write!(f, "Co-evolution sync failed: {}", msg),
        }
    }
}

impl std::error::Error for SkillAgentError {}

impl LearnableSkillAgent {
    /// Create a new Learnable Skill Agent
    pub fn new(config: SkillAgentConfig) -> Self {
        Self {
            library: SkillLibrary::default(),
            coevolution: CoEvolutionEngine {
                reasoning_state: ReasoningState::default(),
                adaptation_rules: Vec::new(),
                history: Vec::new(),
            },
            config,
            history: Vec::new(),
        }
    }

    /// Execute a task skill
    pub fn execute_task_skill(
        &mut self,
        task_skill_id: &str,
        input: HashMap<String, String>,
    ) -> Result<HashMap<String, String>, SkillAgentError> {
        let task_skill = self
            .library
            .task_skills
            .iter()
            .find(|s| s.id == task_skill_id)
            .ok_or_else(|| SkillAgentError::SkillNotFound(task_skill_id.to_string()))?;

        let step_ids = task_skill.step_sequence.clone();
        let mut output = input.clone();
        let mut all_success = true;

        // Execute step skills in sequence
        for step_id in &step_ids {
            let step_skill = self
                .library
                .step_skills
                .iter()
                .find(|s| s.id == step_id.as_str())
                .ok_or_else(|| SkillAgentError::SkillNotFound(step_id.clone()))?;

            // Simulate execution
            let step_success = step_skill.fitness > 0.3;
            if step_success {
                output.insert(
                    format!("{}_output", step_skill.name),
                    format!("processed_{}", output.len()),
                );
            } else {
                all_success = false;
                break;
            }
        }

        // Record execution
        let record = SkillExecutionRecord {
            id: format!("exec_{}", uuid::Uuid::new_v4()),
            task_skill_id: task_skill_id.to_string(),
            step_skill_ids: step_ids,
            input: input.clone(),
            output: output.clone(),
            success: all_success,
            fitness_delta: 0.0,
            duration_ms: 100,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);

        if all_success {
            Ok(output)
        } else {
            Err(SkillAgentError::LearningFailed(
                "Step execution failed".to_string(),
            ))
        }
    }

    /// Learn from execution feedback
    pub fn learn_from_execution(
        &mut self,
        execution_id: &str,
        success: bool,
        feedback: &str,
    ) -> Result<(), SkillAgentError> {
        let record = self
            .history
            .iter()
            .find(|r| r.id == execution_id)
            .ok_or_else(|| SkillAgentError::SkillNotFound(execution_id.to_string()))?;

        let fitness_delta = if success {
            self.config.learning_rate
        } else {
            -self.config.learning_rate
        };

        // Update task skill fitness
        if let Some(task_skill) = self
            .library
            .task_skills
            .iter_mut()
            .find(|s| s.id == record.task_skill_id)
        {
            let old_fitness = task_skill.fitness;
            task_skill.fitness = (task_skill.fitness + fitness_delta).max(0.0).min(1.0);

            let event = LearningEvent {
                id: format!("learn_{}", uuid::Uuid::new_v4()),
                fitness_before: old_fitness,
                fitness_after: task_skill.fitness,
                context: feedback.to_string(),
                insight: format!(
                    "Execution {} with fitness delta {:.3}",
                    if success { "succeeded" } else { "failed" },
                    fitness_delta
                ),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            task_skill.learning_history.push(event);
        }

        // Update step skills fitness
        for step_id in &record.step_skill_ids {
            if let Some(step_skill) = self
                .library
                .step_skills
                .iter_mut()
                .find(|s| s.id == *step_id)
            {
                let old_fitness = step_skill.fitness;
                step_skill.fitness = (step_skill.fitness + fitness_delta).max(0.0).min(1.0);

                let event = LearningEvent {
                    id: format!("learn_{}", uuid::Uuid::new_v4()),
                    fitness_before: old_fitness,
                    fitness_after: step_skill.fitness,
                    context: feedback.to_string(),
                    insight: format!("Step skill fitness updated"),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                step_skill.learning_history.push(event);
            }
        }

        Ok(())
    }

    /// Register a new task skill
    pub fn register_task_skill(&mut self, skill: TaskSkill) -> Result<(), SkillAgentError> {
        if self.library.task_skills.len() + self.library.step_skills.len()
            >= self.config.max_library_size
        {
            return Err(SkillAgentError::LibraryFull);
        }

        // Track composition
        self.library
            .composition_map
            .insert(skill.id.clone(), skill.step_sequence.clone());

        self.library.task_skills.push(skill);
        Ok(())
    }

    /// Register a new step skill
    pub fn register_step_skill(&mut self, skill: StepSkill) -> Result<(), SkillAgentError> {
        if self.library.task_skills.len() + self.library.step_skills.len()
            >= self.config.max_library_size
        {
            return Err(SkillAgentError::LibraryFull);
        }

        self.library.step_skills.push(skill);
        Ok(())
    }

    /// Remove low-fitness skills (pruning)
    pub fn prune(&mut self) -> usize {
        let before = self.library.task_skills.len() + self.library.step_skills.len();

        self.library.task_skills.retain(|s| {
            s.fitness >= self.config.min_fitness_threshold || !s.learning_history.is_empty()
        });

        self.library.step_skills.retain(|s| {
            s.fitness >= self.config.min_fitness_threshold || !s.learning_history.is_empty()
        });

        before - (self.library.task_skills.len() + self.library.step_skills.len())
    }

    /// Trigger co-evolution sync
    pub fn sync_with_reasoning(&mut self) -> Result<(), SkillAgentError> {
        // Analyze recent failures
        let recent_failures: Vec<&SkillExecutionRecord> = self
            .history
            .iter()
            .rev()
            .take(10)
            .filter(|r| !r.success)
            .collect();

        if recent_failures.is_empty() {
            return Ok(());
        }

        // Detect patterns
        let mut skill_failure_counts: HashMap<String, u32> = HashMap::new();
        for record in &recent_failures {
            *skill_failure_counts
                .entry(record.task_skill_id.clone())
                .or_default() += 1;
        }

        // Create adaptation suggestions
        for (skill_id, count) in &skill_failure_counts {
            if *count >= 3 {
                let rule = AdaptationRule {
                    id: format!("rule_{}", uuid::Uuid::new_v4()),
                    trigger: format!("skill {} failed {} times", skill_id, count),
                    action: "Reduce learning rate for this skill".to_string(),
                    priority: *count,
                    active: true,
                };
                self.coevolution.adaptation_rules.push(rule);
            }
        }

        // Record co-evolution event
        let event = CoevolutionEvent {
            id: format!("coevo_{}", uuid::Uuid::new_v4()),
            affected_skills: skill_failure_counts.keys().cloned().collect(),
            adaptation: "Failure pattern analysis".to_string(),
            fitness_delta: 0.0,
            sync_result: format!("Detected {} failure patterns", skill_failure_counts.len()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.coevolution.history.push(event);
        self.coevolution.reasoning_state.last_sync = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Get agent statistics
    pub fn stats(&self) -> SkillAgentStats {
        let total_task_skills = self.library.task_skills.len();
        let total_step_skills = self.library.step_skills.len();
        let total_executions = self.history.len();
        let successful_executions = self.history.iter().filter(|r| r.success).count();

        let avg_task_fitness = if total_task_skills > 0 {
            self.library
                .task_skills
                .iter()
                .map(|s| s.fitness)
                .sum::<f64>()
                / total_task_skills as f64
        } else {
            0.0
        };

        let avg_step_fitness = if total_step_skills > 0 {
            self.library
                .step_skills
                .iter()
                .map(|s| s.fitness)
                .sum::<f64>()
                / total_step_skills as f64
        } else {
            0.0
        };

        SkillAgentStats {
            total_task_skills,
            total_step_skills,
            total_executions,
            successful_executions,
            execution_success_rate: if total_executions > 0 {
                successful_executions as f64 / total_executions as f64
            } else {
                0.0
            },
            avg_task_skill_fitness: avg_task_fitness,
            avg_step_skill_fitness: avg_step_fitness,
            coevolution_events: self.coevolution.history.len(),
        }
    }
}

/// Skill agent statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAgentStats {
    pub total_task_skills: usize,
    pub total_step_skills: usize,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub execution_success_rate: f64,
    pub avg_task_skill_fitness: f64,
    pub avg_step_skill_fitness: f64,
    pub coevolution_events: usize,
}

impl std::fmt::Display for SkillAgentStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        Learnable Skill Agent Stats")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Task skills:      {}", self.total_task_skills)?;
        writeln!(f, "Step skills:      {}", self.total_step_skills)?;
        writeln!(f, "Executions:       {}", self.total_executions)?;
        writeln!(
            f,
            "Success rate:     {:.2}%",
            self.execution_success_rate * 100.0
        )?;
        writeln!(f, "Avg task fitness: {:.4}", self.avg_task_skill_fitness)?;
        writeln!(f, "Avg step fitness: {:.4}", self.avg_step_skill_fitness)?;
        writeln!(f, "Coevolution:      {}", self.coevolution_events)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_step_skill(name: &str) -> StepSkill {
        StepSkill {
            id: format!("step_{}", name),
            name: name.to_string(),
            description: format!("Step skill: {}", name),
            input_schema: vec!["input".to_string()],
            output_schema: vec!["output".to_string()],
            fitness: 0.8,
            parameters: HashMap::new(),
            learning_history: Vec::new(),
            version: 1,
        }
    }

    fn sample_task_skill(step_ids: Vec<String>) -> TaskSkill {
        TaskSkill {
            id: format!("task_{}", uuid::Uuid::new_v4()),
            name: "Test Task".to_string(),
            description: "A test task skill".to_string(),
            goal: "Complete the test".to_string(),
            step_sequence: step_ids,
            fitness: 0.7,
            learning_history: Vec::new(),
            version: 1,
            applicable_contexts: vec!["test".to_string()],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_agent_creation() {
        let agent = LearnableSkillAgent::new(SkillAgentConfig::default());
        assert_eq!(agent.library.task_skills.len(), 0);
        assert_eq!(agent.library.step_skills.len(), 0);
    }

    #[test]
    fn test_register_skills() {
        let mut agent = LearnableSkillAgent::new(SkillAgentConfig::default());

        let step = sample_step_skill("parse");
        agent.register_step_skill(step).unwrap();

        let task = sample_task_skill(vec!["step_parse".to_string()]);
        agent.register_task_skill(task).unwrap();

        assert_eq!(agent.library.step_skills.len(), 1);
        assert_eq!(agent.library.task_skills.len(), 1);
    }

    #[test]
    fn test_execute_task_skill() {
        let mut agent = LearnableSkillAgent::new(SkillAgentConfig::default());

        let step = StepSkill {
            id: "s1".to_string(),
            name: "Step1".to_string(),
            description: "First step".to_string(),
            input_schema: Vec::new(),
            output_schema: Vec::new(),
            fitness: 0.9,
            parameters: HashMap::new(),
            learning_history: Vec::new(),
            version: 1,
        };
        agent.register_step_skill(step).unwrap();

        let task = TaskSkill {
            id: "t1".to_string(),
            name: "Task1".to_string(),
            description: "First task".to_string(),
            goal: "Goal".to_string(),
            step_sequence: vec!["s1".to_string()],
            fitness: 0.8,
            learning_history: Vec::new(),
            version: 1,
            applicable_contexts: Vec::new(),
            metadata: HashMap::new(),
        };
        agent.register_task_skill(task).unwrap();

        let input = HashMap::from([("data".to_string(), "test".to_string())]);
        let result = agent.execute_task_skill("t1", input);

        assert!(result.is_ok());
        assert_eq!(agent.history.len(), 1);
    }

    #[test]
    fn test_learn_from_execution() {
        let mut agent = LearnableSkillAgent::new(SkillAgentConfig::default());

        let step = sample_step_skill("learn_step");
        agent.register_step_skill(step).unwrap();

        let task = sample_task_skill(vec!["step_learn_step".to_string()]);
        agent.register_task_skill(task).unwrap();

        let input = HashMap::new();
        let _ = agent.execute_task_skill(&agent.library.task_skills[0].id.clone(), input);

        let exec_id = agent.history[0].id.clone();
        let result = agent.learn_from_execution(&exec_id, true, "Good execution");

        assert!(result.is_ok());
    }

    #[test]
    fn test_prune() {
        let mut agent = LearnableSkillAgent::new(SkillAgentConfig::default());

        // Add low-fitness skill
        let low_skill = StepSkill {
            id: "low".to_string(),
            name: "Low".to_string(),
            description: "Low fitness".to_string(),
            input_schema: Vec::new(),
            output_schema: Vec::new(),
            fitness: 0.1,
            parameters: HashMap::new(),
            learning_history: Vec::new(),
            version: 1,
        };
        agent.register_step_skill(low_skill).unwrap();

        // Add high-fitness skill
        let high_skill = StepSkill {
            id: "high".to_string(),
            name: "High".to_string(),
            description: "High fitness".to_string(),
            input_schema: Vec::new(),
            output_schema: Vec::new(),
            fitness: 0.9,
            parameters: HashMap::new(),
            learning_history: Vec::new(),
            version: 1,
        };
        agent.register_step_skill(high_skill).unwrap();

        let pruned = agent.prune();
        assert_eq!(pruned, 1);
        assert_eq!(agent.library.step_skills.len(), 1);
    }

    #[test]
    fn test_sync_with_reasoning() {
        let mut agent = LearnableSkillAgent::new(SkillAgentConfig::default());

        // Add some failed executions
        for _ in 0..5 {
            agent.history.push(SkillExecutionRecord {
                id: format!("exec_{}", uuid::Uuid::new_v4()),
                task_skill_id: "t1".to_string(),
                step_skill_ids: Vec::new(),
                input: HashMap::new(),
                output: HashMap::new(),
                success: false,
                fitness_delta: -0.1,
                duration_ms: 50,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }

        let result = agent.sync_with_reasoning();
        assert!(result.is_ok());
        assert!(!agent.coevolution.adaptation_rules.is_empty());
    }

    #[test]
    fn test_stats() {
        let agent = LearnableSkillAgent::new(SkillAgentConfig::default());
        let stats = agent.stats();
        assert_eq!(stats.total_task_skills, 0);
        assert_eq!(stats.total_executions, 0);
    }
}
