#![forbid(unsafe_code)]

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub layer: String,
    pub domain: String,
}

#[async_trait]
pub trait DomainRegistry<T: Send + Sync>: Send + Sync {
    fn registry_name(&self) -> &str;

    fn register(&mut self, id: String, entry: Arc<T>);

    fn get(&self, id: &str) -> Option<Arc<T>>;

    fn list(&self) -> Vec<RegistryEntry>;

    fn has(&self, id: &str) -> bool;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_entry_serde_roundtrip() {
        let entry = RegistryEntry {
            id: "r1".into(),
            name: "Registry1".into(),
            version: "1.0".into(),
            description: "test registry".into(),
            layer: "L1".into(),
            domain: "core".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: RegistryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "r1");
        assert_eq!(back.layer, "L1");
    }

    #[test]
    fn registry_entry_fields() {
        let entry = RegistryEntry {
            id: "abc".into(),
            name: "ABC".into(),
            version: "0.1.0".into(),
            description: "desc".into(),
            layer: "L2".into(),
            domain: "mind".into(),
        };
        assert_eq!(entry.id, "abc");
        assert_eq!(entry.domain, "mind");
    }
}
