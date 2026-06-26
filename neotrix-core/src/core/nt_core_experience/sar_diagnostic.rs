use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct SarReport {
    pub timestamp: u64,
    pub setting: String,
    pub analytical_finding: String,
    pub recommendation: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ConsciousnessVitals {
    pub coherence: f64,
    pub arousal: f64,
    pub valence: f64,
    pub cognitive_load: f64,
    pub negentropy_slope: f64,
    pub meta_accuracy: f64,
    pub health_score: f64,
    pub goal_drift: f64,
}

impl ConsciousnessVitals {
    pub fn default() -> Self {
        Self {
            coherence: 0.8,
            arousal: 0.5,
            valence: 0.5,
            cognitive_load: 0.3,
            negentropy_slope: 0.01,
            meta_accuracy: 0.8,
            health_score: 1.0,
            goal_drift: 0.0,
        }
    }
}

pub struct SarDiagnostic {
    history: VecDeque<SarReport>,
    max_history: usize,
    cycle: u64,
}

impl SarDiagnostic {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(50),
            max_history: 50,
            cycle: 0,
        }
    }

    pub fn diagnose(&mut self, vitals: ConsciousnessVitals) -> SarReport {
        self.cycle += 1;
        let report = self.build_report(vitals);
        self.history.push_back(report.clone());
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        report
    }

    fn build_report(&self, v: ConsciousnessVitals) -> SarReport {
        let setting = self.describe_setting(v);
        let finding = self.describe_finding(v);
        let (recommendation, confidence) = self.describe_recommendation(v);
        SarReport {
            timestamp: self.cycle,
            setting,
            analytical_finding: finding,
            recommendation,
            confidence,
        }
    }

    fn describe_setting(&self, v: ConsciousnessVitals) -> String {
        format!(
            "arousal={:.2}, coherence={:.2}, load={:.2}, health={:.2}, drift={:.2}",
            v.arousal, v.coherence, v.cognitive_load, v.health_score, v.goal_drift
        )
    }

    fn describe_finding(&self, v: ConsciousnessVitals) -> String {
        let mut signals = Vec::new();

        if v.coherence < 0.3 {
            signals.push("低相干性 — 认知碎片化");
        } else if v.coherence > 0.8 {
            signals.push("高相干性 — 认知整合良好");
        }

        if v.arousal < 0.2 {
            signals.push("低唤醒 — 可能陷入默认模式");
        } else if v.arousal > 0.9 {
            signals.push("高唤醒 — 认知负荷风险");
        }

        if v.cognitive_load > 0.8 {
            signals.push("高负荷 — 工作空间过载");
        }

        if v.negentropy_slope < -0.01 {
            signals.push("负熵下降 — 系统停滞或退化");
        } else if v.negentropy_slope > 0.05 {
            signals.push("负熵快速上升 — 高效学习期");
        }

        if v.meta_accuracy < 0.5 {
            signals.push("元精度不足 — 自评估偏差");
        }

        if v.health_score < 0.6 {
            signals.push("健康分偏低 — 子系统可能异常");
        }

        if v.goal_drift > 0.3 {
            signals.push("目标漂移 — 行为与意图偏离");
        }

        if signals.is_empty() {
            "稳态运行，无明显异常信号".to_string()
        } else {
            signals.join("; ")
        }
    }

    fn describe_recommendation(&self, v: ConsciousnessVitals) -> (String, f64) {
        let mut recommendations = Vec::new();
        let mut evidence_count = 0u32;

        if v.coherence < 0.3 {
            recommendations.push("触发 DMN 整合 (default_mode.tick)");
            evidence_count += 1;
        }
        if v.arousal < 0.2 {
            recommendations.push("好奇心动因注入 (curiosity_drive)");
            evidence_count += 1;
        } else if v.arousal > 0.9 {
            recommendations.push("认知负荷门控 (cognitive_load.gate)");
            evidence_count += 1;
        }
        if v.cognitive_load > 0.8 {
            recommendations.push("降低工作空间竞争, 优先睡眠 consolidation");
            evidence_count += 1;
        }
        if v.negentropy_slope < -0.01 {
            recommendations.push("触发探索管道 (exploration_orchestrate)");
            evidence_count += 1;
        }
        if v.meta_accuracy < 0.5 {
            recommendations.push("重新校准 EpistemicSelfModel");
            evidence_count += 1;
        }
        if v.health_score < 0.6 {
            recommendations.push("健康巡查 + 自适应修复 (health_patrol)");
            evidence_count += 1;
        }
        if v.goal_drift > 0.3 {
            recommendations.push("目标对齐检查 + goal_decomposer 重分解");
            evidence_count += 1;
        }

        if recommendations.is_empty() {
            ("继续当前周期, 无干预需要".to_string(), 0.95)
        } else {
            let confidence = (evidence_count as f64).min(3.0) / 3.0;
            (recommendations.join(" → "), confidence)
        }
    }

    pub fn recent_reports(&self, n: usize) -> Vec<&SarReport> {
        self.history.iter().rev().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnose_normal() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals::default();
        let report = d.diagnose(vitals);
        assert!(report.setting.contains("arousal"));
        assert!(!report.recommendation.is_empty());
    }

    #[test]
    fn test_diagnose_low_coherence() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            coherence: 0.2,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.analytical_finding.contains("碎片化"));
        assert!(report.recommendation.contains("DMN"));
    }

    #[test]
    fn test_diagnose_high_load() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            cognitive_load: 0.9,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.analytical_finding.contains("过载"));
        assert!(report.recommendation.contains("睡眠"));
    }

    #[test]
    fn test_diagnose_negentropy_drop() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            negentropy_slope: -0.05,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.analytical_finding.contains("退化"));
        assert!(report.recommendation.contains("探索"));
    }

    #[test]
    fn test_diagnose_goal_drift() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            goal_drift: 0.5,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.analytical_finding.contains("漂移"));
        assert!(report.recommendation.contains("goal_decomposer"));
    }

    #[test]
    fn test_recent_reports() {
        let mut d = SarDiagnostic::new();
        for _ in 0..5 {
            d.diagnose(ConsciousnessVitals::default());
        }
        assert_eq!(d.recent_reports(3).len(), 3);
    }

    #[test]
    fn test_history_bounded() {
        let mut d = SarDiagnostic::new();
        for _ in 0..100 {
            d.diagnose(ConsciousnessVitals::default());
        }
        assert_eq!(d.history.len(), 50);
    }

    #[test]
    fn test_high_arousal_recommendation() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            arousal: 0.95,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.recommendation.contains("cognitive_load"));
    }

    #[test]
    fn test_multi_signal_confidence() {
        let mut d = SarDiagnostic::new();
        let vitals = ConsciousnessVitals {
            coherence: 0.2,
            cognitive_load: 0.9,
            health_score: 0.4,
            ..ConsciousnessVitals::default()
        };
        let report = d.diagnose(vitals);
        assert!(report.confidence > 0.6);
    }
}
