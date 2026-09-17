//! 可见性指标
//!
//! 定义通用的可见性指标、报告等类型，支持所有需要SEO/营销分析的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::search_engine::SearchEngine;

/// 可见性评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityScore {
    /// 总体评分 (0-100)
    pub overall: u8,
    /// 搜索可见性
    pub search: u8,
    /// 社交可见性
    pub social: u8,
    /// 内容可见性
    pub content: u8,
    /// 技术可见性
    pub technical: u8,
}

impl VisibilityScore {
    /// 创建新评分
    pub fn new() -> Self {
        Self {
            overall: 0,
            search: 0,
            social: 0,
            content: 0,
            technical: 0,
        }
    }

    /// 计算总体评分
    pub fn calculate_overall(&mut self) {
        self.overall = ((self.search as f64 * 0.4
            + self.social as f64 * 0.2
            + self.content as f64 * 0.25
            + self.technical as f64 * 0.15)
            .round() as u8)
            .min(100);
    }
}

impl Default for VisibilityScore {
    fn default() -> Self {
        Self::new()
    }
}

/// 可见性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityMetric {
    /// 指标名称
    pub name: String,
    /// 指标值
    pub value: f64,
    /// 单位
    pub unit: String,
    /// 变化趋势
    pub trend: Trend,
    /// 变化百分比
    pub change_percent: Option<f64>,
}

/// 趋势
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trend {
    Up,
    Down,
    Stable,
    Unknown,
}

impl std::fmt::Display for Trend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "↑"),
            Self::Down => write!(f, "↓"),
            Self::Stable => write!(f, "→"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

/// 可见性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityReport {
    /// 报告标题
    pub title: String,
    /// 生成时间
    pub generated_at: u64,
    /// 可见性评分
    pub score: VisibilityScore,
    /// 各项指标
    pub metrics: Vec<VisibilityMetric>,
    /// 关键词排名
    pub keyword_rankings: Vec<KeywordRanking>,
    /// 竞争对手分析
    pub competitor_analysis: Vec<CompetitorVisibility>,
    /// 建议
    pub recommendations: Vec<String>,
}

/// 关键词排名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRanking {
    /// 关键词
    pub keyword: String,
    /// 排名位置
    pub position: u32,
    /// 搜索引擎
    pub engine: SearchEngine,
    /// 变化
    pub change: i32,
    /// URL
    pub url: String,
}

/// 竞争对手可见性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorVisibility {
    /// 竞争对手名称
    pub name: String,
    /// 域名
    pub domain: String,
    /// 可见性评分
    pub score: u8,
    /// 主要关键词
    pub top_keywords: Vec<String>,
    /// 估计流量
    pub estimated_traffic: Option<u64>,
}

/// SEO审计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAuditResult {
    /// 页面URL
    pub url: String,
    /// 问题列表
    pub issues: Vec<SeoIssue>,
    /// 评分
    pub score: u8,
    /// 通过的检查
    pub passed_checks: Vec<String>,
}

/// SEO问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoIssue {
    /// 问题类型
    pub issue_type: SeoIssueType,
    /// 严重程度
    pub severity: IssueSeverity,
    /// 问题描述
    pub description: String,
    /// 建议修复
    pub recommendation: String,
}

/// SEO问题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeoIssueType {
    MissingTitle,
    DuplicateTitle,
    MissingMetaDescription,
    DuplicateMetaDescription,
    MissingH1,
    DuplicateH1,
    MissingAltText,
    BrokenLinks,
    SlowPageSpeed,
    MobileUnfriendly,
    MissingSchema,
    DuplicateContent,
    ThinContent,
    Other,
}

/// 问题严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_score() {
        let mut score = VisibilityScore::new();
        score.search = 80;
        score.social = 60;
        score.content = 70;
        score.technical = 90;
        score.calculate_overall();

        assert!(score.overall > 0);
        assert!(score.overall <= 100);
    }

    #[test]
    fn test_trend_display() {
        assert_eq!(Trend::Up.to_string(), "↑");
        assert_eq!(Trend::Down.to_string(), "↓");
        assert_eq!(Trend::Stable.to_string(), "→");
    }

    #[test]
    fn test_keyword_ranking() {
        let ranking = KeywordRanking {
            keyword: "example".into(),
            position: 5,
            engine: SearchEngine::Google,
            change: 2,
            url: "https://example.com".into(),
        };

        assert_eq!(ranking.position, 5);
        assert_eq!(ranking.change, 2);
    }
}
