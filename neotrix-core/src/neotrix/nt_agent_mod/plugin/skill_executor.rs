use crate::neotrix::nt_agent_mod::plugin::skill_manifest::SkillManifest;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    PermissionDenied,
}

#[derive(Debug, Clone)]
pub struct SkillExecution {
    pub skill_name: String,
    pub trigger_word: String,
    pub input: String,
    pub output: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: ExecutionStatus,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct SkillExecutor {
    skills: Vec<SkillManifest>,
    execution_history: Vec<SkillExecution>,
    max_history: usize,
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

impl SkillExecutor {
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            execution_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn with_max_history(mut self, n: usize) -> Self {
        self.max_history = n;
        self
    }

    pub fn register_skill(&mut self, manifest: SkillManifest) {
        // Replace existing skill with same name
        if let Some(pos) = self.skills.iter().position(|s| s.name == manifest.name) {
            self.skills[pos] = manifest;
        } else {
            self.skills.push(manifest);
        }
    }

    pub fn unregister_skill(&mut self, name: &str) {
        self.skills.retain(|s| s.name != name);
    }

    pub fn find_matching_skills(&self, input: &str) -> Vec<&SkillManifest> {
        self.skills
            .iter()
            .filter(|s| s.matches_trigger(input))
            .collect()
    }

    pub fn execute(&mut self, skill_name: &str, input: &str) -> Result<String, String> {
        let skill = self
            .skills
            .iter()
            .find(|s| s.name == skill_name)
            .ok_or_else(|| format!("Skill '{}' not found", skill_name))?;

        let trigger = skill
            .trigger_words
            .iter()
            .find(|tw| input.to_lowercase().contains(&tw.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| skill_name.to_string());

        let started_at = now_epoch_ms();
        let execution = SkillExecution {
            skill_name: skill_name.to_string(),
            trigger_word: trigger,
            input: input.to_string(),
            output: None,
            started_at,
            completed_at: None,
            status: ExecutionStatus::Running,
            error: None,
        };

        self.execution_history.push(execution);
        if self.execution_history.len() > self.max_history {
            self.execution_history.remove(0);
        }

        // Record completion
        let completed_at = now_epoch_ms();
        if let Some(last) = self.execution_history.last_mut() {
            last.completed_at = Some(completed_at);
            last.status = ExecutionStatus::Completed;
            last.output = Some(format!(
                "{} — execution not yet implemented",
                skill.description
            ));
        }

        Ok(format!(
            "{} — execution not yet implemented",
            skill.description
        ))
    }

    pub fn execution_history(&self) -> &[SkillExecution] {
        &self.execution_history
    }

    pub fn recent_executions(&self, n: usize) -> Vec<&SkillExecution> {
        self.execution_history.iter().rev().take(n).collect()
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn active_triggers(&self) -> Vec<String> {
        let mut triggers: Vec<String> = self
            .skills
            .iter()
            .flat_map(|s| s.trigger_words.clone())
            .collect();
        triggers.sort();
        triggers.dedup();
        triggers
    }

    pub fn skills(&self) -> &[SkillManifest] {
        &self.skills
    }
}

impl Default for SkillExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill(name: &str, triggers: Vec<&str>) -> SkillManifest {
        SkillManifest {
            name: name.to_string(),
            description: format!("Skill {}", name),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: triggers.iter().map(|s| s.to_string()).collect(),
            tags: vec![],
            dependencies: vec![],
            permission_level:
                crate::neotrix::nt_mind::self_iterating::pipeline::PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        }
    }

    #[test]
    fn test_register_skill() {
        let mut exec = SkillExecutor::new();
        assert_eq!(exec.skill_count(), 0);
        exec.register_skill(sample_skill("deploy", vec!["deploy", "release"]));
        assert_eq!(exec.skill_count(), 1);
    }

    #[test]
    fn test_find_matching_skills() {
        let mut exec = SkillExecutor::new();
        exec.register_skill(sample_skill("deploy", vec!["deploy"]));
        exec.register_skill(sample_skill("build", vec!["build", "compile"]));

        let matches = exec.find_matching_skills("please deploy the app");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "deploy");
    }

    #[test]
    fn test_execute_records_history() {
        let mut exec = SkillExecutor::new();
        exec.register_skill(sample_skill("greet", vec!["hello"]));
        let result = exec.execute("greet", "hello world");
        assert!(result.is_ok());

        let history = exec.execution_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].skill_name, "greet");
        assert_eq!(history[0].status, ExecutionStatus::Completed);
    }

    #[test]
    fn test_execution_history_length() {
        let mut exec = SkillExecutor::new().with_max_history(3);
        exec.register_skill(sample_skill("s", vec!["a"]));
        for i in 0..5 {
            let _ = exec.execute("s", &format!("a {}", i));
        }
        assert_eq!(exec.execution_history().len(), 3);
    }

    #[test]
    fn test_unregister_skill() {
        let mut exec = SkillExecutor::new();
        exec.register_skill(sample_skill("tmp", vec!["temp"]));
        assert_eq!(exec.skill_count(), 1);
        exec.unregister_skill("tmp");
        assert_eq!(exec.skill_count(), 0);
    }

    #[test]
    fn test_active_triggers() {
        let mut exec = SkillExecutor::new();
        exec.register_skill(sample_skill("a", vec!["deploy", "release"]));
        exec.register_skill(sample_skill("b", vec!["deploy", "build"]));
        let mut triggers = exec.active_triggers();
        triggers.sort();
        assert_eq!(triggers, vec!["build", "deploy", "release"]);
    }

    #[test]
    fn test_execute_nonexistent_skill() {
        let mut exec = SkillExecutor::new();
        let result = exec.execute("nope", "hello");
        assert!(result.is_err());
    }
}
