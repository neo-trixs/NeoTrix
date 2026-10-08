//! # Git-for-Memory (Memoria pattern)
//!
//! 内存的 Git 风格管理: snapshot/branch/merge/rollback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 内存分支
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBranch {
    pub name: String,
    pub head: Option<String>,
    pub created_at: String,
    pub purpose: String,
}

/// 内存提交
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCommit {
    pub hash: String,
    pub parent: Option<String>,
    pub entries: Vec<String>,
    pub message: String,
    pub timestamp: String,
}

/// 合并结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub merged_entries: Vec<String>,
}

/// Git-for-Memory 存储
pub struct GitMemoryStore {
    pub current_branch: String,
    pub branches: HashMap<String, MemoryBranch>,
    pub commits: Vec<MemoryCommit>,
}

impl Default for GitMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GitMemoryStore {
    pub fn new() -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".into(), MemoryBranch {
            name: "main".into(),
            head: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            purpose: "stable".into(),
        });
        Self {
            current_branch: "main".into(),
            branches,
            commits: vec![],
        }
    }
    
    /// 创建快照
    pub fn snapshot(&mut self, entries: Vec<String>, message: String) -> String {
        let hash = format!("snap_{}", self.commits.len());
        let parent = self.branches.get(&self.current_branch)
            .and_then(|b| b.head.clone());
        
        let commit = MemoryCommit {
            hash: hash.clone(),
            parent,
            entries,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.commits.push(commit);
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head = Some(hash.clone());
        }
        hash
    }
    
    /// 创建分支
    pub fn branch(&mut self, name: &str, purpose: &str) {
        let head = self.branches.get(&self.current_branch)
            .and_then(|b| b.head.clone());
        self.branches.insert(name.into(), MemoryBranch {
            name: name.into(),
            head,
            created_at: chrono::Utc::now().to_rfc3339(),
            purpose: purpose.into(),
        });
    }
    
    /// 切换分支
    pub fn checkout(&mut self, name: &str) -> bool {
        if self.branches.contains_key(name) {
            self.current_branch = name.into();
            true
        } else {
            false
        }
    }
    
    /// 回滚到指定提交
    pub fn rollback(&mut self, commit_hash: &str) -> bool {
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head = Some(commit_hash.into());
            true
        } else {
            false
        }
    }
}
