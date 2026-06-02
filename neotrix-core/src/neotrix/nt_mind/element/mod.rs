pub mod bus;
pub mod registry;
pub mod capability_element;
pub mod memory_element;
pub mod skill_element;

use std::any::Any;
use std::fmt::Debug;

pub type ElementId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementType {
    Core,
    Feature,
    Monitor,
    Network,
    Ui,
}

#[derive(Debug, Clone)]
pub enum ElementError {
    InitFailed(String),
    StartFailed(String),
    StopFailed(String),
    DestroyFailed(String),
    DependencyNotMet(String),
    VersionMismatch { element: String, required: String, found: String },
    BusError(String),
    RuntimeError(String),
    NotFound(String),
    AlreadyRegistered(String),
}

impl std::fmt::Display for ElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementError::InitFailed(msg) => write!(f, "init failed: {}", msg),
            ElementError::StartFailed(msg) => write!(f, "start failed: {}", msg),
            ElementError::StopFailed(msg) => write!(f, "stop failed: {}", msg),
            ElementError::DestroyFailed(msg) => write!(f, "destroy failed: {}", msg),
            ElementError::DependencyNotMet(msg) => write!(f, "dependency not met: {}", msg),
            ElementError::VersionMismatch { element, required, found } => {
                write!(f, "version mismatch for {}: required {} found {}", element, required, found)
            }
            ElementError::BusError(msg) => write!(f, "bus error: {}", msg),
            ElementError::RuntimeError(msg) => write!(f, "runtime error: {}", msg),
            ElementError::NotFound(id) => write!(f, "element not found: {}", id),
            ElementError::AlreadyRegistered(id) => write!(f, "element already registered: {}", id),
        }
    }
}

impl std::error::Error for ElementError {}

#[derive(Debug, Clone)]
pub struct CapabilityAccess {
    pub name: &'static str,
    pub description: &'static str,
    pub operations: Vec<CapabilityOp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityOp {
    Query,
    Command,
    Subscribe,
    Provide,
}

pub trait Element: Debug + Send + Sync + Any + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn element_type(&self) -> ElementType;

    fn init(&mut self, bus: &bus::ElementBus) -> Result<(), ElementError>;
    fn start(&mut self) -> Result<(), ElementError>;
    fn stop(&mut self) -> Result<(), ElementError>;
    fn destroy(&mut self) -> Result<(), ElementError>;

    fn depends_on(&self) -> Vec<&str> {
        vec![]
    }

    fn provides(&self) -> Vec<CapabilityAccess> {
        vec![]
    }

    fn state_version(&self) -> u32 {
        1
    }

    fn compatible_with(&self, _other_version: &str) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
