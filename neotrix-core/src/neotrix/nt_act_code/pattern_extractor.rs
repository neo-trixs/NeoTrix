//! PatternExtractor — 从编辑历史中提取代码修改模式
//!
//! 分析 EditHistoryTracker 中的历史记录, 发现:
//!   - 频繁修改同一文件/同一问题的模式
//!   - 可复用的代码变换 (如"加测试 stub")
//!   - 失败模式 (哪些修改容易出错)

use std::cmp::Reverse;
use super::edit_history::{EditEntry, EditHistoryTracker};

/// 提取出的可复用模式
#[derive(Debug, Clone)]
pub struct ExtractedPattern {
    pub name: String,
    pub file_pattern: String,
    pub issue_types: Vec<String>,
    pub frequency: usize,
    pub success_rate: f64,
    pub avg_diff_size: f64,
    pub template_source: Option<String>,
    pub template_target: Option<String>,
}

/// 相似编辑集群
#[derive(Debug, Clone)]
pub struct EditCluster {
    pub name: String,
    pub entries: Vec<usize>,  // indices into history
    pub common_issue_types: Vec<String>,
    pub files: Vec<String>,
}

/// 模式提取器
#[derive(Debug)]
pub struct PatternExtractor;

impl PatternExtractor {
    /// 从完整历史中提取所有模式
    pub fn extract_all(history: &EditHistoryTracker) -> Vec<ExtractedPattern> {
        let mut patterns = Vec::new();

        // 1. 按文件路径聚类
        let stats = history.file_stats();
        for stat in &stats {
            if stat.total_edits < 3 {
                continue;
            }
            patterns.push(ExtractedPattern {
                name: format!("频繁编辑: {}", stat.file),
                file_pattern: stat.file.clone(),
                issue_types: Self::collect_issue_types(history, &stat.file),
                frequency: stat.total_edits,
                success_rate: if stat.total_edits > 0 {
                    stat.successful_edits as f64 / stat.total_edits as f64
                } else {
                    0.0
                },
                avg_diff_size: Self::avg_diff_size(history, &stat.file),
                template_source: None,
                template_target: None,
            });
        }

        // 2. 按问题类型聚类
        let issue_types = Self::collect_all_issue_types(history);
        for it in &issue_types {
            let entries = history.get_history_for_issue_type(it);
            if entries.len() < 3 {
                continue;
            }
            let success = entries.iter().filter(|e| e.success).count();
            patterns.push(ExtractedPattern {
                name: format!("问题类型: {}", it),
                file_pattern: "*".into(),
                issue_types: vec![it.clone()],
                frequency: entries.len(),
                success_rate: success as f64 / entries.len() as f64,
                avg_diff_size: entries.iter().map(|e| e.diff_summary.len() as f64).sum::<f64>()
                    / entries.len() as f64,
                template_source: None,
                template_target: None,
            });
        }

        patterns.sort_by_key(|a| Reverse(a.frequency));
        patterns
    }

    /// 将相似编辑聚类
    pub fn cluster(history: &EditHistoryTracker) -> Vec<EditCluster> {
        let entries: Vec<&EditEntry> = history.all_entries().iter().collect();
        let mut clusters: Vec<EditCluster> = Vec::new();
        let mut assigned = vec![false; entries.len()];

        for i in 0..entries.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = EditCluster {
                name: format!("cluster_{}", clusters.len()),
                entries: vec![i],
                common_issue_types: vec![entries[i].issue_type.clone()],
                files: vec![entries[i].file.clone()],
            };
            assigned[i] = true;

            for j in i + 1..entries.len() {
                if assigned[j] { continue; }
                // 相同文件 + 相同问题类型 → 同一集群
                if entries[j].file == entries[i].file
                    && entries[j].issue_type == entries[i].issue_type
                {
                    cluster.entries.push(j);
                    assigned[j] = true;
                }
            }

            if cluster.entries.len() >= 2 {
                clusters.push(cluster);
            }
        }

        clusters.sort_by_key(|a| Reverse(a.entries.len()));
        clusters
    }

    /// 从编辑历史生成新模板 (基于成功变换)
    pub fn generate_template_from_history(
        history: &EditHistoryTracker,
    ) -> Vec<super::template_registry::CodeTemplate> {
        let mut templates = Vec::new();
        let clusters = Self::cluster(history);

        for cluster in &clusters {
            if cluster.entries.len() < 3 {
                continue;
            }

            let entries: Vec<&EditEntry> = cluster.entries.iter()
                .map(|&i| &history.all_entries()[i])
                .collect();

            // 只从成功编辑生成模板
            let successful: Vec<&&EditEntry> = entries.iter().filter(|e| e.success).collect();
            if successful.len() < 2 {
                continue;
            }

                let file_name = cluster.files.first()
                    .and_then(|f| f.rsplit('/').next())
                    .unwrap_or("unknown");
                let name = format!(
                    "auto_{}_{}",
                    cluster.common_issue_types.first().map(|s| s.to_lowercase()).unwrap_or_else(|| "unknown".to_string()),
                    file_name,
                );

            templates.push(super::template_registry::CodeTemplate {
                name,
                category: super::template_registry::TemplateCategory::FunctionExtraction,
                source_pattern: "".into(),
                target_template: "".into(),
                applicability: vec!["rs".into()],
                required_imports: vec![],
                confidence: successful.len() as f64 / cluster.entries.len() as f64,
            });
        }

        templates
    }

    // ─── 内部 ───

    fn collect_issue_types(history: &EditHistoryTracker, file: &str) -> Vec<String> {
        let mut types: Vec<String> = history.get_history_for_file(file)
            .iter()
            .map(|e| e.issue_type.clone())
            .collect();
        types.sort();
        types.dedup();
        types
    }

    fn collect_all_issue_types(history: &EditHistoryTracker) -> Vec<String> {
        let mut types: Vec<String> = history.all_entries()
            .iter()
            .map(|e| e.issue_type.clone())
            .collect();
        types.sort();
        types.dedup();
        types
    }

    fn avg_diff_size(history: &EditHistoryTracker, file: &str) -> f64 {
        let entries = history.get_history_for_file(file);
        if entries.is_empty() {
            return 0.0;
        }
        entries.iter().map(|e| e.diff_summary.len() as f64).sum::<f64>() / entries.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_code::edit_history::EditHistoryTracker;
    use std::path::PathBuf;

    fn setup_history() -> EditHistoryTracker {
        let path = std::env::temp_dir().join(format!("neotrix_pattern_test_{}.json", std::process::id()));
        let mut h = EditHistoryTracker::load_from_path(path);
        h.record_change("src/a.rs", "MissingTests", "old", "new", true).unwrap();
        h.record_change("src/a.rs", "MissingTests", "old", "new2", true).unwrap();
        h.record_change("src/a.rs", "MissingTests", "old", "new3", true).unwrap();
        h.record_change("src/b.rs", "CompileWarning", "old", "new", false).unwrap();
        h.record_change("src/b.rs", "CompileWarning", "old", "new2", true).unwrap();
        h.record_change("src/b.rs", "CompileWarning", "old", "new3", true).unwrap();
        h
    }

    fn tmp_tracker() -> (EditHistoryTracker, PathBuf) {
        let path = std::env::temp_dir().join(format!("neotrix_pattern_extract_test_{}.json", std::process::id()));
        (EditHistoryTracker::load_from_path(path.clone()), path)
    }

    #[test]
    fn test_extract_all_returns_patterns() {
        let h = setup_history();
        let patterns = PatternExtractor::extract_all(&h);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_extract_all_order_by_frequency() {
        let h = setup_history();
        let patterns = PatternExtractor::extract_all(&h);
        for w in patterns.windows(2) {
            assert!(w[0].frequency >= w[1].frequency);
        }
    }

    #[test]
    fn test_cluster_groups() {
        let h = setup_history();
        let clusters = PatternExtractor::cluster(&h);
        for c in &clusters {
            assert!(c.entries.len() >= 2);
        }
    }

    #[test]
    fn test_cluster_same_file_same_type() {
        let (mut h, _path) = tmp_tracker();
        h.record_change("src/x.rs", "A", "1", "2", true).unwrap();
        h.record_change("src/x.rs", "A", "2", "3", true).unwrap();
        h.record_change("src/y.rs", "B", "1", "2", true).unwrap();
        let clusters = PatternExtractor::cluster(&h);
        assert!(clusters.len() >= 1);
        assert_eq!(clusters[0].files[0], "src/x.rs");
    }

    #[test]
    fn test_generate_template_from_history() {
        let h = setup_history();
        let templates = PatternExtractor::generate_template_from_history(&h);
        // at least the MissingTests on a.rs and CompileWarning on b.rs clusters
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_collect_issue_types() {
        let h = setup_history();
        let types = PatternExtractor::collect_all_issue_types(&h);
        assert!(types.contains(&"MissingTests".to_string()));
        assert!(types.contains(&"CompileWarning".to_string()));
    }
}
