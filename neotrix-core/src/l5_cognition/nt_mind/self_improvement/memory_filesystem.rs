//! Memory-as-Filesystem — MemGPT-inspired self-editable memory blocks
//!
//! Agent can modify its own memory blocks mid-conversation (R-MEM09).
//! Blocks are labeled ("core_memory", "archive_memory") and stored in a HashMap
//! for O(1) access. Each block tracks creation and update timestamps.
//!
//! Design: Each MemoryBlock is an atomic unit the agent can read/write/create.
//! MemoryFilesystem manages the block store with label-based indexing and
//! capacity enforcement per label tier.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Types
// ============================================================

/// Memory block label — matches MemGPT's core/recall/archive tiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLabel {
    /// Core memory — always in context, agent-editable
    Core,
    /// Archive memory — compressed, offloaded
    Archive,
    /// Recall memory — recent conversation turns
    Recall,
    /// Custom label for domain-specific blocks
    Custom(String),
}

impl MemoryLabel {
    pub fn as_str(&self) -> &str {
        match self {
            MemoryLabel::Core => "core_memory",
            MemoryLabel::Archive => "archive_memory",
            MemoryLabel::Recall => "recall_memory",
            MemoryLabel::Custom(s) => s,
        }
    }

    pub fn default_capacity(&self) -> usize {
        match self {
            MemoryLabel::Core => 50,
            MemoryLabel::Archive => 500,
            MemoryLabel::Recall => 200,
            MemoryLabel::Custom(_) => 100,
        }
    }
}

/// A single memory block — the atomic unit of agent-editable memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub id: String,
    pub label: MemoryLabel,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
    /// 回合域的「最近活跃回合」。
    ///
    /// ⚠️ `updated_at` 是 **Unix 秒**（见 `age_secs()`），而 `SleepComputer`
    /// 按**回合**推进；原实现用 `updated_at % (current_turn+1)` 把两者混算
    /// （源码自称pseudo-age）⇒ age 取决于挂钟取模，**同一次运行可绿可红**。
    /// 回合语义的年龄一律走本字段。
    #[serde(default)]
    pub last_access_turn: u64,
    pub access_count: u64,
    pub tags: Vec<String>,
}

impl MemoryBlock {
    pub fn new(id: impl Into<String>, label: MemoryLabel, content: impl Into<String>) -> Self {
        let now = unix_now();
        Self {
            id: id.into(),
            label,
            content: content.into(),
            created_at: now,
            updated_at: now,
            last_access_turn: 0,
            access_count: 0,
            tags: Vec::new(),
        }
    }

    pub fn age_secs(&self) -> u64 {
        unix_now().saturating_sub(self.updated_at)
    }

    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.updated_at = unix_now();
    }
}

/// Edit proposal for a memory block — sent through the self-edit pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEditProposal {
    pub block_id: String,
    pub old_content: String,
    pub new_content: String,
    pub reason: String,
    pub proposed_at: u64,
}

/// Result of applying an edit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditResult {
    Applied,
    Rejected(String),
    BlockNotFound(String),
}

// ============================================================
// MemoryFilesystem
// ============================================================

/// MemGPT-inspired filesystem for agent-editable memory blocks.
///
/// The agent can create, read, update, and list memory blocks.
/// Blocks are indexed by label for efficient retrieval.
#[derive(Debug, Clone)]
pub struct MemoryFilesystem {
    pub(crate) blocks: HashMap<String, MemoryBlock>,
    pub(crate) label_index: HashMap<MemoryLabel, Vec<String>>,
    label_capacities: HashMap<MemoryLabel, usize>,
}

impl Default for MemoryFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryFilesystem {
    pub fn new() -> Self {
        let mut label_index = HashMap::new();
        label_index.insert(MemoryLabel::Core, Vec::new());
        label_index.insert(MemoryLabel::Archive, Vec::new());
        label_index.insert(MemoryLabel::Recall, Vec::new());

        let mut label_capacities = HashMap::new();
        label_capacities.insert(MemoryLabel::Core, MemoryLabel::Core.default_capacity());
        label_capacities.insert(MemoryLabel::Archive, MemoryLabel::Archive.default_capacity());
        label_capacities.insert(MemoryLabel::Recall, MemoryLabel::Recall.default_capacity());

        Self {
            blocks: HashMap::new(),
            label_index,
            label_capacities,
        }
    }

    /// Create a new memory block (R-MEM09: agent self-editable)
    pub fn create_block(
        &mut self,
        id: impl Into<String>,
        label: MemoryLabel,
        content: impl Into<String>,
    ) -> Result<&MemoryBlock, String> {
        let id = id.into();
        if self.blocks.contains_key(&id) {
            return Err(format!("block '{}' already exists", id));
        }

        let capacity = self.label_capacities.get(&label).copied().unwrap_or(100);
        let ids = self.label_index.entry(label.clone()).or_insert_with(Vec::new);

        if ids.len() >= capacity {
            // Evict oldest block by access_count (FIFO fallback)
            if let Some(evict_id) = ids.first().cloned() {
                self.blocks.remove(&evict_id);
                ids.remove(0);
            }
        }

        let block = MemoryBlock::new(&id, label.clone(), content);
        self.blocks.insert(id.clone(), block);
        ids.push(id.clone());

        // ⚠️ 原为 `self.blocks.get(&id).unwrap()`：L178 刚 insert ⇒ 必 Some，
        //   unwrap **纯冗余**。⇒ 直接用 `remove`/返回值之外的路径都不合适
        //   （需保留），故用 `get` + 显式早退（语义：拿不到就是错误）。
        self.blocks
            .get(&id)
            .ok_or_else(|| format!("记忆块插入后仍取不到: {id}"))
    }

    /// Read a memory block by id
    pub fn read_block(&mut self, id: &str) -> Option<&mut MemoryBlock> {
        if let Some(block) = self.blocks.get_mut(id) {
            block.mark_accessed();
            Some(block)
        } else {
            None
        }
    }

    /// Write (update) a memory block's content (R-MEM09 core operation)
    pub fn write_block(&mut self, id: &str, new_content: impl Into<String>) -> EditResult {
        match self.blocks.get_mut(id) {
            Some(block) => {
                block.content = new_content.into();
                block.updated_at = unix_now();
                block.access_count += 1;
                EditResult::Applied
            }
            None => EditResult::BlockNotFound(id.to_string()),
        }
    }

    /// List all blocks, optionally filtered by label
    pub fn list_blocks(&self, label: Option<&MemoryLabel>) -> Vec<&MemoryBlock> {
        match label {
            Some(lbl) => self
                .label_index
                .get(lbl)
                .map(|ids| ids.iter().filter_map(|id| self.blocks.get(id)).collect())
                .unwrap_or_default(),
            None => self.blocks.values().collect(),
        }
    }

    /// Get block count
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Get block count by label
    pub fn label_count(&self, label: &MemoryLabel) -> usize {
        self.label_index.get(label).map_or(0, |ids| ids.len())
    }

    /// Remove a block by id
    pub fn remove_block(&mut self, id: &str) -> Option<MemoryBlock> {
        if let Some(block) = self.blocks.remove(id) {
            if let Some(ids) = self.label_index.get_mut(&block.label) {
                ids.retain(|i| i != id);
            }
            Some(block)
        } else {
            None
        }
    }

    /// Search blocks by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryBlock> {
        self.blocks
            .values()
            .filter(|b| b.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Search blocks by content substring
    pub fn search_by_content(&self, query: &str) -> Vec<&MemoryBlock> {
        let q = query.to_lowercase();
        self.blocks
            .values()
            .filter(|b| b.content.to_lowercase().contains(&q))
            .collect()
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_read_block() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("user_pref", MemoryLabel::Core, "User prefers dark mode")
            .unwrap();
        let block = fs.read_block("user_pref").unwrap();
        assert_eq!(block.content, "User prefers dark mode");
        assert_eq!(block.access_count, 1);
    }

    #[test]
    fn test_write_block() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "old").unwrap();
        let result = fs.write_block("b1", "new content");
        assert_eq!(result, EditResult::Applied);
        assert_eq!(fs.read_block("b1").unwrap().content, "new content");
    }

    #[test]
    fn test_write_nonexistent_returns_not_found() {
        let mut fs = MemoryFilesystem::new();
        let result = fs.write_block("ghost", "content");
        assert!(matches!(result, EditResult::BlockNotFound(_)));
    }

    #[test]
    fn test_create_duplicate_returns_err() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "first").unwrap();
        let result = fs.create_block("b1", MemoryLabel::Core, "second");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_blocks_by_label() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("c1", MemoryLabel::Core, "core 1").unwrap();
        fs.create_block("c2", MemoryLabel::Core, "core 2").unwrap();
        fs.create_block("a1", MemoryLabel::Archive, "archive 1").unwrap();

        let core = fs.list_blocks(Some(&MemoryLabel::Core));
        assert_eq!(core.len(), 2);
        let archive = fs.list_blocks(Some(&MemoryLabel::Archive));
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_remove_block() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "temp").unwrap();
        assert_eq!(fs.block_count(), 1);
        fs.remove_block("b1");
        assert_eq!(fs.block_count(), 0);
    }

    #[test]
    fn test_eviction_when_capacity_exceeded() {
        let mut fs = MemoryFilesystem::new();
        fs.label_capacities.insert(MemoryLabel::Core, 3);
        for i in 0..5 {
            fs.create_block(
                format!("b{}", i),
                MemoryLabel::Core,
                format!("content {}", i),
            )
            .unwrap();
        }
        // Should evict oldest, keeping capacity at 3
        assert_eq!(fs.label_count(&MemoryLabel::Core), 3);
    }

    #[test]
    fn test_search_by_content() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "User likes Rust").unwrap();
        fs.create_block("b2", MemoryLabel::Core, "User prefers dark mode").unwrap();
        let results = fs.search_by_content("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "b1");
    }
}
