//! MCP Protocol Layer
//!
//! Model Context Protocol implementation — tools, resources, and protocol
//! routing for NeoTrix agent capabilities. Follows R-P1 (zero unsafe)
//! and R-P128 (no raw pointers).

pub mod protocol;
pub mod resource_endpoint;
pub mod tool_endpoint;

pub use protocol::McpProtocol;
pub use resource_endpoint::{McpResourceEndpoint, McpResourceRegistry};
pub use tool_endpoint::{McpEndpointRegistry, McpToolEndpoint};
/// 迁移期重导出（deprecated 别名，调用方正迁往 McpEndpointRegistry）
#[allow(deprecated)]
pub use tool_endpoint::McpToolRegistry;
