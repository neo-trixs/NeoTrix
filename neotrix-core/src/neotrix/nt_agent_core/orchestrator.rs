use super::error::AgentError;
use super::message::{AgentId, AgentRole};
use std::collections::HashMap;
use std::time::{Duration, Instant};

static NEXT_TEAM_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static NEXT_TASK_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn name(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

#[derive(Debug, Clone)]
pub struct AgentTeam {
    pub team_id: u64,
    pub name: String,
    pub members: Vec<(AgentId, AgentRole)>,
    pub coordinator: AgentId,
    pub created_at: Instant,
    pub task_count: u64,
}

#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub task_id: u64,
    pub description: String,
    pub team_id: u64,
    pub subtasks: Vec<(u64, AgentId, String)>,
    pub status: TaskStatus,
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub teams_formed: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub active_teams: usize,
    pub avg_completion_time: Duration,
}

impl OrchestratorStats {
    pub fn new() -> Self {
        Self {
            teams_formed: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            active_teams: 0,
            avg_completion_time: Duration::from_secs(0),
        }
    }
}

pub struct TeamOrchestrator {
    teams: HashMap<u64, AgentTeam>,
    task_assignments: HashMap<u64, TaskAssignment>,
    max_team_size: usize,
    stats: OrchestratorStats,
}

impl TeamOrchestrator {
    pub fn new(max_team_size: usize) -> Self {
        Self {
            teams: HashMap::new(),
            task_assignments: HashMap::new(),
            max_team_size: max_team_size.max(1),
            stats: OrchestratorStats::new(),
        }
    }

    pub fn form_team(
        &mut self,
        name: &str,
        coordinator: AgentId,
        members: Vec<(AgentId, AgentRole)>,
    ) -> Result<u64, AgentError> {
        if members.len() > self.max_team_size {
            return Err(AgentError::InvalidState(format!(
                "Team size {} exceeds max {}",
                members.len(),
                self.max_team_size
            )));
        }

        if members.iter().any(|(_, r)| *r == AgentRole::Coordinator) {
            return Err(AgentError::InvalidState(
                "Only the designated coordinator should be Coordinator role".into(),
            ));
        }

        let coordinator_role = members.iter().find(|(id, _)| *id == coordinator);
        if coordinator_role.is_none() {
            let coord_is_member = members.iter().any(|(id, _)| *id == coordinator);
            if !coord_is_member {
                return Err(AgentError::NotFound(
                    "Coordinator must be a team member".into(),
                ));
            }
        }

        let team_id = NEXT_TEAM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let team = AgentTeam {
            team_id,
            name: name.to_string(),
            members,
            coordinator,
            created_at: Instant::now(),
            task_count: 0,
        };

        self.stats.teams_formed += 1;
        self.teams.insert(team_id, team);
        self.stats.active_teams = self.teams.len();
        Ok(team_id)
    }

    pub fn disband_team(&mut self, team_id: u64) {
        self.teams.remove(&team_id);
        self.stats.active_teams = self.teams.len();
    }

    pub fn assign_task(&mut self, team_id: u64, description: &str) -> Result<u64, AgentError> {
        let team = self
            .teams
            .get(&team_id)
            .ok_or_else(|| AgentError::NotFound(format!("Team {} not found", team_id)))?;

        let task_id = NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let subtasks = self.decompose_task(description, team);

        let assignment = TaskAssignment {
            task_id,
            description: description.to_string(),
            team_id,
            subtasks,
            status: TaskStatus::Pending,
            created_at: Instant::now(),
        };

        if let Some(t) = self.teams.get_mut(&team_id) {
            t.task_count += 1;
        }
        self.task_assignments.insert(task_id, assignment);
        Ok(task_id)
    }

    pub fn decompose_task(
        &self,
        description: &str,
        team: &AgentTeam,
    ) -> Vec<(u64, AgentId, String)> {
        let lower = description.to_lowercase();
        let mut needed: Vec<AgentRole> = Vec::new();

        if lower.contains("research") || lower.contains("find") || lower.contains("search") {
            needed.push(AgentRole::Researcher);
        }
        if lower.contains("verify")
            || lower.contains("validate")
            || lower.contains("check")
            || lower.contains("test")
        {
            needed.push(AgentRole::Verifier);
        }
        if lower.contains("review") || lower.contains("critique") || lower.contains("audit") {
            needed.push(AgentRole::Critic);
        }
        if lower.contains("synthesize")
            || lower.contains("merge")
            || lower.contains("combine")
            || lower.contains("summary")
        {
            needed.push(AgentRole::Synthesizer);
        }
        if lower.contains("execute")
            || lower.contains("implement")
            || lower.contains("build")
            || lower.contains("code")
        {
            needed.push(AgentRole::Specialist);
        }
        if needed.is_empty() {
            needed.push(AgentRole::Specialist);
        }

        let mut assigned: Vec<AgentId> = Vec::new();
        let mut subtasks: Vec<(u64, AgentId, String)> = Vec::new();

        for role in &needed {
            let desc = match role {
                AgentRole::Researcher => "Gather information",
                AgentRole::Verifier => "Validate outputs",
                AgentRole::Critic => "Review and challenge",
                AgentRole::Synthesizer => "Merge multiple outputs",
                AgentRole::Specialist => "Domain-specific execution",
                AgentRole::Coordinator => "Coordinate subtasks",
            };

            let pick = team
                .members
                .iter()
                .find(|(id, r)| r == role && !assigned.iter().any(|a| a == id))
                .or_else(|| {
                    team.members.iter().find(|(id, r)| {
                        *r == AgentRole::Coordinator && !assigned.iter().any(|a| a == id)
                    })
                })
                .or_else(|| {
                    team.members
                        .iter()
                        .find(|(id, _)| !assigned.iter().any(|a| a == id))
                });

            if let Some((aid, _)) = pick {
                let sid = NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                assigned.push(aid.clone());
                subtasks.push((sid, aid.clone(), desc.to_string()));
            }
        }

        subtasks
    }

    pub fn update_subtask(&mut self, task_id: u64, subtask_id: u64, status: TaskStatus) {
        if let Some(task) = self.task_assignments.get_mut(&task_id) {
            if let Some(pos) = task
                .subtasks
                .iter()
                .position(|(id, _, _)| *id == subtask_id)
            {
                if status.is_terminal() {
                    task.subtasks.remove(pos);
                }
            }

            if task.subtasks.is_empty() && !task.status.is_terminal() {
                match status {
                    TaskStatus::Failed => {
                        task.status = TaskStatus::Failed;
                        self.stats.tasks_failed += 1;
                    }
                    TaskStatus::Cancelled => {
                        task.status = TaskStatus::Cancelled;
                    }
                    _ => {
                        if status.is_terminal() || task.subtasks.is_empty() {
                            task.status = TaskStatus::Completed;
                            self.stats.tasks_completed += 1;
                        }
                    }
                }
            } else if !task.status.is_terminal() {
                task.status = TaskStatus::InProgress;
            }
        }
    }

    pub fn team_summary(&self, team_id: u64) -> Option<String> {
        let team = self.teams.get(&team_id)?;
        let mut lines = Vec::new();
        lines.push(format!("Team: {} (id={})", team.name, team.team_id));
        lines.push(format!("Coordinator: {}", team.coordinator));
        lines.push(format!("Members: {}", team.members.len()));
        for (id, role) in &team.members {
            lines.push(format!("  {} as {:?}", id, role));
        }
        lines.push(format!("Tasks completed: {}", team.task_count));
        Some(lines.join("\n"))
    }

    pub fn stats(&self) -> &OrchestratorStats {
        &self.stats
    }

    pub fn team(&self, team_id: u64) -> Option<&AgentTeam> {
        self.teams.get(&team_id)
    }

    pub fn task(&self, task_id: u64) -> Option<&TaskAssignment> {
        self.task_assignments.get(&task_id)
    }

    pub fn list_teams(&self) -> impl Iterator<Item = &AgentTeam> {
        self.teams.values()
    }

    pub fn list_tasks(&self) -> impl Iterator<Item = &TaskAssignment> {
        self.task_assignments.values()
    }

    pub fn active_task_count(&self) -> usize {
        self.task_assignments
            .values()
            .filter(|t| !t.status.is_terminal())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(name: &str) -> AgentId {
        AgentId::with_random_instance(name, "1.0")
    }

    #[test]
    fn test_form_team() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");
        let verif = agent("verif");

        let team_id = orch
            .form_team(
                "alpha-team",
                coord.clone(),
                vec![
                    (coord.clone(), AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                    (verif, AgentRole::Verifier),
                ],
            )
            .unwrap();

        assert_eq!(orch.stats().teams_formed, 1);
        assert_eq!(orch.stats().active_teams, 1);

        let team = orch.team(team_id).unwrap();
        assert_eq!(team.name, "alpha-team");
        assert_eq!(team.coordinator, coord);
        assert_eq!(team.members.len(), 3);
    }

    #[test]
    fn test_form_team_exceeds_max() {
        let mut orch = TeamOrchestrator::new(2);
        let coord = agent("coord");
        let a = agent("a");
        let b = agent("b");
        let c = agent("c");

        let result = orch.form_team(
            "big",
            coord.clone(),
            vec![
                (coord, AgentRole::Coordinator),
                (a, AgentRole::Specialist),
                (b, AgentRole::Verifier),
                (c, AgentRole::Critic),
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_disband_team() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");

        let tid = orch
            .form_team(
                "tmp",
                coord.clone(),
                vec![
                    (coord, AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                ],
            )
            .unwrap();
        assert_eq!(orch.stats().active_teams, 1);

        orch.disband_team(tid);
        assert_eq!(orch.stats().active_teams, 0);
        assert!(orch.team(tid).is_none());
    }

    #[test]
    fn test_assign_task_basic() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");

        let tid = orch
            .form_team(
                "dev",
                coord.clone(),
                vec![
                    (coord.clone(), AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                ],
            )
            .unwrap();

        let task_id = orch.assign_task(tid, "Implement feature X").unwrap();
        let task = orch.task(task_id).unwrap();
        assert_eq!(task.description, "Implement feature X");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_assign_task_invalid_team() {
        let mut orch = TeamOrchestrator::new(5);
        let result = orch.assign_task(999, "work");
        assert!(result.is_err());
    }

    #[test]
    fn test_decompose_task_heuristic() {
        let mut orch = TeamOrchestrator::new(10);
        let coord = agent("coord");
        let spec = agent("spec");
        let verif = agent("verif");
        let res = agent("res");
        let crit = agent("crit");
        let synth = agent("synth");

        let tid = orch
            .form_team(
                "full",
                coord.clone(),
                vec![
                    (coord.clone(), AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                    (verif, AgentRole::Verifier),
                    (res, AgentRole::Researcher),
                    (crit, AgentRole::Critic),
                    (synth, AgentRole::Synthesizer),
                ],
            )
            .unwrap();

        let task_id = orch
            .assign_task(
                tid,
                "Research, implement, verify, review and synthesize the new feature",
            )
            .unwrap();
        let task = orch.task(task_id).unwrap();
        // Should have subtasks for: research, implement, verify, review, synthesize
        assert!(!task.subtasks.is_empty());
    }

    #[test]
    fn test_update_subtask_completion() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");
        let verif = agent("verif");

        let tid = orch
            .form_team(
                "test-team",
                coord.clone(),
                vec![
                    (coord.clone(), AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                    (verif, AgentRole::Verifier),
                ],
            )
            .unwrap();

        let task_id = orch.assign_task(tid, "Build and test").unwrap();
        let task = orch.task(task_id).unwrap();
        let subtask_ids: Vec<u64> = task.subtasks.iter().map(|(id, _, _)| *id).collect();

        // Mark all subtasks completed
        for sid in &subtask_ids {
            orch.update_subtask(task_id, *sid, TaskStatus::Completed);
        }
        // Last completed should mark task completed
        let last_sid = *subtask_ids.last().unwrap_or(&0);
        orch.update_subtask(task_id, last_sid, TaskStatus::Completed);

        assert_eq!(orch.task(task_id).unwrap().status, TaskStatus::Completed);
        assert_eq!(orch.stats().tasks_completed, 1);
    }

    #[test]
    fn test_update_subtask_failure() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");

        let tid = orch
            .form_team(
                "fail-team",
                coord.clone(),
                vec![
                    (coord, AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                ],
            )
            .unwrap();

        let task_id = orch.assign_task(tid, "Risky task").unwrap();
        let task = orch.task(task_id).unwrap();
        if let Some((sid, _, _)) = task.subtasks.first() {
            orch.update_subtask(task_id, *sid, TaskStatus::Failed);
        }

        assert_eq!(orch.task(task_id).unwrap().status, TaskStatus::Failed);
        assert_eq!(orch.stats().tasks_failed, 1);
    }

    #[test]
    fn test_team_summary() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("lead");
        let spec = agent("dev");

        let tid = orch
            .form_team(
                "summary-team",
                coord.clone(),
                vec![
                    (coord, AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                ],
            )
            .unwrap();

        let summary = orch.team_summary(tid);
        assert!(summary.is_some());
        let text = summary.unwrap();
        assert!(text.contains("summary-team"));
        assert!(text.contains("Coordinator"));
        assert!(text.contains("Members: 2"));
    }

    #[test]
    fn test_team_summary_not_found() {
        let orch = TeamOrchestrator::new(5);
        assert!(orch.team_summary(999).is_none());
    }

    #[test]
    fn test_task_status_names() {
        assert_eq!(TaskStatus::Pending.name(), "Pending");
        assert_eq!(TaskStatus::InProgress.name(), "InProgress");
        assert_eq!(TaskStatus::Completed.name(), "Completed");
        assert_eq!(TaskStatus::Failed.name(), "Failed");
        assert_eq!(TaskStatus::Cancelled.name(), "Cancelled");
    }

    #[test]
    fn test_task_status_terminal() {
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_active_task_count() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("coord");
        let spec = agent("spec");

        let tid = orch
            .form_team(
                "test",
                coord.clone(),
                vec![
                    (coord, AgentRole::Coordinator),
                    (spec, AgentRole::Specialist),
                ],
            )
            .unwrap();

        let t1 = orch.assign_task(tid, "Task 1").unwrap();
        let t2 = orch.assign_task(tid, "Task 2").unwrap();
        assert_eq!(orch.active_task_count(), 2);

        if let Some(task) = orch.task(t1) {
            if let Some((sid, _, _)) = task.subtasks.first() {
                orch.update_subtask(t1, *sid, TaskStatus::Completed);
            }
        }

        // After completing one task
        assert_eq!(orch.active_task_count(), 1);

        if let Some(task) = orch.task(t2) {
            if let Some((sid, _, _)) = task.subtasks.first() {
                orch.update_subtask(t2, *sid, TaskStatus::Cancelled);
            }
        }
        assert_eq!(orch.active_task_count(), 0);
    }

    #[test]
    fn test_coordinator_must_be_member() {
        let mut orch = TeamOrchestrator::new(5);
        let coord = agent("outsider");
        let spec = agent("spec");

        let result = orch.form_team("no-coord", coord, vec![(spec, AgentRole::Specialist)]);
        assert!(result.is_err());
    }
}
