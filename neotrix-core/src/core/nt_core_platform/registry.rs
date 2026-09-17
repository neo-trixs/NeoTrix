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
