use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::core::CrtTimeScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalScheduleStrategy {
    PriorityOnly,
    RoundRobin,
    MotivationDriven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl GoalPriority {
    pub fn rank(&self) -> u8 {
        match self {
            GoalPriority::Low => 0,
            GoalPriority::Medium => 1,
            GoalPriority::High => 2,
            GoalPriority::Critical => 3,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            GoalPriority::Low => "low",
            GoalPriority::Medium => "medium",
            GoalPriority::High => "high",
            GoalPriority::Critical => "critical",
        }
    }
}

impl PartialOrd for GoalPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GoalPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

pub struct RateLimiter {
    pub max_calls_per_hour: u64,
    pub call_timestamps: VecDeque<std::time::Instant>,
}

impl RateLimiter {
    pub fn new(max_calls_per_hour: u64) -> Self {
        Self {
            max_calls_per_hour,
            call_timestamps: VecDeque::new(),
        }
    }

    pub fn allow_call(&mut self) -> bool {
        let now = std::time::Instant::now();
        let one_hour = std::time::Duration::from_secs(3600);

        while let Some(&ts) = self.call_timestamps.front() {
            if now.duration_since(ts) > one_hour {
                self.call_timestamps.pop_front();
            } else {
                break;
            }
        }

        if self.call_timestamps.len() < self.max_calls_per_hour as usize {
            self.call_timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    pub fn reset() -> Self {
        Self::new(100)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    HalfOpen,
    Open,
}

pub struct CircuitBreaker {
    pub state: CircuitState,
    pub failure_count: u64,
    pub stall_count: u64,
    pub max_failures: u64,
    pub max_stalls: u64,
    pub cooldown_secs: u64,
    pub last_failure: Option<std::time::Instant>,
    pub last_stall_reason: Option<String>,
}

impl CircuitBreaker {
    pub fn new(max_failures: u64, max_stalls: u64, cooldown_secs: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            stall_count: 0,
            max_failures,
            max_stalls,
            cooldown_secs,
            last_failure: None,
            last_stall_reason: None,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.stall_count = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) -> bool {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());

        if self.failure_count >= self.max_failures || self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Open;
            true
        } else {
            false
        }
    }

    pub fn record_stall(&mut self) -> bool {
        self.stall_count += 1;
        self.last_failure = Some(std::time::Instant::now());

        if self.stall_count >= self.max_stalls || self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Open;
            true
        } else {
            false
        }
    }

    pub fn is_open(&self) -> bool {
        match self.state {
            CircuitState::Open => {
                if self.cooldown_secs > 0 {
                    if let Some(last) = self.last_failure {
                        let elapsed = std::time::Instant::now().duration_since(last);
                        if elapsed >= std::time::Duration::from_secs(self.cooldown_secs) {
                            return false;
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalState {
    Pursuing,
    Paused,
    Achieved,
    Unmet,
    BudgetLimited,
}

impl GoalState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, GoalState::Achieved | GoalState::Unmet | GoalState::BudgetLimited)
    }

    pub fn label(&self) -> &str {
        match self {
            GoalState::Pursuing => "pursuing",
            GoalState::Paused => "paused",
            GoalState::Achieved => "achieved",
            GoalState::Unmet => "unmet",
            GoalState::BudgetLimited => "budget_limited",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            GoalState::Pursuing => "\u{1f504}",
            GoalState::Paused => "\u{23f8}",
            GoalState::Achieved => "\u{2705}",
            GoalState::Unmet => "\u{274c}",
            GoalState::BudgetLimited => "\u{26a0}",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalConfig {
    pub max_iterations: u64,
    pub max_cost_usd: f64,
    pub max_duration_secs: u64,
    pub token_budget: u64,
    pub improvement_threshold: f64,
    pub completion_signal: String,
    pub stall_threshold: u64,
    pub max_calls_per_hour: u64,
    pub circuit_breaker_cooldown_secs: u64,
    /// CRT time scale for multi-scale temporal planning
    pub crt_scale: CrtTimeScale,
    /// Multi-goal scheduling strategy
    pub schedule_strategy: GoalScheduleStrategy,
    /// Enable E8 hexagram state based priority adjustment
    pub e8_priority_enabled: bool,
}

impl Default for GoalConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            max_cost_usd: 10.0,
            max_duration_secs: 3600,
            token_budget: 5_000_000,
            improvement_threshold: 0.01,
            completion_signal: "GOAL_COMPLETE".to_string(),
            stall_threshold: 5,
            max_calls_per_hour: 100,
            circuit_breaker_cooldown_secs: 1800,
            crt_scale: CrtTimeScale::Huntian,
            schedule_strategy: GoalScheduleStrategy::MotivationDriven,
            e8_priority_enabled: false,
        }
    }
}

impl GoalConfig {
    pub fn with_crt_scale(mut self, scale: CrtTimeScale) -> Self {
        self.crt_scale = scale;
        self
    }

    /// Get CRT-aware max iterations: CRT scale overrides if more specific.
    pub fn crt_adjusted_max_iterations(&self) -> u64 {
        let scale_iterations = self.crt_scale.max_iterations();
        scale_iterations.min(self.max_iterations)
    }

    /// Build a CrtPlan from this config.
    pub fn to_crt_plan(&self) -> crate::core::CrtPlan {
        crate::core::CrtPlan::new(self.crt_scale, self.max_duration_secs as f64)
    }

    pub fn crt_scale_label(&self) -> &str {
        self.crt_scale.label()
    }

    pub fn crt_chinese_name(&self) -> &str {
        self.crt_scale.chinese_name()
    }

    /// Adjust priority based on E8 hexagram hamming distance.
    /// dist 0-1: boost 1 level
    /// dist 2-3: keep same
    /// dist ≥4: drop 1 level
    pub fn e8_adjusted_priority(&self, base: GoalPriority, dist: u8) -> GoalPriority {
        match dist {
            0..=1 => match base {
                GoalPriority::Low => GoalPriority::Medium,
                GoalPriority::Medium => GoalPriority::High,
                GoalPriority::High => GoalPriority::Critical,
                GoalPriority::Critical => GoalPriority::Critical,
            },
            2..=3 => base,
            _ => match base {
                GoalPriority::Critical => GoalPriority::High,
                GoalPriority::High => GoalPriority::Medium,
                GoalPriority::Medium => GoalPriority::Low,
                GoalPriority::Low => GoalPriority::Low,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalIterationRecord {
    pub iteration: u64,
    pub score_before: f64,
    pub score_after: f64,
    pub reward: f64,
    pub improved: bool,
    pub cost_estimate: f64,
    pub tokens_used: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanLevel {
    Macro,
    Meso,
    Micro,
}

impl PlanLevel {
    pub fn label(&self) -> &str {
        match self {
            PlanLevel::Macro => "macro",
            PlanLevel::Meso => "meso",
            PlanLevel::Micro => "micro",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTemplate {
    pub level: PlanLevel,
    pub name: String,
    pub description: String,
    pub sub_plans: Vec<PlanTemplate>,
    pub skip_condition: Option<String>,
    pub reflection_trigger: Option<String>,
    pub expected_duration_cycles: usize,
    pub completion_criteria: Option<String>,
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
