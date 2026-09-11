//! Context Management Enhanced — 上下文管理增强
//!
//! 吸收 Codex PR (上下文管理/多文件编辑):
//! - 上下文窗口管理
//! - 优先级队列
//! - 上下文压缩
//! - 会话状态
//! - 多文件协调

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 上下文管理器
#[allow(dead_code)]
pub struct ContextManager {
    windows: HashMap<String, ContextWindow>,
    priority_queue: Vec<ContextItem>,
    compression_engine: CompressionEngine,
    config: ContextConfig,
    stats: ContextStats,
    /// Paged KV virtualization state (KVMem CSA2).
    paged_kv: PagedKvState,
}

/// Context storage strategy — dual-mode switching (KVMem CSA2).
///
/// For <256K tokens: use compaction (summarize/drop low-priority items).
/// For >256K tokens: use paged KV virtualization (GPU→Host→NVMe tiered).
/// The threshold is configurable (default 256K tokens).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextStrategy {
    /// Compaction mode: summarize + drop low-priority items when window full.
    /// Fast, low memory, suitable for short sessions (<256K tokens).
    Compaction,
    /// Paged KV mode: GPU→Host→NVMe tiered storage with page-level access.
    /// Supports arbitrarily long sessions, constant GPU memory (~35 GiB).
    PagedKv,
}

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::Compaction
    }
}

/// Paged KV virtualization state (KVMem arXiv:2609.04852).
///
/// Manages GPU→Host→NVMe tiered KV storage with page-level granularity.
/// GPU memory stays constant regardless of workspace size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedKvState {
    /// Current strategy (auto-switched based on token count).
    pub strategy: ContextStrategy,
    /// Token threshold for switching from compaction to paged KV.
    pub switch_threshold: usize,
    /// GPU page count (constant, ~35 GiB / page_size).
    pub gpu_pages: usize,
    /// Host page count (spillover from GPU).
    pub host_pages: usize,
    /// NVMe page count (cold storage for old context).
    pub nvme_pages: usize,
    /// Page size in tokens (default 32, matches KVMem block-level granularity).
    pub page_size: usize,
    /// Current total tokens across all tiers.
    pub total_tokens: usize,
    /// GPU utilization (0.0 - 1.0).
    pub gpu_utilization: f64,
    /// Working set: Retained/Incoming/Outgoing decomposition.
    pub working_set: WorkingSet,
}

impl Default for PagedKvState {
    fn default() -> Self {
        Self {
            strategy: ContextStrategy::Compaction,
            switch_threshold: 256_000,
            gpu_pages: 1024, // ~32K tokens at 32 tokens/page
            host_pages: 0,
            nvme_pages: 0,
            page_size: 32,
            total_tokens: 0,
            gpu_utilization: 0.0,
            working_set: WorkingSet::default(),
        }
    }
}

/// Working set decomposition for page-level KV management (KVMem insight).
///
/// At each agent step, the working set is decomposed into:
/// - **Retained**: pages still needed (GPU-resident, reused directly)
/// - **Incoming**: new pages entering GPU
/// - **Outgoing**: pages evicted from GPU to host/NVMe
///
/// Inter-step KL divergence is ~37× higher than intra-step (KVMem finding),
/// so the working set updates once per step, not per token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSet {
    /// Retained page indices (GPU-resident, no transfer needed).
    pub retained: Vec<usize>,
    /// Incoming page indices (must be transferred to GPU).
    pub incoming: Vec<usize>,
    /// Outgoing page indices (will be evicted from GPU).
    pub outgoing: Vec<usize>,
}

impl Default for WorkingSet {
    fn default() -> Self {
        Self {
            retained: Vec::new(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

/// Memory tier for paged KV storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryTier {
    Gpu,
    Host,
    Nvme,
}

/// 上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_window_size: usize,
    pub compression_ratio: f64,
    pub priority_levels: u32,
    pub enable_auto_compression: bool,
    pub session_timeout: u64,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_window_size: 8000,
            compression_ratio: 0.5,
            priority_levels: 5,
            enable_auto_compression: true,
            session_timeout: 3600,
        }
    }
}

/// 上下文窗口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub id: String,
    pub session_id: String,
    pub items: Vec<ContextItem>,
    pub current_size: usize,
    pub max_size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// 上下文项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: String,
    pub item_type: ItemType,
    pub content: String,
    pub priority: u32,
    pub token_count: usize,
    pub metadata: HashMap<String, serde_json::Value>,
    pub pinned: bool,
}

/// 项类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    UserMessage,
    AssistantMessage,
    SystemMessage,
    ToolCall,
    ToolResult,
    CodeSnippet,
    FileContent,
    Error,
}

/// 压缩引擎
#[allow(dead_code)]
pub struct CompressionEngine {
    compression_map: HashMap<String, String>,
}

/// 上下文统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_windows: u64,
    pub total_items: u64,
    pub items_compressed: u64,
    pub avg_window_size: f64,
    pub compression_ratio: f64,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub user_id: Option<String>,
    pub current_window: String,
    pub history: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// 多文件协调结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFileCoordination {
    pub files: Vec<FileContext>,
    pub dependencies: Vec<FileDependency>,
    pub edit_plan: Vec<EditOperation>,
    pub conflicts: Vec<EditConflict>,
}

/// 文件上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: String,
    pub content: String,
    pub language: String,
    pub size: usize,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// 文件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDependency {
    pub source: String,
    pub target: String,
    pub dependency_type: String,
}

/// 编辑操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub file_path: String,
    pub operation_type: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
}

/// 编辑冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditConflict {
    pub file_path: String,
    pub conflict_type: String,
    pub range: (u32, u32),
    pub resolution: Option<String>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(config: ContextConfig) -> Self {
        Self {
            windows: HashMap::new(),
            priority_queue: Vec::new(),
            compression_engine: CompressionEngine {
                compression_map: HashMap::new(),
            },
            config,
            stats: ContextStats {
                total_windows: 0,
                total_items: 0,
                items_compressed: 0,
                avg_window_size: 0.0,
                compression_ratio: 0.0,
            },
            paged_kv: PagedKvState::default(),
        }
    }

    /// Create a context manager with a specific paged KV configuration.
    pub fn with_paged_kv(mut self, paged_kv: PagedKvState) -> Self {
        self.paged_kv = paged_kv;
        self
    }

    /// Get the current context storage strategy based on token count.
    pub fn current_strategy(&self) -> &ContextStrategy {
        &self.paged_kv.strategy
    }

    /// Check if the context should switch strategies based on total tokens.
    ///
    /// Returns `Some(strategy)` if a switch is recommended, `None` otherwise.
    pub fn should_switch_strategy(&self) -> Option<ContextStrategy> {
        match self.paged_kv.strategy {
            ContextStrategy::Compaction => {
                if self.paged_kv.total_tokens >= self.paged_kv.switch_threshold {
                    Some(ContextStrategy::PagedKv)
                } else {
                    None
                }
            }
            ContextStrategy::PagedKv => {
                // Only switch back if tokens drop significantly below threshold
                if self.paged_kv.total_tokens < self.paged_kv.switch_threshold / 2 {
                    Some(ContextStrategy::Compaction)
                } else {
                    None
                }
            }
        }
    }

    /// Update token count and auto-switch strategy if needed.
    pub fn update_token_count(&mut self, tokens: usize) {
        self.paged_kv.total_tokens = tokens;
        self.paged_kv.gpu_utilization =
            (tokens as f64 / (self.paged_kv.gpu_pages * self.paged_kv.page_size) as f64).min(1.0);

        if let Some(new_strategy) = self.should_switch_strategy() {
            self.paged_kv.strategy = new_strategy;
        }
    }

    /// Update the working set for paged KV mode (call once per agent step).
    ///
    /// Decomposes the current page set into retained/incoming/outgoing
    /// based on which pages were accessed in the current step.
    pub fn update_working_set(&mut self, accessed_pages: &[usize]) {
        let retained: Vec<usize> = accessed_pages
            .iter()
            .filter(|&&p| self.paged_kv.working_set.retained.contains(&p))
            .copied()
            .collect();

        let incoming: Vec<usize> = accessed_pages
            .iter()
            .filter(|&&p| !self.paged_kv.working_set.retained.contains(&p))
            .copied()
            .collect();

        let outgoing: Vec<usize> = self
            .paged_kv
            .working_set
            .retained
            .iter()
            .filter(|&&p| !accessed_pages.contains(&p))
            .copied()
            .collect();

        self.paged_kv.working_set = WorkingSet {
            retained,
            incoming,
            outgoing,
        };
    }

    /// Get the recommended memory tier for a page based on access recency.
    pub fn page_tier(&self, page_index: usize, last_access_step: usize, current_step: usize) -> MemoryTier {
        let age = current_step.saturating_sub(last_access_step);
        if self.paged_kv.working_set.retained.contains(&page_index) {
            MemoryTier::Gpu
        } else if age < 10 {
            MemoryTier::Host
        } else {
            MemoryTier::Nvme
        }
    }

    /// 创建新窗口
    pub fn create_window(&mut self, session_id: &str) -> String {
        let window_id = uuid::Uuid::new_v4().to_string();
        let window = ContextWindow {
            id: window_id.clone(),
            session_id: session_id.to_string(),
            items: Vec::new(),
            current_size: 0,
            max_size: self.config.max_window_size,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        };

        self.windows.insert(window_id.clone(), window);
        self.stats.total_windows += 1;
        window_id
    }

    /// 添加上下文项
    pub fn add_item(&mut self, window_id: &str, item: ContextItem) -> Result<(), String> {
        // Check if compression is needed (immutable borrow, scoped)
        let need_compress = {
            let window = self.windows.get(window_id)
                .ok_or_else(|| format!("Window {} not found", window_id))?;
            window.current_size + item.token_count > window.max_size
        };

        if need_compress {
            if self.config.enable_auto_compression {
                self.compress_window(window_id)?;
            } else {
                return Err("Window full".into());
            }
        }

        // Now push the item (no outstanding borrows on self.windows)
        self.priority_queue.push(item.clone());
        self.priority_queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        let window = self.windows.get_mut(window_id)
            .ok_or_else(|| format!("Window {} not found", window_id))?;
        let token_count = item.token_count;
        window.items.push(item);
        window.current_size += token_count;
        window.last_accessed = chrono::Utc::now();

        self.stats.total_items += 1;
        Ok(())
    }

    /// 压缩窗口
    fn compress_window(&mut self, window_id: &str) -> Result<(), String> {
        let window = self.windows.get_mut(window_id)
            .ok_or_else(|| format!("Window {} not found", window_id))?;

        // 移除低优先级的非固定项
        let target_size = (window.max_size as f64 * self.config.compression_ratio) as usize;
        let mut removed_count = 0;

        // 按优先级排序，保留高优先级和固定项
        let mut items: Vec<ContextItem> = window.items.drain(..).collect();
        items.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut current_size = 0;
        for item in items {
            if item.pinned || current_size + item.token_count <= target_size {
                window.items.push(item.clone());
                current_size += item.token_count;
            } else {
                removed_count += 1;
            }
        }

        window.current_size = current_size;
        self.stats.items_compressed += removed_count as u64;

        Ok(())
    }

    /// 获取上下文
    pub fn get_context(&self, window_id: &str) -> Option<&Vec<ContextItem>> {
        self.windows.get(window_id).map(|w| &w.items)
    }

    /// 搜索上下文
    pub fn search_context(&self, window_id: &str, query: &str) -> Vec<&ContextItem> {
        if let Some(window) = self.windows.get(window_id) {
            window.items.iter()
                .filter(|item| item.content.contains(query))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 协调多文件编辑
    pub fn coordinate_multi_file_edit(&self, files: Vec<FileContext>, edit_plan: Vec<EditOperation>) -> MultiFileCoordination {
        let dependencies = Vec::new();
        let mut conflicts = Vec::new();

        // 分析依赖关系
        for (i, op1) in edit_plan.iter().enumerate() {
            for (j, op2) in edit_plan.iter().enumerate() {
                if i != j && op1.file_path == op2.file_path {
                    // 检查范围冲突
                    if op1.start_line <= op2.end_line && op1.end_line >= op2.start_line {
                        conflicts.push(EditConflict {
                            file_path: op1.file_path.clone(),
                            conflict_type: "overlapping_range".into(),
                            range: (op1.start_line.max(op2.start_line), op1.end_line.min(op2.end_line)),
                            resolution: None,
                        });
                    }
                }
            }
        }

        MultiFileCoordination {
            files,
            dependencies,
            edit_plan,
            conflicts,
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ContextStats {
        &self.stats
    }
}

#[cfg(test)]
mod context_strategy_tests {
    use super::*;

    fn make_manager() -> ContextManager {
        ContextManager::new(ContextConfig::default())
    }

    #[test]
    fn test_default_strategy_is_compaction() {
        let mgr = make_manager();
        assert!(matches!(mgr.current_strategy(), ContextStrategy::Compaction));
    }

    #[test]
    fn test_switch_to_paged_kv_above_threshold() {
        let mut mgr = make_manager();
        // Default threshold is 256K tokens
        mgr.update_token_count(300_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::PagedKv));
    }

    #[test]
    fn test_stay_compaction_below_threshold() {
        let mut mgr = make_manager();
        mgr.update_token_count(100_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::Compaction));
    }

    #[test]
    fn test_switch_back_to_compaction_on_drop() {
        let mut mgr = make_manager();
        // Switch to paged KV
        mgr.update_token_count(300_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::PagedKv));

        // Drop below half threshold → switch back
        mgr.update_token_count(100_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::Compaction));
    }

    #[test]
    fn test_stay_paged_kv_above_half_threshold() {
        let mut mgr = make_manager();
        mgr.update_token_count(300_000);
        // Drop to 150K (above 128K = 256K/2) → stay paged KV
        mgr.update_token_count(150_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::PagedKv));
    }

    #[test]
    fn test_gpu_utilization_updates() {
        let mut mgr = make_manager();
        mgr.update_token_count(1000);
        assert!(mgr.paged_kv.gpu_utilization > 0.0);
    }

    #[test]
    fn test_working_set_update() {
        let mut mgr = make_manager();
        mgr.paged_kv.working_set.retained = vec![0, 1, 2];
        mgr.update_working_set(&[1, 2, 3]);
        // 1,2 retained; 3 incoming; 0 outgoing
        assert!(mgr.paged_kv.working_set.retained.contains(&1));
        assert!(mgr.paged_kv.working_set.retained.contains(&2));
        assert!(mgr.paged_kv.working_set.incoming.contains(&3));
        assert!(mgr.paged_kv.working_set.outgoing.contains(&0));
    }

    #[test]
    fn test_page_tier_gpu_for_retained() {
        let mut mgr = make_manager();
        mgr.paged_kv.working_set.retained = vec![5];
        let tier = mgr.page_tier(5, 0, 100);
        assert!(matches!(tier, MemoryTier::Gpu));
    }

    #[test]
    fn test_page_tier_host_for_recent() {
        let mgr = make_manager();
        let tier = mgr.page_tier(99, 95, 100); // age=5 < 10
        assert!(matches!(tier, MemoryTier::Host));
    }

    #[test]
    fn test_page_tier_nvme_for_old() {
        let mgr = make_manager();
        let tier = mgr.page_tier(99, 50, 100); // age=50 >= 10
        assert!(matches!(tier, MemoryTier::Nvme));
    }

    #[test]
    fn test_custom_threshold() {
        let mut mgr = make_manager();
        mgr.paged_kv.switch_threshold = 100_000;
        mgr.update_token_count(150_000);
        assert!(matches!(mgr.current_strategy(), ContextStrategy::PagedKv));
    }

    #[test]
    fn test_with_paged_kv_builder() {
        let custom_paged = PagedKvState {
            switch_threshold: 50_000,
            gpu_pages: 2048,
            page_size: 64,
            ..PagedKvState::default()
        };
        let mgr = make_manager().with_paged_kv(custom_paged);
        assert_eq!(mgr.paged_kv.switch_threshold, 50_000);
        assert_eq!(mgr.paged_kv.gpu_pages, 2048);
    }
}
