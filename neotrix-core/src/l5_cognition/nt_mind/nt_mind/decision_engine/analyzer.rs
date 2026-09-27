//! Decision analyzer — produces Analysis from scored options
//!
//! R-P123: 按认知域拆分决策流程
//! R-P124: 配置集中管理，支持 Default trait

use serde::{Deserialize, Serialize};

use super::scorer::ScoredOption;

/// 判定 top 与 runner-up "几乎并列" 的绝对分差阈值(含边界)
/// 平局判定阈值 + 浮点容差 —— 0.50-0.49 的实差是 0.010000000000000009,
/// 不加容差会把真平局判成"有差距" (test_analyze_tied_options 实锤)。
const TIE_GAP_THRESHOLD: f64 = 0.01 + 1e-9;
/// 判定 Low risk 的最小相对领先比例: (top - second) / top
const LOW_RISK_GAP_RATIO: f64 = 0.4;
/// 判定 Medium risk 的最小相对领先比例: (top - second) / top
const MEDIUM_RISK_GAP_RATIO: f64 = 0.1;

/// Analysis result from evaluating scored options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// Recommended option identifier
    pub recommendation: String,
    /// Confidence in the recommendation (0.0..1.0)
    pub confidence: f64,
    /// Explanation of why this option was recommended
    pub rationale: String,
    /// Risk assessment of the recommended option
    pub risk_assessment: String,
}

/// Analyzes scored options to produce a recommendation
#[derive(Debug, Clone)]
pub struct DecisionAnalyzer;

impl Default for DecisionAnalyzer {
    fn default() -> Self {
        Self
    }
}

impl DecisionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze scored options and produce an Analysis
    pub fn analyze(&self, scored: &[ScoredOption]) -> Analysis {
        if scored.is_empty() {
            return Analysis {
                recommendation: String::new(),
                confidence: 0.0,
                rationale: "No options to analyze".to_string(),
                risk_assessment: "N/A".to_string(),
            };
        }

        let top = &scored[0];
        let confidence = self.compute_confidence(scored);
        let rationale = self.build_rationale(scored, confidence);
        let risk_assessment = self.assess_risk(scored);

        Analysis {
            recommendation: top.option_id.clone(),
            confidence,
            rationale,
            risk_assessment,
        }
    }

    /// Confidence based on gap between top and runner-up
    fn compute_confidence(&self, scored: &[ScoredOption]) -> f64 {
        if scored.len() == 1 {
            return 0.8; // single option: moderate confidence
        }

        let top = scored[0].weighted_score;
        let runner_up = scored[1].weighted_score;

        if top <= 0.0 {
            return 0.1;
        }

        let gap = top - runner_up;
        // Gap of 0.2+ maps to confidence ~0.9; gap of 0 maps to ~0.5
        let confidence = 0.5 + (gap / top).min(0.4) * 1.0;
        confidence.clamp(0.1, 0.95)
    }

    /// Build rationale string
    fn build_rationale(&self, scored: &[ScoredOption], confidence: f64) -> String {
        let top = &scored[0];
        if scored.len() == 1 {
            return format!(
                "Option '{}' is the only available choice (score: {:.3}).",
                top.option_id, top.weighted_score
            );
        }

        let runner_up = &scored[1];
        let gap = top.weighted_score - runner_up.weighted_score;

        if gap <= TIE_GAP_THRESHOLD {
            format!(
                "Options '{}' and '{}' are virtually tied ({:.3} vs {:.3}). Consider additional criteria.",
                top.option_id, runner_up.option_id, top.weighted_score, runner_up.weighted_score
            )
        } else {
            format!(
                "Option '{}' leads with score {:.3} (gap: {:.3} over '{}'). Confidence: {:.0}%.",
                top.option_id, top.weighted_score, gap, runner_up.option_id, confidence * 100.0
            )
        }
    }

    /// Assess risk based on score distribution
    fn assess_risk(&self, scored: &[ScoredOption]) -> String {
        if scored.len() <= 1 {
            return "Low risk: single option, no alternatives to compare.".to_string();
        }

        let top = scored[0].weighted_score;
        let second = scored[1].weighted_score;

        // 以 top 与 runner-up 的绝对分差为准(而非均值), 保证 2 选项下 Medium 档可达
        let gap = top - second;
        let low_cut = top * LOW_RISK_GAP_RATIO;
        let medium_cut = top * MEDIUM_RISK_GAP_RATIO;

        if gap >= low_cut {
            "Low risk: clear winner significantly outperforms alternatives.".to_string()
        } else if gap >= medium_cut {
            "Medium risk: winner has moderate advantage. Consider sensitivity analysis.".to_string()
        } else {
            "High risk: options are closely clustered. Recommendation may be sensitive to weight changes.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scored() -> Vec<ScoredOption> {
        vec![
            ScoredOption { option_id: "a".into(), weighted_score: 0.85, rank: 1 },
            ScoredOption { option_id: "b".into(), weighted_score: 0.60, rank: 2 },
            ScoredOption { option_id: "c".into(), weighted_score: 0.40, rank: 3 },
        ]
    }

    #[test]
    fn test_analyze_top() {
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&make_scored());
        assert_eq!(analysis.recommendation, "a");
        assert!(analysis.confidence > 0.5);
        assert!(!analysis.rationale.is_empty());
        assert!(!analysis.risk_assessment.is_empty());
    }

    #[test]
    fn test_analyze_empty() {
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&[]);
        assert_eq!(analysis.recommendation, "");
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_analyze_single_option() {
        let scored = vec![
            ScoredOption { option_id: "only".into(), weighted_score: 0.7, rank: 1 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert_eq!(analysis.recommendation, "only");
        assert_eq!(analysis.confidence, 0.8);
    }

    #[test]
    fn test_analyze_tied_options() {
        let scored = vec![
            ScoredOption { option_id: "x".into(), weighted_score: 0.50, rank: 1 },
            ScoredOption { option_id: "y".into(), weighted_score: 0.49, rank: 2 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert_eq!(analysis.recommendation, "x");
        assert!(analysis.rationale.contains("tied"));
    }

    #[test]
    fn test_risk_high_when_clustered() {
        let scored = vec![
            ScoredOption { option_id: "a".into(), weighted_score: 0.50, rank: 1 },
            ScoredOption { option_id: "b".into(), weighted_score: 0.48, rank: 2 },
            ScoredOption { option_id: "c".into(), weighted_score: 0.46, rank: 3 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert!(analysis.risk_assessment.contains("High risk"));
    }

    #[test]
    fn test_risk_low_when_clear_winner() {
        let scored = vec![
            ScoredOption { option_id: "a".into(), weighted_score: 0.90, rank: 1 },
            ScoredOption { option_id: "b".into(), weighted_score: 0.30, rank: 2 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert!(analysis.risk_assessment.contains("Low risk"));
    }

    #[test]
    fn test_risk_medium() {
        let scored = vec![
            ScoredOption { option_id: "a".into(), weighted_score: 0.60, rank: 1 },
            ScoredOption { option_id: "b".into(), weighted_score: 0.50, rank: 2 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert!(analysis.risk_assessment.contains("Medium risk"));
    }

    #[test]
    fn test_analysis_serialization() {
        let analysis = Analysis {
            recommendation: "opt_a".into(),
            confidence: 0.85,
            rationale: "Because it's best.".into(),
            risk_assessment: "Low risk.".into(),
        };
        let json = serde_json::to_string(&analysis).unwrap();
        let back: Analysis = serde_json::from_str(&json).unwrap();
        assert_eq!(back.recommendation, "opt_a");
        assert!((back.confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_analyzer_default() {
        let analyzer = DecisionAnalyzer::default();
        let analysis = analyzer.analyze(&[]);
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_confidence_gap_large() {
        let scored = vec![
            ScoredOption { option_id: "a".into(), weighted_score: 0.90, rank: 1 },
            ScoredOption { option_id: "b".into(), weighted_score: 0.50, rank: 2 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        // Large gap should yield high confidence
        assert!(analysis.confidence > 0.7);
    }

    #[test]
    fn test_rationale_tied_options() {
        let scored = vec![
            ScoredOption { option_id: "x".into(), weighted_score: 0.50, rank: 1 },
            ScoredOption { option_id: "y".into(), weighted_score: 0.495, rank: 2 },
        ];
        let analyzer = DecisionAnalyzer::new();
        let analysis = analyzer.analyze(&scored);
        assert!(analysis.rationale.contains("tied"));
    }
}
