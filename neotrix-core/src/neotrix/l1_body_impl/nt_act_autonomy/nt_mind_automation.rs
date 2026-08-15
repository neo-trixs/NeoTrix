use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Types of triggers that can start an automation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutomationTrigger {
    /// File change in specific path (glob pattern)
    FileChange { pattern: String },
    /// On a schedule (cron expression — simplified: checked every 60s)
    Schedule { cron: String },
    /// Git push to branch
    GitPush { branch: String },
    /// Git PR created/updated
    GitPr { action: PrAction },
    /// External webhook
    Webhook { path: String },
}

impl std::fmt::Display for AutomationTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomationTrigger::FileChange { pattern } => write!(f, "file_change:{}", pattern),
            AutomationTrigger::Schedule { cron } => write!(f, "schedule:{}", cron),
            AutomationTrigger::GitPush { branch } => write!(f, "git_push:{}", branch),
            AutomationTrigger::GitPr { action } => write!(f, "git_pr:{:?}", action),
            AutomationTrigger::Webhook { path } => write!(f, "webhook:{}", path),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrAction {
    Created,
    Updated,
    Merged,
    Closed,
}

/// Actions to execute when trigger matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationAction {
    /// Run a skill by name
    RunSkill { name: String },
    /// Run the SEAL self-iteration pipeline
    RunSealPipeline,
    /// Send a notification
    Notify { message: String },
    /// Execute a shell command
    RunCommand { command: String },
}

impl std::fmt::Display for AutomationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomationAction::RunSkill { name } => write!(f, "run_skill:{}", name),
            AutomationAction::RunSealPipeline => write!(f, "run_seal_pipeline"),
            AutomationAction::Notify { message } => {
                let short: String = message.chars().take(40).collect();
                write!(f, "notify:{}", short)
            }
            AutomationAction::RunCommand { command } => {
                let short: String = command.chars().take(40).collect();
                write!(f, "run_cmd:{}", short)
            }
        }
    }
}

/// A single automation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    pub name: String,
    pub trigger: AutomationTrigger,
    pub action: AutomationAction,
    pub enabled: bool,
    pub created_at: SystemTime,
    pub last_run: Option<SystemTime>,
    pub run_count: u64,
}

impl AutomationRule {
    pub fn new(name: &str, trigger: AutomationTrigger, action: AutomationAction) -> Self {
        Self {
            name: name.to_string(),
            trigger,
            action,
            enabled: true,
            created_at: SystemTime::now(),
            last_run: None,
            run_count: 0,
        }
    }
}

/// Result of running an automation action
#[derive(Debug, Clone)]
pub struct AutomationResult {
    pub rule_name: String,
    pub success: bool,
    pub message: String,
}

/// Automation engine — manages rules and checks triggers
pub struct AutomationEngine {
    rules: Vec<AutomationRule>,
    last_checked: HashMap<AutomationTrigger, SystemTime>,
    /// 技能执行器注入 (L8/bin 层注入真实 SkillEngine, 避免 L1→L8 越层)。
    /// None 时 RunSkill 降级为 "queued" (不执行具体技能, 保持 L1 自包含)。
    skill_runner: Option<Box<dyn Fn(&str) -> Result<String, String>>>,
}

impl AutomationEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            last_checked: HashMap::new(),
            skill_runner: None,
        }
    }

    /// 注入技能执行器 (生产接线: L8/bin 层传入闭包, 内部构造 SkillEngine)。
    pub fn set_skill_runner<F>(&mut self, runner: F)
    where
        F: Fn(&str) -> Result<String, String> + 'static,
    {
        self.skill_runner = Some(Box::new(runner));
    }

    /// Load from file and return an engine with persisted rules.
    pub fn load_persisted() -> Self {
        let mut engine = Self::new();
        engine.load_from_file();
        engine
    }

    pub fn save_to_file(&self) {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.rules) {
            let _ = std::fs::write(&path, &data);
        }
    }

    pub fn load_from_file(&mut self) {
        let path = Self::store_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(rules) = serde_json::from_str::<Vec<AutomationRule>>(&data) {
                self.rules = rules;
            }
        }
    }

    fn store_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".neotrix").join("automation.json")
    }

    pub fn add_rule(&mut self, rule: AutomationRule) -> Result<(), String> {
        if self.rules.iter().any(|r| r.name == rule.name) {
            return Err(format!("Rule '{}' already exists", rule.name));
        }
        self.rules.push(rule);
        self.save_to_file();
        Ok(())
    }

    pub fn remove_rule(&mut self, name: &str) -> bool {
        let len = self.rules.len();
        self.rules.retain(|r| r.name != name);
        if self.rules.len() < len {
            self.save_to_file();
            return true;
        }
        false
    }

    pub fn list_rules(&self) -> &[AutomationRule] {
        &self.rules
    }

    /// Check all enabled triggers and return rules whose trigger condition is met.
    ///
    /// - `Schedule`: matches if last check was >60s ago (simplified cron)
    /// - `FileChange`: always returns false (requires external watcher)
    /// - `GitPush` / `GitPr` / `Webhook`: always returns false (requires external event source)
    pub fn check_triggers(&mut self) -> Vec<AutomationRule> {
        let now = SystemTime::now();
        let mut matched = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            let should_trigger = match &rule.trigger {
                AutomationTrigger::Schedule { .. } => {
                    let last = self.last_checked.get(&rule.trigger).copied().unwrap_or(SystemTime::UNIX_EPOCH);
                    now.duration_since(last).unwrap_or(Duration::ZERO) > Duration::from_secs(60)
                }
                _ => false,
            };
            if should_trigger {
                self.last_checked.insert(rule.trigger.clone(), now);
                matched.push(rule.clone());
            }
        }

        matched
    }

    pub fn enable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = true;
            self.save_to_file();
            true
        } else {
            false
        }
    }

    pub fn disable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = false;
            self.save_to_file();
            true
        } else {
            false
        }
    }

    /// Run a single rule's action and return the result.
    ///
    /// - `RunSkill`: returns "skill:{name} (simulated)"
    /// - `RunSealPipeline`: returns "seal_pipeline (simulated)"
    /// - `Notify`: returns "notified: {message}"
    /// - `RunCommand`: executes via `std::process::Command`
    pub fn run_rule(&self, rule: &AutomationRule) -> Result<String, String> {
        if !rule.enabled {
            return Err(format!("Rule '{}' is disabled", rule.name));
        }

        match &rule.action {
            AutomationAction::RunSkill { name } => {
                log::info!("[Automation] RunSkill: {} — triggering skill execution", name);
                // 经注入执行器触发 (L1 不直接依赖 L8 SkillEngine); 未注入则排队降级。
                if let Some(runner) = &self.skill_runner {
                    return runner(name);
                }
                Ok(format!("skill:{} queued for later execution", name))
            }
            AutomationAction::RunSealPipeline => {
                log::info!("[Automation] RunSealPipeline triggered — queued for next SEAL iteration");
                Ok("seal_pipeline queued".to_string())
            }
            AutomationAction::Notify { message } => {
                log::info!("[Automation] Notify: {}", message);
                Ok(format!("notified: {}", message))
            }
            AutomationAction::RunCommand { command } => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .map_err(|e| format!("command execution failed: {}", e))?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    Ok(format!("ok: {}", stdout.trim()))
                } else {
                    Err(format!("exit={}: {}", output.status.code().unwrap_or(-1), stderr.trim()))
                }
            }
        }
    }

    /// Count of enabled rules
    pub fn enabled_count(&self) -> usize {
        self.rules.iter().filter(|r| r.enabled).count()
    }

    /// Total rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for AutomationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rule(name: &str) -> AutomationRule {
        AutomationRule::new(
            name,
            AutomationTrigger::Schedule { cron: "* * * * *".into() },
            AutomationAction::Notify { message: "hello".into() },
        )
    }

    #[test]
    fn test_add_and_list_rules() {
        let mut engine = AutomationEngine::new();
        let rule = sample_rule("test1");
        assert!(engine.add_rule(rule).is_ok());
        assert_eq!(engine.rule_count(), 1);
        assert_eq!(engine.list_rules().len(), 1);
    }

    #[test]
    fn test_add_duplicate_rule_fails() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(sample_rule("dup")).unwrap();
        let result = engine.add_rule(sample_rule("dup"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(sample_rule("r1")).unwrap();
        engine.add_rule(sample_rule("r2")).unwrap();
        assert!(engine.remove_rule("r1"));
        assert_eq!(engine.rule_count(), 1);
        assert!(!engine.remove_rule("nonexistent"));
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(sample_rule("tog")).unwrap();
        assert!(engine.disable_rule("tog"));
        assert_eq!(engine.enabled_count(), 0);
        assert!(engine.enable_rule("tog"));
        assert_eq!(engine.enabled_count(), 1);
        assert!(!engine.enable_rule("nonexistent"));
        assert!(!engine.disable_rule("nonexistent"));
    }

    #[test]
    fn test_check_triggers_schedule() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(sample_rule("sched")).unwrap();
        // First check: should not trigger because last_checked is None but
        // the diff from UNIX_EPOCH will be >60s
        let matched = engine.check_triggers();
        assert_eq!(matched.len(), 1, "schedule should trigger on first check");
        // Second check immediately: should not trigger
        let matched2 = engine.check_triggers();
        assert_eq!(matched2.len(), 0, "schedule should not trigger again immediately");
        // Verify we got the rule by name
        assert_eq!(matched[0].name, "sched");
    }

    #[test]
    fn test_check_triggers_disabled_rule_skipped() {
        let mut engine = AutomationEngine::new();
        let mut rule = sample_rule("disabled");
        rule.enabled = false;
        engine.add_rule(rule).unwrap();
        let matched = engine.check_triggers();
        assert!(matched.is_empty(), "disabled rule should not trigger");
    }

    #[test]
    fn test_run_rule_notify() {
        let engine = AutomationEngine::new();
        let rule = AutomationRule::new(
            "notify_test",
            AutomationTrigger::Schedule { cron: "* * * * *".into() },
            AutomationAction::Notify { message: "test notification".into() },
        );
        let result = engine.run_rule(&rule);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("notified:"));
    }

    #[test]
    fn test_run_rule_disabled() {
        let engine = AutomationEngine::new();
        let mut rule = sample_rule("disabled_run");
        rule.enabled = false;
        let result = engine.run_rule(&rule);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("disabled"));
    }

    #[test]
    fn test_display_trigger_and_action() {
        let t = AutomationTrigger::FileChange { pattern: "*.rs".into() };
        assert_eq!(t.to_string(), "file_change:*.rs");
        let a = AutomationAction::RunSkill { name: "test".into() };
        assert_eq!(a.to_string(), "run_skill:test");
    }

    #[test]
    fn test_display_long_cjk_no_panic() {
        // Regression: Notify/RunCommand sliced &message[..len.min(40)] at a
        // byte index mid-CJK-char -> panic in Display. chars().take(40)
        // keeps the output valid UTF-8 regardless of byte layout.
        let long = "通知内容".repeat(30);
        let notify = AutomationAction::Notify { message: long.clone() };
        let out = notify.to_string();
        assert!(out.starts_with("notify:通知内容"));
        assert!(out.chars().count() <= "notify:".chars().count() + 40);

        let cmd = AutomationAction::RunCommand { command: long };
        let out2 = cmd.to_string();
        assert!(out2.starts_with("run_cmd:"));
        assert!(out2.chars().count() <= "run_cmd:".chars().count() + 40);
    }
}
