use std::collections::HashMap;
use std::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════
// Consistent Hash — 一致性哈希环 (基于虚拟节点的负载均衡)
// ═══════════════════════════════════════════════════════════════════

/// 一致性哈希环 — 基于虚拟节点的负载均衡
pub struct ConsistentHash {
    ring: RwLock<Vec<(u64, String)>>,
    replicas: u32,
}

impl ConsistentHash {
    pub fn new(replicas: u32) -> Self {
        Self {
            ring: RwLock::new(Vec::new()),
            replicas,
        }
    }

    fn hash(key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn add_node(&self, node: &str) {
        let mut ring = self.ring.write().unwrap_or_else(|e| e.into_inner());
        for i in 0..self.replicas {
            let key = format!("{}#{}", node, i);
            let hash = Self::hash(&key);
            ring.push((hash, node.to_string()));
        }
        ring.sort_by_key(|(h, _)| *h);
    }

    pub fn remove_node(&self, node: &str) {
        let mut ring = self.ring.write().unwrap_or_else(|e| e.into_inner());
        ring.retain(|(_, n)| n != node);
    }

    pub fn get_node(&self, key: &str) -> Option<String> {
        let ring = self.ring.read().unwrap_or_else(|e| e.into_inner());
        if ring.is_empty() {
            return None;
        }
        let hash = Self::hash(key);
        let pos = ring.partition_point(|(h, _)| *h < hash);
        Some(ring[pos % ring.len()].1.clone())
    }

    pub fn get_distribution(&self, keys: &[String]) -> HashMap<String, usize> {
        let mut dist: HashMap<String, usize> = HashMap::new();
        for key in keys {
            if let Some(node) = self.get_node(key) {
                *dist.entry(node).or_insert(0) += 1;
            }
        }
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_distribution() {
        let ch = ConsistentHash::new(100);
        ch.add_node("node_a");
        ch.add_node("node_b");
        ch.add_node("node_c");

        let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();
        let dist = ch.get_distribution(&keys);
        assert_eq!(dist.len(), 3);
        for count in dist.values() {
            assert!(*count > 100, "distribution too uneven: {:?}", dist);
        }
    }

    #[test]
    fn test_node_removal() {
        let ch = ConsistentHash::new(100);
        ch.add_node("node_a");
        ch.add_node("node_b");

        let before = ch.get_node("test_key").unwrap();
        ch.remove_node(&before);
        let after = ch.get_node("test_key").unwrap();
        assert_ne!(before, after);
    }
}
