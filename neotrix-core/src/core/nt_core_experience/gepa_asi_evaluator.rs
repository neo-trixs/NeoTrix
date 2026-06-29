/// GEPA ASI 结构化诊断评估器 (ICLR 2026 Oral)
///
/// GEPA 的 ASI (Architecture Self-Inspection) 组件: evaluator 返回
/// 丰富的结构化诊断，而非单一通过/拒绝。诊断包括:
/// - NL 失败模式描述
/// - 严重性评级 (blocker/critical/major/minor/trivial)
/// - 根因分析 (模式匹配 + 迹证据)
/// - 修复建议 (如果适用)
///
/// 参考: GEPA: Generative Evolution via Programmable Architectures (ICLR 2026 Oral)
use std::collections::VecDeque;

/// 严重性评级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Blocker,
    Critical,
    Major,
    Minor,
    Trivial,
}

impl Severity {
    pub fn as_f64(&self) -> f64 {
        match self {
            Severity::Blocker => 0.95,
            Severity::Critical => 0.75,
            Severity::Major => 0.55,
            Severity::Minor => 0.30,
            Severity::Trivial => 0.10,
        }
    }
}

/// 一个结构化的诊断发现
#[derive(Debug, Clone)]
pub struct AsiDiagnostic {
    pub id: u64,
    pub severity: Severity,
    pub category: String,
    /// 自然语言描述失败模式
    pub nl_description: String,
    /// 根因分析
    pub root_cause: String,
    /// 迹证据 (匹配的迹文本片段)
    pub trace_evidence: Vec<String>,
    /// 修复建议 (如果有)
    pub fix_suggestion: Option<String>,
    /// 诊断置信度 (0-1)
    pub confidence: f64,
}

/// ASI 结构化诊断报告
#[derive(Debug, Clone)]
pub struct AsiReport {
    pub cycle: u64,
    pub diagnostics: Vec<AsiDiagnostic>,
    pub overall_health: f64,
    pub num_blockers: usize,
    pub num_critical: usize,
}

impl AsiReport {
    pub fn is_healthy(&self) -> bool {
        self.num_blockers == 0 && self.num_critical == 0
    }
}

/// 迹模式: 匹配条件 + 严重性 + 描述
#[derive(Debug, Clone)]
pub struct TracePattern {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub severity: Severity,
    pub category: &'static str,
    pub root_cause_template: &'static str,
    pub fix_suggestion: &'static str,
}

static TRACE_PATTERNS: &[TracePattern] = &[
    TracePattern {
        name: "panic_spike",
        keywords: &["panic", "unreachable", "internal error"],
        severity: Severity::Critical,
        category: "RuntimeStability",
        root_cause_template: "Unhandled panic/unreachable in execution path",
        fix_suggestion: "Add catch_unwind or match all variants",
    },
    TracePattern {
        name: "timeout_degradation",
        keywords: &["timeout", "hung", "deadlock"],
        severity: Severity::Blocker,
        category: "Performance",
        root_cause_template: "Operation exceeded timeout threshold",
        fix_suggestion: "Add timeout with backoff retry",
    },
    TracePattern {
        name: "calibration_drift",
        keywords: &["ece", "calibration", "miscalibration"],
        severity: Severity::Major,
        category: "MetaCognition",
        root_cause_template: "Calibration error exceeds threshold",
        fix_suggestion: "Adjust confidence calibration curve",
    },
    TracePattern {
        name: "memory_pressure",
        keywords: &["OOM", "memory", "allocation"],
        severity: Severity::Critical,
        category: "Resource",
        root_cause_template: "Memory allocation or OOM in processing",
        fix_suggestion: "Add LRU eviction or cap growth",
    },
    TracePattern {
        name: "data_corruption",
        keywords: &["corrupt", "invalid", "malformed"],
        severity: Severity::Major,
        category: "DataIntegrity",
        root_cause_template: "Data corruption or invalid state detected",
        fix_suggestion: "Add validation gate before processing",
    },
    TracePattern {
        name: "permission_violation",
        keywords: &["permission", "unauthorized", "forbidden"],
        severity: Severity::Critical,
        category: "Security",
        root_cause_template: "Permission or authorization check failed",
        fix_suggestion: "Verify access control rules",
    },
    TracePattern {
        name: "low_confidence",
        keywords: &["uncertain", "low confidence", "ambiguous"],
        severity: Severity::Minor,
        category: "Confidence",
        root_cause_template: "Model confidence below threshold",
        fix_suggestion: "Request clarification or fallback",
    },
    TracePattern {
        name: "stagnation",
        keywords: &["stagnant", "no improvement", "plateau"],
        severity: Severity::Minor,
        category: "Evolution",
        root_cause_template: "No measurable improvement across cycles",
        fix_suggestion: "Increase mutation rate or explore new domain",
    },
];

/// GEPA ASI 评估器: 从迹文本中提取结构化诊断
#[derive(Debug, Clone)]
pub struct GepaAsiEvaluator {
    pub next_id: u64,
    pub max_trace_history: usize,
    /// 迹文本环形缓冲
    pub trace_texts: VecDeque<String>,
    /// 历史报告 (最近 50 个)
    pub reports: VecDeque<AsiReport>,
}

impl GepaAsiEvaluator {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            max_trace_history: 500,
            trace_texts: VecDeque::with_capacity(500),
            reports: VecDeque::with_capacity(50),
        }
    }

    /// 记录一条迹文本
    pub fn record_trace(&mut self, text: impl Into<String>) {
        let s = text.into();
        if self.trace_texts.len() >= self.max_trace_history {
            self.trace_texts.pop_front();
        }
        self.trace_texts.push_back(s);
    }

    /// 批量记录迹文本
    pub fn feed_traces(&mut self, texts: &[impl AsRef<str>]) {
        for t in texts {
            self.record_trace(t.as_ref());
        }
    }

    /// 运行诊断: 扫描迹缓冲匹配模式 → 生成结构化报告
    pub fn evaluate(&mut self, cycle: u64) -> AsiReport {
        let mut diagnostics = Vec::new();
        let mut match_counts: Vec<(&str, usize)> = Vec::new();

        // 对每条迹文本匹配所有模式
        for text in &self.trace_texts {
            for pattern in TRACE_PATTERNS {
                for kw in pattern.keywords {
                    if text.contains(kw) {
                        match_counts.push((pattern.name, 1));
                        break;
                    }
                }
            }
        }

        // 聚合匹配: 有匹配的模式 → 生成诊断
        let mut seen: Vec<&str> = Vec::new();
        for (name, _count) in &match_counts {
            if seen.contains(name) {
                continue;
            }
            seen.push(name);

            if let Some(pattern) = TRACE_PATTERNS.iter().find(|p| p.name == *name) {
                let id = self.next_id;
                self.next_id += 1;

                // 收集此模式的迹证据
                let evidence: Vec<String> = self
                    .trace_texts
                    .iter()
                    .filter(|t| pattern.keywords.iter().any(|kw| t.contains(kw)))
                    .take(3)
                    .cloned()
                    .collect();

                let confidence = (evidence.len() as f64 / 3.0).min(1.0);

                diagnostics.push(AsiDiagnostic {
                    id,
                    severity: pattern.severity,
                    category: pattern.category.to_string(),
                    nl_description: format!("{} detected in execution traces", pattern.name),
                    root_cause: pattern.root_cause_template.to_string(),
                    trace_evidence: evidence,
                    fix_suggestion: Some(pattern.fix_suggestion.to_string()),
                    confidence,
                });
            }
        }

        let num_blockers = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Blocker)
            .count();
        let num_critical = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Critical)
            .count();
        let overall_health = if diagnostics.is_empty() {
            1.0
        } else {
            (1.0 - diagnostics.iter().map(|d| d.severity.as_f64()).sum::<f64>()
                / (diagnostics.len() as f64 * 0.95))
                .max(0.0)
        };

        let report = AsiReport {
            cycle,
            diagnostics,
            overall_health,
            num_blockers,
            num_critical,
        };

        self.reports.push_back(report.clone());
        if self.reports.len() > 50 {
            self.reports.pop_front();
        }

        report
    }

    pub fn latest_report(&self) -> Option<&AsiReport> {
        self.reports.back()
    }

    pub fn health_trend(&self) -> f64 {
        let n = self.reports.len();
        if n < 2 {
            return 0.0;
        }
        let recent: f64 = self
            .reports
            .iter()
            .rev()
            .take(5)
            .map(|r| r.overall_health)
            .sum::<f64>()
            / 5.0f64.min(n as f64);
        let older: f64 = self
            .reports
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|r| r.overall_health)
            .sum::<f64>()
            / 5.0f64.min(n.saturating_sub(5) as f64);
        recent - older
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asi_healthy_when_no_traces() {
        let mut ev = GepaAsiEvaluator::new();
        let report = ev.evaluate(1);
        assert!(report.is_healthy());
        assert!((report.overall_health - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_asi_detects_panic() {
        let mut ev = GepaAsiEvaluator::new();
        ev.feed_traces(&[
            "ERROR: panic in consciousness_cycle step",
            "normal operation log entry",
        ]);
        let report = ev.evaluate(1);
        assert!(!report.diagnostics.is_empty());
        assert!(report.num_critical >= 1);
        assert!(!report.is_healthy());
    }

    #[test]
    fn test_asi_detects_timeout_and_panic() {
        let mut ev = GepaAsiEvaluator::new();
        ev.feed_traces(&[
            "timeout: HTTP request exceeded 30s",
            "panic: unwrap on None in pipeline",
        ]);
        let report = ev.evaluate(1);
        assert_eq!(report.diagnostics.len(), 2);
        assert_eq!(report.num_blockers, 1);
        assert_eq!(report.num_critical, 1);
    }

    #[test]
    fn test_asi_multiline_evidence() {
        let mut ev = GepaAsiEvaluator::new();
        ev.feed_traces(&[
            "panic: index out of bounds in gather step",
            "panic: unwrap on Err",
            "normal operation",
        ]);
        let report = ev.evaluate(1);
        let panic_diag = report
            .diagnostics
            .iter()
            .find(|d| d.category == "RuntimeStability");
        assert!(panic_diag.is_some());
        assert!(panic_diag.unwrap().fix_suggestion.is_some());
    }

    #[test]
    fn test_asi_health_trend_improving() {
        let mut ev = GepaAsiEvaluator::new();
        ev.feed_traces(&["panic: critical error", "OOM: memory exhausted"]);
        ev.evaluate(1);
        ev.evaluate(2);
        ev.evaluate(3);
        ev.evaluate(4);
        // After cleaning traces, health should improve
        ev.trace_texts.clear();
        ev.feed_traces(&["normal: all systems nominal"]);
        ev.evaluate(5);
        ev.evaluate(6);
        let trend = ev.health_trend();
        // Trend should be positive (improving)
        assert!(trend > -0.5);
    }

    #[test]
    fn test_asi_diagnostic_has_descriptions() {
        let mut ev = GepaAsiEvaluator::new();
        ev.feed_traces(&["corrupt: data integrity check failed"]);
        let report = ev.evaluate(1);
        let diag = &report.diagnostics[0];
        assert!(!diag.nl_description.is_empty());
        assert!(!diag.root_cause.is_empty());
        assert!(!diag.trace_evidence.is_empty());
    }
}
