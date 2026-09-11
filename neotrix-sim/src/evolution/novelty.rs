use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BehaviorDescriptor {
    pub agent_id: u32,
    pub traits: Vec<f32>,
    pub novelty_score: f32,
}

pub struct NoveltySearch {
    pub archive: Vec<BehaviorDescriptor>,
    pub k_neighbors: usize,
    pub threshold: f32,
}

impl NoveltySearch {
    pub fn new() -> Self {
        Self {
            archive: Vec::new(),
            k_neighbors: 15,
            threshold: 0.1,
        }
    }

    pub fn add_to_archive(&mut self, descriptor: BehaviorDescriptor) -> bool {
        if self.novelty_score(&descriptor) > self.threshold {
            self.archive.push(descriptor);
            true
        } else {
            false
        }
    }

    pub fn novelty_score(&self, candidate: &BehaviorDescriptor) -> f32 {
        if self.archive.is_empty() {
            return 1.0;
        }

        let mut distances: Vec<f32> = self.archive.iter()
            .map(|a| self.distance(&candidate.traits, &a.traits))
            .collect();

        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let k = self.k_neighbors.min(distances.len());
        distances[..k].iter().sum::<f32>() / k as f32
    }

    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn compute_novelty(&mut self, descriptors: &mut [BehaviorDescriptor]) {
        for desc in descriptors.iter_mut() {
            desc.novelty_score = self.novelty_score(desc);
        }
    }

    pub fn select_novel(&self, candidates: &[BehaviorDescriptor], n: usize) -> Vec<usize> {
        let mut indexed: Vec<(usize, f32)> = candidates.iter()
            .enumerate()
            .map(|(i, d)| (i, d.novelty_score))
            .collect();
        
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.into_iter().take(n).map(|(i, _)| i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novelty_empty_archive() {
        let search = NoveltySearch::new();
        let desc = BehaviorDescriptor {
            agent_id: 0,
            traits: vec![1.0, 2.0, 3.0],
            novelty_score: 0.0,
        };
        assert_eq!(search.novelty_score(&desc), 1.0);
    }

    #[test]
    fn test_novelty_with_archive() {
        let mut search = NoveltySearch::new();
        search.archive.push(BehaviorDescriptor {
            agent_id: 0,
            traits: vec![1.0, 2.0, 3.0],
            novelty_score: 0.0,
        });
        
        let desc = BehaviorDescriptor {
            agent_id: 1,
            traits: vec![1.0, 2.0, 3.0],
            novelty_score: 0.0,
        };
        
        assert_eq!(search.novelty_score(&desc), 0.0);
    }

    #[test]
    fn test_novelty_add_archive() {
        let mut search = NoveltySearch::new();
        let desc = BehaviorDescriptor {
            agent_id: 0,
            traits: vec![1.0, 2.0, 3.0],
            novelty_score: 0.0,
        };
        assert!(search.add_to_archive(desc));
        assert_eq!(search.archive.len(), 1);
    }
}
