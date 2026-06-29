use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamPattern {
    Supervisor,
    Pipeline,
    RoundRobin,
    Debate,
    FanOutFanIn,
    ExpertPool,
    HierarchicalDelegation,
}

impl TeamPattern {
    pub fn name(&self) -> &'static str {
        match self {
            TeamPattern::Supervisor => "supervisor",
            TeamPattern::Pipeline => "pipeline",
            TeamPattern::RoundRobin => "round_robin",
            TeamPattern::Debate => "debate",
            TeamPattern::FanOutFanIn => "fan_out_fan_in",
            TeamPattern::ExpertPool => "expert_pool",
            TeamPattern::HierarchicalDelegation => "hierarchical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub role: String,
    pub specialization: String,
    pub confidence: f64,
    pub task_count: u64,
    pub success_rate: f64,
    pub hierarchical_level: usize,
}

impl TeamMember {
    pub fn new(id: &str, role: &str, specialization: &str) -> Self {
        Self {
            id: id.to_string(),
            role: role.to_string(),
            specialization: specialization.to_string(),
            confidence: 0.5,
            task_count: 0,
            success_rate: 0.0,
            hierarchical_level: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTask {
    pub id: String,
    pub description: String,
    pub required_specialization: Option<String>,
    pub completed: bool,
    pub result: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamOrchestrator {
    pub pattern: TeamPattern,
    pub members: Vec<TeamMember>,
    pub round_robin_index: usize,
    pub completed_tasks: Vec<TeamTask>,
    pub max_members: usize,
    pub pool_specialization: String,
}

impl TeamOrchestrator {
    pub fn new(pattern: TeamPattern) -> Self {
        Self {
            pattern,
            members: Vec::new(),
            round_robin_index: 0,
            completed_tasks: Vec::new(),
            max_members: 10,
            pool_specialization: "general".to_string(),
        }
    }

    pub fn with_pool_specialization(mut self, specialization: &str) -> Self {
        self.pool_specialization = specialization.to_string();
        self
    }

    pub fn add_member(&mut self, role: &str, specialization: &str) -> String {
        let id = format!("{}_{}", self.pattern.name(), self.members.len() + 1);
        self.members
            .push(TeamMember::new(&id, role, specialization));
        id
    }

    pub fn add_member_with_level(
        &mut self,
        role: &str,
        specialization: &str,
        level: usize,
    ) -> String {
        let id = format!("{}_{}", self.pattern.name(), self.members.len() + 1);
        let mut member = TeamMember::new(&id, role, specialization);
        member.hierarchical_level = level;
        self.members.push(member);
        id
    }

    pub fn dispatch(&mut self, task: TeamTask) -> Option<String> {
        let m_id = {
            let member_id = match self.pattern {
                TeamPattern::Supervisor => self.pick_supervisor(&task),
                TeamPattern::Pipeline => self.pick_pipeline(&task),
                TeamPattern::RoundRobin => self.pick_round_robin(),
                TeamPattern::Debate => self.pick_debate(&task),
                TeamPattern::FanOutFanIn => self.pick_fan_out_fan_in(&task),
                TeamPattern::ExpertPool => self.pick_expert_pool(&task),
                TeamPattern::HierarchicalDelegation => self.pick_hierarchical(&task),
            }?;
            member_id.id.clone()
        };
        if let Some(m) = self.members.iter_mut().find(|p| p.id == m_id) {
            m.task_count += 1;
        }
        self.completed_tasks.push(task);
        Some(m_id)
    }

    fn pick_supervisor(&self, task: &TeamTask) -> Option<&TeamMember> {
        if let Some(ref spec) = task.required_specialization {
            self.members
                .iter()
                .filter(|m| m.specialization == *spec)
                .max_by(|a, b| {
                    a.success_rate
                        .partial_cmp(&b.success_rate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        } else {
            self.members.iter().max_by(|a, b| {
                a.success_rate
                    .partial_cmp(&b.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        }
    }

    fn pick_pipeline(&self, _task: &TeamTask) -> Option<&TeamMember> {
        let idx = self.completed_tasks.len() % self.members.len().max(1);
        self.members.get(idx)
    }

    fn pick_round_robin(&mut self) -> Option<&TeamMember> {
        if self.members.is_empty() {
            return None;
        }
        let idx = self.round_robin_index % self.members.len();
        self.round_robin_index = self.round_robin_index.wrapping_add(1);
        self.members.get(idx)
    }

    fn pick_debate(&self, task: &TeamTask) -> Option<&TeamMember> {
        if let Some(ref spec) = task.required_specialization {
            let matches: Vec<&TeamMember> = self
                .members
                .iter()
                .filter(|m| m.specialization == *spec)
                .collect();
            if matches.len() >= 2 {
                matches
                    .get(self.completed_tasks.len() % matches.len())
                    .copied()
            } else {
                matches.first().copied()
            }
        } else {
            self.members.first()
        }
    }

    fn pick_fan_out_fan_in<'a>(&'a self, task: &TeamTask) -> Option<&'a TeamMember> {
        let matching: Vec<&TeamMember> = if let Some(ref spec) = task.required_specialization {
            self.members
                .iter()
                .filter(|m| m.specialization == *spec)
                .collect()
        } else {
            self.members.iter().collect()
        };
        if matching.is_empty() {
            return self.members.first();
        }
        matching
            .iter()
            .min_by(|a, b| a.task_count.cmp(&b.task_count))
            .copied()
    }

    fn pick_expert_pool<'a>(&'a self, task: &TeamTask) -> Option<&'a TeamMember> {
        if let Some(ref spec) = task.required_specialization {
            let exact: Vec<&TeamMember> = self
                .members
                .iter()
                .filter(|m| m.specialization == *spec)
                .collect();
            if !exact.is_empty() {
                return exact
                    .iter()
                    .max_by(|a, b| {
                        a.success_rate
                            .partial_cmp(&b.success_rate)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied();
            }
            let partial: Vec<&TeamMember> = self
                .members
                .iter()
                .filter(|m| m.specialization.contains(spec) || spec.contains(&m.specialization))
                .collect();
            if !partial.is_empty() {
                return partial
                    .iter()
                    .max_by(|a, b| {
                        a.confidence
                            .partial_cmp(&b.confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied();
            }
        }
        self.members.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn pick_hierarchical<'a>(&'a self, task: &TeamTask) -> Option<&'a TeamMember> {
        if self.members.is_empty() {
            return None;
        }
        if let Some(ref spec) = task.required_specialization {
            let matches: Vec<&TeamMember> = self
                .members
                .iter()
                .filter(|m| m.specialization == *spec)
                .collect();
            if !matches.is_empty() {
                return matches
                    .iter()
                    .max_by(|a, b| {
                        let a_score = a.hierarchical_level as f64 * 0.4 + a.success_rate * 0.6;
                        let b_score = b.hierarchical_level as f64 * 0.4 + b.success_rate * 0.6;
                        a_score
                            .partial_cmp(&b_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied();
            }
        }
        self.members
            .iter()
            .max_by(|a, b| a.hierarchical_level.cmp(&b.hierarchical_level))
    }

    pub fn record_outcome(&mut self, member_id: &str, success: bool) {
        if let Some(member) = self.members.iter_mut().find(|m| m.id == member_id) {
            let total = member.task_count as f64;
            let prev_successes = member.success_rate * (total - 1.0);
            member.success_rate = if total > 0.0 {
                (prev_successes + if success { 1.0 } else { 0.0 }) / total
            } else {
                0.0
            };
            member.confidence = member.success_rate * 0.7 + 0.3;
        }
    }

    pub fn team_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        summary.insert("member_count".to_string(), self.members.len() as f64);
        summary.insert(
            "completed_tasks".to_string(),
            self.completed_tasks.len() as f64,
        );
        summary.insert("pattern".to_string(), 0.0);
        if !self.members.is_empty() {
            let avg_success: f64 = self.members.iter().map(|m| m.success_rate).sum::<f64>()
                / self.members.len() as f64;
            summary.insert("avg_success_rate".to_string(), avg_success);
        }
        summary
    }

    pub fn diagnostic(&self) -> String {
        let s = self.team_summary();
        format!(
            "team:pattern={}|members={}|tasks={}|avg_success={:.2}|pool={}",
            self.pattern.name(),
            s.get("member_count").unwrap_or(&0.0),
            s.get("completed_tasks").unwrap_or(&0.0),
            s.get("avg_success_rate").unwrap_or(&0.0),
            self.pool_specialization,
        )
    }
}
