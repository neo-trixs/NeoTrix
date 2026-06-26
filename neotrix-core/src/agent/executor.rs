use crate::agent::blackboard::Blackboard;
use crate::agent::cognitive_memory::CognitiveMemory;
use crate::agent::memory_optimizer::BanditScorer;
use crate::agent::playbook::PlaybookEngine;
use crate::agent::step_generator::StepGenerator;
use crate::agent::sub_agent::SubAgentPool;
use crate::agent::worktree::WorktreeManager;
use crate::core::KnowledgeSource;
use crate::neotrix::nt_mind::{ReasoningBrain, SelfIteratingBrain};

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AgentStatus {
    Idle,
    Running,
    Thinking,
    WaitingForTool,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub message: String,
    pub status: AgentStatus,
}

pub struct Agent {
    pub brain: ReasoningBrain,
    pub iterating_brain: Option<SelfIteratingBrain>,
    pub status: AgentStatus,
    // 新模块集成
    pub sub_agent_pool: Option<SubAgentPool>,
    pub blackboard: Option<Blackboard>,
    pub cognitive_memory: Option<CognitiveMemory>,
    pub playbook_engine: Option<PlaybookEngine>,
    pub bandit_scorer: Option<BanditScorer>,
    pub step_generator: Option<StepGenerator>,
    pub worktree_manager: Option<WorktreeManager>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            brain: ReasoningBrain::new(),
            iterating_brain: None,
            status: AgentStatus::Idle,
            sub_agent_pool: None,
            blackboard: None,
            cognitive_memory: None,
            playbook_engine: None,
            bandit_scorer: None,
            step_generator: None,
            worktree_manager: None,
        }
    }

    pub fn with_brain(brain: ReasoningBrain) -> Self {
        Self {
            brain,
            iterating_brain: None,
            status: AgentStatus::Idle,
            sub_agent_pool: None,
            blackboard: None,
            cognitive_memory: None,
            playbook_engine: None,
            bandit_scorer: None,
            step_generator: None,
            worktree_manager: None,
        }
    }

    pub fn absorb(&mut self, source: KnowledgeSource) {
        self.brain.absorb(source);
    }

    pub fn status(&self) -> &AgentStatus {
        &self.status
    }

    pub fn with_sub_agent_pool(mut self, pool: SubAgentPool) -> Self {
        self.sub_agent_pool = Some(pool);
        self
    }

    pub fn with_blackboard(mut self, board: Blackboard) -> Self {
        self.blackboard = Some(board);
        self
    }

    pub fn with_cognitive_memory(mut self, mem: CognitiveMemory) -> Self {
        self.cognitive_memory = Some(mem);
        self
    }

    pub fn with_playbook_engine(mut self, engine: PlaybookEngine) -> Self {
        self.playbook_engine = Some(engine);
        self
    }

    pub fn with_bandit_scorer(mut self, scorer: BanditScorer) -> Self {
        self.bandit_scorer = Some(scorer);
        self
    }

    pub fn with_worktree_manager(mut self, mgr: WorktreeManager) -> Self {
        self.worktree_manager = Some(mgr);
        self
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::sub_agent::SubAgentConfig;
    use crate::core::KnowledgeSource;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new();
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(agent.iterating_brain.is_none());
        assert!(agent.sub_agent_pool.is_none());
        assert!(agent.blackboard.is_none());
        assert!(agent.cognitive_memory.is_none());
        assert!(agent.playbook_engine.is_none());
        assert!(agent.bandit_scorer.is_none());
        assert!(agent.step_generator.is_none());
        assert!(agent.worktree_manager.is_none());
    }

    #[test]
    fn test_agent_with_brain() {
        let brain = ReasoningBrain::new();
        let agent = Agent::with_brain(brain);
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_agent_default() {
        let agent = Agent::default();
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_agent_absorb() {
        let mut agent = Agent::new();
        agent.absorb(KnowledgeSource::HeroUI);
        let cap_sum: f64 = agent.brain.capability.arr.iter().sum();
        assert!(cap_sum > 0.0);
    }

    #[test]
    fn test_agent_absorb_multiple_sources() {
        let mut agent = Agent::new();
        agent.absorb(KnowledgeSource::HeroUI);
        agent.absorb(KnowledgeSource::BaseUI);
        agent.absorb(KnowledgeSource::ArcUI);
        let cap_sum: f64 = agent.brain.capability.arr.iter().sum();
        assert!(cap_sum > 0.0);
    }

    #[test]
    fn test_agent_status_reflects_idle() {
        let agent = Agent::new();
        assert_eq!(agent.status(), &AgentStatus::Idle);
    }

    #[test]
    fn test_agent_builder_with_sub_agent_pool() {
        let config = SubAgentConfig {
            max_concurrency: 5,
            ..Default::default()
        };
        let pool = SubAgentPool::new(config);
        let agent = Agent::new().with_sub_agent_pool(pool);
        assert!(agent.sub_agent_pool.is_some());
    }

    #[test]
    fn test_agent_builder_with_blackboard() {
        let board = Blackboard::new(100);
        let agent = Agent::new().with_blackboard(board);
        assert!(agent.blackboard.is_some());
    }

    #[test]
    fn test_agent_builder_with_cognitive_memory() {
        let mem = CognitiveMemory::new();
        let agent = Agent::new().with_cognitive_memory(mem);
        assert!(agent.cognitive_memory.is_some());
    }

    #[test]
    fn test_agent_builder_with_playbook_engine() {
        let engine = PlaybookEngine::new(10, 0.5);
        let agent = Agent::new().with_playbook_engine(engine);
        assert!(agent.playbook_engine.is_some());
    }

    #[test]
    fn test_agent_builder_with_bandit_scorer() {
        let scorer = BanditScorer::new(5.0);
        let agent = Agent::new().with_bandit_scorer(scorer);
        assert!(agent.bandit_scorer.is_some());
    }

    #[test]
    fn test_agent_builder_with_worktree_manager() {
        let mgr = WorktreeManager::new(std::path::Path::new("test_worktree"));
        let agent = Agent::new().with_worktree_manager(mgr);
        assert!(agent.worktree_manager.is_some());
    }

    #[test]
    fn test_agent_full_builder_chain() {
        let agent = Agent::new()
            .with_sub_agent_pool(SubAgentPool::new(SubAgentConfig {
                max_concurrency: 3,
                ..Default::default()
            }))
            .with_blackboard(Blackboard::new(100))
            .with_cognitive_memory(CognitiveMemory::new())
            .with_playbook_engine(PlaybookEngine::new(10, 0.5))
            .with_bandit_scorer(BanditScorer::new(5.0))
            .with_worktree_manager(WorktreeManager::new(std::path::Path::new("full_test")));

        assert!(agent.sub_agent_pool.is_some());
        assert!(agent.blackboard.is_some());
        assert!(agent.cognitive_memory.is_some());
        assert!(agent.playbook_engine.is_some());
        assert!(agent.bandit_scorer.is_some());
        assert!(agent.worktree_manager.is_some());
    }

    #[test]
    fn test_agent_absorb_normalizes_capability() {
        let mut agent = Agent::new();
        for _ in 0..10 {
            agent.absorb(KnowledgeSource::HeroUI);
        }
        let max_val = agent
            .brain
            .capability
            .arr
            .iter()
            .cloned()
            .fold(0.0f64, |acc, x| acc.max(x));
        assert!(
            max_val <= 1.0 + 1e-9,
            "max norm should be ≤ 1.0, got {}",
            max_val
        );
    }

    #[test]
    fn test_agent_status_clone_and_debug() {
        let s1 = AgentStatus::Idle;
        let s2 = AgentStatus::Running;
        let s3 = AgentStatus::Thinking;
        let s4 = AgentStatus::WaitingForTool;
        let s5 = AgentStatus::Error("msg".into());
        let _ = format!("{:?}", s1);
        let _ = format!("{:?}", s2);
        let _ = format!("{:?}", s3);
        let _ = format!("{:?}", s4);
        let _ = format!("{:?}", s5);
        assert_eq!(s1.clone(), AgentStatus::Idle);
        assert_eq!(s5.clone(), AgentStatus::Error("msg".into()));
    }

    #[test]
    fn test_agent_status_partial_eq() {
        assert_eq!(AgentStatus::Idle, AgentStatus::Idle);
        assert_ne!(AgentStatus::Idle, AgentStatus::Running);
        assert_eq!(
            AgentStatus::Error("a".into()),
            AgentStatus::Error("a".into())
        );
        assert_ne!(
            AgentStatus::Error("a".into()),
            AgentStatus::Error("b".into())
        );
    }
}
