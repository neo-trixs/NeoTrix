//! 茧性能追踪器
//!
//! 监控和追踪茧系统的性能指标，提供告警和优化建议。

use super::cocoons::CocoonMetrics;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// 茧性能追踪器
pub struct PerformanceTracker {
    pub metrics_history: VecDeque<CocoonMetrics>,
    pub alert_thresholds: AlertThresholds,
    pub recent_recalls: VecDeque<RecallEvent>,
}

/// 告警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub min_recall_accuracy: f64,
    pub max_memory_utilization: f64,
    pub min_relevance_score: f64,
    pub max_recall_history: usize,
}

/// 召回事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallEvent {
    pub success: bool,
    pub relevance: f64,
    pub timestamp: u64,
}

impl PerformanceTracker {
    /// 创建新的性能追踪器
    pub fn new() -> Self {
        Self {
            metrics_history: VecDeque::new(),
            alert_thresholds: AlertThresholds::default(),
            recent_recalls: VecDeque::new(),
        }
    }

    /// 记录召回事件
    pub fn record_recall(&mut self, success: bool, relevance: f64) {
        let event = RecallEvent {
            success,
            relevance,
            timestamp: timestamp_now(),
        };

        self.recent_recalls.push_back(event);

        // 保持历史在限制内
        while self.recent_recalls.len() > self.alert_thresholds.max_recall_history {
            self.recent_recalls.pop_front();
        }

        // 更新聚合指标
        self.update_metrics();
    }

    /// 检查是否需要告警
    pub fn should_alert(&self) -> Option<String> {
        if self.recent_recalls.is_empty() {
            return None;
        }

        let recent: Vec<&RecallEvent> = self.recent_recalls.iter().rev().take(20).collect();

        // 计算召回准确率
        let successful = recent.iter().filter(|e| e.success).count();
        let accuracy = successful as f64 / recent.len() as f64;

        if accuracy < self.alert_thresholds.min_recall_accuracy {
            return Some(format!(
                "Low recall accuracy: {:.2} (threshold: {:.2})",
                accuracy, self.alert_thresholds.min_recall_accuracy
            ));
        }

        // 计算平均相关性
        let avg_relevance: f64 =
            recent.iter().map(|e| e.relevance).sum::<f64>() / recent.len() as f64;

        if avg_relevance < self.alert_thresholds.min_relevance_score {
            return Some(format!(
                "Low average relevance: {:.2} (threshold: {:.2})",
                avg_relevance, self.alert_thresholds.min_relevance_score
            ));
        }

        // 检查记忆利用率
        if let Some(last_metrics) = self.metrics_history.back() {
            if last_metrics.memory_utilization > self.alert_thresholds.max_memory_utilization {
                return Some(format!(
                    "High memory utilization: {:.2} (threshold: {:.2})",
                    last_metrics.memory_utilization, self.alert_thresholds.max_memory_utilization
                ));
            }
        }

        None
    }

    /// 获取优化建议
    pub fn recommendations(&self) -> Vec<String> {
        let mut recs = Vec::new();

        if self.recent_recalls.is_empty() {
            return recs;
        }

        let recent: Vec<&RecallEvent> = self.recent_recalls.iter().rev().take(20).collect();

        // 召回准确率建议
        let successful = recent.iter().filter(|e| e.success).count();
        let accuracy = successful as f64 / recent.len() as f64;

        if accuracy < 0.5 {
            recs.push(
                "Consider reducing memory fragmentation by consolidating small cocoons".into(),
            );
        } else if accuracy < 0.7 {
            recs.push("Adjust retention policy to keep stronger memories".into());
        }

        // 相关性建议
        let avg_relevance: f64 =
            recent.iter().map(|e| e.relevance).sum::<f64>() / recent.len() as f64;

        if avg_relevance < 0.4 {
            recs.push("Review domain weights to improve recall relevance".into());
        }

        // 利用率建议
        if let Some(last_metrics) = self.metrics_history.back() {
            if last_metrics.memory_utilization > 0.9 {
                recs.push("Memory utilization is critical. Run consolidation immediately".into());
            } else if last_metrics.memory_utilization > 0.7 {
                recs.push("Memory utilization is high. Consider pruning weak memories".into());
            }
        }

        // 趋势建议
        if self.metrics_history.len() >= 2 {
            let current = self.metrics_history.back().unwrap();
            let previous = self.metrics_history.back().unwrap();

            if current.total_recall_attempts > previous.total_recall_attempts * 2 {
                recs.push(
                    "Recall attempts are increasing rapidly. Check for memory access patterns"
                        .into(),
                );
            }
        }

        recs
    }

    /// 获取性能摘要
    pub fn summary(&self) -> MetaMetricsPerformanceSummary {
        if self.recent_recalls.is_empty() {
            return MetaMetricsPerformanceSummary::default();
        }

        let recent: Vec<&RecallEvent> = self.recent_recalls.iter().rev().take(20).collect();

        let successful = recent.iter().filter(|e| e.success).count();
        let accuracy = successful as f64 / recent.len() as f64;

        let avg_relevance: f64 =
            recent.iter().map(|e| e.relevance).sum::<f64>() / recent.len() as f64;

        let memory_utilization = self
            .metrics_history
            .back()
            .map(|m| m.memory_utilization)
            .unwrap_or(0.0);

        MetaMetricsPerformanceSummary {
            recall_accuracy: accuracy,
            avg_relevance,
            memory_utilization,
            total_recalls: self.recent_recalls.len(),
            alerts: self.should_alert(),
        }
    }

    /// 更新聚合指标
    fn update_metrics(&mut self) {
        if self.recent_recalls.is_empty() {
            return;
        }

        let recent: Vec<&RecallEvent> = self.recent_recalls.iter().rev().take(50).collect();

        let successful = recent.iter().filter(|e| e.success).count();
        let avg_relevance: f64 =
            recent.iter().map(|e| e.relevance).sum::<f64>() / recent.len() as f64;

        // 简单的利用率估算（基于历史趋势）
        let utilization = if let Some(last) = self.metrics_history.back() {
            let base = last.memory_utilization;
            let delta = if successful > 0 { 0.01 } else { -0.01 };
            (base + delta).clamp(0.0, 1.0)
        } else {
            0.5 // 默认值
        };

        let metrics = CocoonMetrics {
            total_recall_attempts: self.recent_recalls.len() as u64,
            successful_recalls: successful as u64,
            avg_relevance_score: avg_relevance,
            memory_utilization: utilization,
        };

        self.metrics_history.push_back(metrics);

        // 保持历史在合理范围内
        while self.metrics_history.len() > 100 {
            self.metrics_history.pop_front();
        }
    }
}

/// 性能摘要
#[derive(Debug, Default)]
pub struct MetaMetricsPerformanceSummary {
    pub recall_accuracy: f64,
    pub avg_relevance: f64,
    pub memory_utilization: f64,
    pub total_recalls: usize,
    pub alerts: Option<String>,
}

/// 默认实现
impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_recall_accuracy: 0.6,
            max_memory_utilization: 0.8,
            min_relevance_score: 0.5,
            max_recall_history: 1000,
        }
    }
}

/// 时间戳生成
fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl std::fmt::Display for PerformanceTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = self.summary();
        writeln!(f, "=== Performance Tracker ===")?;
        writeln!(f, "Total recalls: {}", summary.total_recalls)?;
        writeln!(f, "Recall accuracy: {:.3}", summary.recall_accuracy)?;
        writeln!(f, "Average relevance: {:.3}", summary.avg_relevance)?;
        writeln!(f, "Memory utilization: {:.3}", summary.memory_utilization)?;
        if let Some(alert) = &summary.alerts {
            writeln!(f, "ALERT: {}", alert)?;
        }
        Ok(())
    }
}
