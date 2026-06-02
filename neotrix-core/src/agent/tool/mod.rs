pub mod lifecycle;
pub mod impls;
pub mod builtin_adapter;
pub mod registry;
pub mod sandbox;
pub mod updater;
pub mod watcher;

pub use builtin_adapter::{handle_web_scrape, handle_security_audit, handle_react_doctor, register_adapter_tools, init_builtin_tools};
pub use impls::{WebScrapeTool, SecurityAuditTool, ReactDoctorTool};
pub use lifecycle::{
    AgentTool, ToolManifest, McpServerDecl, ToolApi, ToolContext, ToolOutput, ToolError,
    ToolPermission, ToolStorage, ToolFs, ToolIpc, ToolLogger,
};
pub use sandbox::{SandboxManager, ToolSandbox, SandboxedStorage, SandboxedFs};
pub use updater::{ToolUpdateChecker, ToolUpdate, UpdateSeverity, UpdateCheckResult};
pub use watcher::{ToolWatcher, TweakManifest, UpdateInfo, UpdatePolicy};
