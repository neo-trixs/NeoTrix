use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArchiveNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,
    pub description: String,
    pub step_type: String,
    pub cycle: u64,
    pub lineage: Vec<u64>,
    pub performance: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ArchiveTree {
    pub nodes: HashMap<u64, ArchiveNode>,
    pub root_id: u64,
    pub next_id: u64,
    pub max_nodes: usize,
}

impl ArchiveTree {
    fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: 1,
            next_id: 1,
            max_nodes,
        }
    }

    fn add_node_raw(&mut self, node: ArchiveNode) {
        let id = node.id;
        self.nodes.insert(id, node);
    }

    fn lru_candidates(&self, count: usize) -> Vec<u64> {
        let mut candidates: Vec<(&u64, &ArchiveNode)> = self
            .nodes
            .iter()
            .filter(|(id, n)| **id != self.root_id && n.children.is_empty())
            .collect();
        candidates.sort_by_key(|(_, n)| n.cycle);
        candidates
            .into_iter()
            .take(count)
            .map(|(id, _)| *id)
            .collect()
    }

    fn depth(&self) -> usize {
        self.nodes
            .values()
            .map(|n| n.lineage.len())
            .max()
            .unwrap_or(0)
    }

    fn leaf_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.children.is_empty())
            .count()
    }

    fn avg_performance(&self) -> f64 {
        let count = self.nodes.len();
        if count == 0 {
            return 0.0;
        }
        let sum: f64 = self.nodes.values().map(|n| n.performance).sum();
        sum / count as f64
    }
}

#[derive(Debug, Clone)]
pub struct ArchiveManager {
    pub tree: ArchiveTree,
    pub active_branch: u64,
    pub pruned_count: u64,
}

#[derive(Debug, Clone)]
pub struct ArchiveStats {
    pub total_nodes: usize,
    pub depth: usize,
    pub leaf_count: usize,
    pub avg_performance: f64,
    pub pruned_count: u64,
}

impl ArchiveManager {
    pub fn new() -> Self {
        let mut tree = ArchiveTree::new(500);
        let root = ArchiveNode {
            id: 1,
            parent_id: None,
            children: Vec::new(),
            description: "origin".to_string(),
            step_type: "commit".to_string(),
            cycle: 0,
            lineage: vec![1],
            performance: 1.0,
            metadata: HashMap::new(),
        };
        tree.add_node_raw(root);
        tree.next_id = 2;
        Self {
            tree,
            active_branch: 1,
            pruned_count: 0,
        }
    }

    pub fn add_node(
        &mut self,
        parent_id: u64,
        description: &str,
        step_type: &str,
        cycle: u64,
        performance: f64,
    ) -> Option<u64> {
        if !self.tree.nodes.contains_key(&parent_id) {
            return None;
        }
        if self.tree.nodes.len() >= self.tree.max_nodes {
            let candidates = self.tree.lru_candidates(1);
            if let Some(&victim) = candidates.first() {
                self.tree.nodes.remove(&victim);
                self.pruned_count += 1;
            } else {
                return None;
            }
        }
        let id = self.tree.next_id;
        self.tree.next_id += 1;

        let parent_lineage = self.tree.nodes[&parent_id].lineage.clone();
        let mut lineage = parent_lineage.clone();
        lineage.push(id);

        let node = ArchiveNode {
            id,
            parent_id: Some(parent_id),
            children: Vec::new(),
            description: description.to_string(),
            step_type: step_type.to_string(),
            cycle,
            lineage,
            performance: performance.clamp(0.0, 1.0),
            metadata: HashMap::new(),
        };
        let child_id = node.id;
        self.tree.add_node_raw(node);
        if let Some(parent) = self.tree.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
        Some(child_id)
    }

    pub fn fork(&mut self, ancestor_id: u64, description: &str, cycle: u64) -> Option<u64> {
        if !self.tree.nodes.contains_key(&ancestor_id) {
            return None;
        }
        self.add_node(ancestor_id, description, "mutation", cycle, 0.5)
    }

    pub fn rollback(&mut self, node_id: u64) -> bool {
        if node_id == self.tree.root_id {
            return false;
        }
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            node.metadata
                .insert("rolled_back".to_string(), "true".to_string());
            if let Some(parent_id) = node.parent_id {
                self.active_branch = parent_id;
                return true;
            }
        }
        false
    }

    pub fn get_lineage(&self, node_id: u64) -> Vec<ArchiveNode> {
        let mut result = Vec::new();
        let mut current = node_id;
        let root_id = self.tree.root_id;
        loop {
            if let Some(node) = self.tree.nodes.get(&current) {
                result.push(node.clone());
                if current == root_id {
                    break;
                }
                current = match node.parent_id {
                    Some(pid) => pid,
                    None => break,
                };
            } else {
                break;
            }
        }
        result.reverse();
        result
    }

    pub fn best_path(&self) -> Vec<ArchiveNode> {
        let mut path = Vec::new();
        let root_id = self.tree.root_id;
        if !self.tree.nodes.contains_key(&root_id) {
            return path;
        }

        let mut current_id = root_id;
        loop {
            if let Some(node) = self.tree.nodes.get(&current_id) {
                path.push(node.clone());
                if node.children.is_empty() {
                    break;
                }
                let best_child = node
                    .children
                    .iter()
                    .filter_map(|cid| self.tree.nodes.get(cid))
                    .max_by(|a, b| {
                        a.performance
                            .partial_cmp(&b.performance)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                match best_child {
                    Some(child) => current_id = child.id,
                    None => break,
                }
            } else {
                break;
            }
        }
        path
    }

    pub fn prune(&mut self, below_performance: f64) {
        loop {
            let to_remove: Vec<u64> = self
                .tree
                .nodes
                .iter()
                .filter(|(id, n)| {
                    **id != self.tree.root_id
                        && n.children.is_empty()
                        && n.performance < below_performance
                })
                .map(|(id, _)| *id)
                .collect();

            if to_remove.is_empty() {
                break;
            }

            for id in &to_remove {
                if let Some(node) = self.tree.nodes.remove(id) {
                    self.pruned_count += 1;
                    if let Some(parent) = self.tree.nodes.get_mut(&node.parent_id.unwrap_or(0)) {
                        parent.children.retain(|c| c != id);
                    }
                }
            }
        }
    }

    pub fn merge(&mut self, branch_node_id: u64, target_node_id: u64) {
        let branch_perf = self.tree.nodes.get(&branch_node_id).map(|n| n.performance);
        let target_perf = self.tree.nodes.get(&target_node_id).map(|n| n.performance);

        if let (Some(bp), Some(tp)) = (branch_perf, target_perf) {
            if let Some(target) = self.tree.nodes.get_mut(&target_node_id) {
                target.performance = (bp + tp) / 2.0;
            }
        }

        if let (Some(branch), Some(target)) = (
            self.tree.nodes.get(&branch_node_id).cloned(),
            self.tree.nodes.get(&target_node_id).cloned(),
        ) {
            if let Some(target) = self.tree.nodes.get_mut(&target_node_id) {
                for (k, v) in &branch.metadata {
                    let merged_key = format!("{}(merged)", k);
                    target
                        .metadata
                        .entry(merged_key)
                        .or_insert_with(|| v.clone());
                    target
                        .metadata
                        .entry(k.clone())
                        .or_insert_with(|| v.clone());
                }
            }

            self.tree.nodes.remove(&branch_node_id);
            self.pruned_count += 1;

            if let Some(parent) = branch.parent_id {
                if let Some(parent_node) = self.tree.nodes.get_mut(&parent) {
                    parent_node.children.retain(|c| *c != branch_node_id);
                }
            }

            if let Some(target_parent) = target.parent_id {
                if let Some(target_parent_node) = self.tree.nodes.get_mut(&target_parent) {
                    if !target_parent_node.children.contains(&branch_node_id) {}
                }
            }
        }
    }

    pub fn stats(&self) -> ArchiveStats {
        ArchiveStats {
            total_nodes: self.tree.nodes.len(),
            depth: self.tree.depth(),
            leaf_count: self.tree.leaf_count(),
            avg_performance: self.tree.avg_performance(),
            pruned_count: self.pruned_count,
        }
    }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "ArchiveManager: nodes={} depth={} leaves={} avg_perf={:.3} pruned={}",
            s.total_nodes, s.depth, s.leaf_count, s.avg_performance, s.pruned_count
        )
    }
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let am = ArchiveManager::new();
        assert_eq!(am.tree.nodes.len(), 1);
        assert!(am.tree.nodes.contains_key(&1));
        assert_eq!(am.tree.nodes[&1].description, "origin");
    }

    #[test]
    fn test_add_node() {
        let mut am = ArchiveManager::new();
        let child_id = am.add_node(1, "first step", "proposal", 1, 0.8);
        assert!(child_id.is_some());
        let id = child_id.unwrap();
        assert!(am.tree.nodes.contains_key(&id));
        assert!(am.tree.nodes[&1].children.contains(&id));
    }

    #[test]
    fn test_add_node_exceeds_max() {
        let mut am = ArchiveManager::new();
        am.tree.max_nodes = 2;
        let a = am.add_node(1, "a", "mutation", 1, 0.5);
        assert!(a.is_some());
        let b = am.add_node(1, "b", "mutation", 2, 0.6);
        assert!(b.is_some());
        let c = am.add_node(1, "c", "mutation", 3, 0.7);
        assert!(c.is_none());
    }

    #[test]
    fn test_fork() {
        let mut am = ArchiveManager::new();
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        let b = am.add_node(a, "B", "mutation", 2, 0.6).unwrap();
        let _c = am.add_node(b, "C", "mutation", 3, 0.7).unwrap();
        let fork_id = am.fork(a, "forked from A", 4);
        assert!(fork_id.is_some());
        let f = fork_id.unwrap();
        assert!(am.tree.nodes[&a].children.contains(&f));
        assert_eq!(am.tree.nodes[&f].parent_id, Some(a));
    }

    #[test]
    fn test_rollback() {
        let mut am = ArchiveManager::new();
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        am.active_branch = a;
        let success = am.rollback(a);
        assert!(success);
        assert_eq!(am.active_branch, 1);
        assert_eq!(
            am.tree.nodes[&a]
                .metadata
                .get("rolled_back")
                .map(|s| s.as_str()),
            Some("true")
        );
    }

    #[test]
    fn test_rollback_root() {
        let mut am = ArchiveManager::new();
        let success = am.rollback(1);
        assert!(!success);
        assert_eq!(am.active_branch, 1);
    }

    #[test]
    fn test_get_lineage() {
        let mut am = ArchiveManager::new();
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        let b = am.add_node(a, "B", "mutation", 2, 0.6).unwrap();
        let c = am.add_node(b, "C", "commit", 3, 0.9).unwrap();
        let lineage = am.get_lineage(c);
        assert_eq!(lineage.len(), 4);
        assert_eq!(lineage[0].description, "origin");
        assert_eq!(lineage[1].description, "A");
        assert_eq!(lineage[2].description, "B");
        assert_eq!(lineage[3].description, "C");
    }

    #[test]
    fn test_best_path() {
        let mut am = ArchiveManager::new();
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        let _b = am.add_node(a, "B_low", "mutation", 2, 0.3).unwrap();
        let c = am.add_node(a, "C_high", "mutation", 2, 0.9).unwrap();
        let _d = am.add_node(c, "D_low", "mutation", 3, 0.4).unwrap();
        let e = am.add_node(c, "E_high", "mutation", 3, 0.95).unwrap();

        let path = am.best_path();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0].description, "origin");
        assert_eq!(path[1].description, "A");
        assert_eq!(path[2].description, "C_high");
        assert_eq!(path[3].description, "E_high");
    }

    #[test]
    fn test_prune() {
        let mut am = ArchiveManager::new();
        am.add_node(1, "low", "mutation", 1, 0.1);
        am.add_node(1, "mid", "mutation", 2, 0.5);
        am.add_node(1, "high", "mutation", 3, 0.9);
        am.prune(0.3);
        let count_before = am.pruned_count;
        am.prune(0.3);
        assert_eq!(am.pruned_count, count_before + 1);
        assert!(am
            .tree
            .nodes
            .values()
            .any(|n| (n.performance - 0.5).abs() < 0.01));
        assert!(am
            .tree
            .nodes
            .values()
            .any(|n| (n.performance - 0.9).abs() < 0.01));
    }

    #[test]
    fn test_merge() {
        let mut am = ArchiveManager::new();
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        let b = am.add_node(a, "B", "mutation", 2, 0.7).unwrap();
        let c = am.add_node(a, "C", "mutation", 2, 0.3).unwrap();

        let mut meta = HashMap::new();
        meta.insert("source".to_string(), "branch".to_string());
        if let Some(node) = am.tree.nodes.get_mut(&b) {
            node.metadata = meta;
        }

        am.merge(b, c);
        let target = &am.tree.nodes[&c];
        assert!((target.performance - 0.5).abs() < 0.01);
        assert!(target.metadata.contains_key("source(merged)"));
        assert!(!am.tree.nodes.contains_key(&b));
    }

    #[test]
    fn test_stats() {
        let mut am = ArchiveManager::new();
        am.add_node(1, "A", "proposal", 1, 0.8);
        am.add_node(1, "B", "mutation", 2, 0.6);
        let s = am.stats();
        assert_eq!(s.total_nodes, 3);
        assert_eq!(s.leaf_count, 2);
        assert!(s.depth >= 1);
        assert!((s.avg_performance - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_summary() {
        let mut am = ArchiveManager::new();
        am.add_node(1, "test", "proposal", 1, 0.75);
        let s = am.summary();
        assert!(s.contains("ArchiveManager:"));
        assert!(s.contains("nodes="));
        assert!(s.contains("avg_perf="));
    }

    #[test]
    fn test_lru_eviction() {
        let mut am = ArchiveManager::new();
        am.tree.max_nodes = 3;
        let a = am.add_node(1, "A", "mutation", 1, 0.5).unwrap();
        let _b = am.add_node(a, "B", "mutation", 2, 0.6).unwrap();
        let c = am.add_node(a, "C", "mutation", 3, 0.7).unwrap();
        let d = am.add_node(a, "D", "mutation", 4, 0.8);
        assert!(d.is_some());
        assert!(am.tree.nodes.contains_key(&1));
        assert!(am.tree.nodes.contains_key(&c));
        assert!(am.tree.nodes.contains_key(&d.unwrap()));
    }

    #[test]
    fn test_metadata() {
        let mut am = ArchiveManager::new();
        let id = am.add_node(1, "meta_test", "proposal", 1, 0.5).unwrap();
        let node = am.tree.nodes.get_mut(&id).unwrap();
        node.metadata
            .insert("key1".to_string(), "value1".to_string());
        node.metadata
            .insert("key2".to_string(), "value2".to_string());

        let retrieved = &am.tree.nodes[&id];
        assert_eq!(retrieved.metadata.get("key1").unwrap(), "value1");
        assert_eq!(retrieved.metadata.get("key2").unwrap(), "value2");
    }

    #[test]
    fn test_add_node_missing_parent() {
        let mut am = ArchiveManager::new();
        let result = am.add_node(999, "orphan", "proposal", 1, 0.5);
        assert!(result.is_none());
    }
}
