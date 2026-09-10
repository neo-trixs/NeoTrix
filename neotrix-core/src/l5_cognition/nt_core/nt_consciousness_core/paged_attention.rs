#![forbid(unsafe_code)]

//! PagedAttention 块管理 (PagedAttention Block Management)
//!
//! 固定大小块分配、引用计数共享块、哈希前缀匹配去重、内存高效 KV 存储
//! 参考 vLLM PagedAttention 和 KVMem 分页 KV 虚拟化

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 块大小 (token 数)
pub const DEFAULT_BLOCK_SIZE: usize = 16;

/// 块 ID
pub type BlockId = u64;

/// 物理块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalBlock {
    /// 块 ID
    pub id: BlockId,
    /// 存储的 token 数
    pub token_count: usize,
    /// 块大小
    pub block_size: usize,
    /// 内容哈希 (用于前缀匹配)
    pub content_hash: u64,
    /// 引用计数
    pub ref_count: u32,
    /// 是否为空块
    pub is_empty: bool,
    /// 创建时间
    pub created_at: String,
    /// 最后访问时间
    pub last_accessed: String,
}

/// 逻辑块 (引用物理块)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalBlock {
    /// 逻辑块 ID
    pub id: u64,
    /// 物理块 ID
    pub physical_block_id: BlockId,
    /// 在物理块内的偏移
    pub offset: usize,
    /// 有效 token 数
    pub valid_tokens: usize,
}

/// 序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    /// 序列 ID
    pub id: String,
    /// 逻辑块列表
    pub blocks: Vec<LogicalBlock>,
    /// 总 token 数
    pub total_tokens: usize,
    /// 创建时间
    pub created_at: String,
}

/// 块表 (Block Table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTable {
    /// 序列 ID → 逻辑块列表
    pub tables: HashMap<String, Vec<LogicalBlock>>,
}

/// PagedAttention 管理器
pub struct PagedAttentionManager {
    /// 物理块池
    physical_blocks: HashMap<BlockId, PhysicalBlock>,
    /// 块表
    block_table: BlockTable,
    /// 哈希 → 物理块 ID 映射 (用于去重)
    hash_to_block: HashMap<u64, BlockId>,
    /// 下一个块 ID
    next_block_id: BlockId,
    /// 块大小
    block_size: usize,
    /// 总块数
    #[allow(dead_code)]
    total_blocks: usize,
    /// 统计
    stats: AttentionStats,
}

/// 注意力统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttentionStats {
    /// 总块分配数
    pub total_allocations: u64,
    /// 总块释放数
    pub total_deallocations: u64,
    /// 当前分配块数
    pub allocated_blocks: usize,
    /// 去重命中次数
    pub dedup_hits: u64,
    /// 总序列数
    pub total_sequences: usize,
    /// 平均每序列块数
    pub avg_blocks_per_sequence: f64,
}

impl PagedAttentionManager {
    /// 创建新的 PagedAttention 管理器
    pub fn new(block_size: usize, total_blocks: usize) -> Self {
        let mut manager = Self {
            physical_blocks: HashMap::new(),
            block_table: BlockTable {
                tables: HashMap::new(),
            },
            hash_to_block: HashMap::new(),
            next_block_id: 0,
            block_size,
            total_blocks,
            stats: AttentionStats::default(),
        };

        // 预分配物理块
        for _ in 0..total_blocks {
            manager.alloc_physical_block();
        }

        manager
    }

    /// 分配物理块
    fn alloc_physical_block(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;

        let now = chrono::Utc::now().to_rfc3339();
        let block = PhysicalBlock {
            id,
            token_count: 0,
            block_size: self.block_size,
            content_hash: 0,
            ref_count: 0,
            is_empty: true,
            created_at: now.clone(),
            last_accessed: now,
        };

        self.physical_blocks.insert(id, block);
        self.stats.total_allocations += 1;
        self.stats.allocated_blocks += 1;
        id
    }

    /// 分配块给序列
    pub fn allocate_block(&mut self, sequence_id: &str, content_hash: u64) -> LogicalBlock {
        // 检查是否有可去重的块
        if let Some(&existing_id) = self.hash_to_block.get(&content_hash) {
            if let Some(physical) = self.physical_blocks.get_mut(&existing_id) {
                physical.ref_count += 1;
                physical.last_accessed = chrono::Utc::now().to_rfc3339();
                self.stats.dedup_hits += 1;

                let logical = LogicalBlock {
                    id: self.block_table.tables.get(sequence_id)
                        .map(|blocks| blocks.len() as u64)
                        .unwrap_or(0),
                    physical_block_id: existing_id,
                    offset: 0,
                    valid_tokens: physical.token_count,
                };

                self.block_table.tables
                    .entry(sequence_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(logical.clone());

                return logical;
            }
        }

        // 分配新物理块
        let physical_id = self.alloc_physical_block();
        if let Some(physical) = self.physical_blocks.get_mut(&physical_id) {
            physical.content_hash = content_hash;
            physical.ref_count = 1;
        }

        self.hash_to_block.insert(content_hash, physical_id);

        let logical = LogicalBlock {
            id: self.block_table.tables.get(sequence_id)
                .map(|blocks| blocks.len() as u64)
                .unwrap_or(0),
            physical_block_id: physical_id,
            offset: 0,
            valid_tokens: 0,
        };

        self.block_table.tables
            .entry(sequence_id.to_string())
            .or_insert_with(Vec::new)
            .push(logical.clone());

        self.update_stats();
        logical
    }

    /// 释放序列的所有块
    pub fn deallocate_sequence(&mut self, sequence_id: &str) {
        if let Some(blocks) = self.block_table.tables.remove(sequence_id) {
            for logical in blocks {
                if let Some(physical) = self.physical_blocks.get_mut(&logical.physical_block_id) {
                    physical.ref_count = physical.ref_count.saturating_sub(1);
                    self.stats.total_deallocations += 1;

                    // 如果引用计数为 0，可以回收
                    if physical.ref_count == 0 {
                        physical.is_empty = true;
                        physical.token_count = 0;
                        physical.content_hash = 0;
                    }
                }
            }
            self.update_stats();
        }
    }

    /// 哈希前缀匹配
    pub fn find_prefix_match(&self, hash_chain: &[u64]) -> Option<(BlockId, usize)> {
        let mut matched_blocks = 0;
        let mut last_block_id = None;

        for &hash in hash_chain {
            if let Some(&block_id) = self.hash_to_block.get(&hash) {
                if let Some(physical) = self.physical_blocks.get(&block_id) {
                    if physical.ref_count > 0 {
                        matched_blocks += 1;
                        last_block_id = Some(block_id);
                        continue;
                    }
                }
            }
            break;
        }

        if matched_blocks > 0 {
            last_block_id.map(|id| (id, matched_blocks))
        } else {
            None
        }
    }

    /// 更新统计
    fn update_stats(&mut self) {
        self.stats.total_sequences = self.block_table.tables.len();
        let total_blocks: usize = self.block_table.tables.values()
            .map(|blocks| blocks.len())
            .sum();
        self.stats.avg_blocks_per_sequence = if self.stats.total_sequences > 0 {
            total_blocks as f64 / self.stats.total_sequences as f64
        } else {
            0.0
        };
    }

    /// 获取块内容
    pub fn get_block_content(&self, block_id: BlockId) -> Option<&PhysicalBlock> {
        self.physical_blocks.get(&block_id)
    }

    /// 获取序列的所有逻辑块
    pub fn get_sequence_blocks(&self, sequence_id: &str) -> Option<&Vec<LogicalBlock>> {
        self.block_table.tables.get(sequence_id)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &AttentionStats {
        &self.stats
    }

    /// 获取物理块数
    pub fn physical_block_count(&self) -> usize {
        self.physical_blocks.len()
    }

    /// 获取可用物理块数
    pub fn available_blocks(&self) -> usize {
        self.physical_blocks.values()
            .filter(|b| b.is_empty || b.ref_count == 0)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 100);
        assert_eq!(manager.physical_block_count(), 100);
        assert_eq!(manager.available_blocks(), 100);
    }

    #[test]
    fn test_allocate_block() {
        let mut manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 10);
        let logical = manager.allocate_block("seq1", 12345);
        
        assert_eq!(logical.physical_block_id, 0);
        assert_eq!(manager.stats().total_sequences, 1);
    }

    #[test]
    fn test_deduplication() {
        let mut manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 10);
        
        // 两个序列使用相同哈希
        let block1 = manager.allocate_block("seq1", 99999);
        let block2 = manager.allocate_block("seq2", 99999);
        
        // 应该去重
        assert_eq!(block1.physical_block_id, block2.physical_block_id);
        assert_eq!(manager.stats().dedup_hits, 1);
    }

    #[test]
    fn test_deallocate_sequence() {
        let mut manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 10);
        manager.allocate_block("seq1", 11111);
        manager.allocate_block("seq1", 22222);
        
        manager.deallocate_sequence("seq1");
        assert_eq!(manager.stats().total_sequences, 0);
    }

    #[test]
    fn test_prefix_match() {
        let mut manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 10);
        manager.allocate_block("seq1", 100);
        manager.allocate_block("seq1", 200);
        manager.allocate_block("seq1", 300);
        
        let result = manager.find_prefix_match(&[100, 200, 300]);
        assert!(result.is_some());
        let (block_id, count) = result.unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_sequence_blocks() {
        let mut manager = PagedAttentionManager::new(DEFAULT_BLOCK_SIZE, 10);
        manager.allocate_block("seq1", 100);
        manager.allocate_block("seq1", 200);
        
        let blocks = manager.get_sequence_blocks("seq1");
        assert!(blocks.is_some());
        assert_eq!(blocks.unwrap().len(), 2);
    }
}
