use std::fmt;
use std::time::Duration;

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

pub trait MemoryModule: Send + Sync {
    fn store(&mut self, key: &str, value: &[u8]);
    fn retrieve(&self, key: &str) -> Option<Vec<u8>>;
    fn capacity(&self) -> usize;
}

pub trait PlanningModule: Send + Sync {
    fn plan(&self, task: &TaskType) -> Vec<String>;
    fn evaluate_plan(&self, plan: &[String]) -> f64;
}

pub trait ActionModule: Send + Sync {
    fn execute(&self, action: &str) -> Result<Vec<u8>, Error>;
    fn rollback(&self, action: &str) -> Result<(), Error>;
}

pub trait ToolModule: Send + Sync {
    fn available_tools(&self) -> Vec<String>;
    fn invoke(&self, tool: &str, input: &[u8]) -> Result<Vec<u8>, Error>;
}

pub trait HarnessProtocol: Send + Sync {
    fn memory_module(&self) -> Box<dyn MemoryModule>;
    fn planning_module(&self) -> Box<dyn PlanningModule>;
    fn action_module(&self) -> Box<dyn ActionModule>;
    fn tool_module(&self) -> Box<dyn ToolModule>;
}

#[derive(Debug, Clone)]
pub struct HarnessRecord {
    pub task_type: TaskType,
    pub modules: HarnessModuleConfig,
    pub success_rate: f64,
    pub avg_duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct HarnessModuleConfig {
    pub memory_capacity: usize,
    pub planning_depth: usize,
    pub action_timeout: Duration,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HarnessConfig {
    pub memory_capacity: usize,
    pub planning_depth: usize,
    pub action_timeout: Duration,
    pub enabled_tools: Vec<String>,
    pub max_retries: usize,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            memory_capacity: 1024,
            planning_depth: 5,
            action_timeout: Duration::from_secs(30),
            enabled_tools: Vec::new(),
            max_retries: 3,
        }
    }
}

struct DefaultMemoryModule {
    capacity: usize,
    store: std::collections::HashMap<String, Vec<u8>>,
}

impl DefaultMemoryModule {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            store: std::collections::HashMap::new(),
        }
    }
}

impl MemoryModule for DefaultMemoryModule {
    fn store(&mut self, key: &str, value: &[u8]) {
        if self.store.len() < self.capacity {
            self.store.insert(key.to_string(), value.to_vec());
        }
    }

    fn retrieve(&self, key: &str) -> Option<Vec<u8>> {
        self.store.get(key).cloned()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

struct DefaultPlanningModule;

impl PlanningModule for DefaultPlanningModule {
    fn plan(&self, _task: &TaskType) -> Vec<String> {
        vec!["analyze".to_string(), "execute".to_string(), "verify".to_string()]
    }

    fn evaluate_plan(&self, _plan: &[String]) -> f64 {
        0.7
    }
}

struct DefaultActionModule;

impl ActionModule for DefaultActionModule {
    fn execute(&self, action: &str) -> Result<Vec<u8>, Error> {
        Ok(action.as_bytes().to_vec())
    }

    fn rollback(&self, _action: &str) -> Result<(), Error> {
        Ok(())
    }
}

struct DefaultToolModule;

impl ToolModule for DefaultToolModule {
    fn available_tools(&self) -> Vec<String> {
        vec!["default_tool".to_string()]
    }

    fn invoke(&self, tool: &str, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut output = tool.as_bytes().to_vec();
        output.extend_from_slice(input);
        Ok(output)
    }
}

struct DefaultHarness {
    config: HarnessConfig,
}

impl HarnessProtocol for DefaultHarness {
    fn memory_module(&self) -> Box<dyn MemoryModule> {
        Box::new(DefaultMemoryModule::new(self.config.memory_capacity))
    }

    fn planning_module(&self) -> Box<dyn PlanningModule> {
        Box::new(DefaultPlanningModule)
    }

    fn action_module(&self) -> Box<dyn ActionModule> {
        Box::new(DefaultActionModule)
    }

    fn tool_module(&self) -> Box<dyn ToolModule> {
        Box::new(DefaultToolModule)
    }
}

pub struct HarnessSynthesizer {
    archive: Vec<HarnessRecord>,
    config: HarnessConfig,
}

impl HarnessSynthesizer {
    pub fn new() -> Self {
        Self {
            archive: Vec::new(),
            config: HarnessConfig::default(),
        }
    }

    pub fn with_config(config: HarnessConfig) -> Self {
        Self {
            archive: Vec::new(),
            config,
        }
    }

    pub fn synthesize(&self, task: &TaskType) -> Box<dyn HarnessProtocol> {
        let _similar = self.find_similar(task);
        Box::new(DefaultHarness {
            config: self.config.clone(),
        })
    }

    pub fn self_repair(&mut self, failed: &dyn HarnessProtocol, error: &Error) {
        let _ = (failed, error);
        self.config.max_retries = self.config.max_retries.saturating_add(1);
    }

    pub fn record_outcome(&mut self, record: HarnessRecord) {
        self.archive.push(record);
    }

    pub fn archive(&self) -> &[HarnessRecord] {
        &self.archive
    }

    fn find_similar(&self, _task: &TaskType) -> Option<&HarnessRecord> {
        self.archive.first()
    }
}

impl Default for HarnessSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}
