use super::FileDiff;
use super::SelfModificationProposal;
use crate::neotrix::nt_mind::evolution_types::{ParentSelection, SelectionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single entry in the HyperAgentArchive, recording one self-modification attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperAgentRecord {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub diffs_applied: Vec<FileDiff>,
    pub score: Option<f64>,
    pub novelty_score: f64,
    pub latent_snapshot: Vec<f64>,
    pub generation: u64,
    pub proposal: SelfModificationProposal,
}

/// Population archive of HyperAgentRecords with parent-selection strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperAgentArchive {
    pub records: Vec<HyperAgentRecord>,
    pub config: SelectionConfig,
}

impl HyperAgentArchive {
    pub fn new(config: SelectionConfig) -> Self {
        Self {
            records: Vec::new(),
            config,
        }
    }

    pub fn add_record(&mut self, record: HyperAgentRecord) {
        self.records.push(record);
        if self.records.len() > self.config.archive_capacity {
            self.records.sort_by(|a, b| {
                let score_a = a.score.unwrap_or(0.0) + a.novelty_score * self.config.novelty_weight;
                let score_b = b.score.unwrap_or(0.0) + b.novelty_score * self.config.novelty_weight;
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.records.truncate(self.config.archive_capacity);
        }
    }

    pub fn select_parent(&self) -> Option<&HyperAgentRecord> {
        if self.records.is_empty() {
            return None;
        }
        match self.config.strategy {
            ParentSelection::Best => self
                .records
                .iter()
                .filter_map(|r| r.score.map(|s| (s, r)))
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, r)| r)
                .or_else(|| self.records.last()),
            ParentSelection::Latest => self.records.last(),
            ParentSelection::Random => self.records.first(),
            ParentSelection::ScoreProp => self.select_score_proportional(),
            ParentSelection::ScoreChildProp => self.select_score_child_prop(),
            ParentSelection::DiversityWeighted => self.select_diversity_weighted(),
        }
    }

    fn select_score_proportional(&self) -> Option<&HyperAgentRecord> {
        let scored: Vec<(&HyperAgentRecord, f64)> = self
            .records
            .iter()
            .filter_map(|r| r.score.map(|s| (r, s.max(0.0) + 0.01)))
            .collect();
        if scored.is_empty() {
            return self.records.last();
        }
        let total: f64 = scored.iter().map(|(_, s)| s).sum();
        let mut r = rand::random::<f64>() * total;
        for (record, score) in &scored {
            r -= score;
            if r <= 0.0 {
                return Some(record);
            }
        }
        scored.last().map(|(r, _)| *r)
    }

    fn select_score_child_prop(&self) -> Option<&HyperAgentRecord> {
        let child_counts: HashMap<Uuid, usize> = self
            .records
            .iter()
            .filter_map(|r| r.parent_id)
            .fold(HashMap::new(), |mut acc, pid| {
                *acc.entry(pid).or_insert(0) += 1;
                acc
            });
        let scored: Vec<(&HyperAgentRecord, f64)> = self
            .records
            .iter()
            .map(|r| {
                let score = r.score.unwrap_or(0.0).max(0.0) + 0.01;
                let children = child_counts.get(&r.id).copied().unwrap_or(1) as f64;
                (r, score * children)
            })
            .collect();
        if scored.is_empty() {
            return self.records.last();
        }
        let total: f64 = scored.iter().map(|(_, s)| s).sum();
        let mut r = rand::random::<f64>() * total;
        for (record, score) in &scored {
            r -= score;
            if r <= 0.0 {
                return Some(record);
            }
        }
        scored.last().map(|(r, _)| *r)
    }

    fn select_diversity_weighted(&self) -> Option<&HyperAgentRecord> {
        self.records
            .iter()
            .map(|r| {
                let base_score = r.score.unwrap_or(0.0);
                let weighted = base_score * (1.0 + self.config.novelty_weight * r.novelty_score);
                (weighted, r)
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, r)| r)
            .or_else(|| self.records.last())
    }

    pub fn compute_novelty(&self, latent: &[f64]) -> f64 {
        if self.records.is_empty() {
            return 1.0;
        }
        self.records
            .iter()
            .map(|r| cosine_distance(latent, &r.latent_snapshot))
            .fold(0.0_f64, f64::max)
    }

    pub fn latest_generation(&self) -> u64 {
        self.records.iter().map(|r| r.generation).max().unwrap_or(0)
    }

    pub fn best_score(&self) -> Option<f64> {
        self.records
            .iter()
            .filter_map(|r| r.score)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn sample_stepping_stones(&self, n: usize) -> Vec<&HyperAgentRecord> {
        let mut indices: Vec<usize> = (0..self.records.len()).collect();
        if indices.len() > n {
            use rand::seq::SliceRandom;
            indices.shuffle(&mut rand::thread_rng());
            indices.truncate(n);
        }
        indices.iter().map(|&i| &self.records[i]).collect()
    }

    pub fn total_stepping_stone_energy(&self) -> f64 {
        self.records.iter().filter_map(|r| r.score).sum()
    }

    pub fn stepping_stone_score(&self) -> f64 {
        self.best_score().unwrap_or(0.0)
    }
}

pub(crate) fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let dim = a.len().min(b.len());
    if dim == 0 {
        return 0.0;
    }
    let dot: f64 = (0..dim).map(|i| a[i] * b[i]).sum();
    let norm_a: f64 = (0..dim).map(|i| a[i] * a[i]).sum::<f64>().sqrt();
    let norm_b: f64 = (0..dim).map(|i| b[i] * b[i]).sum::<f64>().sqrt();
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 1.0;
    }
    let cos_sim = (dot / (norm_a * norm_b)).clamp(-1.0, 1.0);
    1.0 - (cos_sim + 1.0) / 2.0
}
