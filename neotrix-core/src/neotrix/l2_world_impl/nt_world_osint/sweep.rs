//! OSINT 并行源扫描 + 增量检测 (吸收自 Crucix)
//!
//! 确定性纯模拟 — 无网络、无 tokio。每个源按固定规则产出条目,
//! 缺 key 的源降级 (Degraded::no_key); DeltaDetector 识别新条目并按关键词分级。

use crate::core::nt_core_self_test::SelfTest;

/// 一个扫描源
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SweepSource {
    pub name: &'static str,
    pub priority: u8,
    pub needs_key: bool,
}

impl SweepSource {
    pub const fn new(name: &'static str, priority: u8, needs_key: bool) -> Self {
        Self { name, priority, needs_key }
    }
}

/// 默认扫描源列表 (8-10 个)
pub fn default_sources() -> Vec<SweepSource> {
    vec![
        SweepSource::new("twitter", 9, true),
        SweepSource::new("github", 8, true),
        SweepSource::new("news", 7, false),
        SweepSource::new("deepweb", 6, true),
        SweepSource::new("dns", 5, false),
        SweepSource::new("shodan", 5, true),
        SweepSource::new("crtsh", 4, false),
        SweepSource::new("urlscan", 4, true),
        SweepSource::new("rss", 3, false),
        SweepSource::new("pastebin", 2, true),
    ]
}

/// 单源扫描结果
#[derive(Debug, Clone, PartialEq)]
pub enum SweepResult {
    Entry { source: &'static str, data: String },
    Degraded { source: &'static str, reason: &'static str },
}

/// 扫描汇总报告
#[derive(Debug, Clone, PartialEq)]
pub struct SweepReport {
    pub scanned: usize,
    pub degraded: usize,
    pub total_entries: usize,
}

impl SweepReport {
    pub fn summarize(sources: &[SweepSource], results: &[SweepResult]) -> Self {
        let scanned = sources.len();
        let degraded = results
            .iter()
            .filter(|r| matches!(r, SweepResult::Degraded { .. }))
            .count();
        let total_entries = results
            .iter()
            .filter(|r| matches!(r, SweepResult::Entry { .. }))
            .count();
        Self { scanned, degraded, total_entries }
    }
}

/// 并行源扫描器 (确定性模拟)
#[derive(Debug, Clone, Default)]
pub struct SweepRunner;

impl SweepRunner {
    pub fn new() -> Self {
        Self
    }

    /// 每个可用源产出 (priority % 3 + 1) 条确定性的条目;
    /// needs_key 但无对应 key → Degraded { reason: "no_key" }。
    pub fn run(&self, sources: &[SweepSource], api_keys: &[&str]) -> Vec<SweepResult> {
        sources
            .iter()
            .flat_map(|src| {
                let has_key = !src.needs_key || api_keys.iter().any(|k| *k == src.name);
                if !has_key {
                    return vec![SweepResult::Degraded { source: src.name, reason: "no_key" }];
                }
                let n = (src.priority as usize) % 3 + 1;
                (0..n)
                    .map(|i| SweepResult::Entry {
                        source: src.name,
                        data: format!("{}:item-{}", src.name, i),
                    })
                    .collect()
            })
            .collect()
    }
}

/// 条目严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Info,
}

/// 增量条目
#[derive(Debug, Clone, PartialEq)]
pub struct DeltaEntry {
    pub entry: String,
    pub severity: Severity,
}

/// 增量检测器
#[derive(Debug, Clone, Default)]
pub struct DeltaDetector;

impl DeltaDetector {
    pub fn new() -> Self {
        Self
    }

    /// 分级启发: vuln/exploit→Critical, breach/leak→High, update/release→Info, 默认 Medium
    pub fn grade(entry: &str) -> Severity {
        let e = entry.to_lowercase();
        if e.contains("vuln") || e.contains("exploit") {
            Severity::Critical
        } else if e.contains("breach") || e.contains("leak") {
            Severity::High
        } else if e.contains("update") || e.contains("release") {
            Severity::Info
        } else {
            Severity::Medium
        }
    }

    /// 新条目 (不在 previous 中) → DeltaEntry
    pub fn detect(&self, previous: &[&str], current: &[&str]) -> Vec<DeltaEntry> {
        current
            .iter()
            .filter(|entry| !previous.contains(entry))
            .map(|entry| DeltaEntry {
                entry: (*entry).to_string(),
                severity: Self::grade(entry),
            })
            .collect()
    }

    /// 按严重级别聚合告警计数
    pub fn alert(&self, entries: &[DeltaEntry]) -> AlertSummary {
        AlertSummary::from_entries(entries)
    }
}

/// 告警汇总
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlertSummary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub info: usize,
    pub total: usize,
}

impl AlertSummary {
    pub fn from_entries(entries: &[DeltaEntry]) -> Self {
        let mut s = Self::default();
        for e in entries {
            s.total += 1;
            match e.severity {
                Severity::Critical => s.critical += 1,
                Severity::High => s.high += 1,
                Severity::Medium => s.medium += 1,
                Severity::Info => s.info += 1,
            }
        }
        s
    }
}

/// SelfTest (T1): 扫描降级 + 增量 + 分级 + 聚合
pub struct SweepDeltaSelfTest;

impl SelfTest for SweepDeltaSelfTest {
    fn name(&self) -> &str {
        "nt_world_osint_sweep_delta"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let sources = default_sources();
        if sources.len() < 8 {
            return Err(vec![format!("expected >=8 sources, got {}", sources.len())]);
        }

        let runner = SweepRunner::new();
        let results = runner.run(&sources, &["twitter", "shodan"]);
        let report = SweepReport::summarize(&sources, &results);
        if report.degraded == 0 {
            return Err(vec!["no-key sources should degrade".into()]);
        }

        let detector = DeltaDetector::new();
        let previous = ["dns:item-0"];
        let deltas = detector.detect(&previous, &["dns:item-0", "vuln:CVE-2026-0001"]);
        if deltas.len() != 1 {
            return Err(vec![format!("expected 1 delta, got {}", deltas.len())]);
        }
        if deltas[0].severity != Severity::Critical {
            return Err(vec!["vuln should grade Critical".into()]);
        }

        let summary = detector.alert(&deltas);
        if summary.critical != 1 || summary.total != 1 {
            return Err(vec![format!("alert aggregation wrong: {summary:?}")]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sources_metadata() {
        let sources = default_sources();
        assert_eq!(sources.len(), 10);
        assert!(sources.iter().any(|s| s.name == "twitter" && s.priority == 9 && s.needs_key));
        assert!(sources.iter().any(|s| s.name == "dns" && !s.needs_key));
        let mut seen = std::collections::HashSet::new();
        for s in &sources {
            assert!(seen.insert(s.name), "duplicate source {}", s.name);
        }
    }

    #[test]
    fn test_sweep_all_keyed_ok() {
        let runner = SweepRunner::new();
        let keys: Vec<&str> = default_sources()
            .iter()
            .filter(|s| s.needs_key)
            .map(|s| s.name)
            .collect();
        let results = runner.run(&default_sources(), &keys);
        assert!(results.iter().all(|r| matches!(r, SweepResult::Entry { .. })));
        let report = SweepReport::summarize(&default_sources(), &results);
        assert_eq!(report.degraded, 0);
        assert_eq!(report.scanned, 10);
    }

    #[test]
    fn test_sweep_degraded_no_key() {
        let runner = SweepRunner::new();
        let results = runner.run(&default_sources(), &[]);
        let report = SweepReport::summarize(&default_sources(), &results);
        assert!(report.degraded > 0);
        let degraded: Vec<&str> = results
            .iter()
            .filter_map(|r| match r {
                SweepResult::Degraded { source, reason } if *reason == "no_key" => Some(*source),
                _ => None,
            })
            .collect();
        assert!(degraded.contains(&"twitter"));
        assert!(degraded.contains(&"pastebin"));
        assert!(!degraded.contains(&"dns"));
    }

    #[test]
    fn test_sweep_report_counts() {
        let runner = SweepRunner::new();
        let results = runner.run(&default_sources(), &[]);
        let report = SweepReport::summarize(&default_sources(), &results);
        assert_eq!(report.scanned, 10);
        assert_eq!(report.degraded, 6);
        assert_eq!(report.total_entries, 8);
        let keyed: usize = default_sources().iter().filter(|s| s.needs_key).count();
        assert_eq!(report.degraded, keyed);
    }

    #[test]
    fn test_delta_detects_new_entries() {
        let detector = DeltaDetector::new();
        let previous = ["news:item-0", "dns:item-0"];
        let current = ["news:item-0", "dns:item-0", "news:item-1", "github:item-0"];
        let deltas = detector.detect(&previous, &current);
        assert_eq!(deltas.len(), 2);
        assert_eq!(deltas[0].entry, "news:item-1");
        assert_eq!(deltas[1].entry, "github:item-0");
    }

    #[test]
    fn test_delta_unchanged_entries_excluded() {
        let detector = DeltaDetector::new();
        let previous = ["a", "b", "c"];
        let current = ["a", "b", "c"];
        assert!(detector.detect(&previous, &current).is_empty());
    }

    #[test]
    fn test_delta_severity_grading() {
        assert_eq!(DeltaDetector::grade("vuln:CVE-2026-1234"), Severity::Critical);
        assert_eq!(DeltaDetector::grade("breach at acme"), Severity::High);
        assert_eq!(DeltaDetector::grade("software update available"), Severity::Info);
        assert_eq!(DeltaDetector::grade("random notice"), Severity::Medium);
    }

    #[test]
    fn test_alert_aggregation() {
        let detector = DeltaDetector::new();
        let deltas = vec![
            DeltaEntry { entry: "vuln:x".into(), severity: Severity::Critical },
            DeltaEntry { entry: "breach:y".into(), severity: Severity::High },
            DeltaEntry { entry: "update:z".into(), severity: Severity::Info },
            DeltaEntry { entry: "notice".into(), severity: Severity::Medium },
            DeltaEntry { entry: "vuln:w".into(), severity: Severity::Critical },
        ];
        let summary = detector.alert(&deltas);
        assert_eq!(summary.critical, 2);
        assert_eq!(summary.high, 1);
        assert_eq!(summary.medium, 1);
        assert_eq!(summary.info, 1);
        assert_eq!(summary.total, 5);
    }

    #[test]
    fn test_alert_empty() {
        let summary = DeltaDetector::new().alert(&[]);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.critical + summary.high + summary.medium + summary.info, 0);
    }

    #[test]
    fn test_selftest_sweep_delta_passes() {
        let t = SweepDeltaSelfTest;
        assert_eq!(t.name(), "nt_world_osint_sweep_delta");
        assert!(t.self_test().is_ok());
    }
}
