/// 进化信号
#[derive(Debug, Clone)]
pub enum EvolutionSignal {
    SourceHealth { source: String, healthy: bool },
    SearchQuality { source: String, success_rate: f64 },
    LatencyAlert { source: String, latency_ms: u64 },
}

/// 进化信号发射器
pub struct EvolutionBridge;

impl EvolutionBridge {
    /// 发射源健康度信号
    pub fn emit_source_health(source: &str, healthy: bool) {
        log::info!(
            "[evolution] source {} health: {}",
            source,
            if healthy { "UP" } else { "DOWN" }
        );
    }

    /// 发射搜索质量信号
    pub fn emit_search_quality(source: &str, success_rate: f64) {
        if success_rate < 0.5 {
            log::warn!(
                "[evolution] source {} low quality: {:.1}%",
                source,
                success_rate * 100.0
            );
        }
    }

    /// 发射延迟告警
    pub fn emit_latency_alert(source: &str, latency_ms: u64) {
        if latency_ms > 5000 {
            log::warn!(
                "[evolution] source {} high latency: {}ms",
                source,
                latency_ms
            );
        }
    }
}
