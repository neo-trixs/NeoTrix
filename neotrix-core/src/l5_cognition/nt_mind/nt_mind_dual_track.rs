use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    Reasoning { complexity: f64 },
    Perception { target: String },
    Action { target: String },
    Hybrid { components: Vec<String> },
}

pub struct Error {
    pub message: String,
    pub recoverable: bool,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait Action: Send + Sync {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>, Error>;
    fn id(&self) -> &str;
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn invoke(&self, input: &[u8]) -> Result<Vec<u8>, Error>;
}

pub struct AgentCrew {
    pub agents: Vec<AgentRole>,
    pub process: Process,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Process {
    Sequential,
    Hierarchical,
}

pub struct AgentRole {
    pub role: String,
    pub goal: String,
    pub backstory: String,
    pub tools: Vec<Box<dyn Tool>>,
}

pub struct StepId(pub usize);

pub struct Condition(pub String);

pub struct Workflow {
    pub steps: Vec<FlowStep>,
}

pub struct FlowStep {
    pub action: Box<dyn Action>,
    pub next: Vec<(Condition, StepId)>,
}

#[derive(Debug, Clone)]
pub struct NoveltyComplexityMatrix {
    pub novelty: f64,
    pub complexity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    Crew,
    Flow,
}

pub struct DualTrackRouter {
    novelty_complexity: NoveltyComplexityMatrix,
}

impl DualTrackRouter {
    pub fn new(novelty: f64, complexity: f64) -> Self {
        Self {
            novelty_complexity: NoveltyComplexityMatrix { novelty, complexity },
        }
    }

    pub fn route(&self, _task: &TaskType) -> Track {
        let n = self.novelty_complexity.novelty;
        let c = self.novelty_complexity.complexity;

        match (n > 0.5, c > 0.5) {
            (false, false) => Track::Flow,
            (false, true) => Track::Crew,
            (true, false) => Track::Flow,
            (true, true) => Track::Crew,
        }
    }

    pub fn novelty_complexity(&self) -> &NoveltyComplexityMatrix {
        &self.novelty_complexity
    }

    pub fn update_scores(&mut self, novelty: f64, complexity: f64) {
        self.novelty_complexity.novelty = novelty.clamp(0.0, 1.0);
        self.novelty_complexity.complexity = complexity.clamp(0.0, 1.0);
    }
}

impl Default for DualTrackRouter {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

pub struct CrewRunner {
    crew: AgentCrew,
}

impl CrewRunner {
    pub fn new(crew: AgentCrew) -> Self {
        Self { crew }
    }

    pub fn run(&self, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut current = input.to_vec();
        for agent in &self.crew.agents {
            let _ = &agent.role;
            current = current;
        }
        Ok(current)
    }

    pub fn agent_count(&self) -> usize {
        self.crew.agents.len()
    }
}

pub struct FlowRunner {
    workflow: Workflow,
}

impl FlowRunner {
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow }
    }

    pub fn run(&self, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut current = input.to_vec();
        let mut idx = 0;
        while idx < self.workflow.steps.len() {
            let step = &self.workflow.steps[idx];
            current = step.action.execute(&current)?;
            if step.next.is_empty() {
                break;
            }
            idx = step.next[0].1 .0;
        }
        Ok(current)
    }

    pub fn step_count(&self) -> usize {
        self.workflow.steps.len()
    }
}
