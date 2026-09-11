pub struct BehaviorDescriptor {
    pub agent_id: u32,
    pub features: Vec<f32>,
    pub novelty_score: f32,
}

pub struct NoveltySearch {
    pub archive: Vec<BehaviorDescriptor>,
    pub k_neighbors: usize,
}

impl NoveltySearch {
    pub fn new() -> Self {
        NoveltySearch {
            archive: Vec::new(),
            k_neighbors: 5,
        }
    }

    pub fn calculate_novelty(&self, features: &[f32]) -> f32 {
        if self.archive.is_empty() {
            return 1.0;
        }
        let mut distances: Vec<f32> = self
            .archive
            .iter()
            .map(|a| {
                features
                    .iter()
                    .zip(a.features.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f32>()
                    .sqrt()
            })
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        distances.iter().take(self.k_neighbors).sum::<f32>()
            / self.k_neighbors.min(distances.len()) as f32
    }

    pub fn add_to_archive(&mut self, descriptor: BehaviorDescriptor) {
        self.archive.push(descriptor);
    }

    pub fn is_novel(&self, features: &[f32], threshold: f32) -> bool {
        self.calculate_novelty(features) > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_archive_returns_one() {
        let ns = NoveltySearch::new();
        assert_eq!(ns.calculate_novelty(&[0.5, 0.5]), 1.0);
    }

    #[test]
    fn identical_descriptor_returns_zero() {
        let mut ns = NoveltySearch::new();
        ns.add_to_archive(BehaviorDescriptor {
            agent_id: 0,
            features: vec![0.5, 0.5],
            novelty_score: 0.0,
        });
        assert_eq!(ns.calculate_novelty(&[0.5, 0.5]), 0.0);
    }

    #[test]
    fn is_novel_above_threshold() {
        let mut ns = NoveltySearch::new();
        ns.add_to_archive(BehaviorDescriptor {
            agent_id: 0,
            features: vec![0.0, 0.0],
            novelty_score: 0.0,
        });
        assert!(ns.is_novel(&[1.0, 1.0], 0.5));
        assert!(!ns.is_novel(&[0.0, 0.0], 0.5));
    }
}
