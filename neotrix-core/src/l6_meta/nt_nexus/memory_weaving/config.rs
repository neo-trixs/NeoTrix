use serde::{Deserialize, Serialize};

/// Configuration for the memory weaving subsystem.
///
/// Controls pattern mining thresholds, graph size limits,
/// and consolidation scheduling (R-MEM07 episode provenance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveConfig {
    /// Minimum number of sessions a pattern must appear in before
    /// it is considered for cross-session bridging.
    pub min_pattern_frequency: usize,

    /// Hard cap on the number of nodes in the WeaveGraph.
    /// Older / least-connected nodes are pruned when exceeded.
    pub max_graph_nodes: usize,

    /// Interval (in seconds) between automatic consolidation runs.
    pub consolidation_interval: u64,

    /// Maximum age (in seconds) for a pattern to remain in the graph
    /// without being accessed before it becomes a stale-pruning candidate.
    pub max_pattern_age_secs: u64,

    /// Minimum Jaccard similarity between two sessions' keyword sets
    /// for a bridge to be formed.
    pub min_bridge_similarity: f64,
}

impl Default for WeaveConfig {
    fn default() -> Self {
        Self {
            min_pattern_frequency: 2,
            max_graph_nodes: 10_000,
            consolidation_interval: 3600,
            max_pattern_age_secs: 86_400 * 30, // 30 days
            min_bridge_similarity: 0.15,
        }
    }
}

impl WeaveConfig {
    /// Conservative config for resource-constrained environments.
    pub fn minimal() -> Self {
        Self {
            min_pattern_frequency: 3,
            max_graph_nodes: 1_000,
            consolidation_interval: 7200,
            max_pattern_age_secs: 86_400 * 7,
            min_bridge_similarity: 0.25,
        }
    }

    /// Aggressive config for large deployments.
    pub fn expansive() -> Self {
        Self {
            min_pattern_frequency: 2,
            max_graph_nodes: 50_000,
            consolidation_interval: 1800,
            max_pattern_age_secs: 86_400 * 90,
            min_bridge_similarity: 0.10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        let cfg = WeaveConfig::default();
        assert!(cfg.min_pattern_frequency >= 1);
        assert!(cfg.max_graph_nodes >= 100);
        assert!(cfg.consolidation_interval >= 60);
        assert!(cfg.min_bridge_similarity > 0.0 && cfg.min_bridge_similarity <= 1.0);
    }

    #[test]
    fn minimal_stricter() {
        let m = WeaveConfig::minimal();
        let d = WeaveConfig::default();
        assert!(m.min_pattern_frequency >= d.min_pattern_frequency);
        assert!(m.max_graph_nodes <= d.max_graph_nodes);
        assert!(m.min_bridge_similarity >= d.min_bridge_similarity);
    }

    #[test]
    fn expansive_looser() {
        let e = WeaveConfig::expansive();
        let d = WeaveConfig::default();
        assert!(e.max_graph_nodes >= d.max_graph_nodes);
        assert!(e.min_bridge_similarity <= d.min_bridge_similarity);
    }

    #[test]
    fn serde_roundtrip() {
        let cfg = WeaveConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: WeaveConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.min_pattern_frequency, back.min_pattern_frequency);
        assert_eq!(cfg.max_graph_nodes, back.max_graph_nodes);
    }
}
