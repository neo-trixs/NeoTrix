
use rand::Rng;

use super::{
    EvolutionArchive, ArchiveEntry, SelectionConfig, ParentSelection,
    MetaAgent,
};

impl EvolutionArchive {
    pub fn new(config: SelectionConfig) -> Self {
        Self { entries: Vec::new(), config }
    }

    pub fn add(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    /// Select a parent based on the configured strategy
    pub fn select_parent(&self) -> Option<&ArchiveEntry> {
        if self.entries.is_empty() {
            return None;
        }
        match self.config.strategy {
            ParentSelection::Best => self.select_by_best(),
            ParentSelection::Latest => self.select_by_latest(),
            ParentSelection::Random => self.select_by_random(),
            ParentSelection::ScoreProp => self.select_by_score_prop(),
            ParentSelection::ScoreChildProp => self.select_by_score_child_prop(),
        }
    }

    pub fn select_by_best(&self) -> Option<&ArchiveEntry> {
        self.entries.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn select_by_latest(&self) -> Option<&ArchiveEntry> {
        self.entries.iter().max_by_key(|e| e.timestamp)
    }

    pub fn select_by_random(&self) -> Option<&ArchiveEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let idx = rand::thread_rng().gen_range(0..self.entries.len());
        Some(&self.entries[idx])
    }

    fn select_by_score_prop(&self) -> Option<&ArchiveEntry> {
        let total: f64 = self.entries.iter().map(|e| e.score.max(0.0)).sum();
        if total <= 0.0 {
            return self.select_by_random();
        }
        let mut r = rand::thread_rng().gen_range(0.0..total);
        for entry in &self.entries {
            r -= entry.score.max(0.0);
            if r <= 0.0 {
                return Some(entry);
            }
        }
        self.entries.last()
    }

    fn select_by_score_child_prop(&self) -> Option<&ArchiveEntry> {
        self.select_by_score_prop()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the best entry by score
    pub fn best(&self) -> Option<&ArchiveEntry> {
        self.entries.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Prune low-scoring entries when at capacity
    pub fn prune(&mut self, min_score: f64) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| e.score >= min_score);
        if self.entries.len() > self.config.archive_capacity {
            self.entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            self.entries.truncate(self.config.archive_capacity);
        }
        before - self.entries.len()
    }
}

impl MetaAgent {
    /// Should this meta-agent continue improving?
    pub fn should_continue(&self) -> bool {
        self.iteration < self.config.budget as u64
    }

    /// Rollback mechanism: if score drops below parent, discard
    pub fn should_rollback(&self, child_score: f64, parent_score: f64) -> bool {
        child_score < parent_score * 0.9
    }
}
