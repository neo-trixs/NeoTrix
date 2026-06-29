use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_agent::lead_agent::{LeadAgent, LeadAgentConfig, PlanEffort};
use crate::core::nt_core_agent::preview::{PreviewEngine, PreviewResult};
use crate::core::nt_core_agent::sub_agent::{LeadAgentPlan, SubAgentResult};
use crate::neotrix::nt_mind::goal_loop::types::{GoalConfig, GoalLoopState, GoalPriority};

fn now_iso() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        1970 + (days / 365) as u32,
        ((days % 365) / 30 + 1) as u32,
        ((days % 365) % 30 + 1) as u32,
        hours,
        minutes,
        seconds,
        millis
    )
}

fn parse_iso_to_epoch(iso: &str) -> u64 {
    // "2026-06-17T12:34:56.789Z" -> seconds since epoch
    if iso.len() < 20 {
        return 0;
    }
    let year: u64 = iso[0..4].parse().unwrap_or(0);
    let month: u64 = iso[5..7].parse().unwrap_or(1);
    let day: u64 = iso[8..10].parse().unwrap_or(1);
    let hour: u64 = iso[11..13].parse().unwrap_or(0);
    let min: u64 = iso[14..16].parse().unwrap_or(0);
    let sec: u64 = iso[17..19].parse().unwrap_or(0);
    let years_since_epoch = year.saturating_sub(1970);
    let leap_days = years_since_epoch / 4 + 1;
    let days_since_epoch = years_since_epoch * 365
        + leap_days
        + [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            .iter()
            .take(month as usize)
            .sum::<u64>()
        + day
        - 1;
    days_since_epoch * 86400 + hour * 3600 + min * 60 + sec
}

fn goal_id() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("goal-{}", d.as_millis())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentGoal {
    pub id: String,
    pub description: String,
    pub priority: GoalPriority,
    pub state: GoalLoopState,
    pub created_at: String,
    pub updated_at: String,
    pub max_retries: u32,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub tags: Vec<String>,
    pub plan: Option<LeadAgentPlan>,
    pub results: Vec<SubAgentResult>,
    pub deadline_epoch_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub goal_id: String,
    pub description: String,
    pub progress_pct: f64,
    pub subtasks_completed: usize,
    pub subtasks_total: usize,
    pub current_phase: String,
    pub eta_estimate: String,
    pub state: GoalLoopState,
}

pub struct PersistentGoalManager {
    pub goals: HashMap<String, PersistentGoal>,
    pub max_goals: usize,
    pub lead_agent: LeadAgent,
    pub config: GoalConfig,
    persistence_path: Option<PathBuf>,
}

impl PersistentGoalManager {
    pub fn new(lead_config: LeadAgentConfig, goal_config: GoalConfig) -> Self {
        Self {
            goals: HashMap::new(),
            max_goals: 20,
            lead_agent: LeadAgent::new(lead_config),
            config: goal_config,
            persistence_path: None,
        }
    }

    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.persistence_path = Some(path);
        self
    }

    pub fn create_goal(
        &mut self,
        description: &str,
        priority: GoalPriority,
    ) -> Result<String, String> {
        if self.goals.len() >= self.max_goals {
            return Err(format!("Max goals ({}) reached", self.max_goals));
        }
        let id = goal_id();
        let now = now_iso();
        let goal = PersistentGoal {
            id: id.clone(),
            description: description.to_string(),
            priority,
            state: GoalLoopState::Pursuing,
            created_at: now.clone(),
            updated_at: now,
            max_retries: 3,
            retry_count: 0,
            last_error: None,
            tags: Vec::new(),
            plan: None,
            results: Vec::new(),
            deadline_epoch_secs: None,
        };
        self.goals.insert(id.clone(), goal);
        let _ = self.save();
        Ok(id)
    }

    pub fn preview_goal(&mut self, description: &str) -> PreviewResult {
        PreviewEngine::generate_options(description)
    }

    pub fn execute_goal(
        &mut self,
        goal_id: &str,
        plan_effort: PlanEffort,
    ) -> Result<Vec<SubAgentResult>, String> {
        let goal = self
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| format!("Goal {} not found", goal_id))?;
        if goal.state == GoalLoopState::Achieved {
            return Err("Goal already achieved".into());
        }
        if goal.retry_count >= goal.max_retries {
            goal.state = GoalLoopState::Unmet;
            return Err("Max retries exceeded".into());
        }
        if let Some(deadline) = goal.deadline_epoch_secs {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > deadline {
                goal.state = GoalLoopState::BudgetLimited;
                return Err("Goal deadline passed".into());
            }
        }
        goal.updated_at = now_iso();
        self.lead_agent.config.plan_effort = plan_effort;
        let plan = self.lead_agent.plan(&goal.description);
        goal.plan = Some(plan);
        let results = self.lead_agent.execute_goal(&goal.description);
        let all_success = results.iter().all(|r| r.success);
        goal.results.extend(results.clone());
        if all_success {
            goal.state = GoalLoopState::Achieved;
        } else {
            goal.retry_count += 1;
            let errors: Vec<String> = results
                .iter()
                .filter(|r| !r.success)
                .filter_map(|r| r.error.clone())
                .collect();
            if !errors.is_empty() {
                goal.last_error = Some(errors.join("; "));
            }
            if goal.retry_count >= goal.max_retries {
                goal.state = GoalLoopState::Unmet;
            }
        }
        let _ = self.save();
        Ok(results)
    }

    pub fn resume_paused_goal(&mut self, goal_id: &str) -> Result<(), String> {
        let goal = self
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| format!("Goal {} not found", goal_id))?;
        if goal.state != GoalLoopState::Paused {
            return Err("Goal is not paused".into());
        }
        goal.state = GoalLoopState::Pursuing;
        goal.updated_at = now_iso();
        let _ = self.save();
        Ok(())
    }

    pub fn pause_goal(&mut self, goal_id: &str) -> Result<(), String> {
        let goal = self
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| format!("Goal {} not found", goal_id))?;
        if goal.state != GoalLoopState::Pursuing {
            return Err("Goal is not pursuing".into());
        }
        goal.state = GoalLoopState::Paused;
        goal.updated_at = now_iso();
        let _ = self.save();
        Ok(())
    }

    pub fn cancel_goal(&mut self, goal_id: &str) -> Result<(), String> {
        let goal = self
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| format!("Goal {} not found", goal_id))?;
        if goal.state.is_terminal() {
            return Err("Goal already in terminal state".into());
        }
        goal.state = GoalLoopState::Unmet;
        goal.updated_at = now_iso();
        let _ = self.save();
        Ok(())
    }

    pub fn goal_progress(&self, goal_id: &str) -> Option<GoalProgress> {
        self.goals.get(goal_id).map(|goal| {
            let total = goal
                .plan
                .as_ref()
                .map(|p| p.decomposition.sub_tasks.len())
                .unwrap_or(0);
            let done = self.lead_agent.state.completed_tasks.len();
            let progress = if total > 0 {
                (done as f64 / total as f64 * 100.0).min(100.0)
            } else {
                0.0
            };
            GoalProgress {
                goal_id: goal_id.to_string(),
                description: goal.description.clone(),
                progress_pct: progress,
                subtasks_completed: done.min(total),
                subtasks_total: total,
                current_phase: if goal.plan.is_none() {
                    "planning".into()
                } else if !self.lead_agent.all_tasks_complete() {
                    "executing".into()
                } else {
                    "completed".into()
                },
                eta_estimate: "TBD".into(),
                state: goal.state.clone(),
            }
        })
    }

    pub fn active_goals(&self) -> Vec<&PersistentGoal> {
        self.goals
            .values()
            .filter(|g| !g.state.is_terminal())
            .collect()
    }

    pub fn achieved_goals(&self) -> Vec<&PersistentGoal> {
        self.goals
            .values()
            .filter(|g| g.state == GoalLoopState::Achieved)
            .collect()
    }

    pub fn summary(&self) -> String {
        let active = self.active_goals().len();
        let achieved = self.achieved_goals().len();
        let total = self.goals.len();
        format!(
            "PersistentGoalManager: {} total, {} active, {} achieved",
            total, active, achieved
        )
    }

    pub fn prune_old_goals(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.goals.retain(|_, g| {
            if g.state.is_terminal() {
                let updated_epoch = parse_iso_to_epoch(&g.updated_at);
                if updated_epoch > 0 && now.saturating_sub(updated_epoch) > max_age_secs {
                    return false;
                }
            }
            true
        });
    }

    pub fn set_deadline(&mut self, goal_id: &str, deadline_epoch_secs: u64) -> Result<(), String> {
        let goal = self
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| format!("Goal {} not found", goal_id))?;
        goal.deadline_epoch_secs = Some(deadline_epoch_secs);
        Ok(())
    }

    pub fn check_deadlines(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for goal in self.goals.values_mut() {
            if goal.state == GoalLoopState::Pursuing || goal.state == GoalLoopState::Paused {
                if let Some(deadline) = goal.deadline_epoch_secs {
                    if now > deadline {
                        goal.state = GoalLoopState::BudgetLimited;
                    }
                }
            }
        }
    }

    pub fn max_goals(&self) -> usize {
        self.max_goals
    }

    pub fn total_goal_count(&self) -> usize {
        self.goals.len()
    }

    // ── Persistence ──

    fn save(&self) -> Result<(), String> {
        let path = match &self.persistence_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };
        let data = serde_json::to_string(&self.goals).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &data).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    fn load(&mut self, path: &std::path::Path) -> Result<(), String> {
        let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let goals: HashMap<String, PersistentGoal> =
            serde_json::from_str(&data).map_err(|e| e.to_string())?;
        self.goals = goals;
        Ok(())
    }

    pub fn try_load(&mut self, path: &std::path::Path) {
        if path.exists() {
            let _ = self.load(path);
        }
        self.persistence_path = Some(path.to_path_buf());
    }
}
