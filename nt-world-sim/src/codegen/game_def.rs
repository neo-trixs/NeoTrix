use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDefinition {
    pub name: String,
    pub version: String,
    pub engine: EngineConfig,
    pub entities: HashMap<String, EntityDef>,
    pub systems: HashMap<String, SystemDef>,
    pub resources: HashMap<String, ResourceDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub target: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDef {
    pub components: Vec<String>,
    pub systems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDef {
    pub priority: i32,
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub fields: HashMap<String, String>,
}
