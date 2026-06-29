use log;
use std::path::PathBuf;

use crate::agent::AgentTeam;
use crate::core::nt_core_gwt::resonance::OscillatorNetwork;
use crate::core::nt_core_self::MotivationState;
use crate::core::{optimal_starting_mode, CrtTimeScale, ReasoningHexagram};
use crate::neotrix::nt_act_orchestrator::Orchestrator;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::goal_loop::tracker::GoalTracker;
use crate::neotrix::nt_mind::goal_loop::types::{
    CircuitBreaker, GoalConfig, GoalLoopState, GoalPriority, GoalScheduleStrategy, PlanTemplate,
    RateLimiter,
};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind_distiller::SessionDistiller;

const MAX_COMPLETED_GOALS: usize = 1000;

fn state_icon(state: &GoalLoopState) -> &str {
    match state {
        GoalLoopState::Pursuing => "\u{1f504}",
        GoalLoopState::Paused => "\u{23f8}",
        GoalLoopState::Achieved => "\u{2705}",
        GoalLoopState::Unmet => "\u{274c}",
        GoalLoopState::BudgetLimited => "\u{26a0}",
    }
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{:width$}", s, width = max)
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

pub struct GoalLoop {
    pub active_goal: Option<GoalTracker>,
    pub completed_goals: Vec<GoalTracker>,
    persistence_path: PathBuf,
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
}

impl Default for GoalLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalLoop {
    pub fn new() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("goals.json");
        Self {
            active_goal: None,
            completed_goals: Vec::new(),
            persistence_path: path,
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
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            active_goal: None,
            completed_goals: Vec::new(),
            persistence_path: path,
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
                if mot.should_explore && !goal.description.contains("explore") {
                    goal.priority = GoalPriority::High;
                }
                if mot.confidence < 0.4 && mot.error_rate > 0.2 {
                    goal.priority = GoalPriority::Critical;
                }
            }
        }
    }

    pub fn enqueue_goal(
        &mut self,
        brain: &mut SelfIteratingBrain,
        description: &str,
        config: Option<GoalConfig>,
    ) -> usize {
        if self.goal_queue.len() >= self.max_queue {
            return self.goal_queue.len();
        }
        if self.goal_queue.iter().any(|g| g.description == description) {
            return self.goal_queue.len();
        }
        if self
            .active_goal
            .as_ref()
            .map(|g| g.description == description)
            .unwrap_or(false)
        {
            return self.goal_queue.len();
        }
        let id = uuid::Uuid::new_v4().to_string();
        let cfg = config.unwrap_or_default();
        let mut tracker = GoalTracker::new(id, description.to_string(), cfg);
        tracker.score_before = brain.brain.evaluate_capability(TaskType::General);
        tracker.score_current = tracker.score_before;
        self.goal_queue.push(tracker);
        self.goal_queue
            .sort_by_key(|b| std::cmp::Reverse(b.priority));
        self.goal_queue.len()
    }

    pub fn dequeue_next(&mut self) -> Option<GoalTracker> {
        if self.goal_queue.is_empty() {
            return None;
        }
        self.goal_queue
            .sort_by_key(|b| std::cmp::Reverse(b.priority));
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
                    if goal.description.contains("debug")
                        || goal.description.contains("investigate")
                    {
                        goal.priority = GoalPriority::Critical;
                    }
                }
            }
            if mot.confidence < 0.4 && mot.error_rate > 0.2 {
                for goal in &mut self.goal_queue {
                    if goal.description.contains("validate")
                        || goal.description.contains("reinforce")
                    {
                        goal.priority = GoalPriority::Critical;
                    }
                }
            }
        }
        self.goal_queue
            .sort_by_key(|b| std::cmp::Reverse(b.priority));
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
        self.goal_queue
            .sort_by_key(|b| std::cmp::Reverse(b.priority));
    }

    pub fn start_goal(
        &mut self,
        brain: &mut SelfIteratingBrain,
        description: &str,
        config: Option<GoalConfig>,
    ) -> &GoalTracker {
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
            tracker.state = GoalLoopState::Achieved;
            tracker.updated_at = chrono::Utc::now().to_rfc3339();
            self.completed_goals.push(tracker);
            self.prune_completed_goals();
        }
    }

    pub fn pause_goal(&mut self) {
        if let Some(ref mut tracker) = self.active_goal {
            if tracker.state == GoalLoopState::Pursuing {
                tracker.state = GoalLoopState::Paused;
                tracker.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
    }

    pub fn resume_goal(&mut self) {
        if let Some(ref mut tracker) = self.active_goal {
            if tracker.state == GoalLoopState::Paused {
                tracker.state = GoalLoopState::Pursuing;
                tracker.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
    }

    pub fn clear_goal(&mut self) {
        if let Some(mut tracker) = self.active_goal.take() {
            tracker.state = GoalLoopState::Unmet;
            tracker.updated_at = chrono::Utc::now().to_rfc3339();
            self.completed_goals.push(tracker);
            self.prune_completed_goals();
        }
    }

    /// Prune oldest entries when completed_goals exceeds MAX_COMPLETED_GOALS.
    fn prune_completed_goals(&mut self) {
        if self.completed_goals.len() > MAX_COMPLETED_GOALS {
            let excess = self.completed_goals.len() - MAX_COMPLETED_GOALS;
            self.completed_goals.drain(0..excess);
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
                    format!(
                        "{}h {}m",
                        g.elapsed_secs() / 3600,
                        (g.elapsed_secs() % 3600) / 60
                    )
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
                    state_icon(&g.state),
                    g.state.label(),
                    g.iterations_completed,
                    g.config.max_iterations,
                    if g.config.max_iterations > 0 {
                        (g.iterations_completed as f64 / g.config.max_iterations as f64) * 100.0
                    } else {
                        0.0
                    },
                    g.total_cost_estimate,
                    g.config.max_cost_usd,
                    elapsed_human,
                    g.config.max_duration_secs,
                    g.score_before,
                    g.score_current,
                    g.score_current - g.score_before,
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
        let mut lines = vec![format!(
            "\nQueue: {} goals (max {})",
            self.goal_queue.len(),
            self.max_queue
        )];
        for (i, g) in self.goal_queue.iter().enumerate().take(3) {
            lines.push(format!(
                "  {}. [{}] {}",
                i + 1,
                g.priority.label(),
                truncate(&g.description, 40)
            ));
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
            lines.push(format!(
                "│ {}. {} {} {:25} │",
                i + 1,
                icon,
                truncate(&g.description, 28),
                g.state.label()
            ));
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
        let mut distiller = SessionDistiller::new();
        let report = distiller.generate_distillation_report();
        if !report.suggestions.is_empty() {
            log::info!(
                "[goal] 🧠 distilled {} patterns from {} sessions",
                report.patterns.len(),
                report.session_count
            );
            for s in &report.suggestions {
                log::info!("[goal]   → {}", s);
            }
        }
        report.suggestions
    }

    pub fn save(&self) -> NeoTrixResult<()> {
        if let Some(parent) = self.persistence_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| NeoTrixError::General {
                msg: e.to_string(),
                backtrace: None,
            })?;
        }
        let data = serde_json::json!({
            "active_goal": self.active_goal,
            "completed_goals": self.completed_goals,
            "goal_queue": self.goal_queue,
        });
        let json = serde_json::to_string_pretty(&data).map_err(|e| NeoTrixError::General {
            msg: e.to_string(),
            backtrace: None,
        })?;
        let tmp = self.persistence_path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| NeoTrixError::General {
            msg: e.to_string(),
            backtrace: None,
        })?;
        std::fs::rename(&tmp, &self.persistence_path).map_err(|e| NeoTrixError::General {
            msg: e.to_string(),
            backtrace: None,
        })?;
        Ok(())
    }

    pub fn load(&mut self) {
        if self.persistence_path.exists() {
            if let Ok(json) = std::fs::read_to_string(&self.persistence_path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json) {
                    self.active_goal = data["active_goal"].as_object().and_then(|_| {
                        serde_json::from_value(data["active_goal"].clone())
                            .inspect_err(|e| log::warn!("[goal-loop] parse active_goal: {}", e))
                            .ok()
                    });
                    self.completed_goals = data["completed_goals"]
                        .as_array()
                        .and_then(|_| {
                            serde_json::from_value(data["completed_goals"].clone())
                                .inspect_err(|e| {
                                    log::warn!("[goal-loop] parse completed_goals: {}", e)
                                })
                                .ok()
                        })
                        .unwrap_or_default();
                    self.goal_queue = data["goal_queue"]
                        .as_array()
                        .and_then(|_| {
                            serde_json::from_value(data["goal_queue"].clone())
                                .inspect_err(|e| log::warn!("[goal-loop] parse goal_queue: {}", e))
                                .ok()
                        })
                        .unwrap_or_default();
                    let restored = self.completed_goals.len();
                    let queued = self.goal_queue.len();
                    if restored > 0 {
                        log::info!(
                            "[bg-goal] restored {} completed goals from persistence",
                            restored
                        );
                    }
                    if queued > 0 {
                        log::info!("[bg-goal] restored {} queued goals", queued);
                    }
                    if let Some(ref g) = self.active_goal {
                        if g.state == GoalLoopState::Pursuing {
                            log::info!(
                                "[bg-goal] restored pursuing goal: {}",
                                truncate(&g.description, 40)
                            );
                        }
                    }
                }
            }
        }
    }

    /// Attach a LoopTemplate by ID, copying its exit conditions, max_iterations, and goal.
    pub fn attach_loop_template(&mut self, template_id: &str) -> bool {
        let templates = crate::core::nt_core_experience::loop_templates::default_templates();
        if let Some(template) = templates.into_iter().find(|t| t.id == template_id) {
            if let Some(ref mut tracker) = self.active_goal {
                tracker.loop_template_id = Some(template.id);
                tracker.template_exit_conditions = template.exit_conditions;
                tracker.config.max_iterations = template.max_iterations as u64;
                return true;
            }
        }
        false
    }

    /// Initialize oscillator network for resonance-based goal selection.
    pub fn init_oscillators(&mut self, num_goals: usize) {
        self.oscillator_network = Some(OscillatorNetwork::new(num_goals));
    }

    /// Compute resonance coherence — how well current goals synchronize.
    /// Returns a value in [0.0, 1.0], or 0.5 if no oscillator network is initialized.
    pub fn resonance_coherence(&self) -> f64 {
        self.oscillator_network
            .as_ref()
            .map(|osc| osc.phase_coherence())
            .unwrap_or(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::truncate;
    use super::GoalLoop;
    use crate::core::nt_core_self::MotivationState;

    #[test]
    fn test_resonance_coherence_default() {
        let gl = GoalLoop::new();
        let coherence = gl.resonance_coherence();
        assert!(coherence >= 0.0 && coherence <= 1.0);
        assert!(
            (coherence - 0.5).abs() < 1e-6,
            "no oscillator network should yield 0.5"
        );
    }

    #[test]
    fn test_resonance_coherence_after_init() {
        let mut gl = GoalLoop::new();
        gl.init_oscillators(5);
        let coherence = gl.resonance_coherence();
        assert!(
            coherence >= 0.0 && coherence <= 1.0,
            "coherence should be in [0,1], got {}",
            coherence
        );
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
}
