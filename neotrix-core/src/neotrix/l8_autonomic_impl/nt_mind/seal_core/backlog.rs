//! SEAL 存稿/缓存管理器 — 对标网文"滚动存稿"制度
//!
//! 网文工业化生产三支柱之一：作者提前写 5-14 章存稿，发布时定时放送，
//! 新稿滚动补入，避免断更。SEAL 流水线同构：预计算/预生成的结果先入存稿
//! （backlog），主流水线按需取用，避免管线空转（"断更"）。
//!
//! 对标映射（novel-causal-chain-analysis.md 启发5）：
//!   网文存稿制度 → SEAL 存稿/缓存（Obsidian rune 缓存层）
//!   滚动 5-14 章   → 存稿容量上限 + 满则淘汰最旧
//!   定时发布       → pop() 消费（发布到主流水线）
//!   全勤规则       → 存稿不足时告警（避免断更）

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// 存稿条目（对标"一章存稿"）
#[derive(Debug, Clone)]
pub struct BacklogEntry {
    pub id: String,
    /// 存稿类型（如 distillation / embedding / insight）
    pub kind: &'static str,
    /// 预生成内容
    pub content: String,
    pub created_at: u64,
    /// 是否已消费（发布）
    pub consumed: bool,
}

impl BacklogEntry {
    pub fn new(kind: &'static str, content: impl Into<String>) -> Self {
        Self {
            id: format!("bl-{}", now_ts()),
            kind,
            content: content.into(),
            created_at: now_ts(),
            consumed: false,
        }
    }
}

/// 滚动存稿管理器（对标网文存稿箱）
#[derive(Debug, Clone)]
pub struct BacklogManager {
    /// 存稿队列（FIFO：先写先发）
    queue: VecDeque<BacklogEntry>,
    /// 存稿容量上限（对标"滚动 5-14 章"）
    capacity: usize,
    /// 累计写入/消费统计
    pub total_pushed: u64,
    pub total_consumed: u64,
}

impl Default for BacklogManager {
    fn default() -> Self {
        Self::new(14) // 对标网文滚动存稿 5-14 章
    }
}

impl BacklogManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            capacity: capacity.max(1),
            total_pushed: 0,
            total_consumed: 0,
        }
    }

    /// 写入存稿（滚动：满则淘汰最旧，对标"新稿补位旧稿"）
    pub fn push(&mut self, entry: BacklogEntry) -> Option<BacklogEntry> {
        self.total_pushed += 1;
        let evicted = if self.queue.len() >= self.capacity {
            self.queue.pop_front() // 淘汰最旧存稿
        } else {
            None
        };
        self.queue.push_back(entry);
        evicted
    }

    /// 消费（发布）最旧存稿 — 对标"定时放送存稿"
    pub fn pop(&mut self) -> Option<BacklogEntry> {
        let mut entry = self.queue.pop_front()?;
        entry.consumed = true;
        self.total_consumed += 1;
        Some(entry)
    }

    /// 查看最旧存稿（不消费）
    pub fn peek(&self) -> Option<&BacklogEntry> {
        self.queue.front()
    }

    /// 存稿数量
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// 存稿容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 存稿填充率（对标"存稿充足度"）
    pub fn fill_ratio(&self) -> f64 {
        self.queue.len() as f64 / self.capacity as f64
    }

    /// 断更风险：存稿不足（对标网文"存稿耗尽"）
    pub fn risk_of_break(&self) -> bool {
        self.queue.len() < (self.capacity / 2).max(1)
    }

    /// 按类型统计存稿
    pub fn count_by_kind(&self, kind: &'static str) -> usize {
        self.queue.iter().filter(|e| e.kind == kind).count()
    }

    /// 存稿状态报告
    pub fn report(&self) -> String {
        format!(
            "存稿 {}/{} (填充率 {:.0}%) | 累计写入 {} 消费 {} | 断更风险: {}",
            self.queue.len(),
            self.capacity,
            self.fill_ratio() * 100.0,
            self.total_pushed,
            self.total_consumed,
            if self.is_of_break() { "⚠️ 是" } else { "否" },
        )
    }

    fn is_of_break(&self) -> bool {
        self.is_empty() || self.is_of_break_impl()
    }

    fn is_of_break_impl(&self) -> bool {
        self.risk_of_break()
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_fifo() {
        let mut bl = BacklogManager::new(5);
        bl.push(BacklogEntry::new("distill", "稿1"));
        bl.push(BacklogEntry::new("distill", "稿2"));
        assert_eq!(bl.len(), 2);
        let first = bl.pop().unwrap();
        assert_eq!(first.content, "稿1");
        assert!(first.consumed);
        assert_eq!(bl.len(), 1);
    }

    #[test]
    fn test_rolling_eviction() {
        // 容量 3，写入 5 → 淘汰最旧 2 个（滚动存稿）
        let mut bl = BacklogManager::new(3);
        for i in 0..5 {
            bl.push(BacklogEntry::new("embed", format!("稿{}", i)));
        }
        assert_eq!(bl.len(), 3);
        let first = bl.peek().unwrap();
        assert_eq!(first.content, "稿2", "最旧的稿0/稿1 应被淘汰");
    }

    #[test]
    fn test_break_risk() {
        let mut bl = BacklogManager::new(14);
        assert!(bl.is_of_break(), "空存稿有断更风险");
        for i in 0..8 {
            bl.push(BacklogEntry::new("analysis", format!("稿{}", i)));
        }
        assert!(!bl.is_of_break(), "8/14 存稿充足");
        assert_eq!(bl.count_by_kind("analysis"), 8);
    }

    #[test]
    fn test_report_contains_stats() {
        let mut bl = BacklogManager::new(14);
        bl.push(BacklogEntry::new("distill", "稿1"));
        let report = bl.report();
        assert!(report.contains("存稿"));
        assert!(report.contains("1/14"));
    }
}