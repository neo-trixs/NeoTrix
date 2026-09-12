//! L1 基础设施 — Agent Card (能力自描述)
//!
//! 每个 Provider 发布 Agent Card:
//! - id, name, description, capabilities, tags, endpoint
//! - 支持 A2A 协议格式
//! - Registry 自动发现 + 心跳

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Agent Card — 能力自描述清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<AgentCapability>,
    pub tags: Vec<String>,
    pub endpoint: Option<String>,
    pub auth_type: Option<String>,
    pub max_concurrent: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
}

/// Agent 能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub tags: Vec<String>,
}

impl AgentCard {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            schema_version: "1.0".into(),
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: "0.1.0".into(),
            capabilities: Vec::new(),
            tags: Vec::new(),
            endpoint: None,
            auth_type: None,
            max_concurrent: 10,
            created_at: now,
            last_heartbeat: now,
        }
    }

    pub fn with_capability(mut self, cap: AgentCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    /// 心跳更新
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }

    /// 是否存活 (最近 60s 有心跳)
    pub fn is_alive(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now - self.last_heartbeat < 60
    }
}

/// Agent Card 注册表
pub(crate) struct AgentCardRegistry {
    cards: HashMap<String, AgentCard>,
}

impl Default for AgentCardRegistry {
    fn default() -> Self { Self::new() }
}

impl AgentCardRegistry {
    pub fn new() -> Self { Self { cards: HashMap::new() } }

    pub fn register(&mut self, card: AgentCard) {
        self.cards.insert(card.id.clone(), card);
    }

    pub fn unregister(&mut self, id: &str) {
        self.cards.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&AgentCard> {
        self.cards.get(id)
    }

    pub fn heartbeat(&mut self, id: &str) {
        if let Some(card) = self.cards.get_mut(id) {
            card.heartbeat();
        }
    }

    /// 按能力查找
    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentCard> {
        self.cards.values()
            .filter(|c| c.capabilities.iter().any(|cap| cap.name == capability))
            .collect()
    }

    /// 按标签查找
    pub fn find_by_tag(&self, tag: &str) -> Vec<&AgentCard> {
        self.cards.values()
            .filter(|c| c.tags.contains(&tag.to_string()))
            .collect()
    }

    /// 查找所有存活的
    pub fn alive_agents(&self) -> Vec<&AgentCard> {
        self.cards.values().filter(|c| c.is_alive()).collect()
    }

    /// 清理过期
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.cards.retain(|_, c| now - c.last_heartbeat < 300);
    }

    pub fn all_cards(&self) -> Vec<&AgentCard> {
        self.cards.values().collect()
    }
}

// 全局 Agent Card 注册表
lazy_static::lazy_static! {
    static ref GLOBAL_CARDS: Mutex<AgentCardRegistry> = Mutex::new(AgentCardRegistry::new());
}

pub(crate) fn agent_card_register(card: AgentCard) {
    GLOBAL_CARDS.lock().unwrap().register(card);
}

pub(crate) fn agent_card_get(id: &str) -> Option<AgentCard> {
    GLOBAL_CARDS.lock().unwrap().get(id).cloned()
}

pub(crate) fn agent_card_find_by_capability(cap: &str) -> Vec<AgentCard> {
    GLOBAL_CARDS.lock().unwrap().find_by_capability(cap).into_iter().cloned().collect()
}

pub(crate) fn agent_card_alive() -> Vec<AgentCard> {
    GLOBAL_CARDS.lock().unwrap().alive_agents().into_iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_lifecycle() {
        let mut card = AgentCard::new("test", "Test Agent", "A test agent");
        assert!(card.is_alive());

        card.heartbeat();
        assert!(card.is_alive());
    }

    #[test]
    fn test_registry_find() {
        let mut reg = AgentCardRegistry::new();
        let card = AgentCard::new("a1", "Agent 1", "Search agent")
            .with_capability(AgentCapability {
                name: "search".into(),
                description: "Search".into(),
                input_schema: None,
                output_schema: None,
                tags: vec![],
            })
            .with_tag("search");
        reg.register(card);
        assert_eq!(reg.find_by_capability("search").len(), 1);
        assert_eq!(reg.find_by_tag("search").len(), 1);
    }
}
