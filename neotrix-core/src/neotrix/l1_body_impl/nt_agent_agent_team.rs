use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Lead,
    Researcher,
    Coder,
    Reviewer,
    Tester,
    Architect,
    Observer,
}

impl AgentRole {
    pub fn label(&self) -> &'static str {
        match self {
            AgentRole::Lead => "Lead",
            AgentRole::Researcher => "Researcher",
            AgentRole::Coder => "Coder",
            AgentRole::Reviewer => "Reviewer",
            AgentRole::Tester => "Tester",
            AgentRole::Architect => "Architect",
            AgentRole::Observer => "Observer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    ReviewNeeded,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct TeamTask {
    pub id: usize,
    pub description: String,
    pub assigned_role: AgentRole,
    pub status: TaskStatus,
    pub dependencies: Vec<usize>,
    pub created: Instant,
    pub completed: Option<Instant>,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct TeamMessage {
    pub from_role: AgentRole,
    pub to_role: AgentRole,
    pub content: String,
    pub task_id: Option<usize>,
    pub timestamp: Instant,
    pub urgent: bool,
}

#[derive(Debug, Clone)]
pub struct AgentProfile {
    pub role: AgentRole,
    pub name: String,
    pub skills: Vec<String>,
    pub tasks_completed: usize,
    pub reliability_score: f64,
    pub avg_response_ms: f64,
    pub joined: Instant,
}

impl AgentProfile {
    pub fn new(role: AgentRole, name: &str) -> Self {
        Self {
            role,
            name: name.to_string(),
            skills: Vec::new(),
            tasks_completed: 0,
            reliability_score: 1.0,
            avg_response_ms: 0.0,
            joined: Instant::now(),
        }
    }

    pub fn add_skill(&mut self, skill: &str) {
        if !self.skills.contains(&skill.to_string()) {
            self.skills.push(skill.to_string());
        }
    }
}

pub struct AgentTeam {
    name: String,
    members: HashMap<AgentRole, AgentProfile>,
    tasks: Vec<TeamTask>,
    messages: VecDeque<TeamMessage>,
    task_counter: usize,
    workflow_active: bool,
    start_time: Option<Instant>,
}

impl AgentTeam {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: HashMap::new(),
            tasks: Vec::new(),
            messages: VecDeque::new(),
            task_counter: 0,
            workflow_active: false,
            start_time: None,
        }
    }

    pub fn add_member(&mut self, profile: AgentProfile) {
        self.members.insert(profile.role, profile);
    }

    pub fn has_role(&self, role: AgentRole) -> bool {
        self.members.contains_key(&role)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn create_task(&mut self, description: &str, role: AgentRole, priority: u8) -> usize {
        self.task_counter += 1;
        let task = TeamTask {
            id: self.task_counter,
            description: description.to_string(),
            assigned_role: role,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            created: Instant::now(),
            completed: None,
            priority,
        };
        self.tasks.push(task);
        self.task_counter
    }

    pub fn add_dependency(&mut self, task_id: usize, depends_on: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if !task.dependencies.contains(&depends_on) && task_id != depends_on {
                task.dependencies.push(depends_on);
            }
        }
    }

    pub fn assign_tasks(&mut self) -> Vec<(usize, AgentRole)> {
        let mut assigned = Vec::new();
        let completed_ids: std::collections::HashSet<usize> = self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id)
            .collect();
        for task in self.tasks.iter_mut() {
            if task.status == TaskStatus::Pending {
                let deps_met = task.dependencies.iter().all(|dep_id| {
                    completed_ids.contains(dep_id)
                });
                if deps_met && self.members.contains_key(&task.assigned_role) {
                    task.status = TaskStatus::Assigned;
                    assigned.push((task.id, task.assigned_role));
                }
                // deps_met but role member missing: keep Pending so the task is
                // re-evaluated on the next assign_tasks pass once add_member
                // supplies the role. Previously this became Blocked, which
                // assign_tasks never re-examines — permanently stuck.
            }
        }
        assigned
    }

    pub fn send_message(&mut self, msg: TeamMessage) {
        self.messages.push_back(msg);
    }

    pub fn read_messages(&mut self, role: AgentRole) -> Vec<TeamMessage> {
        let remaining: VecDeque<TeamMessage> = self.messages.iter()
            .filter(|m| m.to_role != role)
            .cloned()
            .collect();
        let for_role = self.messages.iter()
            .filter(|m| m.to_role == role)
            .cloned()
            .collect();
        self.messages = remaining;
        for_role
    }

    pub fn complete_task(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
            task.completed = Some(Instant::now());
            if let Some(member) = self.members.get_mut(&task.assigned_role) {
                member.tasks_completed += 1;
            }
        }
    }

    pub fn fail_task(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Failed;
        }
    }

    pub fn get_ready_tasks(&self) -> Vec<&TeamTask> {
        let completed_ids: HashSet<usize> = self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id)
            .collect();
        self.tasks.iter()
            .filter(|t| {
                if t.status == TaskStatus::Assigned {
                    true
                } else if t.status == TaskStatus::Pending {
                    t.dependencies.iter().all(|dep_id| completed_ids.contains(dep_id))
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 1.0;
        }
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        (completed as f64 / self.tasks.len() as f64).max(0.0).min(1.0)
    }

    pub fn start_workflow(&mut self) {
        self.workflow_active = true;
        self.start_time = Some(Instant::now());
    }

    pub fn stop_workflow(&mut self) -> Duration {
        self.workflow_active = false;
        self.start_time.map(|s| s.elapsed()).unwrap_or(Duration::ZERO)
    }

    pub fn is_active(&self) -> bool {
        self.workflow_active
    }

    pub fn team_stats(&self) -> TeamStats {
        let total = self.tasks.len();
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = self.tasks.iter().filter(|t| t.status == TaskStatus::Failed).count();
        let avg_reliability = if self.members.is_empty() {
            0.0
        } else {
            self.members.values().map(|m| m.reliability_score).sum::<f64>() / self.members.len() as f64
        };
        let total_tasks = self.members.values().map(|m| m.tasks_completed).sum();
        TeamStats {
            team_name: self.name.clone(),
            total_tasks: total,
            completed_tasks: completed,
            failed_tasks: failed,
            member_count: self.members.len(),
            completion_ratio: self.progress(),
            avg_member_reliability: avg_reliability.max(0.0).min(1.0),
            total_messages: self.messages.len(),
            total_tasks_done: total_tasks,
            active: self.workflow_active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TeamStats {
    pub team_name: String,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub member_count: usize,
    pub completion_ratio: f64,
    pub avg_member_reliability: f64,
    pub total_messages: usize,
    pub total_tasks_done: usize,
    pub active: bool,
}

pub struct TaskDecomposer;

impl TaskDecomposer {
    pub fn decompose(goal: &str, max_subtasks: usize) -> Vec<String> {
        let mut subtasks = Vec::new();
        let lines: Vec<&str> = goal.lines().collect();
        if lines.len() > 1 {
            for line in lines.iter().take(max_subtasks) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    subtasks.push(trimmed.to_string());
                }
            }
        }
        if subtasks.is_empty() {
            subtasks.push(format!("Research: {}", goal));
            subtasks.push(format!("Implement: {}", goal));
            subtasks.push(format!("Review: {}", goal));
            subtasks.push(format!("Test: {}", goal));
        }
        subtasks.truncate(max_subtasks);
        subtasks
    }

    pub fn suggest_team(goal: &str) -> Vec<AgentRole> {
        let lower = goal.to_lowercase();
        let mut team = vec![AgentRole::Lead];
        if lower.contains("code") || lower.contains("implement") || lower.contains("build") {
            team.push(AgentRole::Coder);
        }
        if lower.contains("research") || lower.contains("search") || lower.contains("find") {
            team.push(AgentRole::Researcher);
        }
        if lower.contains("review") || lower.contains("audit") || lower.contains("check") {
            team.push(AgentRole::Reviewer);
        }
        if lower.contains("test") || lower.contains("verify") || lower.contains("validate") {
            team.push(AgentRole::Tester);
        }
        team.push(AgentRole::Observer);
        team
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_team() -> AgentTeam {
        let mut team = AgentTeam::new("dev-team");
        team.add_member(AgentProfile::new(AgentRole::Lead, "alice"));
        team.add_member(AgentProfile::new(AgentRole::Coder, "bob"));
        team.add_member(AgentProfile::new(AgentRole::Reviewer, "carol"));
        team.add_member(AgentProfile::new(AgentRole::Tester, "dave"));
        team
    }

    #[test]
    fn test_team_creation() {
        let team = sample_team();
        assert_eq!(team.member_count(), 4);
        assert!(team.has_role(AgentRole::Lead));
        assert!(!team.has_role(AgentRole::Researcher));
    }

    #[test]
    fn test_task_lifecycle() {
        let mut team = sample_team();
        let t1 = team.create_task("implement feature", AgentRole::Coder, 1);
        let t2 = team.create_task("review feature", AgentRole::Reviewer, 2);
        team.add_dependency(t2, t1);
        let assigned = team.assign_tasks();
        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].1, AgentRole::Coder);
        team.complete_task(t1);
        let assigned2 = team.assign_tasks();
        assert_eq!(assigned2.len(), 1);
        assert_eq!(assigned2[0].1, AgentRole::Reviewer);
    }

    #[test]
    fn test_progress_tracking() {
        let mut team = sample_team();
        let t1 = team.create_task("task1", AgentRole::Coder, 1);
        let t2 = team.create_task("task2", AgentRole::Coder, 1);
        assert!((team.progress() - 0.0).abs() < 0.001);
        team.complete_task(t1);
        assert!((team.progress() - 0.5).abs() < 0.001);
        team.complete_task(t2);
        assert!((team.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_message_passing() {
        let mut team = sample_team();
        team.send_message(TeamMessage {
            from_role: AgentRole::Lead,
            to_role: AgentRole::Coder,
            content: "start coding".into(),
            task_id: None,
            timestamp: Instant::now(),
            urgent: false,
        });
        let coder_msgs = team.read_messages(AgentRole::Coder);
        assert_eq!(coder_msgs.len(), 1);
        assert_eq!(coder_msgs[0].content, "start coding");
        let lead_msgs = team.read_messages(AgentRole::Lead);
        assert_eq!(lead_msgs.len(), 0);
    }

    #[test]
    fn test_task_decomposition() {
        let subtasks = TaskDecomposer::decompose("Build a web app with Rust", 4);
        assert_eq!(subtasks.len(), 4);
        assert!(subtasks.iter().any(|s| s.contains("Implement")));
    }

    #[test]
    fn test_team_suggestion() {
        let team = TaskDecomposer::suggest_team("implement and test new feature");
        assert!(team.contains(&AgentRole::Coder));
        assert!(team.contains(&AgentRole::Tester));
        assert!(team.contains(&AgentRole::Lead));
    }

    #[test]
    fn test_team_stats() {
        let mut team = sample_team();
        team.start_workflow();
        let t1 = team.create_task("t1", AgentRole::Coder, 1);
        team.complete_task(t1);
        let stats = team.team_stats();
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.total_tasks, 1);
        assert!(stats.active);
        team.stop_workflow();
        assert!(!team.is_active());
    }

    #[test]
    fn test_role_labels() {
        assert_eq!(AgentRole::Lead.label(), "Lead");
        assert_eq!(AgentRole::Observer.label(), "Observer");
    }

    #[test]
    fn test_agent_profile() {
        let mut p = AgentProfile::new(AgentRole::Architect, "eve");
        p.add_skill("Rust");
        p.add_skill("System Design");
        assert_eq!(p.skills.len(), 2);
        p.add_skill("Rust");
        assert_eq!(p.skills.len(), 2);
    }

    #[test]
    fn test_blocked_task() {
        let mut team = sample_team();
        let t1 = team.create_task("task1", AgentRole::Coder, 1);
        let t2 = team.create_task("task2", AgentRole::Coder, 1);
        team.add_dependency(t2, t1);
        team.assign_tasks();
        team.fail_task(t1);
        assert!(!team.get_ready_tasks().iter().any(|t| t.id == t2));
    }

    #[test]
    fn test_task_with_missing_role_stays_pending_until_member_added() {
        // Regression: deps-met but role-member-missing tasks were set to Blocked,
        // which assign_tasks never re-examines — adding the member later left the
        // task permanently stuck. Now they stay Pending and get assigned.
        let mut team = AgentTeam::new("role-test");
        // No Researcher member yet.
        let t1 = team.create_task("research", AgentRole::Researcher, 1);
        let assigned_before = team.assign_tasks();
        assert!(!assigned_before.iter().any(|(id, _)| *id == t1),
            "no member for role: task must not be assigned yet");
        let t1_state = team.tasks.iter().find(|t| t.id == t1).unwrap();
        assert_eq!(t1_state.status, TaskStatus::Pending,
            "task must stay Pending (not Blocked) so it can be re-evaluated");

        team.add_member(AgentProfile::new(AgentRole::Researcher, "eve"));
        let assigned_after = team.assign_tasks();
        assert!(assigned_after.iter().any(|(id, _)| *id == t1),
            "task must become Assigned once the role member arrives");
    }
}
