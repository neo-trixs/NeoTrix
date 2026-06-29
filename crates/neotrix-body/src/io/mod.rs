//! # IOBus — Device Driver Model
//!
//! Architecture: Bus ← Device ← Driver trait
//! Inspired by Linux device model. Each IO capability is a Device
//! registered on the IOBus, activated through a Driver implementation.

use std::collections::HashMap;
use std::fmt;

/// Result type for IO operations
pub type IoResult<T> = Result<T, IoError>;

#[derive(Debug, Clone)]
pub enum IoError {
    DeviceNotFound(String),
    DeviceBusy(String),
    DriverError { device: String, reason: String },
    UnsupportedOperation(String),
    Timeout(String),
    Storage(String),
    NotFound(String),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceNotFound(d) => write!(f, "device not found: {d}"),
            Self::DeviceBusy(d) => write!(f, "device busy: {d}"),
            Self::DriverError { device, reason } => write!(f, "driver error on {device}: {reason}"),
            Self::UnsupportedOperation(op) => write!(f, "unsupported operation: {op}"),
            Self::Timeout(d) => write!(f, "timeout on {d}"),
            Self::Storage(msg) => write!(f, "storage error: {msg}"),
            Self::NotFound(id) => write!(f, "not found: {id}"),
        }
    }
}

/// Device identifier
pub type DeviceId = String;

/// Device capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceCapability {
    LlmCompletion,
    LlmEmbedding,
    McpRequest,
    LspDiagnostics,
    HttpRequest,
    WebSocket,
    FileRead,
    FileWrite,
    SearchQuery,
    VisionProcess,
    AudioProcess,
}

/// A registered IO device
#[derive(Debug, Clone)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub capabilities: Vec<DeviceCapability>,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Error(String),
}

/// IOBus — central registry for IO devices
#[derive(Debug, Clone)]
pub struct IOBus {
    devices: HashMap<DeviceId, Device>,
}

impl IOBus {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn register(&mut self, device: Device) {
        self.devices.insert(device.id.clone(), device);
    }

    pub fn unregister(&mut self, id: &str) -> Option<Device> {
        self.devices.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&Device> {
        self.devices.get(id)
    }

    pub fn find_by_capability(&self, cap: DeviceCapability) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.capabilities.contains(&cap))
            .collect()
    }

    pub fn set_status(&mut self, id: &str, status: DeviceStatus) {
        if let Some(device) = self.devices.get_mut(id) {
            device.status = status;
        }
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn all_online(&self) -> Vec<&Device> {
        self.devices.values().filter(|d| matches!(d.status, DeviceStatus::Online)).collect()
    }
}

impl Default for IOBus {
    fn default() -> Self {
        Self::new()
    }
}

pub mod llm;
pub mod mcp;
pub mod http;
pub mod filesystem;
pub mod llm_v2;
pub mod mcp_v2;
pub mod shutdown;
pub mod queue_persist;
pub mod doc_converter;
pub mod finance_pipeline;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iobus_register_device() {
        let mut bus = IOBus::new();
        bus.register(Device {
            id: "llm-1".into(),
            name: "GPT-4".into(),
            capabilities: vec![DeviceCapability::LlmCompletion],
            status: DeviceStatus::Online,
        });
        assert_eq!(bus.len(), 1);
        assert!(bus.get("llm-1").is_some());
    }

    #[test]
    fn test_iobus_find_by_capability() {
        let mut bus = IOBus::new();
        bus.register(Device {
            id: "llm-1".into(),
            name: "GPT-4".into(),
            capabilities: vec![DeviceCapability::LlmCompletion],
            status: DeviceStatus::Online,
        });
        bus.register(Device {
            id: "mcp-1".into(),
            name: "MCP Hub".into(),
            capabilities: vec![DeviceCapability::McpRequest],
            status: DeviceStatus::Online,
        });
        let found = bus.find_by_capability(DeviceCapability::LlmCompletion);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_iobus_unregister() {
        let mut bus = IOBus::new();
        bus.register(Device {
            id: "test".into(),
            name: "test".into(),
            capabilities: vec![],
            status: DeviceStatus::Online,
        });
        assert!(bus.unregister("test").is_some());
        assert!(bus.is_empty());
    }

    #[test]
    fn test_set_status() {
        let mut bus = IOBus::new();
        bus.register(Device {
            id: "test".into(),
            name: "test".into(),
            capabilities: vec![],
            status: DeviceStatus::Online,
        });
        bus.set_status("test", DeviceStatus::Offline);
        assert_eq!(bus.get("test").unwrap().status, DeviceStatus::Offline);
    }
}
