//! MetaAgent — Self-Referential Loop (HyperAgents fusion, P2-5)
//!
//! Implements the core self-referential improvement architecture:
//! - Evolutionary population archive with parent selection strategies
//! - MetaAgent that observes codebase and proposes modifications
//! - Safety gates (diff filtering, protected paths, rollback)
//! - Staged evaluation protocol for cost control
//!
//! Reference: arXiv:2603.19461 — Jenny Zhang et al. (Meta FAIR)

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod execution;
pub mod planning;
pub mod reflection;

pub use execution::*;
pub use planning::*;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A code diff patch file (HyperAgents-style .diff format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: PathBuf,
    pub diff_content: String,
    pub parent_hash: String,
}

impl FileDiff {
    pub fn new(file_path: PathBuf, diff_content: String, parent_hash: String) -> Self {
        Self { file_path, diff_content, parent_hash }
    }

    /// Estimate the size of the diff (lines changed)
    pub fn diff_size(&self) -> usize {
        if self.diff_content.is_empty() {
            return 0;
        }
        self.diff_content.lines().count()
    }
}

/// Parent selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentSelection {
    ScoreChildProp,
    ScoreProp,
    Best,
    Latest,
    Random,
}

/// Parent selection config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionConfig {
    pub strategy: ParentSelection,
    pub temperature: f64,
    pub min_generations: u64,
    pub archive_capacity: usize,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            strategy: ParentSelection::Best,
            temperature: 1.0,
            min_generations: 1,
            archive_capacity: 100,
        }
    }
}

/// An entry in the evolutionary archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: String,
    pub parent_id: Option<String>,
    pub score: f64,
    pub diffs: Vec<FileDiff>,
    pub generation: u64,
    pub timestamp: u64,
    pub lineage: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// The evolutionary archive (population)
pub struct EvolutionArchive {
    pub entries: Vec<ArchiveEntry>,
    pub config: SelectionConfig,
}

/// What the modification targets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModificationTarget {
    TaskAgent,
    MetaAgent,
    ImprovementMechanism,
    CapabilityExtension,
}

/// Safety check result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SafetyCheckResult {
    Passed,
    Failed { reason: String },
    NeedsHumanReview { concern: String },
}

/// A modification proposal from the meta-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationProposal {
    pub target: ModificationTarget,
    pub diffs: Vec<FileDiff>,
    pub expected_impact: String,
    pub safety_check: SafetyCheckResult,
}

/// A hyperagent = the combination of task capability + meta capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperAgent {
    pub id: String,
    pub parent_id: Option<String>,
    pub score: Option<f64>,
    pub diffs_applied: Vec<FileDiff>,
    pub generation: u64,
}

/// Configuration for the meta agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAgentConfig {
    pub budget: u32,
    pub self_referential: bool,
    pub protected_paths: Vec<String>,
    pub llm_temperature: f64,
}

impl Default for MetaAgentConfig {
    fn default() -> Self {
        Self {
            budget: 10,
            self_referential: true,
            protected_paths: vec!["domains/".into(), "tests/".into()],
            llm_temperature: 0.7,
        }
    }
}

/// Stage evaluation protocol — eval in stages to save cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEvaluation {
    pub subset_size: usize,
    pub full_size: usize,
    pub subset_threshold: f64,
}

impl Default for StagedEvaluation {
    fn default() -> Self {
        Self {
            subset_size: 10,
            full_size: 100,
            subset_threshold: 0.3,
        }
    }
}

/// The meta agent — observes and proposes code modifications
pub struct MetaAgent {
    pub config: MetaAgentConfig,
    pub archive: EvolutionArchive,
    pub eval_config: StagedEvaluation,
    pub iteration: u64,
}

/// Result of a single generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub generation: u64,
    pub proposals_generated: usize,
    pub proposals_accepted: usize,
    pub best_score: f64,
    pub archive_size: usize,
    pub rollbacks: usize,
}
