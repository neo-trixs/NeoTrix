//! EditHistoryTracker — 所有代码变更的持久化记录
//!
//! 存储到 `~/.neotrix/edit_history.json`
//! 每条记录: 文件路径, 时间戳, issue 类型, 前后 SHA256, diff 摘要, 成功/失败

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

/// 单条编辑记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditEntry {
    pub timestamp: i64,
    pub file: String,
    pub issue_type: String,
    pub old_hash: String,
    pub new_hash: String,
    pub success: bool,
    pub diff_summary: String,
    pub session_id: String,
}

/// 文件级别的编辑统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditStats {
    pub file: String,
    pub total_edits: usize,
    pub successful_edits: usize,
    pub failed_edits: usize,
    pub last_edit: i64,
    pub common_patterns: Vec<String>,
}

/// 代码变更追踪器
#[derive(Debug, Clone)]
pub struct EditHistoryTracker {
    path: PathBuf,
    entries: Vec<EditEntry>,
    session_id: String,
}

impl EditHistoryTracker {
    /// 创建追踪器, 加载已有历史
    pub fn new() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("edit_history.json");
        Self::load_from_path(path)
    }

    pub fn load_from_path(path: PathBuf) -> Self {
        let session_id = Self::generate_session_id();
        let entries = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { path, entries, session_id }
    }

    /// 记录一次代码变更
    pub fn record_change(
        &mut self,
        file: &str,
        issue_type: &str,
        old_content: &str,
        new_content: &str,
        success: bool,
    ) -> Result<(), String> {
        let old_hash = Self::sha256(old_content);
        let new_hash = Self::sha256(new_content);
        let diff_summary = Self::compute_diff_summary(old_content, new_content);

        let entry = EditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            file: file.to_string(),
            issue_type: issue_type.to_string(),
            old_hash,
            new_hash,
            success,
            diff_summary,
            session_id: self.session_id.clone(),
        };

        self.entries.push(entry);
        self.persist()
    }

    /// 获取某个文件的所有编辑记录
    pub fn get_history_for_file(&self, file: &str) -> Vec<&EditEntry> {
        self.entries.iter().filter(|e| e.file == file).collect()
    }

    /// 获取某一类型的所有编辑记录
    pub fn get_history_for_issue_type(&self, issue_type: &str) -> Vec<&EditEntry> {
        self.entries.iter().filter(|e| e.issue_type == issue_type).collect()
    }

    /// 获取最近的 N 条记录
    pub fn recent(&self, n: usize) -> Vec<&EditEntry> {
        let mut entries: Vec<&EditEntry> = self.entries.iter().collect();
        entries.sort_by_key(|a| Reverse(a.timestamp));
        entries.into_iter().take(n).collect()
    }

    /// 获取所有记录
    pub fn all_entries(&self) -> &[EditEntry] {
        &self.entries
    }

    /// 按文件统计编辑记录
    pub fn file_stats(&self) -> Vec<FileEditStats> {
        let mut map: HashMap<String, Vec<&EditEntry>> = HashMap::new();
        for entry in &self.entries {
            map.entry(entry.file.clone()).or_default().push(entry);
        }

        let mut stats: Vec<FileEditStats> = map
            .into_iter()
            .map(|(file, entries)| {
                let total = entries.len();
                let successful = entries.iter().filter(|e| e.success).count();
                let failed = total - successful;
                let last_edit = entries.iter().map(|e| e.timestamp).max().unwrap_or(0);

                // 提取常见模式 (diff_summary 中出现最频繁的短语)
                let mut phrase_counts: HashMap<String, usize> = HashMap::new();
                for entry in &entries {
                    for word in entry.diff_summary.split_whitespace() {
                        if word.len() > 3 {
                            *phrase_counts.entry(word.to_string()).or_default() += 1;
                        }
                    }
                }
                let mut phrases: Vec<(String, usize)> = phrase_counts.into_iter().collect();
                phrases.sort_by_key(|a| Reverse(a.1));
                let common_patterns = phrases.into_iter().take(5).map(|(p, _)| p).collect();

                FileEditStats {
                    file,
                    total_edits: total,
                    successful_edits: successful,
                    failed_edits: failed,
                    last_edit,
                    common_patterns,
                }
            })
            .collect();

        stats.sort_by_key(|a| Reverse(a.total_edits));
        stats
    }

    /// 清空历史 (用于测试)
    pub fn clear(&mut self) -> Result<(), String> {
        self.entries.clear();
        self.persist()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ─── 内部 ───

    fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(&self.path, &json).map_err(|e| format!("写入失败: {}", e))
    }

    fn sha256(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("s{:x}", nanos)
    }

    fn compute_diff_summary(old: &str, new: &str) -> String {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        let added = new_lines.len().saturating_sub(old_lines.len());
        let removed = old_lines.len().saturating_sub(new_lines.len());
        let changed = old_lines
            .iter()
            .zip(new_lines.iter())
            .filter(|(a, b)| a != b)
            .count();
        format!("+{} -{} ~{}", added, removed, changed)
    }
}

impl Default for EditHistoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_tracker(name: &str) -> (EditHistoryTracker, PathBuf) {
        let path = std::env::temp_dir().join(format!("neotrix_edit_{}_{}.json", name, std::process::id()));
        (EditHistoryTracker::load_from_path(path.clone()), path)
    }

    #[test]
    fn test_new_tracker_is_empty() {
        let (tracker, path) = tmp_tracker("empty");
        assert!(tracker.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_record_change() {
        let (mut tracker, path) = tmp_tracker("change");
        tracker.record_change("test.rs", "MissingTests", "old", "new", true).unwrap();
        assert_eq!(tracker.len(), 1);
        let entry = &tracker.all_entries()[0];
        assert_eq!(entry.file, "test.rs");
        assert!(entry.success);
        assert!(entry.timestamp > 0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_record_failure() {
        let (mut tracker, path) = tmp_tracker("fail");
        tracker.record_change("test.rs", "CompileWarning", "old", "new_but_failed", false).unwrap();
        let stats = tracker.file_stats();
        let f = stats.iter().find(|s| s.file == "test.rs").unwrap();
        assert_eq!(f.total_edits, 1);
        assert_eq!(f.failed_edits, 1);
        assert_eq!(f.successful_edits, 0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_history_for_file() {
        let (mut tracker, path) = tmp_tracker("hist_file");
        tracker.record_change("a.rs", "A", "1", "2", true).unwrap();
        tracker.record_change("b.rs", "B", "1", "2", true).unwrap();
        tracker.record_change("a.rs", "A", "2", "3", true).unwrap();
        assert_eq!(tracker.get_history_for_file("a.rs").len(), 2);
        assert_eq!(tracker.get_history_for_file("b.rs").len(), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_history_for_issue_type() {
        let (mut tracker, path) = tmp_tracker("hist_type");
        tracker.record_change("a.rs", "MissingTests", "1", "2", true).unwrap();
        tracker.record_change("b.rs", "CompileWarning", "1", "2", true).unwrap();
        assert_eq!(tracker.get_history_for_issue_type("MissingTests").len(), 1);
        assert_eq!(tracker.get_history_for_issue_type("CompileWarning").len(), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_file_stats_ordering() {
        let (mut tracker, path) = tmp_tracker("stats");
        tracker.record_change("hot.rs", "A", "1", "2", true).unwrap();
        tracker.record_change("hot.rs", "A", "2", "3", true).unwrap();
        tracker.record_change("cold.rs", "B", "1", "2", true).unwrap();
        let stats = tracker.file_stats();
        assert!(stats[0].total_edits >= stats[1].total_edits);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_record_after_clear() {
        let (mut tracker, path) = tmp_tracker("clear");
        tracker.record_change("test.rs", "A", "1", "2", true).unwrap();
        tracker.clear().unwrap();
        assert!(tracker.is_empty());
        tracker.record_change("test.rs", "A", "1", "2", true).unwrap();
        assert_eq!(tracker.len(), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_diff_summary_removed_lines() {
        let summary = EditHistoryTracker::compute_diff_summary("a\nb\nc", "a");
        assert_eq!(summary, "+0 -2 ~0");
    }
}
