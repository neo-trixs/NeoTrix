#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FringeMixStrategy {
    pub central_percentile: f64,
    pub peripheral_percentile: f64,
    pub default_ratio: f64,
}

impl Default for FringeMixStrategy {
    fn default() -> Self {
        Self {
            central_percentile: 0.75,
            peripheral_percentile: 0.25,
            default_ratio: 1.0,
        }
    }
}

impl FringeMixStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_centrality(&self, graph: &HashMap<u64, Vec<u64>>) -> HashMap<u64, f64> {
        let mut centrality: HashMap<u64, f64> = graph.keys().map(|k| (*k, 1.0)).collect();
        if centrality.is_empty() {
            return centrality;
        }
        for _ in 0..10 {
            let mut new_centrality = HashMap::with_capacity(centrality.len());
            let total: f64 = centrality.values().sum();
            if total == 0.0 {
                break;
            }
            for (node, neighbors) in graph.iter() {
                let mut score = 0.0;
                for neighbor in neighbors.iter() {
                    let deg = graph.get(neighbor).map(|n| n.len()).unwrap_or(1).max(1);
                    score += centrality.get(neighbor).copied().unwrap_or(0.0) / deg as f64;
                }
                new_centrality.insert(*node, score);
            }
            let new_total: f64 = new_centrality.values().sum();
            if new_total > 0.0 {
                for v in new_centrality.values_mut() {
                    *v /= new_total;
                }
            }
            centrality = new_centrality;
        }
        centrality
    }

    fn percentile(values: &mut Vec<f64>, p: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((values.len() as f64) * p).ceil() as usize;
        let idx = idx.max(1).min(values.len()) - 1;
        values[idx]
    }

    pub fn classify_nodes(&self, centrality: &HashMap<u64, f64>) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
        if centrality.is_empty() {
            return (vec![], vec![], vec![]);
        }
        let mut values: Vec<f64> = centrality.values().copied().collect();
        let p75 = Self::percentile(&mut values, self.central_percentile);
        let p25 = Self::percentile(&mut values, self.peripheral_percentile);

        let mut central = Vec::new();
        let mut peripheral = Vec::new();
        let mut mid = Vec::new();
        for (node, score) in centrality.iter() {
            if *score >= p75 && values.len() > 1 {
                central.push(*node);
            } else if *score <= p25 && values.len() > 1 {
                peripheral.push(*node);
            } else {
                mid.push(*node);
            }
        }
        (central, peripheral, mid)
    }

    pub fn sample_mix(&self, nodes: &[u64], ratio: f64) -> Vec<u64> {
        if nodes.is_empty() {
            return vec![];
        }
        let n = ((nodes.len() as f64) * ratio).round() as usize;
        let n = n.min(nodes.len());
        nodes.iter().take(n).copied().collect()
    }

    pub fn score_with_centrality(
        &self,
        results: &[(u64, f64)],
        centralities: &HashMap<u64, f64>,
        alpha: f64,
    ) -> Vec<(u64, f64)> {
        results
            .iter()
            .map(|(id, score)| {
                let c = centralities.get(id).copied().unwrap_or(0.0);
                (*id, *score * (1.0 + alpha * c))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_centrality_on_star_graph() {
        let strat = FringeMixStrategy::default();
        let mut graph = HashMap::new();
        graph.insert(1, vec![2, 3, 4]);
        graph.insert(2, vec![1]);
        graph.insert(3, vec![1]);
        graph.insert(4, vec![1]);
        let centrality = strat.compute_centrality(&graph);
        assert_eq!(centrality.len(), 4);
        let center_score = centrality.get(&1).copied().unwrap_or(0.0);
        let leaf_score = centrality.get(&2).copied().unwrap_or(0.0);
        assert!(center_score > leaf_score);
    }

    #[test]
    fn test_classify_nodes_partitions() {
        let strat = FringeMixStrategy::default();
        let mut centrality = HashMap::new();
        centrality.insert(1, 0.4);
        centrality.insert(2, 0.3);
        centrality.insert(3, 0.2);
        centrality.insert(4, 0.1);
        let (central, peripheral, _mid) = strat.classify_nodes(&centrality);
        assert_eq!(central.len(), 1);
        assert_eq!(peripheral.len(), 1);
    }

    #[test]
    fn test_empty_graph() {
        let strat = FringeMixStrategy::default();
        let graph = HashMap::new();
        let centrality = strat.compute_centrality(&graph);
        assert!(centrality.is_empty());
    }

    #[test]
    fn test_single_node_graph() {
        let strat = FringeMixStrategy::default();
        let mut graph = HashMap::new();
        graph.insert(1, vec![]);
        let centrality = strat.compute_centrality(&graph);
        assert_eq!(centrality.len(), 1);
    }

    #[test]
    fn test_sample_mix_ratio() {
        let strat = FringeMixStrategy::default();
        let nodes = vec![1, 2, 3, 4];
        let sampled = strat.sample_mix(&nodes, 0.5);
        assert_eq!(sampled.len(), 2);
    }

    #[test]
    fn test_sample_mix_empty() {
        let strat = FringeMixStrategy::default();
        assert!(strat.sample_mix(&[], 0.5).is_empty());
    }

    #[test]
    fn test_score_with_centrality_boosts() {
        let strat = FringeMixStrategy::default();
        let results = vec![(1, 0.5), (2, 0.3)];
        let mut centralities = HashMap::new();
        centralities.insert(1, 0.8);
        centralities.insert(2, 0.1);
        let scored = strat.score_with_centrality(&results, &centralities, 0.3);
        assert!(scored[0].1 > scored[1].1);
    }

    #[test]
    fn test_classify_nodes_empty() {
        let strat = FringeMixStrategy::default();
        let (c, p, m) = strat.classify_nodes(&HashMap::new());
        assert!(c.is_empty() && p.is_empty() && m.is_empty());
    }
}
