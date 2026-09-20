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
pub use tool_endpoint::{McpToolEndpoint, McpToolRegistry};
