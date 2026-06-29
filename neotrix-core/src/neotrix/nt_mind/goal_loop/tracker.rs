use serde::{Deserialize, Serialize};

use super::types::{GoalConfig, GoalIterationRecord, GoalLoopState, GoalPriority};

const CONTINUATION_PROMPT: &str = r#"
## Goal Continuation Check
Assess whether the goal "{goal}" has been achieved:
- If ALL of the measurable acceptance criteria are met: signal GOAL_COMPLETE
- If partial progress was made but more work remains: describe what was done and what's next
- If blocked by an unresolvable issue: describe the blocker

Respond with one of:
//STATUS: ACHIEVED — all criteria met
//STATUS: IN_PROGRESS — progress made, continuing
//STATUS: BLOCKED — cannot proceed, describe why
"#;

const BUDGET_LIMIT_PROMPT: &str = r#"
## Budget Status
- Iterations used: {iterations}/{max_iterations}
- Estimated cost: ${cost}/{max_cost}
- Tokens consumed: {tokens}/{budget}

Stay focused on the highest-impact remaining task for "{goal}".
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTracker {
    pub id: String,
    pub description: String,
    pub state: GoalLoopState,
    pub config: GoalConfig,
    pub started_at: String,
    pub updated_at: String,
    pub iterations_completed: u64,
    pub total_cost_estimate: f64,
    pub tokens_consumed: u64,
    pub score_before: f64,
    pub score_current: f64,
    pub last_reward: f64,
    pub stalled_count: u64,
    pub priority: GoalPriority,
    pub history: Vec<GoalIterationRecord>,
    pub loop_template_id: Option<String>,
    pub template_exit_conditions: Vec<String>,
}

impl GoalTracker {
    pub fn new(id: String, description: String, config: GoalConfig) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            description,
            state: GoalLoopState::Pursuing,
            config,
            started_at: now.clone(),
            updated_at: now,
            iterations_completed: 0,
            total_cost_estimate: 0.0,
            tokens_consumed: 0,
            score_before: 0.0,
            score_current: 0.0,
            last_reward: 0.0,
            stalled_count: 0,
            priority: GoalPriority::Medium,
            history: Vec::new(),
            loop_template_id: None,
            template_exit_conditions: Vec::new(),
        }
    }

    pub fn elapsed_secs(&self) -> i64 {
        let start = match chrono::DateTime::parse_from_rfc3339(&self.started_at) {
            Ok(dt) => Some(dt),
            Err(e) => {
                log::warn!("[tracker] parse started_at '{}': {}", self.started_at, e);
                None
            }
        };
        let now = chrono::Utc::now();
        start
            .map(|s| (now - s.with_timezone(&chrono::Utc)).num_seconds())
            .unwrap_or(0)
    }

    pub fn budget_exhausted(&self) -> Option<GoalLoopState> {
        if self.iterations_completed >= self.config.max_iterations {
            return Some(GoalLoopState::BudgetLimited);
        }
        if self.total_cost_estimate >= self.config.max_cost_usd {
            return Some(GoalLoopState::BudgetLimited);
        }
        if self.elapsed_secs() as u64 >= self.config.max_duration_secs {
            return Some(GoalLoopState::BudgetLimited);
        }
        if self.tokens_consumed >= self.config.token_budget {
            return Some(GoalLoopState::BudgetLimited);
        }
        None
    }

    pub fn continuation_prompt(&self) -> String {
        CONTINUATION_PROMPT.replace("{goal}", &self.description)
    }

    pub fn budget_prompt(&self) -> String {
        BUDGET_LIMIT_PROMPT
            .replace("{iterations}", &self.iterations_completed.to_string())
            .replace("{max_iterations}", &self.config.max_iterations.to_string())
            .replace("{cost}", &format!("{:.4}", self.total_cost_estimate))
            .replace("{max_cost}", &format!("{:.2}", self.config.max_cost_usd))
            .replace("{tokens}", &self.tokens_consumed.to_string())
            .replace("{budget}", &self.config.token_budget.to_string())
            .replace("{goal}", &self.description)
    }
}
