//! Git Knowledge Loop — Git 学习回路
//!
//! 吸收 Git Knowledge Loop:
//! - 从提交历史学习
//! - 变更模式分析
//! - 代码演化追踪
//! - 知识沉淀
//! - 自动文档生成

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Git 学习引擎
pub struct GitLearningEngine {
    commits: Vec<CommitInfo>,
    patterns: Vec<CommitPattern>,
    knowledge: Vec<GitKnowledge>,
    config: GitLearningConfig,
    stats: GitLearningStats,
}

/// Git 学习配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLearningConfig {
    pub max_commits: usize,
    pub pattern_detection: bool,
    pub knowledge_extraction: bool,
    pub auto_documentation: bool,
}

impl Default for GitLearningConfig {
    fn default() -> Self {
        Self {
            max_commits: 1000,
            pattern_detection: true,
            knowledge_extraction: true,
            auto_documentation: true,
        }
    }
}

/// 提交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub files_changed: Vec<FileChange>,
    pub insertions: u32,
    pub deletions: u32,
    pub tags: Vec<String>,
}

/// 文件变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub insertions: u32,
    pub deletions: u32,
    pub complexity_delta: Option<f64>,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// 提交模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPattern {
    pub pattern_type: String,
    pub description: String,
    pub frequency: u32,
    pub examples: Vec<String>,
    pub confidence: f64,
}

/// Git 知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitKnowledge {
    pub id: String,
    pub knowledge_type: String,
    pub content: serde_json::Value,
    pub source_commits: Vec<String>,
    pub confidence: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Git 学习统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLearningStats {
    pub commits_analyzed: u64,
    pub patterns_detected: u64,
    pub knowledge_extracted: u64,
    pub avg_analysis_time: f64,
}

/// 演化报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionReport {
    pub period: String,
    pub total_commits: usize,
    pub active_contributors: Vec<String>,
    pub hotspots: Vec<Hotspot>,
    pub trends: Vec<Trend>,
    pub recommendations: Vec<String>,
}

/// 热点文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub file_path: String,
    pub change_frequency: u32,
    pub complexity_trend: String,
    pub risk_level: String,
}

/// 趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub trend_type: String,
    pub description: String,
    pub direction: String,
    pub magnitude: f64,
}

impl GitLearningEngine {
    /// 创建新的 Git 学习引擎
    pub fn new(config: GitLearningConfig) -> Self {
        Self {
            commits: Vec::new(),
            patterns: Vec::new(),
            knowledge: Vec::new(),
            config,
            stats: GitLearningStats {
                commits_analyzed: 0,
                patterns_detected: 0,
                knowledge_extracted: 0,
                avg_analysis_time: 0.0,
            },
        }
    }

    /// 分析提交
    pub fn analyze_commit(&mut self, commit: CommitInfo) {
        self.commits.push(commit.clone());
        self.stats.commits_analyzed += 1;

        if self.config.pattern_detection {
            self.detect_patterns(&commit);
        }

        if self.config.knowledge_extraction {
            self.extract_knowledge(&commit);
        }
    }

    /// 检测模式
    fn detect_patterns(&mut self, commit: &CommitInfo) {
        // 检测常见模式
        let message_lower = commit.message.to_lowercase();

        // 模式: 修复 bug
        if message_lower.contains("fix") || message_lower.contains("bug") {
            self.add_pattern("bug_fix", "Bug fix commit", commit);
        }

        // 模式: 新功能
        if message_lower.contains("feat") || message_lower.contains("add") || message_lower.contains("implement") {
            self.add_pattern("feature", "Feature addition commit", commit);
        }

        // 模式: 重构
        if message_lower.contains("refactor") || message_lower.contains("clean") {
            self.add_pattern("refactor", "Refactoring commit", commit);
        }

        // 模式: 文档
        if message_lower.contains("doc") || message_lower.contains("readme") {
            self.add_pattern("documentation", "Documentation commit", commit);
        }
    }

    /// 添加模式
    fn add_pattern(&mut self, pattern_type: &str, description: &str, commit: &CommitInfo) {
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.pattern_type == pattern_type) {
            existing.frequency += 1;
            existing.examples.push(commit.hash.clone());
        } else {
            self.patterns.push(CommitPattern {
                pattern_type: pattern_type.to_string(),
                description: description.to_string(),
                frequency: 1,
                examples: vec![commit.hash.clone()],
                confidence: 0.8,
            });
            self.stats.patterns_detected += 1;
        }
    }

    /// 提取知识
    fn extract_knowledge(&mut self, commit: &CommitInfo) {
        // 提取文件变更知识
        for change in &commit.files_changed {
            let knowledge = GitKnowledge {
                id: uuid::Uuid::new_v4().to_string(),
                knowledge_type: "file_change".into(),
                content: serde_json::json!({
                    "file": change.path,
                    "change_type": change.change_type,
                    "insertions": change.insertions,
                    "deletions": change.deletions,
                }),
                source_commits: vec![commit.hash.clone()],
                confidence: 0.9,
                created_at: chrono::Utc::now(),
            };
            self.knowledge.push(knowledge);
        }

        // 提取作者知识
        let author_knowledge = GitKnowledge {
            id: uuid::Uuid::new_v4().to_string(),
            knowledge_type: "author_activity".into(),
            content: serde_json::json!({
                "author": commit.author,
                "commit_count": 1,
                "insertions": commit.insertions,
                "deletions": commit.deletions,
            }),
            source_commits: vec![commit.hash.clone()],
            confidence: 0.95,
            created_at: chrono::Utc::now(),
        };
        self.knowledge.push(author_knowledge);
        self.stats.knowledge_extracted += 1;
    }

    /// 生成演化报告
    pub fn generate_report(&self) -> EvolutionReport {
        let contributors: Vec<String> = self.commits.iter()
            .map(|c| c.author.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // 找到热点文件
        let mut file_changes: HashMap<String, u32> = HashMap::new();
        for commit in &self.commits {
            for change in &commit.files_changed {
                *file_changes.entry(change.path.clone()).or_insert(0) += 1;
            }
        }

        let mut hotspots: Vec<Hotspot> = file_changes.iter()
            .map(|(path, &count)| Hotspot {
                file_path: path.clone(),
                change_frequency: count,
                complexity_trend: "stable".into(),
                risk_level: if count > 10 { "high".into() } else if count > 5 { "medium".into() } else { "low".into() },
            })
            .collect();
        hotspots.sort_by(|a, b| b.change_frequency.cmp(&a.change_frequency));

        // 生成趋势
        let trends = self.patterns.iter().map(|p| Trend {
            trend_type: p.pattern_type.clone(),
            description: p.description.clone(),
            direction: if p.frequency > 5 { "increasing".into() } else { "stable".into() },
            magnitude: p.frequency as f64,
        }).collect();

        // 生成建议
        let mut recommendations = Vec::new();
        if hotspots.iter().any(|h| h.risk_level == "high") {
            recommendations.push("Consider refactoring high-change files".into());
        }
        if contributors.len() < 3 {
            recommendations.push("Increase code review coverage".into());
        }

        EvolutionReport {
            period: "last_30_days".into(),
            total_commits: self.commits.len(),
            active_contributors: contributors,
            hotspots,
            trends,
            recommendations,
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &GitLearningStats {
        &self.stats
    }
}
