use std::collections::HashMap;

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) -> bool {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return false;
        }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
            self.size[ry] += self.size[rx];
        } else if self.rank[rx] > self.rank[ry] {
            self.parent[ry] = rx;
            self.size[rx] += self.size[ry];
        } else {
            self.parent[ry] = rx;
            self.size[rx] += self.size[ry];
            self.rank[rx] += 1;
        }
        true
    }

    #[allow(dead_code)]
    fn cluster_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        self.size[root]
    }
}

#[derive(Debug, Clone)]
pub struct ConceptCluster {
    pub id: usize,
    pub member_ids: Vec<String>,
    pub centroid: Vec<f32>,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct ConceptEmergenceEngine {
    pub min_cluster_size: usize,
    pub max_concepts: usize,
    pub similarity_threshold: f64,
}

impl Default for ConceptEmergenceEngine {
    fn default() -> Self {
        Self {
            min_cluster_size: 3,
            max_concepts: 64,
            similarity_threshold: 0.72,
        }
    }
}

impl ConceptEmergenceEngine {
    pub fn discover_concepts(&self, nodes: &[(String, Vec<f32>)]) -> Vec<ConceptCluster> {
        if nodes.len() < self.min_cluster_size {
            return Vec::new();
        }

        let n = nodes.len();
        let dim = nodes[0].1.len();
        let threshold = self.similarity_threshold as f32;

        let mut uf = UnionFind::new(n);
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = cosine_similarity(&nodes[i].1, &nodes[j].1);
                if sim >= threshold {
                    uf.union(i, j);
                }
            }
        }

        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = uf.find(i);
            groups.entry(root).or_default().push(i);
        }

        let mut clusters: Vec<ConceptCluster> = groups
            .into_iter()
            .filter(|(_, members)| members.len() >= self.min_cluster_size)
            .enumerate()
            .map(|(idx, (_root, member_indices))| {
                let member_ids: Vec<String> =
                    member_indices.iter().map(|&i| nodes[i].0.clone()).collect();
                let centroid = compute_centroid(
                    &member_indices.iter().map(|&i| &nodes[i].1).collect::<Vec<_>>(),
                    dim,
                );
                let label = auto_label(&member_ids, idx);
                ConceptCluster {
                    id: idx,
                    member_ids,
                    centroid,
                    label,
                }
            })
            .collect();

        if clusters.len() > self.max_concepts {
            clusters.sort_by(|a, b| b.member_ids.len().cmp(&a.member_ids.len()));
            clusters.truncate(self.max_concepts);
            for (i, c) in clusters.iter_mut().enumerate() {
                c.id = i;
            }
        }

        clusters
    }
}

use crate::core::nt_core_math::cosine_similarity_f32_f32 as cosine_similarity;

fn compute_centroid(vectors: &[&Vec<f32>], dim: usize) -> Vec<f32> {
    if vectors.is_empty() {
        return vec![0.0; dim];
    }
    let mut centroid = vec![0.0f32; dim];
    for v in vectors {
        for (c, &val) in centroid.iter_mut().zip(v.iter()) {
            *c += val;
        }
    }
    let n = vectors.len() as f32;
    for c in centroid.iter_mut() {
        *c /= n;
    }
    centroid
}

fn auto_label(member_ids: &[String], cluster_idx: usize) -> String {
    if member_ids.is_empty() {
        return format!("concept_{}", cluster_idx);
    }
    let root = member_ids
        .iter()
        .min_by_key(|id| id.len())
        .map(|id| id.as_str())
        .unwrap_or("unknown");
    let base = if let Some(pos) = root.rfind('/') {
        &root[pos + 1..]
    } else {
        root
    };
    let truncated: String = base.chars().take(24).collect();
    format!("{}({})", truncated, member_ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes(pairs: Vec<(&str, Vec<f32>)>) -> Vec<(String, Vec<f32>)> {
        pairs.into_iter().map(|(s, v)| (s.to_string(), v)).collect()
    }

    #[test]
    fn identical_vectors_form_cluster() {
        let engine = ConceptEmergenceEngine {
            min_cluster_size: 2,
            similarity_threshold: 0.9,
            max_concepts: 10,
        };
        let nodes = make_nodes(vec![
            ("a/1", vec![1.0, 0.0, 0.0]),
            ("a/2", vec![1.0, 0.0, 0.0]),
            ("a/3", vec![1.0, 0.0, 0.0]),
        ]);
        let clusters = engine.discover_concepts(&nodes);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].member_ids.len(), 3);
    }

    #[test]
    fn below_min_cluster_size_filtered() {
        let engine = ConceptEmergenceEngine {
            min_cluster_size: 3,
            similarity_threshold: 0.9,
            max_concepts: 10,
        };
        let nodes = make_nodes(vec![
            ("a/1", vec![1.0, 0.0]),
            ("a/2", vec![1.0, 0.0]),
        ]);
        let clusters = engine.discover_concepts(&nodes);
        assert!(clusters.is_empty());
    }

    #[test]
    fn separate_clusters() {
        let engine = ConceptEmergenceEngine {
            min_cluster_size: 2,
            similarity_threshold: 0.95,
            max_concepts: 10,
        };
        let nodes = make_nodes(vec![
            ("x/1", vec![1.0, 0.0, 0.0]),
            ("x/2", vec![1.0, 0.0, 0.0]),
            ("y/1", vec![0.0, 1.0, 0.0]),
            ("y/2", vec![0.0, 1.0, 0.0]),
        ]);
        let clusters = engine.discover_concepts(&nodes);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn centroid_computation() {
        let vecs: Vec<Vec<f32>> = vec![vec![2.0, 0.0], vec![0.0, 2.0], vec![2.0, 2.0]];
        let refs: Vec<&Vec<f32>> = vecs.iter().collect();
        let c = compute_centroid(&refs, 2);
        assert!((c[0] - 4.0 / 3.0).abs() < 1e-6);
        assert!((c[1] - 4.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_sim_orthogonal() {
        assert!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
    }

    #[test]
    fn cosine_sim_parallel() {
        let s = cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn max_concepts_limits_output() {
        let engine = ConceptEmergenceEngine {
            min_cluster_size: 2,
            similarity_threshold: 0.9,
            max_concepts: 2,
        };
        let nodes = make_nodes(vec![
            ("a/1", vec![1.0, 0.0, 0.0]),
            ("a/2", vec![1.0, 0.0, 0.0]),
            ("a/3", vec![1.0, 0.0, 0.0]),
            ("b/1", vec![0.0, 1.0, 0.0]),
            ("b/2", vec![0.0, 1.0, 0.0]),
            ("b/3", vec![0.0, 1.0, 0.0]),
            ("c/1", vec![0.0, 0.0, 1.0]),
            ("c/2", vec![0.0, 0.0, 1.0]),
            ("c/3", vec![0.0, 0.0, 1.0]),
        ]);
        let clusters = engine.discover_concepts(&nodes);
        assert!(clusters.len() <= 2);
    }
}
