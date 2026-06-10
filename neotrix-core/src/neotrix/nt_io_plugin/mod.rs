use std::fmt;
use std::time::Instant;

pub mod discovery;
pub mod registry;
pub mod builtin;
#[cfg(feature = "sandbox")]
pub mod wasm;

pub use registry::PluginRegistry;
#[cfg(feature = "sandbox")]
pub use wasm::WasmPluginWrapper;
pub use discovery::DiscoveredSkill;

/// Events dispatched to all registered plugins.
#[derive(Debug, Clone)]
pub enum PluginEvent {
    ConfigChanged,
    SessionStarted,
    SessionEnded,
    TaskReceived(String),
    TaskCompleted(String),
    BrainTick,
    Shutdown,
}

impl fmt::Display for PluginEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginEvent::ConfigChanged => write!(f, "ConfigChanged"),
            PluginEvent::SessionStarted => write!(f, "SessionStarted"),
            PluginEvent::SessionEnded => write!(f, "SessionEnded"),
            PluginEvent::TaskReceived(task) => write!(f, "TaskReceived({})", task),
            PluginEvent::TaskCompleted(task) => write!(f, "TaskCompleted({})", task),
            PluginEvent::BrainTick => write!(f, "BrainTick"),
            PluginEvent::Shutdown => write!(f, "Shutdown"),
        }
    }
}

/// Where the plugin originates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSource {
    BuiltIn,
    Wasm,
    DynamicLib,
    SkillMd,
    SkillJson,
}

impl fmt::Display for PluginSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginSource::BuiltIn => write!(f, "built-in"),
            PluginSource::Wasm => write!(f, "wasm"),
            PluginSource::DynamicLib => write!(f, "dynamic-lib"),
            PluginSource::SkillMd => write!(f, "skill-md"),
            PluginSource::SkillJson => write!(f, "skill-json"),
        }
    }
}

/// Metadata about a registered plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub source: PluginSource,
    pub loaded_at: Instant,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Loaded,
    Unloaded,
    Error(String),
}

impl fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginStatus::Loaded => write!(f, "loaded"),
            PluginStatus::Unloaded => write!(f, "unloaded"),
            PluginStatus::Error(e) => write!(f, "error({})", e),
        }
    }
}

/// Core trait for all NeoTrix plugins.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn on_load(&self) -> Result<(), String>;
    fn on_unload(&self) -> Result<(), String>;
    fn on_event(&self, event: &PluginEvent) -> Result<(), String>;
}
