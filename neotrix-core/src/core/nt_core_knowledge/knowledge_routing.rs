use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// A routing entry mapping a knowledge topic to one or more holders
#[derive(Debug, Clone)]
pub struct RoutingEntry {
    /// The VSA hash of the topic (simplified: deterministic hash)
    pub topic_hash: u64,
    /// Human-readable topic label
    pub topic_label: String,
    /// Which agents/subsystems claim to have knowledge about this topic
    pub holders: Vec<String>,
    /// When this entry was last updated
    pub last_updated_ns: u64,
    /// Confidence that the holders actually have this knowledge (0.0-1.0)
    pub confidence: f64,
}

/// VSA-based content routing table
pub struct KnowledgeRoutingTable {
    /// Routing entries by topic hash
    pub routes: HashMap<u64, RoutingEntry>,
    /// Reverse index: agent → topics they claim
    pub agent_topics: HashMap<String, Vec<u64>>,
    /// Maximum entries before LRU eviction
    pub max_entries: usize,
    /// Default TTL in nanoseconds (1 hour)
    pub default_ttl_ns: u64,
}

impl KnowledgeRoutingTable {
    pub fn new(max_entries: usize) -> Self {
        Self {
            routes: HashMap::new(),
            agent_topics: HashMap::new(),
            max_entries,
            default_ttl_ns: 3_600_000_000_000,
        }
    }

    /// Compute a deterministic topic hash from a label
    pub fn topic_hash(label: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        label.hash(&mut hasher);
        hasher.finish()
    }

    /// An agent announces it has knowledge about a topic
    pub fn provide(&mut self, agent_id: &str, topic_label: &str, confidence: f64) {
        let hash = Self::topic_hash(topic_label);
        let now = now_nanos();

        if self.routes.len() >= self.max_entries && !self.routes.contains_key(&hash) {
            let oldest = self
                .routes
                .iter()
                .min_by_key(|(_, e)| e.last_updated_ns)
                .map(|(k, _)| *k);
            if let Some(old) = oldest {
                self.routes.remove(&old);
                self.agent_topics
                    .values_mut()
                    .for_each(|v| v.retain(|h| *h != old));
            }
        }

        let entry = self.routes.entry(hash).or_insert(RoutingEntry {
            topic_hash: hash,
            topic_label: topic_label.to_string(),
            holders: Vec::new(),
            last_updated_ns: now,
            confidence,
        });

        if !entry.holders.contains(&agent_id.to_string()) {
            entry.holders.push(agent_id.to_string());
        }
        entry.last_updated_ns = now;

        self.agent_topics
            .entry(agent_id.to_string())
            .or_default()
            .push(hash);
    }

    /// Find providers for a topic
    pub fn find_providers(&self, topic_label: &str) -> Option<&RoutingEntry> {
        let hash = Self::topic_hash(topic_label);
        self.routes.get(&hash)
    }

    /// Find topics that an agent provides
    pub fn find_topics(&self, agent_id: &str) -> Vec<&RoutingEntry> {
        match self.agent_topics.get(agent_id) {
            Some(hashes) => hashes.iter().filter_map(|h| self.routes.get(h)).collect(),
            None => Vec::new(),
        }
    }

    /// Remove expired entries
    pub fn prune_expired(&mut self) {
        let now = now_nanos();
        let expired: Vec<u64> = self
            .routes
            .iter()
            .filter(|(_, e)| now - e.last_updated_ns > self.default_ttl_ns)
            .map(|(k, _)| *k)
            .collect();
        for hash in expired {
            if let Some(entry) = self.routes.remove(&hash) {
                for holder in &entry.holders {
                    if let Some(topics) = self.agent_topics.get_mut(holder) {
                        topics.retain(|h| *h != hash);
                    }
                }
            }
        }
    }

    /// How many routes are active
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Summary of all routes
    pub fn summary(&self) -> Vec<(String, usize, f64)> {
        self.routes
            .values()
            .map(|e| (e.topic_label.clone(), e.holders.len(), e.confidence))
            .collect()
    }
}

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(cap: usize) -> KnowledgeRoutingTable {
        KnowledgeRoutingTable::new(cap)
    }

    #[test]
    fn test_provide_and_find_providers() {
        let mut table = make_table(10);
        table.provide("agent_E8", "hypercomplex_numbers", 0.9);
        let entry = table.find_providers("hypercomplex_numbers").unwrap();
        assert_eq!(entry.holders, vec!["agent_E8"]);
        assert!((entry.confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_multiple_agents_same_topic() {
        let mut table = make_table(10);
        table.provide("agent_E8", "VSA_theory", 0.85);
        table.provide("agent_KB", "VSA_theory", 0.75);
        let entry = table.find_providers("VSA_theory").unwrap();
        assert_eq!(entry.holders.len(), 2);
        assert!(entry.holders.contains(&"agent_E8".to_string()));
        assert!(entry.holders.contains(&"agent_KB".to_string()));
    }

    #[test]
    fn test_lru_eviction() {
        let mut table = make_table(2);
        table.provide("a1", "topic_alpha", 0.8);
        table.provide("a2", "topic_beta", 0.8);
        // Force slight time skew — third insert should evict oldest
        table.provide("a3", "topic_gamma", 0.8);
        assert_eq!(table.route_count(), 2);
        assert!(
            table.find_providers("topic_alpha").is_none()
                || table.find_providers("topic_beta").is_none()
        );
        assert!(table.find_providers("topic_gamma").is_some());
    }

    #[test]
    fn test_agent_topic_reverse_lookup() {
        let mut table = make_table(10);
        table.provide("agent_Vision", "image_segmentation", 0.9);
        table.provide("agent_Vision", "object_detection", 0.85);
        table.provide("agent_KG", "knowledge_graph", 0.8);
        let vision_topics = table.find_topics("agent_Vision");
        assert_eq!(vision_topics.len(), 2);
        let labels: Vec<&str> = vision_topics
            .iter()
            .map(|e| e.topic_label.as_str())
            .collect();
        assert!(labels.contains(&"image_segmentation"));
        assert!(labels.contains(&"object_detection"));
        let kg_topics = table.find_topics("agent_KG");
        assert_eq!(kg_topics.len(), 1);
    }

    #[test]
    fn test_prune_expired() {
        let mut table = make_table(10);
        table.default_ttl_ns = 1;
        table.provide("a1", "ephemeral_topic", 0.5);
        assert_eq!(table.route_count(), 1);
        table.prune_expired();
        assert_eq!(table.route_count(), 0);
        assert!(table.find_providers("ephemeral_topic").is_none());
    }
}
