use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// FindingsAggregator — 统一健康发现面板
///
/// Qodo Findings page + repowise 25 biomarkers 启发:
/// 将分散在 50+ 模块中的健康指标聚合为统一的结构化"发现"。
/// 每个发现包含类别、严重性、来源模块、趋势、修复建议。
/// 支持 30-day 聚合分析。

/// 发现类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindingCategory {
    CodeHealth,
    Performance,
    Security,
    Stability,
    Calibration,
    Memory,
    Evolution,
    Wiring,
    Knowledge,
    Perception,
}

impl FindingCategory {
    pub fn name(&self) -> &'static str {
        match self {
            FindingCategory::CodeHealth => "code_health",
            FindingCategory::Performance => "performance",
            FindingCategory::Security => "security",
            FindingCategory::Stability => "stability",
            FindingCategory::Calibration => "calibration",
            FindingCategory::Memory => "memory",
            FindingCategory::Evolution => "evolution",
            FindingCategory::Wiring => "wiring",
            FindingCategory::Knowledge => "knowledge",
            FindingCategory::Perception => "perception",
        }
    }
}

/// 发现
#[derive(Debug, Clone)]
pub struct Finding {
    pub id: u64,
    pub category: FindingCategory,
    pub severity: u8,
    pub title: String,
    pub description: String,
    pub source_module: String,
    pub value: f64,
    pub threshold: f64,
    pub trend: String,
    pub fix_suggestion: String,
    pub timestamp: u64,
}

impl Finding {
    pub fn is_critical(&self) -> bool { self.severity >= 8 }

    pub fn delta_from_threshold(&self) -> f64 { self.value - self.threshold }
}

/// 发现统计
#[derive(Debug, Clone, Default)]
pub struct FindingStats {
    pub total: usize,
    pub critical: usize,
    pub by_category: HashMap<FindingCategory, usize>,
    pub by_severity: HashMap<u8, usize>,
    pub recent_trend: String,
}

/// FindingsAggregator 配置
#[derive(Debug, Clone)]
pub struct FindingsAggregatorConfig {
    pub max_findings: usize,
    pub critical_threshold: u8,
    pub history_window: usize,
}

impl Default for FindingsAggregatorConfig {
    fn default() -> Self {
        Self { max_findings: 200, critical_threshold: 8, history_window: 30 }
    }
}

/// FindingsAggregator
#[derive(Debug, Clone)]
pub struct FindingsAggregator {
    config: FindingsAggregatorConfig,
    findings: Vec<Finding>,
    history: Vec<Vec<Finding>>,
    next_id: u64,
}

impl FindingsAggregator {
    pub fn new(config: FindingsAggregatorConfig) -> Self {
        Self { config, findings: Vec::new(), history: Vec::new(), next_id: 1 }
    }

    pub fn add_finding(
        &mut self,
        category: FindingCategory,
        severity: u8,
        title: String,
        description: String,
        source_module: String,
        value: f64,
        threshold: f64,
        trend: String,
        fix_suggestion: String,
    ) -> u64 {
        if self.findings.len() >= self.config.max_findings {
            self.findings.remove(0);
        }
        let id = self.next_id;
        self.next_id += 1;
        let finding = Finding {
            id,
            category,
            severity: severity.min(10),
            title,
            description,
            source_module,
            value,
            threshold,
            trend,
            fix_suggestion,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        };
        self.findings.push(finding);
        id
    }

    pub fn snapshot(&mut self) {
        self.history.push(self.findings.clone());
        if self.history.len() > self.config.history_window {
            self.history.remove(0);
        }
    }

    pub fn stats(&self) -> FindingStats {
        let mut stats = FindingStats::default();
        stats.total = self.findings.len();
        for f in &self.findings {
            if f.is_critical() { stats.critical += 1; }
            *stats.by_category.entry(f.category).or_insert(0) += 1;
            *stats.by_severity.entry(f.severity).or_insert(0) += 1;
        }
        if self.history.len() >= 2 {
            let recent = self.history.last().map(|h| h.len()).unwrap_or(0);
            let prev = self.history.iter().rev().nth(1).map(|h| h.len()).unwrap_or(0);
            stats.recent_trend = if recent > prev + 2 { "worsening".into() }
                else if recent < prev - 2 { "improving".into() }
                else { "stable".into() };
        } else {
            stats.recent_trend = "insufficient_data".into();
        }
        stats
    }

    pub fn critical_findings(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.is_critical()).collect()
    }

    pub fn by_category(&self, category: FindingCategory) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.category == category).collect()
    }

    pub fn by_source(&self, module: &str) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.source_module == module).collect()
    }

    pub fn finding_count(&self) -> usize { self.findings.len() }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "FindingsAggregator: {} findings ({} critical, {} categories), trend: {}",
            s.total, s.critical, s.by_category.len(), s.recent_trend
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(agg: &mut FindingsAggregator, cat: FindingCategory, sev: u8, val: f64) -> u64 {
        agg.add_finding(cat, sev, "test".into(), "desc".into(), "mod".into(), val, 0.5, "stable".into(), "fix".into())
    }

    #[test]
    fn test_empty_aggregator() {
        let agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        assert_eq!(agg.finding_count(), 0);
    }

    #[test]
    fn test_add_finding() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        let id = make_finding(&mut agg, FindingCategory::CodeHealth, 5, 0.7);
        assert_eq!(agg.finding_count(), 1);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_stats_tracking() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        make_finding(&mut agg, FindingCategory::CodeHealth, 9, 0.8);
        make_finding(&mut agg, FindingCategory::Stability, 3, 0.4);
        let stats = agg.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.critical, 1);
    }

    #[test]
    fn test_max_findings_eviction() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig { max_findings: 3, ..Default::default() });
        for i in 0..5 { make_finding(&mut agg, FindingCategory::CodeHealth, 1, i as f64); }
        assert_eq!(agg.finding_count(), 3);
    }

    #[test]
    fn test_critical_findings() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        make_finding(&mut agg, FindingCategory::CodeHealth, 9, 0.9);
        make_finding(&mut agg, FindingCategory::Stability, 3, 0.3);
        assert_eq!(agg.critical_findings().len(), 1);
    }

    #[test]
    fn test_by_category() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        make_finding(&mut agg, FindingCategory::CodeHealth, 5, 0.7);
        make_finding(&mut agg, FindingCategory::Memory, 4, 0.6);
        assert_eq!(agg.by_category(FindingCategory::CodeHealth).len(), 1);
        assert_eq!(agg.by_category(FindingCategory::Memory).len(), 1);
        assert_eq!(agg.by_category(FindingCategory::Perception).len(), 0);
    }

    #[test]
    fn test_by_source() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        agg.add_finding(FindingCategory::CodeHealth, 5, "t".into(), "d".into(), "module_a".into(), 0.7, 0.5, "stable".into(), "fix".into());
        agg.add_finding(FindingCategory::Memory, 4, "t".into(), "d".into(), "module_b".into(), 0.6, 0.5, "stable".into(), "fix".into());
        assert_eq!(agg.by_source("module_a").len(), 1);
    }

    #[test]
    fn test_severity_clamped() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        agg.add_finding(FindingCategory::CodeHealth, 15, "t".into(), "d".into(), "mod".into(), 0.9, 0.5, "stable".into(), "fix".into());
        let f = &agg.findings[0];
        assert!(f.severity <= 10);
    }

    #[test]
    fn test_snapshot_trend() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        agg.snapshot();
        make_finding(&mut agg, FindingCategory::CodeHealth, 5, 0.7);
        agg.snapshot();
        let stats = agg.stats();
        assert_eq!(stats.recent_trend, "improving");
    }

    #[test]
    fn test_summary_format() {
        let mut agg = FindingsAggregator::new(FindingsAggregatorConfig::default());
        make_finding(&mut agg, FindingCategory::CodeHealth, 9, 0.8);
        let s = agg.summary();
        assert!(s.contains("FindingsAggregator"));
    }
}
