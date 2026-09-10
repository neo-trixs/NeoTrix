//! AddressableStore — 基于 ARC (Addressable Recall Compaction) 模式的追加式存储
//!
//! 每次工具观测被追加存储并分配唯一 `§id`，活动视图仅保留引用指针（Citation），
//! 完整内容按需召回。内存高效：活动视图小，完整内容存储但不在上下文中。

use std::collections::HashMap;

/// 单条工具观测记录 — 追加式存储
#[derive(Debug, Clone)]
pub struct Observation {
    /// 唯一标识符，§id 格式，例如 "§001"
    pub id: String,
    /// 工具名称
    pub tool_name: String,
    /// 工具输入
    pub input: String,
    /// 工具输出
    pub output: String,
    /// 时间戳（Unix 秒）
    pub timestamp: i64,
    /// 近似 token 数量
    pub token_count: usize,
}

/// 引用指针 — 在活动视图中替代完整内容
#[derive(Debug, Clone)]
pub struct Citation {
    /// §id 引用
    pub id: String,
    /// 一行摘要，用于上下文
    pub summary: String,
    /// 通过引用节省的 token 数量
    pub token_count: usize,
}

/// 压缩结果
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// 保留完整内容的观测数量
    pub full_observations: usize,
    /// 替换为引用的观测数量
    pub cited_observations: usize,
    /// 节省的 token 总数
    pub tokens_saved: usize,
}

/// 追加式可寻址存储 — 用于工具观测的 ARC 模式实现
///
/// 工作原理：
/// 1. 所有工具观测追加写入，永不修改或删除
/// 2. 活动视图通过 Citation 引用 §id，而非内联完整内容
/// 3. 任意历史观测可通过 §id 按需召回
/// 4. 压缩（compact）将旧观测替换为引用，仅保留最近 N 条完整内容
#[derive(Debug)]
pub struct AddressableStore {
    observations: Vec<Observation>,
    index: HashMap<String, usize>,
    next_id: u32,
}

impl Default for AddressableStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AddressableStore {
    /// 创建空的 AddressableStore
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
            index: HashMap::new(),
            next_id: 1,
        }
    }

    /// 追加一条新观测，返回其 §id
    #[allow(dead_code)]
    pub fn append(&mut self, tool_name: &str, input: &str, output: &str) -> String {
        let id = format!("§{:03}", self.next_id);
        let token_count = estimate_tokens(tool_name) + estimate_tokens(input) + estimate_tokens(output);
        let observation = Observation {
            id: id.clone(),
            tool_name: tool_name.to_string(),
            input: input.to_string(),
            output: output.to_string(),
            timestamp: now_timestamp(),
            token_count,
        };
        let idx = self.observations.len();
        self.observations.push(observation);
        self.index.insert(id.clone(), idx);
        self.next_id += 1;
        id
    }

    /// 通过 §id 召回完整观测
    #[allow(dead_code)]
    pub fn recall(&self, id: &str) -> Option<&Observation> {
        self.index.get(id).and_then(|&idx| self.observations.get(idx))
    }

    /// 为指定 §id 列表创建引用（用于活动视图替代完整内容）
    #[allow(dead_code)]
    pub fn cite(&self, ids: &[String]) -> Vec<Citation> {
        ids.iter()
            .filter_map(|id| {
                self.recall(id).map(|obs| Citation {
                    id: obs.id.clone(),
                    summary: make_summary(&obs.tool_name, &obs.output),
                    token_count: obs.token_count,
                })
            })
            .collect()
    }

    /// 压缩：仅保留最近 N 条完整内容，其余替换为引用
    #[allow(dead_code)]
    pub fn compact(&self, keep_recent: usize) -> CompactionResult {
        let total = self.observations.len();
        if keep_recent >= total {
            return CompactionResult {
                full_observations: total,
                cited_observations: 0,
                tokens_saved: 0,
            };
        }

        let cited_count = total - keep_recent;
        let tokens_saved: usize = self.observations[..cited_count]
            .iter()
            .map(|obs| obs.token_count)
            .sum();

        CompactionResult {
            full_observations: keep_recent,
            cited_observations: cited_count,
            tokens_saved,
        }
    }

    /// 所有观测的总 token 数
    #[allow(dead_code)]
    pub fn total_tokens(&self) -> usize {
        self.observations.iter().map(|obs| obs.token_count).sum()
    }

    /// 已存储观测数量
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// 是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// 获取指定范围内的观测引用
    #[allow(dead_code)]
    pub fn range(&self, start: usize, end: usize) -> Vec<&Observation> {
        let end = end.min(self.observations.len());
        self.observations.get(start..end)
            .map(|slice| slice.iter().collect())
            .unwrap_or_default()
    }

    /// 获取最新一条观测
    #[allow(dead_code)]
    pub fn latest(&self) -> Option<&Observation> {
        self.observations.last()
    }
}

/// 粗略估算 token 数（按空格分词，1 token ≈ 4 字符 或 1 词）
fn estimate_tokens(text: &str) -> usize {
    let chars = text.len();
    let words = text.split_whitespace().count();
    // 取字符估算和词估算的较大值
    std::cmp::max(chars / 4, words)
}

/// 生成一行摘要
fn make_summary(tool_name: &str, output: &str) -> String {
    let preview: String = output.chars().take(60).collect();
    if output.len() > 60 {
        format!("[{tool_name}] {preview}...")
    } else {
        format!("[{tool_name}] {preview}")
    }
}

/// 获取当前 Unix 时间戳（秒）
fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_recall() {
        let mut store = AddressableStore::new();
        let id = store.append("web_search", "query", "result data");
        assert_eq!(id, "§001");

        let obs = store.recall(&id).unwrap();
        assert_eq!(obs.tool_name, "web_search");
        assert_eq!(obs.input, "query");
        assert_eq!(obs.output, "result data");
    }

    #[test]
    fn test_cite() {
        let mut store = AddressableStore::new();
        let id1 = store.append("tool_a", "in1", "out1");
        let id2 = store.append("tool_b", "in2", "out2");

        let citations = store.cite(&[id1, id2]);
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].summary, "[tool_a] out1");
        assert_eq!(citations[1].id, "§002");
    }

    #[test]
    fn test_compact() {
        let mut store = AddressableStore::new();
        for i in 0..10 {
            store.append("tool", &format!("in{i}"), &format!("out{i}"));
        }

        let result = store.compact(3);
        assert_eq!(result.full_observations, 3);
        assert_eq!(result.cited_observations, 7);
        assert!(result.tokens_saved > 0);
    }

    #[test]
    fn test_total_tokens_and_len() {
        let mut store = AddressableStore::new();
        store.append("t1", "hello world", "output text here");
        store.append("t2", "foo bar baz", "another output");

        assert_eq!(store.len(), 2);
        assert!(store.total_tokens() > 0);
    }

    #[test]
    fn test_recall_missing() {
        let store = AddressableStore::new();
        assert!(store.recall("§999").is_none());
    }

    #[test]
    fn test_latest() {
        let mut store = AddressableStore::new();
        assert!(store.latest().is_none());
        store.append("t", "i", "o");
        assert_eq!(store.latest().unwrap().id, "§001");
    }

    #[test]
    fn test_is_empty() {
        let mut store = AddressableStore::new();
        assert!(store.is_empty());
        store.append("t", "i", "o");
        assert!(!store.is_empty());
    }
}
