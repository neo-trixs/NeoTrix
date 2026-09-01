//! Reflection executor — 批判 → 修订闭环执行器 (P1 补齐)。
//!
//! 原有 IntraReflection 只有检测器 (`analyze` 产出 ReflectionReport)，
//! 无执行器: 报告生成后即被丢弃, 批判意见从未转化为修订动作, 闭环断裂。
//! 本模块补齐执行器: 基于报告分数生成结构化批判 (critique), 并产出
//! 修订后的响应 (revise), 形成 检测 → 批判 → 修订 的完整 Reflection 闭环,
//! 对齐主流推理模型的生成式 Reflection (如 Reflexion 的 verbal RL)。

use super::types::ReflectionReport;

/// 批判严重度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
}

/// 单条批判意见。
#[derive(Debug, Clone)]
pub struct Critique {
    pub dimension: &'static str,
    pub score: f64,
    pub severity: Severity,
    pub message: String,
}

/// 修订结果。
#[derive(Debug, Clone)]
pub struct Revision {
    pub critiques: Vec<Critique>,
    pub revised_response: String,
    pub revised: bool,
    pub overall_score: f64,
}

/// Reflection 闭环执行器。
#[derive(Debug, Clone)]
pub struct ReflectionExecutor {
    pub coherence_threshold: f64,
    pub efficiency_threshold: f64,
    pub error_density_threshold: f64,
    pub mode_stability_threshold: f64,
}

impl Default for ReflectionExecutor {
    fn default() -> Self {
        Self {
            coherence_threshold: 0.6,
            efficiency_threshold: 0.4,
            error_density_threshold: 0.25,
            mode_stability_threshold: 0.3,
        }
    }
}

impl ReflectionExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 批判阶段: 将报告中的低分维度转为结构化批判意见。
    pub fn critique(&self, report: &ReflectionReport) -> Vec<Critique> {
        let mut critiques = Vec::new();

        if report.coherence_score < self.coherence_threshold {
            critiques.push(Critique {
                dimension: "coherence",
                score: report.coherence_score,
                severity: if report.coherence_score < 0.3 { Severity::High } else { Severity::Medium },
                message: format!(
                    "low coherence ({:.2}) — reasoning steps lack logical continuity; \
                     restructure with explicit step linking and intermediate summaries.",
                    report.coherence_score
                ),
            });
        }

        if report.efficiency_score < self.efficiency_threshold && report.efficiency_score > 0.0 {
            critiques.push(Critique {
                dimension: "efficiency",
                score: report.efficiency_score,
                severity: Severity::Medium,
                message: format!(
                    "low efficiency ({:.2}) — trace longer than expected; prune redundant steps.",
                    report.efficiency_score
                ),
            });
        }

        if report.error_density > self.error_density_threshold {
            critiques.push(Critique {
                dimension: "error_density",
                score: report.error_density,
                severity: if report.error_density > 0.5 { Severity::High } else { Severity::Medium },
                message: format!(
                    "high error density ({:.2}) — switch to a more conservative E8 mode \
                     and add validation gates.",
                    report.error_density
                ),
            });
        }

        if report.mode_stability < self.mode_stability_threshold {
            critiques.push(Critique {
                dimension: "mode_stability",
                score: report.mode_stability,
                severity: Severity::Low,
                message: format!(
                    "excessive E8 mode switching ({:.2}) — frequent mode changes disrupt \
                     reasoning continuity; consolidate consecutive same-mode states.",
                    report.mode_stability
                ),
            });
        }

        critiques
    }

    /// 修订阶段: 基于批判意见生成修订后的响应。
    /// 无批判 → 原样返回 (revised=false); 有批判 → 追加修订说明。
    pub fn revise(&self, report: &ReflectionReport, original: &str) -> Revision {
        let critiques = self.critique(report);
        let overall_score = (report.coherence_score
            + report.efficiency_score
            + (1.0 - report.error_density)
            + report.mode_stability)
            / 4.0;

        if critiques.is_empty() {
            return Revision {
                critiques,
                revised_response: original.to_string(),
                revised: false,
                overall_score,
            };
        }

        let mut revised = String::with_capacity(original.len() + 256);
        revised.push_str(original);
        revised.push_str("\n\n[reflection-revision]");
        for c in &critiques {
            revised.push_str(&format!("\n- {:?} ({:.2}): {}", c.severity, c.score, c.message));
        }
        if let Some(suggestion) = report.suggestions.first() {
            revised.push_str(&format!("\n  suggestion: {}", suggestion));
        }

        Revision {
            critiques,
            revised_response: revised,
            revised: true,
            overall_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_report() -> ReflectionReport {
        ReflectionReport::new(0.9, 0.8, 0.05, 0.9, vec![], vec![], 0)
    }

    fn poor_report() -> ReflectionReport {
        ReflectionReport::new(
            0.2, 0.1, 0.7, 0.1,
            vec!["error cluster detected".to_string()],
            vec!["switch to conservative mode".to_string()],
            0,
        )
    }

    #[test]
    fn test_critique_healthy_empty() {
        let ex = ReflectionExecutor::new();
        let critiques = ex.critique(&healthy_report());
        assert!(critiques.is_empty(), "healthy report must yield no critiques");
    }

    #[test]
    fn test_critique_poor_has_high_severity() {
        let ex = ReflectionExecutor::new();
        let critiques = ex.critique(&poor_report());
        assert!(!critiques.is_empty(), "poor report must yield critiques");
        assert!(critiques.iter().any(|c| c.severity == Severity::High));
        assert!(critiques.iter().any(|c| c.dimension == "coherence"));
    }

    #[test]
    fn test_revise_healthy_unchanged() {
        let ex = ReflectionExecutor::new();
        let rev = ex.revise(&healthy_report(), "original answer");
        assert!(!rev.revised, "healthy report must not trigger revision");
        assert_eq!(rev.revised_response, "original answer");
    }

    #[test]
    fn test_revise_poor_appends_revision() {
        let ex = ReflectionExecutor::new();
        let rev = ex.revise(&poor_report(), "original answer");
        assert!(rev.revised, "poor report must trigger revision");
        assert!(rev.revised_response.contains("[reflection-revision]"));
        assert!(rev.revised_response.contains("coherence"));
        assert!(rev.overall_score < 0.5, "poor report overall score must be low");
    }

    #[test]
    fn test_revise_includes_suggestion() {
        let ex = ReflectionExecutor::new();
        let rev = ex.revise(&poor_report(), "original");
        assert!(rev.revised_response.contains("switch to conservative mode"));
    }
}