//! Behavior Monitor - 行为监控器
//!
//! 提取 resonator_network 精髓
//! 设计原则：多源信号共振检测

/// 行为信号
#[derive(Debug, Clone)]
pub struct BehaviorSignal {
    pub source: String,
    pub signal_type: SignalType,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    /// 输入频率异常
    InputFrequency,
    /// 输出模式异常
    OutputPattern,
    /// 资源使用异常
    ResourceUsage,
    /// 时间模式异常
    TemporalPattern,
}

/// 行为监控结果
#[derive(Debug, Clone)]
pub struct MonitorResult {
    pub signals: Vec<BehaviorSignal>,
    pub anomaly_score: f64,
    pub is_anomalous: bool,
    pub recommendations: Vec<String>,
}

/// 行为监控器
pub struct BehaviorMonitor {
    /// 历史信号窗口
    signal_window: Vec<BehaviorSignal>,
    /// 异常阈值
    anomaly_threshold: f64,
    /// 最大窗口大小
    max_window_size: usize,
}

impl BehaviorMonitor {
    pub fn new() -> Self {
        Self {
            signal_window: Vec::new(),
            anomaly_threshold: 0.7,
            max_window_size: 100,
        }
    }

    /// 记录信号
    pub fn record_signal(&mut self, signal: BehaviorSignal) {
        self.signal_window.push(signal);
        if self.signal_window.len() > self.max_window_size {
            self.signal_window.remove(0);
        }
    }

    /// 分析行为
    pub fn analyze(&self) -> MonitorResult {
        let signals = self.signal_window.clone();

        // 计算异常分数
        let anomaly_score = self.calculate_anomaly_score(&signals);

        // 生成建议
        let recommendations = self.generate_recommendations(anomaly_score);

        MonitorResult {
            signals,
            anomaly_score,
            is_anomalous: anomaly_score > self.anomaly_threshold,
            recommendations,
        }
    }

    /// 计算异常分数
    fn calculate_anomaly_score(&self, signals: &[BehaviorSignal]) -> f64 {
        if signals.is_empty() {
            return 0.0;
        }

        // 简化实现：基于信号多样性和频率
        let unique_sources: std::collections::HashSet<_> =
            signals.iter().map(|s| s.source.clone()).collect();
        let source_diversity = unique_sources.len() as f64 / 10.0; // 归一化

        let avg_value: f64 = signals.iter().map(|s| s.value).sum::<f64>() / signals.len() as f64;

        (source_diversity * 0.3 + avg_value * 0.7).min(1.0)
    }

    /// 生成建议
    fn generate_recommendations(&self, anomaly_score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if anomaly_score > 0.8 {
            recommendations.push("Consider rate limiting".to_string());
            recommendations.push("Enable enhanced monitoring".to_string());
        } else if anomaly_score > 0.5 {
            recommendations.push("Increase monitoring frequency".to_string());
        }

        recommendations
    }
}

impl Default for BehaviorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_empty() {
        let monitor = BehaviorMonitor::new();
        let result = monitor.analyze();
        assert!(!result.is_anomalous);
    }

    #[test]
    fn test_monitor_normal() {
        let mut monitor = BehaviorMonitor::new();
        for i in 0..5 {
            monitor.record_signal(BehaviorSignal {
                source: "test".to_string(),
                signal_type: SignalType::InputFrequency,
                value: 0.3,
                timestamp: i,
            });
        }
        let result = monitor.analyze();
        assert!(!result.is_anomalous);
    }
}
