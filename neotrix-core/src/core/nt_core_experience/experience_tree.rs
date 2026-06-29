use std::time::{Duration, SystemTime};

/// Source grounding — 将经验节点锚定到外部来源
#[derive(Debug, Clone, Default)]
pub struct SourceGrounding {
    /// 来源类型（"repo_readme", "code_analysis", "literature", "self_insight"）
    pub source_type: String,
    /// 来源标识（URL / file path / paper DOI）
    pub source_id: String,
    /// 来源中的具体位置（section / line range / paragraph）
    pub source_location: String,
    /// 直接引用的原文片段
    pub source_excerpt: String,
    /// 该来源可信度
    pub source_credibility: f64,
}

/// A single node in the experience tree.
#[derive(Debug, Clone)]
pub struct ExperienceNode {
    pub id: u64,
    pub insight: String,
    pub category: String,
    pub confidence: f64,
    pub access_count: u64,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub vsa_signature: Vec<u8>,
    pub source_cycle: u64,
    pub pruned: bool,
    /// 来源锚定 — 修复"无证据扎根"缺陷
    pub source_grounding: Option<SourceGrounding>,
}

impl ExperienceNode {
    pub fn decay_confidence(&mut self, decay_rate: f64, elapsed_cycles: u64) {
        self.confidence *= decay_rate.powi(elapsed_cycles as i32);
        if self.confidence < 0.01 {
            self.confidence = 0.0;
        }
    }

    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now();
    }

    pub fn hamming_distance(&self, other: &[u8]) -> u64 {
        let max_len = self.vsa_signature.len().min(other.len());
        self.vsa_signature[..max_len]
            .iter()
            .zip(other[..max_len].iter())
            .map(|(a, b)| (a ^ b).count_ones() as u64)
            .sum()
    }
}

/// Pruning thresholds for the four channels.
#[derive(Debug, Clone)]
pub struct PruningConfig {
    /// Channel 1: confidence decay rate per cycle (default 0.95)
    pub confidence_decay_rate: f64,
    /// Below this confidence, node is pruned (default 0.05)
    pub confidence_threshold: f64,
    /// Channel 2: max nodes to keep by access frequency (default 200)
    pub max_nodes_frequency: usize,
    /// Channel 3: seconds after which untouched nodes are stale (default 7 days)
    pub recency_window_secs: u64,
    /// Channel 4: VSA hamming similarity threshold for coalescing (default 4)
    pub vsa_similarity_threshold: u64,
    /// Max total nodes before pruning triggers (default 500)
    pub max_total_nodes: usize,
    /// Prune this many low-confidence nodes per cycle (default 10)
    pub batch_prune_count: usize,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            confidence_decay_rate: 0.95,
            confidence_threshold: 0.05,
            max_nodes_frequency: 200,
            recency_window_secs: 7 * 24 * 3600,
            vsa_similarity_threshold: 4,
            max_total_nodes: 500,
            batch_prune_count: 10,
        }
    }
}

/// Statistics from pruning operations.
#[derive(Debug, Clone, Default)]
pub struct PruneStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub pruned_nodes: usize,
    pub coalesced_nodes: usize,
    pub avg_confidence: f64,
    pub total_access_count: u64,
}

impl PruneStats {
    pub fn update_from_tree(&mut self, tree: &ExperienceTree) {
        self.total_nodes = tree.all_nodes.len();
        self.active_nodes = tree.all_nodes.iter().filter(|n| !n.pruned).count();
        self.pruned_nodes = tree.all_nodes.iter().filter(|n| n.pruned).count();
        let active: Vec<_> = tree.all_nodes.iter().filter(|n| !n.pruned).collect();
        self.avg_confidence = if active.is_empty() {
            0.0
        } else {
            active.iter().map(|n| n.confidence).sum::<f64>() / active.len() as f64
        };
        self.total_access_count = tree.all_nodes.iter().map(|n| n.access_count).sum();
    }
}

/// In-memory experience tree with four-channel pruning.
#[derive(Debug, Clone)]
pub struct ExperienceTree {
    pub all_nodes: Vec<ExperienceNode>,
    pub config: PruningConfig,
    pub stats: PruneStats,
    next_id: u64,
}

impl ExperienceTree {
    pub fn new(config: PruningConfig) -> Self {
        Self {
            all_nodes: Vec::with_capacity(config.max_total_nodes),
            config,
            stats: PruneStats::default(),
            next_id: 1,
        }
    }

    pub fn add_node(
        &mut self,
        insight: String,
        category: String,
        confidence: f64,
        vsa_signature: Vec<u8>,
        source_cycle: u64,
    ) -> u64 {
        self.add_node_with_grounding(
            insight,
            category,
            confidence,
            vsa_signature,
            source_cycle,
            None,
        )
    }

    /// 带来源锚定的添加节点 — 修复"无证据扎根"缺陷
    pub fn add_node_with_grounding(
        &mut self,
        insight: String,
        category: String,
        confidence: f64,
        vsa_signature: Vec<u8>,
        source_cycle: u64,
        source_grounding: Option<SourceGrounding>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let now = SystemTime::now();
        self.all_nodes.push(ExperienceNode {
            id,
            insight,
            category,
            confidence,
            access_count: 0,
            created_at: now,
            last_accessed: now,
            vsa_signature,
            source_cycle,
            pruned: false,
            source_grounding,
        });
        if self.all_nodes.len() > self.config.max_total_nodes {
            self.run_prune();
        }
        id
    }

    pub fn add_distilled_insight(&mut self, insight: &str, confidence: f64, cycle: u64) -> u64 {
        let vsa = deterministic_vsa(insight);
        self.add_node(
            insight.to_string(),
            "coprocessor".to_string(),
            confidence,
            vsa,
            cycle,
        )
    }

    pub fn record_access(&mut self, id: u64) {
        if let Some(node) = self.all_nodes.iter_mut().find(|n| n.id == id) {
            node.record_access();
        }
    }

    pub fn active_nodes(&self) -> Vec<&ExperienceNode> {
        self.all_nodes.iter().filter(|n| !n.pruned).collect()
    }

    /// Run all four pruning channels.
    pub fn run_prune(&mut self) {
        self.prune_confidence_decay();
        self.prune_access_frequency();
        self.prune_temporal_recency();
        self.coalesce_vsa_similarity();
        self.stats.total_nodes = self.all_nodes.len();
        self.stats.active_nodes = self.all_nodes.iter().filter(|n| !n.pruned).count();
        self.stats.pruned_nodes = self.all_nodes.iter().filter(|n| n.pruned).count();
        let active: Vec<_> = self.all_nodes.iter().filter(|n| !n.pruned).collect();
        self.stats.avg_confidence = if active.is_empty() {
            0.0
        } else {
            active.iter().map(|n| n.confidence).sum::<f64>() / active.len() as f64
        };
        self.stats.total_access_count = self.all_nodes.iter().map(|n| n.access_count).sum();
    }

    /// Channel 1: decay confidence over time, mark low-confidence as pruned.
    fn prune_confidence_decay(&mut self) {
        let elapsed = 1;
        for node in &mut self.all_nodes {
            if !node.pruned {
                node.decay_confidence(self.config.confidence_decay_rate, elapsed);
            }
        }
        let threshold = self.config.confidence_threshold;
        for node in &mut self.all_nodes {
            if !node.pruned && node.confidence < threshold {
                node.pruned = true;
            }
        }
    }

    /// Channel 2: keep only top-N active nodes by access frequency.
    fn prune_access_frequency(&mut self) {
        let max_freq = self.config.max_nodes_frequency;
        let active: Vec<(usize, u64)> = self
            .all_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.pruned)
            .map(|(i, n)| (i, n.access_count))
            .collect();
        if active.len() <= max_freq {
            return;
        }
        let mut sorted = active.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        let keep: std::collections::HashSet<usize> =
            sorted.iter().take(max_freq).map(|(i, _)| *i).collect();
        for (i, _) in &active {
            if !keep.contains(i) {
                self.all_nodes[*i].pruned = true;
            }
        }
    }

    /// Channel 3: prune nodes untouched beyond recency window.
    fn prune_temporal_recency(&mut self) {
        let window = Duration::from_secs(self.config.recency_window_secs);
        let now = SystemTime::now();
        for node in &mut self.all_nodes {
            if !node.pruned {
                if let Ok(elapsed) = now.duration_since(node.last_accessed) {
                    if elapsed > window {
                        node.pruned = true;
                    }
                }
            }
        }
    }

    /// Channel 4: merge nodes with similar VSA signatures.
    fn coalesce_vsa_similarity(&mut self) {
        let threshold = self.config.vsa_similarity_threshold;
        let indices: Vec<usize> = self
            .all_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.pruned)
            .map(|(i, _)| i)
            .collect();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let i_idx = indices[i];
                let j_idx = indices[j];
                if self.all_nodes[j_idx].pruned {
                    continue;
                }
                let dist =
                    self.all_nodes[i_idx].hamming_distance(&self.all_nodes[j_idx].vsa_signature);
                if dist < threshold {
                    let acc_i = self.all_nodes[i_idx].access_count;
                    let acc_j = self.all_nodes[j_idx].access_count;
                    let conf_i = self.all_nodes[i_idx].confidence;
                    let conf_j = self.all_nodes[j_idx].confidence;
                    let total = acc_i + acc_j;
                    if total > 0 {
                        self.all_nodes[i_idx].confidence =
                            (conf_i * acc_i as f64 + conf_j * acc_j as f64) / total as f64;
                    } else {
                        self.all_nodes[i_idx].confidence = (conf_i + conf_j) / 2.0;
                    }
                    self.all_nodes[i_idx].access_count = total;
                    self.all_nodes[i_idx].insight = format!(
                        "{}\n{}",
                        self.all_nodes[i_idx].insight, self.all_nodes[j_idx].insight
                    );
                    self.all_nodes[j_idx].pruned = true;
                    self.stats.coalesced_nodes += 1;
                }
            }
        }
    }

    pub fn mark_alive(&mut self, id: u64) {
        if let Some(node) = self.all_nodes.iter_mut().find(|n| n.id == id) {
            node.pruned = false;
            node.last_accessed = SystemTime::now();
            node.confidence = node.confidence.max(0.3);
        }
    }

    pub fn stats_report(&self) -> String {
        format!(
            "ExperienceTree: {} total / {} active / {} pruned / {} coalesced | avg_conf {:.3} | total_access {}",
            self.stats.total_nodes,
            self.stats.active_nodes,
            self.stats.pruned_nodes,
            self.stats.coalesced_nodes,
            self.stats.avg_confidence,
            self.stats.total_access_count,
        )
    }
}

/// Deterministic VSA signature from text (fold hash).
fn deterministic_vsa(text: &str) -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();
    let mut sig = Vec::with_capacity(64);
    for i in 0..64 {
        sig.push(((hash >> ((i * 4) % 64)) & 0xFF) as u8);
    }
    sig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut tree = ExperienceTree::new(PruningConfig::default());
        let id = tree.add_node(
            "test insight".into(),
            "test".into(),
            0.8,
            vec![1, 2, 3, 4],
            1,
        );
        assert_eq!(tree.all_nodes.len(), 1);
        assert!(!tree.all_nodes[0].pruned);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_confidence_decay() {
        let mut node = ExperienceNode {
            id: 1,
            insight: "t".into(),
            category: "t".into(),
            confidence: 0.8,
            access_count: 0,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            vsa_signature: vec![],
            source_cycle: 0,
            pruned: false,
            source_grounding: None,
        };
        node.decay_confidence(0.5, 4);
        assert!((node.confidence - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_prune_confidence_channel() {
        let mut tree = ExperienceTree::new(PruningConfig {
            confidence_threshold: 0.1,
            ..Default::default()
        });
        tree.add_node("high conf".into(), "t".into(), 0.9, vec![], 1);
        tree.add_node("low conf".into(), "t".into(), 0.01, vec![], 1);
        tree.run_prune();
        let active = tree.active_nodes();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].insight, "high conf");
    }

    #[test]
    fn test_record_access() {
        let mut tree = ExperienceTree::new(PruningConfig::default());
        let id = tree.add_node("acc test".into(), "t".into(), 0.5, vec![], 1);
        tree.record_access(id);
        assert_eq!(tree.all_nodes[0].access_count, 1);
    }

    #[test]
    fn test_prune_frequency_channel() {
        let mut tree = ExperienceTree::new(PruningConfig {
            max_nodes_frequency: 2,
            ..Default::default()
        });
        tree.add_node("freq1".into(), "t".into(), 0.5, vec![], 1);
        tree.add_node("freq2".into(), "t".into(), 0.5, vec![], 1);
        tree.add_node("freq3".into(), "t".into(), 0.5, vec![], 1);
        tree.record_access(1);
        tree.record_access(1);
        tree.record_access(2);
        tree.run_prune();
        let active = tree.active_nodes();
        assert!(active.len() <= 2);
    }

    #[test]
    fn test_vsa_coalescing() {
        let mut tree = ExperienceTree::new(PruningConfig {
            vsa_similarity_threshold: 10,
            ..Default::default()
        });
        let sig1 = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let sig2 = vec![0, 1, 0, 1, 0, 1, 0, 0];
        tree.add_node("node a".into(), "t".into(), 0.5, sig1.clone(), 1);
        tree.add_node("node b".into(), "t".into(), 0.5, sig2.clone(), 1);
        tree.run_prune();
        let active = tree.active_nodes();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_add_distilled_insight() {
        let mut tree = ExperienceTree::new(PruningConfig::default());
        let id = tree.add_distilled_insight("test insight from coproc", 0.8, 42);
        assert!(id > 0);
        assert_eq!(tree.all_nodes.len(), 1);
    }

    #[test]
    fn test_mark_alive() {
        let mut tree = ExperienceTree::new(PruningConfig::default());
        let id = tree.add_node("revive me".into(), "t".into(), 0.01, vec![], 1);
        tree.run_prune();
        assert!(tree.active_nodes().is_empty());
        tree.mark_alive(id);
        assert_eq!(tree.active_nodes().len(), 1);
    }

    #[test]
    fn test_stats_report() {
        let mut tree = ExperienceTree::new(PruningConfig::default());
        tree.add_node("s1".into(), "t".into(), 0.9, vec![], 1);
        tree.add_node("s2".into(), "t".into(), 0.8, vec![], 1);
        let report = tree.stats_report();
        assert!(report.contains("active"));
        assert!(report.contains("avg_conf"));
    }

    #[test]
    fn test_prune_all_channels() {
        let mut tree = ExperienceTree::new(PruningConfig {
            confidence_threshold: 0.3,
            max_nodes_frequency: 3,
            recency_window_secs: 1,
            vsa_similarity_threshold: 8,
            max_total_nodes: 100,
            batch_prune_count: 10,
            confidence_decay_rate: 0.5,
        });
        tree.add_node(
            "keep high conf".into(),
            "t".into(),
            0.9,
            vec![0, 0, 0, 0],
            1,
        );
        tree.add_node("low conf".into(), "t".into(), 0.2, vec![1, 1, 1, 1], 1);
        tree.add_node(
            "similar to first".into(),
            "t".into(),
            0.8,
            vec![0, 0, 0, 1],
            1,
        );
        tree.run_prune();
        let active = tree.active_nodes();
        assert!(active.len() <= 2);
    }

    #[test]
    fn test_deterministic_vsa() {
        let sig1 = deterministic_vsa("hello world");
        let sig2 = deterministic_vsa("hello world");
        let sig3 = deterministic_vsa("different text");
        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3);
        assert_eq!(sig1.len(), 64);
    }
}
