//! Git Memory Store — Git-for-Memory (M2 from Memoria)
//!
//! Snapshot/branch/merge/rollback for knowledge. Each write creates a snapshot;
//! branches isolate experiments; merges reconcile divergent memory states;
//! rollback restores a prior knowledge configuration.

use std::collections::HashMap;
use std::fmt;

/// Git memory store error type
#[derive(Debug, Clone)]
pub enum GitStoreError {
    /// Snapshot not found
    SnapshotNotFound(String),
    /// Branch not found
    BranchNotFound(String),
    /// Branch already exists
    BranchExists(String),
    /// Merge conflict — entries diverged
    MergeConflict {
        source: String,
        target: String,
        conflicting_keys: Vec<String>,
    },
    /// Merge cooldown — too soon since last merge
    MergeCooldown {
        last_merge_at: u64,
        cooldown_secs: u64,
    },
    /// Rollback target not in history
    RollbackTargetInvalid(String),
}

impl fmt::Display for GitStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SnapshotNotFound(id) => write!(f, "snapshot not found: {id}"),
            Self::BranchNotFound(name) => write!(f, "branch not found: {name}"),
            Self::BranchExists(name) => write!(f, "branch already exists: {name}"),
            Self::MergeConflict {
                source,
                target,
                conflicting_keys,
            } => write!(
                f,
                "merge conflict {source}→{target}: {} keys",
                conflicting_keys.len()
            ),
            Self::MergeCooldown {
                last_merge_at,
                cooldown_secs,
            } => write!(
                f,
                "merge cooldown: {cooldown_secs}s since last merge at {last_merge_at}"
            ),
            Self::RollbackTargetInvalid(hash) => {
                write!(f, "rollback target not in history: {hash}")
            }
        }
    }
}

impl std::error::Error for GitStoreError {}

pub type Result<T> = std::result::Result<T, GitStoreError>;

/// Snapshot identifier (content-addressed hash)
pub type SnapshotId = String;

/// A single memory entry in a snapshot
#[derive(Debug, Clone)]
pub struct GitStoreGitStoreMemoryEntry {
    /// Unique key
    pub key: String,
    /// Entry value (arbitrary bytes)
    pub value: Vec<u8>,
    /// Timestamp of last modification
    pub modified_at: u64,
}

/// A snapshot — immutable point-in-time memory state
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Content hash
    pub id: SnapshotId,
    /// Parent snapshot (None for initial)
    pub parent: Option<SnapshotId>,
    /// Memory entries in this snapshot
    pub entries: Vec<GitStoreMemoryEntry>,
    /// Human-readable message
    pub message: String,
    /// Creation timestamp
    pub timestamp: u64,
}

/// Branch — a named pointer to a snapshot
#[derive(Debug, Clone)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Head snapshot
    pub head: SnapshotId,
    /// Purpose tag
    pub purpose: BranchPurpose,
}

/// Branch purpose classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchPurpose {
    /// Stable production memory
    Stable,
    /// Experimental / temporary
    Experiment,
    /// Read-only archive
    Archive,
}

/// Merge result
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// Fast-forward — source was ahead of target
    FastForward {
        new_head: SnapshotId,
        entries_applied: usize,
    },
    /// Three-way merge — diverged branches reconciled
    Merged {
        new_head: SnapshotId,
        entries_applied: usize,
        conflicts_resolved: usize,
    },
    /// No changes to merge
    AlreadyUpToDate,
}

/// Governance configuration for merge safety
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Cooldown period between merges (seconds)
    pub merge_cooldown_secs: u64,
    /// Auto-archive branches older than this (seconds)
    pub auto_archive_threshold_secs: u64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            merge_cooldown_secs: 300,
            auto_archive_threshold_secs: 86400,
        }
    }
}

/// Git-for-Memory store — snapshot/branch/merge/rollback for knowledge
#[derive(Debug)]
pub struct GitMemoryStore {
    /// Current branch name
    current_branch: String,
    /// All branches: name → Branch
    branches: HashMap<String, Branch>,
    /// All snapshots: id → Snapshot
    snapshots: HashMap<SnapshotId, Snapshot>,
    /// Last merge timestamp
    last_merge_at: u64,
    /// Governance configuration
    governance: GovernanceConfig,
}

impl GitMemoryStore {
    /// Create a new store with an initial "main" branch
    pub fn new(now: u64) -> Self {
        let initial_id = "init".to_string();
        let initial = Snapshot {
            id: initial_id.clone(),
            parent: None,
            entries: Vec::new(),
            message: "initial commit".into(),
            timestamp: now,
        };

        let mut snapshots = HashMap::new();
        snapshots.insert(initial_id.clone(), initial);

        let mut branches = HashMap::new();
        branches.insert(
            "main".to_string(),
            Branch {
                name: "main".into(),
                head: initial_id.clone(),
                purpose: BranchPurpose::Stable,
            },
        );

        Self {
            current_branch: "main".into(),
            branches,
            snapshots,
            last_merge_at: now,
            governance: GovernanceConfig::default(),
        }
    }

    /// Current HEAD snapshot
    pub fn head(&self) -> Option<&Snapshot> {
        self.branches
            .get(&self.current_branch)
            .and_then(|b| self.snapshots.get(&b.head))
    }

    /// Current branch name
    pub fn current_branch(&self) -> &str {
        &self.current_branch
    }

    /// Create a snapshot (commit) on the current branch
    pub fn snapshot(&mut self, entries: Vec<GitStoreMemoryEntry>, message: String, now: u64) -> Result<SnapshotId> {
        let parent = self.head().map(|s| s.id.clone());
        let id = format!("snap_{}", now);

        let snap = Snapshot {
            id: id.clone(),
            parent,
            entries,
            message,
            timestamp: now,
        };

        self.snapshots.insert(id.clone(), snap);
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head = id.clone();
        }

        Ok(id)
    }

    /// Create a new branch from a snapshot
    pub fn branch(&mut self, from: &SnapshotId, name: &str, purpose: BranchPurpose) -> Result<()> {
        if self.branches.contains_key(name) {
            return Err(GitStoreError::BranchExists(name.into()));
        }
        if !self.snapshots.contains_key(from) {
            return Err(GitStoreError::SnapshotNotFound(from.into()));
        }

        self.branches.insert(
            name.to_string(),
            Branch {
                name: name.into(),
                head: from.clone(),
                purpose,
            },
        );
        Ok(())
    }

    /// Switch to an existing branch
    pub fn checkout(&mut self, name: &str) -> Result<()> {
        if !self.branches.contains_key(name) {
            return Err(GitStoreError::BranchNotFound(name.into()));
        }
        self.current_branch = name.to_string();
        Ok(())
    }

    /// Merge source branch into target branch
    pub fn merge(
        &mut self,
        source: &SnapshotId,
        target: &SnapshotId,
        now: u64,
    ) -> Result<MergeResult> {
        let elapsed = now.saturating_sub(self.last_merge_at);
        if elapsed < self.governance.merge_cooldown_secs {
            return Err(GitStoreError::MergeCooldown {
                last_merge_at: self.last_merge_at,
                cooldown_secs: self.governance.merge_cooldown_secs - elapsed,
            });
        }

        let source_snap = self
            .snapshots
            .get(source)
            .ok_or_else(|| GitStoreError::SnapshotNotFound(source.into()))?;

        let target_snap = self
            .snapshots
            .get(target)
            .ok_or_else(|| GitStoreError::SnapshotNotFound(target.into()))?;

        // Check if already up to date
        if source_snap.parent.as_ref() == Some(target) {
            self.last_merge_at = now;
            return Ok(MergeResult::AlreadyUpToDate);
        }

        // Merge entries — source entries override target entries on key conflict
        let mut merged_entries: Vec<GitStoreMemoryEntry> = target_snap.entries.clone();
        let mut applied = 0;
        let mut conflicts = 0;

        for source_entry in &source_snap.entries {
            if let Some(target_entry) = merged_entries.iter_mut().find(|e| e.key == source_entry.key)
            {
                if target_entry.value != source_entry.value {
                    conflicts += 1;
                }
                *target_entry = source_entry.clone();
            } else {
                merged_entries.push(source_entry.clone());
            }
            applied += 1;
        }

        let new_id = format!("merge_{}", now);
        let new_snap = Snapshot {
            id: new_id.clone(),
            parent: Some(target.clone()),
            entries: merged_entries,
            message: format!("merge {source} into {target}"),
            timestamp: now,
        };

        self.snapshots.insert(new_id.clone(), new_snap);
        self.last_merge_at = now;

        Ok(MergeResult::Merged {
            new_head: new_id,
            entries_applied: applied,
            conflicts_resolved: conflicts,
        })
    }

    /// Rollback current branch to a prior snapshot
    pub fn rollback(&mut self, to: &SnapshotId) -> Result<()> {
        if !self.snapshots.contains_key(to) {
            return Err(GitStoreError::SnapshotNotFound(to.into()));
        }

        // Walk history to verify target is an ancestor
        let mut current = self.head().map(|s| s.id.clone());
        let mut found = false;
        while let Some(ref id) = current {
            if id == to {
                found = true;
                break;
            }
            current = self
                .snapshots
                .get(id)
                .and_then(|s| s.parent.clone());
        }

        if !found {
            return Err(GitStoreError::RollbackTargetInvalid(to.into()));
        }

        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head = to.clone();
        }

        Ok(())
    }

    /// Walk commit log from HEAD backwards
    pub fn log(&self, max_entries: usize) -> Vec<&Snapshot> {
        let mut result = Vec::new();
        let mut current = self.head().map(|s| s.id.clone());

        while let Some(ref id) = current {
            if result.len() >= max_entries {
                break;
            }
            if let Some(snap) = self.snapshots.get(id) {
                result.push(snap);
                current = snap.parent.clone();
            } else {
                break;
            }
        }

        result
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get branch count
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let store = GitMemoryStore::new(1000);
        assert_eq!(store.current_branch(), "main");
        assert_eq!(store.snapshot_count(), 1);
        assert_eq!(store.branch_count(), 1);
    }

    #[test]
    fn test_snapshot_and_checkout() {
        let mut store = GitMemoryStore::new(1000);
        let entries = vec![GitStoreMemoryEntry {
            key: "k1".into(),
            value: b"v1".to_vec(),
            modified_at: 1000,
        }];
        let id = store.snapshot(entries, "add k1".into(), 2000).unwrap();
        assert_eq!(id, "snap_2000");

        let branch = store.branches.get("main").unwrap();
        assert_eq!(branch.head, id);
    }

    #[test]
    fn test_branch_creation() {
        let mut store = GitMemoryStore::new(1000);
        let head = store.head().unwrap().id.clone();
        store
            .branch(&head, "experiment", BranchPurpose::Experiment)
            .unwrap();
        assert_eq!(store.branch_count(), 2);

        // Duplicate branch name fails
        assert!(store
            .branch(&head, "experiment", BranchPurpose::Experiment)
            .is_err());
    }

    #[test]
    fn test_rollback() {
        let mut store = GitMemoryStore::new(1000);
        let id1 = store
            .snapshot(vec![], "first".into(), 2000)
            .unwrap();
        let _ = store
            .snapshot(vec![], "second".into(), 3000)
            .unwrap();

        store.rollback(&id1).unwrap();
        assert_eq!(store.head().unwrap().id, id1);
    }

    #[test]
    fn test_log() {
        let mut store = GitMemoryStore::new(1000);
        let _ = store.snapshot(vec![], "commit 1".into(), 2000);
        let _ = store.snapshot(vec![], "commit 2".into(), 3000);

        let log = store.log(10);
        assert_eq!(log.len(), 3); // initial + 2 commits
    }

    #[test]
    fn test_merge_cooldown() {
        let mut store = GitMemoryStore::new(1000);
        store.governance.merge_cooldown_secs = 100;

        let head = store.head().unwrap().id.clone();
        store
            .branch(&head, "other", BranchPurpose::Experiment)
            .unwrap();
        store.checkout("other").unwrap();
        let other_head = store.head().unwrap().id.clone();

        store.checkout("main").unwrap();
        let main_head = store.head().unwrap().id.clone();

        // First merge succeeds
        let _ = store.merge(&other_head, &main_head, 2000);

        // Second merge within cooldown fails
        let result = store.merge(&other_head, &main_head, 2100);
        assert!(result.is_err());
    }
}
