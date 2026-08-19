use std::path::PathBuf;

use super::super::types::{RateLimiter, CircuitBreaker, GoalState, GoalConfig, GoalPriority, GoalScheduleStrategy, PlanTemplate};
use super::super::tracker::GoalTracker;
use super::super::super::self_iterating::SelfIteratingBrain;
use crate::core::{CrtTimeScale, ReasoningHexagram, optimal_starting_mode};
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_act_orchestrator::Orchestrator;
use crate::agent::AgentTeam;
use crate::core::nt_core_self::MotivationState;
use crate::core::nt_core_gwt::resonance::OscillatorNetwork;
use crate::neotrix::nt_mind_distiller::{CommandDistiller, DistilledOutput, SessionDistiller};
use crate::neotrix::nt_core_error::{NeoTrixResult, NeoTrixError};

fn state_icon(state: &GoalState) -> &str {
    match state {
        GoalState::Pursuing => "\u{1f504}",
        GoalState::Paused => "\u{23f8}",
        GoalState::Achieved => "\u{2705}",
        GoalState::Unmet => "\u{274c}",
        GoalState::BudgetLimited => "\u{26a0}",
    }
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{:width$}", s, width = max)
    } else {
        format!("{}...", s.chars().take(max.saturating_sub(3)).collect::<String>())
    }
}

pub struct GoalLoop {
    pub active_goal: Option<GoalTracker>,
    pub completed_goals: Vec<GoalTracker>,
    pub rate_limiter: RateLimiter,
    pub circuit_breaker: CircuitBreaker,
    pub orchestrator: Option<Orchestrator>,
    pub agent_team: Option<std::sync::Arc<std::sync::Mutex<AgentTeam>>>,
    pub motivation_hint: Option<MotivationState>,
    pub goal_queue: Vec<GoalTracker>,
    pub max_queue: usize,
    pub active_plan: Option<PlanTemplate>,
    pub plan_stack: Vec<PlanTemplate>,
    pub oscillator_network: Option<OscillatorNetwork>,
    /// 最近一次可逆蒸馏输出 — CommandDistiller 生产接线 (R-P79):
    /// run_distillation 蒸馏原始会话日志源后保留, 供 expand_last_distillation 还原。
    pub last_distilled_output: Option<DistilledOutput>,
    /// 最近一次可逆蒸馏落盘目录 — expand 时按同一 artifact_dir 还原。
    last_distill_dir: Option<PathBuf>,
}

impl Default for GoalLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalLoop {
    pub fn new() -> Self {
        Self {
            active_goal: None,
            completed_goals: Vec::new(),
            rate_limiter: RateLimiter::new(100),
            circuit_breaker: CircuitBreaker::new(3, 3, 1800),
            orchestrator: None,
            agent_team: None,
            motivation_hint: None,
            goal_queue: Vec::new(),
            max_queue: 5,
            active_plan: None,
            plan_stack: Vec::new(),
            oscillator_network: None,
            last_distilled_output: None,
            last_distill_dir: None,
        }
    }

    pub fn with_path(_path: PathBuf) -> Self {
        Self {
            active_goal: None,
            completed_goals: Vec::new(),
            rate_limiter: RateLimiter::new(100),
            circuit_breaker: CircuitBreaker::new(3, 3, 1800),
            orchestrator: None,
            agent_team: None,
            motivation_hint: None,
            goal_queue: Vec::new(),
            max_queue: 5,
            active_plan: None,
            plan_stack: Vec::new(),
            oscillator_network: None,
            last_distilled_output: None,
            last_distill_dir: None,
        }
    }

    pub fn with_orchestrator(mut self, orch: Orchestrator) -> Self {
        self.orchestrator = Some(orch);
        self
    }

    pub fn with_agent_team(mut self, team: std::sync::Arc<std::sync::Mutex<AgentTeam>>) -> Self {
        self.agent_team = Some(team);
        self
    }

    pub fn set_motivation(&mut self, state: MotivationState) {
        self.motivation_hint = Some(state);
    }

    pub fn prioritize_from_motivation(&mut self) {
        if let Some(ref mot) = self.motivation_hint {
            if let Some(ref mut goal) = self.active_goal {
                if mot.error_rate > 0.3 && !goal.description.contains("debug") {
                    goal.priority = GoalPriority::High;
                }
                if mot.should_explore
                    && !goal.description.contains("explore") {
                        goal.priority = GoalPriority::High;
                    }
                if mot.confidence < 0.4 && mot.error_rate > 0.2 {
                    goal.priority = GoalPriority::Critical;
                }
            }
        }
    }

    pub fn enqueue_goal(&mut self, brain: &mut SelfIteratingBrain, description: &str, config: Option<GoalConfig>) -> usize {
        if self.goal_queue.len() >= self.max_queue {
            return self.goal_queue.len();
        }
        if self.goal_queue.iter().any(|g| g.description == description) {
            return self.goal_queue.len();
        }
        if self.active_goal.as_ref().map(|g| g.description == description).unwrap_or(false) {
            return self.goal_queue.len();
        }
        let id = uuid::Uuid::new_v4().to_string();
        let cfg = config.unwrap_or_default();
        let mut tracker = GoalTracker::new(id, description.to_string(), cfg);
        tracker.score_before = brain.brain.evaluate_capability(TaskType::General);
        tracker.score_current = tracker.score_before;
        self.goal_queue.push(tracker);
        self.goal_queue.sort_by_key(|b| std::cmp::Reverse(b.priority));
        self.goal_queue.len()
    }

    pub fn dequeue_next(&mut self) -> Option<GoalTracker> {
        if self.goal_queue.is_empty() {
            return None;
        }
        self.goal_queue.sort_by_key(|b| std::cmp::Reverse(b.priority));
        Some(self.goal_queue.remove(0))
    }

    pub fn rebalance_from_motivation(&mut self) {
        if let Some(ref mot) = self.motivation_hint {
            if mot.should_explore {
                for goal in &mut self.goal_queue {
                    if goal.description.contains("explore") {
                        goal.priority = GoalPriority::High;
                    }
                }
            }
            if mot.error_rate > 0.3 {
                for goal in &mut self.goal_queue {
                    if goal.description.contains("debug") || goal.description.contains("investigate") {
                        goal.priority = GoalPriority::Critical;
                    }
                }
            }
            if mot.confidence < 0.4 && mot.error_rate > 0.2 {
                for goal in &mut self.goal_queue {
                    if goal.description.contains("validate") || goal.description.contains("reinforce") {
                        goal.priority = GoalPriority::Critical;
                    }
                }
            }
        }
        self.goal_queue.sort_by_key(|b| std::cmp::Reverse(b.priority));
    }

    pub fn apply_e8_priority(&mut self, hexagram: ReasoningHexagram) {
        if let Some(ref mut goal) = self.active_goal {
            let ideal = optimal_starting_mode(&goal.description);
            let dist = hexagram.hamming_dist(&ideal) as u8;
            goal.priority = goal.config.e8_adjusted_priority(goal.priority, dist);
        }
        for goal in &mut self.goal_queue {
            let ideal = optimal_starting_mode(&goal.description);
            let dist = hexagram.hamming_dist(&ideal) as u8;
            goal.priority = goal.config.e8_adjusted_priority(goal.priority, dist);
        }
        self.goal_queue.sort_by_key(|b| std::cmp::Reverse(b.priority));
    }

    pub fn start_goal(&mut self, brain: &mut SelfIteratingBrain, description: &str, config: Option<GoalConfig>) -> &GoalTracker {
        let id = uuid::Uuid::new_v4().to_string();
        let cfg = config.unwrap_or_default();
        let score_before = brain.brain.evaluate_capability(TaskType::General);
        let mut tracker = GoalTracker::new(id, description.to_string(), cfg);
        tracker.score_before = score_before;
        tracker.score_current = score_before;
        self.active_goal = Some(tracker);
        self.active_goal.as_ref().expect("active_goal set above")
    }

    pub fn achieve_goal(&mut self) {
        if let Some(mut tracker) = self.active_goal.take() {
            tracker.state = GoalState::Achieved;
            tracker.updated_at = chrono::Utc::now().to_rfc3339();
            self.completed_goals.push(tracker);
            self.completed_goals.truncate(100);
        }
    }

    pub fn pause_goal(&mut self) {
        if let Some(ref mut tracker) = self.active_goal {
            if tracker.state == GoalState::Pursuing {
                tracker.state = GoalState::Paused;
                tracker.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
    }

    pub fn resume_goal(&mut self) {
        if let Some(ref mut tracker) = self.active_goal {
            if tracker.state == GoalState::Paused {
                tracker.state = GoalState::Pursuing;
                tracker.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
    }

    pub fn clear_goal(&mut self) {
        if let Some(mut tracker) = self.active_goal.take() {
            tracker.state = GoalState::Unmet;
            tracker.updated_at = chrono::Utc::now().to_rfc3339();
            self.completed_goals.push(tracker);
            self.completed_goals.truncate(100);
        }
    }

    pub fn status(&self) -> String {
        let main = match &self.active_goal {
            Some(g) => {
                let elapsed_human = if g.elapsed_secs() < 60 {
                    format!("{}s", g.elapsed_secs())
                } else if g.elapsed_secs() < 3600 {
                    format!("{}m {}s", g.elapsed_secs() / 60, g.elapsed_secs() % 60)
                } else {
                    format!("{}h {}m", g.elapsed_secs() / 3600, (g.elapsed_secs() % 3600) / 60)
                };
                format!(
                    "╭─ /goal status ─────────────────────────────╮\n\
                     │ Goal:       {}                       │\n\
                     │ State:      {} {:14}            │\n\
                     │ Iterations: {}/{} ({:.1}%)              │\n\
                     │ Cost:       ${:.4}/{:.2}                    │\n\
                     │ Duration:   {}/{}s                 │\n\
                     │ Score:      {:.3} → {:.3} ({:+.3})        │\n\
                     │ Stalled:    {}x                            │\n\
                     ╰─────────────────────────────────────────────╯",
                    truncate(&g.description, 28),
                    state_icon(&g.state), g.state.label(),
                    g.iterations_completed, g.config.max_iterations,
                    if g.config.max_iterations > 0 { (g.iterations_completed as f64 / g.config.max_iterations as f64) * 100.0 } else { 0.0 },
                    g.total_cost_estimate, g.config.max_cost_usd,
                    elapsed_human, g.config.max_duration_secs,
                    g.score_before, g.score_current, g.score_current - g.score_before,
                    g.stalled_count,
                )
            }
            None => "No active goal. Use /goal <description> to start one.".to_string(),
        };
        format!("{}{}", main, self.queue_summary())
    }

    fn queue_summary(&self) -> String {
        if self.goal_queue.is_empty() {
            return String::new();
        }
        let mut lines = vec![format!("\nQueue: {} goals (max {})", self.goal_queue.len(), self.max_queue)];
        for (i, g) in self.goal_queue.iter().enumerate().take(3) {
            lines.push(format!("  {}. [{}] {}", i + 1, g.priority.label(), truncate(&g.description, 40)));
        }
        if self.goal_queue.len() > 3 {
            lines.push(format!("  ... and {} more", self.goal_queue.len() - 3));
        }
        lines.join("\n")
    }

    pub fn history_summary(&self) -> String {
        if self.completed_goals.is_empty() {
            return "No completed goals.".to_string();
        }
        let mut lines = vec!["╭─ Goal History ────────────────────────────╮".to_string()];
        for (i, g) in self.completed_goals.iter().rev().enumerate().take(10) {
            let icon = state_icon(&g.state);
            lines.push(format!("│ {}. {} {} {:25} │",
                i + 1, icon, truncate(&g.description, 28), g.state.label()));
        }
        lines.push("╰─────────────────────────────────────────────╯".to_string());
        lines.join("\n")
    }

    pub fn auto_goal_config() -> GoalConfig {
        GoalConfig {
            max_iterations: 20,
            max_cost_usd: 2.0,
            max_duration_secs: 1800,
            token_budget: 1_000_000,
            improvement_threshold: 0.005,
            completion_signal: "AUTO_GOAL_COMPLETE".to_string(),
            stall_threshold: 3,
            max_calls_per_hour: 100,
            circuit_breaker_cooldown_secs: 1800,
            crt_scale: CrtTimeScale::Huntian,
            schedule_strategy: GoalScheduleStrategy::MotivationDriven,
            e8_priority_enabled: false,
        }
    }

    pub fn run_distillation(&mut self) -> Vec<String> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.run_distillation_with_paths(
            home.join(".neotrix").join("session-logs"),
            PathBuf::from("AGENTS.md"),
            home.join(".neotrix").join("distill-artifacts"),
        )
    }

    /// 带显式路径的可逆蒸馏 — 生产路径 (run_distillation, pursue.rs) 与
    /// 集成测试共用同一实现。CommandDistiller 直接蒸馏原始会话日志源
    /// (而非序列化报告, 报告不含错误行), 错误行优先保留并落盘 artifact;
    /// 记录 error_lines 最多的输出, 供 expand_last_distillation 还原。
    pub(crate) fn run_distillation_with_paths(
        &mut self,
        session_logs_dir: PathBuf,
        agents_path: PathBuf,
        artifact_dir: PathBuf,
    ) -> Vec<String> {
        let mut distiller = SessionDistiller::with_paths(session_logs_dir, agents_path);
        let report = distiller.generate_distillation_report();
        if !report.suggestions.is_empty() {
            println!("[goal] 🧠 distilled {} patterns from {} sessions", report.patterns.len(), report.session_count);
            for s in &report.suggestions {
                println!("[goal]   → {}", s);
            }
        }
        // 可逆蒸馏 (repowise absorb): 原始会话日志 → 错误先行压缩落盘 artifact, 可 expand 还原。
        let distilled = CommandDistiller::with_dir(artifact_dir);
        let distill_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut stored: Option<DistilledOutput> = None;
        for (name, content) in distiller.load_session_logs() {
            let id = format!("goal-distill-{}-{}", distill_id, name);
            match distilled.distill(&id, &content, 2048) {
                Ok(out) => {
                    println!(
                        "[goal] 📦 reversible distill '{}': {} chars → {} ({:.0}%), {} error lines, expand: {}",
                        name, out.total_chars, out.kept_chars, out.ratio * 100.0, out.error_lines, out.artifact_path
                    );
                    // 保留 error_lines 最多的输出 — 生产路径后续 expand 优先还原问题最多的会话。
                    if stored.as_ref().map(|s| out.error_lines > s.error_lines).unwrap_or(true) {
                        stored = Some(out);
                    }
                }
                Err(e) => log::warn!("[goal] reversible distill '{}' failed: {}", name, e),
            }
        }
        if let Some(out) = stored {
            self.last_distilled_output = Some(out);
            self.last_distill_dir = Some(distilled.artifact_dir);
        }
        report.suggestions
    }

    /// 生产路径 expand — 还原最近一次可逆蒸馏的原始会话日志内容。
    pub fn expand_last_distillation(&self) -> Result<String, String> {
        let out = self.last_distilled_output.as_ref().ok_or_else(|| "no distillation stored yet".to_string())?;
        let dir = self.last_distill_dir.clone().unwrap_or_else(|| CommandDistiller::new().artifact_dir);
        CommandDistiller::with_dir(dir).expand(&out.id)
    }

    pub fn save(&self) -> NeoTrixResult<()> {
        let data = serde_json::json!({
            "active_goal": self.active_goal,
            "completed_goals": self.completed_goals,
            "goal_queue": self.goal_queue,
        });
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        crate::core::nt_core_state::save("goals", &json)
            .map_err(|e| NeoTrixError::Io(e))
    }

    pub fn load(&mut self) {
        if let Some(json) = crate::core::nt_core_state::load("goals") {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json) {
                self.active_goal = data["active_goal"]
                    .as_object()
                    .and_then(|_| serde_json::from_value(data["active_goal"].clone()).inspect_err(|e| log::warn!("[goal-loop] parse active_goal: {}", e)).ok());
                self.completed_goals = data["completed_goals"]
                    .as_array()
                    .and_then(|_| serde_json::from_value(data["completed_goals"].clone()).inspect_err(|e| log::warn!("[goal-loop] parse completed_goals: {}", e)).ok())
                    .unwrap_or_default();
                self.goal_queue = data["goal_queue"]
                    .as_array()
                    .and_then(|_| serde_json::from_value(data["goal_queue"].clone()).inspect_err(|e| log::warn!("[goal-loop] parse goal_queue: {}", e)).ok())
                    .unwrap_or_default();
                let restored = self.completed_goals.len();
                let queued = self.goal_queue.len();
                if restored > 0 {
                    println!("[bg-goal] restored {} completed goals from persistence", restored);
                }
                if queued > 0 {
                    println!("[bg-goal] restored {} queued goals", queued);
                }
                if let Some(ref g) = self.active_goal {
                    if g.state == GoalState::Pursuing {
                        println!("[bg-goal] restored pursuing goal: {}", truncate(&g.description, 40));
                    }
                }
            }
        }
    }

    /// Initialize oscillator network for resonance-based goal selection.
    pub fn init_oscillators(&mut self, num_goals: usize) {
        self.oscillator_network = Some(OscillatorNetwork::new(num_goals));
    }

    /// Compute resonance coherence — how well current goals synchronize.
    /// Returns a value in [0.0, 1.0], or 0.5 if no oscillator network is initialized.
    pub fn resonance_coherence(&self) -> f64 {
        self.oscillator_network.as_ref()
            .map(|osc| osc.phase_coherence())
            .unwrap_or(0.5)
    }
}


#[cfg(test)]
mod tests {
    use super::GoalLoop;
    use super::truncate;
    use crate::core::nt_core_self::MotivationState;

    #[test]
    fn test_basic() {
        assert!(true);
    }

    #[test]
    fn test_resonance_coherence_default() {
        let gl = GoalLoop::new();
        let coherence = gl.resonance_coherence();
        assert!(coherence >= 0.0 && coherence <= 1.0);
        assert!((coherence - 0.5).abs() < 1e-6, "no oscillator network should yield 0.5");
    }

    #[test]
    fn test_resonance_coherence_after_init() {
        let mut gl = GoalLoop::new();
        gl.init_oscillators(5);
        let coherence = gl.resonance_coherence();
        assert!(coherence >= 0.0 && coherence <= 1.0,
            "coherence should be in [0,1], got {}", coherence);
    }

    #[test]
    fn test_truncate_short_string() {
        let result = truncate("hello", 10);
        assert_eq!(result, "hello     ");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("this is a very long string", 15);
        assert!(result.len() <= 15);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_exact_length() {
        let result = truncate("exact", 5);
        assert_eq!(result, "exact");
    }

    #[test]
    fn test_auto_goal_config_defaults() {
        let config = GoalLoop::auto_goal_config();
        assert_eq!(config.max_iterations, 20);
        assert!((config.max_cost_usd - 2.0).abs() < 1e-6);
        assert_eq!(config.max_duration_secs, 1800);
        assert_eq!(config.token_budget, 1_000_000);
        assert_eq!(config.completion_signal, "AUTO_GOAL_COMPLETE");
    }

    #[test]
    fn test_history_summary_empty() {
        let gl = GoalLoop::new();
        let summary = gl.history_summary();
        assert_eq!(summary, "No completed goals.");
    }

    #[test]
    fn test_prioritize_from_motivation_high_error() {
        let mut gl = GoalLoop::new();
        gl.set_motivation(MotivationState {
            intrinsic_reward: 0.1,
            confidence: 0.3,
            error_rate: 0.5,
            novelty_score: 0.1,
            should_explore: false,
            suggested_domains: vec![],
            suggested_strategies: vec![],
        });
        // Without an active goal, prioritize_from_motivation is a no-op
        gl.prioritize_from_motivation();
        assert!(gl.active_goal.is_none());
    }

    #[test]
    fn test_with_path_custom_location() {
        let path = std::env::temp_dir().join("neotrix_test_custom_goals.json");
        let gl = GoalLoop::with_path(path.clone());
        assert!(gl.active_goal.is_none());
        assert_eq!(gl.completed_goals.len(), 0);
        let _ = std::fs::remove_file(&path);
    }

    // 集成测试: CommandDistiller 在生产蒸馏路径中真实运行 —
    // 会话日志源流过 run_distillation_with_paths, error_lines 被保留,
    // DistilledOutput 落盘存储, expand_last_distillation 还原原始内容。
    #[test]
    fn test_run_distillation_wires_command_distiller_production_path() {
        use std::path::PathBuf;

        let tmp = tempfile::tempdir().unwrap();
        let logs_dir = tmp.path().join("session-logs");
        std::fs::create_dir_all(&logs_dir).unwrap();
        std::fs::write(
            logs_dir.join("2026-08-14.md"),
            "cargo check\n   Compiling x\nerror[E0277]: type mismatch\nwarning: unused\n   Finished\n",
        ).unwrap();
        let artifact_dir = tmp.path().join("distill-artifacts");

        let mut gl = GoalLoop::new();
        let suggestions = gl.run_distillation_with_paths(
            logs_dir.clone(),
            PathBuf::from("AGENTS.md"),
            artifact_dir.clone(),
        );
        assert!(suggestions.is_empty(), "no session patterns expected, got {:?}", suggestions);

        let out = gl.last_distilled_output
            .as_ref()
            .expect("run_distillation must store DistilledOutput for later expand");
        assert!(out.error_lines >= 1, "error lines from session log must be retained, got {}", out.error_lines);
        assert!(out.artifact_path.contains("distill-artifacts"), "artifact must land in configured dir: {}", out.artifact_path);

        let expanded = gl.expand_last_distillation()
            .expect("expand_last_distillation must restore original");
        assert!(expanded.contains("E0277"), "expand must restore error lines: {}", expanded);
        assert!(expanded.contains("cargo check"));
    }

    #[test]
    fn test_expand_last_distillation_without_distill_errors() {
        let gl = GoalLoop::new();
        assert!(gl.expand_last_distillation().is_err(), "no distillation stored yet → Err");
    }
}
