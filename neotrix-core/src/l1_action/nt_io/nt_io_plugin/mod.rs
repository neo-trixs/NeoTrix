use std::fmt;
use std::time::Instant;

pub mod registry;
pub mod builtin;
pub mod plugin_system;
pub mod capability_plugins;
#[cfg(feature = "sandbox")]
pub mod wasm;

pub use registry::PluginRegistry;
#[cfg(feature = "sandbox")]
pub use wasm::WasmPluginWrapper;

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
}

impl fmt::Display for PluginSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginSource::BuiltIn => write!(f, "built-in"),
            PluginSource::Wasm => write!(f, "wasm"),
            PluginSource::DynamicLib => write!(f, "dynamic-lib"),
        }
    }
}

/// Metadata about a registered plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub source: PluginSource,
    pub capability: &'static str,
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
    /// 底层状态（异常插件需恢复）；默认实现可在 struct 上覆盖。
    fn on_load(&self) -> Result<(), String>;
    fn on_unload(&self) -> Result<(), String>;
    fn on_event(&self, event: &PluginEvent) -> Result<(), String>;
    /// 允许把 `&dyn Plugin` 向下转成具体插件类型（由具体 impl 返回 `self`）。
    fn as_any(&self) -> &dyn std::any::Any;
    /// 外延能力类别，供 PluginRegistry 按 capability 分流。默认 generic。
    fn capability(&self) -> &'static str { "generic" }
}
