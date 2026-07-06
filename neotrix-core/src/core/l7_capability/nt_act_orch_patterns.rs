use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum AgentError {
    AgentNotFound(String),
    ExecutionFailed(String),
    Timeout(String),
    HandoffFailed(String),
    InvalidConfig(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::AgentNotFound(msg) => write!(f, "agent not found: {}", msg),
            AgentError::ExecutionFailed(msg) => write!(f, "execution failed: {}", msg),
            AgentError::Timeout(msg) => write!(f, "timeout: {}", msg),
            AgentError::HandoffFailed(msg) => write!(f, "handoff failed: {}", msg),
            AgentError::InvalidConfig(msg) => write!(f, "invalid config: {}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub agent_name: String,
    pub content: String,
    pub token_usage: u64,
    pub duration_ms: f64,
    pub confidence: f64,
    pub tool_calls: Vec<String>,
}

impl AgentOutput {
    pub fn new(agent_name: &str, content: &str) -> Self {
        AgentOutput {
            agent_name: agent_name.to_string(),
            content: content.to_string(),
            token_usage: 0,
            duration_ms: 0.0,
            confidence: 1.0,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgentContext {
    pub session_id: String,
    pub task: String,
    pub shared_state: HashMap<String, String>,
    pub parent_span: Option<String>,
}

pub trait AgentUnit: Send + Sync {
    fn name(&self) -> &str;
    fn role(&self) -> &str;
    fn execute(&self, input: &str, context: &AgentContext) -> Result<AgentOutput, AgentError>;
}

pub trait Orchestrator: Send + Sync {
    fn execute(&self, task: &str) -> Result<AgentOutput, AgentError>;
    fn pattern_name(&self) -> &str;
    fn stats(&self) -> OrchestratorStats;
}

#[derive(Debug, Clone, Default)]
pub struct OrchestratorStats {
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub total_tokens: u64,
    pub total_duration_ms: f64,
}

impl OrchestratorStats {
    fn record_success(&mut self, tokens: u64, duration_ms: f64) {
        self.total_tasks += 1;
        self.successful_tasks += 1;
        self.total_tokens += tokens;
        self.total_duration_ms += duration_ms;
    }

    fn record_failure(&mut self, duration_ms: f64) {
        self.total_tasks += 1;
        self.failed_tasks += 1;
        self.total_duration_ms += duration_ms;
    }
}

// ── Supervisor ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub max_rounds: usize,
    pub delegate_on_error: bool,
    pub require_all_complete: bool,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        SupervisorConfig {
            max_rounds: 5,
            delegate_on_error: true,
            require_all_complete: true,
        }
    }
}

pub struct SupervisorOrchestrator {
    supervisor: Box<dyn AgentUnit>,
    workers: Vec<Box<dyn AgentUnit>>,
    config: SupervisorConfig,
    stats: Mutex<OrchestratorStats>,
}

impl SupervisorOrchestrator {
    pub fn new(supervisor: Box<dyn AgentUnit>, config: SupervisorConfig) -> Self {
        SupervisorOrchestrator {
            supervisor,
            workers: Vec::new(),
            config,
            stats: Mutex::new(OrchestratorStats::default()),
        }
    }

    pub fn add_worker(&mut self, agent: Box<dyn AgentUnit>) {
        self.workers.push(agent);
    }
}

impl Orchestrator for SupervisorOrchestrator {
    fn execute(&self, task: &str) -> Result<AgentOutput, AgentError> {
        let start = Instant::now();
        let ctx = AgentContext {
            session_id: format!("supervisor-{}", task.len()),
            task: task.to_string(),
            shared_state: HashMap::new(),
            parent_span: None,
        };

        let plan = self.supervisor.execute(task, &ctx)?;

        if self.workers.is_empty() {
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_success(plan.token_usage, duration);
            return Ok(plan);
        }

        let mut results: Vec<AgentOutput> = Vec::new();
        for worker in &self.workers {
            let worker_input = if results.is_empty() {
                task
            } else {
                &plan.content
            };
            match worker.execute(worker_input, &ctx) {
                Ok(out) => results.push(out),
                Err(e) => {
                    if !self.config.delegate_on_error {
                        let duration = start.elapsed().as_secs_f64() * 1000.0;
                        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_failure(duration);
                        return Err(AgentError::ExecutionFailed(format!(
                            "worker failed: {}",
                            e
                        )));
                    }
                }
            }
        }

        if self.config.require_all_complete && results.len() < self.workers.len() {
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_failure(duration);
            return Err(AgentError::ExecutionFailed(
                "not all workers completed".into(),
            ));
        }

        let combined: String = results
            .iter()
            .map(|r| format!("{}: {}", r.agent_name, r.content))
            .collect::<Vec<_>>()
            .join("\n");
        let total_tokens: u64 = results.iter().map(|r| r.token_usage).sum();
        let duration = start.elapsed().as_secs_f64() * 1000.0;

        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_success(total_tokens, duration);

        Ok(AgentOutput {
            agent_name: self.supervisor.name().to_string(),
            content: combined,
            token_usage: total_tokens,
            duration_ms: duration,
            confidence: plan.confidence,
            tool_calls: plan.tool_calls,
        })
    }

    fn pattern_name(&self) -> &str {
        "supervisor"
    }

    fn stats(&self) -> OrchestratorStats {
        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).clone()
    }
}

// ── Swarm ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SwarmConfig {
    pub max_handoffs: usize,
    pub timeout_secs: u64,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        SwarmConfig {
            max_handoffs: 10,
            timeout_secs: 30,
        }
    }
}

pub struct SwarmOrchestrator {
    agents: Vec<Box<dyn AgentUnit>>,
    config: SwarmConfig,
    handoff_count: Mutex<usize>,
    stats: Mutex<OrchestratorStats>,
}

impl SwarmOrchestrator {
    pub fn new(config: SwarmConfig) -> Self {
        SwarmOrchestrator {
            agents: Vec::new(),
            config,
            handoff_count: Mutex::new(0),
            stats: Mutex::new(OrchestratorStats::default()),
        }
    }

    pub fn add_agent(&mut self, agent: Box<dyn AgentUnit>) {
        self.agents.push(agent);
    }

    pub fn handoff(&self, from: &str, to: &str, _context: &AgentContext) -> bool {
        let from_exists = self.agents.iter().any(|a| a.name() == from);
        let to_exists = self.agents.iter().any(|a| a.name() == to);
        if from_exists && to_exists {
            *self.handoff_count.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }) += 1;
            true
        } else {
            false
        }
    }
}

impl Orchestrator for SwarmOrchestrator {
    fn execute(&self, task: &str) -> Result<AgentOutput, AgentError> {
        if self.agents.is_empty() {
            return Err(AgentError::InvalidConfig("no agents in swarm".into()));
        }

        let start = Instant::now();
        let ctx = AgentContext {
            session_id: format!("swarm-{}", task.len()),
            task: task.to_string(),
            shared_state: HashMap::new(),
            parent_span: None,
        };

        let mut current_input = task.to_string();
        let mut current_idx = 0;
        let mut total_tokens = 0;
        let mut handoffs = 0;

        loop {
            let agent = &self.agents[current_idx];
            let result = agent.execute(&current_input, &ctx)?;

            total_tokens += result.token_usage;
            current_input = result.content.clone();

            handoffs += 1;
            if handoffs >= self.config.max_handoffs {
                let duration = start.elapsed().as_secs_f64() * 1000.0;
                self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_success(total_tokens, duration);
                return Ok(AgentOutput {
                    agent_name: agent.name().to_string(),
                    content: current_input,
                    token_usage: total_tokens,
                    duration_ms: duration,
                    confidence: result.confidence,
                    tool_calls: result.tool_calls,
                });
            }

            current_idx = (current_idx + 1) % self.agents.len();
        }
    }

    fn pattern_name(&self) -> &str {
        "swarm"
    }

    fn stats(&self) -> OrchestratorStats {
        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).clone()
    }
}

// ── Pipeline ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub stages: Vec<String>,
    pub fail_fast: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            stages: Vec::new(),
            fail_fast: true,
        }
    }
}

pub struct PipelineOrchestrator {
    agents: HashMap<String, Box<dyn AgentUnit>>,
    config: PipelineConfig,
    stats: Mutex<OrchestratorStats>,
}

impl PipelineOrchestrator {
    pub fn new(config: PipelineConfig) -> Self {
        PipelineOrchestrator {
            agents: HashMap::new(),
            config,
            stats: Mutex::new(OrchestratorStats::default()),
        }
    }

    pub fn add_stage(&mut self, name: &str, agent: Box<dyn AgentUnit>) {
        self.agents.insert(name.to_string(), agent);
    }
}

impl Orchestrator for PipelineOrchestrator {
    fn execute(&self, task: &str) -> Result<AgentOutput, AgentError> {
        if self.config.stages.is_empty() {
            return Err(AgentError::InvalidConfig("no stages defined".into()));
        }

        let start = Instant::now();
        let ctx = AgentContext {
            session_id: format!("pipeline-{}", task.len()),
            task: task.to_string(),
            shared_state: HashMap::new(),
            parent_span: None,
        };

        let mut current_input = task.to_string();
        let mut total_tokens = 0;
        let mut last_agent = String::new();
        let mut last_confidence = 1.0;
        let mut last_tool_calls = Vec::new();

        for stage_name in &self.config.stages {
            let agent = self
                .agents
                .get(stage_name)
                .ok_or_else(|| AgentError::AgentNotFound(stage_name.clone()))?;

            let result = agent.execute(&current_input, &ctx)?;

            total_tokens += result.token_usage;
            last_agent = result.agent_name.clone();
            last_confidence = result.confidence;
            last_tool_calls = result.tool_calls.clone();
            current_input = result.content;
        }

        let duration = start.elapsed().as_secs_f64() * 1000.0;
        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).record_success(total_tokens, duration);

        Ok(AgentOutput {
            agent_name: last_agent,
            content: current_input,
            token_usage: total_tokens,
            duration_ms: duration,
            confidence: last_confidence,
            tool_calls: last_tool_calls,
        })
    }

    fn pattern_name(&self) -> &str {
        "pipeline"
    }

    fn stats(&self) -> OrchestratorStats {
        self.stats.lock().unwrap_or_else(|e| { log::warn!("[orch] mutex poisoned: {}", e); e.into_inner() }).clone()
    }
}

// ── OrchestrationPattern ────────────────────────────────────

#[derive(Debug, Clone)]
pub enum OrchestrationPattern {
    Supervisor(SupervisorConfig),
    Swarm(SwarmConfig),
    Pipeline(PipelineConfig),
}

pub fn create_orchestrator(
    pattern: OrchestrationPattern,
    agents: Vec<Box<dyn AgentUnit>>,
) -> Result<Box<dyn Orchestrator>, AgentError> {
    match pattern {
        OrchestrationPattern::Supervisor(config) => {
            if agents.len() < 2 {
                return Err(AgentError::InvalidConfig(
                    "supervisor pattern requires at least 2 agents (1 supervisor + 1 worker)".into(),
                ));
            }
            let mut agents_iter = agents.into_iter();
            let supervisor = agents_iter.next().ok_or_else(|| AgentError::InvalidConfig(
                "supervisor pattern requires at least 2 agents (1 supervisor + 1 worker)".into(),
            ))?;
            let mut orch = SupervisorOrchestrator::new(supervisor, config);
            for worker in agents_iter {
                orch.add_worker(worker);
            }
            Ok(Box::new(orch))
        }
        OrchestrationPattern::Swarm(config) => {
            if agents.is_empty() {
                return Err(AgentError::InvalidConfig(
                    "swarm pattern requires at least 1 agent".into(),
                ));
            }
            let mut orch = SwarmOrchestrator::new(config);
            for agent in agents {
                orch.add_agent(agent);
            }
            Ok(Box::new(orch))
        }
        OrchestrationPattern::Pipeline(config) => {
            if config.stages.is_empty() {
                return Err(AgentError::InvalidConfig(
                    "pipeline pattern requires at least 1 stage".into(),
                ));
            }
            let mut orch = PipelineOrchestrator::new(config);
            for agent in agents {
                let name = agent.name().to_string();
                orch.add_stage(&name, agent);
            }
            Ok(Box::new(orch))
        }
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAgent {
        name: String,
        role: String,
        response: String,
        fail: bool,
    }

    impl MockAgent {
        fn new(name: &str, role: &str, response: &str) -> Self {
            MockAgent {
                name: name.to_string(),
                role: role.to_string(),
                response: response.to_string(),
                fail: false,
            }
        }

        fn new_failing(name: &str, role: &str) -> Self {
            MockAgent {
                name: name.to_string(),
                role: role.to_string(),
                response: String::new(),
                fail: true,
            }
        }
    }

    impl AgentUnit for MockAgent {
        fn name(&self) -> &str {
            &self.name
        }

        fn role(&self) -> &str {
            &self.role
        }

        fn execute(&self, input: &str, _context: &AgentContext) -> Result<AgentOutput, AgentError> {
            if self.fail {
                return Err(AgentError::ExecutionFailed(format!(
                    "{} failed on: {}",
                    self.name, input
                )));
            }
            Ok(AgentOutput {
                agent_name: self.name.clone(),
                content: format!("{} processed: {}", self.name, self.response),
                token_usage: 10,
                duration_ms: 1.0,
                confidence: 0.95,
                tool_calls: Vec::new(),
            })
        }
    }

    // ── Supervisor tests ──

    #[test]
    fn test_supervisor_basic() {
        let sup = Box::new(MockAgent::new("sup", "supervisor", "plan: use workers"));
        let w1 = Box::new(MockAgent::new("w1", "worker", "result A"));
        let w2 = Box::new(MockAgent::new("w2", "worker", "result B"));

        let mut orch = SupervisorOrchestrator::new(sup, SupervisorConfig::default());
        orch.add_worker(w1);
        orch.add_worker(w2);

        let result = orch.execute("test task").unwrap();
        assert_eq!(result.agent_name, "sup");
        assert!(result.content.contains("w1"));
        assert!(result.content.contains("w2"));
        assert_eq!(result.token_usage, 20);
    }

    #[test]
    fn test_supervisor_no_workers() {
        let sup = Box::new(MockAgent::new("sup", "supervisor", "direct result"));
        let orch = SupervisorOrchestrator::new(sup, SupervisorConfig::default());
        let result = orch.execute("task").unwrap();
        assert_eq!(result.agent_name, "sup");
    }

    #[test]
    fn test_supervisor_worker_error_not_delegate() {
        let sup = Box::new(MockAgent::new("sup", "supervisor", "plan"));
        let w1 = Box::new(MockAgent::new_failing("w1", "worker"));

        let mut orch = SupervisorOrchestrator::new(
            sup,
            SupervisorConfig {
                delegate_on_error: false,
                ..SupervisorConfig::default()
            },
        );
        orch.add_worker(w1);

        let result = orch.execute("task");
        assert!(result.is_err());
    }

    #[test]
    fn test_supervisor_worker_error_delegate() {
        let sup = Box::new(MockAgent::new("sup", "supervisor", "plan"));
        let w1 = Box::new(MockAgent::new_failing("w1", "worker"));
        let w2 = Box::new(MockAgent::new("w2", "worker", "result B"));

        let mut orch = SupervisorOrchestrator::new(
            sup,
            SupervisorConfig {
                delegate_on_error: true,
                require_all_complete: false,
                ..SupervisorConfig::default()
            },
        );
        orch.add_worker(w1);
        orch.add_worker(w2);

        let result = orch.execute("task").unwrap();
        assert_eq!(result.agent_name, "sup");
    }

    // ── Swarm tests ──

    #[test]
    fn test_swarm_basic() {
        let a1 = Box::new(MockAgent::new("a1", "agent", "from a1"));
        let a2 = Box::new(MockAgent::new("a2", "agent", "from a2"));
        let a3 = Box::new(MockAgent::new("a3", "agent", "from a3"));

        let mut orch = SwarmOrchestrator::new(SwarmConfig {
            max_handoffs: 3,
            ..SwarmConfig::default()
        });
        orch.add_agent(a1);
        orch.add_agent(a2);
        orch.add_agent(a3);

        let result = orch.execute("swarm task").unwrap();
        assert!(result.content.contains("from a3"));
        assert_eq!(result.token_usage, 30);
    }

    #[test]
    fn test_swarm_empty_agents() {
        let orch = SwarmOrchestrator::new(SwarmConfig::default());
        let result = orch.execute("task");
        assert!(matches!(result, Err(AgentError::InvalidConfig(_))));
    }

    #[test]
    fn test_swarm_handoff() {
        let a1 = Box::new(MockAgent::new("alice", "agent", "hello"));
        let a2 = Box::new(MockAgent::new("bob", "agent", "world"));

        let orch = SwarmOrchestrator::new(SwarmConfig::default());
        // handoff is only available through &self (not &mut) but we add agents via public API
        // we test via the stats check after adding agents
        let mut mutable = orch;
        mutable.add_agent(a1);
        mutable.add_agent(a2);

        let ctx = AgentContext::default();
        assert!(mutable.handoff("alice", "bob", &ctx));
        assert!(!mutable.handoff("alice", "nonexistent", &ctx));
        assert!(!mutable.handoff("nonexistent", "bob", &ctx));
    }

    // ── Pipeline tests ──

    #[test]
    fn test_pipeline_basic() {
        let a1 = Box::new(MockAgent::new("stage1", "parser", "parsed"));
        let a2 = Box::new(MockAgent::new("stage2", "analyzer", "analyzed"));
        let a3 = Box::new(MockAgent::new("stage3", "formatter", "formatted"));

        let config = PipelineConfig {
            stages: vec!["stage1".into(), "stage2".into(), "stage3".into()],
            fail_fast: true,
        };
        let mut orch = PipelineOrchestrator::new(config);
        orch.add_stage("stage1", a1);
        orch.add_stage("stage2", a2);
        orch.add_stage("stage3", a3);

        let result = orch.execute("raw input").unwrap();
        assert!(result.content.contains("stage3"));
        assert_eq!(result.token_usage, 30);
    }

    #[test]
    fn test_pipeline_agent_not_found() {
        let config = PipelineConfig {
            stages: vec!["missing".into()],
            fail_fast: true,
        };
        let orch = PipelineOrchestrator::new(config);
        let result = orch.execute("task");
        assert!(matches!(result, Err(AgentError::AgentNotFound(_))));
    }

    #[test]
    fn test_pipeline_no_stages() {
        let orch = PipelineOrchestrator::new(PipelineConfig::default());
        let result = orch.execute("task");
        assert!(matches!(result, Err(AgentError::InvalidConfig(_))));
    }

    // ── Factory tests ──

    #[test]
    fn test_factory_supervisor() {
        let agents: Vec<Box<dyn AgentUnit>> = vec![
            Box::new(MockAgent::new("sup", "supervisor", "plan")),
            Box::new(MockAgent::new("w1", "worker", "result")),
        ];
        let orch = create_orchestrator(
            OrchestrationPattern::Supervisor(SupervisorConfig::default()),
            agents,
        )
        .unwrap();
        assert_eq!(orch.pattern_name(), "supervisor");
        let result = orch.execute("task").unwrap();
        assert!(result.content.contains("w1"));
    }

    #[test]
    fn test_factory_swarm() {
        let agents: Vec<Box<dyn AgentUnit>> = vec![
            Box::new(MockAgent::new("a1", "agent", "x")),
            Box::new(MockAgent::new("a2", "agent", "y")),
        ];
        let config = SwarmConfig {
            max_handoffs: 2,
            ..SwarmConfig::default()
        };
        let orch =
            create_orchestrator(OrchestrationPattern::Swarm(config), agents).unwrap();
        assert_eq!(orch.pattern_name(), "swarm");
        let result = orch.execute("task").unwrap();
        assert_eq!(result.token_usage, 20);
    }

    #[test]
    fn test_factory_pipeline() {
        let agents: Vec<Box<dyn AgentUnit>> = vec![
            Box::new(MockAgent::new("s1", "stage", "out1")),
            Box::new(MockAgent::new("s2", "stage", "out2")),
        ];
        let config = PipelineConfig {
            stages: vec!["s1".into(), "s2".into()],
            fail_fast: true,
        };
        let orch = create_orchestrator(OrchestrationPattern::Pipeline(config), agents).unwrap();
        assert_eq!(orch.pattern_name(), "pipeline");
        let result = orch.execute("task").unwrap();
        assert_eq!(result.token_usage, 20);
    }

    #[test]
    fn test_factory_supervisor_too_few_agents() {
        let agents: Vec<Box<dyn AgentUnit>> = vec![Box::new(MockAgent::new("solo", "agent", "x"))];
        let result = create_orchestrator(
            OrchestrationPattern::Supervisor(SupervisorConfig::default()),
            agents,
        );
        assert!(matches!(result, Err(AgentError::InvalidConfig(_))));
    }

    #[test]
    fn test_factory_swarm_empty() {
        let result = create_orchestrator(
            OrchestrationPattern::Swarm(SwarmConfig::default()),
            Vec::new(),
        );
        assert!(matches!(result, Err(AgentError::InvalidConfig(_))));
    }

    #[test]
    fn test_factory_pipeline_no_stages() {
        let result = create_orchestrator(
            OrchestrationPattern::Pipeline(PipelineConfig::default()),
            Vec::new(),
        );
        assert!(matches!(result, Err(AgentError::InvalidConfig(_))));
    }

    // ── Stats tests ──

    #[test]
    fn test_supervisor_stats() {
        let sup = Box::new(MockAgent::new("sup", "supervisor", "plan"));
        let w1 = Box::new(MockAgent::new("w1", "worker", "a"));
        let mut orch = SupervisorOrchestrator::new(sup, SupervisorConfig::default());
        orch.add_worker(w1);

        orch.execute("task1").unwrap();
        orch.execute("task2").unwrap();

        let stats = orch.stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 2);
        assert!(stats.total_duration_ms > 0.0);
    }

    #[test]
    fn test_pipeline_single_stage() {
        let a1 = Box::new(MockAgent::new("only", "stage", "result"));
        let config = PipelineConfig {
            stages: vec!["only".into()],
            fail_fast: true,
        };
        let mut orch = PipelineOrchestrator::new(config);
        orch.add_stage("only", a1);
        let result = orch.execute("in").unwrap();
        assert!(result.content.contains("result"));
        assert_eq!(result.token_usage, 10);
    }
}
