//! MetaAgent — Self-Referential Loop (HyperAgents fusion, P2-5)
//!
//! Implements the core self-referential improvement architecture:
//! - Evolutionary population archive with parent selection strategies
//! - MetaAgent that observes codebase and proposes modifications
//! - Safety gates (diff filtering, protected paths, rollback)
//! - Staged evaluation protocol for cost control
//!
//! Reference: arXiv:2603.19461 — Jenny Zhang et al. (Meta FAIR)

use crate::neotrix::nt_mind::evolution_types::{ParentSelection, SelectionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod execution;
pub mod planning;
#[cfg(test)]
pub mod reflection;

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
        Self {
            file_path,
            diff_content,
            parent_hash,
        }
    }

    /// Estimate the size of the diff (lines changed)
    pub fn diff_size(&self) -> usize {
        if self.diff_content.is_empty() {
            return 0;
        }
        self.diff_content.lines().count()
    }
}

/// An entry in the evolutionary archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: String,
    pub parent_id: Option<String>,
    pub score: f64,
    #[serde(default)]
    pub diversity_score: f64,
    pub diffs: Vec<FileDiff>,
    pub generation: u64,
    pub timestamp: u64,
    pub lineage: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// A node in the skill tree — one archived skill/concept/code change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTreeNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub score: f64,
    #[serde(default)]
    pub diversity_score: f64,
    pub visit_count: u64,
    pub diffs: Vec<FileDiff>,
    pub generation: u64,
    pub depth: u32,
    pub timestamp: u64,
    pub lineage: Vec<String>,
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub collective_scores: HashMap<String, f64>,
}

impl SkillTreeNode {
    pub fn from_archive_entry(entry: ArchiveEntry) -> Self {
        SkillTreeNode {
            id: entry.id,
            parent_id: entry.parent_id,
            children: Vec::new(),
            score: entry.score,
            diversity_score: entry.diversity_score,
            visit_count: 1,
            diffs: entry.diffs,
            generation: entry.generation,
            depth: entry.lineage.len() as u32,
            timestamp: entry.timestamp,
            lineage: entry.lineage,
            metadata: entry.metadata,
            collective_scores: HashMap::new(),
        }
    }

    pub fn to_archive_entry(&self) -> ArchiveEntry {
        ArchiveEntry {
            id: self.id.clone(),
            parent_id: self.parent_id.clone(),
            score: self.score,
            diversity_score: self.diversity_score,
            diffs: self.diffs.clone(),
            generation: self.generation,
            timestamp: self.timestamp,
            lineage: self.lineage.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn qd_score(&self, diversity_bonus: f64) -> f64 {
        self.score * (1.0 + diversity_bonus * self.diversity_score)
    }

    pub fn uct_score(&self, parent_visits: u64, exploration_constant: f64) -> f64 {
        if self.visit_count == 0 {
            return self.score + exploration_constant * 1000.0;
        }
        self.score
            + exploration_constant * ((parent_visits as f64).ln() / self.visit_count as f64).sqrt()
    }
}

/// The tree-structured archive, replacing flat EvolutionArchive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTreeArchive {
    pub nodes: HashMap<String, SkillTreeNode>,
    pub root_ids: Vec<String>,
    pub config: SelectionConfig,
    next_id_counter: u64,
}

/// Stats snapshot for bridge code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTreeStats {
    pub node_count: usize,
    pub depth: u32,
    pub root_count: usize,
    pub branch_count: usize,
}

impl SkillTreeArchive {
    pub fn new(config: SelectionConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            root_ids: Vec::new(),
            config,
            next_id_counter: 0,
        }
    }

    /// Insert a node into the tree. Returns the node id.
    pub fn add_node(&mut self, mut node: SkillTreeNode) -> String {
        if node.id.is_empty() {
            node.id = format!("node_{}", self.next_id_counter);
            self.next_id_counter += 1;
        }
        let id = node.id.clone();
        if node.parent_id.is_none() && !self.root_ids.contains(&id) {
            self.root_ids.push(id.clone());
        }
        // Wire parent→child edge
        if let Some(parent_id) = &node.parent_id.clone() {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                if !parent.children.contains(&id) {
                    parent.children.push(id.clone());
                }
            }
        }
        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn get_node(&self, id: &str) -> Option<&SkillTreeNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut SkillTreeNode> {
        self.nodes.get_mut(id)
    }

    pub fn root_nodes(&self) -> Vec<&SkillTreeNode> {
        self.root_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    pub fn leaf_nodes(&self) -> Vec<&SkillTreeNode> {
        self.nodes
            .values()
            .filter(|n| n.children.is_empty())
            .collect()
    }

    /// UCT-based selection: score + C * sqrt(ln(total_visits) / visit_count)
    pub fn select_uct(&self, exploration_constant: f64) -> Option<&SkillTreeNode> {
        let total_visits = self.nodes.values().map(|n| n.visit_count).sum::<u64>();
        if total_visits == 0 {
            return self.leaf_nodes().first().copied();
        }
        let ln_total = (total_visits as f64).ln();
        self.leaf_nodes()
            .into_iter()
            .filter(|n| n.visit_count > 0)
            .max_by(|a, b| {
                let uct_a =
                    a.score + exploration_constant * (ln_total / a.visit_count as f64).sqrt();
                let uct_b =
                    b.score + exploration_constant * (ln_total / b.visit_count as f64).sqrt();
                uct_a
                    .partial_cmp(&uct_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Best node by QD score: score * (1.0 + diversity_bonus * diversity_score)
    pub fn best(&self) -> Option<&SkillTreeNode> {
        self.nodes.values().max_by(|a, b| {
            let a_qd = a.score * (1.0 + self.config.diversity_bonus * a.diversity_score);
            let b_qd = b.score * (1.0 + self.config.diversity_bonus * b.diversity_score);
            a_qd.partial_cmp(&b_qd).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Prune low-scoring entries. Retains the most diverse entry as stepping-stone.
    pub fn prune(&mut self, min_score: f64) -> usize {
        let before = self.nodes.len();
        let stepping_stone = self
            .nodes
            .values()
            .max_by(|a, b| {
                a.diversity_score
                    .partial_cmp(&b.diversity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|e| e.id.clone());

        let keep_ids: std::collections::HashSet<String> = self
            .nodes
            .values()
            .filter(|n| n.score >= min_score || Some(&n.id) == stepping_stone.as_ref())
            .map(|n| n.id.clone())
            .collect();

        let mut all_keep = keep_ids.clone();
        for id in &keep_ids {
            let mut current = id.clone();
            while let Some(pid) = self.nodes.get(&current).and_then(|n| n.parent_id.clone()) {
                all_keep.insert(pid.clone());
                current = pid;
            }
        }
        self.nodes.retain(|id, _| all_keep.contains(id));
        self.root_ids.retain(|id| all_keep.contains(id));
        for node in self.nodes.values_mut() {
            node.children.retain(|c| all_keep.contains(c));
        }

        if self.nodes.len() > self.config.archive_capacity {
            let mut scored: Vec<(String, f64)> = self
                .nodes
                .iter()
                .map(|(id, n)| {
                    let qd = n.score * (1.0 + self.config.diversity_bonus * n.diversity_score);
                    (id.clone(), qd)
                })
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let capped: std::collections::HashSet<String> = scored
                .into_iter()
                .take(self.config.archive_capacity)
                .map(|(id, _)| id)
                .collect();
            self.nodes.retain(|id, _| capped.contains(id));
            self.root_ids.retain(|id| capped.contains(id));
            for node in self.nodes.values_mut() {
                node.children.retain(|c| capped.contains(c));
            }
        }
        before - self.nodes.len()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn total_generations(&self) -> u64 {
        self.nodes.values().map(|n| n.generation).max().unwrap_or(0)
    }

    pub fn stats(&self) -> SkillTreeStats {
        let depth = self.nodes.values().map(|n| n.depth).max().unwrap_or(0);
        let branch_count = self.nodes.values().filter(|n| n.children.len() > 1).count();
        SkillTreeStats {
            node_count: self.nodes.len(),
            depth,
            root_count: self.root_ids.len(),
            branch_count,
        }
    }

    /// Flatten tree into a vector of ArchiveEntry (backward compat)
    pub fn entries_as_vec(&self) -> Vec<ArchiveEntry> {
        self.nodes
            .values()
            .map(|n| ArchiveEntry {
                id: n.id.clone(),
                parent_id: n.parent_id.clone(),
                score: n.score,
                diversity_score: n.diversity_score,
                diffs: n.diffs.clone(),
                generation: n.generation,
                timestamp: n.timestamp,
                lineage: n.lineage.clone(),
                metadata: n.metadata.clone(),
            })
            .collect()
    }

    /// Convert an ArchiveEntry into a SkillTreeNode and insert it
    pub fn push_entry(&mut self, entry: ArchiveEntry) -> String {
        let node = SkillTreeNode {
            id: entry.id,
            parent_id: entry.parent_id.clone(),
            children: Vec::new(),
            score: entry.score,
            diversity_score: entry.diversity_score,
            visit_count: 1,
            diffs: entry.diffs,
            generation: entry.generation,
            depth: entry.lineage.len() as u32,
            timestamp: entry.timestamp,
            lineage: entry.lineage,
            metadata: entry.metadata,
            collective_scores: HashMap::new(),
        };
        self.add_node(node)
    }

    pub fn latest_node(&self) -> Option<&SkillTreeNode> {
        self.nodes.values().max_by_key(|n| n.timestamp)
    }

    // --- backward compat with EvolutionArchive API ---

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn add(&mut self, entry: ArchiveEntry) {
        self.push_entry(entry);
    }

    pub fn generate_id(&mut self) -> String {
        self.next_id_counter += 1;
        format!("st_{}", self.next_id_counter)
    }

    pub fn select_parent(&self) -> Option<&SkillTreeNode> {
        self.select_parent_with_strategy(&self.config.strategy)
    }

    fn select_parent_with_strategy(&self, strategy: &ParentSelection) -> Option<&SkillTreeNode> {
        match strategy {
            ParentSelection::Best => self.best(),
            ParentSelection::Latest => self
                .nodes
                .values()
                .max_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            ParentSelection::Random => {
                let leaves = self.leaf_nodes();
                if leaves.is_empty() {
                    return None;
                }
                let idx = rand::random::<usize>() % leaves.len();
                Some(leaves[idx])
            }
            _ => self.select_uct(1.414),
        }
    }

    /// Convert old flat EvolutionArchive to tree structure
    #[allow(deprecated)]
    pub fn from_flat_archive(flat: &EvolutionArchive) -> Self {
        let mut tree = SkillTreeArchive::new(flat.config.clone());
        // First pass: insert all nodes
        for entry in &flat.entries {
            let node = SkillTreeNode::from_archive_entry(entry.clone());
            tree.nodes.insert(entry.id.clone(), node);
        }
        // Second pass: wire parent→child edges and identify roots
        for entry in &flat.entries {
            let id = &entry.id;
            if let Some(ref parent_id) = entry.parent_id {
                if let Some(parent) = tree.nodes.get_mut(parent_id) {
                    if !parent.children.contains(id) {
                        parent.children.push(id.clone());
                    }
                }
            } else if !tree.root_ids.contains(id) {
                tree.root_ids.push(id.clone());
            }
        }
        if tree.root_ids.is_empty() {
            tree.root_ids = tree.nodes.keys().cloned().collect();
        }
        tree
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        serde_json::to_vec(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        serde_json::from_slice(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let data = self.to_bytes()?;
        let p = std::path::Path::new(path);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, data)?;
        std::fs::rename(&tmp, p)
    }

    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        match std::fs::read(path) {
            Ok(data) => Self::from_bytes(&data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::new(SelectionConfig::default()))
            }
            Err(e) => Err(e),
        }
    }
}

/// Deprecated — preserved for deserializing old-format archives during migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(note = "Use SkillTreeArchive instead")]
pub struct EvolutionArchive {
    pub entries: Vec<ArchiveEntry>,
    pub config: SelectionConfig,
}

#[allow(deprecated)]
impl EvolutionArchive {
    pub fn new(config: SelectionConfig) -> Self {
        Self {
            entries: Vec::new(),
            config,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn add(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    pub fn select_by_best(&self) -> Option<&ArchiveEntry> {
        self.entries.iter().max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn select_by_latest(&self) -> Option<&ArchiveEntry> {
        self.entries.iter().max_by_key(|e| e.timestamp)
    }

    pub fn select_by_random(&self) -> Option<&ArchiveEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let idx = rand::random::<usize>() % self.entries.len();
        self.entries.get(idx)
    }

    pub fn select_parent(&self) -> Option<&ArchiveEntry> {
        self.select_by_best()
    }

    pub fn prune(&mut self, min_score: f64) -> usize {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.score >= min_score);
        len_before - self.entries.len()
    }

    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        match std::fs::read(path) {
            Ok(data) => serde_json::from_slice(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::new(SelectionConfig::default()))
            }
            Err(e) => Err(e),
        }
    }
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
    pub archive: SkillTreeArchive,
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
