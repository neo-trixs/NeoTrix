//! MCP Resource Endpoint
//!
//! Resource trait and registry for the Model Context Protocol.
//! Resources are addressable by URI and expose content-type metadata.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// Trait
// ============================================================================

/// A single MCP resource endpoint.
pub trait McpResourceEndpoint: Send + Sync + 'static {
    /// Unique resource URI (e.g. "file:///tmp/data.csv").
    fn uri(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// MIME type of the resource content.
    fn mime_type(&self) -> &str;

    /// Read and return the resource content as a string.
    fn read(&self) -> Result<String, String>;
}

// ============================================================================
// Resource Description (serializable snapshot)
// ============================================================================

/// Serialized description of a resource for listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceDescription {
    pub uri: String,
    pub name: String,
    pub mime_type: String,
}

// ============================================================================
// Registry
// ============================================================================

/// Thread-safe registry of MCP resource endpoints.
pub struct McpResourceRegistry {
    resources: RwLock<HashMap<String, Arc<dyn McpResourceEndpoint>>>,
}

impl McpResourceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
        }
    }

    /// Register a resource. Overwrites any existing resource with the same URI.
    pub fn register(&self, endpoint: Arc<dyn McpResourceEndpoint>) {
        if let Ok(mut map) = self.resources.write() {
            map.insert(endpoint.uri().to_string(), endpoint);
        }
    }

    /// List all registered resources as serializable descriptions.
    pub fn list_resources(&self) -> Vec<ResourceDescription> {
        match self.resources.read() {
            Ok(map) => map
                .values()
                .map(|r| ResourceDescription {
                    uri: r.uri().to_string(),
                    name: r.name().to_string(),
                    mime_type: r.mime_type().to_string(),
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Read a resource by URI.
    pub fn read_resource(&self, uri: &str) -> Result<String, String> {
        let endpoint = {
            let resources = self.resources.read().map_err(|e| e.to_string())?;
            resources
                .get(uri)
                .cloned()
                .ok_or_else(|| format!("Resource not found: {}", uri))?
        };
        endpoint.read()
    }

    /// Number of registered resources.
    pub fn resource_count(&self) -> usize {
        self.resources.read().map(|m| m.len()).unwrap_or(0)
    }
}

impl Default for McpResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstantResource {
        uri: String,
        content: String,
    }

    impl McpResourceEndpoint for ConstantResource {
        fn uri(&self) -> &str {
            &self.uri
        }
        fn name(&self) -> &str {
            "constant"
        }
        fn mime_type(&self) -> &str {
            "text/plain"
        }
        fn read(&self) -> Result<String, String> {
            Ok(self.content.clone())
        }
    }

    struct FailResource;

    impl McpResourceEndpoint for FailResource {
        fn uri(&self) -> &str {
            "fail://broken"
        }
        fn name(&self) -> &str {
            "fail"
        }
        fn mime_type(&self) -> &str {
            "text/plain"
        }
        fn read(&self) -> Result<String, String> {
            Err("resource unavailable".into())
        }
    }

    #[test]
    fn test_register_and_read() {
        let registry = McpResourceRegistry::new();
        assert_eq!(registry.resource_count(), 0);

        registry.register(Arc::new(ConstantResource {
            uri: "const://a".into(),
            content: "hello".into(),
        }));
        assert_eq!(registry.resource_count(), 1);

        let content = registry.read_resource("const://a").unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_list_resources() {
        let registry = McpResourceRegistry::new();
        registry.register(Arc::new(ConstantResource {
            uri: "const://a".into(),
            content: "a".into(),
        }));
        registry.register(Arc::new(ConstantResource {
            uri: "const://b".into(),
            content: "b".into(),
        }));

        let list = registry.list_resources();
        assert_eq!(list.len(), 2);

        let uris: Vec<&str> = list.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"const://a"));
        assert!(uris.contains(&"const://b"));
    }

    #[test]
    fn test_read_resource_not_found() {
        let registry = McpResourceRegistry::new();
        let result = registry.read_resource("missing://none");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_read_resource_failure_propagation() {
        let registry = McpResourceRegistry::new();
        registry.register(Arc::new(FailResource));
        let result = registry.read_resource("fail://broken");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "resource unavailable");
    }

    #[test]
    fn test_overwrite_registration() {
        let registry = McpResourceRegistry::new();
        registry.register(Arc::new(ConstantResource {
            uri: "dup://x".into(),
            content: "first".into(),
        }));
        registry.register(Arc::new(ConstantResource {
            uri: "dup://x".into(),
            content: "second".into(),
        }));
        assert_eq!(registry.resource_count(), 1);
        assert_eq!(registry.read_resource("dup://x").unwrap(), "second");
    }
}
