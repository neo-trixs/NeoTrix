//! Shared evolution types — unified ParentSelection and SelectionConfig.
//!
//! Originally duplicated across `self_iterating::hyperarchive` and `meta_agent`.
//! Merged during F1 dedup pass.

use serde::{Deserialize, Serialize};

/// Strategy for selecting a parent record from the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentSelection {
    ScoreChildProp,
    ScoreProp,
    Best,
    Latest,
    Random,
    DiversityWeighted,
}

/// Configuration controlling archive parent-selection behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionConfig {
    pub strategy: ParentSelection,
    pub temperature: f64,
    pub min_generations: u64,
    pub archive_capacity: usize,
    pub novelty_weight: f64,
    pub diversity_bonus: f64,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            strategy: ParentSelection::Best,
            temperature: 1.0,
            min_generations: 1,
            archive_capacity: 100,
            novelty_weight: 0.3,
            diversity_bonus: 0.3,
        }
    }
}
