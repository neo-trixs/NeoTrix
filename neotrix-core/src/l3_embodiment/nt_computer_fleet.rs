//! Cross-OS computer fleet management for NeoTrix.
//!
//! Integrates patterns from trycua/cua:
//! - Manage multiple NtComputer instances
//! - Unified action interface
//! - Cross-OS adapter for normalized behavior
//! - Fleet health monitoring

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::nt_computer::{ComputerError, NtComputer};

/// Fleet action types for unified computer interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FleetAction {
    Screenshot,
    ReadFile { path: String },
    RunCommand { cmd: String },
    Click { x: i32, y: i32 },
    Type { text: String },
    Scroll { delta: i32 },
    KeyPress { key: String },
}

/// Unified result type for fleet operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetResult {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
    pub computer_id: String,
    pub action: FleetAction,
}

/// Computer info for fleet discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerInfo {
    pub id: String,
    pub os: String,
    pub status: ComputerStatus,
    pub capabilities: Vec<String>,
}

/// Computer status in the fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputerStatus {
    Online,
    Offline,
    Busy,
    Error,
}

/// Cross-OS adapter for normalized behavior.
pub struct CrossOsAdapter {
    os_normalizations: HashMap<String, OsNormalization>,
}

#[derive(Debug, Clone)]
pub struct OsNormalization {
    pub path_separator: String,
    pub line_ending: String,
    pub shell: String,
    pub home_dir: String,
}

impl Default for CrossOsAdapter {
    fn default() -> Self {
        let mut os_normalizations = HashMap::new();

        os_normalizations.insert(
            "macos".into(),
            OsNormalization {
                path_separator: "/".into(),
                line_ending: "\n".into(),
                shell: "/bin/zsh".into(),
                home_dir: "~".into(),
            },
        );

        os_normalizations.insert(
            "linux".into(),
            OsNormalization {
                path_separator: "/".into(),
                line_ending: "\n".into(),
                shell: "/bin/bash".into(),
                home_dir: "~".into(),
            },
        );

        os_normalizations.insert(
            "windows".into(),
            OsNormalization {
                path_separator: "\\".into(),
                line_ending: "\r\n".into(),
                shell: "cmd.exe".into(),
                home_dir: "%USERPROFILE%".into(),
            },
        );

        Self { os_normalizations }
    }
}

impl CrossOsAdapter {
    pub fn normalize_path(&self, os: &str, path: &str) -> String {
        if let Some(norm) = self.os_normalizations.get(os) {
            path.replace('/', &norm.path_separator)
        } else {
            path.to_string()
        }
    }

    pub fn get_shell(&self, os: &str) -> &str {
        self.os_normalizations
            .get(os)
            .map(|n| n.shell.as_str())
            .unwrap_or("/bin/sh")
    }
}

/// Fleet manager for multiple NtComputer instances.
pub struct ComputerFleet {
    computers: HashMap<String, Box<dyn NtComputer>>,
    adapter: CrossOsAdapter,
}

impl ComputerFleet {
    pub fn new() -> Self {
        Self {
            computers: HashMap::new(),
            adapter: CrossOsAdapter::default(),
        }
    }

    /// Register a computer in the fleet.
    pub fn register(&mut self, id: String, computer: Box<dyn NtComputer>) {
        self.computers.insert(id, computer);
    }

    /// Get a computer by ID.
    pub fn get(&self, id: &str) -> Option<&dyn NtComputer> {
        self.computers.get(id).map(|c| c.as_ref())
    }

    /// Execute an action on a specific computer.
    pub fn execute_on(
        &mut self,
        computer_id: &str,
        action: FleetAction,
    ) -> Result<FleetResult, ComputerError> {
        let computer = self
            .computers
            .get_mut(computer_id)
            .ok_or_else(|| {
                ComputerError::Unsupported(format!("Computer {} not found", computer_id))
            })?;

        let result = match action {
            FleetAction::Screenshot => {
                let screen = computer.screenshot().ok_or_else(|| {
                    ComputerError::Unsupported("screenshot not supported".into())
                })?;
                FleetResult {
                    success: true,
                    data: Some(format!("{:?}", screen)),
                    error: None,
                    computer_id: computer_id.to_string(),
                    action: FleetAction::Screenshot,
                }
            }
            FleetAction::ReadFile { path } => {
                let file_result = computer.read_file(std::path::Path::new(&path))?;
                FleetResult {
                    success: true,
                    data: Some(String::from_utf8_lossy(&file_result.content).to_string()),
                    error: None,
                    computer_id: computer_id.to_string(),
                    action: FleetAction::ReadFile { path },
                }
            }
            FleetAction::RunCommand { cmd } => match computer.run_command(&cmd, &[]) {
                Ok(output) => FleetResult {
                    success: true,
                    data: Some(output.stdout),
                    error: None,
                    computer_id: computer_id.to_string(),
                    action: FleetAction::RunCommand { cmd },
                },
                Err(e) => FleetResult {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    computer_id: computer_id.to_string(),
                    action: FleetAction::RunCommand { cmd },
                },
            },
            _ => FleetResult {
                success: false,
                data: None,
                error: Some("Action not implemented".into()),
                computer_id: computer_id.to_string(),
                action,
            },
        };

        Ok(result)
    }

    /// List all available computers.
    pub fn list_available(&self) -> Vec<ComputerInfo> {
        self.computers
            .iter()
            .map(|(id, computer)| ComputerInfo {
                id: id.clone(),
                os: computer.system_info().os,
                status: ComputerStatus::Online,
                capabilities: vec!["screenshot".into(), "read_file".into(), "run_command".into()],
            })
            .collect()
    }

    /// Get the cross-OS adapter.
    pub fn adapter(&self) -> &CrossOsAdapter {
        &self.adapter
    }

    /// Get fleet size.
    pub fn len(&self) -> usize {
        self.computers.len()
    }

    /// Check if fleet is empty.
    pub fn is_empty(&self) -> bool {
        self.computers.is_empty()
    }
}

impl Default for ComputerFleet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_creation() {
        let fleet = ComputerFleet::new();
        assert!(fleet.is_empty());
    }

    #[test]
    fn test_cross_os_adapter() {
        let adapter = CrossOsAdapter::default();
        assert_eq!(adapter.normalize_path("macos", "/usr/bin"), "/usr/bin");
        assert_eq!(adapter.get_shell("macos"), "/bin/zsh");
    }
}
