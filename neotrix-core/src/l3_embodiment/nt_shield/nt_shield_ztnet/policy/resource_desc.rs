//! C5: Resource Descriptor — 资源描述
//!
//! 定义受保护资源的元数据

use std::collections::HashMap;

/// 资源描述
#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub attributes: HashMap<String, String>,
    pub required_permissions: Vec<String>,
}

impl ResourceDescriptor {
    pub fn new(id: String, name: String, resource_type: String) -> Self {
        Self {
            id,
            name,
            resource_type,
            attributes: HashMap::new(),
            required_permissions: vec![],
        }
    }

    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    pub fn with_permission(mut self, permission: String) -> Self {
        self.required_permissions.push(permission);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_creation() {
        let resource = ResourceDescriptor::new(
            "t1".into(),
            "Test Tunnel".into(),
            "tunnel".into(),
        )
        .with_attribute("max_bandwidth".into(), "100Mbps".into())
        .with_permission("connect".into());

        assert_eq!(resource.id, "t1");
        assert_eq!(resource.required_permissions.len(), 1);
    }
}
