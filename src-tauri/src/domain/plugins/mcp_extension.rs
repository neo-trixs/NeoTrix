//! # MCP Extension Domain Plugin
//!
//! Implements Claude's `.mcpb` / `.ntb` pattern: portable MCP packages
//! that bundle tools, prompts, and resources into a shareable format.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         MCP Extension                        │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Package │  │  Registry│  │  Runtime │  │
//! │  │  Builder │  │          │  │  Loader  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      .ntb Package Format             │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```

use async_trait::async_trait;
use crate::domain::app_handle::{set_app_handle, get_app_handle};
use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ========== Types ==========

/// Package manifest format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package name (e.g., "neo-tools")
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// License
    pub license: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Keywords
    pub keywords: Vec<String>,
    /// Min NeoTrix version required
    pub min_neotrix_version: Option<String>,
    /// Dependencies (other .ntb packages)
    pub dependencies: HashMap<String, String>,
    /// Entry point script (e.g., "server.js", "main.py")
    pub entry_point: String,
    /// Runtime ("node", "python", "rust", "wasm")
    pub runtime: String,
    /// Environment variables required
    pub env_vars: Vec<EnvVar>,
    /// MCP tools provided
    pub tools: Vec<McpTool>,
    /// MCP prompts provided
    pub prompts: Vec<McpPrompt>,
    /// MCP resources provided
    pub resources: Vec<McpResource>,
}

/// Environment variable requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub annotations: HashMap<String, String>,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArg>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
}

/// Installed package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub manifest: PackageManifest,
    pub install_path: String,
    pub installed_at: String,
    pub enabled: bool,
    pub running: bool,
    pub pid: Option<u32>,
    pub config: HashMap<String, serde_json::Value>,
}

/// Package search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub stars: u64,
    pub verified: bool,
    pub installed: bool,
}

/// Extension statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStats {
    pub total_installed: usize,
    pub enabled: usize,
    pub running: usize,
    pub total_tools: usize,
    pub total_prompts: usize,
    pub total_resources: usize,
}

// ========== Plugin ==========

/// MCP Extension Domain Plugin
pub struct McpExtensionPlugin {
    state: Arc<Mutex<ExtensionState>>,
}

struct ExtensionState {
    installed: HashMap<String, InstalledPackage>,
    registry: Vec<PackageResult>,
}

impl McpExtensionPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ExtensionState {
                installed: HashMap::new(),
                registry: Vec::new(),
            })),
        }
    }

    /// Install a package
    fn install(&self, name: &str, version: Option<&str>) -> Result<InstalledPackage, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let manifest = PackageManifest {
            name: name.to_string(),
            version: version.unwrap_or("0.1.0").to_string(),
            description: format!("{} MCP extension package", name),
            author: "NeoTrix Community".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            min_neotrix_version: None,
            dependencies: HashMap::new(),
            entry_point: "server.js".to_string(),
            runtime: "node".to_string(),
            env_vars: vec![],
            tools: vec![McpTool {
                name: format!("{}_tool", name.replace('-', "_")),
                description: format!("Tool provided by {}", name),
                input_schema: serde_json::json!({"type": "object", "properties": {}}),
                annotations: HashMap::new(),
            }],
            prompts: vec![],
            resources: vec![],
        };

        let package = InstalledPackage {
            manifest,
            install_path: format!("~/.neotrix/extensions/{}", name),
            installed_at: chrono::Utc::now().to_rfc3339(),
            enabled: true,
            running: false,
            pid: None,
            config: HashMap::new(),
        };

        state.installed.insert(name.to_string(), package.clone());
        Ok(package)
    }

    /// Uninstall a package
    fn uninstall(&self, name: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        state.installed.remove(name);
        Ok(())
    }

    /// Enable/disable a package
    fn toggle(&self, name: &str, enabled: bool) -> Result<InstalledPackage, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let package = state.installed.get_mut(name)
            .ok_or_else(|| DomainError {
                code: "PACKAGE_NOT_FOUND".into(),
                message: format!("Package '{}' not found", name),
                recoverable: true,
            })?;

        package.enabled = enabled;
        Ok(package.clone())
    }

    /// Start/stop a package
    fn set_running(&self, name: &str, running: bool) -> Result<InstalledPackage, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let package = state.installed.get_mut(name)
            .ok_or_else(|| DomainError {
                code: "PACKAGE_NOT_FOUND".into(),
                message: format!("Package '{}' not found", name),
                recoverable: true,
            })?;

        package.running = running;
        if running {
            package.pid = Some(12345); // Simplified
        } else {
            package.pid = None;
        }

        Ok(package.clone())
    }

    /// List installed packages
    fn list_installed(&self) -> Result<Vec<InstalledPackage>, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        Ok(state.installed.values().cloned().collect())
    }

    /// Search registry
    fn search(&self, query: &str) -> Result<Vec<PackageResult>, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        // Simplified — would search actual registry
        let results: Vec<PackageResult> = state.registry.iter()
            .filter(|p| p.name.contains(query) || p.description.contains(query))
            .cloned()
            .collect();

        Ok(results)
    }

    /// Get stats
    fn get_stats(&self) -> Result<ExtensionStats, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let installed = state.installed.len();
        let enabled = state.installed.values().filter(|p| p.enabled).count();
        let running = state.installed.values().filter(|p| p.running).count();
        let tools: usize = state.installed.values()
            .filter(|p| p.enabled)
            .map(|p| p.manifest.tools.len())
            .sum();
        let prompts: usize = state.installed.values()
            .filter(|p| p.enabled)
            .map(|p| p.manifest.prompts.len())
            .sum();
        let resources: usize = state.installed.values()
            .filter(|p| p.enabled)
            .map(|p| p.manifest.resources.len())
            .sum();

        Ok(ExtensionStats {
            total_installed: installed,
            enabled,
            running,
            total_tools: tools,
            total_prompts: prompts,
            total_resources: resources,
        })
    }
}

impl Default for McpExtensionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainPlugin for McpExtensionPlugin {
    fn name(&self) -> &str {
        "mcp_extension"
    }

    fn description(&self) -> &str {
        "MCP package management — install, configure, and run .ntb extension packages"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "install".into(),
                description: "Install an MCP extension package".into(),
                params: vec![
                    ParamSpec { name: "name".into(), typ: "string".into(), required: true, description: "Package name".into() },
                    ParamSpec { name: "version".into(), typ: "string".into(), required: false, description: "Version (default: latest)".into() },
                ],
            },
            ActionSpec {
                name: "uninstall".into(),
                description: "Uninstall an MCP extension package".into(),
                params: vec![ParamSpec { name: "name".into(), typ: "string".into(), required: true, description: "Package name".into() }],
            },
            ActionSpec {
                name: "toggle".into(),
                description: "Enable or disable an installed package".into(),
                params: vec![
                    ParamSpec { name: "name".into(), typ: "string".into(), required: true, description: "Package name".into() },
                    ParamSpec { name: "enabled".into(), typ: "boolean".into(), required: true, description: "Enable or disable".into() },
                ],
            },
            ActionSpec {
                name: "start".into(),
                description: "Start a package runtime".into(),
                params: vec![ParamSpec { name: "name".into(), typ: "string".into(), required: true, description: "Package name".into() }],
            },
            ActionSpec {
                name: "stop".into(),
                description: "Stop a package runtime".into(),
                params: vec![ParamSpec { name: "name".into(), typ: "string".into(), required: true, description: "Package name".into() }],
            },
            ActionSpec {
                name: "list".into(),
                description: "List installed packages".into(),
                params: vec![],
            },
            ActionSpec {
                name: "search".into(),
                description: "Search the package registry".into(),
                params: vec![ParamSpec { name: "query".into(), typ: "string".into(), required: true, description: "Search query".into() }],
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "Get extension statistics".into(),
                params: vec![],
            },
        ]
    }

    async fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "install" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_PARAMS".into(), message: "Missing 'name'".into(), recoverable: true })?;
                let version = args.get("version").and_then(|v| v.as_str());
                let pkg = self.install(name, version)?;
                Ok(serde_json::to_value(pkg).unwrap_or_default())
            }
            "uninstall" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_PARAMS".into(), message: "Missing 'name'".into(), recoverable: true })?;
                self.uninstall(name)?;
                Ok(serde_json::json!({"success": true}))
            }
            "toggle" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_PARAMS".into(), message: "Missing 'name'".into(), recoverable: true })?;
                let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                let pkg = self.toggle(name, enabled)?;
                Ok(serde_json::to_value(pkg).unwrap_or_default())
            }
            "start" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_PARAMS".into(), message: "Missing 'name'".into(), recoverable: true })?;
                let pkg = self.set_running(name, true)?;
                Ok(serde_json::to_value(pkg).unwrap_or_default())
            }
            "stop" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_PARAMS".into(), message: "Missing 'name'".into(), recoverable: true })?;
                let pkg = self.set_running(name, false)?;
                Ok(serde_json::to_value(pkg).unwrap_or_default())
            }
            "list" => {
                let packages = self.list_installed()?;
                Ok(serde_json::json!({"packages": packages, "total": packages.len()}))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let results = self.search(query)?;
                Ok(serde_json::json!({"results": results, "total": results.len()}))
            }
            "get_stats" => {
                let stats = self.get_stats()?;
                Ok(serde_json::to_value(stats).unwrap_or_default())
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }

    async fn init(&mut self) -> Result<(), DomainError> { Ok(()) }
    async fn shutdown(&mut self) -> Result<(), DomainError> { Ok(()) }
}
