//! # PREDICTIVE-MAINTENANCE — 预测性维护 (healing 子模块)
//!
//! 基于时间序列分析的预测性维护管线:
//! - `anomaly.rs` — 异常检测: z-score + 移动平均
//! - `predictor.rs` — 趋势预测: 线性回归 + 指数平滑
//! - `maintenance.rs` — 维护调度: 问题 → 任务 → 优先级计划
//! - `trend.rs` — 趋势分析: 方向/斜率/置信度

#![forbid(unsafe_code)]

pub mod anomaly;
pub mod maintenance;
pub mod predictor;
pub mod trend;

pub use anomaly::{AnomalyDetector, AnomalyResult, AnomalySeverity};
pub use maintenance::{MaintenancePlan, MaintenanceScheduler, PredictedIssue, Urgency};
pub use predictor::Predictor;
pub use trend::{Trend, TrendAnalyzer, TrendDirection};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_re_exports() {
        let _detector = AnomalyDetector::new(10);
        let _predictor = Predictor::new();
        let _scheduler = MaintenanceScheduler::new();
        let _analyzer = TrendAnalyzer::new();
    }

    #[test]
    fn test_full_pipeline() {
        // Simulate a simple monitoring pipeline.
        let history: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 0.5).collect();

        // 1. Trend analysis.
        let analyzer = TrendAnalyzer::new();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Degrading);

        // 2. Anomaly detection on the last point.
        let detector = AnomalyDetector::new(20);
        // 2026-09-28 修正测试取值: 注释说「Within expected trend」, 但传的是 150.0,
        // 而序列是 100 + 0.5*i (i<30), 末值 114.5 ⇒ 150 偏离 z ≈ 9.7, 必然判异常,
        // 与注释自相矛盾。该序列的**下一个点恰好是 100 + 0.5*30 = 115.0** ——
        // 那才是「趋势内」的值, 也正是本测试想验的。150 应是遗留错值。
        let result = detector.detect(115.0, &history);
        assert!(!result.is_anomaly); // 正好是线性趋势的下一个点, z ≈ 1.76 < 2.0

        // 3. Prediction.
        let predictor = Predictor::new();
        let future = predictor.predict(&history, 5);
        assert_eq!(future.len(), 5);

        // 4. Forecast failure.
        let turns = predictor.forecast_failure(&history, 200.0);
        assert!(turns.is_some());
    }

    #[test]
    fn test_anomaly_then_schedule() {
        let detector = AnomalyDetector::new(10);
        let history: Vec<f64> = vec![10.0; 10];
        let result = detector.detect(100.0, &history);
        assert!(result.is_anomaly);

        // Convert anomaly to predicted issue and schedule
        let scheduler = MaintenanceScheduler::new();
        let issues = vec![PredictedIssue {
            component: "test_comp".into(),
            description: "anomaly detected".into(),
            severity: 0.95,
            turns_until_failure: Some(3),
        }];
        let plan = scheduler.schedule(issues);
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
    }

    #[test]
    fn test_trend_to_prediction_pipeline() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..20).map(|i| 50.0 + i as f64 * 2.0).collect();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Degrading);

        let predictor = Predictor::new();
        let future = predictor.predict(&history, 10);
        assert_eq!(future.len(), 10);
        // Each prediction should be higher than the last
        assert!(future.last() > future.first());
    }

    #[test]
    fn test_prediction_drives_maintenance_priority() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 5.0).collect();
        let turns = predictor.forecast_failure(&history, 200.0);
        assert!(turns.is_some());

        let scheduler = MaintenanceScheduler::new();
        let issues = vec![
            PredictedIssue {
                component: "urgent".into(),
                description: "will fail soon".into(),
                severity: 0.95,
                turns_until_failure: turns,
            },
            PredictedIssue {
                component: "later".into(),
                description: "might degrade".into(),
                severity: 0.4,
                turns_until_failure: Some(100),
            },
        ];
        let plan = scheduler.schedule(issues);
        assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
        assert_eq!(plan.tasks[1].urgency, Urgency::Medium);
    }
}
