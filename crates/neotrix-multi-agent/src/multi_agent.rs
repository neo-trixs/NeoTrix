//! Multi-agent coordination — delegation, scheduling, and lifecycle.

/// Role an agent can play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Researcher,
    Coder,
    Reviewer,
    Coordinator,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::Researcher => "researcher",
            AgentRole::Coder => "coder",
            AgentRole::Reviewer => "reviewer",
            AgentRole::Coordinator => "coordinator",
        }
    }
}

/// Status of an agent instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Failed,
    Completed,
}

/// An agent instance in the multi-agent system.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub task_count: usize,
}

impl Agent {
    pub fn new(id: &str, role: AgentRole) -> Self {
        Self {
            id: id.to_string(),
            role,
            status: AgentStatus::Idle,
            task_count: 0,
        }
    }

    pub fn start(&mut self) {
        self.status = AgentStatus::Running;
    }

    pub fn complete(&mut self) {
        self.status = AgentStatus::Completed;
        self.task_count += 1;
    }

    pub fn fail(&mut self) {
        self.status = AgentStatus::Failed;
    }

    pub fn reset(&mut self) {
        self.status = AgentStatus::Idle;
    }

    pub fn is_available(&self) -> bool {
        self.status == AgentStatus::Idle || self.status == AgentStatus::Completed
    }
}

/// A multi-agent crew.
#[derive(Debug, Clone)]
pub struct AgentCrew {
    pub agents: Vec<Agent>,
    pub name: String,
}

impl AgentCrew {
    pub fn new(name: &str) -> Self {
        Self {
            agents: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    pub fn available_agents(&self) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.is_available()).collect()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn running_count(&self) -> usize {
        self.agents.iter().filter(|a| a.status == AgentStatus::Running).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_names() {
        assert_eq!(AgentRole::Researcher.name(), "researcher");
        assert_eq!(AgentRole::Coder.name(), "coder");
        assert_eq!(AgentRole::Reviewer.name(), "reviewer");
        assert_eq!(AgentRole::Coordinator.name(), "coordinator");
    }

    #[test]
    fn test_agent_new() {
        let agent = Agent::new("a1", AgentRole::Coder);
        assert_eq!(agent.id, "a1");
        assert_eq!(agent.role, AgentRole::Coder);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.task_count, 0);
    }

    #[test]
    fn test_agent_lifecycle() {
        let mut agent = Agent::new("a1", AgentRole::Researcher);
        assert!(agent.is_available());
        agent.start();
        assert_eq!(agent.status, AgentStatus::Running);
        assert!(!agent.is_available());
        agent.complete();
        assert_eq!(agent.status, AgentStatus::Completed);
        assert_eq!(agent.task_count, 1);
        assert!(agent.is_available());
    }

    #[test]
    fn test_agent_fail_and_reset() {
        let mut agent = Agent::new("a1", AgentRole::Reviewer);
        agent.start();
        agent.fail();
        assert_eq!(agent.status, AgentStatus::Failed);
        assert!(!agent.is_available());
        agent.reset();
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(agent.is_available());
    }

    #[test]
    fn test_crew_new() {
        let crew = AgentCrew::new("alpha");
        assert_eq!(crew.name, "alpha");
        assert!(crew.agents.is_empty());
    }

    #[test]
    fn test_crew_add_agent() {
        let mut crew = AgentCrew::new("beta");
        crew.add_agent(Agent::new("a1", AgentRole::Coder));
        crew.add_agent(Agent::new("a2", AgentRole::Reviewer));
        assert_eq!(crew.agent_count(), 2);
    }

    #[test]
    fn test_crew_available_agents() {
        let mut crew = AgentCrew::new("gamma");
        crew.add_agent(Agent::new("a1", AgentRole::Coder));
        crew.add_agent(Agent::new("a2", AgentRole::Reviewer));
        crew.agents[0].start();
        let available = crew.available_agents();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "a2");
    }

    #[test]
    fn test_crew_running_count() {
        let mut crew = AgentCrew::new("delta");
        crew.add_agent(Agent::new("a1", AgentRole::Coder));
        crew.add_agent(Agent::new("a2", AgentRole::Reviewer));
        crew.agents[0].start();
        crew.agents[1].start();
        assert_eq!(crew.running_count(), 2);
        crew.agents[0].complete();
        assert_eq!(crew.running_count(), 1);
    }

    #[test]
    fn test_agent_is_available_completed() {
        let mut agent = Agent::new("a1", AgentRole::Coordinator);
        agent.start();
        agent.complete();
        assert!(agent.is_available());
    }
}
