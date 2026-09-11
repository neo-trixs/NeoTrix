use std::collections::VecDeque;

/// Provider 指标快照
#[allow(dead_code)]
pub struct ProviderMetric {
    /// Provider 唯一标识
    pub provider_id: String,
    /// 响应延迟 (毫秒)
    pub latency_ms: u64,
    /// 每 token 成本
    pub cost_per_token: f64,
    /// 请求成功率 (0.0-1.0)
    pub success_rate: f64,
    /// 综合质量评分 (0.0-1.0)
    pub quality_score: f64,
    /// 采样时间戳 (UNIX 秒)
    pub timestamp: i64,
}

/// 漂移检测结果
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    /// 正常范围，无需干预
    Normal,
    /// 轻微漂移，建议继续监控
    MildDrift {
        /// 漂移指标名称
        metric: String,
        /// 偏离程度 (标准差倍数)
        deviation: f64,
    },
    /// 严重漂移，建议切换 provider
    SevereDrift {
        /// 漂移指标名称
        metric: String,
        /// 偏离程度 (标准差倍数)
        deviation: f64,
        /// 建议切换到的备选 provider
        suggested_alternative: String,
    },
}

/// 漂移检测器 — 基于滚动窗口监控 provider 质量漂移
///
/// 参考 arXiv 2608.25992 Drift-Aware 论文设计:
/// - 使用 Z-score 检测指标偏离历史均值的程度
/// - 双阈值机制: 轻微漂移 (告警) vs 严重漂移 (触发切换)
/// - 滚动窗口自动清理旧数据，避免内存膨胀
#[allow(dead_code)]
pub struct DriftDetector {
    /// 指标历史记录 (全 provider 共用)
    windows: VecDeque<ProviderMetric>,
    /// 单个 provider 的有效窗口大小
    window_size: usize,
    /// 轻微漂移阈值 (标准差倍数, 默认 2.0)
    drift_threshold: f64,
    /// 严重漂移阈值 (标准差倍数, 默认 3.0)
    severe_threshold: f64,
}

#[allow(dead_code)]
impl DriftDetector {
    /// 创建漂移检测器
    ///
    /// # 参数
    /// - `window_size`: 单个 provider 的有效滚动窗口大小
    /// - `drift_threshold`: 轻微漂移阈值 (标准差倍数)
    /// - `severe_threshold`: 严重漂移阈值 (标准差倍数)
    pub fn new(window_size: usize, drift_threshold: f64, severe_threshold: f64) -> Self {
        Self {
            windows: VecDeque::with_capacity(window_size * 10),
            window_size,
            drift_threshold,
            severe_threshold,
        }
    }

    /// 记录一条 provider 指标
    pub fn record(&mut self, metric: ProviderMetric) {
        self.windows.push_back(metric);
        // 保持总容量上限: 最多保留 10 倍窗口大小的记录
        while self.windows.len() > self.window_size * 10 {
            self.windows.pop_front();
        }
    }

    /// 检测指定 provider 的漂移状态
    ///
    /// 返回 `DriftStatus` 表示当前漂移程度。
    /// 样本不足 (<3) 时视为正常，避免误报。
    pub fn detect(&self, provider_id: &str) -> DriftStatus {
        let metrics: Vec<&ProviderMetric> = self
            .windows
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .collect();

        // 样本不足，无法做出可靠判断
        if metrics.len() < 3 {
            return DriftStatus::Normal;
        }

        // 取最近 window_size 条记录计算基线
        let start = metrics.len().saturating_sub(self.window_size);
        let recent = &metrics[start..];

        // 延迟基线
        let avg_latency =
            recent.iter().map(|m| m.latency_ms as f64).sum::<f64>() / recent.len() as f64;
        let var_latency = recent
            .iter()
            .map(|m| (m.latency_ms as f64 - avg_latency).powi(2))
            .sum::<f64>()
            / recent.len() as f64;
        let std_latency = var_latency.sqrt();

        // 质量基线
        let avg_quality = recent.iter().map(|m| m.quality_score).sum::<f64>() / recent.len() as f64;
        let var_quality = recent
            .iter()
            .map(|m| (m.quality_score - avg_quality).powi(2))
            .sum::<f64>()
            / recent.len() as f64;
        let std_quality = var_quality.sqrt();

        let latest = recent.last().unwrap();

        // Z-score 检测延迟漂移
        let latency_z = if std_latency > 0.0 {
            (latest.latency_ms as f64 - avg_latency) / std_latency
        } else {
            0.0
        };

        // Z-score 检测质量漂移 (负向偏离表示质量下降)
        let quality_z = if std_quality > 0.0 {
            (avg_quality - latest.quality_score) / std_quality
        } else {
            0.0
        };

        // 取最严重的漂移指标
        let (worst_metric, worst_z) = if latency_z >= quality_z {
            ("latency".to_string(), latency_z)
        } else {
            ("quality".to_string(), quality_z)
        };

        if worst_z > self.severe_threshold {
            DriftStatus::SevereDrift {
                metric: worst_metric,
                deviation: worst_z,
                suggested_alternative: "backup_provider".to_string(),
            }
        } else if worst_z > self.drift_threshold {
            DriftStatus::MildDrift {
                metric: worst_metric,
                deviation: worst_z,
            }
        } else {
            DriftStatus::Normal
        }
    }

    /// 获取所有 provider 的健康摘要
    ///
    /// 返回 `(provider_id, avg_quality, avg_latency_ms)` 元组列表
    pub fn health_summary(&self) -> Vec<(String, f64, f64)> {
        let mut summaries: Vec<(String, f64, f64)> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for metric in &self.windows {
            if seen.insert(metric.provider_id.clone()) {
                let metrics: Vec<&ProviderMetric> = self
                    .windows
                    .iter()
                    .filter(|m| m.provider_id == metric.provider_id)
                    .collect();
                let avg_quality =
                    metrics.iter().map(|m| m.quality_score).sum::<f64>() / metrics.len() as f64;
                let avg_latency =
                    metrics.iter().map(|m| m.latency_ms as f64).sum::<f64>() / metrics.len() as f64;
                summaries.push((metric.provider_id.clone(), avg_quality, avg_latency));
            }
        }

        summaries
    }

    /// 清理旧数据，将总记录数限制在 max_entries 以内
    pub fn prune(&mut self, max_entries: usize) {
        while self.windows.len() > max_entries {
            self.windows.pop_front();
        }
    }

    /// 获取指定 provider 的记录数量
    pub fn provider_count(&self, provider_id: &str) -> usize {
        self.windows
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .count()
    }

    /// 获取总记录数
    pub fn total_records(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metric(id: &str, latency: u64, quality: f64) -> ProviderMetric {
        ProviderMetric {
            provider_id: id.to_string(),
            latency_ms: latency,
            cost_per_token: 0.001,
            success_rate: 1.0,
            quality_score: quality,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_normal_when_insufficient_samples() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        det.record(make_metric("p1", 100, 0.9));
        det.record(make_metric("p1", 110, 0.88));
        assert_eq!(det.detect("p1"), DriftStatus::Normal);
    }

    #[test]
    fn test_no_drift_on_stable_metrics() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        for _ in 0..10 {
            det.record(make_metric("p1", 100, 0.9));
        }
        assert_eq!(det.detect("p1"), DriftStatus::Normal);
    }

    #[test]
    fn test_mild_latency_drift() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        // 正常基线
        for _ in 0..9 {
            det.record(make_metric("p1", 100, 0.9));
        }
        // 延迟飙升 (超过 2 标准差但不到 3)
        det.record(make_metric("p1", 200, 0.9));
        let status = det.detect("p1");
        assert!(
            matches!(status, DriftStatus::MildDrift { ref metric, .. } if metric == "latency"),
            "expected MildDrift on latency, got {:?}",
            status
        );
    }

    #[test]
    fn test_severe_latency_drift() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        // 正常基线
        for _ in 0..9 {
            det.record(make_metric("p1", 100, 0.9));
        }
        // 极端延迟飙升 (超过 3 标准差)
        det.record(make_metric("p1", 500, 0.9));
        let status = det.detect("p1");
        assert!(
            matches!(status, DriftStatus::SevereDrift { ref metric, .. } if metric == "latency"),
            "expected SevereDrift on latency, got {:?}",
            status
        );
    }

    #[test]
    fn test_quality_drift_detected() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        for _ in 0..9 {
            det.record(make_metric("p1", 100, 0.9));
        }
        // 质量骤降
        det.record(make_metric("p1", 100, 0.3));
        let status = det.detect("p1");
        assert!(
            matches!(
                status,
                DriftStatus::MildDrift { ref metric, .. } | DriftStatus::SevereDrift { ref metric, .. }
                    if metric == "quality"
            ),
            "expected drift on quality, got {:?}",
            status
        );
    }

    #[test]
    fn test_health_summary() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        det.record(make_metric("p1", 100, 0.9));
        det.record(make_metric("p2", 200, 0.7));
        let summary = det.health_summary();
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn test_prune() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        for _ in 0..50 {
            det.record(make_metric("p1", 100, 0.9));
        }
        assert_eq!(det.total_records(), 50);
        det.prune(20);
        assert_eq!(det.total_records(), 20);
    }

    #[test]
    fn test_provider_count() {
        let mut det = DriftDetector::new(10, 2.0, 3.0);
        det.record(make_metric("p1", 100, 0.9));
        det.record(make_metric("p1", 110, 0.88));
        det.record(make_metric("p2", 200, 0.7));
        assert_eq!(det.provider_count("p1"), 2);
        assert_eq!(det.provider_count("p2"), 1);
        assert_eq!(det.provider_count("p3"), 0);
    }

    #[test]
    fn test_unknown_provider_returns_normal() {
        let det = DriftDetector::new(10, 2.0, 3.0);
        assert_eq!(det.detect("nonexistent"), DriftStatus::Normal);
    }
}
