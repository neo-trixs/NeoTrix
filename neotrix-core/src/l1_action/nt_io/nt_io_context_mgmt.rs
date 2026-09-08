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
pub struct ContextManager {
    windows: HashMap<String, ContextWindow>,
    priority_queue: Vec<ContextItem>,
    compression_engine: CompressionEngine,
    config: ContextConfig,
    stats: ContextStats,
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
